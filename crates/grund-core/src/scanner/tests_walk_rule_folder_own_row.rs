//! A read under a rule row's own deeper place clears that row's folders around it, and only a read
//! does (§FS-scan.checks.unread-scan-root.rule-folder, §AR-scanner.1.6). The line this draws is
//! whether a rule of the warned row is read: another row's home nested in the folder stays that
//! row's, which is the shape §DF-unread-rule-folder.2.5 accepts.

use super::tests_walk_rule_folder::{recorded, rule_folder_project};

/// A version-2 project reading Markdown alone, with the rule row's `places` and then `rest`. Its
/// rule row's first place key is on line 8.
fn project(places: &str, rest: &str) -> String {
    format!(
        "grund_config_version = 2\nproject_name = \"demo\"\n\n[schema.sources]\n\
         languages = [\"markdown\"]\n\n[schema.kinds.RULE]\n{places}\nform = \"rule\"\n\
         index = false\n{rest}"
    )
}

/// §FS-scan.checks.unread-scan-root.rule-folder: the row's `files` entry is read and its rule is
/// in force, so its refused neighbour does not make the folder around it unread.
#[test]
fn a_read_files_entry_clears_the_folder_of_its_own_row() {
    let config = rule_folder_project(
        "a_read_files_entry_clears_the_folder_of_its_own_row",
        &project("files = [\"rules/one.md\"]\nfolders = [\"rules\"]", ""),
        &["rules/one.md", "rules/notes.txt"],
    );
    assert_eq!(recorded(&config), Vec::new());
}

/// §FS-scan.checks.unread-scan-root.rule-folder: the same for a nested entry of `folders`.
#[test]
fn a_read_nested_entry_clears_the_folder_of_its_own_row() {
    let config = rule_folder_project(
        "a_read_nested_entry_clears_the_folder_of_its_own_row",
        &project("folders = [\"rules\", \"rules/extra\"]", ""),
        &["rules/extra/one.md", "rules/notes.txt"],
    );
    assert_eq!(recorded(&config), Vec::new());
}

/// §AR-scanner.1.6: the widening asks the row of the deepest holder, so a read under the row's own
/// entry reaches the folder through another row's home between them.
#[test]
fn a_read_entry_clears_its_own_row_past_another_rows_home_between_them() {
    let config = rule_folder_project(
        "a_read_entry_clears_its_own_row_past_another_rows_home",
        &project(
            "folders = [\"rules\", \"rules/notes/deep\"]",
            "\n[schema.kinds.NOTE]\nfolders = [\"rules/notes\"]\n",
        ),
        &["rules/notes/deep/one.md", "rules/two.txt"],
    );
    assert_eq!(recorded(&config), Vec::new());
}

/// §FS-scan.checks.unread-scan-root.rule-folder: a read widens outward to folders that hold the
/// file. `rules/one.md` is not in `rules/extra`, so that entry keeps its own answer.
#[test]
fn a_read_in_the_outer_entry_does_not_clear_a_nested_one() {
    let config = rule_folder_project(
        "a_read_in_the_outer_entry_does_not_clear_a_nested_one",
        &project("folders = [\"rules\", \"rules/extra\"]", ""),
        &["rules/one.md", "rules/extra/notes.txt"],
    );
    assert_eq!(recorded(&config), vec![("rules/extra".to_string(), 8)]);
}

/// §FS-scan.checks.unread-scan-root.rule-folder: a refusal stays with the deepest holder, so one
/// file refused in the nested entry earns one warning, and not one for the folder around it too.
#[test]
fn a_refusal_stays_with_the_deepest_entry_of_its_own_row() {
    let config = rule_folder_project(
        "a_refusal_stays_with_the_deepest_entry_of_its_own_row",
        &project("folders = [\"rules\", \"rules/extra\"]", ""),
        &["rules/extra/notes.txt"],
    );
    assert_eq!(recorded(&config), vec![("rules/extra".to_string(), 8)]);
}

/// §DF-unread-rule-folder.2.5: another rule kind's `files` entry inside the folder is that row's,
/// so reading it says nothing of the warned row's rules.
#[test]
fn another_rows_files_entry_does_not_clear_the_rule_folder() {
    let config = rule_folder_project(
        "another_rows_files_entry_does_not_clear_the_rule_folder",
        &project(
            "folders = [\"rules\"]",
            "\n[schema.kinds.POL]\nfiles = [\"rules/one.md\"]\nform = \"rule\"\n",
        ),
        &["rules/one.md", "rules/notes.txt"],
    );
    assert_eq!(recorded(&config), vec![("rules".to_string(), 8)]);
}
