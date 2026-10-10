//! The v2 epoch (§FS-config-v2.defaults.1): what a v2 project means before its
//! file says anything, applied here and nowhere else. No later binary changes
//! these; a different default is a different version (§FS-config.5.2). Where
//! v2 keeps a v1 value — the marker, `section_separator`, `number_pattern`, the
//! note budgets — it is written out again rather than read from `v1/`, so a v1
//! default can never move a v2 file.

use std::collections::BTreeMap;

use crate::config::project::{
    CitationSyntax, FmtPresentation, Grounding, IdGrammar, Members, NoteStyle, OutputPresentation,
    Presentation, Project, Rules, Schema, Sources,
};
use crate::config::record::ShorthandPolicy;
use crate::config::rows::Nesting;

/// The scan exclusions every version applies; `[schema.sources] exclude` is
/// #456's and refused until it lands (§FS-config-v2.rollout).
const EXCLUDE: &[&str] = &["target", "node_modules", ".git", "dist", "build", ".venv"];
/// The comment prefixes a note may sit behind. v2 executes Markdown alone
/// (§FS-config-v2.defaults.2), so they reach only a fenced example's reading.
const COMMENT_PREFIXES: &[&str] = &["//", "#", ";", "--", "*", "/*"];

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| value.to_string()).collect()
}

/// The project a v2 file means before any key (§FS-config-v2.defaults.1). It
/// has no rows: v1's built-in kind list belongs to v1. The scan and the
/// extensions follow from the rows and `languages`, in `finish.rs`.
pub(super) fn default_project() -> Project {
    Project {
        name: None,
        name_source: None,
        version: 2,
        workspace: Members {
            include_root: true,
            ..Members::default()
        },
        schema: Schema {
            // §FS-config-v2.defaults.1: citations are marked, with no opt-out.
            citation: CitationSyntax {
                marker: "§".to_string(),
                strict: true,
                shorthand: ShorthandPolicy::Canonical,
            },
            ids: IdGrammar {
                format: "{kind}-{slug}".into(),
                section_separator: ".".into(),
                number_pattern: r"\d+".into(),
                slug_pattern: "[a-z][a-z0-9-]*".into(),
                // §FS-config-v2.defaults.1: always enabled, with no switch.
                named_sections: true,
                section_heading_levels: "strict".into(),
            },
            sources: Sources {
                include: None,
                exclude: strings(EXCLUDE),
                extensions: vec!["md".to_string()],
                comment_prefixes: strings(COMMENT_PREFIXES),
                docstring_python: false,
                respect_gitignore: true,
            },
            notes: NoteStyle {
                inline_style: "citation-with-note".into(),
                suggested_lines: 1,
                suggested_lines_source: None,
                max_lines: MAX_LINES,
                max_lines_source: None,
                max_columns: MAX_COLUMNS,
                layout: "any".into(),
                layout_check: "off".into(),
                warn_on_suggested: false,
                extra_lines: Vec::new(),
                extra_columns: Vec::new(),
            },
            leads: None,
            lead_thresholds: Vec::new(),
            rows: Vec::new(),
            nesting: Nesting::NestedPlaceFallsToComplement,
        },
        rules: Rules {
            citations: Default::default(),
            grounding: Grounding {
                require: false,
                level: 1,
                level_source: None,
                kinds: BTreeMap::new(),
                ladder: None,
            },
            resolution: BTreeMap::new(),
        },
        presentation: Presentation {
            fmt: FmtPresentation {
                exclude: Vec::new(),
                cross_refs_enabled: true,
                anchor_format: "github".into(),
            },
            kinds: BTreeMap::new(),
            description: None,
            conversation: None,
            trigger: "$$".to_string(),
            output: OutputPresentation {
                format: "text".into(),
                relative_paths: true,
                color: None,
            },
        },
    }
}

/// The note budgets' `must` defaults, which a softer threshold written alone
/// is ordered against (§FS-config-v2.schema.measures).
pub(super) const MAX_LINES: usize = 3;
pub(super) const MAX_COLUMNS: usize = 100;
