//! One verdict per stub, read by every command
//! (§FS-declarations.checks.broken-stub.4).
//!
//! Each row of the table is one tree around the stub `docs/a.md` →
//! `notes/<target>` and the citation `docs/uses.md`; each column is a command
//! that reads the stub. On every row the columns must agree with the row's one
//! verdict: whether `check` reports the stub broken
//! (§FS-declarations.checks.broken-stub.3), and the homes of `FS-a`
//! (§FS-declarations.checks.duplicate.1). So `show` and `refs` refuse exactly
//! the IDs `check` reports a duplicate, at the same sites (§FS-show.2.2.1,
//! §FS-refs.4.1); `list --size` measures every home and only the healthy ones
//! (§FS-list.3.4.6); `fmt` links only an ID with one home that is not a broken
//! stub (§FS-fmt.6.4.2); and the editor snapshot reports and navigates the same
//! way, its unsaved text answering what the saved text does
//! (§FS-declarations.checks.broken-stub.1).

#[path = "support/stub_verdict.rs"]
mod stub_verdict;

use stub_verdict::{Row, assert_rows_agree};

/// A target declaring `FS-a` once, on line 1.
const ONCE: &str = "# FS-a: A\n\nLead.\n";
/// A target declaring `FS-a` twice, on lines 1 and 5.
const TWICE: &str = "# FS-a: A\n\nLead.\n\n# FS-a: B\n\nOther.\n";
/// A target whose only heading of `FS-a` sits inside a fence.
const FENCED: &str = "```\n# FS-a: A\n```\n";
/// A target that names no `FS-a` at all.
const NONE: &str = "No declaration here.\n";

const STUB: &[(&str, usize)] = &[("docs/a.md", 1)];
const TARGET: &[(&str, usize)] = &[("notes/a.md", 1)];
const TARGET_TWICE: &[(&str, usize)] = &[("notes/a.md", 1), ("notes/a.md", 5)];
/// A broken stub beside the declaration its symlink resolves to.
#[cfg(unix)]
const STUB_AND_TARGET: &[(&str, usize)] = &[("docs/a.md", 1), ("notes/a.md", 1)];

const SCANNED: &[&str] = &["docs", "notes"];
const OUTSIDE: &[&str] = &["docs"];

const fn row(name: &'static str, include: &'static [&'static str], target: &'static str) -> Row {
    Row {
        name,
        include,
        target,
        files: &[],
        links: &[],
        editor: &[],
        broken: false,
        homes: TARGET,
    }
}

const DISK_ROWS: &[Row] = &[
    Row {
        files: &[("notes/a.md", ONCE)],
        ..row("scanned", SCANNED, "a.md")
    },
    Row {
        broken: true,
        homes: STUB,
        ..row("missing", SCANNED, "a.md")
    },
    Row {
        files: &[("notes/a.md", ONCE)],
        ..row("outside-include", OUTSIDE, "a.md")
    },
    Row {
        files: &[("notes/.a.md", ONCE)],
        broken: true,
        homes: STUB,
        ..row("hidden-name", SCANNED, ".a.md")
    },
    Row {
        files: &[("notes/a.zz", ONCE)],
        broken: true,
        homes: STUB,
        ..row("unlisted-extension", SCANNED, "a.zz")
    },
    Row {
        files: &[("notes/a.md", FENCED)],
        broken: true,
        homes: STUB,
        ..row("fenced-heading-scanned", SCANNED, "a.md")
    },
    Row {
        files: &[("notes/a.md", FENCED)],
        broken: true,
        homes: STUB,
        ..row("fenced-heading-outside", OUTSIDE, "a.md")
    },
    Row {
        files: &[("notes/a.md", TWICE)],
        homes: TARGET_TWICE,
        ..row("declared-twice-scanned", SCANNED, "a.md")
    },
    Row {
        files: &[("notes/a.md", TWICE)],
        homes: TARGET_TWICE,
        ..row("declared-twice-outside", OUTSIDE, "a.md")
    },
];

/// A symlink is judged by the name the stub wrote, never the file it resolves
/// to (§FS-declarations.checks.broken-stub.3).
#[cfg(unix)]
const LINK_ROWS: &[Row] = &[
    Row {
        files: &[("notes/a.md", ONCE)],
        links: &[("notes/.a.md", "a.md")],
        broken: true,
        homes: STUB_AND_TARGET,
        ..row("hidden-symlink-scanned", SCANNED, ".a.md")
    },
    Row {
        files: &[("notes/a.md", ONCE)],
        links: &[("notes/.a.md", "a.md")],
        broken: true,
        homes: STUB,
        ..row("hidden-symlink-outside", OUTSIDE, ".a.md")
    },
    Row {
        files: &[("notes/a.md", ONCE)],
        links: &[("notes/a.zz", "a.md")],
        broken: true,
        homes: STUB_AND_TARGET,
        ..row("unlisted-symlink-scanned", SCANNED, "a.zz")
    },
];

/// The editor's text moves every answer exactly as saving it would
/// (§FS-declarations.checks.duplicate.1).
const EDITOR_ROWS: &[Row] = &[
    Row {
        files: &[("notes/a.md", ONCE)],
        editor: &[("notes/a.md", NONE)],
        broken: true,
        homes: STUB,
        ..row("unsaved-removal-scanned", SCANNED, "a.md")
    },
    Row {
        files: &[("notes/a.md", ONCE)],
        editor: &[("notes/a.md", NONE)],
        broken: true,
        homes: STUB,
        ..row("unsaved-removal-outside", OUTSIDE, "a.md")
    },
    Row {
        files: &[("notes/a.md", NONE)],
        editor: &[("notes/a.md", TWICE)],
        homes: TARGET_TWICE,
        ..row("unsaved-second-declaration-scanned", SCANNED, "a.md")
    },
    Row {
        files: &[("notes/a.md", NONE)],
        editor: &[("notes/a.md", TWICE)],
        homes: TARGET_TWICE,
        ..row("unsaved-second-declaration-outside", OUTSIDE, "a.md")
    },
];

#[test]
fn every_command_reads_the_stubs_one_verdict() {
    assert_rows_agree(DISK_ROWS);
}

#[cfg(unix)]
#[test]
fn a_symlinked_target_is_judged_by_the_name_the_stub_wrote() {
    assert_rows_agree(LINK_ROWS);
}

#[test]
fn unsaved_text_moves_every_answer_as_saving_it_would() {
    assert_rows_agree(EDITOR_ROWS);
}
