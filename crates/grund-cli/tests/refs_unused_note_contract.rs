//! §FS-refs.2.1.1 end to end, from issue #936: a whole-ID query of a declared
//! ID nothing cites, run with no path over a complete scan, keeps its empty
//! answer and exit `0` in every rendering and adds one stderr note naming the
//! `list --unused` call that lists it. The calls each note names, and every
//! case that withholds the note, are pinned in `refs_unused_note_withheld.rs`.

#[path = "support/refs_note_fixture.rs"]
mod refs_note_fixture;

use refs_note_fixture::{Fixture, RENDERINGS, unused_note};

/// Query `id` from `cwd` in every rendering, with and without the
/// sectionless `--descendants` that changes nothing (§FS-refs.4), and
/// require the unchanged answer beside the note naming `list`.
fn every_rendering(fixture: &Fixture, cwd: &str, id: &str, list: &str, failures: &mut Vec<String>) {
    let note = unused_note(id, list);
    for descendants in [false, true] {
        for (flags, stdout) in RENDERINGS {
            let mut args = vec!["refs", id];
            if descendants {
                args.push("--descendants");
            }
            args.extend(flags);
            fixture.check(cwd, &args, stdout, &note, failures);
        }
    }
}

/// §FS-refs.2.1.1: one project under either configuration version, from its
/// root and from below it, in all eight renderings.
#[test]
#[ignore = "red until #936 prints the list --unused note; implement removes this"]
fn unused_note_every_rendering() {
    let mut failures = Vec::new();
    for v2 in [false, true] {
        let fixture = Fixture::new(v2, false);
        for cwd in ["", "src", "docs/functional-spec"] {
            every_rendering(
                &fixture,
                cwd,
                "FS-api-audit",
                "grund list --unused",
                &mut failures,
            );
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// §FS-refs.2.1.1: an alias-qualified query names its project; an
/// unqualified one, at the workspace root or member-local, keeps the bare call.
#[test]
#[ignore = "red until #936 prints the list --unused note; implement removes this"]
fn unused_note_workspace_forms() {
    let mut failures = Vec::new();
    for v2 in [false, true] {
        let fixture = Fixture::new(v2, true);
        fixture.write(
            "packages/other/docs/functional-spec/FS-member.md",
            "# FS-member: Member\n\nText.\n",
        );
        for (cwd, id, list) in [
            (
                "",
                "beta/FS-api-audit",
                "grund list --project beta --unused",
            ),
            (
                "src",
                "beta/FS-api-audit",
                "grund list --project beta --unused",
            ),
            ("", "other/FS-member", "grund list --project other --unused"),
            ("", "FS-api-audit", "grund list --unused"),
            ("packages/other", "FS-member", "grund list --unused"),
        ] {
            every_rendering(&fixture, cwd, id, list, &mut failures);
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// §FS-refs.2.1.1: an `E2E` case adds `--kind E2E`, with or without the alias;
/// a custom kind is in the plain listing and takes no filter.
#[test]
#[ignore = "red until #936 prints the list --unused note; implement removes this"]
fn unused_note_kind_forms() {
    let mut failures = Vec::new();
    for v2 in [false, true] {
        for workspace in [false, true] {
            let fixture = Fixture::new(v2, workspace);
            fixture.add_kinds(v2);
            let (alias, project) = if workspace {
                ("beta/", "--project beta ")
            } else {
                ("", "")
            };
            for (id, filter) in [("E2E-login", " --kind E2E"), ("API-thing", "")] {
                let id = format!("{alias}{id}");
                let list = format!("grund list {project}--unused{filter}");
                every_rendering(&fixture, "", &id, &list, &mut failures);
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}
