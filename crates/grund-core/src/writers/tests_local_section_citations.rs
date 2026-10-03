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

/// One row per line names every safe owned local expansion, in source order (§FS-fmt.3.6.1).
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
        [(
            "docs/FS-alpha.md",
            3,
            "local section → canonical: \u{a7}2 → \u{a7}FS-alpha.2, \u{a7}2.1 → \u{a7}FS-alpha.2.1"
        )]
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

/// §FS-fmt.2.4.6's three unresolving shapes — a wholly absent path, a partially
/// resolving one, and an owner with no numbered sections at all — plus a
/// resolving control, and one line carrying a control and a refusal together so
/// a refused token cannot take its whole line with it.
fn absent_target_fixture(name: &str) -> std::path::PathBuf {
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
        &root.join("docs/FS-nine.md"),
        concat!(
            "# FS-nine: The owner stops short\n\n",
            "Resolving control \u{a7}2.1.\n",
            "Wholly absent \u{a7}12.\n",
            "Partly resolving \u{a7}2.7.\n",
            "Both \u{a7}2.1 and \u{a7}12 on one line.\n\n",
            "## 2. Target\n\n### 2.1 Child\n",
        ),
    );
    write(
        &root.join("docs/FS-bare.md"),
        "# FS-bare: No numbered section at all\n\nNone to resolve against \u{a7}1.\n",
    );
    root
}

/// §FS-fmt.2.4.6: the preview names only the sites the write will touch
/// (§FS-fmt.7.3), so a refused candidate is an absence here rather than a
/// labelled row. The label text itself is left exactly as the command writes it
/// today; what this case is about is which rows exist.
#[test]
fn the_dry_run_omits_a_local_path_its_owner_has_no_heading_for() {
    let root = absent_target_fixture("the_dry_run_omits_a_local_path_its_owner_has_no_heading_for");
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
                "docs/FS-nine.md",
                3,
                "local section → canonical: \u{a7}2.1 → \u{a7}FS-nine.2.1"
            ),
            (
                "docs/FS-nine.md",
                6,
                "local section → canonical: \u{a7}2.1 → \u{a7}FS-nine.2.1"
            ),
        ]
    );
}

/// §FS-fmt.2.4.6: the write leaves each unresolving candidate byte-identical and
/// still expands the control, including where the two share a line. `FS-bare.md`
/// holds nothing else, so it comes out of the run byte-for-byte as it went in.
#[test]
fn the_write_declines_a_local_path_its_owner_has_no_heading_for() {
    let root =
        absent_target_fixture("the_write_declines_a_local_path_its_owner_has_no_heading_for");
    format_references(FmtOpts {
        path: root.clone(),
        path_provided: true,
        write: true,
        add_marker: false,
        cross_refs: false,
    })
    .expect("format write");
    assert_eq!(
        std::fs::read_to_string(root.join("docs/FS-nine.md")).expect("read owner"),
        concat!(
            "# FS-nine: The owner stops short\n\n",
            "Resolving control \u{a7}FS-nine.2.1.\n",
            "Wholly absent \u{a7}12.\n",
            "Partly resolving \u{a7}2.7.\n",
            "Both \u{a7}FS-nine.2.1 and \u{a7}12 on one line.\n\n",
            "## 2. Target\n\n### 2.1 Child\n",
        )
    );
    assert_eq!(
        std::fs::read_to_string(root.join("docs/FS-bare.md")).expect("read bare owner"),
        "# FS-bare: No numbered section at all\n\nNone to resolve against \u{a7}1.\n"
    );
}
