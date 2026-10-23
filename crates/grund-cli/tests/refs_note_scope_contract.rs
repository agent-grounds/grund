//! End-to-end wording contract for the scanned-tree hint (§FS-refs.2.1),
//! ported from issue #862 / grund.120's docs-and-src reproducer. A declared
//! ID's no-path answer is #936's note, pinned in `refs_unused_note_contract.rs`.

#[path = "support/refs_note_fixture.rs"]
mod refs_note_fixture;

use refs_note_fixture::Fixture;

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
fn refs_note_scope_single_project() {
    assert_scoped_notes(false, false);
}

/// §FS-refs.2.1: the alias-qualified branch keeps project-specific list advice.
#[test]
fn refs_note_scope_workspace_qualified() {
    assert_scoped_notes(false, true);
}

/// §FS-refs.2.1: the diagnostic wording is independent of configuration version.
#[test]
fn refs_note_scope_v2() {
    assert_scoped_notes(true, false);
}

/// §FS-refs.2.1: no-path typo hints, an uncited declaration under a path and
/// cited hits keep their established streams and status.
#[test]
fn refs_note_scope_unchanged_controls() {
    let mut failures = Vec::new();
    for v2 in [false, true] {
        let fixture = Fixture::new(v2, false);
        let list = fixture.run("src", &["list"]);
        assert!(list.status.success());
        assert!(String::from_utf8_lossy(&list.stdout).contains("FS-api-audit"));
        fixture.check("src", &["FS-api-audit", "."], "Text.\n", "", &mut failures);
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
