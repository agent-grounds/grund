//! Test module: the concern records themselves (§AR-config.1.3) — the three
//! states a row can be in, the two views over the rows, and the façade built
//! from the records (§AR-config.5).

use std::path::PathBuf;

use super::v1::default_project;
use super::*;

fn place(extent: Extent) -> Place {
    Place {
        extent,
        scanned: true,
    }
}

fn prose() -> Kind {
    Kind {
        id_format: None,
        form: Form::Prose,
        index: KindIndex::Default,
        origin: Origin::Local,
    }
}

/// §AR-config.1.3: a non-citable place, a homeless citable kind and the
/// complement are three distinct rows, each in the one shape that says which.
#[test]
fn the_three_row_states_are_distinct_by_construction() {
    let place_only = Row {
        name: "integration".into(),
        places: vec![place(Extent::Folder("tests/integration".into()))],
        kind: None,
    };
    let homeless = Row {
        name: "RULE".into(),
        places: Vec::new(),
        kind: Some(prose()),
    };
    let complement = Row {
        name: "src".into(),
        places: vec![place(Extent::Complement)],
        kind: None,
    };
    let mut project = default_project(false);
    project.schema.rows = vec![place_only, homeless, complement];

    let places: Vec<_> = project.schema.places().map(|(name, _)| name).collect();
    let kinds: Vec<_> = project.schema.kinds().map(|(name, _)| name).collect();
    assert_eq!(places, ["integration", "src"]);
    assert_eq!(kinds, ["RULE"]);
    assert_eq!(project.schema.complement_name(), "src");
    assert!(project.schema.rows[2].is_complement());
    assert!(!project.schema.rows[0].is_complement());

    // The façade's v1 rows keep the three apart the way v1 spells them.
    let rows = project.kind_configs();
    let shape = |row: &KindConfig| (row.citable, row.folder.is_some(), row.file.is_some());
    assert_eq!(shape(&rows[0]), (false, true, false));
    assert_eq!(shape(&rows[1]), (true, false, false));
    assert_eq!(shape(&rows[2]), (false, false, false));
}

/// §AR-config.1.3: with no `[[kinds]]`, `places()` is the nine built-in homes
/// and `kinds()` the seven that declare IDs — the two test places cite and are
/// never cited (§FS-config.3.4).
#[test]
fn the_built_in_rows_have_nine_places_and_seven_kinds() {
    let project = default_project(false);
    let places: Vec<_> = project.schema.places().map(|(name, _)| name).collect();
    let kinds: Vec<_> = project.schema.kinds().map(|(name, _)| name).collect();
    assert_eq!(
        places,
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
    assert_eq!(kinds, ["GRUND", "GOAL", "FS", "AR", "DF", "DA", "RM"]);
    assert_eq!(project.schema.complement_name(), "code");
}

/// §AR-config.5: the defaults' façade is the records' — every run fact from
/// `Run::at`, every project fact from the v1 default project.
#[test]
fn the_default_facade_is_built_from_its_records() {
    let root = PathBuf::from("/repo");
    let config = Config::default_for(root.clone());
    assert_eq!(config.project(), &default_project(false));
    assert_eq!(config.run().root, root);
    assert_eq!(config.cli_base, root);
    assert_eq!(config.kinds, default_project(false).kind_configs());
    assert!(!config.grounding_units);
    assert!(config.classify_citation_sources);

    let legacy = Config::default_for_existing_config(root);
    let fs = legacy.kinds.iter().find(|kind| kind.kind == "FS").unwrap();
    assert_eq!(fs.folder.as_deref(), Some("docs/functional-spec"));
    assert_eq!(fs.file, None);
}

/// §AR-config.5: a per-run setter records the fact on the `Run` and shows it on
/// the façade, and `--require-grounding` is on over a project that said off.
#[test]
fn a_run_setter_writes_the_run_and_the_facade() {
    let mut config = Config::default_for(PathBuf::from("/repo"));
    config.set_scan_full(true);
    config.set_owner_lines(vec![(3, 4)]);
    config.force_require_grounding();
    assert!(config.run().scope.full && config.scan_full);
    assert_eq!(config.run().scope.owner_lines, config.owner_lines);
    assert!(config.run().scope.require_grounding && config.require_grounding);
    assert!(!config.project().rules.grounding.require);
}
