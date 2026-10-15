//! Real-server regression: an excluded stub target is broken at the stub with
//! error severity (§FS-declarations.stubs.target.3, §FS-lsp.1.1).

mod support;

use serde_json::json;
use std::fs;
use support::*;

#[test]
fn a_stub_to_an_excluded_target_errors_at_the_stub() {
    let root = test_root("excluded-stub-target");
    fs::write(
        root.join("grund.toml"),
        "grund_config_version = 1\n\n[reference]\nstrict = false\n\n[id]\nformat = \"{kind}-{slug}\"\n\n\
         [[kinds]]\nkind = \"FS\"\nfolder = \"docs/fs\"\nindex = false\n\n\
         [scan]\ninclude = [\"docs\", \"notes\"]\nexclude = [\"gen\"]\n",
    )
    .expect("write config");
    fs::create_dir_all(root.join("docs/fs")).expect("create spec dir");
    fs::create_dir_all(root.join("notes/gen")).expect("create target dir");
    fs::write(root.join("notes/gen/a.md"), "# FS-x: X\n\nLead.\n").expect("write target");
    let stub = root.join("docs/fs/x.md");
    fs::write(&stub, "# FS-x: [x](../../notes/gen/a.md)\n").expect("write stub");
    let (mut child, mut stdin, receiver) = start_server(&root);

    let diagnostics = recv_diagnostics(&receiver, &mut child, "x.md");
    let warning = diagnostics
        .iter()
        .find(|diagnostic| diagnostic["code"].as_str() == Some("broken-stub"))
        .unwrap_or_else(|| panic!("broken-stub diagnostic: {diagnostics:?}"));
    assert_eq!(warning["severity"], json!(1), "error severity");
    assert_eq!(warning["range"]["start"]["line"], json!(0), "at the stub");
    assert!(
        warning["message"]
            .as_str()
            .is_some_and(|message| message.starts_with(
                "stub link target ../../notes/gen/a.md is not scanned: \
                 [scan] exclude names gen; this became an error in grund 0.19.0 — "
            )),
        "{warning}"
    );
    assert!(
        diagnostics
            .iter()
            .all(|d| d["code"] != "excluded-stub-target")
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
