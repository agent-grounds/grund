//! Real-server coverage for local-section diagnostic parity and graph navigation
//! (§FS-lsp.1.1, §FS-lsp.1.3, §AR-lsp.5).

mod support;

use serde_json::{Value, json};
use std::fs;
use support::*;

fn fixture(name: &str) -> (std::path::PathBuf, std::path::PathBuf, std::path::PathBuf) {
    let root = test_root(name);
    fs::write(
        root.join("grund.toml"),
        concat!(
            "grund_config_version = 1\n\n",
            "[reference]\nstrict = true\n\n",
            "[id]\nformat = \"{kind}-{slug}\"\nnamed_sections = true\n\n",
            "[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n\n",
            "[scan]\ninclude = [\"docs\"]\nextensions = [\"md\"]\n",
        ),
    )
    .expect("write config");
    let spec = root.join("docs/FS-a.md");
    fs::write(
        &spec,
        concat!(
            "# FS-a: A thing\n\n",
            "Valid \u{a7}2.\n",
            "Missing \u{a7}9.9.\n",
            "Unsupported \u{a7}2.goals and \u{a7}2abc.\n",
            "Malformed \u{a7}2..1 and \u{a7}2... and \u{a7}2..goals.\n",
            "Full valid \u{a7}FS-a.2.\n",
            "Full missing \u{a7}FS-a.9.\n\n",
            "## 2. Target\n\n",
            "### 2.1 Child\n",
        ),
    )
    .expect("write spec");
    let outside = root.join("docs/outside.md");
    fs::write(&outside, "# Notes\n\nOwnerless \u{a7}2.\n").expect("write ownerless file");
    (root, spec, outside)
}

fn request(
    stdin: &mut std::process::ChildStdin,
    receiver: &std::sync::mpsc::Receiver<Value>,
    child: &mut std::process::Child,
    id: i64,
    method: &str,
    uri: &str,
    line: u64,
    character: u64,
) -> Value {
    let mut params = json!({
        "textDocument": { "uri": uri },
        "position": { "line": line, "character": character }
    });
    if method == "textDocument/references" {
        params["context"] = json!({ "includeDeclaration": true });
    }
    send_message(
        stdin,
        json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }),
    );
    recv_response_or_panic(receiver, child, id)["result"].clone()
}

fn stop(
    mut child: std::process::Child,
    mut stdin: std::process::ChildStdin,
    receiver: &std::sync::mpsc::Receiver<Value>,
) {
    send_message(
        &mut stdin,
        json!({"jsonrpc": "2.0", "id": 90, "method": "shutdown", "params": null}),
    );
    recv_response_or_panic(receiver, &mut child, 90);
    send_message(
        &mut stdin,
        json!({"jsonrpc": "2.0", "method": "exit", "params": null}),
    );
    drop(stdin);
    wait_for_exit(&mut child);
}

#[test]
fn local_section_diagnostics_keep_cli_messages_and_exact_token_ranges() {
    let (root, _, _) = fixture("local-section-diagnostics");
    let (mut child, mut stdin, receiver) = start_server(&root);
    let diagnostics = recv_diagnostics(&receiver, &mut child, "FS-a.md");

    let expected = [
        (
            "local-section-citation",
            "local section citation \u{a7}2; write \u{a7}FS-a.2",
            2,
            6,
            8,
        ),
        (
            "local-section-citation",
            "local section citation \u{a7}9.9; write \u{a7}FS-a.9.9",
            3,
            8,
            12,
        ),
        ("missing-section", "missing section FS-a.9.9", 3, 8, 12),
        (
            "local-section-citation",
            "unsupported local section citation \u{a7}2.goals; write a full citation or <§>2.goals to show the shape without citing it",
            4,
            12,
            20,
        ),
        (
            "local-section-citation",
            "unsupported local section citation \u{a7}2abc; write a full citation or <§>2abc to show the shape without citing it",
            4,
            25,
            30,
        ),
        (
            "local-section-citation",
            "unsupported local section citation \u{a7}2..1; write a full citation or <§>2..1 to show the shape without citing it",
            5,
            10,
            15,
        ),
        (
            "local-section-citation",
            "unsupported local section citation \u{a7}2...; write a full citation or <§>2... to show the shape without citing it",
            5,
            20,
            25,
        ),
        (
            "local-section-citation",
            "unsupported local section citation \u{a7}2..goals; write a full citation or <§>2..goals to show the shape without citing it",
            5,
            30,
            39,
        ),
    ];
    for (code, message, line, start, end) in expected {
        let diagnostic = diagnostics
            .iter()
            .find(|diagnostic| diagnostic["message"].as_str() == Some(message))
            .unwrap_or_else(|| panic!("missing {message:?} in {diagnostics:?}"));
        assert_eq!(diagnostic["code"], json!(code));
        assert_eq!(
            diagnostic["range"],
            json!({
                "start": { "line": line, "character": start },
                "end": { "line": line, "character": end }
            })
        );
    }
    for character in [11, 21, 31] {
        let definition = request(
            &mut stdin,
            &receiver,
            &mut child,
            20 + character as i64,
            "textDocument/definition",
            &file_uri(&root.join("docs/FS-a.md")),
            5,
            character,
        );
        assert_eq!(definition, json!(null));
    }
    stop(child, stdin, &receiver);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn ownerless_local_section_is_diagnosed_without_a_navigation_target() {
    let (root, _, outside) = fixture("ownerless-local-section-diagnostic");
    let (mut child, mut stdin, receiver) = start_server(&root);
    let diagnostics = recv_diagnostics(&receiver, &mut child, "outside.md");
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic["code"].as_str() == Some("local-section-citation"))
        .unwrap_or_else(|| panic!("ownerless local diagnostic: {diagnostics:?}"));
    assert_eq!(
        diagnostic["message"],
        json!(
            "local section citation \u{a7}2 has no enclosing declaration; write a full citation or <§>2 to show the shape without citing it"
        )
    );
    assert_eq!(
        diagnostic["range"],
        json!({
            "start": { "line": 2, "character": 10 },
            "end": { "line": 2, "character": 12 }
        })
    );
    let definition = request(
        &mut stdin,
        &receiver,
        &mut child,
        2,
        "textDocument/definition",
        &file_uri(&outside),
        2,
        11,
    );
    assert_eq!(definition, json!(null));
    stop(child, stdin, &receiver);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn owned_local_sections_navigate_and_unresolved_forms_never_gain_targets() {
    let (root, spec, outside) = fixture("local-section-navigation");
    let (mut child, mut stdin, receiver) = start_server(&root);
    let spec_uri = file_uri(&spec);

    let definition = request(
        &mut stdin,
        &receiver,
        &mut child,
        2,
        "textDocument/definition",
        &spec_uri,
        2,
        7,
    );
    let links = definition
        .as_array()
        .unwrap_or_else(|| panic!("local definition links: {definition:?}"));
    assert!(links.iter().any(|link| {
        link["targetSelectionRange"]["start"]["line"].as_u64() == Some(9)
            && link["originSelectionRange"]["start"]["character"].as_u64() == Some(6)
            && link["originSelectionRange"]["end"]["character"].as_u64() == Some(8)
    }));

    let references = request(
        &mut stdin,
        &receiver,
        &mut child,
        3,
        "textDocument/references",
        &spec_uri,
        9,
        5,
    );
    assert_eq!(
        references
            .as_array()
            .expect("reference locations")
            .iter()
            .map(|location| location["range"]["start"]["line"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        [9, 2, 6]
    );

    let highlights = request(
        &mut stdin,
        &receiver,
        &mut child,
        4,
        "textDocument/documentHighlight",
        &spec_uri,
        2,
        7,
    );
    assert_eq!(highlights.as_array().expect("highlights").len(), 3);

    let missing = request(
        &mut stdin,
        &receiver,
        &mut child,
        5,
        "textDocument/definition",
        &spec_uri,
        3,
        10,
    );
    assert_eq!(missing, json!(null));
    let ownerless = request(
        &mut stdin,
        &receiver,
        &mut child,
        6,
        "textDocument/definition",
        &file_uri(&outside),
        2,
        11,
    );
    assert_eq!(ownerless, json!(null));

    stop(child, stdin, &receiver);
    let _ = fs::remove_dir_all(root);
}
