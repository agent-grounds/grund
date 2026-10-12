//! Test module: `kinds = []`, the empty kind set (§FS-config.3.4.4.5) — distinct
//! from an omitted `[[kinds]]`, which keeps the built-in rows with the older `FS`
//! home (§FS-config.3.4.4.4); refused when it is not empty or a table follows it;
//! and the meaning each template's config has before `init` writes it
//! (§FS-init.1.6.1, §FS-init.2.4.9).

use std::path::PathBuf;

use crate::config::{Config, load_config};
use crate::templates::{Template, render_grund_toml};
use crate::testing::{test_root, write};

fn load(name: &str, text: &str) -> anyhow::Result<Config> {
    let root = test_root(name);
    write(&root.join("grund.toml"), text);
    load_config(&root)
}

fn refusal(name: &str, text: &str) -> String {
    match load(name, text) {
        Ok(_) => panic!("{name}: loaded, expected a refusal"),
        Err(err) => format!("{err:#}"),
    }
}

/// §FS-config.3.4.4.5: `kinds = []` and an omitted `[[kinds]]` load to
/// different kind sets — none at all, and the nine built-in rows with the
/// older `FS` folder.
#[test]
fn an_empty_kind_set_is_not_an_omitted_one() {
    let empty = load(
        "empty_kinds_declared",
        "grund_config_version = 1\nkinds = []\n",
    )
    .unwrap_or_else(|err| panic!("{err:#}"));
    assert!(empty.kinds.is_empty(), "{:?}", empty.kinds);
    assert!(empty.project().schema.rows.is_empty());
    assert!(empty.project().presentation.kinds.is_empty());
    assert!(empty.grammar.parse_token("FS-001-login").is_none());

    let omitted = load("empty_kinds_omitted", "grund_config_version = 1\n").expect("loads");
    assert_eq!(omitted.kinds.len(), 9);
    let fs = omitted.kinds.iter().find(|kind| kind.kind == "FS");
    assert_eq!(
        fs.and_then(|kind| kind.folder.as_deref()),
        Some("docs/functional-spec"),
        "an omitted [[kinds]] keeps the older FS home"
    );
}

/// §FS-config.3.4.4.5: only the empty array is the empty kind set, spaces
/// inside it included; a non-empty one is a kind table written the wrong way.
#[test]
fn only_the_empty_array_is_accepted() {
    load("empty_kinds_spaced", "kinds = [ ]\n").unwrap_or_else(|err| panic!("{err:#}"));
    let message = refusal("empty_kinds_non_empty", "kinds = [\"FS\"]\n");
    assert!(
        message.contains(
            "grund.toml:1: `kinds` takes only the empty list `kinds = []`, which declares no kinds; declare each kind as a `[[kinds]]` table"
        ),
        "{message}"
    );
}

/// §FS-config.3.4.4.5: a table after the empty set would append to a static
/// array, refused at the table's header; the key under a section stays unknown.
#[test]
fn a_kinds_table_after_the_empty_set_is_refused() {
    let message = refusal(
        "empty_kinds_then_table",
        "kinds = []\n\n[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\n",
    );
    assert!(
        message.contains(
            "grund.toml:3: `[[kinds]]` cannot follow `kinds = []` (line 1), which declares no kinds; delete `kinds = []` to declare kinds as tables"
        ),
        "{message}"
    );
    let nested = refusal("empty_kinds_in_a_section", "[reference]\nkinds = []\n");
    assert!(nested.contains("unknown config key `kinds`"), "{nested}");
}

/// §FS-config.3.4.4.5: the empty set is the one kind set with no citable kind
/// v1 loads; a table whose every row is non-citable is still refused.
#[test]
fn a_table_of_non_citable_kinds_is_still_refused() {
    refusal(
        "empty_kinds_only_non_citable",
        "[[kinds]]\nkind = \"e2e\"\nfolder = \"tests\"\ncitable = false\n",
    );
}

/// §FS-init.1.6.1, §FS-init.2.4.9: each template's config, read back, means
/// the config `init` rendered the managed block from before writing it.
#[test]
fn each_template_config_means_its_pending_config() {
    let root = PathBuf::from(".");
    let cases = [
        (
            Template::SoftwareDevelopment,
            Config::default_for(root.clone()),
        ),
        (Template::None, Config::without_kinds_for(root)),
    ];
    for (template, pending) in cases {
        let written = load(
            &format!("empty_kinds_template_{}", template.name()),
            &render_grund_toml(template, "demo", None),
        )
        .unwrap_or_else(|err| panic!("{}: {err:#}", template.name()));
        assert_eq!(
            written.project().schema.rows,
            pending.project().schema.rows,
            "{}",
            template.name()
        );
        assert_eq!(
            written.project().presentation.kinds,
            pending.project().presentation.kinds,
            "{}",
            template.name()
        );
    }
}
