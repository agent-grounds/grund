//! Real-server proof that a relative heading in a doc-comment declaration body, which
//! grund 0.19.0 reads as a section, is shown at its line with warning severity
//! (§FS-declarations.checks.parse-moves.3, §FS-lsp.1.1).

mod support;

use serde_json::json;
use std::fs;
use support::*;

#[test]
fn a_relative_heading_in_a_doc_comment_warns_at_its_line() {
    let root = test_root("parse-moves-relative-heading");
    fs::write(
        root.join("grund.toml"),
        "grund_config_version = 1\n\n[reference]\nstrict = true\n\n\
         [id]\nformat = \"{kind}-{slug}\"\nnamed_sections = true\n\n\
         [[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n",
    )
    .expect("write config");
    fs::create_dir_all(root.join("docs")).expect("create docs dir");
    fs::create_dir_all(root.join("src")).expect("create src dir");
    fs::write(root.join("docs/use.md"), "Uses \u{a7}FS-thing.\n").expect("write usage");
    fs::write(
        root.join("src/lib.rs"),
        "/// FS-thing: A thing\n///\n/// ## format: Formatting\n///\n\
         /// ### .1: Username\n///\n/// Username text.\npub struct Thing;\n",
    )
    .expect("write source");
    let (mut child, mut stdin, receiver) = start_server(&root);

    let diagnostics = recv_diagnostics(&receiver, &mut child, "lib.rs");
    let warning = diagnostics
        .iter()
        .find(|diagnostic| diagnostic["code"].as_str() == Some("parse-moves"))
        .unwrap_or_else(|| panic!("parse-moves diagnostic: {diagnostics:?}"));
    assert_eq!(warning["severity"], json!(2), "warning severity");
    assert_eq!(
        warning["range"]["start"]["line"],
        json!(4),
        "at the relative heading"
    );
    assert!(
        warning["message"].as_str().is_some_and(|message| message
            .starts_with("from grund 0.19.0 this line is the heading of FS-thing.format.1;")),
        "{warning}"
    );

    send_message(
        &mut stdin,
        json!({"jsonrpc": "2.0", "id": 2, "method": "shutdown", "params": null}),
    );
    recv_response_or_panic(&receiver, &mut child, 2);
    send_message(
        &mut stdin,
        json!({"jsonrpc": "2.0", "method": "exit", "params": null}),
    );
    drop(stdin);
    wait_for_exit(&mut child);
    let _ = fs::remove_dir_all(root);
}
