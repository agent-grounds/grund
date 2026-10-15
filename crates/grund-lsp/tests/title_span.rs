//! Real-server coverage for a section title's span on a C-family heading line:
//! it ends where the `grund <ID> --toc` title ends, so the closing `*/` is left
//! out as comment envelope (§FS-lsp.1.3.4, §FS-languages.comment-wrapper.headings).

mod support;

use serde_json::json;
use std::fs;
use support::*;

/// ` * ## 1. One */` is line 4 of `src/a.c`; `1. One` runs from UTF-16 column
/// 6 to 12, and the ` */` after it is not the title.
const HEADING_LINE: i64 = 4;
const TITLE_START: i64 = 6;
const TITLE_END: i64 = 12;

#[test]
fn a_c_family_section_title_span_ends_before_the_closing_comment() {
    let root = test_root("title-span-block-comment");
    fs::write(
        root.join("grund.toml"),
        "grund_config_version = 1\n\n[reference]\nstrict = true\n\n\
         [id]\nformat = \"{kind}-{slug}\"\n\n\
         [[kinds]]\nkind = \"FS\"\nfolder = \"docs/fs\"\nindex = false\n\n\
         [scan]\ninclude = [\"docs\", \"src\"]\n",
    )
    .expect("write config");
    fs::create_dir_all(root.join("docs/fs")).expect("create spec dir");
    fs::create_dir_all(root.join("src")).expect("create src dir");
    let source = root.join("src/a.c");
    fs::write(
        &source,
        "/**\n * FS-y: The y spec\n *\n * The lead.\n * ## 1. One */\nint y(void) { return 0; }\n",
    )
    .expect("write source");
    let usage = root.join("docs/use.md");
    fs::write(&usage, "# Use\n\nUses \u{a7}FS-y.1 here.\n").expect("write usage");
    let (mut child, mut stdin, receiver) = start_server(&root);

    send_message(
        &mut stdin,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/definition",
            "params": {
                "textDocument": { "uri": file_uri(&usage) },
                "position": { "line": 2, "character": 8 }
            }
        }),
    );
    let definition = recv_response_or_panic(&receiver, &mut child, 2);
    let links = definition["result"].as_array().cloned().unwrap_or_default();
    let link = links
        .iter()
        .find(|link| link["targetUri"].as_str() == Some(file_uri(&source).as_str()))
        .unwrap_or_else(|| panic!("a link into src/a.c: {definition:?}"));
    let range = &link["targetSelectionRange"];
    assert_eq!(range["start"]["line"], json!(HEADING_LINE), "{link}");
    assert_eq!(range["start"]["character"], json!(TITLE_START), "{link}");
    assert_eq!(range["end"]["line"], json!(HEADING_LINE), "{link}");
    assert_eq!(
        range["end"]["character"],
        json!(TITLE_END),
        "the span ends where the --toc title `One` ends, before ` */`: {link}"
    );

    send_message(
        &mut stdin,
        json!({"jsonrpc": "2.0", "id": 3, "method": "shutdown", "params": null}),
    );
    recv_response_or_panic(&receiver, &mut child, 3);
    send_message(
        &mut stdin,
        json!({"jsonrpc": "2.0", "method": "exit", "params": null}),
    );
    drop(stdin);
    wait_for_exit(&mut child);
    let _ = fs::remove_dir_all(root);
}
