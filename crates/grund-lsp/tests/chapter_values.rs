//! A chapter-declared value root is an ordinary named section to the editor:
//! the diagnostic equals the CLI finding, the root hover is the whole heading
//! because there are no marker bytes to trim, and go-to-definition lands on the
//! component heading (§FS-lsp.1.2.1, §FS-lsp.1.3.5, §FS-values.2.5,
//! §FS-values.7).

mod support;

use serde_json::json;
use std::fs;
use support::*;

#[test]
fn chapter_value_ranges_navigation_and_diagnostics_match_the_cli() {
    let root = test_root("chapter-values");
    fs::write(
        root.join("grund.toml"),
        "grund_config_version = 1\n\n\
         [reference]\nstrict = true\n\n\
         [id]\nnamed_sections = true\n\n\
         [[kinds]]\nkind = \"AR\"\nfolder = \"docs\"\nindex = false\nvalue_chapter = \"values\"\n\n\
         [scan]\ninclude = [\"docs\"]\n",
    )
    .expect("write config");
    let document = root.join("docs/power.md");
    let root_heading = "### values.aux-voltage: Auxiliary supply voltage";
    fs::write(
        &document,
        format!(
            "# AR-004-value-probe: Auxiliary power\n\n\
             ## values: Values\n\
             {root_heading}\n\
             #### values.aux-voltage.1: 24\n\
             ## 2. Use\n\
             Bound: `48` (§AR-004-value-probe.values.aux-voltage.1)\n"
        ),
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
    let diagnostics = recv_diagnostics(&receiver, &mut child, "power.md");
    let mismatch = diagnostics
        .iter()
        .find(|diagnostic| diagnostic["code"].as_str() == Some("value-mismatch"))
        .expect("LSP mismatch");
    assert_eq!(
        mismatch["message"].as_str(),
        Some(cli_mismatch.message.as_str())
    );
    assert_eq!(
        mismatch["range"]["start"]["line"].as_u64(),
        cli_mismatch.line.map(|line| line as u64 - 1)
    );

    let uri = file_uri(&document);
    // The root heading has no marker bytes, so its title range is the whole
    // heading rather than a trimmed prefix (§FS-values.2.5).
    let title = hover_result(&mut stdin, &receiver, &mut child, 2, &uri, 3, 8);
    assert_eq!(title["range"]["start"]["character"].as_u64(), Some(4));
    assert_eq!(
        title["range"]["end"]["character"].as_u64(),
        Some(root_heading.len() as u64)
    );

    send_message(
        &mut stdin,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "textDocument/definition",
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": 6, "character": 24 }
            }
        }),
    );
    let definition = recv_response_or_panic(&receiver, &mut child, 3);
    let links = definition["result"].as_array().expect("definition links");
    assert!(
        links.iter().any(|link| {
            link["targetUri"].as_str() == Some(file_uri(&document).as_str())
                && link["targetSelectionRange"]["start"]["line"].as_u64() == Some(4)
        }),
        "binding navigation must land on the existing component section: {links:?}"
    );

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
