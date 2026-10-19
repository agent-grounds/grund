//! Isolated v2 repository for §FS-rules.3.3's per-target ceiling contracts.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

pub struct Repo(pub PathBuf);

impl Repo {
    pub fn new(sites: usize) -> Self {
        let root = std::env::temp_dir().join(format!(
            "grund-835-once-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        assert!(
            Command::new("git")
                .args(["init", "-q"])
                .arg(&root)
                .status()
                .unwrap()
                .success()
        );
        let repo = Self(root);
        repo.write(
            "grund.toml",
            r#"grund_config_version = 2
project_name = "once-contract"
[schema]
id_format = "{kind}-{slug}"
[schema.kinds.GOAL]
files = ["docs/goal.md"]
[schema.kinds.REQ]
files = ["docs/req.md"]
[schema.kinds.FS]
files = ["docs/fs.md"]
[schema.kinds.AR]
files = ["docs/ar.md"]
[schema.kinds.SYS]
files = ["docs/sys.md"]
[schema.kinds.RULE]
folders = ["docs/rules"]
index = false
form = "rule"
[presentation]
rules = "root"
"#,
        );
        for (path, id) in [
            ("docs/goal.md", "GOAL-rules"),
            ("docs/req.md", "REQ-target"),
            ("docs/fs.md", "FS-login"),
            ("docs/ar.md", "AR-target"),
        ] {
            repo.write(path, &format!("# {id}: Target\n\nA target.\n"));
        }
        repo.write("docs/goal.md", "# GOAL-rules: Rules\n\nSee \u{a7}FS-login, \u{a7}REQ-target, \u{a7}AR-target and \u{a7}SYS-overview.\n");
        repo.write("docs/fs.md", "# FS-login: Login\n\nSee \u{a7}GOAL-rules.\n");
        let citations = "See \u{a7}AR-target.\n".repeat(sites);
        repo.write(
            "docs/sys.md",
            &format!("# SYS-overview: Overview\n\n## components: Components\n\n{citations}"),
        );
        repo
    }

    pub fn write(&self, path: &str, contents: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    pub fn rule(&self, title: &str) {
        self.write(
            "docs/rules/once.md",
            &format!("# RULE-once: {title}\n\nConstrain repetition for \u{a7}GOAL-rules.\n"),
        );
        self.write("docs/goal.md", "# GOAL-rules: Rules\n\nSee \u{a7}FS-login, \u{a7}REQ-target, \u{a7}AR-target, \u{a7}SYS-overview and \u{a7}RULE-once.\n");
    }

    pub fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .output()
            .unwrap()
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).unwrap()
}

pub fn check(output: Output, exit: i32, stdout: &str, stderr: &str, label: &str) -> Option<String> {
    (output.status.code() != Some(exit)
        || text(&output.stdout) != stdout
        || text(&output.stderr) != stderr)
        .then(|| format!(
            "{label}: exit {:?}, stdout {:?}, stderr {:?}\nexpected: exit {exit}, stdout {stdout:?}, stderr {stderr:?}",
            output.status.code(), text(&output.stdout), text(&output.stderr)
        ))
}

pub fn assert_clean(wrong: Vec<String>) {
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
