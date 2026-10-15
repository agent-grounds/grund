//! Real-server regression for goto on a citation of a stubbed ID whose target the
//! scan does not reach: the user lands on the declaration in the target, never on
//! the stub (§FS-lsp.1.3, §FS-declarations.stubs.pairing.4).

mod support;

use serde_json::json;
use std::fs;
use support::*;

#[test]
fn goto_on_a_citation_lands_on_an_unscanned_targets_declaration() {
    let root = test_root("stub-unscanned-target-goto");
    fs::write(
        root.join("grund.toml"),
        "grund_config_version = 1\n\n[reference]\nstrict = true\n\n\
         [id]\nformat = \"{kind}-{slug}\"\n\n\
         [[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n\n\
         [scan]\ninclude = [\"docs\"]\n",
    )
    .expect("write config");
    fs::create_dir_all(root.join("docs/sub")).expect("create stub dir");
    fs::create_dir_all(root.join("docs/other")).expect("create citing dir");
    fs::create_dir_all(root.join("notes")).expect("create target dir");
    fs::write(root.join("notes/x.md"), "intro\n\n# FS-x: Ex\n\nLead.\n").expect("write target");
    fs::write(
        root.join("docs/sub/x.md"),
        "# FS-x: [x](../../notes/x.md)\n",
    )
    .expect("write stub");
    let citing = root.join("docs/other/u.md");
    // The dangling citation is there so the file has one diagnostic to publish.
    fs::write(&citing, "Uses \u{a7}FS-x and \u{a7}FS-gone.\n").expect("write citing");
    let (mut child, mut stdin, receiver) = start_server(&root);
    recv_diagnostics(&receiver, &mut child, "u.md");

    send_message(
        &mut stdin,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/definition",
            "params": {
                "textDocument": { "uri": file_uri(&citing) },
                "position": { "line": 0, "character": 8 }
            }
        }),
    );
    let definition = recv_response_or_panic(&receiver, &mut child, 2);
    let location = &definition["result"][0];
    assert_eq!(
        location["targetUri"],
        json!(file_uri(&root.join("notes/x.md"))),
        "lands in the target: {definition}"
    );
    assert_eq!(
        location["targetSelectionRange"]["start"]["line"],
        json!(2),
        "at the declaration's line: {definition}"
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
