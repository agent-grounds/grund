//! Fixture for the numeric source-spelling contract (§FS-config.3.2.8).

use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

pub const FIRST: &str =
    "# ADR-0001: Use Postgres\n\nThe first decision.\n\n## 1. Why\n\nTransactions.\n";
pub const SECOND: &str = "# ADR-0002: Use Redis\n\nThe second decision.\n";
pub const NOTES: &str = "**Notes**\n\nSee \u{a7}ADR-0001 and \u{a7}ADR-0001.1 and \u{a7}ADR-0002, not \u{a7}ADR-0007.\nEquivalent: \u{a7}ADR-1 and \u{a7}ADR-002.\n";

pub struct Repo {
    pub root: PathBuf,
}

impl Repo {
    pub fn new(version: u8, per_kind: bool) -> Self {
        let root = std::env::temp_dir().join(format!(
            "grund-819-numeric-v{version}-{per_kind}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).expect("create isolated numeric spelling fixture");
        let repo = Self { root };
        let numeric = "{kind}-{number}";
        let default = if per_kind { "{kind}-{slug}" } else { numeric };
        let config = if version == 2 {
            let override_format = if per_kind {
                format!("id_format = {numeric:?}\n")
            } else {
                String::new()
            };
            format!(
                "grund_config_version = 2\nproject_name = \"numeric-spelling\"\n\
                 [schema]\nid_format = {default:?}\n\
                 [schema.sources]\nlanguages = [\"markdown\"]\nrespect_gitignore = false\n\
                 [schema.kinds.ADR]\nfolders = [\"docs/adr\"]\nindex = false\n\
                 {override_format}[schema.kinds.code]\nfolders = [\"docs\"]\n"
            )
        } else {
            let override_format = if per_kind {
                format!("format = {numeric:?}\n")
            } else {
                String::new()
            };
            format!(
                "grund_config_version = 1\nproject_name = \"numeric-spelling\"\n\
                 [reference]\nstrict = true\nrequire_grounding = false\n\
                 [id]\nformat = {default:?}\n\
                 [[kinds]]\nkind = \"ADR\"\nfolder = \"docs/adr\"\nindex = false\n\
                 {override_format}[scan]\ninclude = [\"docs\"]\nextensions = [\"md\"]\n\
                 respect_gitignore = false\n[fmt.cross_refs]\nenabled = true\n"
            )
        };
        repo.write("grund.toml", &config);
        repo.write("docs/adr/ADR-0001.md", FIRST);
        repo.write("docs/adr/ADR-0002.md", SECOND);
        repo.write("docs/notes.md", NOTES);
        repo
    }

    pub fn write(&self, path: &str, body: &str) {
        let path = self.root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }

    pub fn read(&self, path: &str) -> String {
        fs::read_to_string(self.root.join(path)).unwrap()
    }

    pub fn run(&self, args: &[&str], code: i32) -> String {
        let output: Output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.root)
            .output()
            .expect("run grund");
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert_eq!(
            output.status.code(),
            Some(code),
            "{args:?}: fixture/command failure\nstdout: {stdout}\nstderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        stdout
    }

    pub fn json(&self, args: &[&str], code: i32) -> Vec<Value> {
        self.run(args, code)
            .lines()
            .map(|line| serde_json::from_str(line).expect("JSON record"))
            .collect()
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

pub fn contains(failures: &mut Vec<String>, actual: &str, expected: &str, surface: &str) {
    if !actual.contains(expected) {
        failures.push(format!("{surface}: expected {expected:?}; got {actual:?}"));
    }
}

pub fn equal(failures: &mut Vec<String>, actual: &Value, expected: &str, surface: &str) {
    if actual != expected {
        failures.push(format!("{surface}: expected {expected:?}; got {actual}"));
    }
}
