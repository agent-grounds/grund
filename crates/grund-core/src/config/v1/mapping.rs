//! §AR-config.3: the v1 lowering table as data — where each key of the format
//! in force, and each meaning a v1 file gets without writing one, lands in the
//! `Project`. Each entry is `(name, record path)`, the name spelled the way
//! §DF-config-concerns.2.3 spells it and the path the page's path or a field
//! below it (§AR-config.3.3 item 3). The unit tests in `config/tests_lowering_keys.rs`
//! and `config/tests_lowering.rs`
//! lower a file per row and hold each row to the one field it names
//! (§AR-config.3.3 item 4).

/// §AR-config.3.1: one row per key.
pub(in crate::config) const KEYS: &[(&str, &str)] = &[
    ("grund_config_version", "version"),
    ("project_name", "name"),
    ("project_description", "presentation.description"),
    ("[reference] conversation", "presentation.conversation"),
    ("[reference] grounding_level", "rules.grounding.level"),
    ("[reference] inline_note_layout", "schema.notes.layout"),
    (
        "[reference] inline_note_layout_check",
        "schema.notes.layout_check",
    ),
    (
        "[reference] inline_note_max_columns",
        "schema.notes.max_columns",
    ),
    (
        "[reference] inline_note_max_lines",
        "schema.notes.max_lines",
    ),
    (
        "[reference] inline_note_suggested_lines",
        "schema.notes.suggested_lines",
    ),
    ("[reference] inline_style", "schema.notes.inline_style"),
    ("[reference] lead_size_warning", "schema.leads"),
    ("[reference] marker", "schema.citation.marker"),
    ("[reference] require_grounding", "rules.grounding.require"),
    ("[reference] shorthand", "schema.citation.shorthand"),
    ("[reference] strict", "schema.citation.strict"),
    ("[reference] trigger", "presentation.trigger"),
    (
        "[reference] warn_on_suggested",
        "schema.notes.warn_on_suggested",
    ),
    ("[id] format", "schema.ids.format"),
    ("[id] named_sections", "schema.ids.named_sections"),
    ("[id] number_pattern", "schema.ids.number_pattern"),
    (
        "[id] section_heading_levels",
        "schema.ids.section_heading_levels",
    ),
    ("[id] section_separator", "schema.ids.section_separator"),
    ("[id] slug_pattern", "schema.ids.slug_pattern"),
    ("[[kinds]] citable", "schema.rows"),
    ("[[kinds]] fetch", "schema.rows"),
    ("[[kinds]] file", "schema.rows"),
    ("[[kinds]] folder", "schema.rows"),
    ("[[kinds]] format", "schema.rows"),
    ("[[kinds]] grounding_level", "rules.grounding.kinds"),
    ("[[kinds]] index", "schema.rows"),
    ("[[kinds]] kind", "schema.rows"),
    ("[[kinds]] require_grounding", "rules.grounding.kinds"),
    ("[[kinds]] resolve", "rules.resolution"),
    ("[[kinds]] rules", "schema.rows"),
    ("[[kinds]] scan", "schema.rows"),
    ("[[kinds]] title", "presentation.kinds"),
    ("[[kinds]] value_chapter", "schema.rows"),
    ("[[kinds]] values", "schema.rows"),
    ("[scan] comment_prefixes", "schema.sources.comment_prefixes"),
    ("[scan] docstring_python", "schema.sources.docstring_python"),
    ("[scan] exclude", "schema.sources.exclude"),
    ("[scan] extensions", "schema.sources.extensions"),
    ("[scan] include", "schema.sources.include"),
    (
        "[scan] respect_gitignore",
        "schema.sources.respect_gitignore",
    ),
    ("[output] color", "presentation.output.color"),
    ("[output] format", "presentation.output.format"),
    (
        "[output] relative_paths",
        "presentation.output.relative_paths",
    ),
    (
        "[fmt.cross_refs] anchor_format",
        "presentation.fmt.anchor_format",
    ),
    (
        "[fmt.cross_refs] enabled",
        "presentation.fmt.cross_refs_enabled",
    ),
    ("[workspace] include_root", "workspace.include_root"),
    ("[workspace] members", "workspace.members"),
    ("[workspace] optional_members", "workspace.optional_members"),
    ("[citations] default", "rules.citations"),
    ("[citations.<KIND>] default", "rules.citations"),
    ("[citations.<KIND>] may", "rules.citations"),
    ("[citations.<KIND>] must", "rules.citations"),
    ("[citations.<KIND>] must-not", "rules.citations"),
    ("[citations.<KIND>] should", "rules.citations"),
    ("[citations.<KIND>] should-not", "rules.citations"),
    ("[fmt] exclude", "presentation.fmt.exclude"),
];

/// §AR-config.3.2: one row per implicit default, each applied by the v1 reader
/// in `defaults.rs` or `kind_table.rs` and nowhere else.
pub(in crate::config) const DEFAULTS: &[(&str, &str)] = &[
    ("built-in kinds", "schema.rows"),
    ("legacy FS home", "schema.rows"),
    ("name-keyed index", "schema.rows"),
    ("fetch resolves must", "rules.resolution"),
    ("kind grounding inherits", "rules.grounding.kinds"),
    ("complement name", "schema.rows"),
    ("nesting", "schema.nesting"),
    ("strict", "schema.citation.strict"),
    ("named sections", "schema.ids.named_sections"),
    ("scan exclusions", "schema.sources.exclude"),
    ("version", "version"),
];
