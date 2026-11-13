//! Configured integration roots follow §FS-repository-maintenance.4.1 and
//! §FS-repository-maintenance.4.2, independently of a parent fixture's cleanup.

mod support;
use support::test_root;

#[path = "../../grund-cli/tests/support/fixture_root_contract.rs"]
mod fixture_root_contract;
