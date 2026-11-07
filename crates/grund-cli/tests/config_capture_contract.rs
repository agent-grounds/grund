//! §FS-config-migrate.4.3: skipping migration capture changes no CLI verdict or bytes.
//! These existing golden cases protect preservation; the failing capture
//! regression lives beside the v1 reader, and check_large_10k's existing
//! instruction benchmark remains the performance acceptance test.

use std::path::PathBuf;

#[path = "support/case_runner.rs"]
#[allow(dead_code)]
mod case_runner;

use case_runner::{CaseKind::E2e, assert_every_case_passed, run_case};

/// §FS-config-migrate.4.1, §FS-config-migrate.4.2, §FS-config-migrate.4.3,
/// §FS-config-migrate.4.4: comments are carried, listed or explicitly dropped.
/// §FS-config-migrate.5.1, §FS-config-migrate.5.2, §FS-config-migrate.5.3:
/// migration still applies its records, report and guidance gates.
/// §FS-config.3.5.16, §FS-config.4.3: ordinary loads keep their located refusal;
/// migration keeps its own diagnosis (§FS-config-migrate.3.1.4).
#[test]
fn config_capture_contract_preserves_cli_outputs() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let cases = root.join("tests/e2e/cases");
    let outcomes = [
        "config-migrate-comments-carried",
        "config-migrate-comments-refused",
        "config-migrate-drop-comments",
        "config-migrate-preview-minimal",
        "config-migrate-write",
        "config-migrate-refuse-values-guidance",
        "config-migrate-refuse-path-shaped-exclude",
        "config-migrate-invalid-config",
        "config-migrate-workspace-member",
        "config-scan-exclude-slash-single-check",
        "config-scan-exclude-slash-single-validate",
        "show-config-invalid-cli-error",
    ]
    .iter()
    .map(|name| run_case(&root, &cases.join(name), E2e))
    .collect::<Vec<_>>();
    assert_every_case_passed("config capture preservation", &outcomes);
}
