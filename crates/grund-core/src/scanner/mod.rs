//! The scanner component (§AR-system.2.5): one walk of the tree that reads every
//! file once and produces `Findings` — every declaration, section, citation,
//! value binding and grounding unit in it (§FS-check.1, §AR-scanner). It consumes
//! the scope and the grammar, knows no rule and no frontend, and never asks
//! whether it is in a workspace.
//!
//! The module boundary is what §AR-system.4 asks for: an item another component
//! reads is re-exported below, and everything else is the component's own
//! (§AR-core-module-layout.1). The submodules are the former `scanner*` and
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
//! went down into `grammar/` (§AR-system.2.1). Those three records were also the
//! only `pub` names the scanner had, so nothing it re-exports below is `pub` and
//! it adds no name to the embedding surface of §AR-core-module-layout.2.

mod citations;
mod context;
mod e2e;
mod embedded_value_context;
mod embedded_values;
mod file_pass;
mod json;
mod legacy;
mod legacy_inline;
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

// What the other components read, still through the crate root while they are
// flat (§AR-system.4). The finalize task narrows this as each caller moves into
// a module of its own.
pub(crate) use context::{file_home_kind, markdown_heading_level, section_path_is_numeric};
pub(crate) use e2e::e2e_case_dir_name;
pub(crate) use embedded_value_context::EMBEDDED_VALUE_MARKER;
pub(crate) use file_pass::CitationLine;
pub(crate) use legacy::{
    collect_local_legacy_markdown_citations, formatter_wrapper_label_is_citation,
    legacy_catalog_ids, match_legacy_tail, promote_qualified_legacy_citations, resolve_id_arg,
};
pub(crate) use scope_probe::effective_scope_reads_any_file;
pub(crate) use tree::{
    ScanError, canonicalize_existing_prefix, overlay_text, scan_tree, scan_tree_strict,
    scan_tree_with_workspace_overlays,
};
pub(crate) use walk::{
    canonical_config_root, root_scope_roots, scan_roots_for, unwalked_home_roots,
    walk_reads_any_file, walk_scannable_files, walk_scannable_files_reporting,
};
pub(crate) use walk_boundaries::{is_hidden, is_scannable};

// What only the crate's own test modules read (§AR-core-module-layout.1): the
// exact embedded-value marker test, the scope probe's injectable half, the two
// narrower tree entry points, and the Markdown value component reader.
#[cfg(test)]
pub(crate) use embedded_value_context::exact_embedded_value_marker;
#[cfg(test)]
pub(crate) use scope_probe::effective_scope_reads_any_file_with;
#[cfg(test)]
pub(crate) use tree::{scan_tree_with_workspace, scan_tree_with_workspace_threshold};
#[cfg(test)]
pub(crate) use values::markdown_component;
