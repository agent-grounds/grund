//! Real-server regression for the stub-shape warning: a heading of the stub shape
//! that grund 0.19.0 reads differently is shown at the heading with warning
//! severity (§FS-declarations.checks.stub-shape, §FS-lsp.1.1).

mod support;

use serde_json::json;
use std::fs;
use support::*;

#[test]
fn a_link_titled_heading_in_a_home_without_docs_warns_at_the_heading() {
    let root = test_root("stub-shape");
    fs::write(
        root.join("grund.toml"),
        "grund_config_version = 1\n\n[reference]\nstrict = false\n\n\
         [[kinds]]\nkind = \"FS\"\nfolder = \"spec/fs\"\nindex = false\n",
    )
    .expect("write config");
    fs::create_dir_all(root.join("spec/fs")).expect("create spec dir");
    fs::create_dir_all(root.join("src")).expect("create src dir");
    fs::write(
        root.join("src/x.rs"),
        "/// FS-x: X\n///\n/// Lead.\npub fn x() {}\n",
    )
    .expect("write target");
    let spec = root.join("spec/fs/x.md");
    fs::write(&spec, "# FS-x: [x](../../src/x.rs)\n").expect("write heading");
    let (mut child, mut stdin, receiver) = start_server(&root);

    let diagnostics = recv_diagnostics(&receiver, &mut child, "x.md");
    let warning = diagnostics
        .iter()
        .find(|diagnostic| diagnostic["code"].as_str() == Some("stub-shape"))
        .unwrap_or_else(|| panic!("stub-shape diagnostic: {diagnostics:?}"));
    assert_eq!(warning["severity"], json!(2), "warning severity");
    assert_eq!(
        warning["range"]["start"]["line"],
        json!(0),
        "at the heading"
    );
    assert!(
        warning["message"]
            .as_str()
            .is_some_and(|message| message.starts_with(
                "FS-x is read as a declaration titled with a link, and that reading is removed \
                 in grund 0.19.0, which reads it as a stub to ../../src/x.rs"
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
