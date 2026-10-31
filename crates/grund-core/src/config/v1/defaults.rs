//! What a version-1 project means before its file says anything
//! (§AR-config.3.2): the built-in rows and every scalar default, applied here
//! and nowhere else, so each is attributable to v1 (§AR-config.2 rule 2).
//! `grund init` writes these same values out as a teaching surface
//! (§FS-init.2.4.3).

use std::collections::BTreeMap;

use super::kind_defaults::{
    DEFAULT_KINDS, default_kind_citable, default_kind_file, default_kind_folder,
    default_kind_index, default_kind_title,
};
use crate::config::project::{
    CitationSyntax, FmtPresentation, Grounding, IdGrammar, Members, NoteStyle, OutputPresentation,
    Presentation, Project, Rules, Schema, Sources, Strength,
};
use crate::config::record::{DEFAULT_GROUNDING_LEVEL, ShorthandPolicy};
use crate::config::rows::{Extent, Form, Kind, Nesting, Origin, Place, Row};
use crate::grammar::classified_extensions;

const DEFAULT_ID_FORMAT: &str = "{kind}-{number}-{slug}";
/// The outer separator of a tree with no config at all (§FS-config.3).
pub(crate) const DEFAULT_SECTION_SEPARATOR: &str = ".";
/// Its citation marker. A coordinate glob is read under both before any config
/// is discovered (§FS-coordinate-globs.errors.shape).
pub(crate) const DEFAULT_MARKER: &str = "§";
const DEFAULT_NUMBER_PATTERN: &str = r"\d+";
const DEFAULT_SLUG_PATTERN: &str = r"[a-z0-9][a-z0-9-]*";
/// The `[scan]` defaults (§FS-config.3.5): what a repo with no `include` walks,
/// and the comment prefixes a declaration or an inline note may sit behind. The
/// compiled grammar takes `comment_prefixes` as an argument and holds no opinion
/// about what it should be (§AR-system.2.1). The extensions it reads are not
/// listed here: see [`default_scan_extensions`]. `requirements.md` stays in
/// `include` although no built-in kind homes there since grund 0.19.0: dropping
/// it would stop reading a file in a project that changed nothing, a verdict no
/// release warned about (§FS-config.3.5). Do not "clean it up".
const DEFAULT_INCLUDE: &[&str] = &["requirements.md", "docs", "e2e", "src"];
const DEFAULT_COMMENT_PREFIXES: &[&str] = &["//", "#", ";", "--", "*", "/*"];
/// §AR-config.3.2 `scan exclusions`.
const DEFAULT_EXCLUDE: &[&str] = &["target", "node_modules", ".git", "dist", "build", ".venv"];

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| value.to_string()).collect()
}

/// The v1 `extensions` default: `md`, then every extension grund's built-in
/// language table names, in table order (§FS-scan.scope.extensions.default).
/// Derived rather than listed, so a language the table gains is read by default
/// and the two can never disagree (§AR-system.principle).
fn default_scan_extensions() -> Vec<String> {
    std::iter::once("md")
        .chain(classified_extensions())
        .map(str::to_string)
        .collect()
}

/// The project a v1 file means before any key: the built-in defaults of
/// §FS-config.3 (§GOAL-zero-config). A tree with no config and a `grund.toml`
/// that omits `[[kinds]]` mean the same project (§FS-config.3.4.4).
pub(in crate::config) fn default_project() -> Project {
    Project {
        name: None,
        name_source: None,
        // §AR-config.3.2 `version`: a file that omits the key is version 1.
        version: 1,
        workspace: Members {
            include_root: true,
            ..Members::default()
        },
        schema: Schema {
            citation: CitationSyntax {
                marker: DEFAULT_MARKER.to_string(),
                // §AR-config.3.2 `strict`.
                strict: true,
                shorthand: ShorthandPolicy::Canonical,
            },
            ids: IdGrammar {
                format: DEFAULT_ID_FORMAT.into(),
                section_separator: DEFAULT_SECTION_SEPARATOR.into(),
                number_pattern: DEFAULT_NUMBER_PATTERN.into(),
                slug_pattern: DEFAULT_SLUG_PATTERN.into(),
                // §AR-config.3.2 `named sections`.
                named_sections: false,
                section_heading_levels: "strict".into(),
            },
            sources: Sources {
                include: Some(strings(DEFAULT_INCLUDE)),
                include_source: None,
                exclude: strings(DEFAULT_EXCLUDE),
                extensions: default_scan_extensions(),
                comment_prefixes: strings(DEFAULT_COMMENT_PREFIXES),
                docstring_python: true,
                respect_gitignore: true,
                model: None,
            },
            notes: NoteStyle {
                inline_style: "citation-with-note".into(),
                suggested_lines: 1,
                suggested_lines_source: None,
                max_lines: 3,
                max_lines_source: None,
                max_columns: 100,
                layout: "any".into(),
                layout_check: "off".into(),
                warn_on_suggested: false,
                extra_lines: Vec::new(),
                extra_columns: Vec::new(),
            },
            leads: None,
            lead_thresholds: Vec::new(),
            rows: built_in_rows(),
            // §AR-config.3.2 `nesting`.
            nesting: Nesting::NestedPlaceFallsToComplement,
        },
        rules: Rules {
            citations: Default::default(),
            grounding: Grounding {
                require: false,
                level: DEFAULT_GROUNDING_LEVEL,
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
            kinds: DEFAULT_KINDS
                .iter()
                .filter_map(|kind| Some((kind.to_string(), default_kind_title(kind)?.to_string())))
                .collect(),
            description: None,
            conversation: None,
            trigger: "$$".to_string(),
            output: OutputPresentation {
                format: "text".into(),
                relative_paths: true,
                color: None,
            },
            rules_at_root: true,
        },
    }
}

/// §AR-config.3.2 `built-in kinds`: the canonical rows of §FS-config.3.4.4, with
/// their homes and citability. A declared `[[kinds]]` table replaces them all.
fn built_in_rows() -> Vec<Row> {
    DEFAULT_KINDS
        .iter()
        .map(|name| {
            let extent = match (default_kind_folder(name), default_kind_file(name)) {
                (Some(folder), _) => Some(Extent::Folder(folder.into())),
                (None, Some(file)) => Some(Extent::File(file.into())),
                (None, None) => None,
            };
            Row {
                name: name.to_string(),
                places: extent
                    .into_iter()
                    .map(|extent| Place {
                        extent,
                        scanned: true,
                        source: None,
                    })
                    .collect(),
                kind: default_kind_citable(name).then(|| Kind {
                    id_format: None,
                    form: Form::Prose,
                    index: default_kind_index(name),
                    origin: Origin::Local,
                    fields: Vec::new(),
                    closed: Strength::May,
                }),
            }
        })
        .collect()
}
