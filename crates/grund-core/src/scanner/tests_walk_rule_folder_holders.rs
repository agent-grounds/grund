//! Which rule folder homes a file counts for (§FS-scan.checks.unread-scan-root.rule-folder): each
//! that holds it with no deeper place holding it, in both versions and wherever the folder lies,
//! which is not who owns the file (§AR-scanner.1.6).

use super::tests_walk_rule_folder::{RULE_FOLDER_V2, recorded, rule_folder_project};
use crate::model::format_path;
use crate::testing::{test_root, write};

/// A version-1 project whose `[[kinds]]` are `rows`, in that order, then `scan`.
fn v1_rows(rows: &[&str], scan: &str) -> String {
    let rows: String = rows
        .iter()
        .map(|row| format!("[[kinds]]\n{row}\n\n"))
        .collect();
    format!("[id]\nformat = \"{{kind}}-{{slug}}\"\n\n{rows}[scan]\n{scan}\n")
}

/// The 1-based line of the first line of `toml` that holds `needle`.
fn line_of(toml: &str, needle: &str) -> usize {
    toml.lines()
        .position(|line| line.contains(needle))
        .expect("the key is in the config")
        + 1
}

/// `rows` in the order written and reversed, so a test holds in both.
fn both_orders(rows: [&str; 2]) -> [[&str; 2]; 2] {
    [rows, [rows[1], rows[0]]]
}

/// §FS-scan.checks.unread-scan-root.rule-folder: version 1 gives a file two nested homes hold to
/// the complement for citing, and its rule is in force, so the folder still gets its answer.
#[test]
fn a_version_1_rule_folder_nested_in_another_home_is_recorded() {
    let rule = "kind = \"RULE\"\nfolder = \"docs/rules\"\nrules = true";
    let spec = "kind = \"FS\"\nfolder = \"docs\"\nindex = false";
    for (order, rows) in both_orders([spec, rule]).iter().enumerate() {
        let toml = v1_rows(rows, "include = [\"docs\"]\nextensions = [\"rs\"]");
        let config = rule_folder_project(
            &format!("a_version_1_rule_folder_nested_{order}"),
            &toml,
            &["docs/rules/rules.md"],
        );
        assert_eq!(
            recorded(&config),
            vec![("docs/rules".to_string(), line_of(&toml, "docs/rules"))],
            "row order {order}"
        );
    }
}

/// §AR-scanner.1.6: a tie credits each row's home, so row order cannot decide whether a rule
/// folder another kind also names is ever credited. Version 2 refuses the shape at load.
#[test]
fn a_version_1_rule_folder_shared_with_another_home_is_recorded() {
    let rule = "kind = \"RULE\"\nfolder = \"docs\"\nrules = true\nindex = false";
    let spec = "kind = \"FS\"\nfolder = \"docs\"\nindex = false";
    for (order, rows) in both_orders([spec, rule]).iter().enumerate() {
        let toml = v1_rows(rows, "include = [\"src\"]\nextensions = [\"rs\"]");
        let config = rule_folder_project(
            &format!("a_version_1_rule_folder_shared_{order}"),
            &toml,
            &["docs/rules.md"],
        );
        let rule_key = toml
            .lines()
            .position(|line| line == "kind = \"RULE\"")
            .expect("the row")
            + 2;
        assert_eq!(
            recorded(&config),
            vec![("docs".to_string(), rule_key)],
            "row order {order}"
        );
    }
}

/// §FS-scan.checks.unread-scan-root.rule-folder: the control for the shared shape, in which a
/// read file in the shared folder is the rule kind's as much as the other row's.
#[test]
fn a_read_file_in_a_shared_rule_folder_keeps_it_silent() {
    let rule = "kind = \"RULE\"\nfolder = \"docs\"\nrules = true\nindex = false";
    let spec = "kind = \"FS\"\nfolder = \"docs\"\nindex = false";
    for (order, rows) in both_orders([spec, rule]).iter().enumerate() {
        let config = rule_folder_project(
            &format!("a_read_file_in_a_shared_rule_folder_{order}"),
            &v1_rows(rows, "include = [\"src\"]\nextensions = [\"rs\"]"),
            &["docs/rules.md", "docs/helper.rs"],
        );
        assert_eq!(recorded(&config), Vec::new(), "row order {order}");
    }
}

/// §AR-scanner.1.6: each row's home has its own answer, so two rule kinds naming one folder each
/// get a warning, at their own row.
#[test]
fn two_rule_kinds_sharing_a_folder_are_each_recorded() {
    let toml = v1_rows(
        &[
            "kind = \"RULE\"\nfolder = \"docs\"\nrules = true\nindex = false",
            "kind = \"POLICY\"\nfolder = \"docs\"\nrules = true\nindex = false",
        ],
        "include = [\"src\"]\nextensions = [\"rs\"]",
    );
    let config = rule_folder_project(
        "two_rule_kinds_sharing_a_folder_are_each_recorded",
        &toml,
        &["docs/rules.md"],
    );
    assert_eq!(
        recorded(&config),
        vec![("docs".to_string(), 6), ("docs".to_string(), 12)],
        "§FS-scan.checks.unread-scan-root.location: one row each"
    );
}

/// §FS-scan.checks.unread-scan-root.rule-folder: version 1's counterpart of
/// `a_file_a_nested_home_owns_does_not_clear_the_rule_folder`, where no row owns the file.
#[test]
fn a_version_1_file_a_nested_home_holds_does_not_clear_the_rule_folder() {
    let toml = v1_rows(
        &[
            "kind = \"RULE\"\nfolder = \"rules\"\nrules = true",
            "kind = \"NOTE\"\nfolder = \"rules/notes\"\nindex = false",
        ],
        "extensions = [\"md\"]",
    );
    let config = rule_folder_project(
        "a_version_1_file_a_nested_home_holds",
        &toml,
        &["rules/rules.txt", "rules/notes/a.md"],
    );
    assert_eq!(
        recorded(&config),
        vec![("rules".to_string(), line_of(&toml, "\"rules\""))],
        "`rules/notes/a.md` is read, and it is NOTE's"
    );
}

/// §FS-scan.checks.unread-scan-root.rule-folder: version 1's counterpart of
/// `a_refused_file_a_nested_home_owns_is_not_the_rule_folders`.
#[test]
fn a_version_1_refused_file_a_nested_home_holds_is_not_the_rule_folders() {
    let config = rule_folder_project(
        "a_version_1_refused_file_a_nested_home_holds",
        &v1_rows(
            &[
                "kind = \"RULE\"\nfolder = \"rules\"\nrules = true",
                "kind = \"NOTE\"\nfolder = \"rules/notes\"\nindex = false",
            ],
            "extensions = [\"md\"]",
        ),
        &["rules/notes/a.txt"],
    );
    assert_eq!(
        recorded(&config),
        Vec::new(),
        "`rules/notes/a.txt` is NOTE's"
    );
}

/// §FS-config-v2.schema.places.ownership: a `code` folder nested in a rule folder is simply the
/// deeper place, so what it holds is not the rule folder's.
#[test]
fn a_file_a_code_folder_inside_the_rule_folder_holds_is_not_the_rule_folders() {
    let config = rule_folder_project(
        "a_file_a_code_folder_inside_the_rule_folder_holds",
        "grund_config_version = 2\nproject_name = \"demo\"\n\n[schema.sources]\n\
         languages = [\"markdown\"]\n\n[schema.kinds.RULE]\nfolders = [\"rules\"]\n\
         form = \"rule\"\n\n[schema.kinds.code]\nfolders = [\"rules/vendor\"]\n",
        &["rules/vendor/a.txt"],
    );
    assert_eq!(recorded(&config), Vec::new(), "the deeper place holds it");
}

/// §FS-config-v2.schema.places: load refuses a folder one row writes twice, under any spelling,
/// and a second row for one kind, so the tally never holds two homes that are one row's folder.
#[test]
fn load_refuses_a_rule_folder_one_row_writes_twice() {
    for (name, folders) in [
        ("same", "[\"rules\", \"rules\"]"),
        ("spelled", "[\"rules\", \"./rules/\"]"),
    ] {
        let root = test_root(&format!("load_refuses_a_rule_folder_written_twice_{name}"));
        write(
            &root.join("grund.toml"),
            &RULE_FOLDER_V2.replace("[\"rules\"]", folders),
        );
        let Err(error) = crate::config::load_config_at(&root, &root) else {
            panic!("{name}: load accepted it");
        };
        assert!(
            format!("{error:#}").contains("a place belongs to one row"),
            "{name}: {error:#}"
        );
    }
    let root = test_root("load_refuses_a_second_row_for_one_rule_kind");
    let rule = "kind = \"RULE\"\nfolder = \"rules\"\nrules = true";
    write(
        &root.join("grund.toml"),
        &v1_rows(&[rule, rule], "extensions = [\"rs\"]"),
    );
    assert!(
        crate::config::load_config_at(&root, &root).is_err(),
        "version 1 too"
    );
}

/// A version-1 project in `<root>/proj` whose rule folder is the one `folder` names from there,
/// with a rule at `<root>/shared/rules.md`. `<root>` is handed to `folder` to spell it, and that
/// spelling comes back with the project, because the config root is canonical and it need not be.
fn outside_root_project(
    name: &str,
    folder: impl Fn(&std::path::Path) -> String,
    extensions: &str,
) -> (crate::config::Config, String) {
    let root = test_root(name);
    let project = root.join("proj");
    write(&root.join("shared/rules.md"), "# RULE-a: A rule.\n");
    write(&project.join("src/a.rs"), "pub fn a() {}\n");
    let written = folder(&root);
    let toml = v1_rows(
        &[
            &format!("kind = \"RULE\"\nfolder = \"{written}\"\nrules = true"),
            "kind = \"FS\"\nfolder = \"src\"\nindex = false",
        ],
        &format!("include = [\"src\"]\nextensions = {extensions}"),
    );
    write(&project.join("grund.toml"), &toml);
    let config = crate::config::load_config_at(&project, &project).expect("load the project");
    (config, written)
}

/// §FS-scan.checks.unread-scan-root.rule-folder: a version-1 folder outside the config root is
/// read when its extension is listed, so it is named when it is not, as a `file` home is.
#[test]
fn a_rule_folder_outside_the_config_root_is_recorded() {
    let (dotted, written) = outside_root_project(
        "a_rule_folder_through_dots",
        |_| "../shared".to_string(),
        "[\"rs\"]",
    );
    assert_eq!(
        recorded(&dotted),
        vec![(written, 6)],
        "§FS-scan.checks.unread-scan-root.message: as the configuration wrote it"
    );
    let (absolute, written) = outside_root_project(
        "a_rule_folder_absolute",
        |root| format_path(&root.join("shared")),
        "[\"rs\"]",
    );
    assert_eq!(recorded(&absolute), vec![(written, 6)], "an absolute path");
}

/// §FS-scan.checks.unread-scan-root.rule-folder: the controls, where the folder's rule is read.
#[test]
fn a_read_rule_folder_outside_the_config_root_is_not_recorded() {
    let (dotted, _) = outside_root_project(
        "a_read_rule_folder_through_dots",
        |_| "../shared".to_string(),
        "[\"rs\", \"md\"]",
    );
    assert_eq!(recorded(&dotted), Vec::new(), "through `..`");
    let (absolute, _) = outside_root_project(
        "a_read_rule_folder_absolute",
        |root| format_path(&root.join("shared")),
        "[\"rs\", \"md\"]",
    );
    assert_eq!(recorded(&absolute), Vec::new(), "an absolute path");
}

/// §AR-scanner.1.6: one key for every place, so a home written as an absolute path inside the
/// config root is credited too, which a config-root-relative test could not do.
#[test]
fn an_absolute_rule_folder_inside_the_config_root_is_recorded() {
    let root = test_root("an_absolute_rule_folder_inside_the_config_root");
    write(&root.join("rules/a.md"), "# RULE-a: A rule.\n");
    let folder = format_path(&root.join("rules"));
    let toml = v1_rows(
        &[&format!(
            "kind = \"RULE\"\nfolder = \"{folder}\"\nrules = true"
        )],
        "extensions = [\"rs\"]",
    );
    write(&root.join("grund.toml"), &toml);
    let config = crate::config::load_config_at(&root, &root).expect("load the project");
    assert_eq!(recorded(&config), vec![(folder, 6)]);
}

/// §AR-scanner.1.6: the deeper-place test holds outside the config root too.
#[test]
fn a_file_a_home_nested_outside_the_config_root_holds_is_not_the_rule_folders() {
    let root = test_root("a_file_a_home_nested_outside_the_config_root");
    let project = root.join("proj");
    write(&root.join("shared/notes/a.txt"), "A note.\n");
    write(&project.join("src/a.rs"), "pub fn a() {}\n");
    write(
        &project.join("grund.toml"),
        &v1_rows(
            &[
                "kind = \"RULE\"\nfolder = \"../shared\"\nrules = true",
                "kind = \"NOTE\"\nfolder = \"../shared/notes\"\nindex = false",
            ],
            "include = [\"src\"]\nextensions = [\"rs\"]",
        ),
    );
    let config = crate::config::load_config_at(&project, &project).expect("load the project");
    assert_eq!(recorded(&config), Vec::new(), "`a.txt` is NOTE's");
}

/// §AR-scanner.1.13: a read and a refused extension are what the tally counts. The predicate's
/// other answers say the walk would not have yielded the file, so they count as neither.
#[test]
fn only_a_refused_extension_counts_against_a_rule_folder() {
    use super::read_predicate::Unread;
    use super::rule_folders::RuleFolderTally;
    let config = rule_folder_project(
        "only_a_refused_extension_counts_against_a_rule_folder",
        RULE_FOLDER_V2,
        &[],
    );
    let rules = config.root.join("rules");
    let file = rules.join("new.md");
    for answer in [
        Unread::HiddenName,
        Unread::ExcludedDirectory,
        Unread::ExcludedPath,
        Unread::IgnoreFile,
        Unread::OutsideRoots,
        Unread::UnwalkedHome,
        Unread::AnotherProject,
        Unread::E2eCase,
        Unread::LoopingLink,
        Unread::Unobserved,
    ] {
        let mut tally = RuleFolderTally::of(config.schema(), config.frame()).expect("a home");
        tally.walking(&rules);
        tally.count(&file, Some(answer));
        assert_eq!(tally.unread().count(), 0, "{answer:?}");
    }
    let mut tally = RuleFolderTally::of(config.schema(), config.frame()).expect("a home");
    tally.walking(&rules);
    tally.count(&file, Some(Unread::Extension));
    assert_eq!(tally.unread().count(), 1, "the control");
}
