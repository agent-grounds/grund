//! The config component (§AR-system.2.3): one validated `Config` per project,
//! read from `grund.toml` and the built-in defaults (§FS-config). It consumes
//! the config file and knows nothing of the tree it describes — no walk, no
//! rule, no frontend.
//!
//! The module boundary is what §AR-system.4 asks for: an item another component
//! reads is re-exported below, and everything else is the component's own
//! (§AR-core-module-layout.1). The submodules are the former `config*` category
//! files plus the `Config` record that sat in `model/`: discovery, the record,
//! the reader, and one file per section of `grund.toml` that carries a grammar
//! and cross-key rules of its own — `[[kinds]]` with its built-in defaults,
//! `[citations]`, `[workspace]`, and the grounding pair — which gained the
//! per-kind level lookup `grounding_level_for_kind` when §AR-system.2.6 became a
//! module, the scanner having read it upward out of a checker file.

mod citations;
mod discovery;
mod grounding;
mod kind;
mod kind_defaults;
mod kind_table;
mod parse;
mod point_sizes;
mod record;
mod workspace_block;

pub use citations::{
    CitationDisjunction, CitationLevel, CitationRules, CitationTarget, KindCitationRules,
    NamespaceMatch,
};
pub use kind::{KindConfig, KindIndex, KindResolution};
pub use point_sizes::{LeadSizeWarning, PointSizeUnit};
pub use record::{AbsentOptionalNamespace, Config, ConfigLocation, ShorthandPolicy};

// What the other components read, still through the crate root while they are
// flat (§AR-system.4). The finalize task narrows this as each caller moves into
// a module of its own.
pub(crate) use citations::render_citation_target;
pub(crate) use discovery::{
    config_file_in, home_form_of, load_config, load_config_at, load_config_at_with_report_base,
};
pub(crate) use grounding::grounding_level_for_kind;
pub(crate) use parse::{parse_string_list, strip_comment};
pub(crate) use record::{
    DEFAULT_GROUNDING_LEVEL, kind_prefixes, kind_uses_values, non_citable_kind_error,
};
pub(crate) use workspace_block::{
    INVALID_ALIAS_PATH_EXPECTED, both_member_lists_message, invalid_alias_path_segment,
    invalid_project_alias_message, is_valid_project_alias, optional_member_alias_segment,
};
