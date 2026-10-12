//! Real-server proof that a declaration on a bare line inside a `/* … */`
//! block is the editor's too: go-to-definition on its citation lands on the
//! bare line, as it would on a ` * ` line (§FS-declarations.line.block-comment).

mod support;

use serde_json::json;
use std::fs;
use support::*;

#[test]
fn definition_reaches_a_declaration_on_a_bare_block_comment_line() {
    let root = test_root("bare-block-comment-line");
    fs::write(
        root.join("grund.toml"),
        "grund_config_version = 1\n\n\
         [reference]\nstrict = true\n\n\
         [id]\nformat = \"{kind}-{slug}\"\n\n\
         [[kinds]]\nkind = \"FS\"\nfolder = \"docs/fs\"\nindex = false\n",
    )
    .expect("write config");
    fs::create_dir_all(root.join("src")).expect("create src");
    fs::create_dir_all(root.join("docs/fs")).expect("create docs");
    let source = root.join("src/lib.rs");
    fs::write(
        &source,
        "/**\nFS-b: Beta\n\nBeta body.\n*/\npub struct B;\n",
    )
    .expect("write source");
    let usage = root.join("docs/use.md");
    fs::write(&usage, "See \u{a7}FS-b.\n").expect("write usage");

    let (mut child, mut stdin, receiver) = start_server(&root);
    send_message(
        &mut stdin,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/definition",
            "params": {
                "textDocument": { "uri": file_uri(&usage) },
                "position": { "line": 0, "character": 6 }
            }
        }),
    );
    let definition = recv_response_or_panic(&receiver, &mut child, 2);

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

    let source_uri = file_uri(&source);
    let lands_on_bare_line = definition["result"].as_array().is_some_and(|links| {
        links.iter().any(|link| {
            link["targetUri"].as_str() == Some(source_uri.as_str())
                && link["targetSelectionRange"]["start"]["line"].as_i64() == Some(1)
        })
    });
    assert!(
        lands_on_bare_line,
        "definition of FS-b did not land on src/lib.rs:2: {definition:?}"
    );
}
