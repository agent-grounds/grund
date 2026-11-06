//! Process-level coverage of §FS-cli.5.1, kept with the end-to-end tests.

#[cfg(unix)]
#[path = "../../../tests/e2e/cli_os_args.rs"]
mod contract;

#[cfg(target_os = "linux")]
#[path = "support/non_utf8_fixture_contract.rs"]
mod fixture_portability;
