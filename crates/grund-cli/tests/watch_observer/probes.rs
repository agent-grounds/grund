//! Absent and lexical input probes before rediscovery (§FS-check.6.1.1).
use crate::support::*;
use grund::WatchObservation;
#[cfg(unix)]
use grund::{watch_test, watch_test::Snapshot};
#[cfg(unix)]
use grund_core::CheckFindingSelection;

#[test]
fn watch_missing_stub_creation_has_coverage_before_completion() {
    let _serial = crate::support::serial();
    let f = Fixture::new();
    f.write(
        "docs/functional-spec/FS-live.md",
        "# FS-live: [definition](../../absent/impl.rs)\n",
    );
    let target = f.0.join("absent/impl.rs");
    let before = f.ordinary();
    assert_eq!(before.status, 1);
    let mut h = Harness::start(&f, None, None);
    h.matches(&before);
    assert!(
        h.history.iter().any(|event| {
            matches!(event, WatchObservation::Input(input) if input.path == target)
        })
    );
    f.write("absent/impl.rs", "/// FS-live: Definition\n");
    let after = f.ordinary();
    assert_eq!(after.status, 0);
    assert_ne!(before.stdout, after.stdout);
    h.matches(&after);
    h.stop(0);
}

/// §FS-check.6.1.1: creation at the subscribed candidate barrier affects the
/// very first selection, rather than merely arranging a later repair run.
#[test]
fn watch_stub_candidate_is_subscribed_before_selection() {
    let _serial = crate::support::serial();
    let f = Fixture::new();
    f.write(
        "docs/functional-spec/FS-live.md",
        "# FS-live: [definition](../../absent/impl.rs)\n",
    );
    let barrier = format!("input:{}", f.0.join("absent/impl.rs").display());
    let mut h = Harness::start(&f, None, Some(&barrier));
    h.stage(&barrier);
    assert!(
        h.history.iter().any(|event| {
            matches!(event, WatchObservation::Subscribed(path, _) if path == &f.0)
        })
    );
    assert!(
        !h.history
            .iter()
            .any(|event| matches!(event, WatchObservation::Completed(..)))
    );
    f.write("absent/impl.rs", "/// FS-live: Definition\n");
    let expected = f.ordinary();
    assert_eq!(expected.status, 0);
    h.release();
    let initial = h.completed();
    assert_eq!(initial.status, expected.status);
    assert_eq!(initial.stdout, expected.stdout);
    assert_eq!(initial.stderr, expected.stderr);
    h.stop(0);
}

/// §FS-check.6.1.1, §AR-bindings.3: ancestor discovery in a Rayon project
/// worker blocks after subscription and before ignore's read.
#[test]
fn watch_workspace_worker_subscribes_ancestor_ignore_before_read() {
    let _serial = crate::support::serial();
    let f = Fixture::new();
    f.write(
        "child/grund.toml",
        &format!("project_name = \"child\"\n{CONFIG}\n[workspace]\nmembers = [\"leaf\"]\n"),
    );
    f.write("child/src/main.rs", BAD);
    f.write("child/leaf/grund.toml", CONFIG);
    f.write("child/leaf/src/main.rs", "// \u{a7}FS-leaf-missing\n");
    let mut opts = f.opts();
    opts.path = f.0.join("child");
    let ordinary = || {
        grund::watch_test::one_shot(
            opts.clone(),
            &grund_core::CheckFindingSelection::default(),
            None,
        )
    };
    let before = ordinary();
    assert!(String::from_utf8_lossy(&before.stdout).contains("FS-missing"));
    let barrier = format!("input:{}", f.0.join(".ignore").display());
    let mut h = Harness::options(
        opts.clone(),
        grund_core::CheckFindingSelection::default(),
        None,
        Some(&barrier),
        None,
    );
    h.stage(&barrier);
    assert!(
        h.history.iter().any(|event| {
            matches!(event, WatchObservation::Subscribed(path, _) if path == &f.0)
        })
    );
    f.write(".ignore", "child/src/main.rs\n");
    let after = ordinary();
    assert_ne!(before.stdout, after.stdout);
    assert!(!String::from_utf8_lossy(&after.stdout).contains("unknown reference FS-missing"));
    h.release();
    let initial = h.completed();
    assert_eq!(initial.stdout, after.stdout);
    assert_eq!(initial.stderr, after.stderr);
    assert_eq!(initial.status, after.status);
    std::fs::remove_file(f.0.join(".ignore")).unwrap();
    h.matches(&before);
    h.stop(before.status);
}

/// §FS-check.6.1.3, §FS-check.6.1.4: lexical anchors survive physical retargeting.
#[cfg(unix)]
#[test]
fn watch_lexical_invocation_link_retargets_and_refreshes() {
    use std::os::unix::fs::symlink;
    let _serial = crate::support::serial();
    let anchor = Fixture::new();
    let first = Fixture::new();
    let second = Fixture::new();
    second.write("src/main.rs", BAD);
    let link = anchor.0.join("project");
    symlink(&first.0, &link).unwrap();
    let mut opts = anchor.opts();
    opts.path = link.clone();
    opts.path_provided = true;
    let ordinary = || -> Snapshot {
        watch_test::one_shot(
            opts.clone(),
            &CheckFindingSelection::default(),
            Some("text"),
        )
    };
    let before = ordinary();
    assert_eq!(before.status, 0);
    let mut h = Harness::options(
        opts.clone(),
        CheckFindingSelection::default(),
        Some("text"),
        None,
        None,
    );
    h.matches(&before);
    let lexical = h
        .history
        .iter()
        .position(|event| matches!(event, WatchObservation::Input(input) if input.path == link))
        .expect("lexical invocation subscribed before discovery");
    let config = h.history.iter().position(|event| {
        matches!(event, WatchObservation::Input(input) if input.path == first.0.join("grund.toml"))
    }).expect("first project discovery");
    assert!(lexical < config);
    symlink(&second.0, anchor.0.join("next-link")).unwrap();
    std::fs::rename(anchor.0.join("next-link"), &link).unwrap();
    let after = ordinary();
    assert_eq!(after.status, 1);
    assert_ne!(before.stdout, after.stdout);
    h.matches(&after);
    second.write("src/main.rs", CLEAN);
    let repaired = ordinary();
    assert_eq!(repaired.status, 0);
    h.matches(&repaired);
    h.stop(0);
}
