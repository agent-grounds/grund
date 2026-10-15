//! Real-server regression for the stub-shape check, which stays catalogued and
//! never fires from grund 0.19.0: a heading whose title is one link, in a kind's
//! home with no directory named `docs` on its path, is a stub, and the editor shows
//! nothing at its heading (§FS-declarations.stubs.shape, §FS-declarations.checks.stub-shape,
//! §FS-lsp.1.1).

mod support;

use serde_json::json;
use std::fs;
use support::*;

#[test]
fn a_link_titled_heading_in_a_home_without_docs_is_a_stub_and_shows_nothing() {
    let root = test_root("stub-shape");
    fs::write(
        root.join("grund.toml"),
        "grund_config_version = 1\n\n[reference]\nstrict = true\n\n\
         [id]\nformat = \"{kind}-{slug}\"\n\n[[kinds]]\nkind = \"FS\"\nfolder = \"spec/fs\"\nindex = false\n",
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
    // The dangling citation is there so the file has one diagnostic to publish.
    fs::write(&spec, "# FS-x: [x](../../src/x.rs)\n\nSee \u{a7}FS-nope.\n").expect("write stub");
    let (mut child, mut stdin, receiver) = start_server(&root);

    let diagnostics = recv_diagnostics(&receiver, &mut child, "x.md");
    let codes: Vec<&str> = diagnostics
        .iter()
        .filter_map(|diagnostic| diagnostic["code"].as_str())
        .collect();
    assert_eq!(
        codes,
        ["dangling"],
        "a stub in a kind's home draws no stub-shape, duplicate or unused: {diagnostics:?}"
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
