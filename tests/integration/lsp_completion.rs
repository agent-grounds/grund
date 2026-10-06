//! Real-server authoring acceptance for §FS-lsp.1.6.1, §FS-lsp.1.6.2,
//! §FS-lsp.1.6.3 and §FS-lsp.1.6.4. Missing advertisement and dispatch fail
//! independently; no future core API is required to compile these tests.
#[path = "lsp_completion_support.rs"]
mod completion_support;
#[path = "lsp_completion_workspace.rs"]
mod completion_workspace;

use completion_support::*;
use serde_json::json;

#[test]
fn advertises_completion_provider_independently_of_requests() {
    let f = Fixture::new();
    let s = Session::new(&f.0);
    let provider = &s.capabilities["completionProvider"];
    assert!(
        provider.is_object(),
        "missing completionProvider: {provider}"
    );
    let triggers = provider["triggerCharacters"].as_array().unwrap();
    assert_eq!(triggers.len(), 2);
    assert!(triggers.contains(&json!("\u{a7}")) && triggers.contains(&json!("$")));
}

#[test]
fn handles_manual_completion_independently_of_advertisement() {
    let f = Fixture::new();
    let mut s = Session::new(&f.0);
    let items = s.completion(&f.0.join("docs/notes.md"), "\u{a7}F", 0, "\u{a7}F".len());
    chosen(&items, "FS-login", "\u{a7}");
}

#[test]
fn marker_trigger_empty_and_longer_prefixes_offer_plain_text_items() {
    let f = Fixture::new();
    let mut s = Session::new(&f.0);
    for prefix in ["\u{a7}", "\u{a7}F", "$$", "$$F", "\u{a7}FS-lo", "$$FS-lo"] {
        let items = s.completion(&f.0.join("docs/notes.md"), prefix, 0, prefix.len());
        let item = chosen(&items, "FS-login", "\u{a7}");
        assert_eq!(apply(prefix, &item["textEdit"]), "\u{a7}FS-login");
        assert!(item["insertTextFormat"].is_null() || item["insertTextFormat"] == 1);
        assert!(item["additionalTextEdits"].is_null() || item["additionalTextEdits"] == json!([]));
        assert_eq!(
            item["filterText"],
            format!(
                "{}FS-login",
                if prefix.starts_with('\u{a7}') {
                    "\u{a7}"
                } else {
                    "$$"
                }
            )
        );
        let metadata = item.to_string();
        assert!(
            metadata.contains("Login") && metadata.contains("docs/functional-spec/FS-login.md"),
            "missing title/path: {item}"
        );
    }
}

#[test]
fn case_sensitive_matching_and_explicit_stable_sort_order() {
    let f = Fixture::new();
    f.write(
        "docs/functional-spec/FS-login-extra.md",
        "# FS-login-extra: Extra\n\nLead.\n",
    );
    let mut s = Session::new(&f.0);
    let path = f.0.join("docs/notes.md");
    for prefix in ["\u{a7}fs", "\u{a7}F-login", "\u{a7}FS-missing"] {
        assert!(
            s.completion(&path, prefix, 0, prefix.len()).is_empty(),
            "no fuzzy fallback"
        );
    }
    let items = s.completion(&path, "\u{a7}FS-login", 0, "\u{a7}FS-login".len());
    let mut sorted = items.clone();
    sorted.sort_by_key(|i| {
        i["sortText"]
            .as_str()
            .expect("explicit sortText")
            .to_owned()
    });
    let texts: Vec<_> = sorted
        .iter()
        .map(|i| i["textEdit"]["newText"].as_str().unwrap())
        .collect();
    assert_eq!(texts, ["\u{a7}FS-login", "\u{a7}FS-login-extra"]);
    assert_eq!(items, sorted, "wire order is deterministic too");
    assert_eq!(
        items,
        s.current_completion(&path, "\u{a7}FS-login", 0, "\u{a7}FS-login".len())
    );
}

#[test]
fn whole_token_replacement_removes_old_suffix_and_preserves_utf16_neighbors() {
    let f = Fixture::new();
    let mut s = Session::new(&f.0);
    let path = f.0.join("docs/notes.md");
    for (text, before, old, after) in [
        (
            "😀 See \u{a7}FS-lost.1 and \u{a7}FS-logout after",
            "😀 See ",
            "\u{a7}FS-lost.1",
            " and \u{a7}FS-logout after",
        ),
        (
            "[\u{a7}FS-lost.1](target.md) after",
            "[",
            "\u{a7}FS-lost.1",
            "](target.md) after",
        ),
        (
            "\u{a7}FS-lost\u{a7}FS-logout",
            "",
            "\u{a7}FS-lost",
            "\u{a7}FS-logout",
        ),
    ] {
        let cursor = before.len() + "\u{a7}FS-lo".len();
        let items = s.completion(&path, text, 0, cursor);
        let edit = &chosen(&items, "FS-login", "\u{a7}")["textEdit"];
        assert_eq!(
            edit["range"],
            json!({"start":{"line":0,"character":before.encode_utf16().count()},
            "end":{"line":0,"character":format!("{before}{old}").encode_utf16().count()}})
        );
        assert_eq!(apply(text, edit), format!("{before}\u{a7}FS-login{after}"));
    }
    assert!(
        s.completion(&path, "\u{a7}FS-login.1", 0, "\u{a7}FS-login.1".len())
            .is_empty()
    );
}

#[test]
fn refuses_protected_contexts_but_accepts_comments_and_python_docstrings() {
    let f = Fixture::new();
    let mut s = Session::new(&f.0);
    for (file, text, line, before_cursor) in [
        ("docs/notes.md", "`\u{a7}FS-lo`", 0, "`\u{a7}FS-lo"),
        (
            "docs/notes.md",
            "[label](\u{a7}FS-lo)",
            0,
            "[label](\u{a7}FS-lo",
        ),
        ("docs/notes.md", "```\n\u{a7}FS-lo\n```", 1, "\u{a7}FS-lo"),
        (
            "docs/notes.md",
            "# FS-login: \u{a7}FS-lo",
            0,
            "# FS-login: \u{a7}FS-lo",
        ),
        (
            "docs/notes.md",
            "<!-- grund:fmt off -->\n\u{a7}FS-lo",
            1,
            "\u{a7}FS-lo",
        ),
        ("docs/notes.md", "<\u{a7}>FS-lo", 0, "<\u{a7}>FS-lo"),
        (
            "src/lib.rs",
            "let x = \"\u{a7}FS-lo\";",
            0,
            "let x = \"\u{a7}FS-lo",
        ),
        ("outside/notes.md", "\u{a7}FS-lo", 0, "\u{a7}FS-lo"),
    ] {
        let path = f.write(file, "\n");
        assert!(
            s.completion(&path, text, line, before_cursor.len())
                .is_empty(),
            "protected context {text}"
        );
    }
    for (file, text) in [
        ("src/lib.rs", "//! \u{a7}FS-lo"),
        ("src/lib.py", "\"\"\"\u{a7}FS-lo"),
    ] {
        let path = f.write(file, "\n");
        let items = s.completion(&path, text, 0, text.len());
        chosen(&items, "FS-login", "\u{a7}");
    }
}

#[test]
fn completion_requires_introducer_even_in_non_strict_mode() {
    let f = Fixture::new();
    f.write(
        "grund.toml",
        "grund_config_version = 1\n[id]\nformat = \"{kind}-{slug}\"\n[reference]\nstrict = false\n",
    );
    let mut s = Session::new(&f.0);
    let path = f.0.join("docs/notes.md");
    assert!(s.completion(&path, "FS-lo", 0, 5).is_empty());
    let items = s.completion(&path, "$$F", 0, 3);
    chosen(&items, "FS-login", "\u{a7}");
}

#[test]
fn source_comment_acceptance_resolves_and_agrees_with_cli_formatting() {
    let f = Fixture::new();
    let mut s = Session::new(&f.0);
    let path = f.0.join("src/lib.rs");
    let text = "//! 😀 $$FS-lo\n";
    let cursor = text.trim_end().len();
    let items = s.completion(&path, text, 0, cursor);
    let accepted = apply(text, &chosen(&items, "FS-login", "\u{a7}")["textEdit"]);
    assert_eq!(accepted, "//! 😀 \u{a7}FS-login\n");
    assert!(
        s.on_type(&path, &accepted, 0, accepted.trim_end().len())
            .is_empty()
    );
    let response = s.request(
        "textDocument/definition",
        position(&path, &accepted, 0, accepted.trim_end().len() - 1),
    );
    assert!(
        response.get("error").is_none(),
        "definition rejected: {response}"
    );
    assert!(
        response["result"].to_string().contains("FS-login.md"),
        "must resolve: {response}"
    );
    f.write("src/lib.rs", &accepted);
    for args in [
        &["check", "src/lib.rs"][..],
        &["fmt", "src/lib.rs", "--check"][..],
    ] {
        let output = cli(&f.0, args);
        assert!(
            output.status.success(),
            "CLI agreement {args:?}: {} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn completion_and_on_type_discard_then_rerequest_in_both_orders() {
    let f = Fixture::new();
    let mut s = Session::new(&f.0);
    let path = f.0.join("docs/notes.md");
    let text = "$$FS-login";
    let old_items = s.completion(&path, text, 0, text.len());
    let old_on_type = s.on_type(&path, text, 0, text.len());
    assert_eq!(old_on_type.len(), 1);
    let converted = apply(text, &old_on_type[0]);
    assert_eq!(converted, "\u{a7}FS-login");
    drop(old_items); // Client invalidates completion response after conversion.
    let items = s.completion(&path, &converted, 0, converted.len());
    let accepted = apply(
        &converted,
        &chosen(&items, "FS-login", "\u{a7}")["textEdit"],
    );
    assert!(s.on_type(&path, &accepted, 0, accepted.len()).is_empty());

    let items = s.completion(&path, text, 0, text.len());
    let obsolete = s.on_type(&path, text, 0, text.len());
    let accepted = apply(text, &chosen(&items, "FS-login", "\u{a7}")["textEdit"]);
    drop(obsolete); // Client invalidates on-type response after acceptance.
    assert_eq!(accepted, "\u{a7}FS-login");
    assert!(s.on_type(&path, &accepted, 0, accepted.len()).is_empty());
}

#[test]
fn continuous_typing_immediate_trigger_choice_and_continued_typing() {
    let f = Fixture::new();
    let mut s = Session::new(&f.0);
    let path = f.0.join("docs/notes.md");
    let mut text = String::new();
    for ch in "$$F".chars() {
        text.push(ch);
        let edits = s.on_type(&path, &text, 0, text.len());
        assert!(edits.is_empty(), "partial prefix must not convert");
        if text == "$$" {
            let items = s.current_completion(&path, &text, 0, text.len());
            assert_eq!(
                apply(&text, &chosen(&items, "FS-login", "\u{a7}")["textEdit"]),
                "\u{a7}FS-login"
            );
        }
    }
    let items = s.current_completion(&path, &text, 0, text.len());
    text = apply(&text, &chosen(&items, "FS-login", "\u{a7}")["textEdit"]);
    for ch in " and more".chars() {
        text.push(ch);
        assert!(s.on_type(&path, &text, 0, text.len()).is_empty());
    }
    assert_eq!(text, "\u{a7}FS-login and more");
}

#[test]
fn declaration_overlay_changes_and_close_refresh_candidates() {
    let f = Fixture::new();
    let mut s = Session::new(&f.0);
    let path = f.0.join("docs/notes.md");
    let decl = f.0.join("docs/functional-spec/FS-login.md");
    let items = s.completion(&path, "\u{a7}FS-lo", 0, "\u{a7}FS-lo".len());
    chosen(&items, "FS-login", "\u{a7}");
    s.text(&decl, "# FS-loaded: Unsaved title\n\nLead.\n");
    let items = s.current_completion(&path, "\u{a7}FS-lo", 0, "\u{a7}FS-lo".len());
    let item = chosen(&items, "FS-loaded", "\u{a7}");
    assert!(item.to_string().contains("Unsaved title"));
    assert!(
        items
            .iter()
            .all(|i| i["textEdit"]["newText"] != "\u{a7}FS-login")
    );
    s.close_document(&decl);
    let items = s.current_completion(&path, "\u{a7}FS-lo", 0, "\u{a7}FS-lo".len());
    chosen(&items, "FS-login", "\u{a7}");
    assert!(
        items
            .iter()
            .all(|i| i["textEdit"]["newText"] != "\u{a7}FS-loaded")
    );
}

#[test]
fn ambiguous_declarations_are_not_offered() {
    let f = Fixture::new();
    f.write(
        "docs/functional-spec/FS-duplicate.md",
        "# FS-login: Duplicate\n\nLead.\n",
    );
    let mut s = Session::new(&f.0);
    let items = s.completion(
        &f.0.join("docs/notes.md"),
        "\u{a7}FS-lo",
        0,
        "\u{a7}FS-lo".len(),
    );
    assert!(
        items
            .iter()
            .all(|i| i["textEdit"]["newText"] != "\u{a7}FS-login")
    );
    chosen(&items, "FS-logout", "\u{a7}");
}
