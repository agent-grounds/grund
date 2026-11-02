//! The editor's declaration title span ends where the declaration's title ends
//! (§FS-lsp.1.3.4): on a `/* … */` line, before the block's first `*/`
//! (§FS-declarations.line.block-comment), while a line comment or a Markdown
//! heading keeps a `*/` its author wrote.

use super::*;
use crate::queries::LspSnapshotOpts;
use crate::testing::{test_root, write};

#[test]
fn declaration_span_ends_before_the_first_block_closer() {
    let root = test_root("lsp_block_closer_spans");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n[id]\nformat = \"{kind}-{slug}\"\n[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n",
    );
    write(&root.join("docs/FS-md.md"), "# FS-md: Doc */\n");
    write(
        &root.join("src/lib.rs"),
        "/** FS-b: Beta */\nfn b() {}\n\n/* FS-tail: Head */ fn e() {} /* x */\n\n/// FS-glob: matches src/**/ files\nfn g() {}\n",
    );
    let snapshot = lsp_snapshot(LspSnapshotOpts {
        path: root,
        path_provided: true,
        open_documents: Default::default(),
    })
    .unwrap();
    let span = |id: &str| {
        let decl = snapshot
            .declarations
            .iter()
            .find(|decl| decl.query_id == id)
            .unwrap_or_else(|| panic!("no declaration {id}"));
        (decl.line, decl.column, decl.text.as_str())
    };
    assert_eq!(span("FS-b"), (1, 5, "FS-b: Beta"));
    assert_eq!(span("FS-tail"), (4, 4, "FS-tail: Head"));
    assert_eq!(span("FS-glob"), (6, 5, "FS-glob: matches src/**/ files"));
    assert_eq!(span("FS-md"), (1, 3, "FS-md: Doc */"));
}
