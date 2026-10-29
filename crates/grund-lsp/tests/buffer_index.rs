//! Real-server regression for the index look under an unsaved index buffer: the
//! look reads the index as a save would write it (§FS-scan.text.buffer), and
//! whether a line is a declaration heading is read off the record of that same
//! text (§AR-scanner.2.8.5), so the buffer gives the missing-entry message a save
//! would (§FS-check.3.18.5.1).

mod support;

use serde_json::{Value, json};
use std::fs;
use std::sync::mpsc;
use std::time::Duration;
use support::*;

/// Every diagnostics publication for `uri_suffix` until the server falls quiet,
/// the last one returned: the answer to the last change the server took.
fn settled_diagnostics(receiver: &mpsc::Receiver<Value>, uri_suffix: &str) -> Vec<Value> {
    let mut last = Vec::new();
    while let Ok(message) = receiver.recv_timeout(Duration::from_millis(1500)) {
        if message.get("method").and_then(Value::as_str) == Some("textDocument/publishDiagnostics")
            && message["params"]["uri"]
                .as_str()
                .is_some_and(|uri| uri.contains(uri_suffix))
        {
            last = message["params"]["diagnostics"]
                .as_array()
                .cloned()
                .unwrap_or_default();
        }
    }
    last
}

/// The missing-entry messages among `diagnostics` (§FS-check.3.18.5.1).
fn messages(diagnostics: &[Value]) -> Vec<&str> {
    diagnostics
        .iter()
        .filter(|diagnostic| diagnostic["code"].as_str() == Some("missing-index-entry"))
        .filter_map(|diagnostic| diagnostic["message"].as_str())
        .collect()
}

const NOT_LISTED: &str = "FS-002-session is not listed in docs/fs/README.md: an entry is a \
     `§`-marked Markdown link to the declaration; write \
     `[\u{a7}FS-002-session](FS-002-session.md#fs-002-session-user-session)` in docs/fs/README.md \
     — became an error in grund 0.13.0";

/// The missing-entry messages on FS-002-session with the index `saved` on disk,
/// then with the index open in a buffer holding `buffer`.
fn missing_entry_under_buffer(name: &str, saved: &str, buffer: &str) -> (Vec<String>, Vec<String>) {
    let root = test_root(name);
    fs::write(
        root.join("grund.toml"),
        "grund_config_version = 1\n\n[[kinds]]\nkind = \"FS\"\nfolder = \"docs/fs\"\n\n\
         [scan]\ninclude = [\"docs\"]\n",
    )
    .expect("write config");
    fs::create_dir_all(root.join("docs/fs")).expect("create docs dir");
    fs::write(
        root.join("docs/fs/FS-001-login.md"),
        "# FS-001-login: User login\n\nCreates a session.\n",
    )
    .expect("write login");
    fs::write(
        root.join("docs/fs/FS-002-session.md"),
        "# FS-002-session: User session\n\nAfter login.\n",
    )
    .expect("write session");
    let readme = root.join("docs/fs/README.md");
    fs::write(&readme, saved).expect("write index");
    let (mut child, mut stdin, receiver) = start_server(&root);
    let before = recv_diagnostics(&receiver, &mut child, "FS-002-session.md");

    send_message(
        &mut stdin,
        json!({"jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {
            "textDocument": {"uri": file_uri(&readme), "languageId": "markdown",
                             "version": 1, "text": saved}}}),
    );
    send_message(
        &mut stdin,
        json!({"jsonrpc": "2.0", "method": "textDocument/didChange", "params": {
            "textDocument": {"uri": file_uri(&readme), "version": 2},
            "contentChanges": [{"text": buffer}]}}),
    );
    let after = settled_diagnostics(&receiver, "FS-002-session.md");

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
    let owned = |diagnostics: &[Value]| {
        messages(diagnostics)
            .into_iter()
            .map(String::from)
            .collect()
    };
    (owned(&before), owned(&after))
}

/// The mention look: the buffer's line 1 is prose and its heading moves to line
/// 3, where the buffer's record holds it, so the heading still mentions nothing.
#[test]
fn shifting_an_unsaved_index_buffer_moves_no_mention() {
    // A declaration heading declares its ID rather than mentioning another
    // (§FS-fmt.6.4), so FS-002-session is not listed in the saved index.
    let saved = "# FS-000-index: Index of FS-002-session\n\n\
                 - [\u{a7}FS-001-login](FS-001-login.md#fs-001-login-user-login)\n";
    let buffer = format!("Intro line.\n\n{saved}");
    let (before, after) = missing_entry_under_buffer("buffer-index-mention", saved, &buffer);
    assert_eq!(before, [NOT_LISTED], "saved");
    assert_eq!(after, [NOT_LISTED], "shifted buffer");
}

/// The entry pass: the buffer's entry citation sits on line 1, where the saved
/// text holds a declaration heading. The heading is asked of the buffer's record,
/// not the saved one, so the buffer lists FS-002-session as saving it would
/// (§FS-check.3.17.4).
#[test]
fn an_entry_in_an_unsaved_index_buffer_lists_the_id() {
    let link = "[\u{a7}FS-002-session](FS-002-session.md#fs-002-session-user-session)";
    let saved = format!(
        "# FS-000-index: Index of {link}\n\n\
         - [\u{a7}FS-001-login](FS-001-login.md#fs-001-login-user-login)\n"
    );
    let buffer = format!("- {link}\n{saved}");
    let (before, after) = missing_entry_under_buffer("buffer-index-entry", &saved, &buffer);
    assert_eq!(before, [NOT_LISTED], "saved");
    assert!(after.is_empty(), "buffer with the entry: {after:?}");
}
