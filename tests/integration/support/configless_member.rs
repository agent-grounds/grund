//! Shared input for reader-selection contracts (§AR-config.1.1, §FS-workspace.2).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

pub const ROOT: &str = "grund_config_version = 2\nproject_name = \"root\"\n\n\
    [workspace]\nmembers = [\"apps/*\"]\ninclude_root = true\n";
pub const FORMAT: &str = "{kind}-{number}-{slug}";
pub const BANNER: &str = "No grund.toml here. grund init writes version 2; until a file exists, check reads version 1's defaults.";

pub struct Tree(pub PathBuf);

impl Tree {
    pub fn empty() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "grund-1446-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self(root)
    }

    /// The filed grund.310 fixture, without giving nocfg a config of its own.
    pub fn workspace() -> Self {
        let tree = Self::empty();
        tree.write("grund.toml", ROOT);
        tree.write(
            "apps/nocfg/docs/functional-spec/login.md",
            "# FS-login: Login\n\nLogin.\n",
        );
        // A fixture citation, built at runtime so the repository checker sees no live ID.
        tree.write(
            "apps/nocfg/docs/functional-spec/README.md",
            &format!(
                "# Functional spec\n\n- [{}FS-login](login.md#fs-login-login)\n",
                '§'
            ),
        );
        tree
    }

    pub fn write(&self, relative: &str, text: &str) {
        let path = self.0.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    pub fn scopes(&self) -> Vec<PathBuf> {
        ["apps/nocfg", "apps/nocfg/docs/functional-spec"]
            .iter()
            .map(|p| self.0.join(p))
            .collect()
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

pub struct Run {
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    command: String,
}

pub fn run(cwd: &Path, args: &[&str]) -> Run {
    let out = Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(cwd)
        .env("NO_COLOR", "1")
        .output()
        .unwrap();
    Run {
        code: out.status.code(),
        stdout: String::from_utf8(out.stdout).unwrap(),
        stderr: String::from_utf8(out.stderr).unwrap(),
        command: format!("cwd={}; grund {}", cwd.display(), args.join(" ")),
    }
}

impl Run {
    pub fn expect(&self, code: i32, needles: &[&str], failures: &mut Vec<String>) {
        if self.code != Some(code) || needles.iter().any(|s| !self.stdout.contains(s)) {
            // Keep selection evidence readable even when show prints every default.
            let excerpt = self.stdout.lines().take(30).collect::<Vec<_>>().join("\n");
            failures.push(format!(
                "{}: expected exit {code}, stdout containing {needles:?}; actual exit {:?}\nstdout (first 30 lines):\n{excerpt}\nstderr:\n{}",
                self.command, self.code, self.stderr
            ));
        }
    }
}
