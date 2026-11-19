//! Process-level coverage of agent-grounds/grund#1384, kept with the end-to-end tests.
//!
//! `grund config adopt-builtins` (§FS-config-adopt-builtins.terms): which file it reads
//! (§FS-config-adopt-builtins.1.1), its flags (§FS-config-adopt-builtins.1.2), the suffixes in
//! play (§FS-config-adopt-builtins.2), the plan's four states (§FS-config-adopt-builtins.3.1,
//! §FS-config-adopt-builtins.3.2, §FS-config-adopt-builtins.3.3,
//! §FS-config-adopt-builtins.3.4), the candidate's edits (§FS-config-adopt-builtins.4.1,
//! §FS-config-adopt-builtins.4.2), its streams and report (§FS-config-adopt-builtins.5.1,
//! §FS-config-adopt-builtins.5.2, §FS-config-adopt-builtins.5.3), the gate
//! (§FS-config-adopt-builtins.6.1, §FS-config-adopt-builtins.6.2,
//! §FS-config-adopt-builtins.6.3, §FS-config-adopt-builtins.6.4), comments
//! (§FS-config-adopt-builtins.7), the write (§FS-config-adopt-builtins.8.1,
//! §FS-config-adopt-builtins.8.2), the second run (§FS-config-adopt-builtins.9), the exits
//! (§FS-config-adopt-builtins.10) and a workspace root (§FS-config-adopt-builtins.11).
//!
//! Migration naming a built-in row (§FS-config-v2.schema.sources.migration,
//! §FS-config-migrate.3.3, §FS-config-migrate.mapping), and hosts compared as a set
//! (§FS-config-migrate.5.1).

#[path = "../../../tests/e2e/config_adopt_builtins.rs"]
mod contract;
