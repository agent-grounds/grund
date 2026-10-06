//! Shared scan navigation (§AR-scanner.2.3, §FS-lsp.1.3, §FS-lsp.4).

#[path = "lsp_completion_support.rs"]
mod support;

use serde_json::json;
use support::{Fixture, Session, file_uri, position};

/// §FS-check.1.1.10: a word-character marker creates the same editor edge as §.
#[test]
fn word_character_marker_definition_after_emoji_uses_utf16_origin() {
    for marker in ["\u{a7}", "_"] {
        let f = Fixture::new();
        f.write(
            "grund.toml",
            &format!(
                "grund_config_version = 1\n[id]\nformat = \"{{kind}}_{{slug}}\"\n\
             [reference]\nmarker = \"{marker}\"\n[fmt.cross_refs]\nenabled = false\n"
            ),
        );
        let target = f.write(
            "docs/functional-spec/FS-login.md",
            "# FS_login: Login\n\nLead.\n",
        );
        let prefix = "//! 😀 ";
        let text = format!("{prefix}{marker}FS_login\n");
        let source = f.write("src/lib.rs", &text);
        let mut session = Session::new(&f.0);
        session.text(&source, &text);
        // The marker starts at UTF-16 character 7, raw byte 9; the cursor is valid
        // on the marker and inside the ID, never inside the emoji's surrogate pair.
        for offset in [prefix.len() + marker.len() + "FS_lo".len(), prefix.len()] {
            let params = position(&source, &text, 0, offset);
            let response = session.request("textDocument/definition", params.clone());
            assert!(
                response.get("error").is_none(),
                "request rejected: {response}"
            );
            let targets = response["result"].as_array().unwrap_or_else(|| {
                panic!(
                    "marker {marker:?}, position {}: expected one definition, got {response}",
                    params["position"]
                )
            });
            assert_eq!(targets.len(), 1, "marker {marker:?}: {response}");
            assert_eq!(
                targets[0]["targetUri"],
                file_uri(&target.canonicalize().unwrap())
            );
            assert_eq!(targets[0]["targetSelectionRange"]["start"]["line"], 0);
            assert_eq!(
                targets[0]["originSelectionRange"],
                json!({
                    "start":{"line":0,"character":prefix.encode_utf16().count()},
                    "end":{"line":0,"character":text.trim_end().encode_utf16().count()}
                })
            );
        }
    }
}
