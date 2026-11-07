// A scratch project for the two-names tests (§FS-scan.identity.reading,
// §FS-scan.identity.findings): files, links, and the built `grund` run from its root.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The version 2 project the reading tests share: `txt` is Rust, and `fs` is FS's home.
pub const TXT_IS_RUST: &str = "grund_config_version = 2\nproject_name = \"names\"\n\
    [schema.sources]\nlanguages = [\"markdown\", \"rust\"]\n\
    [schema.sources.extensions]\ntxt = \"rust\"\n\
    [schema.kinds.FS]\nfolders = [\"fs\"]\nindex = false\n";

/// One finished run of the built binary.
pub struct Run {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl Run {
    /// This run's parsed NDJSON findings.
    pub fn findings(&self) -> Vec<serde_json::Value> {
        self.stdout
            .lines()
            .map(|line| serde_json::from_str(line).expect("a JSON finding"))
            .collect()
    }

    /// The `path:line` of every site of this run's one finding of `code`.
    pub fn sites(&self, code: &str) -> Vec<String> {
        let findings = self.findings();
        let found: Vec<_> = findings.iter().filter(|row| row["code"] == code).collect();
        assert_eq!(found.len(), 1, "one {code} finding in {}", self.stdout);
        let sites = found[0]["sites"].as_array().cloned().unwrap_or_default();
        sites
            .iter()
            .map(|site| format!("{}:{}", site["path"].as_str().unwrap(), site["line"]))
            .collect()
    }
}

/// A project on disk, removed on drop.
pub struct Tree {
    pub root: PathBuf,
}

impl Tree {
    pub fn new(case: &str, config: &str) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1274-{case}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create the project");
        let root = root.canonicalize().expect("canonical project root");
        let tree = Self { root };
        tree.put("grund.toml", config);
        tree
    }

    pub fn put(&self, path: &str, text: &str) {
        let path = self.root.join(path);
        fs::create_dir_all(path.parent().unwrap()).expect("create the parent");
        fs::write(path, text).expect("write the file");
    }

    /// `link -> target`, with `target` written as it is, relative to the link's directory.
    pub fn link(&self, link: &str, target: impl AsRef<Path>) {
        let link = self.root.join(link);
        fs::create_dir_all(link.parent().unwrap()).expect("create the parent");
        std::os::unix::fs::symlink(target, link).expect("create the link");
    }

    pub fn read(&self, path: &str) -> String {
        fs::read_to_string(self.root.join(path)).expect("read the file")
    }

    pub fn grund(&self, args: &[&str]) -> Run {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.root)
            .env("NO_COLOR", "1")
            .output()
            .expect("run grund");
        Run {
            code: output.status.code().expect("an exit code"),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
