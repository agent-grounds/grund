use anyhow::{Context, Result, anyhow};
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use unicode_normalization::UnicodeNormalization;

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
// Temporary re-exports for the duration of this migration: they keep every name
// the flat crate root exposed reachable at `grund_core::<name>` while the other
// components are still `include!`d flat. The finalize task replaces them with an
// explicit list.
pub use checker::*;
pub use config::*;
pub use grammar::*;
pub use model::*;
pub use queries::*;
// The scanner's glob is `pub(crate)`: the component's three public records —
// `FileStructure` and the two it holds — went down into `model/` with this move,
// so it exposes no name of its own to an embedder (§AR-core-module-layout.2).
pub(crate) use scanner::*;
pub use workspace::*;

// §AR-bindings.1: `grund-core` is the shared implementation crate used by the
// published `grund` CLI and, next, the optional LSP server. The category files
// are still included flat to keep this first package split behavior-preserving.
include!("config_cmd.rs");
include!("checker_cmd.rs");
include!("workspace_members_cmd.rs");
include!("show_cmd.rs");
include!("refs_cmd.rs");
include!("cover_cmd.rs");
include!("list_cmd.rs");
include!("completions_cmd.rs");
include!("output.rs");
include!("fmt_complete_findings.rs");
include!("fmt.rs");
include!("fmt_suppress.rs");
include!("fmt_error.rs");
include!("fmt_workspace.rs");
include!("fmt_cmd.rs");
include!("fmt_links.rs");
include!("fmt_link_anchors.rs");
include!("fmt_shorthand_links.rs");
include!("fmt_link_targets.rs");
include!("fmt_value_bindings.rs");
include!("id.rs");
include!("fetch.rs");
include!("fetch_write.rs");
include!("integrations.rs");
include!("init_templates.rs");
include!("init_citation_directions.rs");
include!("init_entrypoints.rs");
include!("init_workspace_members.rs");
include!("init_plan.rs");
include!("init_block.rs");
include!("init_notes.rs");
include!("init_target.rs");
include!("init.rs");
include!("init_cmd.rs");
include!("api.rs");
include!("api_list.rs");
include!("api_refs.rs");
include!("api_report.rs");
include!("compat_cli.rs");
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
