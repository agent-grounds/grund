//! The reachable alias-qualified LSP trigger correction (§FS-config.3.2,
//! §FS-lsp.1.4). The cursor can sit before an existing off-format continuation.

use super::{can_replace_trigger_at, on_type_line_edits};
use crate::testing::{test_root, write};

#[test]
fn kind_heads_lsp_keeps_trigger_before_override_continuation() {
    let root = test_root("kind_heads_lsp");
    let config = "grund_config_version = 2\n[schema]\nid_format = \"{kind}-{number}-{slug}\"\n\
                  [schema.kinds.ADR]\nfiles = [\"adr.md\"]\nid_format = \"{kind}:{number}:{slug}\"\n";
    write(
        &root.join("grund.toml"),
        &format!("{config}\n[workspace]\nmembers = [\"api\"]\n"),
    );
    write(&root.join("api/grund.toml"), config);
    write(&root.join("adr.md"), "# ADR:7:why: Why\n");
    write(&root.join("api/adr.md"), "# ADR:7:why: Why\n");
    let path = root.join("notes.md");
    let line = "See $$api/ADR:7:Upper.";
    write(&path, line);
    let start = line.find("$$").unwrap();
    let token = "api/ADR:7";
    let cursor = start + 2 + token.len();
    assert!(!can_replace_trigger_at(&path, line, start, token).expect("trigger decision"));
    assert!(
        on_type_line_edits(&path, line, 0, cursor, &[])
            .expect("on-type edits")
            .is_empty()
    );
}
