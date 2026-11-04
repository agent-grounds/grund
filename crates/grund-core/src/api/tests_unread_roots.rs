//! The message bytes of the unread-scan-root warning for a root a hidden name keeps
//! out, alone or beside the extension list: one case per row of the hidden reason and
//! fix tables of §FS-scan.checks.unread-scan-root.message, and the `citable = false`
//! ending (§DF-unread-hidden-root.2.1).

use std::path::PathBuf;

use super::report::public_lsp_report;
use super::unread_roots::{unread_root_message, unread_root_warnings};
use crate::config::{Invocation, Selection, config_at, load_config_at};
use crate::model::{CheckReport, Code, Diagnostic, Refused, RootOrigin, RootRow};
use crate::testing::{scan_tree, test_root, write};

/// A row the root gate recorded for `written`, refused by the hidden rule and, where
/// `extension` says so, by the extension list too.
fn hidden(origin: RootOrigin, written: &str, extension: bool) -> RootRow {
    RootRow {
        origin,
        written: written.to_string(),
        config: PathBuf::from("grund.toml"),
        line: 10,
        refused: Refused {
            hidden: true,
            extension,
        },
    }
}

fn home(citable: bool) -> RootOrigin {
    RootOrigin::Home {
        kind: "NOTE".into(),
        citable,
    }
}

#[test]
fn a_hidden_home_with_a_listed_extension_is_told_to_rename() {
    assert_eq!(
        unread_root_message(&hidden(home(true), ".notes.md", false), false),
        "[[kinds]] NOTE home `.notes.md` is not read — its name begins with `.`, and grund \
         reads no hidden file, so nothing in it is checked. Rename the file to a name that \
         is not hidden.",
        "§FS-scan.checks.unread-scan-root.message: only the name refuses it"
    );
}

#[test]
fn a_hidden_include_entry_is_told_to_rename_or_drop() {
    assert_eq!(
        unread_root_message(&hidden(RootOrigin::Include, ".extra.md", false), false),
        "[scan] include entry `.extra.md` is not read — its name begins with `.`, and grund \
         reads no hidden file, so nothing in it is checked. Rename the file to a name that \
         is not hidden, or drop the entry.",
        "§FS-scan.checks.unread-scan-root.message: an include entry may be dropped"
    );
}

#[test]
fn a_hidden_and_unlisted_root_names_both_rules_in_version_1() {
    assert_eq!(
        unread_root_message(&hidden(home(true), ".notes.txt", true), false),
        "[[kinds]] NOTE home `.notes.txt` is not read — its name begins with `.`, and `txt` \
         is not in [scan] extensions, so nothing in it is checked. Rename the file to a name \
         that is not hidden and an extension the list holds.",
        "§DF-unread-hidden-root.2.1: one warning names both rules"
    );
    assert_eq!(
        unread_root_message(&hidden(RootOrigin::Include, ".extra.txt", true), false),
        "[scan] include entry `.extra.txt` is not read — its name begins with `.`, and `txt` \
         is not in [scan] extensions, so nothing in it is checked. Rename the file to a name \
         that is not hidden and an extension the list holds, or drop the entry.",
        "§FS-scan.checks.unread-scan-root.message: the include rename names both rules"
    );
}

#[test]
fn a_hidden_and_unlisted_root_names_both_rules_in_version_2() {
    assert_eq!(
        unread_root_message(&hidden(home(true), ".notes.txt", true), true),
        "[schema.kinds.NOTE] file `.notes.txt` is not read — its name begins with `.`, and \
         no language in [schema.sources] languages reads `.txt`, so nothing in it is checked. \
         Rename the file to a name that is not hidden and an extension a listed language reads.",
        "§FS-scan.checks.unread-scan-root.message: the version-2 hidden rows"
    );
    assert_eq!(
        unread_root_message(&hidden(home(true), ".notes.md", false), true),
        "[schema.kinds.NOTE] file `.notes.md` is not read — its name begins with `.`, and \
         grund reads no hidden file, so nothing in it is checked. Rename the file to a name \
         that is not hidden.",
        "§FS-scan.checks.unread-scan-root.message: only the name refuses it"
    );
}

#[test]
fn a_hidden_root_with_no_extension_says_it_has_none() {
    assert_eq!(
        unread_root_message(&hidden(home(true), ".notes", true), false),
        "[[kinds]] NOTE home `.notes` is not read — its name begins with `.`, and it has no \
         extension, so nothing in it is checked. Rename the file to a name that is not hidden \
         and an extension the list holds.",
        "§FS-scan.checks.unread-scan-root.message: a hidden file with no extension"
    );
}

#[test]
fn a_hidden_home_on_a_non_citable_row_is_offered_scan_false() {
    assert_eq!(
        unread_root_message(&hidden(home(false), ".notes.md", false), false),
        "[[kinds]] NOTE home `.notes.md` is not read — its name begins with `.`, and grund \
         reads no hidden file, so nothing in it is checked. Rename the file to a name that \
         is not hidden, or set `scan = false` on the row if it only lists a place.",
        "§FS-config.3.4.7: a place-only row may say so"
    );
    assert_eq!(
        unread_root_message(&hidden(home(false), ".notes.yaml", true), true),
        "[schema.kinds.NOTE] file `.notes.yaml` is not read — its name begins with `.`, and \
         no language in [schema.sources] languages reads `.yaml`, so nothing in it is checked. \
         Rename the file to a name that is not hidden and an extension a listed language \
         reads, or set `scan = false` on the row if it only lists a place.",
        "§FS-config.3.4.7: the version-2 rename gains the same ending"
    );
}

/// §FS-lsp.4.1: the editor marks the warning at the `grund.toml` that holds the row,
/// whatever directory its process runs in, as `grund check` names it from a
/// subdirectory: the row's location is spelled from the root (§FS-errors.4).
#[test]
fn an_editor_marks_the_warning_at_the_config_not_under_its_working_directory() {
    let root = test_root("unread_root_editor_config_path");
    write(
        &root.join("grund.toml"),
        "[scan]\ninclude = [\"docs/.notes.md\"]\n",
    );
    let config = load_config_at(&root, &root).expect("load the project");
    let warning = Diagnostic {
        code: Code::UNREAD_SCAN_ROOT,
        path: config.config_file.clone(),
        line: Some(2),
        column: None,
        message: String::new(),
        sites: Vec::new(),
        authority: Vec::new(),
    };
    let report = CheckReport {
        warnings: vec![warning],
        ..CheckReport::default()
    };
    let published = public_lsp_report(&config, report);
    assert_eq!(
        PathBuf::from(published.warnings[0].path.as_deref().unwrap()),
        root.join("grund.toml").canonicalize().unwrap()
    );
}

/// A member's config is spelled from the base its workspace loads it under, so
/// the editor joins its row's location back at that base, not at the member's
/// own root (§FS-errors.4).
#[test]
fn an_editor_marks_a_members_warning_at_the_members_config() {
    // Canonical, as a workspace load spells its members' roots: on Windows a plain
    // root shares no prefix with a member's verbatim config path.
    let root = test_root("unread_root_editor_member_config_path")
        .canonicalize()
        .unwrap();
    let member = root.join("member");
    write(
        &member.join("grund.toml"),
        "[scan]\ninclude = [\"docs/.notes.md\"]\n",
    );
    let invocation = Invocation::new(&member, false, Selection::Project);
    let config = config_at(&member, &root, Some(&root), &invocation).expect("load the member");
    assert_eq!(config.config_file, Some(PathBuf::from("member/grund.toml")));
    let warning = Diagnostic {
        code: Code::UNREAD_SCAN_ROOT,
        path: config.config_file.clone(),
        line: Some(2),
        column: None,
        message: String::new(),
        sites: Vec::new(),
        authority: Vec::new(),
    };
    let report = CheckReport {
        warnings: vec![warning],
        ..CheckReport::default()
    };
    let published = public_lsp_report(&config, report);
    assert_eq!(
        PathBuf::from(published.warnings[0].path.as_deref().unwrap()),
        member.join("grund.toml").canonicalize().unwrap()
    );
}

/// A version-1 project with one rule kind, its `folder` key on line 6.
const RULE_FOLDER_V1: &str = "[id]\nformat = \"{kind}-{slug}\"\n\n[[kinds]]\nkind = \"RULE\"\n\
                              folder = \"rules\"\nrules = true\n\n[scan]\nextensions = [\"rs\"]\n";
/// A version-2 project with one rule kind, its `folders` key on line 8.
const RULE_FOLDER_V2: &str = "grund_config_version = 2\nproject_name = \"demo\"\n\n\
                              [schema.sources]\nlanguages = [\"rust\"]\n\n\
                              [schema.kinds.RULE]\nfolders = [\"rules\"]\nform = \"rule\"\n";

/// The located warnings a run of `toml` over `files` earns, through the walk, the record and
/// the message, so these pin the bytes of §FS-scan.checks.unread-scan-root.message whatever
/// shape the record takes.
fn rule_folder_warnings(name: &str, toml: &str, files: &[&str]) -> Vec<(usize, String)> {
    let root = test_root(name);
    write(&root.join("grund.toml"), toml);
    for file in files {
        write(
            &root.join(file),
            "# RULE-a: Each FS must cite at least one RULE.\n",
        );
    }
    let config = load_config_at(&root, &root).expect("load the project");
    let (findings, _) = scan_tree(&config, None, false).expect("scan");
    unread_root_warnings(&config, &findings)
        .into_iter()
        .map(|warning| (warning.line.expect("located"), warning.message))
        .collect()
}

#[test]
fn a_rule_folder_refusing_one_extension_is_named_in_version_1() {
    assert_eq!(
        rule_folder_warnings(
            "a_rule_folder_refusing_one_extension_is_named_in_version_1",
            RULE_FOLDER_V1,
            &["rules/a.md", "rules/b.md"],
        ),
        vec![(
            6,
            "[[kinds]] RULE home `rules` is not read — `md` is not in [scan] extensions, so \
             nothing in it is checked. Add \"md\" to [scan] extensions, or give its files an \
             extension the list holds."
                .to_string()
        )],
        "§FS-scan.checks.unread-scan-root.message: one refused extension, once however many files"
    );
}

#[test]
fn a_rule_folder_refusing_one_extension_is_named_in_version_2() {
    assert_eq!(
        rule_folder_warnings(
            "a_rule_folder_refusing_one_extension_is_named_in_version_2",
            RULE_FOLDER_V2,
            &["rules/a.md", "rules/b.md"],
        ),
        vec![(
            8,
            "[schema.kinds.RULE] folder `rules` is not read — no language in [schema.sources] \
             languages reads `.md`, so nothing in it is checked. Give its files an extension a \
             listed language reads."
                .to_string()
        )],
        "§FS-scan.checks.unread-scan-root.message: one refused extension, version 2"
    );
}

#[test]
fn a_rule_folder_refusing_several_extensions_names_them_in_order_in_version_1() {
    assert_eq!(
        rule_folder_warnings(
            "a_rule_folder_refusing_two_extensions_in_version_1",
            RULE_FOLDER_V1,
            &["rules/a.md", "rules/b.json"],
        ),
        vec![(
            6,
            "[[kinds]] RULE home `rules` is not read — `json` and `md` are not in [scan] \
             extensions, so nothing in it is checked. Add \"json\" or \"md\" to [scan] extensions, \
             or give its files an extension the list holds."
                .to_string()
        )],
        "§FS-scan.checks.unread-scan-root.message: two extensions, in byte order"
    );
    assert_eq!(
        rule_folder_warnings(
            "a_rule_folder_refusing_three_extensions_in_version_1",
            RULE_FOLDER_V1,
            &["rules/a.txt", "rules/b.md", "rules/c.json"],
        ),
        vec![(
            6,
            "[[kinds]] RULE home `rules` is not read — `json`, `md`, and `txt` are not in [scan] \
             extensions, so nothing in it is checked. Add \"json\", \"md\", or \"txt\" to [scan] \
             extensions, or give its files an extension the list holds."
                .to_string()
        )],
        "§FS-scan.checks.unread-scan-root.message: three or more, the last alone takes the `and`"
    );
}

#[test]
fn a_rule_folder_refusing_several_extensions_names_them_in_order_in_version_2() {
    assert_eq!(
        rule_folder_warnings(
            "a_rule_folder_refusing_two_extensions_in_version_2",
            RULE_FOLDER_V2,
            &["rules/a.md", "rules/b.json"],
        ),
        vec![(
            8,
            "[schema.kinds.RULE] folder `rules` is not read — no language in [schema.sources] \
             languages reads `.json` or `.md`, so nothing in it is checked. Give its files an \
             extension a listed language reads."
                .to_string()
        )],
        "§FS-scan.checks.unread-scan-root.message: two extensions, in byte order"
    );
    assert_eq!(
        rule_folder_warnings(
            "a_rule_folder_refusing_three_extensions_in_version_2",
            RULE_FOLDER_V2,
            &["rules/a.txt", "rules/b.md", "rules/c.json"],
        ),
        vec![(
            8,
            "[schema.kinds.RULE] folder `rules` is not read — no language in [schema.sources] \
             languages reads `.json`, `.md`, or `.txt`, so nothing in it is checked. Give its \
             files an extension a listed language reads."
                .to_string()
        )],
        "§FS-scan.checks.unread-scan-root.message: three or more, the last alone takes the `or`"
    );
}

#[test]
fn a_rule_folder_whose_refused_files_have_no_extension_says_so_in_version_1() {
    assert_eq!(
        rule_folder_warnings(
            "a_rule_folder_whose_refused_files_have_no_extension_says_so_in_version_1",
            RULE_FOLDER_V1,
            &["rules/NOTES"],
        ),
        vec![(
            6,
            "[[kinds]] RULE home `rules` is not read — its files have no extension, and [scan] \
             extensions reads a file only by its extension, so nothing in it is checked. Rename \
             its files to an extension the list holds."
                .to_string()
        )],
        "§FS-scan.checks.unread-scan-root.message: a file with none is never told to add it"
    );
}

#[test]
fn a_rule_folder_whose_refused_files_have_no_extension_says_so_in_version_2() {
    assert_eq!(
        rule_folder_warnings(
            "a_rule_folder_whose_refused_files_have_no_extension_says_so_in_version_2",
            RULE_FOLDER_V2,
            &["rules/NOTES"],
        ),
        vec![(
            8,
            "[schema.kinds.RULE] folder `rules` is not read — its files have no extension, and \
             [schema.sources] languages reads a file only by its extension, so nothing in it is \
             checked. Give its files an extension a listed language reads."
                .to_string()
        )],
        "§FS-scan.checks.unread-scan-root.message: no extension, version 2"
    );
}

#[test]
fn a_rule_folder_with_and_without_extensions_names_only_the_extensions() {
    assert_eq!(
        rule_folder_warnings(
            "a_rule_folder_with_and_without_extensions_names_only_the_extensions",
            RULE_FOLDER_V1,
            &["rules/NOTES", "rules/a.md"],
        ),
        vec![(
            6,
            "[[kinds]] RULE home `rules` is not read — `md` is not in [scan] extensions, so \
             nothing in it is checked. Add \"md\" to [scan] extensions, or give its files an \
             extension the list holds."
                .to_string()
        )],
        "§FS-scan.checks.unread-scan-root.message: reading one file clears the warning"
    );
}

/// §FS-scan.checks.unread-scan-root.message: a folder's reason is what the walk refused in it,
/// so the folder's own name, which reads like a file's, reports no extension.
#[test]
fn a_rule_folder_with_a_dot_in_its_name_does_not_report_its_own_extension() {
    let toml = RULE_FOLDER_V1.replace("\"rules\"", "\"rules.d\"");
    assert_eq!(
        rule_folder_warnings(
            "a_rule_folder_with_a_dot_in_its_name_no_extension",
            &toml,
            &["rules.d/NOTES"],
        ),
        vec![(
            6,
            "[[kinds]] RULE home `rules.d` is not read — its files have no extension, and [scan] \
             extensions reads a file only by its extension, so nothing in it is checked. Rename \
             its files to an extension the list holds."
                .to_string()
        )],
        "§FS-scan.checks.unread-scan-root.message: `d` is no refused extension"
    );
    assert_eq!(
        rule_folder_warnings(
            "a_rule_folder_with_a_dot_in_its_name_one_extension",
            &toml,
            &["rules.d/a.md"],
        ),
        vec![(
            6,
            "[[kinds]] RULE home `rules.d` is not read — `md` is not in [scan] extensions, so \
             nothing in it is checked. Add \"md\" to [scan] extensions, or give its files an \
             extension the list holds."
                .to_string()
        )],
        "§FS-scan.checks.unread-scan-root.message: only what the walk refused"
    );
}
