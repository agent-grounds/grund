//! Fetch validates scanner-recorded section structure before either home writer runs
//! (§FS-fetch.3.sections). Port the #1162 reproducer's four failures and three controls,
//! then pin the configuration and prefix semantics that a shared validator must preserve.
#![cfg(unix)]

#[path = "support/external_facts.rs"]
mod support;

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use support::{assert_code, executable, run, stderr, stdout, tree_with_directories, write};

static NEXT: AtomicUsize = AtomicUsize::new(0);
const ID: &str = "AR-001-thing";

struct Fixture {
    root: PathBuf,
    folder: bool,
}

impl Fixture {
    fn new(version: u8, folder: bool, existing: bool, mode: &str, named: bool) -> Self {
        let root = std::env::temp_dir().join(format!(
            "grund-1162-sections-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&root).unwrap();
        let home = if folder { "docs/arch" } else { "docs/arch.md" };
        let index = if folder { "index = false\n" } else { "" };
        let config = if version == 1 {
            let place = if folder { "folder" } else { "file" };
            format!(
                "grund_config_version = 1\n[id]\nnamed_sections = {named}\n\
                 section_heading_levels = \"{mode}\"\n\
                 [[kinds]]\nkind = \"AR\"\n{place} = \"{home}\"\n\
                 fetch = \"scripts/fetch\"\n{index}[scan]\ninclude = [\"docs\"]\n"
            )
        } else {
            let place = if folder { "folders" } else { "files" };
            format!(
                "grund_config_version = 2\nproject_name = \"sections\"\n\
                 [schema]\nid_format = \"{{kind}}-{{number}}-{{slug}}\"\nheading_depth = \"{mode}\"\n\
                 [schema.sources]\nlanguages = [\"markdown\"]\n\
                 [schema.kinds.AR]\n{place} = [\"{home}\"]\n{index}fetch = \"scripts/fetch\"\n"
            )
        };
        write(&root, "grund.toml", &config);
        // Keep unrelated bytes and a sibling even when the target is absent.
        write(&root, "sentinel.bin", "unowned\0bytes\r\n");
        if folder {
            write(
                &root,
                "docs/arch/other.md",
                "# AR-002-other: Other\n\nkept\n",
            );
        } else {
            write(
                &root,
                home,
                "# Architecture\n\n## AR-002-other: Other\n\nkept\n",
            );
        }
        let fixture = Self { root, folder };
        if existing {
            let previous = fixture.snapshot("1. Old");
            if folder {
                write(&fixture.root, fixture.destination(), &previous);
            } else {
                let mut bytes = std::fs::read_to_string(fixture.root.join(home)).unwrap();
                bytes.push_str(&previous);
                write(&fixture.root, home, &bytes);
            }
        }
        fixture
    }

    fn destination(&self) -> &'static str {
        if self.folder {
            "docs/arch/AR-001-thing.md"
        } else {
            "docs/arch.md"
        }
    }

    fn snapshot(&self, headings: &str) -> String {
        let native = if self.folder { "#" } else { "##" };
        let sections = headings
            .lines()
            .map(|heading| format!("{native}# {heading}\n\nprose\n\n"))
            .collect::<String>();
        format!("{native} {ID}: Thing\n\n{sections}")
    }

    fn fetcher(&self, snapshot: &str) {
        executable(
            &self.root,
            "scripts/fetch",
            &format!("#!/bin/sh\ncat <<'OUT'\n{snapshot}OUT\n"),
        );
    }

    fn refused(&self, snapshot: &str, diagnostic: &str, coordinate: &str) {
        self.fetcher(snapshot);
        assert_code(&run(&self.root, &["check"]), 0, "green before fetch");
        let before = tree_with_directories(&self.root);
        let fetched = run(&self.root, &["fetch", ID]);
        // If the defect stands, first prove that the written output triggers the
        // reported checker rule, rather than a fixture/configuration mistake.
        if fetched.status.code() == Some(0) {
            let checked = run(&self.root, &["check"]);
            assert_code(&checked, 1, "accepted output makes check red");
            assert!(stdout(&checked).contains(diagnostic), "{checked:?}");
            eprintln!(
                "accepted invalid {coordinate}: tree changed={}, next check:\n{}",
                tree_with_directories(&self.root) != before,
                stdout(&checked)
            );
        }
        assert_code(&fetched, 2, diagnostic);
        assert!(stdout(&fetched).is_empty(), "refusal belongs on stderr");
        assert!(stderr(&fetched).contains(coordinate), "{fetched:?}");
        assert!(!stderr(&fetched).is_empty());
        assert_eq!(tree_with_directories(&self.root), before);
        assert_code(&run(&self.root, &["check"]), 0, "green after refusal");
    }

    fn accepted(&self, snapshot: &str) {
        self.fetcher(snapshot);
        assert_code(&run(&self.root, &["check"]), 0, "green before fetch");
        let mut expected = tree_with_directories(&self.root);
        if self.folder {
            expected.insert(
                self.destination().into(),
                Some(snapshot.as_bytes().to_vec()),
            );
        } else {
            expected.insert(
                self.destination().into(),
                Some(
                    format!("# Architecture\n\n{snapshot}## AR-002-other: Other\n\nkept\n")
                        .into_bytes(),
                ),
            );
        }
        let fetched = run(&self.root, &["fetch", ID]);
        assert_code(&fetched, 0, "valid section structure");
        assert!(stdout(&fetched).is_empty());
        assert_eq!(tree_with_directories(&self.root), expected);
        assert_code(&run(&self.root, &["check"]), 0, "green after valid fetch");
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.root).unwrap();
    }
}

fn reported_shape(headings: &str, diagnostic: &str, coordinate: &str) {
    for (version, folder, existing, mode) in [(1, false, true, "strict"), (2, true, false, "must")]
    {
        let fixture = Fixture::new(version, folder, existing, mode, true);
        fixture.refused(&fixture.snapshot(headings), diagnostic, coordinate);
    }
}

#[test]
fn numbered_path_deeper_than_heading_is_refused() {
    reported_shape("1.2. Deep", "heading level mismatch", "AR-001-thing.1.2");
}

#[test]
fn named_path_deeper_than_heading_is_refused() {
    // Supply the prefix so this tests the level rule independently of orphans.
    reported_shape(
        "notes: Notes\nnotes.more: More",
        "heading level mismatch",
        "AR-001-thing.notes.more",
    );
}

#[test]
fn duplicate_numbered_path_is_refused() {
    reported_shape("1. One\n1. Again", "duplicate section", "AR-001-thing.1");
}

#[test]
fn duplicate_named_path_is_refused() {
    reported_shape(
        "notes: Notes\nnotes: Again",
        "duplicate section",
        "AR-001-thing.notes",
    );
}

#[test]
fn name_bearing_orphan_is_refused_even_when_depth_is_not_an_error() {
    for (version, folder, mode) in [
        (1, false, "loose"),
        (2, true, "warn"),
        (2, false, "should"),
        (2, true, "may"),
    ] {
        let fixture = Fixture::new(version, folder, false, mode, true);
        // Both all-named and mixed numeric/named paths require a prefix.
        for path in ["notes.more: More", "1.more: More"] {
            fixture.refused(
                &fixture.snapshot(path),
                "orphan section",
                &format!("{ID}.{}", path.split(':').next().unwrap()),
            );
        }
    }
}

#[test]
fn duplicates_are_errors_in_every_heading_depth_mode() {
    for (version, mode) in [
        (1, "strict"),
        (1, "warn"),
        (1, "loose"),
        (2, "must"),
        (2, "warn"),
        (2, "should"),
        (2, "may"),
    ] {
        // v1 named sections disabled: numeric duplicates are still errors.
        let fixture = Fixture::new(version, version == 1, false, mode, false);
        fixture.refused(
            &fixture.snapshot("1. One\n1. Again"),
            "duplicate section",
            "AR-001-thing.1",
        );
    }
}

#[test]
fn relative_and_complete_paths_claiming_one_coordinate_are_refused() {
    reported_shape(
        "notes: First\n.notes: Again",
        "duplicate section",
        "AR-001-thing.notes",
    );
}

#[test]
fn invalid_sections_preserve_an_existing_folder_snapshot() {
    let fixture = Fixture::new(2, true, true, "must", true);
    fixture.refused(
        &fixture.snapshot("notes: Notes\nnotes: Again"),
        "duplicate section",
        "AR-001-thing.notes",
    );
}

#[test]
fn invalid_sections_preserve_absent_file_and_folder_destinations() {
    for (version, folder, mode) in [(2, false, "must"), (1, true, "strict")] {
        let fixture = Fixture::new(version, folder, false, mode, true);
        std::fs::remove_dir_all(fixture.root.join("docs")).unwrap();
        fixture.refused(
            &fixture.snapshot("1. One\n1. Again"),
            "duplicate section",
            "AR-001-thing.1",
        );
    }
}

#[test]
fn reproducer_controls_are_accepted_verbatim_and_check_green() {
    for (version, folder, mode) in [(1, false, "strict"), (2, true, "must")] {
        for headings in ["1. One", "1. One\n2. Two", "notes: Notes"] {
            let fixture = Fixture::new(version, folder, false, mode, true);
            fixture.accepted(&fixture.snapshot(headings));
        }
    }
}

#[test]
fn non_error_depth_strengths_preserve_numeric_prefix_gaps() {
    for (version, mode) in [
        (1, "warn"),
        (1, "loose"),
        (2, "warn"),
        (2, "should"),
        (2, "may"),
    ] {
        let fixture = Fixture::new(version, version == 1, false, mode, true);
        fixture.accepted(&fixture.snapshot("1.2. Deep"));
    }
}

#[test]
fn name_bearing_prefixes_use_the_complete_path_set() {
    let fixture = Fixture::new(2, true, false, "may", true);
    fixture.accepted(&fixture.snapshot("notes.more: More\nnotes: Notes"));
}

#[test]
fn fenced_invalid_sections_are_content_with_named_sections_enabled_or_disabled() {
    for named in [true, false] {
        let fixture = Fixture::new(1, false, false, "strict", named);
        let mut snapshot = fixture.snapshot("1. One");
        snapshot
            .push_str("```markdown\n### 1.2. Deep\n### notes.more: Orphan\n### 1. Again\n```\n\n");
        fixture.accepted(&snapshot);
    }
}
