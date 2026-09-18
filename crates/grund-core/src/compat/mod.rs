//! The deprecated frontend inside the engine (§AR-system.2.9): the
//! `grund_core::main_entry()` path kept for `grund-core = "0.4"` consumers until
//! §REQ-backwards-compatibility.2 closes it, and the only place in this crate
//! that parses argv, writes to a stream or returns an `ExitCode`.
//!
//! It is a frontend, so it may read anything (§AR-system.4) and **nothing may
//! read it**. `tests/integration/test_engine_boundary.py` holds the first half of
//! that: this directory is the set of files that may render, and the set may
//! shrink but never grow. Each file is one command adapter of the dispatcher in
//! `cli.rs`, named for its command with the `_cmd` suffix the flat layout carried
//! dropped, plus `output.rs` — the report printers of §FS-errors.1 and
//! §FS-errors.5.
//!
//! Every data half these adapters once mixed with their printing has left
//! (§AR-core-module-layout.1): the queries into `queries/`, the writers into
//! `writers/`, the check run and the cautions into `api/run.rs` and
//! `api/scope_cautions.rs`, the path and JSON spellings into `model/` and
//! `config/`. What is left is argv, bytes and exit codes.
//!
//! Three things still cross the line the other way, and they are what the
//! retirement of this directory has to answer first. `run_integrations` is `pub`
//! and `grund-cli` imports it from the engine, so `lib.rs` re-exports it until
//! the CLI carries its own copy. And four stderr printers here are called from
//! `workspace/` on the **live** path, because §FS-check.4.7, §FS-check.4.8,
//! §FS-check.4.10 and §FS-workspace.6.1 are settled before any report exists and
//! the query surfaces have no report to carry them
//! (§DF-unlisted-workspace-block.2.3): `warn_if_members_absorb_scan`,
//! `warn_unread_block`, `warn_undecidable_ancestor_claim` and
//! `print_unlisted_workspace_block_warnings`. Those four are the one direction
//! §AR-system.4 forbids outright that this refactor could not remove without
//! redesigning how those findings reach a reader.

mod check;
mod cli;
mod completions;
mod config;
mod cover;
mod fmt;
mod id;
mod init;
mod integrations;
mod integrations_write;
mod list;
mod output;
mod refs;
mod show;
mod workspace_members;

// The deprecated process entry point (§REQ-backwards-compatibility.2) and the
// one renderer the published CLI still imports from the engine.
#[allow(deprecated)]
pub use cli::main_entry;
pub use integrations::run_integrations;
pub use output::print_config_warnings;

// The four stderr lines `workspace/` reaches on the live path — see the module
// doc above (§AR-system.4).
pub(crate) use output::print_unlisted_workspace_block_warnings;
pub(crate) use workspace_members::{
    warn_if_members_absorb_scan, warn_undecidable_ancestor_claim, warn_unread_block,
};

// What only the crate's own test modules read (§AR-core-module-layout.1): three
// command adapters, the `cover` argv parse with its two JSON fragments, and the
// `integrations` argv parse with its descriptors, form and guide URL.
#[cfg(test)]
pub(crate) use check::command_check;
#[cfg(test)]
pub(crate) use cover::{
    compat_cover_citation_json, compat_cover_project_field, parse_compat_cover_args,
};
#[cfg(test)]
pub(crate) use fmt::command_fmt;
#[cfg(test)]
pub(crate) use integrations::{
    SETUP_GUIDE_URL, client_descriptor_json, detection_plan_json, parse_integrations_args,
};
#[cfg(test)]
pub(crate) use integrations_write::EffectiveForm;
#[cfg(test)]
pub(crate) use refs::command_refs;
