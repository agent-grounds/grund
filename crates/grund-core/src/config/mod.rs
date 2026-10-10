//! The config component (§AR-system.2.3): one validated `Config` per project,
//! read from `grund.toml` and the built-in defaults (§FS-config). It consumes
//! the config file and knows nothing of the tree it describes — no walk, no
//! rule, no frontend.
//!
//! The module boundary is what §AR-system.4 asks for: an item another component
//! reads is re-exported below, and everything else is the component's own
//! (§AR-core-module-layout.1.1). The submodules are the former `config*` category
//! files plus the `Config` record that sat in `model/`: discovery, the record,
//! the reader, and one file per section of `grund.toml` that carries a grammar
//! and cross-key rules of its own — `[[kinds]]` with its built-in defaults,
//! `[citations]`, `[workspace]`, and the grounding pair — which gained the
//! per-kind level lookup `grounding_level_for_kind` when §AR-system.2.6 became a
//! module, the scanner having read it upward out of a checker file. The `[fmt]`
//! section gained a file of the same shape when §AR-system.2.8 became one: the
//! `exclude` glob compiler and the validator this reader refuses a malformed
//! pattern with, which were the formatter's and which `parse.rs` had been
//! reading upward (§AR-system.4). It builds the §FS-fmt.2.5.1 exclusion scope
//! out of them too, since §AR-system.2.11: recognizing what a rewrite may not
//! touch is the grammar's and reads no `Config`, so the compiled matcher is
//! what this component hands down.
//! The TOML basic-string escaper came the same way with the second half of that
//! component: config reads TOML and writes it back out — the `index` literal in
//! `kind.rs`, `grund config show`'s dump, and the `grund.toml` the scaffold
//! generates (§FS-init.2.4) — so the escaper belongs beside them rather than in
//! the writer that held it, which `kind.rs` had been reading upward.
//! `run_warnings.rs` is the run's warning channel, on the `Config` every walking
//! command already holds and in the place the `unread_opted_out_blocks` counter
//! it replaces sat (§DA-engine-renders-nothing, §FS-distribution.3.1).
//! `report_paths.rs` arrived the same way when §AR-system.2.9 became a module:
//! `display_path` is what `[output] relative_paths` *means* (§FS-config.3.6), and
//! every component above this one was reading it out of the deprecated path's
//! `output` category. The spelling it renders through is a function of a `Path`
//! alone and went further down, to `model/paths.rs`; this is the half that needs
//! a `Config`.
//! The concern records of §AR-config.1 are `project.rs` and `rows.rs`, with the
//! invocation's facts in `run.rs` and what is derived once in `compiled.rs`;
//! `facade.rs` builds `Config` from them and holds every per-run setter
//! (§AR-config.5), and `v1/` is the version-1 reader with its defaults
//! (§AR-config.3).

mod call_scope;
mod citations;
mod compiled;
mod discovery;
mod facade;
#[cfg(test)]
mod facade_sync;
mod fmt_block;
mod frame;
mod grounding;
mod inputs;
mod kind;
mod point_sizes;
mod project;
mod project_records;
mod record;
mod report_paths;
mod rows;
mod run;
mod run_warnings;
mod scope_roots;
mod slots;
mod v1;
mod validate;
mod workspace_block;

pub(crate) use crate::model::{check_input_observer, observe_input, with_check_input_observer};
pub use citations::{
    CitationDisjunction, CitationLevel, CitationRules, CitationTarget, KindCitationRules,
    NamespaceMatch,
};
pub(crate) use citations::{parse_citation_target_entry, render_citation_target};
pub use compiled::Compiled;
pub use frame::{Display, Frame};
pub(crate) use project_records::ProjectRecords;
// The parts of the records (§AR-config.1): an embedder reaches them through the
// public fields of `Project`, `Run` and `Compiled`, and this component's tests
// name them here.
#[allow(unused_imports)]
pub use compiled::ScanDemand;
pub(crate) use inputs::{
    input_read_dir, input_read_to_string, observe_config, observe_config_candidates,
    observe_ignore_inputs,
};
pub use kind::{KindConfig, KindIndex, KindResolution};
pub use point_sizes::{LeadSizeWarning, PointSizeUnit};
#[allow(unused_imports)]
pub use project::{
    CitationSyntax, Citations, FmtPresentation, Grounding, IdGrammar, KindGrounding, Members,
    NoteStyle, OutputPresentation, Presentation, Project, Rules, Schema, Sources,
};
pub use record::{AbsentOptionalNamespace, Config, ConfigLocation, ShorthandPolicy};
#[allow(unused_imports)]
pub use rows::{Extent, Form, Kind, Nesting, Origin, Place, Row};
#[allow(unused_imports)]
pub use run::{Run, RunScope, RunWorkspace};
#[allow(unused_imports)]
pub use slots::{Content, Handle, Presence, Slot};

// What the other components read, each by this module's path (§AR-system.4):
// the whole of what crosses this boundary, and the only thing outside the
// directory that can name any of it.
pub(crate) use discovery::{
    config_file_in, discovery_start_dir, home_form_of, load_config, load_config_at,
    load_config_at_with_report_base,
};
pub(crate) use fmt_block::fmt_excluded;
pub(crate) use grounding::{
    any_place_grounded, grounding_level_for_kind, homeless_row_grounding, row_grounding,
};
pub(crate) use kind::escape_toml_basic;
pub(crate) use point_sizes::measure_point_text;
pub(crate) use record::{
    DEFAULT_GROUNDING_LEVEL, kind_prefixes, kind_uses_values, kind_value_chapter, known_kinds_line,
    non_citable_kind_error,
};
pub(crate) use report_paths::{display_path, run_warning_findings};
pub(crate) use run_warnings::RunWarning;
pub(crate) use scope_roots::{
    canonical_config_root, root_scope_roots, unwalked_home_roots, unwalked_homes,
};
pub(crate) use v1::{parse_string_list, strip_comment};
pub(crate) use workspace_block::{
    INVALID_ALIAS_PATH_EXPECTED, both_member_lists_message, invalid_alias_path_segment,
    invalid_project_alias_message, is_valid_project_alias, optional_member_alias_segment,
};

// What only the crate's own test modules read (§AR-core-module-layout.1.3): the
// `[fmt] exclude` validator, which the suppression cases drive directly.
#[cfg(test)]
pub(crate) use fmt_block::validate_fmt_exclude;

// The cases that pin this component, one module per behaviour area
// (§AR-core-module-layout.1.3).
#[cfg(test)]
mod tests_discovery;
#[cfg(test)]
mod tests_grounding;
#[cfg(test)]
mod tests_id_grammar;
#[cfg(test)]
mod tests_kind_index;
#[cfg(test)]
mod tests_lowering;
#[cfg(test)]
mod tests_lowering_keys;
#[cfg(test)]
mod tests_non_citable_kinds;
#[cfg(test)]
mod tests_records;
#[cfg(test)]
mod tests_report_paths;
#[cfg(test)]
mod tests_scan_demand;
#[cfg(test)]
mod tests_scan_exclude;
#[cfg(test)]
mod tests_slots;
#[cfg(test)]
mod tests_v2_reader;
#[cfg(test)]
mod tests_validation;

// §FS-distribution.3.3.3: explicit embedding roots never change process cwd.
pub(crate) use call_scope::with_embedding_base;
// §FS-cli.3.4: the run's `--path-base`, scoped like the embedding base.
pub use call_scope::{PathBase, with_report_path_base};
