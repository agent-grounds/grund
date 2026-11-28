//! Fixtures and exact message variants for §FS-check.checks.closed-citation-grounding.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

pub const CODE: &str = "closed-citation-grounding";

pub fn config(version: u32, grounding: &str, inherited: &str) -> String {
    if version == 1 {
        format!(
            "grund_config_version = 1\n[id]\nformat = \"{{kind}}-{{slug}}\"\n\
             named_sections = true\n[[kinds]]\nkind = \"FS\"\nfolder = \"specification\"\n\
             index = false\n[[kinds]]\nkind = \"ce\"\nfolder = \"ce\"\n\
             citable = false\n{grounding}\n[citations.ce]\ndefault = \"must-not\"\n\
             [scan]\ninclude = [\"specification\", \"ce\", \"src\"]\n{inherited}"
        )
    } else {
        format!(
            "grund_config_version = 2\nproject_name = \"p\"\n[schema.kinds.FS]\n\
             folders = [\"specification\"]\nindex = false\n[schema.kinds.ce]\n\
             folders = [\"ce\"]\ncitable = false\n[rules.citations.ce]\n\
             default = \"must-not\"\n{grounding}\n{inherited}"
        )
    }
}

pub fn row(version: u32) -> String {
    config(
        version,
        if version == 1 {
            "require_grounding = true\n"
        } else {
            "[rules.citations.ce.grounding]\nmust = \"file\"\n"
        },
        "",
    )
}

pub fn message(version: u32, source: &str, rung: &str, fallback: bool) -> String {
    let table = if version == 1 {
        "citations"
    } else {
        "rules.citations"
    };
    let setting = match (version, source) {
        (_, "flag") => "--require-grounding".into(),
        (1, "project") => "[reference] require_grounding = true".into(),
        (1, _) => "require_grounding = true on its [[kinds]] row".into(),
        (_, "project") => format!("[rules.citations.grounding] {rung}"),
        _ => format!("[rules.citations.ce.grounding] {rung}"),
    };
    let repair = if version == 1 {
        "set require_grounding = false on its [[kinds]] row".into()
    } else {
        format!(
            "{} [rules.citations.ce.grounding]",
            if source == "row" {
                "empty"
            } else {
                "write an empty"
            }
        )
    };
    let default = if fallback {
        table.to_string()
    } else {
        format!("{table}.ce")
    };
    format!(
        "grund.toml: row `ce`: {setting} asks for citations, but [{default}] \
        default = \"must-not\" permits no target; {repair} to disable grounding for this row, \
        or permit a target in [{table}.ce]"
    )
}

pub struct Tree(pub PathBuf);

impl Tree {
    pub fn new(name: &str, config: &str) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1460-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let tree = Self(root);
        tree.write("grund.toml", config);
        if config.contains("members = [\"member\"]") {
            tree.write(
                "member/grund.toml",
                "grund_config_version = 2\nproject_name = \"member\"\n",
            );
        }
        tree.write(
            "specification/FS-login.md",
            "# FS-login: Login\n\nUsers log in.\n",
        );
        tree.write("ce/A.java", "class A {}\n");
        fs::create_dir_all(tree.0.join("src")).unwrap();
        assert!(
            Command::new("git")
                .args(["init", "-q"])
                .current_dir(&tree.0)
                .status()
                .unwrap()
                .success()
        );
        let init = tree.run(&["init"]);
        assert_eq!(init.status.code(), Some(0), "{}", streams(&init));
        tree
    }

    pub fn write(&self, path: &str, text: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    pub fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .env("NO_COLOR", "1")
            .output()
            .unwrap()
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub fn streams(output: &Output) -> String {
    format!(
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

pub fn warnings(output: &Output) -> Vec<String> {
    String::from_utf8_lossy(&output.stderr)
        .lines()
        .filter(|line| line.contains("asks for citations") || line.contains(CODE))
        .map(str::to_owned)
        .collect()
}
