//! End-to-end wording contract for the scanned-tree hint (§FS-refs.2.1),
//! ported from issue #862 / grund.120's docs-and-src reproducer.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

const V1: &str = "grund_config_version = 1\n\
    [reference]\nstrict = true\n\
    [id]\nformat = \"{kind}-{slug}\"\nnamed_sections = true\n\
    [scan]\ninclude = [\"docs\", \"src\"]\n\
    [[kinds]]\nkind = \"FS\"\nfolder = \"docs/functional-spec\"\nindex = false\n";

const V2: &str = "grund_config_version = 2\n\
    [schema]\nid_format = \"{kind}-{slug}\"\n\
    [schema.kinds.FS]\nfolders = [\"docs/functional-spec\"]\nindex = false\n\
    [schema.kinds.code]\nfolders = [\"src\"]\n";

struct Fixture(PathBuf);

impl Fixture {
    fn new(v2: bool, workspace: bool) -> Self {
        let root = std::env::temp_dir().join(format!(
            "grund-862-refs-note-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("docs/functional-spec")).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        let config = if v2 { V2 } else { V1 };
        let config = if workspace {
            fs::create_dir_all(root.join("packages/other/docs/functional-spec")).unwrap();
            fs::write(root.join("packages/other/grund.toml"), V1).unwrap();
            format!(
                "project_name = \"beta\"\n{config}\n\
                 [workspace]\nmembers = [\"packages/other\"]\n"
            )
        } else {
            config.to_owned()
        };
        fs::write(root.join("grund.toml"), config).unwrap();
        for (slug, title) in [("api-audit", "API audit"), ("login", "Login")] {
            fs::write(
                root.join(format!("docs/functional-spec/FS-{slug}.md")),
                format!("# FS-{slug}: {title}\n\nText.\n"),
            )
            .unwrap();
        }
        fs::write(
            root.join("src/lib.rs"),
            "// See \u{a7}FS-login.\npub fn f() {}\n",
        )
        .unwrap();
        Self(root)
    }

    fn run(&self, cwd: &str, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .current_dir(self.0.join(cwd))
            .args(args)
            .output()
            .expect("run grund")
    }

    fn check(
        &self,
        cwd: &str,
        args: &[&str],
        expected_stdout: &str,
        expected_stderr: &str,
        failures: &mut Vec<String>,
    ) {
        let output = self.run(cwd, args);
        let label = format!("(cd {cwd}) grund {}", args.join(" "));
        if output.status.code() != Some(0) {
            failures.push(format!("{label}: expected exit 0, got {:?}", output.status));
        }
        // Judge streams separately so a wording mismatch cannot hide result drift.
        for (stream, actual, expected) in [
            ("stdout", &output.stdout, expected_stdout),
            ("stderr", &output.stderr, expected_stderr),
        ] {
            let actual = String::from_utf8_lossy(actual).replace("\r\n", "\n");
            if actual != expected {
                failures.push(format!(
                    "{label}: {stream}\nexpected: {expected:?}\nactual: {actual:?}"
                ));
            }
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn scoped_note(id: &str, path: &str, workspace: bool) -> String {
    let advice = if workspace {
        "run `grund list --project beta` to see beta's declared IDs"
    } else {
        "run `grund list` to see project declarations"
    };
    format!(
        "note: {id} is neither declared nor cited in scan path `{path}` — \
         {advice}, including outside this scan\n"
    )
}

fn assert_scoped_notes(v2: bool, workspace: bool) {
    let fixture = Fixture::new(v2, workspace);
    let id = if workspace {
        // The workspace loader retains this root project's declarations, so
        // exercise its note with a real typo rather than change its predicate.
        "beta/FS-api-auditx"
    } else {
        "FS-api-audit"
    };
    let mut failures = Vec::new();
    // Same no-path control as the original reproducer: the ID is declared.
    let declared = if workspace {
        "beta/FS-api-audit"
    } else {
        "FS-api-audit"
    };
    fixture.check("", &["refs", declared], "", "", &mut failures);
    for (cwd, path) in [("", "src"), ("src", "."), ("", "src/lib.rs")] {
        let note = scoped_note(id, path, workspace);
        for (flags, stdout) in [
            (vec![], ""),
            (vec!["--total"], "not cited\n"),
            (vec!["--format", "json"], ""),
            (
                vec!["--format", "json", "--total"],
                "{\"sites\":0,\"files\":0}\n",
            ),
        ] {
            let mut args = vec!["refs", id, path];
            args.extend(flags);
            fixture.check(cwd, &args, stdout, &note, &mut failures);
        }
    }
    // A real typo retains the same hint with the same selected scope.
    let typo = if workspace {
        "beta/FS-api-auditx"
    } else {
        "FS-api-auditx"
    };
    fixture.check(
        "",
        &["refs", typo, "src"],
        "",
        &scoped_note(typo, "src", workspace),
        &mut failures,
    );
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// §FS-refs.2.1: path-qualified wording for the reported v1 directory case,
/// a file path, totals, and text/JSON stream separation.
#[test]
#[ignore = "red until #862 qualifies the scan-path note; implement removes this"]
fn refs_note_scope_single_project() {
    assert_scoped_notes(false, false);
}

/// §FS-refs.2.1: the alias-qualified branch keeps project-specific list advice.
#[test]
#[ignore = "red until #862 qualifies the scan-path note; implement removes this"]
fn refs_note_scope_workspace_qualified() {
    assert_scoped_notes(false, true);
}

/// §FS-refs.2.1: the diagnostic wording is independent of configuration version.
#[test]
#[ignore = "red until #862 qualifies the scan-path note; implement removes this"]
fn refs_note_scope_v2() {
    assert_scoped_notes(true, false);
}

/// §FS-refs.2.1: no-path hints, uncited declarations and cited hits keep their
/// established streams and status. These controls pass before implementation.
#[test]
fn refs_note_scope_unchanged_controls() {
    let mut failures = Vec::new();
    for v2 in [false, true] {
        let fixture = Fixture::new(v2, false);
        let list = fixture.run("src", &["list"]);
        assert!(list.status.success());
        assert!(String::from_utf8_lossy(&list.stdout).contains("FS-api-audit"));
        fixture.check("src", &["FS-api-audit", "."], "Text.\n", "", &mut failures);
        fixture.check("src", &["refs", "FS-api-audit"], "", "", &mut failures);
        fixture.check(
            "",
            &["refs", "FS-api-audit", "docs/functional-spec"],
            "",
            "",
            &mut failures,
        );
        fixture.check(
            "src",
            &["refs", "FS-api-auditx"],
            "",
            "note: FS-api-auditx is neither declared nor cited — run `grund list` to see every declared ID\n",
            &mut failures,
        );
        fixture.check(
            "src",
            &["refs", "FS-login", "."],
            "src/lib.rs:1: \u{a7}FS-login\n",
            "",
            &mut failures,
        );
    }
    let fixture = Fixture::new(false, true);
    fixture.check("", &["refs", "beta/FS-api-audit"], "", "", &mut failures);
    fixture.check(
        "",
        &["refs", "beta/FS-api-auditx"],
        "",
        "note: beta/FS-api-auditx is neither declared nor cited — run `grund list --project beta` to see beta's declared IDs\n",
        &mut failures,
    );
    fixture.check(
        "",
        &["refs", "beta/FS-login", "src"],
        "src/lib.rs:1: \u{a7}FS-login\n",
        "",
        &mut failures,
    );
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}
