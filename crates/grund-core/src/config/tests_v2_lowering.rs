//! Test module: what the v2 reader lowers beyond the contract cases of
//! `tests_v2_reader.rs` (§FS-config-v2) — the forms one setting is never also
//! written in, the note and lead measures, the row keys, which header owns a
//! key, and `config show`'s v2 spelling loading back to the same project
//! (§FS-config.4.2).

use super::*;
use crate::testing::{test_root, write};

const HEAD: &str = "grund_config_version = 2\nproject_name = \"example\"\n[schema.sources]\nlanguages = [\"markdown\"]\n";
const FS_GOAL: &str = "[schema.kinds.FS]\nfiles = [\"requirements.md\"]\n\
                       [schema.kinds.GOAL]\nfiles = [\"docs/goals.md\"]\n";

fn load(name: &str, toml: &str) -> anyhow::Result<Config> {
    let root = test_root(name);
    write(&root.join("grund.toml"), toml);
    load_config(&root)
}

fn project(name: &str, toml: &str) -> Project {
    match load(name, toml) {
        Ok(config) => config.project().clone(),
        Err(err) => panic!("{name}: the v2 file should load: {err:#}"),
    }
}

fn refused(name: &str, toml: &str, line: usize, message: &str) {
    let err = match load(name, toml) {
        Ok(_) => panic!("{name}: the file should be refused"),
        Err(err) => format!("{err:#}"),
    };
    let at = format!("grund.toml:{line}:");
    assert!(err.contains(&at), "{name}: expected {at}, got: {err}");
    assert!(
        err.contains(message),
        "{name}: expected `{message}`, got: {err}"
    );
}

/// §FS-config-v2.reader.3: a table is never also a key, and a strength key of a
/// ladder is never also a table.
#[test]
fn v2_one_setting_has_one_form() {
    let toml = format!("{HEAD}{FS_GOAL}[rules.citations.FS]\ngrounding = \"file\"\n");
    refused(
        "v2_form_ladder_key",
        &toml,
        10,
        "unknown key `grounding` in [rules.citations.FS]",
    );
    let toml = format!("{HEAD}[schema.notes]\nlines = 3\n");
    refused(
        "v2_form_measure_key",
        &toml,
        6,
        "unknown key `lines` in [schema.notes]",
    );
    let toml = format!("{HEAD}[rules.citations.grounding.must]\n");
    let message = "`must` is a strength key in [rules.citations.grounding], not a table";
    refused("v2_form_rung_table", &toml, 5, message);
    let toml = format!("{HEAD}[[schema.kinds]]\n");
    refused(
        "v2_form_array",
        &toml,
        5,
        "unknown config section `[[schema.kinds]]`",
    );
    let toml = format!("{HEAD}[schema.notes.lines]\nmust-not = 3\n");
    refused(
        "v2_form_measure_prohibition",
        &toml,
        6,
        "unknown key `must-not`",
    );
}

/// §FS-config-v2.schema.measures: `text`, `layout` and the lead words lower to
/// v1's style, layout pair and lead budget, with every other threshold kept.
#[test]
fn v2_note_text_layout_and_leads_lower() {
    let toml = format!(
        "{HEAD}[schema.notes.text]\nmust = false\n[schema.notes.layout]\n\
         warn = \"citation-first-colon\"\n[schema.leads.words]\nmust = 900\nwarn = 600\n{FS_GOAL}"
    );
    let schema = project("v2_text_layout_leads", &toml).schema;
    assert_eq!(schema.notes.inline_style, "citation-only");
    assert_eq!(schema.notes.layout, "citation-first-colon");
    assert_eq!(schema.notes.layout_check, "warn");
    let leads = schema.leads.expect("the `warn` lead budget");
    assert_eq!((leads.max, leads.unit), (600, PointSizeUnit::Words));
    let kept: Vec<_> = schema
        .lead_thresholds
        .iter()
        .map(|threshold| (threshold.strength, threshold.value))
        .collect();
    assert_eq!(kept, [(Strength::Must, 900)]);

    let toml = format!("{HEAD}[schema.notes.layout]\nwarn = \"any\"\nmust = \"any\"\n");
    refused(
        "v2_layout_two",
        &toml,
        7,
        "[schema.notes.layout] takes one strength",
    );
    // A softer budget written alone is ordered against the epoch's `must`.
    let toml = format!("{HEAD}[schema.notes.lines]\nshould = 5\n");
    refused(
        "v2_lines_over_default",
        &toml,
        6,
        "`must` (3, the v2 default)",
    );
}

/// §FS-config-v2.schema.places: `index`, `scan` and `citable` mean v1's row keys.
#[test]
fn v2_row_keys_lower() {
    let toml = format!(
        "{HEAD}[schema.kinds.AR]\nfolders = [\"docs/architecture\"]\nindex = \"INDEX.md\"\n\
         [schema.kinds.templates]\ncitable = false\nscan = false\nfolders = [\"templates\"]\n"
    );
    let project = project("v2_row_keys", &toml);
    let [ar, templates] = project.schema.rows.as_slice() else {
        panic!("two rows");
    };
    let kind = ar.kind.as_ref().expect("AR is citable");
    assert_eq!(kind.index, KindIndex::Named("INDEX.md".into()));
    assert!(templates.kind.is_none());
    assert!(!templates.places[0].scanned);
    assert_eq!(
        project.schema.sources.include,
        Some(vec!["docs/architecture".to_string()]),
        "the scan is the scanned places"
    );

    let toml = format!("{HEAD}[schema.kinds.GOAL]\nfiles = [\"goals.md\"]\nindex = false\n");
    refused(
        "v2_index_on_file",
        &toml,
        7,
        "`[schema.kinds.GOAL] index` needs",
    );
    let toml = format!("{HEAD}[schema.kinds.FS]\nfolders = [\"fs\"]\nscan = false\n");
    refused(
        "v2_unwalked_citable",
        &toml,
        7,
        "`[schema.kinds.FS] scan = false` needs",
    );
    let toml = format!("{HEAD}[schema.kinds.test]\ncitable = false\n");
    refused(
        "v2_complement",
        &toml,
        5,
        "`[schema.kinds.test]` is part of the v2 format",
    );
}

/// §FS-config-v2.reader.2: a key belongs to the header written above it, never
/// to an earlier row; and a place inside another row's place is #457's.
#[test]
fn v2_a_key_belongs_to_its_own_header() {
    let toml =
        format!("{HEAD}[schema.kinds.FS]\n[rules.citations.FS]\nfiles = [\"requirements.md\"]\n");
    refused(
        "v2_owner",
        &toml,
        7,
        "unknown key `files` in [rules.citations.FS]",
    );
    let toml = format!(
        "{HEAD}[schema.kinds.FS]\nfolders = [\"docs\"]\n[schema.kinds.GOAL]\nfiles = [\"docs/goals.md\"]\n"
    );
    refused(
        "v2_nested_place",
        &toml,
        8,
        "`[schema.kinds.GOAL] files` is part of the v2 format",
    );
}

/// §FS-config.4.2: a v2 project prints in v2 spelling, and what it prints loads
/// back as v2 to the same effective values; a v1 project prints none.
#[test]
fn v2_show_round_trips() {
    let toml = format!(
        "{HEAD}[workspace]\nmembers = []\n[schema]\nheading_depth = \"warn\"\n\
         [schema.notes.lines]\nmust = 4\nwarn = 2\n[schema.notes.columns]\nshould = 80\n\
         [schema.notes.layout]\nshould = \"citation-first-colon\"\n[schema.leads.words]\nwarn = 500\n\
         {FS_GOAL}[schema.kinds.test]\ncitable = false\nfolders = [\"tests\"]\n\
         [schema.kinds.TICKET]\nfiles = [\"docs/tickets.md\"]\nid_format = \"{{kind}}-{{number}}\"\n\
         fetch = \"scripts/fetch\"\n[rules.citations]\ndefault = \"warn-not\"\n\
         [rules.citations.FS]\nwarn = [\"GOAL|FS\"]\nmust-not = [\"TICKET\"]\n\
         [rules.citations.grounding]\nmust = \"file\"\nwarn = \"h2\"\n\
         [rules.citations.test.grounding]\nwarn = \"file\"\n[rules.resolution]\nTICKET = \"warn\"\n\
         [presentation]\ndescription = \"One line\"\n[presentation.kinds.FS]\ntitle = \"What\"\n"
    );
    let first = load("v2_show_first", &toml).expect("the v2 file loads");
    let shown = first.project().v2_toml().expect("a v2 project prints v2");
    let second = load("v2_show_second", &shown).unwrap_or_else(|err| panic!("{err:#}\n{shown}"));
    assert_eq!(second.project().v2_toml().as_deref(), Some(shown.as_str()));
    assert_eq!(first.kinds, second.kinds);
    let (a, b) = (first.project(), second.project());
    assert_eq!(a.schema.ids, b.schema.ids);
    assert_eq!(a.schema.rows, b.schema.rows);
    assert_eq!(a.rules.citations, b.rules.citations);
    assert_eq!(a.rules.resolution, b.rules.resolution);
    assert_eq!(a.presentation, b.presentation);
    let pair = |config: &Config, name: &str| {
        let kind = config
            .kinds
            .iter()
            .find(|kind| kind.kind == name)
            .expect("row");
        (
            config.kind_grounding(kind),
            config.soft_grounding_rungs(name),
        )
    };
    assert_eq!(pair(&first, "test"), pair(&second, "test"));
    assert_eq!(pair(&first, "FS"), pair(&second, "FS"));

    let v1 = load("v1_show", "[reference]\nstrict = true\n").expect("v1 loads");
    assert_eq!(
        v1.project().v2_toml(),
        None,
        "a v1 config keeps its own bytes"
    );
}
