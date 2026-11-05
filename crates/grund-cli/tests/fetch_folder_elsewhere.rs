//! What one case directory cannot hold about a declaration outside a `folder` home
//! (§FS-fetch.5.3): the next check's verdict, what `grund show` answers across a refusal, the way
//! through, the report base from another directory, and a mode bit git does not carry. The fixtures
//! are the e2e corpus's.
#![cfg(unix)]

#[path = "support/external_facts.rs"]
mod support;

use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use support::*;

/// The refusals of a tree that starts green: `grund check` passes before and after each.
const REFUSED_GREEN: [(&str, &str, &str); 19] = [
    ("source-python", "FS-a", "src/a.py"),
    ("source-python-function", "FS-a", "src/a.py"),
    ("source-rust", "FS-a", "src/a.rs"),
    ("source-go", "FS-a", "src/a.go"),
    ("stub-report-link", "FS-a", "src/a.py"),
    ("stub-relative-link", "FS-a", "src/a.py"),
    ("stub-other-name", "FS-a", "src/a.py"),
    ("stub-h2", "FS-a", "src/a.py"),
    ("stub-to-markdown", "FS-a", "other/a.md"),
    ("stub-under-another-name", "FS-a", "other/t.md"),
    ("markdown", "FS-a", "other/a.md"),
    ("markdown-home-absent", "FS-a", "other/a.md"),
    ("markdown-h2", "FS-a", "other/a.md"),
    ("v1-source", "FS-001-a", "src/a.py"),
    ("v1-stub", "FS-001-a", "src/a.py"),
    ("v1-markdown", "FS-001-a", "other/a.md"),
    ("v1-stub-target-unscanned", "FS-001-a", "src/a.py"),
    ("workspace-member", "alpha/FS-a", "packages/alpha/src/a.py"),
    (
        "workspace-member-stub",
        "alpha/FS-a",
        "packages/alpha/src/a.py",
    ),
];

/// The controls whose tree starts green: `<ID>.md` is created or the home's file replaced, and the
/// tree stays green and fetches again to the same bytes.
const INSTALLED_GREEN: [(&str, &str); 13] = [
    ("home-markdown-replaced", "FS-a"),
    ("broken-stub-missing-replaced", "FS-a"),
    ("broken-stub-lacks-replaced", "FS-a"),
    ("other-id-created", "FS-a"),
    ("citation-markdown-created", "FS-a"),
    ("citation-source-created", "FS-a"),
    ("fenced-created", "FS-a"),
    ("language-not-admitted-created", "FS-a"),
    ("excluded-created", "FS-a"),
    ("v1-outside-include-created", "FS-001-a"),
    ("workspace-other-member-created", "beta/FS-a"),
    ("workspace-root-member-declares-created", "FS-a"),
    ("workspace-member-root-declares-created", "alpha/FS-a"),
];

const WAY_THROUGH: &str = "keep the fact in that file, or take the declaration out of it and \
                           fetch again";

fn refusal(id: &str, file: &str) -> String {
    format!("error: cannot fetch {id}: {file} already declares it; {WAY_THROUGH}\n")
}

/// The local ID the refusal names, where a workspace fetch asks through a project's prefix.
fn local(id: &str) -> &str {
    id.rsplit('/').next().expect("split yields one part")
}

/// A copy of a case's `repo` private to one test, its fetcher executable, since `tree` carries no
/// mode: the tests run at once, and some take a mode away.
fn copy_of(test: &str, case: &str) -> PathBuf {
    let copy = root(&format!("elsewhere-{test}-{case}"));
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/e2e/cases")
        .join(format!("fetch-folder-elsewhere-{case}"))
        .join("repo");
    for (relative, bytes) in tree(&repo) {
        let path = copy.join(&relative);
        fs::create_dir_all(path.parent().expect("a file has a parent")).expect("create parent");
        fs::write(&path, bytes).expect("copy fixture file");
        if relative.ends_with("scripts/fetch") {
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755))
                .expect("make fetcher executable");
        }
    }
    copy
}

fn set_mode(path: &Path, mode: u32) {
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).expect("set fixture mode");
}

/// A superuser reads a mode-000 file, so such a fixture proves nothing there.
fn mode_bars(path: &Path) -> bool {
    fs::read(path).is_err() && fs::read_dir(path).is_err()
}

#[test]
#[ignore = "red until #1214 refuses an ID declared outside the home; implement removes this"]
fn elsewhere_a_refused_fetch_leaves_a_green_tree_green() {
    for (case, id, file) in REFUSED_GREEN {
        let name = format!("{case}-refused");
        let project = copy_of("green", &name);
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
            stderr(&fetch),
            refusal(local(id), file),
            "{case}: fetch stderr"
        );
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

/// The shape that never went red: the stub's target is outside `[scan] include`, so nothing
/// collides, and only the refusal keeps the stub and what `grund show` answers.
#[test]
#[ignore = "red until #1214 refuses an ID declared outside the home; implement removes this"]
fn elsewhere_a_stub_to_an_unscanned_target_keeps_its_text_and_its_answer() {
    let project = copy_of("show", "v1-stub-target-unscanned-refused");
    let stub = fs::read(project.join("fs/FS-001-a.md")).expect("read stub");
    let shown = run(&project, &["show", "FS-001-a", "--full"]);
    assert_code(&shown, 0, "show before");
    assert!(
        stdout(&shown).contains("Local body."),
        "show before: {}",
        stdout(&shown)
    );
    assert_code(&run(&project, &["fetch", "FS-001-a"]), 2, "fetch");
    assert_eq!(
        fs::read(project.join("fs/FS-001-a.md")).expect("read stub"),
        stub
    );
    let again = run(&project, &["show", "FS-001-a", "--full"]);
    assert_eq!(
        stdout(&again),
        stdout(&shown),
        "show moved across the refusal"
    );
    assert_code(&run(&project, &["check"]), 0, "check after");
}

/// The refusal's second answer works: with the declaration taken out of the file named, the same
/// fetch installs, and through a stub it does so because the stub is then broken.
#[test]
#[ignore = "red until #1214 refuses an ID declared outside the home; implement removes this"]
fn elsewhere_taking_the_declaration_out_lets_the_fetch_through() {
    let shapes = [
        ("source-python-refused", "src/a.py", "fs/FS-a.md"),
        ("stub-relative-link-refused", "src/a.py", "fs/FS-a.md"),
        ("markdown-refused", "other/a.md", "fs/FS-a.md"),
    ];
    for (case, declaring, snapshot) in shapes {
        let project = copy_of("way", case);
        assert_code(
            &run(&project, &["fetch", "FS-a"]),
            2,
            &format!("{case}: refused"),
        );
        write(&project, declaring, "x = 1\n");
        let fetch = run(&project, &["fetch", "FS-a"]);
        assert_code(&fetch, 0, &format!("{case}: fetch after"));
        assert_eq!(stderr(&fetch), "", "{case}: fetch stderr");
        let installed = fs::read_to_string(project.join(snapshot)).expect("read snapshot");
        assert_eq!(
            installed, "# FS-a: Alpha\n\nFetched body.\n",
            "{case}: snapshot"
        );
        assert_code(
            &run(&project, &["check"]),
            0,
            &format!("{case}: check after"),
        );
    }
}

/// §FS-fetch.5.3: `<file>` is spelled from the report base, wherever the command was run. Fetch
/// takes no `--path-base`, so the version 1 key is the one way to move that base.
#[test]
#[ignore = "red until #1214 refuses an ID declared outside the home; implement removes this"]
fn elsewhere_the_refusal_spells_the_file_from_the_report_base() {
    let project = copy_of("base", "source-python-refused");
    let from_home = run(&project.join("fs"), &["fetch", "FS-a"]);
    assert_code(&from_home, 2, "from the home");
    assert_eq!(stderr(&from_home), refusal("FS-a", "src/a.py"));

    let workspace = copy_of("base", "workspace-member-refused");
    let from_member = run(&workspace.join("packages/alpha"), &["fetch", "FS-a"]);
    assert_code(&from_member, 2, "from the member");
    assert_eq!(stderr(&from_member), refusal("FS-a", "src/a.py"));

    let project = copy_of("base", "v1-source-refused");
    let config = fs::read_to_string(project.join("grund.toml")).expect("read config");
    write(
        &project,
        "grund.toml",
        &format!("{config}\n[output]\nrelative_paths = false\n"),
    );
    let from_home = run(&project.join("fs"), &["fetch", "FS-001-a"]);
    assert_code(&from_home, 2, "from the home, relative_paths = false");
    assert_eq!(stderr(&from_home), refusal("FS-001-a", "../src/a.py"));
}

/// Guards that pass today: what declares nothing outside the home leaves a green tree green, and
/// a second fetch writes the same bytes.
#[test]
fn elsewhere_an_installed_snapshot_stays_green_and_fetches_again() {
    for (case, id) in INSTALLED_GREEN {
        let project = copy_of("installed", case);
        assert_code(&run(&project, &["fetch", id]), 0, &format!("{case}: fetch"));
        assert_code(
            &run(&project, &["check"]),
            0,
            &format!("{case}: check after"),
        );
        let installed = tree_with_directories(&project);
        let again = run(&project, &["fetch", id]);
        assert_code(&again, 0, &format!("{case}: second fetch"));
        assert_eq!(
            tree_with_directories(&project),
            installed,
            "{case}: second fetch changed the tree"
        );
    }
}

/// One way the scan steps past a path outside the home, and how to put the tree back.
struct Unreadable {
    name: &'static str,
    make: fn(&Path),
    restore: fn(&Path),
    probe: Option<&'static str>,
}

/// Guards that pass today: §FS-check.2.4's per-file failures outside the home do not stop the
/// fetch, and `grund check` already exits 2 on each, so an exit 0 turns nothing red.
#[test]
fn elsewhere_a_path_the_scan_cannot_read_does_not_stop_the_fetch() {
    let cases = [
        Unreadable {
            name: "file",
            make: |project| {
                write(project, "src/b.py", "x = 1\n");
                set_mode(&project.join("src/b.py"), 0o000);
            },
            restore: |project| set_mode(&project.join("src/b.py"), 0o644),
            probe: Some("src/b.py"),
        },
        Unreadable {
            name: "directory",
            make: |project| {
                write(project, "src/b/c.py", "x = 1\n");
                set_mode(&project.join("src/b"), 0o000);
            },
            restore: |project| set_mode(&project.join("src/b"), 0o755),
            probe: Some("src/b"),
        },
        Unreadable {
            name: "dangling-link",
            make: |project| {
                fs::create_dir_all(project.join("src")).expect("create src");
                symlink("gone.py", project.join("src/link.py")).expect("make link");
            },
            restore: |_| {},
            probe: None,
        },
        Unreadable {
            name: "config-root",
            make: |project| set_mode(project, 0o311),
            restore: |project| set_mode(project, 0o755),
            probe: Some("."),
        },
    ];
    for case in cases {
        let project = copy_of(case.name, "other-id-created");
        (case.make)(&project);
        if let Some(probe) = case.probe
            && !mode_bars(&project.join(probe))
        {
            (case.restore)(&project);
            continue;
        }
        let check = run(&project, &["check"]);
        let fetch = run(&project, &["fetch", "FS-a"]);
        let check_code = check.status.code();
        let fetch_code = fetch.status.code();
        let fetch_stderr = stderr(&fetch);
        (case.restore)(&project);
        assert_eq!(
            check_code,
            Some(2),
            "{}: check before: {}",
            case.name,
            stderr(&check)
        );
        assert_eq!(fetch_code, Some(0), "{}: fetch: {fetch_stderr}", case.name);
        assert_eq!(fetch_stderr, "", "{}: fetch stderr", case.name);
        assert!(
            project.join("fs/FS-a.md").is_file(),
            "{}: snapshot not created",
            case.name
        );
    }
}

/// Guards that pass today: inside the home a file discovery cannot read stays a discovery failure
/// under §FS-fetch.5. The line it prints is that point's to word, so only the verdict is held.
#[test]
fn elsewhere_a_file_in_the_home_discovery_cannot_read_still_stops_the_fetch() {
    let project = copy_of("home-undecodable", "other-id-created");
    fs::write(project.join("fs/c.md"), b"\xff\xfe not utf-8\n").expect("write undecodable");
    let before = tree_with_directories(&project);
    assert_code(&run(&project, &["fetch", "FS-a"]), 2, "undecodable");
    assert_eq!(tree_with_directories(&project), before, "undecodable: tree");

    let project = copy_of("home-unreadable", "other-id-created");
    write(&project, "fs/c.md", "# FS-c: Gamma\n");
    let unreadable = project.join("fs/c.md");
    set_mode(&unreadable, 0o000);
    if fs::read(&unreadable).is_ok() {
        set_mode(&unreadable, 0o644);
        return;
    }
    let before = tree_listing(&project);
    let code = run(&project, &["fetch", "FS-a"]).status.code();
    let after = tree_listing(&project);
    set_mode(&unreadable, 0o644);
    assert_eq!(code, Some(2), "unreadable");
    assert_eq!(after, before, "unreadable: tree");
}

/// The paths of a tree whose modes may bar reading one of its files.
fn tree_listing(project: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut pending = vec![project.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir).expect("list fixture directory") {
            let path = entry.expect("list fixture entry").path();
            if path.is_dir() {
                pending.push(path.clone());
            }
            paths.push(path);
        }
    }
    paths.sort();
    paths
}
