//! Black-box: every `missing-section` finding names its fix, whatever the section's spelling
//! (§FS-errors.3.11). A numeric path, an unmarked version 1 citation, a citation outside the
//! default scope under `--full` (§FS-check.3.14) and a citation the section lock knows moved
//! (§FS-section-lock.checks.vanished) end in the same remedy the marked named form already
//! carries (§FS-errors.5.5.1). Each form keeps the message it shipped with as a verbatim prefix,
//! so `missing section <coordinate>` gains `; <fix>` and the named form does not move.
//! Issue #1436 is the report: the tree of [`TICKET`] printed `missing section FS-check.9.9` and
//! stopped.
//!
//! They are tests rather than goldens because every one but the controls was pinned failing
//! before #1436 appended the remedy, and a golden has no skip. The trees are built in the temp
//! directory, so run them with a `TMPDIR` that no git tree or grund config encloses.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

/// The remedy of the `missing-section` row under the default marker, as a message carries it.
const FIX: &str = "; write <\u{a7}> before it to show the shape without citing it";

/// The `map` case the marked named form gives after the escape, for `FS-check` (§FS-errors.3.11).
const MAP: &str = "; run `grund FS-check --toc` to print the lead with the section map";

/// The report's tree, a version 2 project with named sections on: `FS-check` declares sections
/// `2` and `2.1`, and one source line cites a numeric section and a named one it does not have.
const TICKET: &[(&str, &str)] = &[
    (
        "grund.toml",
        "grund_config_version = 2\n\n[schema.kinds.FS]\nfolders = [\"docs/functional-spec\"]\n\
         index = false\n",
    ),
    (
        "docs/functional-spec/FS-check.md",
        "# FS-check: Check\n\nLead.\n\n## 2. Two\n\nBody.\n\n### 2.1. Two one\n\nBody.\n",
    ),
    (
        "src/main.rs",
        "// \u{a7}FS-check.9.9 and \u{a7}FS-check.2.goals\nfn main() {}\n",
    ),
];

/// A version 1 project in optional mode, where a bare token is a citation: `FS-002-session`
/// cites a section `FS-001-login` has and one it does not, neither with a marker.
fn unmarked(missing: &str) -> Vec<(&'static str, String)> {
    vec![
        (
            "grund.toml",
            "grund_config_version = 1\n\n[reference]\nstrict = false\n".to_string(),
        ),
        (
            "docs/functional-spec/README.md",
            "# FS index\n\nEvery `FS` declaration in this folder, one linked entry each.\n\n\
             - [\u{a7}FS-001-login](FS-001-login.md#fs-001-login-user-login)\n\
             - [\u{a7}FS-002-session](FS-002-session.md#fs-002-session-user-session)\n"
                .to_string(),
        ),
        (
            "docs/functional-spec/FS-001-login.md",
            "# FS-001-login: User login\n\nCreates a session described by FS-002-session.\n\n\
             ## 1. Inputs\n\nUsername and password.\n"
                .to_string(),
        ),
        (
            "docs/functional-spec/FS-002-session.md",
            format!(
                "# FS-002-session: User session\n\n\
                 Created once FS-001-login.1 is read and {missing} succeeds.\n"
            ),
        ),
    ]
}

/// A version 1 project whose `sim/` lies outside `[scan] include`, citing a section
/// `FS-001-login` does not have; `src/` cites the one it has.
const OUT_OF_SCOPE: &[(&str, &str)] = &[
    (
        "grund.toml",
        "grund_config_version = 1\n\n[scan]\ninclude = [\"docs\", \"src\"]\n",
    ),
    (
        "docs/functional-spec/README.md",
        "# FS index\n\nEvery `FS` declaration in this folder, one linked entry each.\n\n\
         - [\u{a7}FS-001-login](FS-001-login.md#fs-001-login-a-user-can-log-in)\n",
    ),
    (
        "docs/functional-spec/FS-001-login.md",
        "# FS-001-login: A user can log in\n\n## 1. Rules\n\nThe login behavior.\n",
    ),
    (
        "src/auth.rs",
        "// Implements \u{a7}FS-001-login.1\npub fn login() {}\n",
    ),
    ("sim/world.py", "# Drives \u{a7}FS-001-login.4\n"),
];

/// `## 1. Overview` renumbered to `## 2.` after the lock recorded `FS-x.1.1`, its text unchanged,
/// and a source line still citing the old address.
const RENUMBERED: &[(&str, &str)] = &[
    (
        "grund.toml",
        "grund_config_version = 2\n\n[schema.kinds.FS]\nfolders = [\"docs\"]\nindex = false\n",
    ),
    (
        "docs/FS-x.md",
        "# FS-x: Exports\n\n## 2. Overview\n\n### .1. Goals\n\n\
         An export writes every row once, in the order the table holds them.\n",
    ),
    (".grund/sections/FS-x.lock", "FS-x.1.1 3c3fca09acca0d92\n"),
    (
        "src/export.py",
        "# Writes the rows in table order (\u{a7}FS-x.1.1)\ndef export(rows):\n    \
         return list(rows)\n",
    ),
];

/// A scratch project removed on drop; `name` keeps concurrent tests apart.
struct Tree(PathBuf);

impl Tree {
    fn new<P: AsRef<str>, T: AsRef<str>>(name: &str, files: &[(P, T)]) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1436-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        for (path, text) in files {
            let path = root.join(path.as_ref());
            fs::create_dir_all(path.parent().expect("a file has a parent")).expect("create dirs");
            fs::write(&path, text.as_ref()).expect("write a fixture file");
        }
        Self(root)
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .env("NO_COLOR", "1")
            .output()
            .expect("run grund")
    }

    /// `grund <args>` prints exactly `stdout` and exits `code`; stderr is the caller's to judge.
    fn assert_stdout(&self, args: &[&str], stdout: &str, code: i32) -> Output {
        let output = self.run(args);
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            stdout,
            "stdout of grund {args:?}; stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.status.code(), Some(code), "grund {args:?}");
        output
    }

    /// Each JSON finding `grund <args>` prints, as `(code, severity, message)`.
    fn json_findings(&self, args: &[&str]) -> Vec<(String, String, String)> {
        let output = self.run(args);
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|line| {
                let finding: serde_json::Value =
                    serde_json::from_str(line).unwrap_or_else(|_| panic!("one finding: {line}"));
                let field = |name: &str| finding[name].as_str().unwrap_or_default().to_string();
                (field("code"), field("severity"), field("message"))
            })
            .collect()
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// The control, green before #1436 as well: the marked named form already named its fix, and its
/// bytes are the prefix every other form now follows, so they may not move (§FS-errors.3.11).
#[test]
fn the_named_form_keeps_its_message() {
    let tree = Tree::new("named", TICKET);
    let output = tree.run(&["check"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let named = format!("src/main.rs:1: error: section not found: FS-check.2.goals{FIX}{MAP}\n");
    assert!(stdout.contains(&named), "{stdout}");
    assert_eq!(output.status.code(), Some(1));
}

/// The report: a missing numeric section names the same fix the missing named section beside it
/// does, after its own head (before #1436: `missing section FS-check.9.9` and nothing more).
#[test]
fn a_missing_numeric_section_names_the_fix_a_named_one_does() {
    Tree::new("ticket", TICKET).assert_stdout(
        &["check"],
        &format!(
            "src/main.rs:1: error: missing section FS-check.9.9{FIX}\n\
             src/main.rs:1: error: section not found: FS-check.2.goals{FIX}{MAP}\n"
        ),
        1,
    );
}

/// The same two findings in JSON: one code, one severity, and each message carrying the fix.
#[test]
fn both_are_missing_section_in_json_and_each_names_the_fix() {
    let findings = Tree::new("json", TICKET).json_findings(&["check", "--format", "json"]);
    let expected = [
        format!("missing section FS-check.9.9{FIX}"),
        format!("section not found: FS-check.2.goals{FIX}{MAP}"),
    ]
    .map(|message| ("missing-section".to_string(), "error".to_string(), message));
    assert_eq!(findings, expected);
}

/// A bare version 1 citation of a missing section is owed the fix as well.
#[test]
fn an_unmarked_citation_of_a_missing_section_names_the_fix() {
    Tree::new("unmarked", &unmarked("FS-001-login.9")).assert_stdout(
        &["check"],
        &format!(
            "docs/functional-spec/FS-002-session.md:3: error: missing section FS-001-login.9{FIX}\n"
        ),
        1,
    );
}

/// The control, green before #1436 as well: the fix is true for an unmarked citation, because
/// writing the escape before the bare token clears it (§FS-citations.recognized.escapes). The run
/// is clean, so the version 1 caution stands in place of `success`.
#[test]
fn the_escape_the_fix_names_clears_an_unmarked_citation() {
    let tree = Tree::new("escaped", &unmarked("<\u{a7}>FS-001-login.9"));
    let output = tree.assert_stdout(&["check"], "", 0);
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "warning: grund.toml is version 1; migrate it with grund config migrate --to 2\n"
    );
}

/// Out past `[scan] include` under `--full`, the same message follows the scope prefix, under
/// `out-of-scope-missing-section` (§FS-check.3.14).
#[test]
fn an_out_of_scope_missing_section_names_the_fix() {
    let tree = Tree::new("full", OUT_OF_SCOPE);
    tree.assert_stdout(
        &["check", "--full"],
        &format!(
            "sim/world.py:1: error: outside [scan] include: missing section FS-001-login.4{FIX}\n"
        ),
        1,
    );
    let findings = tree.json_findings(&["check", "--full", "--format", "json"]);
    assert_eq!(
        findings,
        [(
            "out-of-scope-missing-section".to_string(),
            "error".to_string(),
            format!("outside [scan] include: missing section FS-001-login.4{FIX}"),
        )]
    );
}

/// Where the lock names where a numeric section's text went, the clause keeps its place after
/// the coordinate and the fix comes last (§FS-section-lock.checks.vanished).
#[test]
fn the_fix_follows_the_lock_clause_in_the_missing_section_form() {
    Tree::new("renumbered", RENUMBERED).assert_stdout(
        &["check"],
        &format!(
            "src/export.py:1: error: missing section FS-x.1.1; what .grund/sections/FS-x.lock \
             recorded for it now sits at FS-x.2.1{FIX}\n"
        ),
        1,
    );
}
