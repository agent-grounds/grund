//! Scan-root regressions for directory links that cross the canonical project
//! boundary (§FS-config.3.5.1, §AR-scanner.1.6). Descent cases remain in
//! `tests_walk.rs`; these start the walk at or below the link, which is the
//! distinction the gate turns on: a root that *is* an outward link is refused,
//! a root written *below* one is canonicalized before the walk begins and is
//! ordinary scope (§DF-undeclared-blind-spots). The gate also records a file root
//! the extension filter or the hidden-file rule refuses, with its row, for `check`
//! to name (§FS-scan.checks.unread-scan-root).

use crate::model::{Refused, RootOrigin};
use crate::testing::{
    canonical_test_path, legacy_fs_folder_config, scan_tree, scanned, symlink, test_root, write,
};

/// §AR-scanner.1.6 skips a canonical root outside the project "when the written
/// root is a symlink; a plain parent-relative root remains intentional scan
/// scope". A root spelled below a linked ancestor is neither: it is
/// canonicalized before the walk starts, so the walk traverses no link and
/// §FS-config.3.5.1's third sentence — the gate on a root that *is* a directory
/// symlink — does not reach it. Refusing it left a kind home behind a linked
/// parent unread with nothing saying so (§DF-undeclared-blind-spots). The file
/// keeps the spelling the root gave it, as §AR-scanner.1.8 asks of any root.
#[test]
fn a_scan_root_below_an_outward_directory_link_is_ordinary_scope() {
    let base = test_root("a_scan_root_below_an_outward_directory_link_is_ordinary_scope");
    let root = base.join("project");
    write(
        &base.join("external/nested/FS-001-foreign.md"),
        "# FS-001-foreign: Foreign\n",
    );
    symlink("../external", &root.join("linked"));

    let mut configured = legacy_fs_folder_config(root.clone());
    configured.include = Some(vec!["linked/nested".into()]);
    let (configured_findings, _) = scan_tree(&configured, None, false).expect("configured root");
    let explicit = legacy_fs_folder_config(root);
    let (explicit_findings, _) =
        scan_tree(&explicit, Some(&explicit.root.join("linked/nested")), true)
            .expect("explicit root");

    assert_eq!(
        scanned(&configured, &configured_findings),
        vec!["linked/nested/FS-001-foreign.md"],
        "§AR-scanner.1.6: a configured descendant of a linked ancestor is intentional scan scope"
    );
    assert_eq!(
        scanned(&explicit, &explicit_findings),
        vec!["linked/nested/FS-001-foreign.md"],
        "§AR-scanner.1.6: an explicit descendant of a linked ancestor is the same scope"
    );
}

/// The half of §FS-config.3.5.1's third sentence the fixture above does not
/// reach: the written root *is* the outward directory link, configured rather
/// than handed in, and the gate still refuses it.
#[test]
fn a_configured_directory_link_root_leaving_the_project_is_pruned() {
    let base = test_root("a_configured_directory_link_root_leaving_the_project_is_pruned");
    let root = base.join("project");
    write(
        &base.join("external/FS-001-foreign.md"),
        "# FS-001-foreign: Foreign\n",
    );
    symlink("../external", &root.join("linked"));

    let mut config = legacy_fs_folder_config(root);
    config.include = Some(vec!["linked".into()]);
    let (findings, _) = scan_tree(&config, None, false).expect("configured linked root");

    assert!(
        scanned(&config, &findings).is_empty(),
        "§FS-config.3.5.1: the gate applies when a configured scan root is itself a directory symlink"
    );
}

#[test]
fn an_external_explicit_directory_link_root_is_pruned_before_resolution() {
    let base = test_root("an_external_explicit_directory_link_root_is_pruned_before_resolution");
    let root = base.join("project");
    write(
        &base.join("external/FS-001-foreign.md"),
        "# FS-001-foreign: Foreign\n",
    );
    symlink("external", &base.join("external-link"));
    std::fs::create_dir_all(&root).expect("create project root");
    let config = legacy_fs_folder_config(root);

    let (findings, _) =
        scan_tree(&config, Some(&base.join("external-link")), true).expect("explicit linked root");

    assert!(
        scanned(&config, &findings).is_empty(),
        "§FS-config.3.5.1: root selection keeps enough lexical identity to prune an explicit outward directory link"
    );
}

#[test]
fn the_root_fence_preserves_intentional_external_and_in_root_scope() {
    let base = test_root("the_root_fence_preserves_intentional_external_and_in_root_scope");
    let root = base.join("project");
    write(
        &base.join("external/FS-001-external.md"),
        "# FS-001-external: External\n",
    );
    write(
        &root.join("real/FS-002-local.md"),
        "# FS-002-local: Local\n",
    );
    write(
        &base.join("FS-003-file.md"),
        "# FS-003-file: External file\n",
    );
    symlink("real", &root.join("local-link"));
    symlink("../FS-003-file.md", &root.join("linked-file.md"));
    let config = legacy_fs_folder_config(root.clone());

    let (external, _) =
        scan_tree(&config, Some(&base.join("external")), true).expect("plain external root");
    let (in_root_link, _) =
        scan_tree(&config, Some(&root.join("local-link")), true).expect("in-root linked root");
    let (file_link, _) =
        scan_tree(&config, Some(&root.join("linked-file.md")), true).expect("outward file link");

    assert_eq!(
        scanned(&config, &external),
        vec![
            canonical_test_path(&base.join("external/FS-001-external.md"))
                .display()
                .to_string()
        ],
        "§FS-config.3.5.1: a plain non-link external root remains intentional scope"
    );
    assert_eq!(
        scanned(&config, &in_root_link),
        vec!["local-link/FS-002-local.md"],
        "§FS-config.3.5.1: a directory link whose target stays in the project remains readable"
    );
    assert_eq!(
        scanned(&config, &file_link),
        vec!["linked-file.md"],
        "§FS-config.3.5.1: the directory boundary leaves external file links readable"
    );
}

/// A project whose `grund.toml` declares a `TXT` row with `home` as its `file`, on
/// line 6, and `[scan] include = ["docs", <include>]` on line 9, with only `md` listed.
fn unread_root_project(name: &str, home: &str, include: &str) -> crate::config::Config {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        &format!(
            "[id]\nformat = \"{{kind}}-{{slug}}\"\n\n[[kinds]]\nkind = \"TXT\"\nfile = \"{home}\"\n\n\
             [scan]\ninclude = [\"docs\", \"{include}\"]\nextensions = [\"md\"]\n"
        ),
    );
    write(&root.join("docs/a.md"), "# A\n");
    crate::config::load_config_at(&root, &root).expect("load the project")
}

/// §AR-scanner.1.6: the root gate records a file root the extension filter refuses
/// with the row that made it a root and the line of its key, the predicate's own
/// `Unread::Extension` answer (§AR-scanner.1.13, §FS-scan.checks.unread-scan-root).
#[test]
fn a_refused_file_root_is_recorded_with_its_row() {
    let config = unread_root_project(
        "a_refused_file_root_is_recorded_with_its_row",
        "notes.txt",
        "extra.txt",
    );
    write(&config.root.join("notes.txt"), "TXT-a\n");
    write(&config.root.join("extra.txt"), "extra\n");
    let (findings, _) = scan_tree(&config, None, false).expect("scan");
    let rows: Vec<_> = findings
        .unread_roots
        .iter()
        .map(|row| (row.origin.clone(), row.written.as_str(), row.line))
        .collect();
    let home = RootOrigin::Home {
        kind: "TXT".into(),
        citable: true,
    };
    assert_eq!(
        rows,
        vec![
            (RootOrigin::Include, "extra.txt", 9),
            (home, "notes.txt", 6)
        ],
        "§AR-scanner.1.6: each refused file root, with its row, in the roots' order"
    );
    let extension_only = Refused {
        hidden: false,
        extension: true,
    };
    assert!(
        findings
            .unread_roots
            .iter()
            .all(|row| row.refused == extension_only),
        "§AR-scanner.1.13: a name that is not hidden leaves the extension the only refusal"
    );
}

/// §FS-scan.checks.unread-scan-root.silent: a directory root is filtered file by
/// file, which is what the extension list means, so the gate records nothing.
#[test]
fn a_directory_root_is_not_recorded() {
    let config = unread_root_project("a_directory_root_is_not_recorded", "notes.md", "texts");
    write(&config.root.join("texts/a.txt"), "text\n");
    let (findings, _) = scan_tree(&config, None, false).expect("scan");
    assert!(
        findings.unread_roots.is_empty(),
        "§FS-scan.checks.unread-scan-root.silent"
    );
}

/// §FS-scan.scope.hidden: a file root whose own name is hidden is not read, and the
/// gate records it with its row like an extension refusal (§AR-scanner.1.13), whether
/// a `[[kinds]]` home or an `include` entry named it (§FS-scan.checks.unread-scan-root).
#[test]
fn a_hidden_named_root_is_recorded_with_its_row() {
    let config = unread_root_project(
        "a_hidden_named_root_is_recorded_with_its_row",
        ".notes.md",
        ".extra.md",
    );
    write(&config.root.join(".notes.md"), "TXT-a\n");
    write(&config.root.join(".extra.md"), "extra\n");
    let (findings, _) = scan_tree(&config, None, false).expect("scan");
    let rows: Vec<_> = findings
        .unread_roots
        .iter()
        .map(|row| (row.origin.clone(), row.written.as_str(), row.line))
        .collect();
    let home = RootOrigin::Home {
        kind: "TXT".into(),
        citable: true,
    };
    assert_eq!(
        rows,
        vec![
            (RootOrigin::Include, ".extra.md", 9),
            (home, ".notes.md", 6)
        ],
        "§FS-scan.scope.hidden: a hidden file root is named at its row"
    );
    let hidden_only = Refused {
        hidden: true,
        extension: false,
    };
    assert!(
        findings
            .unread_roots
            .iter()
            .all(|row| row.refused == hidden_only),
        "§AR-scanner.1.6: a listed extension leaves the hidden rule the only refusal"
    );
}

/// §DF-unread-hidden-root.2.1: a root both hidden and unlisted is one unread root, so
/// the gate records it once rather than once per rule that refuses it.
#[test]
fn a_root_both_hidden_and_unlisted_is_recorded_once() {
    let config = unread_root_project(
        "a_root_both_hidden_and_unlisted_is_recorded_once",
        ".notes.txt",
        "docs",
    );
    write(&config.root.join(".notes.txt"), "TXT-a\n");
    let (findings, _) = scan_tree(&config, None, false).expect("scan");
    let rows: Vec<_> = findings
        .unread_roots
        .iter()
        .map(|row| (row.written.as_str(), row.line))
        .collect();
    assert_eq!(
        rows,
        vec![(".notes.txt", 6)],
        "§FS-scan.checks.unread-scan-root: one record for one root"
    );
    let both = Refused {
        hidden: true,
        extension: true,
    };
    assert_eq!(
        findings.unread_roots[0].refused, both,
        "§DF-unread-hidden-root.2.1: the one record names both rules"
    );
}

/// §FS-scan.checks.unread-scan-root.silent: a dotfile inside a directory root was
/// named by nothing in the configuration, so the gate records nothing for it.
#[test]
fn a_dotfile_inside_a_directory_root_is_not_recorded() {
    let config = unread_root_project(
        "a_dotfile_inside_a_directory_root_is_not_recorded",
        "notes.md",
        "docs",
    );
    write(&config.root.join("docs/.notes.md"), "TXT-a\n");
    let (findings, _) = scan_tree(&config, None, false).expect("scan");
    assert!(
        findings.unread_roots.is_empty(),
        "§FS-scan.checks.unread-scan-root.silent"
    );
}

/// §FS-values.2.2: a value row's JSON home is read by the value pass rather than the
/// walk, so although the extension list leaves `json` out, the file is read and the
/// gate records nothing: nothing about it is unchecked.
#[test]
fn a_value_rows_json_home_is_not_recorded() {
    let root = test_root("a_value_rows_json_home_is_not_recorded");
    write(
        &root.join("grund.toml"),
        "[id]\nformat = \"{kind}-{slug}\"\n\n[[kinds]]\nkind = \"CONST\"\nfile = \"values.json\"\n\
         values = true\n\n[scan]\ninclude = []\nextensions = [\"md\"]\n",
    );
    write(&root.join("values.json"), "{\"CONST-a\": 1}\n");
    let config = crate::config::load_config_at(&root, &root).expect("load the project");
    let (findings, _) = scan_tree(&config, None, false).expect("scan");
    assert!(
        findings.unread_roots.is_empty(),
        "§FS-values.2.2: the value pass reads it"
    );
}
