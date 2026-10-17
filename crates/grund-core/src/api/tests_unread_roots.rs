//! The message bytes of the unread-scan-root warning for a root a hidden name keeps
//! out, alone or beside the extension list: one case per row of the hidden reason and
//! fix tables of §FS-scan.checks.unread-scan-root.message, and the `citable = false`
//! ending (§DF-unread-hidden-root.2.1).

use std::path::PathBuf;

use super::report::public_lsp_report;
use super::unread_roots::unread_root_message;
use crate::config::{Invocation, Selection, config_at, load_config_at};
use crate::model::{CheckReport, Diagnostic, Refused, RootOrigin, RootRow};
use crate::testing::{test_root, write};

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
        code: "unread-scan-root",
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
    let root = test_root("unread_root_editor_member_config_path");
    let member = root.join("member");
    write(
        &member.join("grund.toml"),
        "[scan]\ninclude = [\"docs/.notes.md\"]\n",
    );
    let invocation = Invocation::new(&member, false, Selection::Project);
    let config = config_at(&member, &root, Some(&root), &invocation).expect("load the member");
    assert_eq!(config.config_file, Some(PathBuf::from("member/grund.toml")));
    let warning = Diagnostic {
        code: "unread-scan-root",
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
