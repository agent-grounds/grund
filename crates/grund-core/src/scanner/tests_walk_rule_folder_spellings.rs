//! The early return in front of a rule folder's lookup is only ever a necessary condition
//! (§AR-scanner.1.6): every way a home can be written still gets the answer the one test gives,
//! for and against (§FS-scan.checks.unread-scan-root.rule-folder).

use super::tests_walk_rule_folder::{RULE_FOLDER_V2, recorded, rule_folder_project};
use crate::config::Config;
use crate::model::format_path;
use crate::testing::{test_root, write};

/// A version-1 project in `<root>/proj`, over `files` relative to `<root>`, whose rule folder is
/// the one `folder` names from there. The project's own `src` is the one `include` and no home.
fn project(
    name: &str,
    folder: impl Fn(&std::path::Path) -> String,
    extensions: &str,
    files: &[&str],
) -> (Config, String) {
    let root = test_root(name);
    let project = root.join("proj");
    std::fs::create_dir_all(&project).expect("make the project");
    for file in files {
        write(&root.join(file), "# RULE-a: A rule.\n");
    }
    let written = folder(&root);
    let toml = format!(
        "[id]\nformat = \"{{kind}}-{{slug}}\"\n\n[[kinds]]\nkind = \"RULE\"\nfolder = \"{written}\"\n\
         rules = true\nindex = false\n\n[scan]\ninclude = [\"src\"]\nextensions = {extensions}\n"
    );
    write(&project.join("grund.toml"), &toml);
    let config = crate::config::load_config_at(&project, &project).expect("load the project");
    (config, written)
}

/// §AR-scanner.1.6: the early return compares by path component, so a file in `rules2` is not in
/// `rules`. The string prefix would pass it, and only the one test behind it would say no.
#[test]
fn a_read_file_in_a_sibling_sharing_the_prefix_does_not_clear_the_folder() {
    let config = rule_folder_project(
        "a_read_file_in_a_sibling_sharing_the_prefix",
        &RULE_FOLDER_V2.replace("\"rust\"", "\"markdown\""),
        &["rules2/a.md", "rules/notes.txt"],
    );
    assert_eq!(recorded(&config), vec![("rules".to_string(), 8)]);
}

/// §AR-scanner.1.6: the early return asks the normalized key and not the spelling.
#[test]
fn a_folder_spelled_with_dots_and_a_slash_gets_its_answer() {
    let toml = RULE_FOLDER_V2
        .replace("\"rust\"", "\"markdown\"")
        .replace("[\"rules\"]", "[\"./rules/\"]");
    let unread = rule_folder_project(
        "a_folder_spelled_with_dots_unread",
        &toml,
        &["rules/notes.txt"],
    );
    assert_eq!(recorded(&unread), vec![("rules".to_string(), 8)]);
    let read = rule_folder_project(
        "a_folder_spelled_with_dots_read",
        &toml,
        &["rules/a.md", "rules/notes.txt"],
    );
    assert_eq!(recorded(&read), Vec::new(), "the control");
}

/// §AR-scanner.1.6: a home written through `..` that comes back inside the config root.
#[test]
fn a_folder_spelled_through_the_config_roots_own_name_gets_its_answer() {
    let written = |_: &std::path::Path| "../proj/rules".to_string();
    let (unread, name) = project(
        "a_folder_through_its_own_root_unread",
        written,
        "[\"rs\"]",
        &["proj/rules/a.md"],
    );
    assert_eq!(recorded(&unread), vec![(name, 6)]);
    let (read, _) = project(
        "a_folder_through_its_own_root_read",
        written,
        "[\"rs\", \"md\"]",
        &["proj/rules/a.md", "proj/rules/notes.txt"],
    );
    assert_eq!(recorded(&read), Vec::new(), "the control");
}

/// §AR-scanner.1.6: an absolute home inside the config root, credited as well as refused.
#[test]
fn an_absolute_folder_inside_the_config_root_is_read() {
    let (read, _) = project(
        "an_absolute_folder_inside_the_config_root_read",
        |root| format_path(&root.join("proj/rules")),
        "[\"rs\", \"md\"]",
        &["proj/rules/a.md", "proj/rules/notes.txt"],
    );
    assert_eq!(recorded(&read), Vec::new());
}

/// §AR-scanner.1.6: `.` holds everything inside the config root, read or refused.
#[test]
fn a_folder_at_the_config_root_gets_its_answer() {
    let (unread, name) = project(
        "a_folder_at_the_config_root_unread",
        |_| ".".to_string(),
        "[\"rs\"]",
        &["proj/sub/a.md"],
    );
    assert_eq!(
        recorded(&unread),
        vec![(name, 6)],
        "`md` and `toml` are refused"
    );
    let (read, _) = project(
        "a_folder_at_the_config_root_read",
        |_| ".".to_string(),
        "[\"rs\", \"md\"]",
        &["proj/sub/a.md", "proj/sub/notes.txt"],
    );
    assert_eq!(recorded(&read), Vec::new(), "the control");
}

/// §AR-scanner.1.6: a home above the config root lies outside it and still holds every file inside
/// it, so a file under a plain relative key cannot be returned early from it.
#[test]
fn a_folder_above_the_config_root_is_read_from_inside_it() {
    for (name, written) in [("dots", ".."), ("absolute", "")] {
        let folder = |root: &std::path::Path| match written {
            "" => format_path(root),
            dots => dots.to_string(),
        };
        let (read, _) = project(
            &format!("a_folder_above_the_config_root_read_{name}"),
            folder,
            "[\"rs\", \"md\"]",
            &["proj/src/a.md", "other/notes.txt"],
        );
        assert_eq!(recorded(&read), Vec::new(), "{name}");
        let (unread, spelled) = project(
            &format!("a_folder_above_the_config_root_unread_{name}"),
            folder,
            "[\"rs\"]",
            &["proj/src/a.md"],
        );
        assert_eq!(recorded(&unread), vec![(spelled, 6)], "{name}: the control");
    }
}

/// §AR-scanner.1.6: `rules` and `../proj/rules` are one folder under one key, so both rows tie
/// whichever spelling reached the file, and each answers at its own row and spelling.
#[test]
fn two_spellings_of_one_folder_tie() {
    let toml = |extensions: &str| {
        format!(
            "[id]\nformat = \"{{kind}}-{{slug}}\"\n\n[[kinds]]\nkind = \"RULE\"\nfolder = \"rules\"\n\
             rules = true\nindex = false\n\n[[kinds]]\nkind = \"POL\"\nfolder = \"../proj/rules\"\n\
             rules = true\nindex = false\n\n[scan]\ninclude = [\"src\"]\nextensions = {extensions}\n"
        )
    };
    let in_proj = |name: &str, toml: &str, files: &[&str]| {
        let project = test_root(name).join("proj");
        write(&project.join("grund.toml"), toml);
        for file in files {
            write(&project.join(file), "# RULE-a: A rule.\n");
        }
        crate::config::load_config_at(&project, &project).expect("load the project")
    };
    let unread = in_proj(
        "two_spellings_of_one_folder_tie_unread",
        &toml("[\"rs\"]"),
        &["rules/a.md"],
    );
    assert_eq!(
        recorded(&unread),
        vec![("rules".to_string(), 6), ("../proj/rules".to_string(), 12)]
    );
    let read = in_proj(
        "two_spellings_of_one_folder_tie_read",
        &toml("[\"rs\", \"md\"]"),
        &["rules/a.md", "rules/notes.txt"],
    );
    assert_eq!(recorded(&read), Vec::new(), "the control");
}
