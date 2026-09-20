//! Actual editor request-handler proof of literal target metadata (§FS-config.3.4.3,
//! §FS-lsp.1.2), preserving preview, usage, navigation and UTF-16 ranges.

use super::*;

fn config(title: Option<&str>) -> String {
    let metadata = title
        .map(|title| format!("title = {}\n", serde_json::to_string(title).unwrap()))
        .unwrap_or_default();
    format!(
        "grund_config_version = 1\n[id]\nformat = \"{{kind}}-{{slug}}\"\n[reference]\nstrict = true\n\
        [[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n{metadata}"
    )
}

fn server(root: &Path) -> (Server, Connection) {
    let (connection, client) = Connection::memory();
    let server = Server::new(
        connection,
        [(Url::from_directory_path(root).unwrap(), root.to_path_buf())]
            .into_iter()
            .collect(),
        true,
    )
    .unwrap();
    (server, client)
}

fn request(
    server: &mut Server,
    client: &Connection,
    method: &str,
    path: &Path,
    line: u32,
    character: u32,
) -> Value {
    server
        .handle_request(Request::new(
            1.into(),
            method.into(),
            json!({
                "textDocument": {"uri": Url::from_file_path(path).unwrap()},
                "position": {"line": line, "character": character},
                "context": {"includeDeclaration": false}
            }),
        ))
        .unwrap();
    let Message::Response(response) = client.receiver.recv().unwrap() else {
        panic!("response")
    };
    assert!(response.error.is_none(), "{:?}", response.error);
    response.result.unwrap()
}

fn expected(body: &str, line: u32, start: u32, end: u32) -> Value {
    json!({"contents":{"kind":"markdown", "value":body},
        "range":{"start":{"line":line,"character":start},"end":{"line":line,"character":end}}})
}

/// §FS-lsp.1.2.8: the target kind's effective title is appended to a successful
/// hover as a `Kind: ` paragraph — on citation previews, declarations, sections,
/// inline-source titles and stubs alike, fenced by the §FS-lsp.1.2.4 backtick
/// convention. No title leaves the hover unchanged, an empty one still adds the
/// paragraph, the metadata stays literal while only the preview is linkified,
/// and usage counts, ranges, navigation and the missing-target suppression are
/// the same at every value.
#[test]
fn kind_title_handler_preserves_each_title_preview_and_navigation() {
    let root = test_root("kind_title_handler");
    let spec = root.join("docs/FS-authored.md");
    let stub = root.join("docs/FS-inline.md");
    let inline = root.join("src/inline.rs");
    let user = root.join("src/user.rs");
    let declaration = "FS-authored: Authored title";
    let stub_title = "FS-inline: [source](../src/inline.rs)";
    write(
        &spec,
        "# FS-authored: Authored title\n\nLead \u{a7}FS-other.\n\n## 1. Detail\n\nDetail body.\n",
    );
    write(
        &root.join("docs/FS-other.md"),
        "# FS-other: Other\n\nOther body.\n",
    );
    write(&stub, &format!("# {stub_title}\n"));
    write(
        &inline,
        "/// FS-inline: Inline title\n/// Inline body.\npub fn item() {}\n",
    );
    write(
        &user,
        "//! \u{a7}FS-authored\n//! \u{a7}FS-authored.1\n//! \u{a7}FS-inline\n//! \u{a7}FS-missing\n",
    );
    let mut baseline_navigation = None;
    for (title, suffix) in [
        (None, ""),
        (
            Some("Product `contracts` \u{a7}FS-other"),
            "\n\nKind: ``Product `contracts` \u{a7}FS-other``",
        ),
        (Some("`edge``"), "\n\nKind: ``` `edge`` ```"),
        (Some(""), "\n\nKind: ``"),
    ] {
        write(&root.join("grund.toml"), &config(title));
        let (mut server, client) = server(&root);
        for (path, line, start, text, usage) in [
            (&spec, 0, 2, declaration, "cited at 2 sites across 1 file"),
            (&spec, 4, 3, "1. Detail", "cited at 1 site across 1 file"),
            (&stub, 0, 2, stub_title, "cited at 1 site across 1 file"),
            (
                &inline,
                0,
                4,
                "FS-inline: Inline title",
                "cited at 1 site across 1 file",
            ),
        ] {
            assert_eq!(
                request(
                    &mut server,
                    &client,
                    "textDocument/hover",
                    path,
                    line,
                    start + 1
                ),
                expected(
                    &format!("`{text}` — {usage}{suffix}"),
                    line,
                    start,
                    start + text.encode_utf16().count() as u32
                )
            );
        }
        // Linkification applies to the preview, while identical citation text in metadata stays literal.
        let other_uri = Url::from_file_path(root.join("docs/FS-other.md")).unwrap();
        let preview = format!(
            "# FS-authored: Authored title\n\nLead [\u{a7}FS-other]({other_uri}#L1).\n\n## 1. Detail\n"
        );
        assert_eq!(
            request(&mut server, &client, "textDocument/hover", &user, 0, 6),
            expected(
                &format!("{preview}{suffix}"),
                0,
                4,
                4 + "\u{a7}FS-authored".encode_utf16().count() as u32
            )
        );
        assert_eq!(
            request(&mut server, &client, "textDocument/hover", &user, 1, 6),
            expected(
                &format!("# FS-authored: Authored title\n## 1. Detail\n\nDetail body.\n{suffix}"),
                1,
                4,
                4 + "\u{a7}FS-authored.1".encode_utf16().count() as u32
            )
        );
        assert_eq!(
            request(&mut server, &client, "textDocument/hover", &user, 3, 6),
            Value::Null
        );
        let definition = request(&mut server, &client, "textDocument/definition", &user, 0, 6);
        let references = request(&mut server, &client, "textDocument/references", &spec, 0, 6);
        assert_eq!(
            definition[0]["targetUri"],
            Url::from_file_path(&spec).unwrap().as_str()
        );
        assert_eq!(references.as_array().unwrap().len(), 2);
        let navigation = (definition, references);
        if let Some(baseline) = &baseline_navigation {
            assert_eq!(&navigation, baseline);
        } else {
            baseline_navigation = Some(navigation);
        }
    }
}

#[test]
fn kind_title_handler_uses_target_snapshot_for_workspace_titles_and_bindings() {
    let root = test_root("kind_title_handler_workspace");
    write(
        &root.join("grund.toml"),
        &(config(Some("Caller title")) + "[workspace]\nmembers = [\"target\", \"citer\"]\n"),
    );
    write(
        &root.join("target/grund.toml"),
        &config(Some("Target title")),
    );
    write(&root.join("citer/grund.toml"), &config(Some("Citer title")));
    let target = root.join("target/docs/FS-price.md");
    let user = root.join("citer/src/user.rs");
    write(
        &target,
        "# FS-price: Authored price\n\n## 1. Price <!-- grund:value -->\n### 1.1. 12\n",
    );
    write(&user, "//! `12` (§target/FS-price.1.1)\n");
    let (mut server, client) = server(&root);
    assert_eq!(
        request(&mut server, &client, "textDocument/hover", &target, 0, 5),
        expected(
            "`FS-price: Authored price` — cited at 1 site across 1 file\n\nKind: `Target title`",
            0,
            2,
            26
        )
    );
    let hover = request(&mut server, &client, "textDocument/hover", &user, 0, 14);
    assert_eq!(
        hover["contents"]["value"],
        "# FS-price: Authored price\n### 1.1. 12\n\n\nKind: `Target title`"
    );
    assert_eq!(
        hover["range"],
        json!({"start":{"line":0,"character":10},"end":{"line":0,"character":30}})
    );
    // A declaration-side request must retain metadata from this snapshot even if
    // config on disk is now invalid; there is no per-hover metadata reload.
    write(&root.join("target/grund.toml"), "this config is invalid\n");
    assert_eq!(
        request(&mut server, &client, "textDocument/hover", &target, 0, 5),
        expected(
            "`FS-price: Authored price` — cited at 1 site across 1 file\n\nKind: `Target title`",
            0,
            2,
            26
        )
    );
}
