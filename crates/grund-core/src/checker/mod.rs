//! The checker component (§AR-system.2.6): the `Report` a run's `Catalog`
//! produce — errors, warnings and suggestions, each rule one pass over part of
//! the findings (§FS-check). It consumes `Catalog`, reads a file's text after
//! the scan only in the rules §AR-checker.placement lists, and knows no frontend.
//! Design: §AR-checker, declared in `report.rs` on the entry point the whole
//! component exists to answer.
//!
//! The module boundary is what §AR-system.4 asks for: an item another component
//! reads is re-exported below, and everything else is the component's own
//! (§AR-core-module-layout.1.1). The submodules are the former `checker*` category
//! files, one per rule family §AR-checker.2 names, plus `support.rs` for what
//! every rule shares — the hint and the report's order. The
//! inline citation style rule came the other way, up out of
//! `grammar/inline_note_layout.rs`, which is where a checker rule sat while both
//! were file-name categories (§AR-system.2.1).
//!
//! What the component does **not** hold any more: five path keys and one
//! `[[kinds]]` lookup that the flat layout parked in `homes.rs` and
//! `grounding.rs`, which the scanner below read upward, went down into `model/`
//! and `config/`, and the candidate joiner went down into `workspace/` and on
//! into `resolver/` with the clause that reads it. Two more went down when
//! §AR-system.2.10 became a component: `is_stub_for_inline_decl` into
//! `model/records.rs`, a predicate over a `Declaration` beside the stub target
//! it resolves, and the resolver of §AR-resolver.1 itself — the
//! citation-to-target lookup with the `WorkspaceCheckTarget` pair it answers
//! with, which every rule here now reads downward and which the grammar's
//! shorthand resolution had been reading upward out of this component
//! (§AR-system.4). The broken-stub rule's reading of a stub's target followed,
//! out of `homes.rs` into `scanner/stub_homes.rs`, when the scan came to record
//! each stub's home with it (§AR-scanner.4.6): one reading, so the stub that rule
//! accepts is the stub the count of homes pairs.
//!
//! `plural`, the plural `s` a count earns, came here from the writers' template
//! renderer with §AR-system.2.8 and left again with §AR-system.2.11: the
//! managed block's budget sentence is the templates' now, and neither reader
//! sits below the other, so the spelling is `model/text.rs` and this component
//! reads it downward (§AR-system.4).

mod agents;
mod chapter_rules;
mod citations;
mod grounding;
mod homes;
mod index;
mod index_entries;
mod index_mention;
mod inline_style;
mod near_miss;
mod reference_scope;
mod references;
mod report;
mod sections;
mod selection;
mod shorthand;
mod sizes;
mod support;
mod value_mismatch;
mod value_roots;
mod values;

pub use selection::{CHECK_FINDING_CODES, CheckFindingSelection};

// What the other components read, each by this module's path (§AR-system.4):
// the whole of what crosses this boundary, and the only thing outside the
// directory that can name any of it.
pub(crate) use chapter_rules::{
    check_chapter_rules, configured_rule_sentences, declared_workspace_vocabulary, parse_ad_hoc,
    parse_ad_hoc_with_workspace, workspace_vocabulary,
};
// The rules component's own tests build the adapter's view the way the checker
// does (§AR-core-module-layout.1.3).
#[cfg(test)]
pub(crate) use crate::testing::{check_findings, check_with_workspace};
#[cfg(test)]
pub(crate) use chapter_rules::markdown_project;
pub(crate) use index::KindIndexFiles;
pub(crate) use index_entries::KindIndexEntries;
pub(crate) use reference_scope::{
    ScanScope, configured_scope, out_of_scope_references, path_report_scope,
    retain_diagnostics_in_report_scope, retain_findings_in_scope, scope_read_any_file,
    workspace_out_of_scope_references,
};
pub(crate) use report::{check_on_disk, check_with_workspace_and_overlays};
pub(crate) use sections::{out_of_scope_section_headings, workspace_out_of_scope_section_headings};
pub(crate) use support::sort_diagnostics;
pub(crate) use values::{
    binding_aims_at_embedded_value_authority, binding_target_has_any_value_authority,
};

// What another component's tests read (§AR-core-module-layout.1.3): the managed
// block path the drift cases drive, and the dangling sentence the scanner's
// config cases compare against; the rest went beside their own cases.
#[cfg(test)]
pub(crate) use crate::testing::check_agent_block_path;
#[cfg(test)]
pub(crate) use agents::check_agent_block_path as compare_agent_block;
#[cfg(test)]
pub(crate) use support::dangling_message;
#[cfg(test)]
pub(crate) use support::diagnostic_cmp;

// The cases that pin this component, one module per behaviour area
// (§AR-core-module-layout.1.3).
#[cfg(test)]
mod tests_alias_hints;
#[cfg(test)]
mod tests_broken_stub_fences;
#[cfg(test)]
mod tests_check_full;
#[cfg(test)]
mod tests_citation_directions;
#[cfg(test)]
mod tests_citation_levels;
#[cfg(test)]
mod tests_declaration_near_miss;
#[cfg(test)]
mod tests_duplicate_sections;
#[cfg(test)]
mod tests_finding_selection;
#[cfg(test)]
mod tests_grounding_per_place;
#[cfg(test)]
mod tests_grounding_style;
#[cfg(test)]
mod tests_inline_note_layout;
#[cfg(test)]
mod tests_kind_index;
#[cfg(test)]
mod tests_kind_index_enrollment;
#[cfg(test)]
mod tests_kind_index_entry_form;
#[cfg(test)]
mod tests_kind_index_mention;
#[cfg(test)]
mod tests_local_section_citations;
#[cfg(test)]
mod tests_managed_block_drift;
#[cfg(test)]
mod tests_nothing_recognized;
#[cfg(test)]
mod tests_post_scan_readers;
#[cfg(test)]
mod tests_shorthand;
#[cfg(test)]
mod tests_unverifiable_rule_scope;
#[cfg(test)]
mod tests_value_json_duplicates;
#[cfg(test)]
mod tests_value_roots;
