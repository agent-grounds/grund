//! Real-server coverage for a mixed section coordinate, a name under a number
//! (§FS-config.3.3.1): `### 1.goals: Goals` under `## 1. Overview` is one
//! address, and the editor answers a citation of it as that section (§FS-lsp.1.3.3).

mod support;

use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin};
use std::sync::mpsc;
use support::*;

/// The citing line in `docs/use.md`, and the UTF-16 span of its whole
/// `<§>FS-x.1.goals` token: `Uses ` is five units, the token thirteen.
const CITE_LINE: i64 = 2;
const TOKEN_START: i64 = 5;
const TOKEN_END: i64 = 18;
/// A cursor inside the `goals` tail, past the numeric prefix `FS-x.1`.
const IN_TAIL: i64 = 14;

struct Fixture {
    root: PathBuf,
    spec: PathBuf,
    usage: PathBuf,
}

/// A named-sections tree whose `FS-x` carries `### 1.goals: Goals` (line 6)
/// beneath `## 1. Overview` (line 2), cited once from `docs/use.md`.
fn fixture(name: &str) -> Fixture {
    let root = test_root(name);
    fs::write(
        root.join("grund.toml"),
        "grund_config_version = 1\n\n[reference]\nstrict = true\n\n\
         [id]\nformat = \"{kind}-{slug}\"\nnamed_sections = true\n\n\
         [scan]\ninclude = [\"docs\"]\nextensions = [\"md\"]\n",
    )
    .expect("write config");
    let spec = root.join("docs/FS-x.md");
    fs::write(
        &spec,
        "# FS-x: Example\n\n## 1. Overview\n\nOverview lead.\n\n\
         ### 1.goals: Goals\n\nGoals target.\n",
    )
    .expect("write spec");
    let usage = root.join("docs/use.md");
    fs::write(&usage, "# Use\n\nUses \u{a7}FS-x.1.goals here.\n").expect("write usage");
    Fixture { root, spec, usage }
}

fn request(
    stdin: &mut ChildStdin,
    receiver: &mpsc::Receiver<Value>,
    child: &mut Child,
    id: i64,
    method: &str,
    uri: &str,
) -> Value {
    send_message(
        stdin,
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": CITE_LINE, "character": IN_TAIL }
            }
        }),
    );
    recv_response_or_panic(receiver, child, id)
}

fn shut_down(
    mut stdin: ChildStdin,
    receiver: &mpsc::Receiver<Value>,
    mut child: Child,
    root: &Path,
) {
    send_message(
        &mut stdin,
        json!({"jsonrpc": "2.0", "id": 99, "method": "shutdown", "params": null}),
    );
    recv_response_or_panic(receiver, &mut child, 99);
    send_message(
        &mut stdin,
        json!({"jsonrpc": "2.0", "method": "exit", "params": null}),
    );
    drop(stdin);
    wait_for_exit(&mut child);
    let _ = fs::remove_dir_all(root);
}

/// §FS-lsp.1.3.3: hover on `<§>FS-x.1.goals` previews the `1.goals` section, not
/// section `1` it sits under — a mixed path is never an alias for its number.
#[test]
fn a_mixed_citation_hovers_as_the_section_it_names() {
    let fx = fixture("mixed-sections-hover");
    let (mut child, mut stdin, receiver) = start_server(&fx.root);

    let hover = hover_result(
        &mut stdin,
        &receiver,
        &mut child,
        2,
        &file_uri(&fx.usage),
        CITE_LINE,
        IN_TAIL,
    );
    let preview = hover["contents"]["value"].as_str().unwrap_or_default();
    assert!(
        preview.contains("Goals target") && !preview.contains("Overview lead"),
        "hover answers FS-x.1.goals, not FS-x.1: {hover:?}"
    );

    shut_down(stdin, &receiver, child, &fx.root);
}

/// §FS-lsp.1.3.3: go-to-definition on `<§>FS-x.1.goals` lands on the
/// `### 1.goals: Goals` heading (line 6), from the whole citation token.
#[test]
fn a_mixed_citation_goes_to_the_section_it_names() {
    let fx = fixture("mixed-sections-definition");
    let (mut child, mut stdin, receiver) = start_server(&fx.root);

    let definition = request(
        &mut stdin,
        &receiver,
        &mut child,
        3,
        "textDocument/definition",
        &file_uri(&fx.usage),
    );
    let links = definition["result"].as_array().cloned().unwrap_or_default();
    assert!(
        links.iter().any(|link| {
            link["targetUri"].as_str() == Some(file_uri(&fx.spec).as_str())
                && link["targetSelectionRange"]["start"]["line"].as_i64() == Some(6)
                && link["originSelectionRange"]["start"]["character"].as_i64() == Some(TOKEN_START)
                && link["originSelectionRange"]["end"]["character"].as_i64() == Some(TOKEN_END)
        }),
        "definition goes to the 1.goals heading: {definition:?}"
    );

    shut_down(stdin, &receiver, child, &fx.root);
}

/// §FS-lsp.1.3.3: documentHighlight marks the whole `<§>FS-x.1.goals` token as
/// one span and never shortens the mixed tail to the numeric prefix `FS-x.1`.
#[test]
fn a_mixed_citation_highlights_as_one_whole_token() {
    let fx = fixture("mixed-sections-highlight");
    let (mut child, mut stdin, receiver) = start_server(&fx.root);

    let highlights = request(
        &mut stdin,
        &receiver,
        &mut child,
        4,
        "textDocument/documentHighlight",
        &file_uri(&fx.usage),
    );
    let spans = highlights["result"].as_array().cloned().unwrap_or_default();
    let whole = json!({
        "start": { "line": CITE_LINE, "character": TOKEN_START },
        "end": { "line": CITE_LINE, "character": TOKEN_END }
    });
    assert!(
        spans.iter().any(|span| span["range"] == whole),
        "the whole FS-x.1.goals citation is one highlight: {highlights:?}"
    );
    assert!(
        spans
            .iter()
            .filter(|span| span["range"]["start"]["line"].as_i64() == Some(CITE_LINE))
            .all(|span| span["range"] == whole),
        "no shortened span on the citing line: {highlights:?}"
    );

    shut_down(stdin, &receiver, child, &fx.root);
}
