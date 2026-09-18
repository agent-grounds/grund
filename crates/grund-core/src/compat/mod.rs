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
//! Nothing here is read from below any more. The four `[workspace]` findings of
//! §FS-check.4.7, §FS-check.4.8, §FS-check.4.10 and §FS-workspace.6.1 were
//! printed from this directory on the **live** path of every walking command,
//! which was the one direction §AR-system.4 forbids outright; each is now a
//! `Diagnostic` in the run's warning channel that every frontend — this one
//! included — renders for itself (§DA-engine-renders-nothing,
//! §FS-distribution.3.1).
//!
//! Nothing here is read from outside the crate either: `main_entry` is the whole
//! of what `lib.rs` re-exports, so the engine's public surface carries no
//! renderer but the deprecated entry point itself (§FS-distribution.3.1). That
//! is what `integrations` left for — its argv and its bytes are
//! `crates/grund-cli/src/cli_integrations*.rs` now (§FS-integrations.1,
//! §AR-bindings.3) — and it is why this dispatcher answers that one command with
//! the migration §FS-distribution.3.1 names rather than a second copy of it. The
//! whole directory goes with `main_entry` in the release its deprecation note
//! names, which is the one claim about a version this component makes and it is
//! made there rather than here (§FS-distribution.4.2).

mod check;
mod cli;
mod completions;
mod config;
mod cover;
mod fmt;
mod id;
mod init;
mod list;
mod output;
mod refs;
mod show;

// The deprecated process entry point (§REQ-backwards-compatibility.2), and the
// whole of what this directory publishes: nothing here renders for another
// crate any more (§FS-distribution.3.1).
#[allow(deprecated)]
pub use cli::main_entry;

// What only the crate's own test modules read (§AR-core-module-layout.1): three
// command adapters and the `cover` argv parse with its two JSON fragments.
#[cfg(test)]
pub(crate) use check::command_check;
#[cfg(test)]
pub(crate) use cover::{
    compat_cover_citation_json, compat_cover_project_field, parse_compat_cover_args,
};
#[cfg(all(test, unix))]
pub(crate) use fmt::command_fmt;
#[cfg(test)]
pub(crate) use refs::command_refs;

// The cases that pin this component, one module per behaviour area
// (§AR-core-module-layout.1).
#[cfg(test)]
mod tests_check_finding_selection;
