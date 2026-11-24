//! Real-server coverage of §FS-languages.fence.language: a citation in a fence that
//! `[schema.sources] fenced` selects is diagnosed and hovered where it sits, as the same
//! citation in a `<pre>` block is (§FS-citations.recognized.contexts, §FS-lsp.1.1).
//! Fixture builders retain their temporary-root owner (§FS-repository-maintenance.4).

mod support;

use serde_json::{Value, json};
use std::fs;
use support::*;

/// A v2 config that reads the citations of a `mermaid` fence.
const SELECTED: &str = "grund_config_version = 2\nproject_name = \"demo\"\n[schema.sources]\n\
                        fenced = [\"mermaid\"]\n[schema.kinds.AR]\nfiles = [\"docs/ar.md\"]\n";

/// The same config with no `fenced` key.
const PLAIN: &str = "grund_config_version = 2\nproject_name = \"demo\"\n\
                     [schema.kinds.AR]\nfiles = [\"docs/ar.md\"]\n";

/// The issue's `README.md`: a `mermaid` fence citing one real and one dangling ID, then `<pre>`.
const ISSUE: &str = "**Demo**\n\n```mermaid\nflowchart LR\n  \
                     gateway[\"**gateway** \u{a7}AR-gateway\"] --> db[\"**db** \u{a7}AR-nonexistent\"]\n\
                     ```\n\n<pre>\ngateway  \u{a7}AR-nonexistent2\n</pre>\n";

/// [`ISSUE`] with its fence written as a `<pre>` block, line for line and column for column.
const ISSUE_PRE: &str = "**Demo**\n\n<pre>\nflowchart LR\n  \
                         gateway[\"**gateway** \u{a7}AR-gateway\"] --> db[\"**db** \u{a7}AR-nonexistent\"]\n\
                         </pre>\n\n<pre>\ngateway  \u{a7}AR-nonexistent2\n</pre>\n";

/// What the server says about `README.md` in a tree holding `config` and `readme`: each
/// diagnostic as `line:character-line:character severity code message`, and the hover on the
/// `AR-gateway` citation the fence's (or the twin's) fifth line holds.
fn observe(name: &str, config: &str, readme: &str) -> (Vec<String>, Value) {
    let root = test_root(name);
    fs::write(root.join("grund.toml"), config).expect("write config");
    fs::write(root.join("docs/ar.md"), "# AR-gateway: Gateway\n").expect("write AR");
    let readme_path = root.join("README.md");
    fs::write(&readme_path, readme).expect("write README");
    let (mut child, mut stdin, receiver) = start_server(&root);

    let diagnostics = recv_diagnostics(&receiver, &mut child, "README.md")
        .iter()
        .map(|d| {
            let range = &d["range"];
            format!(
                "{}:{}-{}:{} {} {} {}",
                range["start"]["line"],
                range["start"]["character"],
                range["end"]["line"],
                range["end"]["character"],
                d["severity"],
                d["code"],
                d["message"]
            )
        })
        .collect();
    let hover = hover_result(
        &mut stdin,
        &receiver,
        &mut child,
        2,
        &file_uri(&readme_path),
        4,
        26,
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
    (diagnostics, hover)
}

/// What the server publishes for `README.md` once both dangling citations are read: each at
/// its site, an error coded `dangling`.
const DIAGNOSED: [&str; 2] = [
    "4:52-4:67 1 \"dangling\" \"unknown reference AR-nonexistent\"",
    "8:9-8:25 1 \"dangling\" \"unknown reference AR-nonexistent2\"",
];

/// The hover on the live `AR-gateway` citation: its declaration, over the citation's span.
fn gateway_hover() -> Value {
    json!({
        "contents": {"kind": "markdown", "value": "# AR-gateway: Gateway\n"},
        "range": {
            "start": {"line": 4, "character": 23},
            "end": {"line": 4, "character": 34}
        }
    })
}

/// The `<pre>` twin, with no `fenced` key: both dangling citations are diagnosed at their
/// sites and the live one hovers. This is what a selected fence must give.
#[test]
fn the_pre_twin_diagnoses_and_hovers_its_citations() {
    let (diagnostics, hover) = observe("fenced-pre-twin", PLAIN, ISSUE_PRE);
    assert_eq!(diagnostics, DIAGNOSED);
    assert_eq!(hover, gateway_hover());
}

/// Without the key the issue's fence is skipped, as every fence is today: only the `<pre>`
/// block's citation is diagnosed, and the fenced one has no hover.
#[test]
fn without_the_key_the_fence_is_not_read() {
    let (diagnostics, hover) = observe("fenced-plain", PLAIN, ISSUE);
    assert_eq!(diagnostics, DIAGNOSED[1..]);
    assert_eq!(hover, Value::Null);
}

/// The issue's `mermaid` fence, selected: the server reads its citations exactly as it reads
/// the `<pre>` twin's, diagnostics and hover alike.
#[test]
fn a_selected_fence_is_diagnosed_and_hovered_like_its_pre_twin() {
    let (diagnostics, hover) = observe("fenced-selected", SELECTED, ISSUE);
    assert_eq!(diagnostics, DIAGNOSED);
    assert_eq!(hover, gateway_hover());
}
