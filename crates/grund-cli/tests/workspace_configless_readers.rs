//! Reader selection at a configless member (§FS-config.4.2, §FS-config-explain.2, §FS-schema.2).

#[path = "../../../tests/integration/workspace_configless_readers.rs"]
mod api;
#[path = "../../../tests/e2e/workspace_configless_readers.rs"]
mod cli;
#[path = "../../../tests/integration/support/configless_member.rs"]
mod fixture;
