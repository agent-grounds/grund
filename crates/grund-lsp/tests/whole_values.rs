//! A binding aimed at a whole value declaration navigates to the declaration
//! itself — the Markdown heading or the JSON key — and its mismatch is the CLI
//! finding (§FS-lsp.1.1.2, §FS-lsp.1.3.5, §FS-values.3.1.2, §FS-values.7).

mod support;

use serde_json::json;
use std::fs;
use support::*;

#[test]
fn whole_value_bindings_navigate_to_the_declaration_and_match_the_cli() {
    let root = test_root("whole-values");
    fs::write(
        root.join("grund.toml"),
        "grund_config_version = 1\n\n\
         [reference]\nstrict = true\n\n\
         [id]\nformat = \"{kind}-{slug}\"\nslug_pattern = \"[a-z][a-z0-9-]*\"\n\n\
         [[kinds]]\nkind = \"CONST\"\nfolder = \"values\"\nindex = false\nvalues = true\n\n\
         [scan]\ninclude = [\"docs\"]\nextensions = [\"md\"]\n",
    )
    .expect("write config");
    let price = root.join("values/price.md");
    fs::create_dir_all(price.parent().expect("values folder")).expect("create values folder");
    fs::write(&price, "# CONST-price: Price\n## 1. 1200\n## 2. USD\n").expect("write price");
    let runtime = root.join("values/runtime.json");
    fs::write(&runtime, "{\n  \"CONST-discount\": [0.25, \"%\"]\n}\n").expect("write json");
    let document = root.join("docs/offer.md");
    fs::write(
        &document,
        "Price: `1200.0 USD` (§CONST-price).\n\n\
         Discount: `0.250 %` (§CONST-discount).\n\n\
         Wrong: `1200.0 EUR` (§CONST-price).\n",
    )
    .expect("write document");

    let cli = grund_core::check(&root).expect("CLI engine report");
    let cli_mismatch = cli
        .errors
        .iter()
        .find(|error| error.code == "value-mismatch")
        .expect("CLI mismatch");

    let (mut child, mut stdin, receiver) = start_server_with_capabilities(
        &root,
        json!({ "textDocument": { "definition": { "linkSupport": true } } }),
    );
    let diagnostics = recv_diagnostics(&receiver, &mut child, "offer.md");
    let mismatch = diagnostics
        .iter()
        .find(|diagnostic| diagnostic["code"].as_str() == Some("value-mismatch"))
        .expect("LSP mismatch");
    assert_eq!(
        mismatch["message"].as_str(),
        Some(cli_mismatch.message.as_str())
    );
    assert_eq!(mismatch["range"]["start"]["line"].as_u64(), Some(4));

    let uri = file_uri(&document);
    for (id, line, character, target, target_line) in
        [(2, 0, 25, &price, 0), (3, 2, 27, &runtime, 1)]
    {
        send_message(
            &mut stdin,
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "method": "textDocument/definition",
                "params": {
                    "textDocument": { "uri": uri },
                    "position": { "line": line, "character": character }
                }
            }),
        );
        let definition = recv_response_or_panic(&receiver, &mut child, id);
        let links = definition["result"].as_array().expect("definition links");
        assert!(
            links.iter().any(|link| {
                link["targetUri"].as_str() == Some(file_uri(target).as_str())
                    && link["targetSelectionRange"]["start"]["line"].as_u64() == Some(target_line)
            }),
            "a root-aimed binding must land on the whole declaration: {links:?}"
        );
    }

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
