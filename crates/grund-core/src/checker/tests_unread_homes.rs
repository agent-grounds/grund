//! Test module: the one answer to whether a run may call an ID absent from a home it could not
//! read (§FS-check.2.4).
//! Fixture builders retain their temporary-root owner (§FS-repository-maintenance.4.1).

use std::path::Path;

use super::reference_scope::path_report_scope;
use super::unread_homes::UnreadNames;
use crate::config::{Config, load_config};
use crate::model::FailureNames;
use crate::scanner::ScanError;
use crate::testing::{test_root, write};

const DENIED: &str = "permission denied";

/// A project whose `CONST` fetches into the home `place` (`folders = [..]` or `files = [..]`) and
/// whose `PLAIN` does not. `source` is a file that makes the home exist, as a value kind's must.
fn project(name: &str, place: &str, source: &str) -> (crate::testing::TemporaryRoot, Config) {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        &format!(
            "grund_config_version = 2\nproject_name = \"demo\"\n\n[schema.kinds.CONST]\n\
             {place}\nform = \"value\"\nfetch = \"scripts/fetch\"\n\n\
             [schema.kinds.PLAIN]\nfolders = [\"plain\"]\n"
        ),
    );
    write(&root.join("docs/offer.md"), "# Offer\n");
    write(&root.join(source), "{}\n");
    let config = load_config(&root).expect("load the config");
    // The root the config resolved is the record's key; Windows spells it differently (`\\?\`).
    (root.physical(), config)
}

fn folder_project(name: &str) -> (crate::testing::TemporaryRoot, Config) {
    project(name, "folders = [\"values\"]", "values/a.json")
}

/// Whether a run over the whole project, failing at `failed`, withholds an ID of `kind`.
fn withholds(root: &Path, config: &Config, kind: &str, failed: &str) -> bool {
    let mut unread = UnreadNames::default();
    let failures = [ScanError::unread_entry(root.join(failed), DENIED)];
    let names = FailureNames::default();
    unread.record(config.frame(), &failures, &names, None);
    asks(&unread, root, config, kind)
}

fn asks(unread: &UnreadNames, root: &Path, config: &Config, kind: &str) -> bool {
    let row = config
        .schema()
        .rows
        .iter()
        .find(|row| row.name == kind)
        .expect("the kind has a row");
    unread.withholds(root, row)
}

#[test]
fn a_failure_at_or_under_the_home_withholds() {
    let (root, config) = folder_project("a_failure_at_or_under_the_home_withholds");

    assert!(withholds(&root, &config, "CONST", "values"));
    assert!(withholds(&root, &config, "CONST", "values/limits.json"));
    assert!(withholds(
        &root,
        &config,
        "CONST",
        "values/deep/limits.json"
    ));
}

#[test]
fn a_sibling_sharing_the_homes_prefix_withholds_nothing() {
    let (root, config) = folder_project("a_sibling_sharing_the_homes_prefix");

    assert!(!withholds(&root, &config, "CONST", "values2/limits.json"));
}

#[test]
fn a_file_home_is_that_file_alone() {
    let (root, config) = project(
        "a_file_home_is_that_file_alone",
        "files = [\"values/a.json\"]",
        "values/a.json",
    );

    assert!(withholds(&root, &config, "CONST", "values/a.json"));
    assert!(!withholds(&root, &config, "CONST", "values/b.json"));
}

/// The name the run prints is outside the home; only the name the walk kept is inside it.
#[test]
fn a_failure_known_only_by_another_name_withholds() {
    let (root, config) = folder_project("a_failure_known_only_by_another_name");
    let survivor = root.join("elsewhere/limits.json");
    let mut names = FailureNames::default();
    names.keep(&survivor, DENIED, &root.join("values/limits.json"));
    let mut unread = UnreadNames::default();

    unread.record(
        config.frame(),
        &[ScanError::unread_entry(survivor, DENIED)],
        &names,
        None,
    );

    assert!(asks(&unread, &root, &config, "CONST"));
}

#[test]
fn a_kind_without_fetch_withholds_nothing() {
    let (root, config) = folder_project("a_kind_without_fetch_withholds_nothing");

    assert!(!withholds(&root, &config, "PLAIN", "plain/limits.json"));
}

/// A narrowed run: the failure is only a caution there, so the finding and its exit stand.
#[test]
fn a_failure_outside_the_report_scope_withholds_nothing() {
    let (root, config) = folder_project("a_failure_outside_the_report_scope");
    let scope = path_report_scope(config.schema(), config.frame(), &root.join("docs"), true)
        .expect("resolve the scope")
        .expect("a path below the root narrows");
    let failures = [ScanError::unread_entry(
        root.join("values/limits.json"),
        DENIED,
    )];
    let mut unread = UnreadNames::default();

    unread.record(
        config.frame(),
        &failures,
        &FailureNames::default(),
        Some(&scope),
    );

    assert!(!asks(&unread, &root, &config, "CONST"));
}

/// A home is a walk root of its own, so a failure above it does not leave it unread.
#[test]
fn a_failure_above_the_home_withholds_nothing() {
    let (root, config) = project(
        "a_failure_above_the_home",
        "folders = [\"data/values\"]",
        "data/values/a.json",
    );

    assert!(!withholds(&root, &config, "CONST", "data"));
    assert!(withholds(&root, &config, "CONST", "data/values/a.json"));
}

/// The answer is the failure's name alone, whatever went wrong there.
#[test]
fn the_answer_reads_the_failures_names_never_its_reason() {
    let (root, config) = folder_project("the_answer_reads_the_failures_names");

    for reason in ["invalid JSON: 2:1", "", DENIED, "unparsable-file"] {
        let failures = [ScanError::unread_entry(
            root.join("values/limits.json"),
            reason,
        )];
        let mut unread = UnreadNames::default();
        unread.record(config.frame(), &failures, &FailureNames::default(), None);
        assert!(asks(&unread, &root, &config, "CONST"), "reason {reason:?}");
    }
}
