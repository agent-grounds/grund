//! Grammar-owned introducer punctuation and adjacent tokens (§FS-lsp.1.6.1,
//! §FS-lsp.1.6.3). Byte edits retain the full old suffix and its neighbors.
//!
//! Unqualified underscore-marked citations are not scanned because of the existing
//! ID word boundary (tool report grund.57). Their cases verify token/edit semantics;
//! the hyphen controls below prove scanning, resolution and replay (§FS-lsp.1.6.4).

use super::tests_editor_completion::{Fixture, apply, ids};

#[test]
fn completion_introducers_inside_ids_preserve_progressive_prefixes_and_neighbors() {
    for (separator, marker, trigger) in [("-", "\u{a7}", "-"), ("_", "_", "$$")] {
        let f = Fixture::new();
        f.config(&format!("[id]\nformat = \"{{kind}}{separator}{{slug}}\"\n[reference]\nmarker = \"{marker}\"\ntrigger = \"{trigger}\"\n[fmt.cross_refs]\nenabled = false\n"));
        for slug in ["login", "logout"] {
            f.put(
                &format!("docs/functional-spec/FS-{slug}.md"),
                &format!("# FS{separator}{slug}: {slug}\n\nLead.\n"),
            );
        }
        let snapshot = f.load();
        let id = format!("FS{separator}login");
        let other = format!("FS{separator}logout");
        for introducer in [marker, trigger] {
            for prefix in [
                "".to_string(),
                "F".into(),
                "FS".into(),
                format!("FS{separator}"),
                format!("FS{separator}lo"),
                id.clone(),
            ] {
                let text = format!("{introducer}{prefix}");
                let items = f.items(&snapshot, "docs/notes.md", &text, 0, text.len());
                let item = items.iter().find(|item| item.id == id).unwrap();
                assert_eq!((item.start, item.end), (0, text.len()), "{text}");
                assert_eq!(apply(&text, item), format!("{marker}{id}"));
                assert_eq!(item.filter_text, format!("{introducer}{id}"));
            }
            for neighbor in [marker, trigger] {
                let before = format!("😀 {marker}{other} ");
                let old = format!("{introducer}FS{separator}lost.1");
                let after = format!("{neighbor}{other} tail");
                let text = format!("{before}{old}{after}");
                let cursor = before.len() + format!("{introducer}FS{separator}lo").len();
                let items = f.items(&snapshot, "docs/notes.md", &text, 0, cursor);
                let item = items.iter().find(|item| item.id == id).unwrap();
                assert_eq!(
                    (item.start, item.end),
                    (before.len(), before.len() + old.len())
                );
                assert_eq!(apply(&text, item), format!("{before}{marker}{id}{after}"));
                let text = format!("{neighbor}{other}{introducer}FS{separator}lo");
                let items = f.items(&snapshot, "docs/notes.md", &text, 0, text.len());
                let item = items.iter().find(|item| item.id == id).unwrap();
                assert_eq!(item.start, neighbor.len() + other.len());
                assert_eq!(apply(&text, item), format!("{neighbor}{other}{marker}{id}"));
            }
            let text = format!("//! {introducer}FS{separator}lo");
            let items = f.items(&snapshot, "src/lib.rs", &text, 0, text.len());
            let item = items.iter().find(|item| item.id == id).unwrap();
            assert_eq!(apply(&text, item), format!("//! {marker}{id}"));
        }
    }
}

#[test]
fn completion_hyphen_overlap_acceptance_resolves_and_formatting_replays_are_noops() {
    for (marker, trigger) in [("\u{a7}", "-"), ("-", "$$")] {
        let f = Fixture::new();
        f.config(&format!("[id]\nformat = \"{{kind}}-{{slug}}\"\n[reference]\nmarker = \"{marker}\"\ntrigger = \"{trigger}\"\n[fmt.cross_refs]\nenabled = false\n"));
        let snapshot = f.load();
        for introducer in [marker, trigger] {
            let before = "//! 😀 ";
            let old = format!("{introducer}FS-lost.1");
            let text = format!("{before}{old} tail");
            let cursor = before.len() + format!("{introducer}FS-lo").len();
            let items = f.items(&snapshot, "src/lib.rs", &text, 0, cursor);
            let item = items.iter().find(|item| item.id == "FS-login").unwrap();
            assert_eq!(
                (item.start, item.end),
                (before.len(), before.len() + old.len())
            );
            assert_eq!(item.filter_text, format!("{introducer}FS-login"));
            let accepted = apply(&text, item);
            assert_eq!(accepted, format!("{before}{marker}FS-login tail"));
            f.put("src/lib.rs", &accepted);
            let resolved = f.load();
            let cite = resolved
                .metadata
                .snapshot
                .citations
                .iter()
                .find(|cite| cite.path.ends_with("src/lib.rs"))
                .unwrap();
            assert_eq!(
                cite.target_path.as_ref().unwrap(),
                &f.0.join("docs/functional-spec/FS-login.md")
                    .canonicalize()
                    .unwrap()
            );
            for text in [accepted.clone(), format!("{accepted} and more")] {
                assert!(
                    super::on_type_line_edits(&f.0.join("src/lib.rs"), &text, 0, text.len(), &[])
                        .unwrap()
                        .is_empty()
                );
            }
            let formatted = crate::api::format_references(crate::api::FmtOpts {
                path: f.0.join("src/lib.rs"),
                path_provided: true,
                ..Default::default()
            })
            .unwrap();
            assert!(formatted.changes.is_empty());
        }
    }
}

#[test]
fn completion_overlap_uses_qualified_targets_and_per_kind_spellings() {
    let f = Fixture::new();
    f.config("project_name = \"root\"\n[workspace]\nmembers = [\"alpha\", \"beta\"]\ninclude_root = false\n");
    f.put("alpha/grund.toml", "grund_config_version = 1\nproject_name = \"alpha\"\n[id]\nformat = \"{kind}-{slug}\"\n[reference]\ntrigger = \"_\"\n");
    f.put("beta/grund.toml", "grund_config_version = 1\nproject_name = \"beta\"\n[id]\nformat = \"{kind}-{slug}\"\n[[kinds]]\nkind = \"FS\"\nfolder = \"docs/functional-spec\"\nformat = \"{kind}_{slug}\"\n");
    f.put("alpha/docs/notes.md", "\n");
    f.put(
        "beta/docs/functional-spec/FS-login.md",
        "# FS_login: Login\n\nLead.\n",
    );
    let snapshot = f.load();
    for introducer in ["\u{a7}", "_"] {
        for neighbor in ["\u{a7}beta/FS_login", "_beta/FS_login", "_beta/"] {
            let text = format!("{introducer}beta/FS_lost.1{neighbor}");
            let cursor = format!("{introducer}beta/FS_lo").len();
            let items = f.items(&snapshot, "alpha/docs/notes.md", &text, 0, cursor);
            assert_eq!(ids(&items), ["beta/FS_login"]);
            assert_eq!(
                apply(&text, &items[0]),
                format!("\u{a7}beta/FS_login{neighbor}")
            );
        }
    }
}
