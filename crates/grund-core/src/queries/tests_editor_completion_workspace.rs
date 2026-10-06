//! Owner and qualified core authoring (§FS-lsp.1.6.1, §FS-lsp.1.6.2,
//! §FS-lsp.1.6.3). Colliding spellings must not create sibling ambiguity.

use super::tests_editor_completion::{Fixture, apply, ids};

fn workspace() -> Fixture {
    let f = Fixture::new();
    f.config("project_name = \"root\"\n[workspace]\nmembers = [\"alpha\", \"beta\"]\ninclude_root = false\n");
    for (alias, marker, trigger, format) in [
        ("alpha", "@@", "%%%", "{kind}-{slug}"),
        ("beta", "¶", "!!", "{kind}_{slug}"),
    ] {
        f.put(&format!("{alias}/grund.toml"), &format!("grund_config_version = 1\nproject_name = \"{alias}\"\n[reference]\nmarker = \"{marker}\"\ntrigger = \"{trigger}\"\n[id]\nformat = \"{format}\"\n"));
        f.put(
            &format!("{alias}/docs/functional-spec/FS-shared.md"),
            &format!("# FS-shared: {alias} title\n\nLead.\n"),
        );
        f.put(&format!("{alias}/docs/notes.md"), "\n");
    }
    f.put(
        "beta/docs/functional-spec/FS-custom.md",
        "# FS_custom: Target grammar\n\nLead.\n",
    );
    f
}

#[test]
fn completion_owner_and_qualified_catalogs_use_target_grammar() {
    let f = workspace();
    let snapshot = f.load();
    let characters = snapshot.completion.trigger_characters();
    assert_eq!(characters, ["!", "%", "@", "¶"]);
    let items = f.items(&snapshot, "alpha/docs/notes.md", "%%%F", 0, 4);
    assert_eq!(ids(&items), ["FS-shared"]);
    assert_eq!(items[0].title, "alpha title");
    assert_eq!(items[0].text, "@@FS-shared");
    assert_eq!(items[0].source_path, "docs/functional-spec/FS-shared.md");
    for prefix in ["@@beta/", "@@beta/FS_cu"] {
        let items = f.items(&snapshot, "alpha/docs/notes.md", prefix, 0, prefix.len());
        let custom = items
            .iter()
            .find(|item| item.id == "beta/FS_custom")
            .unwrap();
        assert_eq!(custom.title, "Target grammar");
        assert_eq!(custom.filter_text, "@@beta/FS_custom");
        assert_eq!(apply(prefix, custom), "@@beta/FS_custom");
    }
    let text = "😀 %%%beta/FS_cursed.1 and @@FS-shared";
    let items = f.items(
        &snapshot,
        "alpha/docs/notes.md",
        text,
        0,
        "😀 %%%beta/FS_cu".len(),
    );
    assert_eq!(
        apply(text, &items[0]),
        "😀 @@beta/FS_custom and @@FS-shared"
    );
    for text in ["@@FS_cu", "@@unknown/F", "@@beta", "$$F", "!!F"] {
        assert!(
            f.items(&snapshot, "alpha/docs/notes.md", text, 0, text.len())
                .is_empty()
        );
    }
    let items = f.items(&snapshot, "beta/docs/notes.md", "!!FS-sh", 0, 7);
    assert_eq!(ids(&items), ["FS-shared"]);
    assert_eq!(items[0].title, "beta title");
    assert_eq!(items[0].text, "¶FS-shared");
    let text = "!!FS-shard.1 tail";
    let items = f.items(&snapshot, "beta/docs/notes.md", text, 0, "!!FS-sh".len());
    assert_eq!(apply(text, &items[0]), "¶FS-shared tail");
}

#[test]
fn completion_unavailable_alias_and_standalone_qualification_offer_no_edits() {
    let f = workspace();
    f.config("project_name = \"root\"\n[workspace]\nmembers = [\"alpha\"]\ninclude_root = false\n");
    let snapshot = f.load();
    assert!(
        f.items(&snapshot, "alpha/docs/notes.md", "@@beta/F", 0, 8)
            .is_empty()
    );
    let standalone = Fixture::new();
    let snapshot = standalone.load();
    assert!(
        standalone
            .items(&snapshot, "docs/notes.md", "$$beta/F", 0, 8)
            .is_empty()
    );
}

#[cfg(unix)]
#[test]
fn completion_outward_file_symlink_withholds_edits() {
    let f = Fixture::new();
    let outside = Fixture::new();
    let path = f.0.join("docs/outward.md");
    std::os::unix::fs::symlink(outside.0.join("docs/notes.md"), &path).unwrap();
    let snapshot = f.load();
    assert!(snapshot.completion.complete(&path, "$$F", 0, 3).is_empty());
}
