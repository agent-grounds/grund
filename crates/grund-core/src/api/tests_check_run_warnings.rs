//! Check warning-channel API contracts (§FS-distribution.3.1).

use anyhow::Result;

use super::{CheckOpts, CheckOutput, check_with_opts};
use crate::testing::{test_root, write};

#[test]
fn check_with_opts_keeps_its_result_output_contract() {
    let root = test_root("check_with_opts_keeps_its_result_output_contract");
    write(&root.join("grund.toml"), "grund_config_version = 1\n");

    let output: Result<CheckOutput> = check_with_opts(CheckOpts {
        path: root,
        path_provided: true,
        ..CheckOpts::default()
    });

    assert!(
        output.is_ok(),
        "the established check API still returns output"
    );
}
