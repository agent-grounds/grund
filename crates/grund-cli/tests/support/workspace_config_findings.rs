//! Issue #1134's cases, run alone: at a workspace root `config validate` names the config
//! findings `check` names there, and `config show` only its own config's (§FS-config.4.1.1,
//! §FS-config.4.2, §FS-check.4.11.3). Every one is also in the full pass.

use super::case_runner::CaseKind::E2e;
use super::case_runner::{assert_every_case_passed, run_case};
use super::repo_root;

const CASES: [&str; 3] = [
    "config-validate-workspace-member-findings",
    "config-validate-workspace-ok",
    "config-show-workspace-root-only",
];

#[test]
#[ignore = "red until #1134 names member config findings under config validate; implement removes this"]
fn workspace_config_findings_contract() {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let outcomes = CASES
        .iter()
        .map(|name| run_case(&root, &cases.join(name), E2e))
        .collect::<Vec<_>>();
    assert_every_case_passed("workspace config findings", &outcomes);
}
