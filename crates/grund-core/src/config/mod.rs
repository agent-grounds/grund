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

mod citations;
mod discovery;
mod fmt_block;
mod grounding;
mod kind;
mod kind_defaults;
mod kind_table;
mod kind_values;
mod parse;
mod point_sizes;
mod record;
mod report_paths;
mod run_warnings;
mod scope_roots;
mod workspace_block;

pub use citations::{
    CitationDisjunction, CitationLevel, CitationRules, CitationTarget, KindCitationRules,
    NamespaceMatch,
};
pub(crate) use citations::{parse_citation_target_entry, render_citation_target};
pub use kind::{KindConfig, KindIndex, KindResolution};
pub use point_sizes::{LeadSizeWarning, PointSizeUnit};
pub use record::{AbsentOptionalNamespace, Config, ConfigLocation, ShorthandPolicy};

// What the other components read, each by this module's path (§AR-system.4):
// the whole of what crosses this boundary, and the only thing outside the
// directory that can name any of it.
pub(crate) use discovery::{
    config_file_in, home_form_of, load_config, load_config_at, load_config_at_with_report_base,
};
pub(crate) use fmt_block::fmt_excluded;
pub(crate) use grounding::grounding_level_for_kind;
pub(crate) use kind::escape_toml_basic;
pub(crate) use parse::{parse_string_list, strip_comment};
pub(crate) use point_sizes::measure_point_text;
pub(crate) use record::{
    DEFAULT_GROUNDING_LEVEL, kind_prefixes, kind_uses_values, kind_value_chapter,
    non_citable_kind_error,
};
pub(crate) use report_paths::{display_path, run_warning_findings};
pub(crate) use run_warnings::RunWarning;
pub(crate) use scope_roots::{
    canonical_config_root, root_scope_roots, unwalked_home_roots, unwalked_homes,
};
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
mod tests_non_citable_kinds;
#[cfg(test)]
mod tests_validation;
