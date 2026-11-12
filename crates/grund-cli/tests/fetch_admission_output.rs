//! What one case directory cannot hold about what the integration prints (§FS-fetch.admission):
//! a value chapter that is written and then resolves (§FS-fetch.3), a write a second fetch keeps
//! byte for byte (§FS-fetch.admission.5), and a refusal that leaves a green tree green
//! (§FS-fetch.admission.3). The fixtures are the e2e corpus's.
#![cfg(unix)]

#[path = "support/external_facts.rs"]
mod support;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use support::*;

const ID: &str = "AR-001-thing";

/// The refusals whose tree starts green and that the next `grund check` would have turned red.
const REFUSED_GREEN: [&str; 6] = [
    "fetch-section-deep-refused",
    "fetch-section-duplicate-refused",
    "fetch-section-gap-warn-refused",
    "fetch-value-chapter-prose-refused",
    "fetch-bare-heading-h4-refused",
    "fetch-warning-only-refused",
];

/// Writes that a second fetch of the same output must leave byte for byte.
const WRITTEN_TWICE_ALIKE: [&str; 5] = [
    "fetch-section-numbered-written",
    "fetch-section-two-folder-written",
    "fetch-closed-fence-written",
    "fetch-refetch-control-file-written",
    "fetch-refetch-final-bare-cr-no-sibling-written",
];

/// A copy of a case's `repo` private to one test, its fetcher executable.
fn copy_of(test: &str, case: &str) -> PathBuf {
    let copy = root(&format!("admission-output-{test}-{case}"));
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/e2e/cases")
        .join(case)
        .join("repo");
    for (relative, bytes) in tree(&repo) {
        let path = copy.join(&relative);
        fs::create_dir_all(path.parent().expect("a file has a parent")).expect("create parent");
        fs::write(&path, bytes).expect("copy fixture file");
        if relative.starts_with("scripts/") {
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755))
                .expect("make fetcher executable");
        }
    }
    copy
}

/// §FS-fetch.3: `fetch` keeps no depth limit, so a value chapter is written and its value is read.
#[test]
#[ignore = "red until #1205 drops fetch's depth limit for admission; implement removes this"]
fn admission_output_a_fetched_value_chapter_resolves_its_value() {
    let project = copy_of("value", "fetch-value-chapter-written");
    assert_code(&run(&project, &["check"]), 0, "check before");
    let fetch = run(&project, &["fetch", ID]);
    assert_code(&fetch, 0, "fetch");
    assert_eq!(stderr(&fetch), "", "fetch stderr");
    let value = run(&project, &["AR-001-thing.values.limit.1"]);
    assert_code(&value, 0, "read the value");
    assert!(
        stdout(&value).contains("##### values.limit.1: 99"),
        "value: {}",
        stdout(&value)
    );
    assert_code(&run(&project, &["check"]), 0, "check after");
}

/// §FS-fetch.admission.3: output the next check would fault leaves a green tree green.
#[test]
#[ignore = "red until #1205 runs check's node-local rules on the candidate; implement removes this"]
fn admission_output_a_refusal_leaves_a_green_tree_green() {
    for case in REFUSED_GREEN {
        let project = copy_of("green", case);
        assert_code(
            &run(&project, &["check"]),
            0,
            &format!("{case}: check before"),
        );
        assert_code(&run(&project, &["fetch", ID]), 2, &format!("{case}: fetch"));
        assert_code(
            &run(&project, &["check"]),
            0,
            &format!("{case}: check after"),
        );
    }
}

/// §FS-fetch.admission.5: what admission writes, a second fetch of the same output keeps.
#[test]
fn admission_output_a_write_fetches_again_to_the_same_bytes() {
    for case in WRITTEN_TWICE_ALIKE {
        let project = copy_of("twice", case);
        assert_code(
            &run(&project, &["fetch", ID]),
            0,
            &format!("{case}: first fetch"),
        );
        let first = tree_with_directories(&project);
        assert_code(
            &run(&project, &["fetch", ID]),
            0,
            &format!("{case}: second fetch"),
        );
        assert_eq!(
            tree_with_directories(&project),
            first,
            "{case}: second fetch"
        );
        assert_code(
            &run(&project, &["check"]),
            0,
            &format!("{case}: check after"),
        );
    }
}
