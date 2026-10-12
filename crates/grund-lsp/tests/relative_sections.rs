//! Real-server coverage for a relative section heading
//! (§FS-declarations.section.relative.shown): its title hover, the range the
//! record gives it, and where a citation of its canonical path goes (§FS-lsp.1.2.4).

mod support;

use serde_json::json;
use std::fs;
use support::*;

/// §FS-lsp.1.2.4: `### .1: Username` hovers as `format.1: Username` over the
/// written title, and a citation of `FS-login.format.1` previews and goes to that
/// written line.
#[test]
fn a_relative_heading_hovers_and_navigates_at_its_canonical_path() {
    let root = test_root("relative-sections");
    fs::write(
        root.join("grund.toml"),
        "grund_config_version = 1\n\n[reference]\nstrict = true\n\n\
         [id]\nformat = \"{kind}-{slug}\"\nnamed_sections = true\n\n\
         [scan]\ninclude = [\"docs\"]\nextensions = [\"md\"]\n",
    )
    .expect("write config");
    let spec = root.join("docs/FS-login.md");
    fs::write(
        &spec,
        "# FS-login: Login\n\n## format: Formatting\n\n### .1: Username\n\n\
         Trim surrounding whitespace.\n",
    )
    .expect("write spec");
    let usage = root.join("docs/use.md");
    fs::write(&usage, "# Use\n\nSee \u{a7}FS-login.format.1.\n").expect("write usage");

    let (mut child, mut stdin, receiver) = start_server(&root);

    let title = hover_result(&mut stdin, &receiver, &mut child, 2, &file_uri(&spec), 4, 6);
    assert_eq!(
        title["contents"]["value"].as_str(),
        Some(
            "`format.1: Username` — cited at 1 site across 1 file\n\n\
             Kind: `What: behavior, requirements, and constraints`"
        ),
        "{title:?}"
    );
    // The written title, `.1: Username`, after the `### ` run.
    assert_eq!(
        title["range"],
        json!({
            "start": { "line": 4, "character": 4 },
            "end": { "line": 4, "character": "### .1: Username".len() }
        })
    );

    let citation = hover_result(
        &mut stdin,
        &receiver,
        &mut child,
        3,
        &file_uri(&usage),
        2,
        6,
    );
    let preview = citation["contents"]["value"]
        .as_str()
        .unwrap_or_else(|| panic!("citation hover: {citation:?}"));
    assert!(
        preview.contains("### format.1: Username") && preview.contains("Trim surrounding"),
        "{preview}"
    );

    send_message(
        &mut stdin,
        json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "textDocument/definition",
            "params": {
                "textDocument": { "uri": file_uri(&usage) },
                "position": { "line": 2, "character": 6 }
            }
        }),
    );
    let definition = recv_response_or_panic(&receiver, &mut child, 4);
    let links = definition["result"]
        .as_array()
        .unwrap_or_else(|| panic!("definition links: {definition:?}"));
    assert!(
        links.iter().any(|link| {
            link["targetUri"].as_str() == Some(file_uri(&spec).as_str())
                && link["targetSelectionRange"]["start"]["line"].as_i64() == Some(4)
        }),
        "{links:?}"
    );

    send_message(
        &mut stdin,
        json!({"jsonrpc": "2.0", "id": 5, "method": "shutdown", "params": null}),
    );
    recv_response_or_panic(&receiver, &mut child, 5);
    send_message(
        &mut stdin,
        json!({"jsonrpc": "2.0", "method": "exit", "params": null}),
    );
    drop(stdin);
    wait_for_exit(&mut child);
    let _ = fs::remove_dir_all(root);
}
