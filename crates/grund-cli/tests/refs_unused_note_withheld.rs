//! §FS-refs.2.1.1 from issue #936, the half that holds before and after the
//! fix: every `list --unused` call a note names lists the queried declaration,
//! and each case the point withholds the note from keeps its answer, its other
//! notes and its exit. The note itself is `refs_unused_note_contract.rs`.

#[path = "support/refs_note_fixture.rs"]
mod refs_note_fixture;

use refs_note_fixture::{Fixture, RENDERINGS};

/// §FS-refs.2.1.1: the call each note form names lists its target, and the
/// plain listing leaves an `E2E` case out, which is why the filter is named.
#[test]
fn hinted_lists_hold_their_target() {
    let mut failures = Vec::new();
    for v2 in [false, true] {
        let fixture = Fixture::new(v2, false);
        fixture.add_kinds(v2);
        for (cwd, list, row) in [
            ("", "grund list --unused", "FS-api-audit"),
            ("src", "grund list --unused", "FS-api-audit"),
            ("", "grund list --unused", "API-thing"),
            ("", "grund list --unused --kind E2E", "E2E-login"),
        ] {
            fixture.check_lists(cwd, list, row, &mut failures);
        }
        let plain = fixture.run("", &["list", "--unused"]);
        if String::from_utf8_lossy(&plain.stdout).contains("E2E-login") {
            failures.push("grund list --unused: listed the E2E case".to_owned());
        }
        let workspace = Fixture::new(v2, true);
        workspace.add_kinds(v2);
        workspace.write(
            "packages/other/docs/functional-spec/FS-member.md",
            "# FS-member: Member\n\nText.\n",
        );
        for (cwd, list, row) in [
            (
                "",
                "grund list --project beta --unused",
                "beta/FS-api-audit",
            ),
            (
                "src",
                "grund list --project beta --unused",
                "beta/FS-api-audit",
            ),
            ("", "grund list --project beta --unused", "beta/API-thing"),
            (
                "",
                "grund list --project beta --unused --kind E2E",
                "beta/E2E-login",
            ),
            ("", "grund list --project other --unused", "other/FS-member"),
            ("", "grund list --unused", "beta/FS-api-audit"),
            ("packages/other", "grund list --unused", "FS-member"),
        ] {
            workspace.check_lists(cwd, list, row, &mut failures);
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// §FS-refs.2.1.1, withheld case 2: a file, a directory or `.` scopes the
/// answer, in every rendering, a qualified workspace query included.
#[test]
fn withheld_under_a_path() {
    let mut failures = Vec::new();
    for v2 in [false, true] {
        let fixture = Fixture::new(v2, false);
        for (cwd, path) in [
            ("", "."),
            ("", "docs"),
            ("", "docs/functional-spec/FS-api-audit.md"),
            ("docs/functional-spec", "."),
        ] {
            for (flags, stdout) in RENDERINGS {
                let mut args = vec!["refs", "FS-api-audit", path];
                args.extend(flags);
                fixture.check(cwd, &args, stdout, "", &mut failures);
            }
        }
        let workspace = Fixture::new(v2, true);
        for (flags, stdout) in RENDERINGS {
            let mut args = vec!["refs", "beta/FS-api-audit", "."];
            args.extend(flags);
            workspace.check("", &args, stdout, "", &mut failures);
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// §FS-refs.2.1.1, withheld cases 1 and 4: a section, inline or by flag and
/// with or without `--descendants`, and the `--glob` and `--from` modes, are
/// not the whole-ID question `--unused` answers.
#[test]
fn withheld_for_sections_globs_and_sources() {
    let mut failures = Vec::new();
    for v2 in [false, true] {
        let fixture = Fixture::new(v2, false);
        fixture.write(
            "docs/functional-spec/FS-api-audit.md",
            "# FS-api-audit: API audit\n\nText.\n\n## 1. Scope\n\nScope.\n",
        );
        for args in [
            &["refs", "FS-api-audit.1"][..],
            &["refs", "FS-api-audit.1", "--descendants"],
            &["refs", "FS-api-audit", "--section", "1"],
            &["refs", "FS-api-audit", "--section", "1", "--descendants"],
            &["refs", "--glob", "FS-api-audit"],
            &["refs", "--glob", "FS-api-*"],
            &["refs", "--from", "FS-api-audit"],
        ] {
            for (flags, stdout) in RENDERINGS {
                // A source's empty total is its own sentence (§FS-refs.from.records).
                let from = args.contains(&"--from") && stdout == "not cited\n";
                let stdout = if from { "cites nothing\n" } else { stdout };
                let mut args = args.to_vec();
                args.extend(flags);
                fixture.check("", &args, stdout, "", &mut failures);
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// §FS-refs.2.1.1, withheld case 5: an answer with a site earns no note, an
/// index entry the listing discounts included, though `list --unused` lists it.
#[test]
fn withheld_for_an_answer_with_sites() {
    let mut failures = Vec::new();
    for v2 in [false, true] {
        let fixture = Fixture::new(v2, false);
        fixture.append(
            "grund.toml",
            if v2 {
                "[schema.kinds.DEC]\nfolders = [\"docs/decisions\"]\n"
            } else {
                "[[kinds]]\nkind = \"DEC\"\nfolder = \"docs/decisions\"\n"
            },
        );
        fixture.write(
            "docs/decisions/DEC-choice.md",
            "# DEC-choice: Choice\n\nText.\n",
        );
        fixture.write(
            "docs/decisions/README.md",
            "# Decisions\n\n- [\u{a7}DEC-choice](DEC-choice.md#dec-choice-choice)\n",
        );
        fixture.check(
            "",
            &["refs", "FS-login"],
            "src/lib.rs:1: \u{a7}FS-login\n",
            "",
            &mut failures,
        );
        fixture.check(
            "",
            &["refs", "DEC-choice"],
            "docs/decisions/README.md:3: \u{a7}DEC-choice (index entry)\n",
            "",
            &mut failures,
        );
        fixture.check_lists("", "grund list --unused", "DEC-choice", &mut failures);
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// §FS-refs.2.1.1, withheld case 3: an incomplete scan exits `2` with its
/// error alone, its empty answer not trustworthy as complete.
#[cfg(unix)]
#[test]
fn withheld_for_an_incomplete_scan() {
    let mut failures = Vec::new();
    for v2 in [false, true] {
        let fixture = Fixture::new(v2, false);
        std::os::unix::fs::symlink(
            "missing.md",
            fixture.0.join("docs/functional-spec/FS-gone.md"),
        )
        .unwrap();
        let error = "error: docs/functional-spec/FS-gone.md: broken symlink: \
                     the target does not exist; make the path readable, or keep it out of \
                     the scan\n";
        for (flags, stdout) in RENDERINGS {
            let mut args = vec!["refs", "FS-api-audit"];
            args.extend(flags);
            fixture.check_exit("", &args, 2, stdout, error, &mut failures);
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}
