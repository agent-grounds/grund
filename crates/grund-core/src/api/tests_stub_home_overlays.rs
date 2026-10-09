//! The count of homes reads a stub's target as a save would write it
//! (§FS-declarations.checks.duplicate.1): with the target open in an editor, the
//! snapshot reports the duplicate the saved text gives, and `show` refuses or
//! answers as the saved text does (§FS-show.2.2.1, §FS-lsp.1.1), because the stub
//! pairs on the same reading as its health (§FS-declarations.checks.broken-stub.1).

use super::{lsp_snapshot, show_with_overlays};
use crate::model::format_path;
use crate::queries::{LspSnapshotOpts, ShowOpts};
use crate::testing::{test_root, write};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// `src/x.rs` declaring the ID both stubs point at.
const DECLARES: &str = "/// FS-x: X\n///\n/// X lead.\npub fn x() {}\n";

/// The same file with the declaration gone. It names the ID nowhere, because a
/// comment naming it is itself a declaration of it.
const DROPS: &str = "pub fn x() {}\n";

/// Two stubs of one ID in `docs/` pointing at `src/x.rs`, which holds `source` on
/// disk and is inside `[scan] include` only when `scanned`.
fn stub_repo(name: &str, scanned: bool, source: &str) -> PathBuf {
    let root = std::fs::canonicalize(test_root(name)).expect("canonical test root");
    let include = if scanned {
        "[\"docs\", \"src\"]"
    } else {
        "[\"docs\"]"
    };
    write(
        &root.join("grund.toml"),
        &format!(
            "grund_config_version = 1\n[reference]\nstrict = true\nrequire_grounding = false\n\
             [id]\nformat = \"{{kind}}-{{slug}}\"\n\
             [[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n\
             [scan]\ninclude = {include}\n"
        ),
    );
    let stub = "# FS-x: [../src/x.rs](../src/x.rs)\n";
    write(&root.join("docs/a.md"), stub);
    write(&root.join("docs/b.md"), stub);
    write(&root.join("src/x.rs"), source);
    root
}

/// `src/x.rs` open in the editor holding `text`, or nothing open.
fn open(root: &Path, text: Option<&str>) -> BTreeMap<PathBuf, String> {
    text.map(|text| (root.join("src/x.rs"), text.to_string()))
        .into_iter()
        .collect()
}

/// `path` relative to `root`, spelled as a report spells it (§FS-errors.4).
fn relative(root: &Path, path: &Path) -> String {
    format_path(path.strip_prefix(root).unwrap_or(path))
}

/// `show FS-x`: `path:line body`, or the refusal.
fn show_x(root: &Path, editor: Option<&str>) -> Result<String, String> {
    let opts = ShowOpts {
        path: root.to_path_buf(),
        ..ShowOpts::default()
    };
    show_with_overlays("FS-x", opts, open(root, editor))
        .map(|out| {
            let path = relative(root, &out.path);
            format!("{path}:{} {}", out.line, out.body.trim_end())
        })
        .map_err(|err| err.to_string())
}

/// Every error of the editor snapshot, as `code path:line message`, sorted.
fn snapshot_errors(root: &Path, editor: Option<&str>) -> Vec<String> {
    let snapshot = lsp_snapshot(LspSnapshotOpts {
        path: root.to_path_buf(),
        path_provided: true,
        open_documents: open(root, editor),
    })
    .expect("lsp snapshot");
    let mut errors: Vec<String> = snapshot
        .report
        .errors
        .iter()
        .map(|finding| {
            let path = relative(root, Path::new(finding.path.as_deref().unwrap_or("?")));
            let line = finding.line.unwrap_or(0);
            format!("{} {path}:{line} {}", finding.code, finding.message)
        })
        .collect();
    errors.sort();
    errors
}

/// `show FS-x` and the snapshot's errors, side by side.
type Answers = (Result<String, String>, Vec<String>);

fn answers(root: &Path, editor: Option<&str>) -> Answers {
    (show_x(root, editor), snapshot_errors(root, editor))
}

/// Holds `editor` over `disk` and asserts that `show` and the snapshot answer
/// `shown` and `errors`, exactly as they do once `editor` is saved.
fn assert_answers_as_saved(
    name: &str,
    scanned: bool,
    (disk, editor): (&str, &str),
    shown: Result<&str, &str>,
    errors: &[&str],
) {
    let expected: Answers = (
        shown.map(str::to_string).map_err(str::to_string),
        errors.iter().map(|error| error.to_string()).collect(),
    );
    let saved = stub_repo(&format!("{name}_saved"), scanned, editor);
    assert_eq!(answers(&saved, None), expected, "saved, scanned={scanned}");
    let edited = stub_repo(name, scanned, disk);
    assert_eq!(
        answers(&edited, Some(editor)),
        expected,
        "under the editor's text, scanned={scanned}"
    );
}

/// With the declaration gone each stub is broken and a home of its own, so the ID
/// has two homes: the duplicate beside the two `broken-stub` errors, and the
/// ambiguity refusal of §FS-show.2.2.1.
fn assert_drop_leaves_two_homes(name: &str, scanned: bool) {
    assert_answers_as_saved(
        name,
        scanned,
        (DECLARES, DROPS),
        Err("ambiguous ID: FS-x (declared at docs/a.md:1, docs/b.md:1)"),
        &[
            "broken-stub docs/a.md:1 stub link target lacks FS-x: ../src/x.rs",
            "broken-stub docs/b.md:1 stub link target lacks FS-x: ../src/x.rs",
            "duplicate docs/a.md:1 duplicate declaration of FS-x (also declared at docs/b.md:1)",
        ],
    );
}

/// With the declaration in place both stubs pair with it, so the ID has one home,
/// at the target (§FS-declarations.checks.duplicate.2).
fn assert_declaration_pairs_the_stubs(name: &str, scanned: bool) {
    let texts = (DROPS, DECLARES);
    assert_answers_as_saved(name, scanned, texts, Ok("src/x.rs:1 X lead."), &[]);
}

#[test]
fn unsaved_drop_in_an_unscanned_target_leaves_the_duplicate() {
    assert_drop_leaves_two_homes("stub_home_overlay_drop_unscanned", false);
}

#[test]
fn unsaved_declaration_in_an_unscanned_target_pairs_the_stubs() {
    assert_declaration_pairs_the_stubs("stub_home_overlay_add_unscanned", false);
}

/// A scanned target the editor's text drops has no record of the ID, so the stub's
/// home is read from the target as well, and that read must be the same text.
#[test]
fn unsaved_drop_in_a_scanned_target_leaves_the_duplicate() {
    assert_drop_leaves_two_homes("stub_home_overlay_drop_scanned", true);
}

/// The guard: the walk already reads a scanned target from the editor's text, and
/// pairs the stubs with the record it finds there.
#[test]
fn unsaved_declaration_in_a_scanned_target_pairs_the_stubs() {
    assert_declaration_pairs_the_stubs("stub_home_overlay_add_scanned", true);
}
