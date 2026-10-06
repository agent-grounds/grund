//! Core authoring contracts (§FS-lsp.1.6.1, §FS-lsp.1.6.2,
//! §FS-lsp.1.6.3, §FS-lsp.1.6.4) independent of UTF-16 transport.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::{CitationCompletion, LspSnapshotOpts, LspSnapshotWithCompletion};
use crate::api::lsp_snapshot_with_completion;
use crate::testing::write;

pub(super) struct Fixture(pub PathBuf);

impl Fixture {
    pub fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = PathBuf::from(
            std::env::var_os("HOME")
                .or_else(|| std::env::var_os("USERPROFILE"))
                .expect("user home"),
        )
        .join("ag/tmp")
        .join(format!(
            "grund-core-completion-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let f = Self(root);
        f.config("[id]\nformat = \"{kind}-{slug}\"\n[fmt.cross_refs]\nenabled = false\n");
        f.put(
            "docs/functional-spec/FS-login.md",
            "# FS-login: Login\n\nLead.\n",
        );
        f.put(
            "docs/functional-spec/FS-logout.md",
            "# FS-logout: Logout\n\nLead.\n",
        );
        for path in ["docs/notes.md", "src/lib.rs", "src/lib.py"] {
            f.put(path, "\n");
        }
        f
    }

    pub fn config(&self, config: &str) {
        self.put("grund.toml", &format!("grund_config_version = 1\n{config}"));
    }
    pub fn put(&self, path: &str, text: &str) {
        write(&self.0.join(path), text);
    }
    pub fn load(&self) -> LspSnapshotWithCompletion {
        self.overlay(BTreeMap::new())
    }
    pub fn overlay(&self, open_documents: BTreeMap<PathBuf, String>) -> LspSnapshotWithCompletion {
        lsp_snapshot_with_completion(LspSnapshotOpts {
            path: self.0.clone(),
            path_provided: true,
            open_documents,
        })
        .unwrap()
    }
    pub fn items(
        &self,
        snapshot: &LspSnapshotWithCompletion,
        file: &str,
        text: &str,
        line: usize,
        cursor: usize,
    ) -> Vec<CitationCompletion> {
        snapshot
            .completion
            .complete(&self.0.join(file), text, line, cursor)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

pub(super) fn ids(items: &[CitationCompletion]) -> Vec<&str> {
    items.iter().map(|item| item.id.as_str()).collect()
}

pub(super) fn apply(text: &str, item: &CitationCompletion) -> String {
    format!("{}{}{}", &text[..item.start], item.text, &text[item.end..])
}

#[test]
fn completion_effective_empty_and_overlapping_introducers() {
    let f = Fixture::new();
    f.config("[id]\nformat = \"{kind}-{slug}\"\n[reference]\nmarker = \"@\"\ntrigger = \"@@@\"\nstrict = false\n");
    let snapshot = f.load();
    for text in ["@", "@F", "@@@", "@@@FS-lo"] {
        let items = f.items(&snapshot, "docs/notes.md", text, 0, text.len());
        assert_eq!(ids(&items), ["FS-login", "FS-logout"]);
        assert_eq!(apply(text, &items[0]), "@FS-login");
        assert_eq!(items[0].start, 0);
    }
    for text in ["F", "FS-lo", "$$F", "@fs", "@@@money"] {
        assert!(
            f.items(&snapshot, "docs/notes.md", text, 0, text.len())
                .is_empty()
        );
    }
    f.config("[id]\nformat = \"{kind}-{slug}\"\n[reference]\nmarker = \"\"\ntrigger = \"%%%\"\nstrict = false\n");
    let snapshot = f.load();
    assert_eq!(snapshot.completion.trigger_characters(), ["%"]);
    assert!(f.items(&snapshot, "docs/notes.md", "F", 0, 1).is_empty());
    let items = f.items(&snapshot, "docs/notes.md", "%%%F", 0, 4);
    assert_eq!(apply("%%%F", &items[0]), "FS-login");
}

#[test]
fn completion_refuses_formatter_scopes_and_accepts_full_docstring_context() {
    let f = Fixture::new();
    f.config("[id]\nformat = \"{kind}-{slug}\"\n[fmt]\nexclude = [\"docs/protected.md\"]\n");
    f.put("docs/protected.md", "\n");
    f.put("outside/notes.md", "\n");
    let snapshot = f.load();
    for (file, text, line, cursor) in [
        ("docs/notes.md", "`$$FS-lo`", 0, 8),
        ("docs/notes.md", "[label]($$FS-lo)", 0, 14),
        ("docs/notes.md", "```\n$$FS-lo\n```", 1, 7),
        ("docs/notes.md", "# FS-login: $$FS-lo", 0, 19),
        ("docs/notes.md", "<!-- grund:fmt off -->\n$$FS-lo", 1, 7),
        (
            "docs/notes.md",
            "<\u{a7}>$$FS-lo",
            0,
            "<\u{a7}>$$FS-lo".len(),
        ),
        ("docs/notes.md", "<\u{a7}>FS-lo", 0, "<\u{a7}>FS-lo".len()),
        ("src/lib.rs", "let x = \"$$FS-lo\";", 0, 16),
        ("src/lib.py", "value = \"\"\"\n$$FS-lo", 1, 7),
        ("docs/protected.md", "$$FS-lo", 0, 7),
        ("outside/notes.md", "$$FS-lo", 0, 7),
    ] {
        assert!(
            f.items(&snapshot, file, text, line, cursor).is_empty(),
            "{file}: {text}"
        );
    }
    for (file, text, line) in [
        ("src/lib.rs", "//! $$FS-lo", 0),
        ("src/lib.py", "\"\"\"Module docs.\n$$FS-lo", 1),
        ("src/lib.py", "def f():\n    \"\"\"Docs.\n    $$FS-lo", 2),
        (
            "docs/notes.md",
            "<!-- grund:fmt off -->\n<!-- grund:fmt on -->\n$$FS-lo",
            2,
        ),
    ] {
        let cursor = text.lines().nth(line).unwrap().len();
        assert_eq!(
            ids(&f.items(&snapshot, file, text, line, cursor)),
            ["FS-login", "FS-logout"]
        );
    }
}

#[test]
fn completion_byte_ranges_consume_suffix_sections_and_preserve_delimiters() {
    let f = Fixture::new();
    let snapshot = f.load();
    for (before, old, after) in [
        ("😀 See ", "$$FS-lost.1", " and \u{a7}FS-logout"),
        ("[", "$$FS-lost.1", "](target.md) after"),
        ("", "$$FS-lost", "\u{a7}FS-logout"),
        ("", "$$FS-lost", ". Next sentence"),
        ("", "$$FS-lost", ".Next sentence"),
    ] {
        let text = format!("{before}{old}{after}");
        let items = f.items(
            &snapshot,
            "docs/notes.md",
            &text,
            0,
            before.len() + "$$FS-lo".len(),
        );
        assert_eq!(
            (items[0].start, items[0].end),
            (before.len(), before.len() + old.len())
        );
        assert_eq!(
            apply(&text, &items[0]),
            format!("{before}\u{a7}FS-login{after}")
        );
    }
    for text in ["$$FS-login.1", "$$FS-login."] {
        assert!(
            f.items(&snapshot, "docs/notes.md", text, 0, text.len())
                .is_empty()
        );
    }
    assert!(
        f.items(&snapshot, "docs/notes.md", "😀 $$F", 0, 1)
            .is_empty()
    );
    assert!(f.items(&snapshot, "docs/notes.md", "$$F", 0, 9).is_empty());
}

#[test]
fn completion_metadata_order_legacy_and_ambiguous_discovery() {
    let f = Fixture::new();
    f.config("[id]\nformat = \"{kind}-{number}-{slug}\"\n[[kinds]]\nkind = \"FS\"\nfolder = \"docs/functional-spec\"\nformat = \"{kind}_{slug}\"\n[[kinds]]\nkind = \"GOAL\"\nfile = \"docs/goals.md\"\n");
    f.put(
        "docs/functional-spec/FS-custom.md",
        "# FS_custom: Custom\n\nLead.\n",
    );
    f.put("docs/goals.md", "# GOAL-007-live: Live\n\nLead.\n");
    let snapshot = f.load();
    let items = f.items(&snapshot, "docs/notes.md", "$$", 0, 2);
    assert_eq!(
        ids(&items),
        ["FS-login", "FS-logout", "FS_custom", "GOAL-007-live"]
    );
    assert!(
        items
            .windows(2)
            .all(|pair| pair[0].sort_text < pair[1].sort_text)
    );
    assert_eq!(items[0].title, "Login");
    assert_eq!(items[0].source_path, "docs/functional-spec/FS-login.md");
    assert_eq!(items[0].filter_text, "$$FS-login");
    f.put(
        "docs/functional-spec/FS-duplicate.md",
        "# FS-login: Duplicate\n\nLead.\n",
    );
    let snapshot = f.load();
    assert_eq!(
        ids(&f.items(&snapshot, "docs/notes.md", "$$FS-lo", 0, 7)),
        ["FS-logout"]
    );
    std::fs::write(f.0.join("docs/functional-spec/FS-bad.md"), [0xff]).unwrap();
    let snapshot = f.load();
    assert!(f.items(&snapshot, "docs/notes.md", "$$F", 0, 3).is_empty());
}

#[test]
fn completion_exact_matches_precede_extensions() {
    let f = Fixture::new();
    f.put(
        "docs/functional-spec/FS-login-extra.md",
        "# FS-login-extra: Extra\n\nLead.\n",
    );
    let snapshot = f.load();
    let items = f.items(&snapshot, "docs/notes.md", "$$FS-login", 0, 10);
    assert_eq!(ids(&items), ["FS-login", "FS-login-extra"]);
    assert!(items[0].sort_text < items[1].sort_text);
}

#[test]
fn completion_section_separator_can_also_be_an_id_literal() {
    let f = Fixture::new();
    f.config("[id]\nformat = \"{kind}-{slug}\"\nsection_separator = \"-\"\n");
    f.put(
        "docs/functional-spec/FS-login-extra.md",
        "# FS-login-extra: Extra\n\nLead.\n",
    );
    let snapshot = f.load();
    let text = "$$FS-login-e";
    let items = f.items(&snapshot, "docs/notes.md", text, 0, text.len());
    assert_eq!(ids(&items), ["FS-login-extra"]);
    assert_eq!(apply(text, &items[0]), "\u{a7}FS-login-extra");
}

#[test]
fn completion_stub_inline_pair_is_one_home_and_broken_stub_is_withheld() {
    let f = Fixture::new();
    f.put(
        "docs/functional-spec/FS-pair.md",
        "# FS-pair: [src/pair.rs](src/pair.rs)\n",
    );
    f.put(
        "src/pair.rs",
        "/// FS-pair: Pair title\n///\n/// Lead.\npub struct Pair;\n",
    );
    f.put(
        "docs/functional-spec/FS-missing.md",
        "# FS-missing: [src/missing.rs](src/missing.rs)\n",
    );
    let snapshot = f.load();
    let items = f.items(&snapshot, "docs/notes.md", "$$FS-p", 0, 6);
    assert_eq!(ids(&items), ["FS-pair"]);
    assert_eq!(items[0].title, "Pair title");
    assert_eq!(items[0].source_path, "src/pair.rs");
    assert!(
        f.items(&snapshot, "docs/notes.md", "$$FS-m", 0, 6)
            .is_empty()
    );
}

#[test]
fn completion_reuses_snapshot_and_notified_overlays_restore_on_close() {
    let f = Fixture::new();
    let initial = f.load();
    // Old public record construction remains available with no extra fields.
    let _old = super::LspSnapshotWithMetadata {
        snapshot: initial.metadata.snapshot.clone(),
        kind_titles: initial.metadata.kind_titles.clone(),
    };
    let decl = f.0.join("docs/functional-spec/FS-login.md");
    std::fs::remove_file(&decl).unwrap();
    f.config("[id]\nformat = \"{kind}-{slug}\"\n[reference]\nmarker = \"@\"\ntrigger = \"%%\"\n");
    let reused = f.items(&initial, "docs/notes.md", "$$FS-lo", 0, 7);
    assert_eq!(ids(&reused), ["FS-login", "FS-logout"]);
    assert_eq!(
        reused[0].text, "\u{a7}FS-login",
        "requests do not reload config or scan"
    );
    f.put(
        "docs/functional-spec/FS-login.md",
        "# FS-login: Login\n\nLead.\n",
    );
    let changed = f.overlay(BTreeMap::from([(
        decl,
        "# FS-loaded: Unsaved\n\nLead.\n".into(),
    )]));
    let items = f.items(&changed, "docs/notes.md", "%%FS-lo", 0, 7);
    assert_eq!(ids(&items), ["FS-loaded", "FS-logout"]);
    assert_eq!(items[0].title, "Unsaved");
    assert_eq!(items[0].text, "@FS-loaded");
    let closed = f.load();
    assert_eq!(
        ids(&f.items(&closed, "docs/notes.md", "%%FS-lo", 0, 7)),
        ["FS-login", "FS-logout"]
    );
}

#[test]
fn completion_acceptance_then_on_type_and_bulk_formatting_are_noops() {
    let f = Fixture::new();
    let snapshot = f.load();
    let original = "//! 😀 $$FS-lo";
    let items = f.items(&snapshot, "src/lib.rs", original, 0, original.len());
    let accepted = apply(original, &items[0]);
    f.put("src/lib.rs", &accepted);
    for text in [accepted.clone(), format!("{accepted} and more")] {
        assert!(
            super::on_type_line_edits(&f.0.join("src/lib.rs"), &text, 0, text.len(), &[])
                .unwrap()
                .is_empty()
        );
    }
    let snapshot = f.load();
    let cite = snapshot
        .metadata
        .snapshot
        .citations
        .iter()
        .find(|cite| cite.path.ends_with("src/lib.rs"))
        .unwrap();
    assert!(cite.target_path.as_ref().unwrap().ends_with("FS-login.md"));
    let formatted = crate::api::format_references(crate::api::FmtOpts {
        path: f.0.join("src/lib.rs"),
        path_provided: true,
        ..Default::default()
    })
    .unwrap();
    assert!(formatted.changes.is_empty());
}
