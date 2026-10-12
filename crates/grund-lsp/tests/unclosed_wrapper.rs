//! Real-server regression for the unclosed-wrapper warning: a `[§ID](` whose
//! destination never closes, with a later `)` on its line, is shown at that line
//! with warning severity (§FS-citations.checks.unclosed-wrapper, §FS-lsp.1.1).

mod support;

use serde_json::json;
use std::fs;
use support::*;

#[test]
fn an_unclosed_wrapper_before_a_later_paren_warns_at_its_line() {
    let root = test_root("unclosed-wrapper");
    fs::write(
        root.join("grund.toml"),
        "grund_config_version = 1\n\n[reference]\nstrict = true\n\n\
         [[kinds]]\nkind = \"FS\"\nfolder = \"docs/fs\"\nindex = false\n",
    )
    .expect("write config");
    fs::create_dir_all(root.join("docs/fs")).expect("create spec dir");
    fs::write(root.join("docs/fs/x.md"), "# FS-x: X\n\nLead.\n").expect("write declaration");
    fs::write(
        root.join("docs/notes.md"),
        "Notes.\n\nThen [\u{a7}FS-x](stale-dest and an aside (keep me) end.\n",
    )
    .expect("write notes");
    let (mut child, mut stdin, receiver) = start_server(&root);

    let diagnostics = recv_diagnostics(&receiver, &mut child, "notes.md");
    let warning = diagnostics
        .iter()
        .find(|diagnostic| diagnostic["code"].as_str() == Some("unclosed-wrapper"))
        .unwrap_or_else(|| panic!("unclosed-wrapper diagnostic: {diagnostics:?}"));
    assert_eq!(warning["severity"], json!(2), "warning severity");
    assert_eq!(warning["range"]["start"]["line"], json!(2), "at its line");
    assert!(
        warning["message"]
            .as_str()
            .is_some_and(|message| message.starts_with(
                "the link after FS-x never closes, so grund fmt --cross-refs leaves it as \
                 written, and from grund 0.19.0 it is not read as FS-x's link"
            )),
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
