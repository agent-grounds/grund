//! `check` names no `grund fetch` command for a key that command has nothing to run for, and
//! gives the repair instead (§FS-check.3.1.3.2). The first cases are the project #1322 reports.
//! Each message is asserted whole, so a load refusal or a vanished finding cannot pass as the
//! change. Each tree is built in the temp directory, because a file named `" "` cannot be a
//! committed fixture on every platform.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const V1_TAIL: &str = "[[kinds]] `AR` fetch names no executable; name the program, or if this \
                       kind is not fetched remove the key and any `resolve`, and its missing IDs \
                       are then errors";
const FETCH_TAIL: &str = "run grund fetch AR-002-other";

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Tree(PathBuf);

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

impl Run {
    fn shown(&self) -> String {
        format!(
            "exit {}\nstdout:\n{}\nstderr:\n{}",
            self.code, self.stdout, self.stderr
        )
    }

    /// Whether `line` is one whole line of the report: the finding, present and exact.
    fn has_line(&self, line: &str) {
        assert!(
            self.stdout
                .lines()
                .chain(self.stderr.lines())
                .any(|l| l == line),
            "expected the line\n{line}\nin\n{}",
            self.shown()
        );
    }
}

impl Tree {
    fn new() -> Self {
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("grund-1322-{}-{n}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create the tree");
        Self(root)
    }

    fn write(&self, relative: &str, content: &str) {
        let path = self.0.join(relative);
        fs::create_dir_all(path.parent().expect("a parent")).expect("create the directory");
        fs::write(path, content).expect("write the file");
    }

    fn grund(&self, dir: &str, args: &[&str]) -> Run {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(self.0.join(dir))
            .env("NO_COLOR", "1")
            .output()
            .expect("run grund");
        Run {
            code: output.status.code().expect("exited with a status"),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }

    /// The report's version 1 project at `dir`, its home citing `citation`.
    fn v1(&self, dir: &str, value: &str, resolve: &str, citation: &str) {
        self.write(
            &format!("{dir}/grund.toml"),
            &format!(
                "project_name = \"g1321\"\n\n[[kinds]]\nkind = \"AR\"\nfile = \"docs/arch.md\"\n\
                 fetch = \"{value}\"\nresolve = \"{resolve}\"\n\n[id]\nnamed_sections = true\n\n\
                 [scan]\ninclude = [\"docs\"]\n"
            ),
        );
        self.write(
            &format!("{dir}/docs/arch.md"),
            &format!(
                "# Architecture\n\n## AR-001-power: Power\n\nHand-written.\n\nSee §{citation}.\n"
            ),
        );
    }
}

fn should(tree: &Tree, tail: &str) -> Run {
    let run = tree.grund(".", &["check"]);
    assert_eq!(run.code, 0, "{}", run.shown());
    run.has_line(&format!(
        "docs/arch.md:7: warning: no snapshot for AR-002-other in docs/arch.md — {tail}"
    ));
    run
}

fn finding_json(run: &Run) -> serde_json::Value {
    let report: serde_json::Value = serde_json::from_str(&run.stdout).expect("JSON report");
    report["findings"]
        .as_array()
        .expect("findings")
        .iter()
        .find(|f| {
            f["message"]
                .as_str()
                .is_some_and(|m| m.contains("AR-002-other"))
        })
        .unwrap_or_else(|| panic!("no finding of AR-002-other: {}", run.shown()))
        .clone()
}

/// §FS-check.3.1.3.2: the report's two files, whose `should` finding gives the repair.
#[test]
#[ignore = "red until #1322 check gives the repair; implement removes this"]
fn a_blank_v1_key_under_should_gives_the_repair() {
    for value in ["", " "] {
        let tree = Tree::new();
        tree.v1(".", value, "should", "AR-002-other");
        let run = should(&tree, V1_TAIL);
        assert!(!run.stdout.contains("run grund fetch"), "{}", run.shown());
        assert!(!run.stderr.contains("run grund fetch"), "{}", run.shown());

        let run = tree.grund(".", &["check", "--format=json"]);
        assert_eq!(run.code, 0, "{}", run.shown());
        let found = finding_json(&run);
        assert_eq!(found["code"], "missing-snapshot", "{}", run.shown());
        assert_eq!(found["severity"], "warning", "{}", run.shown());
        assert_eq!(
            found["message"],
            format!("no snapshot for AR-002-other in docs/arch.md — {V1_TAIL}")
        );
    }
}

/// §FS-check.3.1.3.2: under `must` the same tail follows the `dangling` base, and exits 1.
#[test]
#[ignore = "red until #1322 check gives the repair; implement removes this"]
fn a_blank_v1_key_under_must_gives_the_repair() {
    for value in ["", " "] {
        let tree = Tree::new();
        tree.v1(".", value, "must", "AR-002-other");
        let run = tree.grund(".", &["check"]);
        assert_eq!(run.code, 1, "{}", run.shown());
        run.has_line(&format!(
            "docs/arch.md:7: error: unknown reference AR-002-other; no snapshot in docs/arch.md \
             — {V1_TAIL}"
        ));
        assert!(!run.stdout.contains("run grund fetch"), "{}", run.shown());

        let run = tree.grund(".", &["check", "--format=json"]);
        assert_eq!(run.code, 1, "{}", run.shown());
        let found = finding_json(&run);
        assert_eq!(found["code"], "dangling", "{}", run.shown());
        assert_eq!(found["severity"], "error", "{}", run.shown());
    }
}

/// §FS-check.3.1.3.2: a file named `" "` at the root is still run, so the command stays.
#[test]
#[cfg(unix)]
fn a_whitespace_value_naming_a_file_keeps_the_command() {
    let tree = Tree::new();
    tree.v1(".", " ", "should", "AR-002-other");
    tree.write(" ", "#!/bin/sh\nexit 0\n");
    should(&tree, FETCH_TAIL);
}

/// §FS-check.3.1.3.2: a named program and a nonblank path nothing stands at keep the command.
#[test]
fn a_named_value_keeps_the_command() {
    for value in ["scripts/fetch", "scripts/absent"] {
        let tree = Tree::new();
        tree.v1(".", value, "should", "AR-002-other");
        tree.write("scripts/fetch", "#!/bin/sh\nexit 0\n");
        should(&tree, FETCH_TAIL);
    }
}

/// §FS-check.3.1.3.2: a near-ID hint still takes the tail's place on a blank key.
#[test]
fn a_near_id_hint_keeps_its_place() {
    let tree = Tree::new();
    tree.v1(".", "", "should", "AR-001-powe");
    let run = tree.grund(".", &["check"]);
    assert_eq!(run.code, 0, "{}", run.shown());
    run.has_line(
        "docs/arch.md:7: warning: no snapshot for AR-001-powe in docs/arch.md; did you mean \
         AR-001-power?",
    );
}

/// §FS-check.3.1.3.2: a home `fetch` refuses keeps its own tail on a blank key.
#[test]
fn a_refused_home_keeps_its_tail() {
    let tree = Tree::new();
    tree.write(
        "grund.toml",
        "grund_config_version = 2\nproject_name = \"g1322\"\n\n[schema.kinds.CONST]\n\
         files = [\"values/limits.json\"]\nform = \"value\"\nfetch = \"\"\n\n\
         [rules.resolution]\nCONST = \"warn\"\n",
    );
    tree.write(
        "values/limits.json",
        "{\n  \"CONST-room-limit\": [40, \"rooms\"]\n}\n",
    );
    tree.write("README.md", "See §CONST-zebra-other.\n");
    let run = tree.grund(".", &["check"]);
    assert_eq!(run.code, 0, "{}", run.shown());
    run.has_line(
        "README.md:1: warning: no snapshot for CONST-zebra-other in values/limits.json — declare \
         it in that value source",
    );
}

/// A version 2 citer of `lb/AR-002-other`, whose own `AR` key is `citer_value`.
fn citer(tree: &Tree, citer_value: &str) {
    tree.write(
        "consumer/grund.toml",
        &format!(
            "grund_config_version = 2\nproject_name = \"consumer\"\n\n[schema.kinds.AR]\n\
             files = [\"docs/arch.md\"]\nfetch = \"{citer_value}\"\n\n\
             [workspace.imports.lb]\npath = \"../lib\"\nproject = \"g1321\"\n"
        ),
    );
    tree.write("consumer/docs/arch.md", "# Architecture\n");
    tree.write("consumer/README.md", "See §lb/AR-002-other.\n");
}

/// §FS-imports.writes.answer: the target's blank key decides, whatever the citer holds, and
/// `grund fetch` of the ID keeps its read-only refusal.
#[test]
fn a_blank_key_in_the_target_gives_the_repair_through_an_import() {
    let tree = Tree::new();
    tree.v1("lib", " ", "should", "AR-001-power");
    citer(&tree, "scripts/fetch");
    tree.write("consumer/scripts/fetch", "#!/bin/sh\nexit 0\n");
    #[cfg(unix)]
    tree.write("consumer/ ", "#!/bin/sh\nexit 0\n");
    let run = tree.grund("consumer", &["check"]);
    assert_eq!(run.code, 0, "{}", run.shown());
    run.has_line(&format!(
        "README.md:1: warning: no snapshot for lb/AR-002-other in ../lib/docs/arch.md — {V1_TAIL}"
    ));
    let refused = tree.grund("consumer", &["fetch", "lb/AR-002-other"]);
    assert_eq!(refused.code, 2, "{}", refused.shown());
    assert!(
        refused.stderr.contains(
            "`lb/AR-002-other` is in import `lb`, read-only here — run grund fetch AR-002-other \
             in ../lib"
        ),
        "{}",
        refused.shown()
    );
}

/// §FS-check.3.1.3.2: the target's working key keeps the command, whatever the citer holds.
#[test]
#[cfg(unix)]
fn a_runnable_key_in_the_target_keeps_the_command_through_an_import() {
    let tree = Tree::new();
    tree.v1("lib", " ", "should", "AR-001-power");
    tree.write("lib/ ", "#!/bin/sh\nexit 0\n");
    citer(&tree, "");
    let run = tree.grund("consumer", &["check"]);
    assert_eq!(run.code, 0, "{}", run.shown());
    run.has_line(
        "README.md:1: warning: no snapshot for lb/AR-002-other in ../lib/docs/arch.md — run grund \
         fetch AR-002-other in ../lib",
    );
}
