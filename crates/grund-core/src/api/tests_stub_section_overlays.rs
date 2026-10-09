//! A stub's sections are its target's, read as a save would write them
//! (§FS-check.3.2.1, §FS-lsp.1.1): with a target outside the scan open in an
//! editor, the snapshot reports the missing sections the saved text gives, as it
//! reads the overlay for a stub's health (§FS-declarations.checks.broken-stub.1).

use super::lsp_snapshot;
use crate::model::format_path;
use crate::queries::LspSnapshotOpts;
use crate::testing::{test_root, write};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// `FS-second` with the section `docs/uses.md` cites.
const WITH_SECTION: &str = concat!(
    "/// FS-second: Second\n///\n/// Second lead.\n///\n",
    "/// ## 1. Detail\n///\n/// Second detail.\npub fn second() {}\n",
);

/// `FS-second` without it.
const WITHOUT_SECTION: &str = "/// FS-second: Second\n///\n/// Second lead.\npub fn second() {}\n";

/// What the snapshot reports for `docs/uses.md`'s citation when the section is gone.
const MISSING: &str = "docs/uses.md:1 missing section FS-second.1";

/// A stub in `docs/` pointing at `source.rs`, which holds `source` on disk and lies
/// outside `[scan] include`, and a citation of its section `1`.
fn stub_repo(name: &str, source: &str) -> PathBuf {
    let root = std::fs::canonicalize(test_root(name)).expect("canonical test root");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n[reference]\nstrict = true\nrequire_grounding = false\n\
         [id]\nformat = \"{kind}-{slug}\"\n\
         [[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n\
         [scan]\ninclude = [\"docs\"]\n",
    );
    write(
        &root.join("docs/second.md"),
        "# FS-second: [../source.rs](../source.rs)\n",
    );
    write(&root.join("docs/uses.md"), "Uses \u{a7}FS-second.1.\n");
    write(&root.join("source.rs"), source);
    root
}

/// The snapshot's errors, as `path:line message` relative to `root`, with
/// `source.rs` open holding `editor` where it is given.
fn errors(root: &Path, editor: Option<&str>) -> Vec<String> {
    let open_documents: BTreeMap<PathBuf, String> = editor
        .map(|text| (root.join("source.rs"), text.to_string()))
        .into_iter()
        .collect();
    let snapshot = lsp_snapshot(LspSnapshotOpts {
        path: root.to_path_buf(),
        path_provided: true,
        open_documents,
    })
    .expect("lsp snapshot");
    snapshot
        .report
        .errors
        .iter()
        .map(|finding| {
            let path = Path::new(finding.path.as_deref().unwrap_or("?"));
            let path = format_path(path.strip_prefix(root).unwrap_or(path));
            format!("{path}:{} {}", finding.line.unwrap_or(0), finding.message)
        })
        .collect()
}

/// Holds `editor` over `disk` and asserts the snapshot reports `expected`, exactly
/// as it does once `editor` is saved.
fn assert_reports_as_saved(name: &str, (disk, editor): (&str, &str), expected: &[&str]) {
    let expected: Vec<String> = expected.iter().map(|error| error.to_string()).collect();
    let saved = stub_repo(&format!("{name}_saved"), editor);
    assert_eq!(errors(&saved, None), expected, "saved");
    let edited = stub_repo(name, disk);
    assert_eq!(
        errors(&edited, Some(editor)),
        expected,
        "under the editor's text"
    );
}

#[test]
fn unsaved_section_in_an_unscanned_target_resolves() {
    let texts = (WITHOUT_SECTION, WITH_SECTION);
    assert_reports_as_saved("stub_section_overlay_adds", texts, &[]);
}

#[test]
fn unsaved_drop_of_a_section_in_an_unscanned_target_is_missing() {
    let texts = (WITH_SECTION, WITHOUT_SECTION);
    assert_reports_as_saved("stub_section_overlay_drops", texts, &[MISSING]);
}
