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
