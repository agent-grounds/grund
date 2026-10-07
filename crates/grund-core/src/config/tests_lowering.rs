//! Test module: the default rows of the v1 lowering table (§AR-config.3.3 item
//! 4) — for every row of `v1/mapping.rs` `DEFAULTS`, an empty file and a file
//! with no `[[kinds]]` are lowered and the value the row describes is asserted
//! — and the façade held to the records it is built from (§AR-config.5). The
//! key rows are `tests_lowering_keys.rs`.

use std::path::{Path, PathBuf};

use super::tests_lowering_keys::lower;
use super::v1::default_project;
use super::v1::mapping::DEFAULTS;
use super::*;
use crate::testing::test_root;

/// §AR-config.3.3 item 4: each implicit default, asserted on an empty file and
/// on a file that writes keys but no `[[kinds]]`.
#[test]
fn every_default_means_what_its_row_says() {
    let root = test_root("lowering_defaults");
    let empty = lower(&root, "empty.toml", "");
    let no_kinds = lower(&root, "no_kinds.toml", "project_name = \"demo\"\n");
    let declared = lower(
        &root,
        "declared.toml",
        "[[kinds]]\nkind = \"E2E\"\nfolder = \"e2e/cases\"\n\n\
         [[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nfetch = \"scripts/fetch\"\n",
    );
    let row = |project: &Project, name: &str| {
        project
            .schema
            .rows
            .iter()
            .find(|row| row.name == name)
            .cloned()
            .unwrap_or_else(|| panic!("no row `{name}`"))
    };
    for (name, path) in DEFAULTS {
        match *name {
            "built-in kinds" => {
                for project in [&empty, &no_kinds] {
                    let names: Vec<_> = project.schema.rows.iter().map(|row| &row.name).collect();
                    assert_eq!(
                        names,
                        [
                            "GRUND",
                            "GOAL",
                            "FS",
                            "AR",
                            "DF",
                            "DA",
                            "e2e",
                            "integration",
                            "RM"
                        ]
                    );
                    assert!(row(project, "e2e").kind.is_none());
                    assert_eq!(project.presentation.kinds.len(), 9);
                }
            }
            "legacy FS home" => {
                let legacy = Extent::Folder("docs/functional-spec".into());
                assert_eq!(row(&no_kinds, "FS").places[0].extent, legacy);
                let zero_config = default_project(false);
                let home = Extent::File("requirements.md".into());
                assert_eq!(row(&zero_config, "FS").places[0].extent, home);
            }
            "name-keyed index" => {
                let index = |name: &str| row(&declared, name).kind.unwrap().index;
                assert_eq!(index("E2E"), KindIndex::Disabled);
                assert_eq!(index("FS"), KindIndex::Default);
            }
            "fetch resolves must" => {
                assert_eq!(
                    declared.rules.resolution.get("FS"),
                    Some(&KindResolution::Must)
                );
                assert_eq!(declared.rules.resolution.get("E2E"), None);
            }
            "kind grounding inherits" => {
                assert!(declared.rules.grounding.kinds.is_empty());
                let config = Config::default_for(PathBuf::from("/repo"));
                for kind in &config.kinds {
                    let inherited = (config.require_grounding, config.grounding_level);
                    assert_eq!(config.kind_grounding(kind), inherited);
                }
            }
            "complement name" => {
                for project in [&empty, &no_kinds, &declared] {
                    assert_eq!(project.schema.complement_name(), "code");
                    assert!(project.schema.rows.iter().all(|row| row.name != "code"));
                }
            }
            "nesting" => {
                assert_eq!(empty.schema.nesting, Nesting::NestedPlaceFallsToComplement);
            }
            "strict" => assert!(empty.schema.citation.strict && no_kinds.schema.citation.strict),
            "named sections" => assert!(!empty.schema.ids.named_sections),
            "scan exclusions" => assert_eq!(
                empty.schema.sources.exclude,
                ["target", "node_modules", ".git", "dist", "build", ".venv"]
            ),
            "version" => assert_eq!((empty.version, no_kinds.version), (1, 1)),
            other => panic!("no lowering case for the mapping.rs default `{other}` ({path})"),
        }
    }
}

/// Every façade field but the compiled grammar, which carries no `Debug`.
fn facade(config: &Config) -> Vec<String> {
    let c = config;
    vec![
        format!(
            "{:?}",
            (
                &c.root,
                &c.cli_base,
                &c.config_file,
                &c.redundant_config_file
            )
        ),
        format!(
            "{:?}",
            (
                &c.project_name,
                &c.project_name_source,
                &c.project_description
            )
        ),
        format!("{:?}", (&c.marker, &c.trigger, c.strict, c.shorthand)),
        format!(
            "{:?}",
            (c.require_grounding, c.grounding_level, c.grounding_units)
        ),
        format!(
            "{:?}",
            (&c.conversation, &c.lead_size_warning, &c.inline_style)
        ),
        format!(
            "{:?}",
            (
                c.inline_note_suggested_lines,
                c.inline_note_max_lines,
                c.inline_note_max_columns,
                &c.inline_note_layout,
                &c.inline_note_layout_check,
                c.warn_on_suggested,
            )
        ),
        format!(
            "{:?}",
            (&c.include, c.scan_full, c.scan_resolution_wide, &c.exclude)
        ),
        format!(
            "{:?}",
            (
                &c.extensions,
                &c.comment_prefixes,
                c.docstring_python,
                c.respect_gitignore
            )
        ),
        format!("{:?}", (&c.output_format, c.relative_paths)),
        format!(
            "{:?}",
            (
                &c.id_format,
                &c.section_separator,
                &c.number_pattern,
                &c.slug_pattern,
                c.named_sections,
                &c.section_heading_levels,
            )
        ),
        format!("{:?}", c.kinds),
        format!(
            "{:?}",
            (
                &c.fmt_exclude,
                c.fmt_cross_refs_enabled,
                &c.cross_ref_anchor_format
            )
        ),
        format!(
            "{:?}",
            (
                c.workspace_declared,
                &c.workspace_members,
                &c.workspace_members_source,
                &c.workspace_optional_members,
                &c.workspace_optional_members_source,
                &c.workspace_absent_optional,
                &c.workspace_section_source,
            )
        ),
        format!(
            "{:?}",
            (
                &c.workspace_scope_path,
                c.workspace_include_root,
                &c.workspace_include_root_source,
                &c.workspace_boundary_roots,
                &c.workspace_project_roots,
                c.run_warnings.len(),
            )
        ),
        format!(
            "{:?}",
            (&c.citations, c.classify_citation_sources, &c.owner_lines)
        ),
    ]
}

/// §AR-config.5: `Config::from_records` over a façade's own records rebuilds
/// that façade field for field, for the defaults and for this repository's
/// `grund.toml` — the one place a façade field gets its value.
#[test]
fn from_records_round_trips_the_default_and_this_repository() {
    let default = Config::default_for(PathBuf::from("/repo"));
    let rebuilt = Config::from_records(default.project(), default.run(), default.compiled());
    assert_eq!(facade(&default), facade(&rebuilt));
    assert_eq!(default.kinds[2].file.as_deref(), Some("requirements.md"));
    assert_eq!(default.include.as_deref().map(<[String]>::len), Some(4));

    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let config = load_config_at(&repo, &repo).expect("this repository's grund.toml loads");
    let rebuilt = Config::from_records(config.project(), config.run(), config.compiled());
    assert_eq!(facade(&config), facade(&rebuilt));
    assert_eq!(config.project().name.as_deref(), Some("grund"));
    assert!(config.require_grounding && config.named_sections);
    let e2e = config.kinds.iter().find(|kind| kind.kind == "e2e").unwrap();
    assert!(!e2e.citable && e2e.folder.as_deref() == Some("tests/e2e"));
    let places = config.project().schema.places().count();
    assert!(places > config.project().schema.kinds().count());
}
