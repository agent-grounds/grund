//! Local-to-full formatter rewrites and protected locations
//! (§FS-fmt.2.3, §FS-fmt.2.4, §DF-declaration-local-section-shorthand).

use crate::api::{FmtOpts, format_references};
use crate::testing::{test_root, write};

fn fixture(name: &str) -> std::path::PathBuf {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        concat!(
            "grund_config_version = 1\n",
            "[reference]\nstrict = true\n",
            "[fmt.cross_refs]\nenabled = false\n",
            "[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n",
            "[scan]\ninclude = [\"docs\"]\nextensions = [\"md\"]\n",
        ),
    );
    write(
        &root.join("docs/FS-alpha.md"),
        concat!(
            "# FS-alpha: Alpha\n\n",
            "Rewrite \u{a7}2 and \u{a7}2.1.\n",
            "Malformed \u{a7}2..1, \u{a7}2..., and \u{a7}2..goals.\n",
            "Protected `\u{a7}2`, [link](\u{a7}2), and ownerless stays elsewhere.\n\n",
            "```text\n\u{a7}2\n```\n\n",
            "## 2. Target\n\n### 2.1 Child\n",
        ),
    );
    write(
        &root.join("docs/notes.md"),
        "# Notes\n\nOwnerless \u{a7}2.\n",
    );
    root
}

#[test]
fn formatter_dry_run_names_only_safe_owned_local_replacements() {
    let root = fixture("formatter_dry_run_names_only_safe_owned_local_replacements");
    let output = format_references(FmtOpts {
        path: root,
        path_provided: true,
        write: false,
        add_marker: false,
        cross_refs: false,
    })
    .expect("format dry run");
    assert_eq!(
        output
            .changes
            .iter()
            .map(|change| (change.path.as_str(), change.line, change.label.as_str()))
            .collect::<Vec<_>>(),
        [
            (
                "docs/FS-alpha.md",
                3,
                "local section → canonical: \u{a7}2 → \u{a7}FS-alpha.2"
            ),
            (
                "docs/FS-alpha.md",
                3,
                "local section → canonical: \u{a7}2.1 → \u{a7}FS-alpha.2.1"
            ),
        ]
    );
}

#[test]
fn formatter_write_expands_owned_local_paths_and_preserves_protected_sites() {
    let root = fixture("formatter_write_expands_owned_local_paths_and_preserves_protected_sites");
    format_references(FmtOpts {
        path: root.clone(),
        path_provided: true,
        write: true,
        add_marker: false,
        cross_refs: false,
    })
    .expect("format write");
    let written = std::fs::read_to_string(root.join("docs/FS-alpha.md")).expect("read result");
    assert!(written.contains("Rewrite \u{a7}FS-alpha.2 and \u{a7}FS-alpha.2.1."));
    assert!(written.contains("Malformed \u{a7}2..1, \u{a7}2..., and \u{a7}2..goals."));
    assert!(written.contains("Protected `\u{a7}2`, [link](\u{a7}2)"));
    assert!(written.contains("```text\n\u{a7}2\n```"));
    assert_eq!(
        std::fs::read_to_string(root.join("docs/notes.md")).expect("read ownerless"),
        "# Notes\n\nOwnerless \u{a7}2.\n"
    );
}
