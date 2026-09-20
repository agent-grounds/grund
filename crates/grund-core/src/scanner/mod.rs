//! The scanner component (§AR-system.2.5): one walk of the tree that reads every
//! file once and produces `Findings` — every declaration, section, citation,
//! value binding and grounding unit in it (§FS-check.1, §AR-scanner). It consumes
//! the scope and the grammar, knows no rule and no frontend, and never asks
//! whether it is in a workspace.
//!
//! The module boundary is what §AR-system.4 asks for: an item another component
//! reads is re-exported below, and everything else is the component's own
//! (§AR-core-module-layout.1.1). The submodules are the former `scanner*` and
//! `value_json` category files, one per machine §AR-scanner names — the directory
//! traversal of §AR-scanner.1 in `walk*`, the per-file line pass of
//! §AR-scanner.2 in `file_pass`, the passes it calls out to for citations,
//! values and grounding structure, the tree-level driver that merges one
//! `Findings` out of many in `tree`, and the off-grammar reconciliation of
//! §FS-config.3.2 in `legacy*`.
//!
//! What the component does **not** hold any more: the `FileStructure` records
//! §AR-scanner.2.7 fills went down into `model/`, because `Findings` carries them
//! and they are plain data, and five lexical items the flat layout parked here
//! went down into `grammar/` (§AR-system.2.1). The snapshot canonicalization of
//! §AR-lsp.5.1 followed them into `model/paths.rs` when §AR-system.2.9 became a
//! module, the queries and the writers both rebasing a path against it.
//!
//! What the component does not hold any more either, since §AR-system.2.10
//! became one: the qualified half of §FS-config.3.2.6's catalog reconciliation,
//! which takes the whole loaded project set and is `resolver/legacy_promotion.rs`
//! now — leaving it here would have had this component name the record of one
//! above it (§AR-system.4). The per-candidate promotion it calls stays, and
//! three of its steps are `pub(crate)` for that one reader. Three walk-level
//! facts went down into `config/scope_roots.rs` and `model/paths.rs` with the
//! same move — `root_scope_roots`, `canonical_config_root` and `is_hidden`, each
//! a question about a configuration or a `Path` and not about a tree — because
//! workspace was reading all three upward before any scan exists
//! (§AR-resolver.placement).
//!
//! Two more crossings came with §AR-system.2.11. `formatter_wrapper_label_is_citation`
//! went down into `grammar/fmt_cross_refs.rs`, beside the flattening that is its
//! only other caller: whether a `[…](…)` label is a citation the formatter could
//! have wrapped is a question about a token (§DF-show-cross-ref-flattening).
//! And `agent_entrypoints.rs` came the other way, out of the writers: which
//! entrypoint files a repository *has* is a probe over the tree — it resolves
//! symlinks and reads a file for a managed block — and `grund check`'s companion
//! scan was reading it upward out of a component above the checker
//! (§FS-check.3.5.1, §AR-checker.2.7).
//!
//! What came the other way with that move is `scan_error.rs`: the published form
//! of a scan failure (§FS-check.2.4), which sat in the api's contract while the
//! size catalog and the formatter read it upward (§AR-system.4). It carries the
//! one `pub` name this component now adds to the embedding surface of
//! §AR-core-module-layout.2, so `lib.rs` re-exports `ApiScanError` explicitly
//! beside the component's otherwise `pub(crate)` glob.

mod agent_entrypoints;
mod citations;
mod context;
mod e2e;
mod embedded_value_context;
mod embedded_values;
mod file_pass;
mod json;
mod legacy;
mod legacy_inline;
mod scan_error;
mod scope_probe;
mod tree;
mod units;
mod unmarked_headings;
mod value_context;
mod value_json;
mod value_json_enrollment;
mod values;
mod walk;
mod walk_boundaries;
mod walk_errors;

pub use scan_error::ApiScanError;

// What the other components read, each by this module's path (§AR-system.4):
// the whole of what crosses this boundary, and the only thing outside the
// directory that can name any of it.
pub(crate) use agent_entrypoints::{
    AgentEntrypoint, CANONICAL_AGENT_ENTRYPOINT, COMPANION_AGENT_ENTRYPOINTS,
    CanonicalSurfaceReach, CompanionAgentEntrypoint, InitCompanionAgentEntrypoint,
    agents_with_own_entrypoint, companion_agent_entrypoints, companion_workspace_exists,
    existing_init_companion_agent_entrypoints, is_file_or_symlink, is_symlink_to,
    path_missing_without_following_symlinks,
};
pub(crate) use context::{file_home_kind, markdown_heading_level, section_path_is_numeric};
pub(crate) use e2e::e2e_case_dir_name;
pub(crate) use embedded_value_context::EMBEDDED_VALUE_MARKER;
pub(crate) use legacy::{
    collect_local_legacy_markdown_citations, configured_catalog_ids, legacy_catalog_ids,
    match_legacy_tail, promote_legacy_candidate, resolve_id_arg, sort_citations,
};
pub(crate) use scan_error::api_scan_error;
pub(crate) use scope_probe::effective_scope_reads_any_file;
pub(crate) use tree::{
    ScanError, overlay_text, scan_tree, scan_tree_strict, scan_tree_with_workspace_overlays,
};
pub(crate) use walk::{
    scan_roots_for, walk_reads_any_file, walk_scannable_files, walk_scannable_files_reporting,
};
pub(crate) use walk_boundaries::is_scannable;

// What another component's tests read (§AR-core-module-layout.1.3): the
// workspace-wide tree entry point, which the cross-project citation cases
// drive. The four narrower reads went beside their own cases, below.
#[cfg(test)]
pub(crate) use tree::scan_tree_with_workspace;

// The cases that pin this component, one module per behaviour area
// (§AR-core-module-layout.1.3).
#[cfg(test)]
mod tests_config_scan;
#[cfg(test)]
mod tests_embedded_value_boundaries;
#[cfg(test)]
mod tests_embedded_values;
#[cfg(test)]
mod tests_file_pass;
#[cfg(test)]
mod tests_inline_site;
#[cfg(test)]
mod tests_local_section_citations;
#[cfg(test)]
mod tests_scope_probe;
#[cfg(test)]
mod tests_section_body_scope;
#[cfg(test)]
mod tests_section_outside_declaration;
#[cfg(test)]
mod tests_unmarked_headings;
#[cfg(test)]
mod tests_unwalked_kinds;
#[cfg(test)]
mod tests_value_json;
#[cfg(test)]
mod tests_values;
#[cfg(all(test, unix))]
mod tests_walk;
#[cfg(all(test, unix))]
mod tests_walk_errors;
#[cfg(all(test, unix))]
mod tests_walk_roots;
