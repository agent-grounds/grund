//! Process-level coverage of §FS-cli.5.1, kept with the end-to-end tests.

#[cfg(unix)]
#[path = "../../../tests/e2e/cli_os_args.rs"]
mod contract;
