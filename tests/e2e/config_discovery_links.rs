//! A path through a directory link retains its project's config (§FS-config.1.5).
//! Port of grund.280's reproducer: both versions, whole/above-link controls and the
//! linked directory, descendant directory and manifest. Distinct target-side configs
//! also pin the copied explain/migrate searches, without writing either config.
#![cfg(unix)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const V2: &str = "grund_config_version = 2\nproject_name = \"project\"\n\n[schema]\n\
                  slug_pattern = \"[a-z][a-z-]*\"\n\n[schema.kinds.E2E]\n\
                  folders = [\"e2e/cases\"]\n";
const V1: &str = "grund_config_version = 1\n\n[id]\nformat = \"{kind}-{slug}\"\n\n\
                  [[kinds]]\nkind = \"E2E\"\nfolder = \"tests/e2e\"\nindex = false\n";
const STRANGER: &str = "grund_config_version = 2\nproject_name = \"stranger\"\n";
const REFUSAL: &str =
    "place `e2e/cases` in `[schema.kinds.E2E] folders` links out of the project root";
const READ_ERROR: &str = "error: tests/e2e/b/expected.exit: stream did not contain valid UTF-8";
const V1_WARNING: &str =
    "warning: grund.toml is version 1; migrate it with grund config migrate --to 2";

struct Tree(PathBuf);

impl Tree {
    fn new(name: &str, config: &str, link: &str) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1326-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let tree = Self(root);
        for case in ["a", "b", "c"] {
            tree.write(&format!("outside/cases/{case}/expected.exit"), b"0\n");
        }
        tree.write("outside/cases/b/expected.exit", b"\xff\xfe\n");
        tree.write("project/grund.toml", config.as_bytes());
        let linked = tree.0.join("project").join(link);
        fs::create_dir_all(linked.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(tree.0.join("outside/cases"), linked).unwrap();
        let init = Command::new("git")
            .args(["init", "-q"])
            .current_dir(tree.0.join("project"))
            .output()
            .expect("initialize the report's project");
        assert!(init.status.success(), "{init:?}");
        tree
    }

    fn write(&self, relative: &str, bytes: &[u8]) {
        let path = self.0.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    fn run(&self, args: &[&str]) -> Run {
        let out = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(self.0.join("project"))
            .env("NO_COLOR", "1")
            .output()
            .expect("run grund");
        Run {
            code: out.status.code(),
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        }
    }

    fn check_scopes(&self, link: &str, code: i32, messages: &[&str]) {
        // §FS-check.1.3.6.3: the failure is at or under each of these scopes.
        for args in [
            vec!["check"],
            vec!["check", link.split('/').next().unwrap()],
        ] {
            let mut control = Vec::new();
            self.expect(&args, code, messages, &mut control);
            assert!(
                control.is_empty(),
                "control fixture failed:\n{}",
                control.join("\n")
            );
        }
        let mut failures = Vec::new();
        for path in [
            link.to_string(),
            format!("{link}/b"),
            format!("{link}/b/expected.exit"),
        ] {
            self.expect(&["check", &path], code, messages, &mut failures);
            let absolute = self.0.join("project").join(&path);
            self.expect(
                &["check", absolute.to_str().unwrap()],
                code,
                messages,
                &mut failures,
            );
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    fn expect(&self, args: &[&str], code: i32, messages: &[&str], failures: &mut Vec<String>) {
        let run = self.run(args);
        let text = format!("{}{}", run.stdout, run.stderr);
        if run.code != Some(code) || messages.iter().any(|m| !text.contains(m)) {
            failures.push(format!(
                "grund {}: expected exit {code} and {messages:?}; actual exit {:?}\n{text}",
                args.join(" "),
                run.code
            ));
        }
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

/// §FS-config.1.5: a v2 outward-home refusal is not replaced by defaults.
#[test]
fn v2_narrowed_checks_keep_the_projects_refusal() {
    Tree::new("v2", V2, "e2e/cases").check_scopes("e2e/cases", 2, &["grund.toml:8:", REFUSAL]);
}

/// §FS-config.1.5, §FS-check.1.3.6.3: v1 still reads the in-scope bad manifest.
#[test]
fn v1_narrowed_checks_keep_the_projects_read_failure() {
    Tree::new("v1", V1, "tests/e2e").check_scopes("tests/e2e", 2, &[READ_ERROR, V1_WARNING]);
}

/// §FS-config.1.5: a config above the target cannot replace the governing project.
#[test]
fn a_target_side_config_cannot_replace_the_projects_config() {
    let mut failures = Vec::new();
    for (name, config, link, messages) in [
        ("stranger-v2", V2, "e2e/cases", vec![REFUSAL]),
        ("stranger-v1", V1, "tests/e2e", vec![READ_ERROR, V1_WARNING]),
    ] {
        let tree = Tree::new(name, config, link);
        tree.write("outside/grund.toml", STRANGER.as_bytes());
        tree.expect(&["check"], 2, &messages, &mut failures);
        for path in [
            link.to_string(),
            format!("{link}/b"),
            format!("{link}/b/expected.exit"),
        ] {
            tree.expect(&["check", &path], 2, &messages, &mut failures);
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// §FS-config.1.5: walking commands share config selection and its refusal.
#[test]
fn cover_fmt_and_list_keep_the_projects_config() {
    let tree = Tree::new("commands", V2, "e2e/cases");
    tree.write("outside/grund.toml", STRANGER.as_bytes());
    let mut failures = Vec::new();
    for (whole, narrowed) in [
        (vec!["cover"], vec!["cover", "e2e/cases/b"]),
        (vec!["fmt", "--check"], vec!["fmt", "--check", "e2e/cases"]),
        (vec!["list"], vec!["list", "e2e/cases"]),
    ] {
        tree.expect(&whole, 2, &["grund.toml:8:", REFUSAL], &mut failures);
        tree.expect(&narrowed, 2, &["grund.toml:8:", REFUSAL], &mut failures);
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// §FS-config.1.5: explain's version probe searches the same lexical ancestors.
#[test]
fn explain_uses_the_lexical_projects_version() {
    let tree = Tree::new("explain", V1, "tests/e2e");
    tree.write("outside/grund.toml", STRANGER.as_bytes());
    tree.write("project/note.md", b"Note.\n");
    let mut failures = Vec::new();
    // §FS-config.1.4: ordinary, bare and dotted file starts remain equivalent.
    for path in [
        ".",
        "tests",
        "note.md",
        "./note.md",
        "tests/e2e",
        "tests/e2e/b",
        "tests/e2e/b/expected.exit",
    ] {
        tree.expect(
            &["config", "explain", "project_name", "--path", path],
            0,
            &["grund.toml is read as version 1."],
            &mut failures,
        );
    }
    let absolute = tree.0.join("project/note.md");
    tree.expect(
        &[
            "config",
            "explain",
            "project_name",
            "--path",
            absolute.to_str().unwrap(),
        ],
        0,
        &["grund.toml is read as version 1."],
        &mut failures,
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// §FS-config.1.5: migration previews the lexical config and writes neither file.
#[test]
fn migrate_locates_the_lexical_projects_config_without_writing() {
    let tree = Tree::new("migrate", STRANGER, "e2e/cases");
    let other = b"grund_config_version = 1\nproject_name = \"target-side\"\n\n\
                  [id]\nnamed_sections = true\n\n[scan]\ninclude = []\n";
    tree.write("outside/grund.toml", other);
    tree.write("project/note.md", b"Note.\n");
    let mut failures = Vec::new();
    for path in [
        ".",
        "e2e",
        "note.md",
        "./note.md",
        "e2e/cases",
        "e2e/cases/b",
        "e2e/cases/b/expected.exit",
    ] {
        tree.expect(
            &["config", "migrate", "--to", "2", path],
            0,
            &["is already version 2; nothing to migrate"],
            &mut failures,
        );
    }
    for (relative, expected) in [
        ("project/grund.toml", STRANGER.as_bytes()),
        ("outside/grund.toml", other.as_slice()),
    ] {
        assert_eq!(
            fs::read(tree.0.join(Path::new(relative))).unwrap(),
            expected
        );
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
