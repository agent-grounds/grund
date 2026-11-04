//! A rule kind's folder home the run reads no file of (§FS-scan.checks.unread-scan-root.rule-folder):
//! the walk's own loop credits each file to the homes that hold it, whichever root reached it,
//! and the verdict comes after every root (§AR-scanner.1.6, §AR-scanner.1.13).

use crate::testing::{scan_tree, test_root, write};

/// A version-1 project, with no `[scan] include`, whose rule kind's `folder` key is on line 6.
const RULE_FOLDER_V1: &str = "[id]\nformat = \"{kind}-{slug}\"\n\n[[kinds]]\nkind = \"RULE\"\n\
                              folder = \"rules\"\nrules = true\n\n[scan]\nextensions = [\"rs\"]\n";
/// A version-2 project whose rule kind's `folders` key is on line 8.
pub(super) const RULE_FOLDER_V2: &str = "grund_config_version = 2\nproject_name = \"demo\"\n\n\
                              [schema.sources]\nlanguages = [\"rust\"]\n\n\
                              [schema.kinds.RULE]\nfolders = [\"rules\"]\nform = \"rule\"\n";

/// `toml` over `files`, each file holding a rule heading.
pub(super) fn rule_folder_project(name: &str, toml: &str, files: &[&str]) -> crate::config::Config {
    let root = test_root(name);
    write(&root.join("grund.toml"), toml);
    for file in files {
        write(
            &root.join(file),
            "# RULE-a: Each FS must cite at least one RULE.\n",
        );
    }
    crate::config::load_config_at(&root, &root).expect("load the project")
}

/// What the run records, as the configuration wrote it and at which line.
pub(super) fn recorded(config: &crate::config::Config) -> Vec<(String, usize)> {
    let (findings, _) = scan_tree(config, None, false).expect("scan");
    findings
        .unread_roots
        .iter()
        .map(|row| (row.written.clone(), row.line))
        .collect()
}

/// §FS-scan.checks.unread-scan-root.rule-folder: nothing in the folder was read and one file
/// was refused by its extension, in the walk's own loop (§AR-scanner.1.13).
#[test]
fn an_unread_rule_folder_is_recorded_with_its_key_line() {
    let v2 = rule_folder_project(
        "an_unread_rule_folder_is_recorded_v2",
        RULE_FOLDER_V2,
        &["rules/a.md"],
    );
    assert_eq!(
        recorded(&v2),
        vec![("rules".to_string(), 8)],
        "§FS-scan.checks.unread-scan-root.location: the `folders` key"
    );
    let v1 = rule_folder_project(
        "an_unread_rule_folder_is_recorded_v1",
        RULE_FOLDER_V1,
        &["rules/a.md"],
    );
    assert_eq!(
        recorded(&v1),
        vec![("rules".to_string(), 6)],
        "§FS-scan.checks.unread-scan-root.location: the `folder` key"
    );
}

/// §AR-scanner.1.6: a file is credited to the home that holds it, so a folder under an
/// `include` root, reached by that root and as a root of its own, is one folder.
#[test]
fn a_rule_folder_under_an_include_root_is_recorded_once() {
    let config = rule_folder_project(
        "a_rule_folder_under_an_include_root_is_recorded_once",
        &RULE_FOLDER_V1.replace("\"rules\"", "\"docs/rules\""),
        &["docs/rules/a.md"],
    );
    assert_eq!(
        recorded(&config),
        vec![("docs/rules".to_string(), 6)],
        "§FS-scan.checks.unread-scan-root.rule-folder: the default `include` holds `docs`"
    );
}

/// §AR-scanner.1.6: `--full` adds the config root, where a double count or warning would show.
#[test]
fn a_rule_folder_under_full_is_recorded_once() {
    let mut v1 = rule_folder_project(
        "a_rule_folder_under_full_is_recorded_once_v1",
        RULE_FOLDER_V1,
        &["rules/a.md"],
    );
    v1.set_scan_full(true);
    assert_eq!(
        recorded(&v1),
        vec![("rules".to_string(), 6)],
        "§FS-scan.checks.unread-scan-root.scope"
    );
    let mut v2 = rule_folder_project(
        "a_rule_folder_under_full_is_recorded_once_v2",
        RULE_FOLDER_V2,
        &["rules/a.md"],
    );
    v2.set_scan_full(true);
    assert_eq!(
        recorded(&v2),
        vec![("rules".to_string(), 8)],
        "§FS-scan.checks.unread-scan-root.scope"
    );
}

/// §FS-scan.scope.roots: every walked home is a root, and `exclude` never prunes a root, so
/// naming the folder itself in `exclude` stops nothing, in either version.
#[test]
fn a_rule_folder_exclude_names_is_still_recorded_in_both_versions() {
    let v1 = rule_folder_project(
        "a_rule_folder_exclude_names_v1",
        &format!("{RULE_FOLDER_V1}exclude = [\"rules\"]\n"),
        &["rules/a.md"],
    );
    assert_eq!(
        recorded(&v1),
        vec![("rules".to_string(), 6)],
        "§FS-scan.checks.unread-scan-root.rule-folder: version 1"
    );
    let v2 = rule_folder_project(
        "a_rule_folder_exclude_names_v2",
        &RULE_FOLDER_V2.replace(
            "languages = [\"rust\"]\n",
            "languages = [\"rust\"]\nexclude = [\"rules\"]\n",
        ),
        &["rules/a.md"],
    );
    assert_eq!(
        recorded(&v2),
        vec![("rules".to_string(), 9)],
        "§FS-scan.checks.unread-scan-root.rule-folder: version 2"
    );
}

/// §FS-scan.checks.unread-scan-root.rule-folder: a file a nested home owns counts for that
/// home, so reading it does not clear the rule folder around it.
#[test]
fn a_file_a_nested_home_owns_does_not_clear_the_rule_folder() {
    let config = rule_folder_project(
        "a_file_a_nested_home_owns_does_not_clear_the_rule_folder",
        "grund_config_version = 2\nproject_name = \"demo\"\n\n[schema.sources]\n\
         languages = [\"markdown\"]\n\n[schema.kinds.RULE]\nfolders = [\"rules\"]\n\
         form = \"rule\"\n\n[schema.kinds.NOTE]\nfolders = [\"rules/notes\"]\n",
        &["rules/rules.txt", "rules/notes/a.md"],
    );
    assert_eq!(
        recorded(&config),
        vec![("rules".to_string(), 8)],
        "§FS-config-v2.schema.places.ownership: `rules/notes/a.md` is NOTE's, and read"
    );
}

/// §FS-scan.checks.unread-scan-root.rule-folder: nothing in an empty folder is unread. Git
/// cannot carry an empty directory, so this control is here and not an end-to-end case.
#[test]
fn an_empty_rule_folder_is_not_recorded() {
    for (name, toml) in [("v1", RULE_FOLDER_V1), ("v2", RULE_FOLDER_V2)] {
        let config = rule_folder_project(&format!("an_empty_rule_folder_{name}"), toml, &[]);
        std::fs::create_dir_all(config.root.join("rules")).expect("make the folder");
        assert_eq!(
            recorded(&config),
            Vec::new(),
            "§FS-scan.scope.homes: {name}"
        );
    }
}

/// §FS-scan.checks.unread-scan-root.rule-folder: a hidden name is not an extension refusal.
#[test]
fn a_rule_folder_of_hidden_files_is_not_recorded() {
    for (name, toml) in [("v1", RULE_FOLDER_V1), ("v2", RULE_FOLDER_V2)] {
        let config = rule_folder_project(
            &format!("a_rule_folder_of_hidden_files_{name}"),
            toml,
            &["rules/.a.md"],
        );
        assert_eq!(
            recorded(&config),
            Vec::new(),
            "§FS-scan.scope.hidden: {name}"
        );
    }
}

/// §FS-scan.checks.unread-scan-root.rule-folder: one read file is enough, in version 1 too.
#[test]
fn a_rule_folder_with_a_read_file_is_not_recorded() {
    let config = rule_folder_project(
        "a_rule_folder_with_a_read_file_is_not_recorded",
        &RULE_FOLDER_V1.replace("[\"rs\"]", "[\"rs\", \"md\"]"),
        &["rules/a.md", "rules/b.json"],
    );
    assert_eq!(recorded(&config), Vec::new(), "the mixed folder");
}

/// §AR-scanner.1.6: the verdict comes after every root. The `include` root reads the file in
/// `drafts`, which `exclude` prunes from the folder's own walk, so a verdict taken per root
/// would call the folder unread over `b.txt`.
#[test]
fn a_rule_folder_another_root_read_a_file_of_is_not_recorded() {
    let config = rule_folder_project(
        "a_rule_folder_another_root_read_a_file_of_is_not_recorded",
        "[id]\nformat = \"{kind}-{slug}\"\n\n[[kinds]]\nkind = \"RULE\"\nfolder = \"rules\"\n\
         rules = true\n\n[scan]\ninclude = [\"rules/drafts\"]\nextensions = [\"md\"]\n\
         exclude = [\"drafts\"]\n",
        &["rules/drafts/a.md", "rules/b.txt"],
    );
    assert_eq!(
        recorded(&config),
        Vec::new(),
        "credited to the home, not to a root"
    );
}

/// §FS-config-v2.schema.places.ownership: a refused file a nested home owns is that home's,
/// so it does not make an otherwise empty rule folder around it unread.
#[test]
fn a_refused_file_a_nested_home_owns_is_not_the_rule_folders() {
    let config = rule_folder_project(
        "a_refused_file_a_nested_home_owns_is_not_the_rule_folders",
        "grund_config_version = 2\nproject_name = \"demo\"\n\n[schema.sources]\n\
         languages = [\"markdown\"]\n\n[schema.kinds.RULE]\nfolders = [\"rules\"]\n\
         form = \"rule\"\n\n[schema.kinds.NOTE]\nfolders = [\"rules/notes\"]\n",
        &["rules/notes/a.txt"],
    );
    assert_eq!(
        recorded(&config),
        Vec::new(),
        "`rules/notes/a.txt` is NOTE's"
    );
}

/// §AR-scanner.1.6: the tally is one per home, so a read file in one entry of `folders` does
/// not clear another.
#[test]
fn a_read_entry_of_folders_does_not_clear_another() {
    let config = rule_folder_project(
        "a_read_entry_of_folders_does_not_clear_another",
        "grund_config_version = 2\nproject_name = \"demo\"\n\n[schema.sources]\n\
         languages = [\"markdown\"]\n\n[schema.kinds.RULE]\nfolders = [\"alpha\", \"zeta\"]\n\
         form = \"rule\"\n",
        &["alpha/a.md", "zeta/z.txt"],
    );
    assert_eq!(
        recorded(&config),
        vec![("zeta".to_string(), 8)],
        "`alpha/a.md` is read, and it is `alpha`'s alone"
    );
}

/// §AR-lsp.5.1.1: an editor in a project that declares `[workspace]` walks only the path it
/// names. Part of a folder can refuse a file without the folder being unread, so only a home
/// some walked root is at or above gets a verdict.
#[test]
fn a_walk_of_part_of_a_rule_folder_takes_no_verdict() {
    let config = rule_folder_project(
        "a_walk_of_part_of_a_rule_folder_takes_no_verdict",
        &RULE_FOLDER_V2.replace("\"rust\"", "\"markdown\""),
        &["rules/a.md", "rules/sub/b.txt"],
    );
    let sub = config.root.join("rules/sub");
    let (findings, _) = scan_tree(&config, Some(&sub), true).expect("scan");
    assert_eq!(
        findings.unread_roots,
        Vec::new(),
        "§FS-scan.checks.unread-scan-root.rule-folder: `rules/a.md` is read, out of this walk"
    );
    let whole = rule_folder_project(
        "a_walk_of_a_whole_rule_folder_takes_its_verdict",
        RULE_FOLDER_V2,
        &["rules/a.md"],
    );
    let rules = whole.root.join("rules");
    let (findings, _) = scan_tree(&whole, Some(&rules), true).expect("scan");
    assert_eq!(
        findings.unread_roots.len(),
        1,
        "§AR-scanner.1.6: a root at the folder walks all of it"
    );
}

/// §FS-scan.checks.unread-scan-root.rule-folder: the run read no file the folder holds is what
/// earns the warning, so a file root's read counts where the folder's own walk prunes the file.
#[test]
fn a_rule_folder_a_file_root_read_a_file_of_is_not_recorded() {
    let config = rule_folder_project(
        "a_rule_folder_a_file_root_read_a_file_of_is_not_recorded",
        "[id]\nformat = \"{kind}-{slug}\"\n\n[[kinds]]\nkind = \"RULE\"\nfolder = \"rules\"\n\
         rules = true\n\n[scan]\ninclude = [\"rules/drafts/a.md\"]\nextensions = [\"md\"]\n\
         exclude = [\"drafts\"]\n",
        &["rules/drafts/a.md", "rules/b.txt"],
    );
    assert_eq!(recorded(&config), Vec::new(), "credited at the root gate");
}
