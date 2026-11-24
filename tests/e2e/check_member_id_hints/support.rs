//! Fixture and CLI assertions for §FS-check.3.1.5.
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

pub const ID: &str = "AR-005-invariant-monitor";
pub const ROOT: &str = "docs/ar/AR-001-root.md";
pub const MARK: &str = "\u{a7}";

pub struct Tree(pub PathBuf);

impl Tree {
    pub fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let tree = Self(std::env::temp_dir().join(format!(
            "grund-1489-hints-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        )));
        fs::create_dir(&tree.0).unwrap();
        assert!(
            Command::new("git")
                .args(["init", "-q"])
                .arg(&tree.0)
                .status()
                .unwrap()
                .success()
        );
        tree.project("", false);
        tree.project("worldsim", false);
        tree.members(&["worldsim"]);
        tree.write(ROOT, &format!("# AR-001-root: Root\n\nThe root design builds on the monitor ({MARK}{ID}).\nControl, a same-project near miss: ({MARK}AR-001-roo).\n"));
        tree.write(
            "docs/ar/README.md",
            &format!("# AR\n\n- [{MARK}AR-001-root](AR-001-root.md#ar-001-root-root)\n"),
        );
        tree.declare("worldsim", ID);
        tree.write(
            "worldsim/docs/ar/README.md",
            &format!(
                "# AR\n\n- [{MARK}{ID}]({ID}.md#ar-005-invariant-monitor-invariant-monitor)\n"
            ),
        );
        tree
    }

    pub fn write(&self, path: &str, body: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }

    pub fn edit(&self, path: &str, from: &str, to: &str) {
        let text = fs::read_to_string(self.0.join(path)).unwrap();
        assert!(text.contains(from), "missing fixture edit {from:?}");
        self.write(path, &text.replace(from, to));
    }

    pub fn project(&self, alias: &str, numbered: bool) {
        let format = if numbered {
            "{kind}-{number}-{slug}"
        } else {
            "{kind}-{slug}"
        };
        self.write(&format!("{alias}/grund.toml").trim_start_matches('/'), &format!(
            "grund_config_version = 2\n\n[schema]\nid_format = \"{format}\"\nslug_pattern = \"[a-z0-9][a-z0-9-]*\"\n\n[schema.kinds.AR]\nfolders = [\"docs/ar\"]\n"
        ));
        fs::create_dir_all(self.0.join(alias).join("docs/ar")).unwrap();
    }

    pub fn members(&self, aliases: &[&str]) {
        let path = self.0.join("grund.toml");
        let config = fs::read_to_string(&path).unwrap();
        let config = config.split("\n[workspace]").next().unwrap();
        fs::write(
            path,
            format!("{config}\n[workspace]\nmembers = {aliases:?}\n"),
        )
        .unwrap();
    }

    pub fn declare(&self, alias: &str, id: &str) {
        self.write(
            &format!("{alias}/docs/ar/{id}.md").trim_start_matches('/'),
            &format!("# {id}: Invariant monitor\n\nThe monitor.\n"),
        );
    }

    pub fn citation(&self, token: &str) {
        self.write(ROOT, &format!("# AR-001-root: Root\n\n{token}\n"));
    }

    pub fn run(&self, dir: &str, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(self.0.join(dir))
            .env("NO_COLOR", "1")
            .output()
            .unwrap()
    }

    pub fn check(&self, dir: &str, path: Option<&str>) -> Vec<Value> {
        let mut args = vec!["check", "--format", "json"];
        args.extend(path);
        let run = self.run(dir, &args);
        assert_eq!(run.status.code(), Some(1), "{run:?}");
        // A v1 fixture also emits its existing config-version caution.
        for line in stderr(&run).lines() {
            let warning: Value = serde_json::from_str(line).unwrap();
            assert_eq!(warning["code"], "config-version-1", "{run:?}");
            assert_eq!(warning["severity"], "warning");
        }
        stdout(&run)
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

pub fn stdout(run: &Output) -> &str {
    std::str::from_utf8(&run.stdout).unwrap()
}
pub fn stderr(run: &Output) -> &str {
    std::str::from_utf8(&run.stderr).unwrap()
}

pub fn at<'a>(rows: &'a [Value], path: &str, line: u64) -> &'a Value {
    let rows: Vec<_> = rows
        .iter()
        .filter(|row| row["path"] == path && row["line"] == line)
        .collect();
    assert_eq!(rows.len(), 1, "one finding at {path}:{line}: {rows:?}");
    rows[0]
}

pub fn dangling(row: &Value, expected: &str) {
    assert_eq!(row["code"], "dangling", "{row}");
    assert_eq!(row["severity"], "error", "{row}");
    assert_eq!(row["message"], expected, "{row}");
}

pub fn hinted(id: &str, candidate: &str) -> String {
    format!("unknown reference {id}; did you mean {candidate}?")
}
