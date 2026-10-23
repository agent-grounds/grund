//! §FS-list.1.3.2 (issue #962): `grund list --help` teaches only `--unused` forms that
//! work under the default kinds `grund init` writes, where `--kind E2E` is refused,
//! while a config-v2 `E2E` kind keeps §FS-list.1.3.1's rule. Run alone through the
//! ordinary e2e fixture runner; each case also runs in the full e2e suite.
#[path = "support/case_runner.rs"]
#[allow(dead_code)]
mod case_runner;

use case_runner::{CaseKind::E2e, assert_every_case_passed, run_case};

#[test]
fn list_unused_teaching_follows_the_config() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let cases = root.join("tests/e2e/cases");
    let outcomes = [
        "cli-help-list",
        "list-unused-init-layout-bare",
        "list-unused-init-layout-kind-fs",
        "list-unused-init-layout-kind-e2e-refused",
        "list-unused-e2e-v2",
        "list-unused-e2e-v2-excluded",
    ]
    .iter()
    .map(|name| run_case(&root, &cases.join(name), E2e))
    .collect::<Vec<_>>();
    assert_every_case_passed("list --unused teaching", &outcomes);
}
