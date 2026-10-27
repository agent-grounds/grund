//! A front-matter coordinate in the editor (§FS-fields.front-matter.surfaces):
//! its finding is the CLI's at the key's line, a citation of a key navigates to
//! that line, and its hover is the key's slice (§FS-lsp.1.1.2, §FS-lsp.1.2.1).

mod support;

use serde_json::json;
use std::fs;
use support::*;

#[test]
fn a_front_matter_key_resolves_navigates_and_hovers_as_the_cli_shows_it() {
    let root = test_root("front-matter-field");
    fs::write(
        root.join("grund.toml"),
        "grund_config_version = 2\nproject_name = \"madr\"\n\n\
         [schema.kinds.ADR]\nfolders = [\"docs/adr\"]\nindex = false\n\n\
         [schema.kinds.ADR.fields.front-matter]\n\n\
         [schema.kinds.ADR.fields.front-matter.keys.status]\n\
         one_of = [\"proposed\", \"accepted\"]\n\n\
         [schema.kinds.page]\ncitable = false\nfiles = [\"README.md\"]\n",
    )
    .expect("write config");
    let decision = root.join("docs/adr/cache.md");
    fs::create_dir_all(decision.parent().expect("adr folder")).expect("create adr folder");
    fs::write(
        &decision,
        "---\nstatus: draft\ndate: 2026-02-03\n---\n\n# ADR-cache: Cache\n\nText.\n",
    )
    .expect("write decision");
    let readme = root.join("README.md");
    fs::write(
        &readme,
        "# Readme\n\nThe status is \u{a7}ADR-cache.front-matter.status.\n",
    )
    .expect("write readme");

    let cli = grund_core::check(&root).expect("CLI engine report");
    let invalid = cli
        .errors
        .iter()
        .find(|error| error.code == "invalid-field-value")
        .expect("CLI invalid-field-value");
    assert_eq!(invalid.line, Some(2));

    let (mut child, mut stdin, receiver) = start_server_with_capabilities(
        &root,
        json!({ "textDocument": { "definition": { "linkSupport": true } } }),
    );
    // §FS-lsp.1.1.2: the CLI's finding, at the key's line.
    let diagnostics = recv_diagnostics(&receiver, &mut child, "cache.md");
    let found = diagnostics
        .iter()
        .find(|diagnostic| diagnostic["code"].as_str() == Some("invalid-field-value"))
        .unwrap_or_else(|| panic!("front-matter diagnostic: {diagnostics:?}"));
    assert_eq!(found["message"].as_str(), Some(invalid.message.as_str()));
    assert_eq!(found["range"]["start"]["line"].as_u64(), Some(1));

    let uri = file_uri(&readme);
    send_message(
        &mut stdin,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/definition",
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": 2, "character": 20 }
            }
        }),
    );
    let definition = recv_response_or_panic(&receiver, &mut child, 2);
    let links = definition["result"].as_array().expect("definition links");
    assert!(
        links.iter().any(|link| {
            link["targetUri"].as_str() == Some(file_uri(&decision).as_str())
                && link["targetSelectionRange"]["start"]["line"].as_u64() == Some(1)
        }),
        "a key's citation must land on the key's line: {links:?}"
    );

    // §FS-lsp.1.2.1: the hover is what `show` prints, the key's slice.
    let hover = hover_result(&mut stdin, &receiver, &mut child, 3, &uri, 2, 20);
    let body = hover["contents"]["value"]
        .as_str()
        .unwrap_or_else(|| panic!("front-matter hover body: {hover:?}"));
    assert!(body.contains("status: draft"), "{body}");

    send_message(
        &mut stdin,
        json!({ "jsonrpc": "2.0", "id": 4, "method": "shutdown", "params": null }),
    );
    recv_response_or_panic(&receiver, &mut child, 4);
    send_message(
        &mut stdin,
        json!({ "jsonrpc": "2.0", "method": "exit", "params": null }),
    );
    drop(stdin);
    wait_for_exit(&mut child);
    let _ = fs::remove_dir_all(root);
}
