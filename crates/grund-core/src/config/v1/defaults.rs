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
    Presentation, Project, Rules, Schema, Sources,
};
use crate::config::record::{DEFAULT_GROUNDING_LEVEL, ShorthandPolicy};
use crate::config::rows::{Extent, Form, Kind, Nesting, Origin, Place, Row};

const DEFAULT_ID_FORMAT: &str = "{kind}-{number}-{slug}";
const DEFAULT_SECTION_SEPARATOR: &str = ".";
const DEFAULT_NUMBER_PATTERN: &str = r"\d+";
const DEFAULT_SLUG_PATTERN: &str = r"[a-z0-9][a-z0-9-]*";
/// The `[scan]` defaults (§FS-config.3.5): what a repo with no `include` walks,
/// the extensions it reads, and the comment prefixes a declaration or an
/// inline note may sit behind. The compiled grammar takes `comment_prefixes`
/// as an argument and holds no opinion about what it should be
/// (§AR-system.2.1).
const DEFAULT_INCLUDE: &[&str] = &["requirements.md", "docs", "e2e", "src"];
const DEFAULT_SCAN_EXTENSIONS: &[&str] = &[
    "md", "rs", "go", "java", "kt", "ts", "tsx", "js", "py", "c", "cpp", "swift", "scala", "rb",
    "php", "cs", "lisp", "scm", "clj", "sql", "hs", "lhs", "lua", "ada", "adb", "ads",
];
const DEFAULT_COMMENT_PREFIXES: &[&str] = &["//", "#", ";", "--", "*", "/*"];
/// §AR-config.3.2 `scan exclusions`.
const DEFAULT_EXCLUDE: &[&str] = &["target", "node_modules", ".git", "dist", "build", ".venv"];
/// §AR-config.3.2 `legacy FS home`: the folder a `grund.toml` written before
/// the `requirements.md` default still gets for `FS`.
const LEGACY_FS_FOLDER: &str = "docs/functional-spec";

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| value.to_string()).collect()
}

/// The project a v1 file means before any key: the built-in defaults of
/// §FS-config.3 (§GOAL-zero-config). `existing_config` is §AR-config.3.2's
/// `legacy FS home` — an already-authored `grund.toml` that predates the
/// `requirements.md` default and declares no `[[kinds]]` keeps `FS` in its
/// old folder until it opts into the file.
pub(in crate::config) fn default_project(existing_config: bool) -> Project {
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
                marker: "§".to_string(),
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
                exclude: strings(DEFAULT_EXCLUDE),
                extensions: strings(DEFAULT_SCAN_EXTENSIONS),
                comment_prefixes: strings(DEFAULT_COMMENT_PREFIXES),
                docstring_python: true,
                respect_gitignore: true,
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
            rows: built_in_rows(existing_config),
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
        },
    }
}

/// §AR-config.3.2 `built-in kinds`: the canonical rows of §FS-config.3.4.4, with
/// their homes and citability. A declared `[[kinds]]` table replaces them all.
fn built_in_rows(existing_config: bool) -> Vec<Row> {
    DEFAULT_KINDS
        .iter()
        .map(|name| {
            let extent = match (default_kind_folder(name), default_kind_file(name)) {
                _ if existing_config && *name == "FS" => {
                    Some(Extent::Folder(LEGACY_FS_FOLDER.into()))
                }
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
                    })
                    .collect(),
                kind: default_kind_citable(name).then(|| Kind {
                    id_format: None,
                    form: Form::Prose,
                    index: default_kind_index(name),
                    origin: Origin::Local,
                }),
            }
        })
        .collect()
}
