//! A buffer opened under a link (§FS-scan.identity.buffers): it is read as its file, whichever
//! name the editor opened it under, and takes no on-type edit where the run refuses the file.
//! Unix-only: the cases are about links.

use super::LspSnapshotOpts;
use super::tests_editor_completion::Fixture;
use super::{DeclaredId, on_type_line_edits};
use crate::api::lsp_snapshot;
use crate::testing::symlink;

const CONFIG: &str = "grund_config_version = 2\nproject_name = \"names\"\n\
    [schema.sources]\nlanguages = [\"markdown\", \"rust\"]\n\
    [schema.kinds.FS]\nfolders = [\"docs/functional-spec\"]\nindex = false\n";
/// Markdown converts the trigger; Rust keeps a string literal's bytes (§FS-fmt.2.3).
const LINE: &str = "let s = \"$$FS-login x\";\n";

fn fixture() -> Fixture {
    let f = Fixture::new();
    f.put("grund.toml", CONFIG);
    f
}

/// How many edits the keystroke after the trigger's token offers in the buffer at `file`.
fn edits_in(f: &Fixture, file: &str) -> usize {
    let declarations = vec![DeclaredId {
        path: &f.0,
        id: "FS-login",
    }];
    let cursor = LINE.find("FS-login").expect("the token") + "FS-login".len();
    on_type_line_edits(&f.0.join(file), LINE, 0, cursor, &declarations)
        .expect("on-type edits")
        .len()
}

#[test]
fn a_buffer_under_a_link_is_read_as_its_file() {
    let f = fixture();
    f.put("src/real.rs", "\n");
    symlink("real.rs", &f.0.join("src/alias.md"));
    symlink("../docs/notes.md", &f.0.join("src/notes.rs"));
    assert_eq!(edits_in(&f, "docs/notes.md"), 1, "the Markdown control");
    assert_eq!(edits_in(&f, "src/real.rs"), 0, "the Rust control");
    assert_eq!(
        edits_in(&f, "src/alias.md"),
        0,
        "a `.md` name of a Rust file"
    );
    assert_eq!(
        edits_in(&f, "src/notes.rs"),
        1,
        "a `.rs` name of a Markdown file"
    );
}

/// The refusal is the run's, read off the snapshot a keystroke already holds, and no name of the
/// file escapes it: not the links, and not the target's own name, which no finding lists
/// (§FS-scan.identity.buffers).
#[test]
fn the_snapshot_says_a_file_is_refused_under_every_name() {
    let f = fixture();
    f.put("LICENSE", "\n");
    f.put("docs/other.md", "\n");
    symlink("../LICENSE", &f.0.join("docs/license.md"));
    let refused = |name: &str| {
        let snapshot = lsp_snapshot(LspSnapshotOpts {
            path: f.0.clone(),
            path_provided: true,
            ..LspSnapshotOpts::default()
        })
        .expect("snapshot");
        snapshot.reading_refused(&f.0.join(name))
    };
    assert!(!refused("docs/license.md"), "one name is no refusal");
    assert!(!refused("LICENSE"), "one name is no refusal");
    symlink("../LICENSE", &f.0.join("docs/license.rs"));
    for name in ["docs/license.md", "docs/license.rs", "LICENSE"] {
        assert!(refused(name), "{name}");
    }
    assert!(!refused("docs/other.md"), "another file is not refused");
}
