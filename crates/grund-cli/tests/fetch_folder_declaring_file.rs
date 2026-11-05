//! What one case directory cannot hold about a file in a `folder` home that declares the requested
//! ID (§FS-fetch.5.2): the next check's verdict, the way through the refusal, the report base from
//! another directory, and a mode bit git does not carry. The fixtures are the e2e corpus's.
#![cfg(unix)]

#[path = "support/external_facts.rs"]
mod support;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use support::*;

/// The reproducer's nine cases: each starts green, with one comment declaration in the home.
const REFUSED: [(&str, &str); 9] = [
    ("fetch-folder-source-v2-python-refused", "FS-a"),
    ("fetch-folder-source-v2-python-nested-refused", "FS-a"),
    ("fetch-folder-source-v2-python-function-refused", "FS-a"),
    ("fetch-folder-source-v2-rust-refused", "FS-a"),
    ("fetch-folder-source-v2-go-refused", "FS-a"),
    ("fetch-folder-source-v2-ruby-refused", "FS-a"),
    ("fetch-folder-source-v1-txt-refused", "FS-001-a"),
    ("fetch-folder-source-v1-python-refused", "FS-001-a"),
    ("fetch-folder-source-v1-rust-refused", "FS-001-a"),
];

/// The controls: nothing in these homes declares the ID, so `<ID>.md` is created and is green.
const CREATED: [(&str, &str); 6] = [
    ("fetch-folder-source-other-id-created", "FS-a"),
    ("fetch-folder-source-citation-created", "FS-a"),
    ("fetch-folder-source-not-a-doc-position-created", "FS-a"),
    ("fetch-folder-source-language-not-admitted-created", "FS-a"),
    (
        "fetch-folder-source-extension-not-scanned-created",
        "FS-001-a",
    ),
    ("fetch-folder-source-excluded-created", "FS-001-a"),
];

const WAY_THROUGH: &str = "keep the fact in that file, or take the declaration out of it and \
                           fetch again";

fn refusal(id: &str, file: &str) -> String {
    format!("error: cannot fetch {id}: {file} already declares it; {WAY_THROUGH}")
}

/// A copy of a case's `repo` private to one test, its fetcher executable, since `tree` carries no
/// mode: the tests run at once, and one of them takes a file's mode away.
fn copy_of(test: &str, case: &str) -> PathBuf {
    let copy = root(&format!("declaring-file-{test}-{case}"));
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/e2e/cases")
        .join(case)
        .join("repo");
    for (relative, bytes) in tree(&repo) {
        let body = String::from_utf8(bytes).expect("these fixtures are UTF-8");
        if relative == "scripts/fetch" {
            executable(&copy, &relative, &body);
        } else {
            write(&copy, &relative, &body);
        }
    }
    copy
}

#[test]
fn a_refused_fetch_leaves_a_green_tree_green() {
    for (case, id) in REFUSED {
        let project = copy_of("green", case);
        assert_code(
            &run(&project, &["check"]),
            0,
            &format!("{case}: check before"),
        );
        let before = tree_with_directories(&project);
        let fetch = run(&project, &["fetch", id]);
        assert_code(&fetch, 2, &format!("{case}: fetch"));
        assert_eq!(stdout(&fetch), "", "{case}: fetch stdout");
        assert_eq!(
            tree_with_directories(&project),
            before,
            "{case}: refusal changed the tree"
        );
        assert_code(
            &run(&project, &["check"]),
            0,
            &format!("{case}: check after"),
        );
    }
}

/// Guards that pass today: a file that does not declare the ID must not stop the fetch.
#[test]
fn a_created_snapshot_leaves_the_tree_green_and_fetches_again() {
    for (case, id) in CREATED {
        let project = copy_of("created", case);
        assert_code(&run(&project, &["fetch", id]), 0, &format!("{case}: fetch"));
        assert_code(
            &run(&project, &["check"]),
            0,
            &format!("{case}: check after"),
        );
        let created = tree_with_directories(&project);
        assert_code(
            &run(&project, &["fetch", id]),
            0,
            &format!("{case}: second fetch"),
        );
        assert_eq!(
            tree_with_directories(&project),
            created,
            "{case}: second fetch moved"
        );
    }
}

#[test]
fn taking_the_declaration_out_of_the_file_is_the_way_through() {
    let project = copy_of("way-through", "fetch-folder-source-v2-python-refused");
    assert_code(
        &run(&project, &["fetch", "FS-a"]),
        2,
        "fetch while the file declares",
    );
    write(&project, "fs/a.py", "x = 1\n");
    assert_code(
        &run(&project, &["fetch", "FS-a"]),
        0,
        "fetch once it does not",
    );
    assert_code(&run(&project, &["check"]), 0, "check after");
    assert!(
        project.join("fs/FS-a.md").is_file(),
        "the snapshot is created"
    );
}

#[test]
fn the_refusal_spells_the_file_from_the_report_base() {
    // §FS-cli.3.4: the default base is the config root, wherever the run stands.
    let project = copy_of("base", "fetch-folder-source-v2-python-refused");
    let fetch = run(&project.join("fs"), &["fetch", "FS-a"]);
    assert_code(&fetch, 2, "from the home");
    assert_eq!(stderr(&fetch).trim_end(), refusal("FS-a", "fs/a.py"));

    // §FS-config.3.6.1: `relative_paths = false` is the invocation base, so it reads from there.
    let project = copy_of("base", "fetch-folder-source-v1-python-refused");
    let config = fs::read_to_string(project.join("grund.toml")).expect("read the config");
    write(
        &project,
        "grund.toml",
        &format!("{config}\n[output]\nrelative_paths = false\n"),
    );
    let fetch = run(&project.join("fs"), &["fetch", "FS-001-a"]);
    assert_code(&fetch, 2, "from the home, on the invocation base");
    assert_eq!(stderr(&fetch).trim_end(), refusal("FS-001-a", "a.py"));
}

#[test]
fn a_file_it_cannot_read_is_a_discovery_failure() {
    let project = copy_of("unreadable", "fetch-folder-source-v2-python-refused");
    let before = tree_with_directories(&project);
    let source = project.join("fs/a.py");
    fs::set_permissions(&source, fs::Permissions::from_mode(0o000)).expect("take its mode");
    // The mode does not bar root, which proves nothing here.
    let barred = fs::read(&source).is_err();
    let check = run(&project, &["check"]);
    let fetch = run(&project, &["fetch", "FS-a"]);
    fs::set_permissions(&source, fs::Permissions::from_mode(0o644)).expect("give its mode back");
    if !barred {
        return;
    }
    let checked = stderr(&check);
    let line = checked
        .lines()
        .find(|line| line.starts_with("error: fs/a.py: "))
        .expect("check names the file it cannot read");
    assert_code(&fetch, 2, "fetch");
    assert_eq!(stdout(&fetch), "", "fetch stdout");
    assert_eq!(stderr(&fetch).trim_end(), line, "the line check prints");
    assert_eq!(
        tree_with_directories(&project),
        before,
        "refusal changed the tree"
    );
}
