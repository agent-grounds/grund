//! What no committed fixture can hold about an empty string in a version 1 `[scan] extensions`
//! (§FS-scan.scope.extensions.empty): a name ending in a dot, which Git for Windows refuses to check
//! out, and a name whose extension is not UTF-8, which some filesystems refuse to hold. Both are
//! made at run time. Under `["md", ""]` each is read, and under `["md"]` neither is
//! (§FS-scan.scope.extensions.root), and these tests keep what version 1 reads in place.
#![cfg(unix)]

#[path = "support/external_facts.rs"]
mod support;

use std::ffi::OsStr;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use support::*;

const WITH_ENTRY: &str = r#"["md", ""]"#;
const WITHOUT_ENTRY: &str = r#"["md"]"#;
const DECLARATION: &str = "# FS-001-a: Alpha\n#\n# Local body.\n";
const CAUTION: &str =
    "warning: grund.toml is version 1; migrate it with grund config migrate --to 2\n";
/// `FACTS.<0xff>`: what follows the last dot is one byte that is not UTF-8.
const NOT_UTF8: &[u8] = b"FACTS.\xff";

fn project(test: &str, extensions: &str, home: &str) -> PathBuf {
    let project = root(&format!("empty-extension-{test}"));
    let config = format!(
        "project_name = \"r\"\n\n[scan]\nextensions = {extensions}\n\n[[kinds]]\nkind = \"FS\"\nfile = \"{home}\"\n"
    );
    write(&project, "grund.toml", &config);
    project
}

/// A home `docs/fs.md` and, walked beside it, a file named [`NOT_UTF8`] declaring FS-001-a. `None`
/// where the filesystem will not hold that name.
fn beside_a_name_not_utf8(test: &str, extensions: &str) -> Option<PathBuf> {
    let project = project(test, extensions, "docs/fs.md");
    write(&project, "docs/fs.md", "# FS-002-b: Beta\n\nLocal body.\n");
    holds_a_name_not_utf8(&project.join("docs")).then_some(project)
}

/// Whether `dir` now holds a file named [`NOT_UTF8`], byte for byte: a filesystem may refuse the
/// name, as macOS does, or keep it under another spelling.
fn holds_a_name_not_utf8(dir: &Path) -> bool {
    if let Err(error) = fs::write(dir.join(OsStr::from_bytes(NOT_UTF8)), DECLARATION) {
        eprintln!("skipped: this filesystem refuses a name that is not UTF-8: {error}");
        return false;
    }
    fs::read_dir(dir)
        .expect("read the fixture directory")
        .flatten()
        .any(|entry| entry.file_name().as_bytes() == NOT_UTF8)
}

#[test]
fn the_entry_reads_a_home_whose_name_ends_in_a_dot() {
    let project = project("trailing-dot-with", WITH_ENTRY, "FACTS.");
    write(&project, "FACTS.", DECLARATION);

    let list = run(&project, &["list"]);
    assert_code(&list, 0, "list");
    assert_eq!(stdout(&list), "FS-001-a  FACTS.:1  Alpha\n");

    let check = run(&project, &["check"]);
    assert_code(&check, 0, "check");
    assert_eq!(
        stdout(&check),
        "FACTS.:1: warning: declared but never cited: FS-001-a; cite it\n"
    );
    assert_eq!(stderr(&check), CAUTION);
}

#[test]
fn without_the_entry_a_home_whose_name_ends_in_a_dot_is_named_not_read() {
    let project = project("trailing-dot-without", WITHOUT_ENTRY, "FACTS.");
    write(&project, "FACTS.", DECLARATION);

    let list = run(&project, &["list"]);
    assert_code(&list, 0, "list");
    assert_eq!(stdout(&list), "");

    let check = run(&project, &["check"]);
    assert_code(&check, 0, "check");
    assert_eq!(
        stdout(&check),
        "grund.toml:8: warning: [[kinds]] FS home `FACTS.` is not read — it has no extension, and \
         [scan] extensions reads a file only by its extension, so nothing in it is checked. Rename \
         the file to an extension the list holds.\n"
    );
    assert_eq!(stderr(&check), CAUTION);
}

#[test]
fn the_entry_reads_a_walked_file_whose_extension_is_not_utf8() {
    let Some(project) = beside_a_name_not_utf8("not-utf8-with", WITH_ENTRY) else {
        return;
    };
    let list = run(&project, &["list"]);
    assert_code(&list, 0, "list");
    let listed = stdout(&list);
    assert!(listed.contains("FS-001-a"), "{listed}");
    assert!(listed.contains("FS-002-b"), "{listed}");
}

#[test]
fn without_the_entry_a_walked_file_whose_extension_is_not_utf8_is_not_read() {
    let Some(project) = beside_a_name_not_utf8("not-utf8-without", WITHOUT_ENTRY) else {
        return;
    };
    let list = run(&project, &["list"]);
    assert_code(&list, 0, "list");
    let listed = stdout(&list);
    assert!(!listed.contains("FS-001-a"), "{listed}");
    assert!(listed.contains("FS-002-b"), "{listed}");
}

/// Under the entry, a refusal's detail asks the scan's extension test of each file migration's own
/// walk finds, so it names a name whose extension is not UTF-8 as it names `FACTS.`
/// (§FS-config-migrate.3.1.9). The test has to see the path itself: the name a report prints has
/// its bad byte replaced, and what follows the last dot of that spelling is a valid extension the
/// list does not hold.
#[test]
fn a_refusal_detail_asks_the_extension_test_under_the_entry() {
    let project = root("empty-extension-detail");
    let config = format!(
        "project_name = \"r\"\n\n[id]\nnamed_sections = true\n\n[scan]\ninclude = [\"docs\"]\nexclude = [\"docs/x\"]\nextensions = {WITH_ENTRY}\n"
    );
    write(&project, "grund.toml", &config);
    write(&project, "docs/x/a.md", "plain\n");
    write(&project, "docs/x/FACTS.", "plain\n");
    if !holds_a_name_not_utf8(&project.join("docs/x")) {
        return;
    }

    let migrate = run(&project, &["config", "migrate", "--to", "2"]);
    assert_code(&migrate, 1, "config migrate");
    let report = stderr(&migrate);
    assert!(
        report.contains("it would stop reading 3 files\n"),
        "{report}"
    );
    assert!(
        report
            .contains("  would stop reading: docs/x/FACTS., docs/x/FACTS.\u{fffd}, docs/x/a.md\n"),
        "{report}"
    );
}
