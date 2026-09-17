//! The checker component (§AR-system.2.6): the `Report` a run's `Findings`
//! produce — errors, warnings and suggestions, each rule one pass over part of
//! the findings (§FS-check). It consumes `Findings`, reads no file except in the
//! two rules that must, and knows no frontend. Design: §AR-checker, declared in
//! `report.rs` on the entry point the whole component exists to answer.
//!
//! The module boundary is what §AR-system.4 asks for: an item another component
//! reads is re-exported below, and everything else is the component's own
//! (§AR-core-module-layout.1). The submodules are the former `checker*` category
//! files, one per rule family §AR-checker.2 names, plus `support.rs` for what
//! every rule shares — the resolver, the hint, and the report's order. The
//! inline citation style rule came the other way, up out of
//! `grammar/inline_note_layout.rs`, which is where a checker rule sat while both
//! were file-name categories (§AR-system.2.1).
//!
//! What the component does **not** hold any more: five path keys and one
//! `[[kinds]]` lookup that the flat layout parked in `homes.rs` and
//! `grounding.rs`, which the scanner below read upward, went down into `model/`
//! and `config/`, and the candidate joiner went down into `workspace/`.
//!
//! One more item came up when §AR-system.2.8 became a module: `plural`, the
//! plural `s` a count earns, which `inline_style.rs` had been reading out of the
//! writers' template renderer. This is the lowest component that reads it, so it
//! lands here and the writers read it downward (§AR-system.4).

mod agents;
mod citations;
mod grounding;
mod homes;
mod index;
mod index_entries;
mod inline_style;
mod near_miss;
mod references;
mod report;
mod sections;
mod selection;
mod sizes;
mod support;
mod values;

pub use selection::{CHECK_FINDING_CODES, CheckFindingSelection};

// What the other components read, still through the crate root while they are
// flat (§AR-system.4). The finalize task narrows this as each caller moves into
// a module of its own.
pub(crate) use homes::{file_declares_inline_home, is_stub_for_inline_decl};
pub(crate) use index::KindIndexFiles;
pub(crate) use index_entries::KindIndexEntries;
pub(crate) use inline_style::plural;
pub(crate) use references::{
    ReferenceTier, WorkspaceCheckTarget, configured_scope, out_of_scope_references,
    retain_findings_in_scope, workspace_out_of_scope_references,
};
pub(crate) use report::{check_findings, check_with_workspace, check_with_workspace_and_overlays};
pub(crate) use sections::{out_of_scope_section_headings, workspace_out_of_scope_section_headings};
pub(crate) use support::{diagnostic_cmp, sort_diagnostics};
pub(crate) use values::binding_target_has_any_value_authority;

// What only the crate's own test modules read (§AR-core-module-layout.1): the
// managed-block halves the drift cases drive, the dangling sentence, the release
// ramp the index rule is on, and the three steps of the unknown-project hint.
#[cfg(test)]
pub(crate) use agents::{check_agent_block_path, section_in_block};
#[cfg(test)]
pub(crate) use index::{INDEX_RULE_PRIOR_RELEASE, INDEX_RULE_RELEASE};
#[cfg(test)]
pub(crate) use references::{nearest_project_aliases, tag_out_of_scope, unknown_project_message};
#[cfg(test)]
pub(crate) use support::dangling_message;
