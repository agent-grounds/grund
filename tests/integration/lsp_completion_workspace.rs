//! Owner-effective and qualified acceptance for §FS-lsp.1.6.1 and §FS-lsp.1.6.2.
use super::completion_support::*;
use serde_json::json;

fn workspace() -> Fixture {
    let f = Fixture::new();
    f.write("grund.toml", "grund_config_version = 1\nproject_name = \"root\"\n[workspace]\nmembers = [\"alpha\", \"beta\"]\ninclude_root = false\n");
    for (member, marker, trigger) in [("alpha", "@@", "%%%"), ("beta", "¶", "!!")] {
        f.write(&format!("{member}/grund.toml"), &format!("grund_config_version = 1\nproject_name = \"{member}\"\n[reference]\nmarker = \"{marker}\"\ntrigger = \"{trigger}\"\n[id]\nformat = \"{{kind}}-{{number}}-{{slug}}\"\n[fmt.cross_refs]\nenabled = false\n"));
        for id in ["FS-007-shared".to_string(), format!("FS-042-{member}")] {
            f.write(
                &format!("{member}/docs/functional-spec/{id}.md"),
                &format!("# {id}: {member} title\n\nLead.\n"),
            );
        }
        f.write(&format!("{member}/docs/notes.md"), "\n");
    }
    f
}

#[test]
fn member_triggers_are_advertised_as_deduplicated_single_characters() {
    let f = workspace();
    let s = Session::new(&f.0);
    let provider = &s.capabilities["completionProvider"];
    assert!(
        provider.is_object(),
        "missing completionProvider: {provider}"
    );
    let triggers = provider["triggerCharacters"].as_array().unwrap();
    let mut unique = std::collections::BTreeSet::new();
    for ch in triggers {
        let ch = ch.as_str().unwrap();
        assert_eq!(ch.chars().count(), 1);
        assert!(unique.insert(ch));
    }
    for ch in ["@", "%", "¶", "!"] {
        assert!(unique.contains(ch), "missing member trigger {ch}");
    }
}

#[test]
fn colliding_member_ids_remain_local_and_qualified_ids_use_target_catalog() {
    let f = workspace();
    let mut s = Session::new(&f.0);
    let path = f.0.join("alpha/docs/notes.md");
    for prefix in ["@@F", "%%%F", "%%%"] {
        let items = s.completion(&path, prefix, 0, prefix.len());
        chosen(&items, "FS-042-alpha", "@@");
        let shared = chosen(&items, "FS-007-shared", "@@");
        assert!(shared.to_string().contains("alpha title"));
        assert!(items.iter().all(|i| !i.to_string().contains("beta title")));
    }
    for prefix in ["$$F", "!!F", "@@unknown/FS-", "@@beta/"] {
        let items = s.completion(&path, prefix, 0, prefix.len());
        if prefix == "@@beta/" {
            let item = chosen(&items, "beta/FS-042-beta", "@@");
            assert_eq!(item["filterText"], "@@beta/FS-042-beta");
            assert!(
                item.to_string().contains("beta title")
                    && item
                        .to_string()
                        .contains("docs/functional-spec/FS-042-beta.md")
            );
        } else {
            assert!(items.is_empty(), "unavailable prefix {prefix}");
        }
    }
    let text = "😀 %%%beta/FS-042-old.1 tail";
    let items = s.completion(&path, text, 0, "😀 %%%beta/FS-042".len());
    let accepted = apply(text, &chosen(&items, "beta/FS-042-beta", "@@")["textEdit"]);
    assert_eq!(accepted, "😀 @@beta/FS-042-beta tail");
    s.text(&path, &accepted);
    let response = s.request(
        "textDocument/definition",
        position(&path, &accepted, 0, "😀 @@beta/FS-042".len()),
    );
    assert!(
        response["result"]
            .to_string()
            .contains("beta/docs/functional-spec/FS-042-beta.md"),
        "qualified resolution: {response}"
    );
    let beta = f.0.join("beta/docs/notes.md");
    let items = s.completion(&beta, "!!F", 0, 3);
    chosen(&items, "FS-042-beta", "¶");
    assert!(items.iter().all(|i| !i.to_string().contains("alpha title")));
}

#[test]
fn configured_punctuation_and_longest_complete_introducer_define_range() {
    let f = Fixture::new();
    f.write("grund.toml", "grund_config_version = 1\n[id]\nformat = \"{kind}_{slug}\"\n[reference]\nmarker = \"@\"\ntrigger = \"@@@\"\n");
    f.write(
        "docs/functional-spec/FS-login.md",
        "# FS_login: Login\n\nLead.\n",
    );
    f.write(
        "docs/functional-spec/FS-logout.md",
        "# FS_logout: Logout\n\nLead.\n",
    );
    let mut s = Session::new(&f.0);
    let text = "//! 😀 @@@FS_lost tail";
    let items = s.completion(&f.0.join("src/lib.rs"), text, 0, "//! 😀 @@@FS_lo".len());
    let item = chosen(&items, "FS_login", "@");
    assert_eq!(item["filterText"], "@@@FS_login");
    assert_eq!(apply(text, &item["textEdit"]), "//! 😀 @FS_login tail");
}

#[test]
fn empty_marker_disables_marker_entry_but_trigger_acceptance_remains_available() {
    let f = Fixture::new();
    f.write("grund.toml", "grund_config_version = 1\n[id]\nformat = \"{kind}-{slug}\"\n[reference]\nmarker = \"\"\ntrigger = \"%%%\"\nstrict = false\n");
    let mut s = Session::new(&f.0);
    let path = f.0.join("docs/notes.md");
    for prefix in ["F", "FS-lo", "\u{a7}F"] {
        assert!(s.completion(&path, prefix, 0, prefix.len()).is_empty());
    }
    let items = s.completion(&path, "%%%F", 0, 4);
    assert_eq!(
        apply("%%%F", &chosen(&items, "FS-login", "")["textEdit"]),
        "FS-login"
    );
}

#[test]
fn configured_kind_order_is_encoded_in_sort_text() {
    let f = Fixture::new();
    f.write("grund.toml", "grund_config_version = 1\n[id]\nformat = \"{kind}-{slug}\"\n[[kinds]]\nkind = \"FS\"\nfolder = \"docs/functional-spec\"\n[[kinds]]\nkind = \"GOAL\"\nfile = \"docs/goals.md\"\n");
    f.write("docs/goals.md", "# GOAL-zebra: Zebra\n\nLead.\n");
    let mut s = Session::new(&f.0);
    let mut items = s.completion(&f.0.join("docs/notes.md"), "\u{a7}", 0, "\u{a7}".len());
    items.sort_by_key(|i| i["sortText"].as_str().expect("sortText").to_owned());
    let texts: Vec<_> = items
        .iter()
        .map(|i| i["textEdit"]["newText"].as_str().unwrap())
        .collect();
    assert_eq!(
        texts,
        ["\u{a7}FS-login", "\u{a7}FS-logout", "\u{a7}GOAL-zebra"]
    );
}

#[test]
fn per_kind_format_and_retained_legacy_spellings_remain_candidates() {
    let f = Fixture::new();
    f.write("grund.toml", "grund_config_version = 1\n[id]\nformat = \"{kind}-{number}-{slug}\"\n[[kinds]]\nkind = \"FS\"\nfolder = \"docs/functional-spec\"\nformat = \"{kind}_{slug}\"\n[[kinds]]\nkind = \"GOAL\"\nfile = \"docs/goals.md\"\n");
    f.write(
        "docs/functional-spec/FS-custom.md",
        "# FS_custom: Custom\n\nLead.\n",
    );
    f.write("docs/goals.md", "# GOAL-007-live: Live\n\nLead.\n");
    let mut s = Session::new(&f.0);
    let path = f.0.join("docs/notes.md");
    let items = s.completion(&path, "\u{a7}FS_", 0, "\u{a7}FS_".len());
    chosen(&items, "FS_custom", "\u{a7}");
    let items = s.completion(&path, "\u{a7}FS-", 0, "\u{a7}FS-".len());
    chosen(&items, "FS-login", "\u{a7}");
    let items = s.completion(&path, "\u{a7}GOAL-0", 0, "\u{a7}GOAL-0".len());
    chosen(&items, "GOAL-007-live", "\u{a7}");
}

#[test]
fn qualified_completion_recognizes_target_grammar_after_alias_slash() {
    let f = workspace();
    f.write("beta/grund.toml", "grund_config_version = 1\nproject_name = \"beta\"\n[id]\nformat = \"{kind}_{slug}\"\n[reference]\nmarker = \"¶\"\ntrigger = \"!!\"\n");
    f.write(
        "beta/docs/functional-spec/FS-custom.md",
        "# FS_custom: Target grammar\n\nLead.\n",
    );
    let mut s = Session::new(&f.0);
    let path = f.0.join("alpha/docs/notes.md");
    let text = "@@beta/FS_cu";
    let items = s.completion(&path, text, 0, text.len());
    let item = chosen(&items, "beta/FS_custom", "@@");
    assert_eq!(apply(text, &item["textEdit"]), "@@beta/FS_custom");
    let local = "@@FS_cu";
    assert!(s.completion(&path, local, 0, local.len()).is_empty());
}

#[test]
fn unreadable_declaration_discovery_withholds_completion_edits() {
    let f = Fixture::new();
    std::fs::write(f.0.join("docs/functional-spec/FS-unreadable.md"), [0xff]).unwrap();
    let mut s = Session::new(&f.0);
    assert!(
        s.completion(&f.0.join("docs/notes.md"), "\u{a7}F", 0, "\u{a7}F".len())
            .is_empty()
    );
}

#[cfg(unix)]
#[test]
fn outward_file_symlink_is_not_a_completion_edit_target() {
    let f = Fixture::new();
    let outside = Fixture::new();
    let path = f.0.join("docs/outward.md");
    std::os::unix::fs::symlink(outside.0.join("docs/notes.md"), &path).unwrap();
    let mut s = Session::new(&f.0);
    assert!(
        s.completion(&path, "\u{a7}F", 0, "\u{a7}F".len())
            .is_empty()
    );
}

#[test]
fn watched_config_change_updates_manual_prefixes_without_restart() {
    let f = Fixture::new();
    let mut s = Session::new(&f.0);
    let path = f.0.join("docs/notes.md");
    let items = s.completion(&path, "$$F", 0, 3);
    chosen(&items, "FS-login", "\u{a7}");
    let config = f.write("grund.toml", "grund_config_version = 1\n[id]\nformat = \"{kind}-{slug}\"\n[reference]\nmarker = \"@@\"\ntrigger = \"%%\"\n");
    s.notify(
        "workspace/didChangeWatchedFiles",
        json!({"changes":[{"uri":file_uri(&config),"type":2}]}),
    );
    let items = s.completion(&path, "%%F", 0, 3);
    chosen(&items, "FS-login", "@@");
    assert!(s.completion(&path, "$$F", 0, 3).is_empty());
}
