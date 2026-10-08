//! A stub's target is read as a save would write it
//! (§FS-declarations.checks.broken-stub.1): with the target open in an editor,
//! `show` and the editor snapshot answer what they answer once that text is
//! saved (§FS-show.2.3.4, §FS-show.2.3.7, §FS-lsp.1.1), whether or not the
//! target is in the scan.

use super::{lsp_snapshot, show_with_overlays};
use crate::queries::{LspSnapshotOpts, ShowOpts};
use crate::testing::{test_root, write};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// `source.rs` declaring both IDs its two stubs point at.
const DECLARES: &str = concat!(
    "/// FS-first: First\n///\n/// First lead.\npub fn first() {}\n\n",
    "/// FS-second: Second\n///\n/// Second lead.\npub fn second() {}\n",
);

/// The same file with `FS-second` dropped. It names `FS-second` nowhere, because a
/// comment such as `// FS-second was deleted` is itself a declaration of it.
const DROPS: &str = concat!(
    "/// FS-first: First\n///\n/// First lead.\npub fn first() {}\n\n",
    "pub fn second() {}\n",
);

/// The same file with only `FS-second`'s lead rewritten.
const RELEADS: &str = concat!(
    "/// FS-first: First\n///\n/// First lead.\npub fn first() {}\n\n",
    "/// FS-second: Second\n///\n/// Overlay lead.\npub fn second() {}\n",
);

/// The second line of §FS-show.2.3.4, for the `docs/second.md` stub.
const REFUSAL: &str = concat!(
    "broken stub: FS-second (stub at docs/second.md:1 points at ../source.rs, ",
    "which contains no inline declaration of FS-second)",
);

/// The `broken-stub` error §FS-declarations.checks.broken-stub puts on that stub.
const BROKEN: &str = "docs/second.md:1 stub link target lacks FS-second: ../source.rs";

/// Two stubs in `docs/` pointing at one `source.rs`, which holds `source` on disk
/// and is inside `[scan] include` only when `scanned`.
fn stub_repo(name: &str, scanned: bool, source: &str) -> PathBuf {
    let root = std::fs::canonicalize(test_root(name)).expect("canonical test root");
    let include = if scanned {
        "[\"docs\", \"source.rs\"]"
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
    write(
        &root.join("docs/first.md"),
        "# FS-first: [../source.rs](../source.rs)\n",
    );
    write(
        &root.join("docs/second.md"),
        "# FS-second: [../source.rs](../source.rs)\n",
    );
    write(
        &root.join("docs/uses.md"),
        "Uses \u{a7}FS-first and \u{a7}FS-second.\n",
    );
    write(&root.join("source.rs"), source);
    root
}

/// `source.rs` open in the editor holding `text`, or nothing open.
fn open(root: &Path, text: Option<&str>) -> BTreeMap<PathBuf, String> {
    text.map(|text| (root.join("source.rs"), text.to_string()))
        .into_iter()
        .collect()
}

/// `show FS-second`: the body, or the refusal.
fn show_second(root: &Path, editor: Option<&str>) -> Result<String, String> {
    let opts = ShowOpts {
        path: root.to_path_buf(),
        ..ShowOpts::default()
    };
    show_with_overlays("FS-second", opts, open(root, editor))
        .map(|out| out.body.trim_end().to_string())
        .map_err(|err| err.to_string())
}

/// The editor snapshot's `broken-stub` errors, as `path:line message` with the
/// path relative to `root`.
fn broken_stubs(root: &Path, editor: Option<&str>) -> Vec<String> {
    let snapshot = lsp_snapshot(LspSnapshotOpts {
        path: root.to_path_buf(),
        path_provided: true,
        open_documents: open(root, editor),
    })
    .expect("lsp snapshot");
    snapshot
        .report
        .errors
        .iter()
        .filter(|finding| finding.code == "broken-stub")
        .map(|finding| {
            let path = Path::new(finding.path.as_deref().unwrap_or("?"));
            let path = path.strip_prefix(root).unwrap_or(path).display();
            format!("{path}:{} {}", finding.line.unwrap_or(0), finding.message)
        })
        .collect()
}

/// `show FS-second` and the snapshot's `broken-stub` errors, side by side.
type Answers = (Result<String, String>, Vec<String>);

fn answers(root: &Path, editor: Option<&str>) -> Answers {
    (show_second(root, editor), broken_stubs(root, editor))
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

#[test]
fn unsaved_drop_in_a_scanned_target_breaks_the_stub() {
    let texts = (DECLARES, DROPS);
    assert_answers_as_saved(
        "stub_overlay_drop_scanned",
        true,
        texts,
        Err(REFUSAL),
        &[BROKEN],
    );
}

#[test]
fn unsaved_drop_in_an_unscanned_target_breaks_the_stub() {
    let texts = (DECLARES, DROPS);
    assert_answers_as_saved(
        "stub_overlay_drop_unscanned",
        false,
        texts,
        Err(REFUSAL),
        &[BROKEN],
    );
}

#[test]
fn unsaved_declaration_repairs_the_stub() {
    let body = "Second lead.";
    for scanned in [true, false] {
        let name = format!("stub_overlay_restore_{scanned}");
        assert_answers_as_saved(&name, scanned, (DROPS, DECLARES), Ok(body), &[]);
    }
}

/// The guard: the body was already read from the editor's text, and stays so.
#[test]
fn unsaved_lead_is_the_body_shown() {
    let body = "Overlay lead.";
    for scanned in [true, false] {
        let name = format!("stub_overlay_lead_{scanned}");
        assert_answers_as_saved(&name, scanned, (DECLARES, RELEADS), Ok(body), &[]);
    }
}
