//! §FS-init.2.2.2 (issue #973): a `grund init` re-run whose every reported path is
//! `exists ` keeps the `next:` block while the selected home is missing
//! (§FS-init.2.2.2.1) or no citable kind is declared (§FS-init.2.2.2.2), and a
//! complete setup still prints none. Run alone through the ordinary e2e fixture
//! runner; each case also runs in the full e2e suite.
#[path = "support/case_runner.rs"]
#[allow(dead_code)]
mod case_runner;

use case_runner::{CaseKind::E2e, assert_every_case_passed, run_case};

#[test]
fn init_rerun_keeps_the_next_block_until_setup_is_complete() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let cases = root.join("tests/e2e/cases");
    let outcomes = [
        "init-rerun-fs-home-missing",
        "init-rerun-req-home-missing",
        "init-rerun-no-kinds",
        "init-dry-run-rerun-fs-home-missing",
        "init-check-rerun-no-kinds",
        "init-rerun-complete-setup",
    ]
    .iter()
    .map(|name| run_case(&root, &cases.join(name), E2e))
    .collect::<Vec<_>>();
    assert_every_case_passed("init re-run next: block", &outcomes);
}
