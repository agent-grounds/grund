//! Issue #1138's cases, run alone: at a workspace root `check` and `config validate` name the
//! config findings of each `include_root = false` block the workspace expansion reaches, as
//! they name a member's, and a narrowed or imported run names none (§FS-check.4.11.3,
//! §FS-check.checks.config-version-1, §FS-config.4.1.1). Every one is also in the full pass.

use super::case_runner::CaseKind::E2e;
use super::case_runner::{assert_every_case_passed, run_case};
use super::repo_root;

const CASES: [&str; 8] = [
    "workspace-grouping-block-version-1-check",
    "workspace-grouping-block-version-1-validate",
    "workspace-grouping-block-version-1-ignore",
    "workspace-grouping-block-version-1-json",
    "workspace-grouping-block-config-findings-check",
    "workspace-grouping-block-config-findings-validate",
    "workspace-grouping-block-narrowed-member",
    "workspace-grouping-block-imported",
];

#[test]
fn grouping_block_config_findings_contract() {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let outcomes = CASES
        .iter()
        .map(|name| run_case(&root, &cases.join(name), E2e))
        .collect::<Vec<_>>();
    assert_every_case_passed("grouping block config findings", &outcomes);
}
