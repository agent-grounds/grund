//! Process-level coverage of §FS-config-migrate.6.5.1 with no working directory.

#[cfg(unix)]
#[path = "../../../tests/e2e/config_migrate_deleted_directory.rs"]
mod contract;
