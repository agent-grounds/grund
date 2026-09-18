// §AR-system.2.2: the model component is one Rust module, so its records are
// declared in `model/` and what crosses the boundary is what `model/mod.rs`
// re-exports (§AR-core-module-layout.1).
mod model;
// §AR-system.2.1: the grammar component is one Rust module too, so the lexical
// facts are declared in `grammar/` and what crosses the boundary is what
// `grammar/mod.rs` re-exports (§AR-core-module-layout.1).
mod grammar;
// §AR-system.2.3: config is one Rust module too, so the effective `Config` and
// the reader that fills it are declared in `config/` and what crosses the
// boundary is what `config/mod.rs` re-exports (§AR-core-module-layout.1).
mod config;
// §AR-system.2.4: workspace is one Rust module too, so member expansion, the
// claims and the loaded project set are declared in `workspace/` and what
// crosses the boundary is what `workspace/mod.rs` re-exports.
mod workspace;
// §AR-system.2.5: the scanner is one Rust module too, so the walk, the per-file
// pass and everything they record are declared in `scanner/` and what crosses
// the boundary is what `scanner/mod.rs` re-exports (§AR-scanner).
mod scanner;
// §AR-system.2.6: the checker is one Rust module too, so every rule and the
// report they fill are declared in `checker/` and what crosses the boundary is
// what `checker/mod.rs` re-exports (§AR-checker).
mod checker;
// §AR-system.2.7: the queries are one Rust module too, so every answer read
// from `Findings` is declared in `queries/` and what crosses the boundary is
// what `queries/mod.rs` re-exports (§FS-show, §FS-list, §FS-lsp).
mod queries;
// §AR-system.2.8: the writers are one Rust module too — the formatter, the ID
// proposal, the init scaffold (§FS-init) and the integrations artifacts
// (§FS-integrations) — and `writers/mod.rs` says what crosses the boundary.
mod writers;
// §AR-system.2.9: the embedding surface is one Rust module too, so the public
// contract and the adapters that fill it are declared in `api/` and what an
// embedder reaches is what `api/mod.rs` re-exports (§AR-bindings.2).
mod api;
// §AR-system.2.9: and its one exception, the deprecated `main_entry()` path,
// which is the only directory here that parses argv, writes to a stream or
// returns an `ExitCode`. It may read anything; nothing may read it.
mod compat;

// Temporary re-exports for the duration of this migration: they keep every name
// the flat crate root exposed reachable at `grund_core::<name>` while the other
// components are still `include!`d flat. The finalize task replaces them with an
// explicit list.
pub use api::*;
pub use checker::*;
pub use config::*;
pub use grammar::*;
pub use model::*;
pub use queries::*;
pub use workspace::*;
pub use writers::*;

// The scanner's glob is `pub(crate)`: `ApiScanError` came in from the api's
// contract (§AR-system.2.9) and is its one `pub` name, so that one is listed
// explicitly (§AR-core-module-layout.2).
pub use scanner::ApiScanError;

pub(crate) use scanner::*;

// The deprecated frontend's public names, listed rather than globbed: the 0.4
// process entry point §REQ-backwards-compatibility.2 keeps, and the two below.
#[allow(deprecated)]
pub use compat::main_entry;

// The `warning:` shape both `config` frontends share (§FS-config.4.2), and the
// one renderer `grund-cli` still imports from the engine, which the
// compat-retirement step replaces with a copy of its own (§AR-system.2.9).
pub use compat::{print_config_warnings, run_integrations};

// Nothing may *read* `compat/` (§AR-system.4), so its glob is `pub(crate)`: it
// carries the four stderr lines `workspace/` still reaches on the live path, and
// the crate's own `tests_*` modules.
pub(crate) use compat::*;

// The crate's own `tests_*` modules are `include!`d into this root and reach
// their vocabulary through `use super::*`, so the std names their fixtures spell
// are imported here rather than in each of them (§AR-core-module-layout.1).
#[cfg(test)]
use anyhow::Result;
#[cfg(test)]
use std::collections::{BTreeMap, BTreeSet};
#[cfg(test)]
use std::fs;
#[cfg(test)]
use std::path::{Path, PathBuf};
#[cfg(test)]
use std::process::ExitCode;

// Tests, one module per category (§AR-core-module-layout.1). `tests_support`
// holds the fixtures they share and must come first.
include!("tests_support.rs");
include!("tests_config_discovery.rs");
include!("tests_config_scan.rs");
include!("tests_id_grammar.rs");
include!("tests_check_full.rs");
include!("tests_kind_index.rs");
include!("tests_kind_index_config.rs");
include!("tests_non_citable_kinds.rs");
include!("tests_unwalked_kinds.rs");
include!("tests_kind_index_entry_form.rs");
include!("tests_kind_index_enrollment.rs");
include!("tests_check_full_scope.rs");
include!("tests_nothing_recognized.rs");
include!("tests_declaration_near_miss.rs");
include!("tests_duplicate_sections.rs");
include!("tests_section_body_scope.rs");
include!("tests_section_outside_declaration.rs");
include!("tests_comment_block.rs");
include!("tests_comment_block_position.rs");
include!("tests_grounding_style.rs");
include!("tests_grounding_per_place.rs");
include!("tests_grounding_config.rs");
include!("tests_inline_note_layout.rs");
include!("tests_inline_note_layout_check.rs");
include!("tests_scanner.rs");
include!("tests_unmarked_headings.rs");
include!("tests_values.rs");
include!("tests_embedded_values.rs");
include!("tests_embedded_value_boundaries.rs");
include!("tests_scanner_walk.rs");
include!("tests_scanner_walk_roots.rs");
include!("tests_scanner_walk_errors.rs");
include!("tests_shorthand.rs");
include!("tests_shorthand_rewrite.rs");
include!("tests_shorthand_docstring.rs");
include!("tests_shorthand_surfaces.rs");
include!("tests_shorthand_numeric_run.rs");
include!("tests_fmt_suppression.rs");
include!("tests_fmt_workspace.rs");
include!("tests_citation_directions.rs");
include!("tests_citation_directions_render.rs");
include!("tests_managed_block_drift.rs");
include!("tests_check_finding_selection.rs");
include!("tests_workspace.rs");
include!("tests_workspace_message_paths.rs");
include!("tests_workspace_nested.rs");
include!("tests_workspace_claims.rs");
include!("tests_workspace_claim_answers.rs");
include!("tests_workspace_absorbed_scan.rs");
include!("tests_unlisted_workspace_block.rs");
include!("tests_unread_opted_out_block.rs");
include!("tests_alias_hints.rs");
include!("tests_workspace_members.rs");
include!("tests_workspace_optional_members.rs");
include!("tests_cover_workspace.rs");
include!("tests_init_agents.rs");
include!("tests_init_scan_guidance.rs");
include!("tests_init_target.rs");
include!("tests_integrations.rs");
include!("tests_integrations_config.rs");
include!("tests_resolver.rs");
include!("tests_clickable_citations.rs");
include!("tests_api.rs");
include!("tests_refs_query_failures.rs");
include!("tests_external_facts.rs");
include!("tests_fmt_errors.rs");
include!("tests_lsp_hover.rs");
