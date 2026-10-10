//! Test module: the version-2 reader (§FS-config-v2), held on the records it
//! lowers into rather than on one tree's reports (§FS-config-v2.mapping). Each
//! v2 case writes a file, loads it, and reads the `Project` back; each refusal
//! names the line it is located at (§FS-config.4.3). The `v1_*` cases are the
//! historical halves of §FS-config-v2.mapping, which must not move.

use super::*;
use crate::testing::{test_root, write};

/// The envelope and the Markdown-only source line every executable v2 file
/// needs while the v2 language set is refused (§FS-config-v2.defaults.2).
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

/// The refusal's text, which must name `grund.toml:<line>:` and carry `message`.
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

/// §FS-config-v2.defaults.1: an omitted key takes the v2 epoch, in the records.
#[test]
fn v2_defaults_lower_into_the_records() {
    let project = project("v2_defaults", &format!("{HEAD}{FS_GOAL}"));
    assert_eq!(project.version, 2);
    assert!(project.schema.citation.strict);
    assert_eq!(project.schema.citation.marker, "§");
    assert!(project.schema.ids.named_sections);
    assert_eq!(project.schema.ids.format, "{kind}-{slug}");
    assert_eq!(project.schema.ids.slug_pattern, "[a-z][a-z0-9-]*");
    assert_eq!(project.schema.ids.section_heading_levels, "strict");
    assert!(project.schema.sources.respect_gitignore);
}

/// §FS-config-v2.defaults.1: a v2 file has the kinds it declares and no
/// built-in ones, in the order its headers are written (§FS-config-v2.reader.4).
#[test]
fn v2_rows_are_the_declared_ones_in_authored_order() {
    let toml = format!(
        "{HEAD}[schema.kinds.GOAL]\nfiles = [\"docs/goals.md\"]\n\
         [schema.kinds.FS]\nfiles = [\"requirements.md\"]\n"
    );
    let project = project("v2_row_order", &toml);
    let names: Vec<_> = project
        .schema
        .rows
        .iter()
        .map(|row| row.name.as_str())
        .collect();
    assert_eq!(names, ["GOAL", "FS"]);
}

/// §FS-config-v2.schema.1: `heading_depth` lowers to the depth check's modes;
/// `may` is v1's `loose` (§FS-config-v2.mapping).
#[test]
fn v2_heading_depth_lowers_to_the_depth_modes() {
    for (strength, mode) in [("must", "strict"), ("warn", "warn"), ("may", "loose")] {
        let toml = format!("{HEAD}[schema]\nheading_depth = \"{strength}\"\n{FS_GOAL}");
        let project = project(&format!("v2_heading_depth_{strength}"), &toml);
        assert_eq!(
            project.schema.ids.section_heading_levels, mode,
            "{strength}"
        );
    }
}

/// §FS-config-v2.schema.1: the grammar keys mean what their v1 counterparts do.
#[test]
fn v2_schema_grammar_keys_lower() {
    let toml = format!(
        "{HEAD}[schema]\nid_format = \"{{kind}}-{{number}}-{{slug}}\"\n\
         slug_pattern = \"[a-z0-9][a-z0-9-]*\"\n{FS_GOAL}"
    );
    let project = project("v2_grammar_keys", &toml);
    assert_eq!(project.schema.ids.format, "{kind}-{number}-{slug}");
    assert_eq!(project.schema.ids.slug_pattern, "[a-z0-9][a-z0-9-]*");
}

/// §FS-config-v2.reader.2: a parent TOML creates implicitly is not an
/// explicit header, so writing it afterwards is not a repetition.
#[test]
fn v2_an_implicit_parent_is_not_a_repeated_table() {
    let toml = format!("{HEAD}[schema]\nmarker = \"§\"\n{FS_GOAL}");
    assert_eq!(project("v2_implicit_parent", &toml).version, 2);
}

/// §FS-config-v2.reader.2: an explicit header written twice is refused at the second.
#[test]
fn v2_a_repeated_table_is_refused_at_the_second() {
    let toml = format!("{HEAD}[schema]\nmarker = \"§\"\n{FS_GOAL}[schema]\n");
    let message = "repeated table `[schema]` (first written at line 5)";
    refused("v2_repeated_schema", &toml, 11, message);
}

/// §FS-config-v2.reader.1: a key written twice in one table.
#[test]
fn v2_a_duplicate_key_is_refused() {
    let toml = format!("{HEAD}[schema.kinds.FS]\nfiles = [\"a.md\"]\nfiles = [\"b.md\"]\n");
    let message = "duplicate key `files` in [schema.kinds.FS] (first written at line 6)";
    refused("v2_duplicate_key", &toml, 7, message);
}

/// §FS-config-v2.reader.1: a key at a table that does not admit it, and v1's
/// tables, are unknown in v2.
#[test]
fn v2_wrong_scopes_and_v1_tables_are_refused() {
    let toml = format!("{HEAD}[schema]\nfetch = \"scripts/fetch\"\n");
    refused(
        "v2_wrong_scope",
        &toml,
        6,
        "unknown key `fetch` in [schema]",
    );
    let toml = format!("{HEAD}[schema.kinds.FS]\nlanguages = [\"markdown\"]\n");
    refused(
        "v2_row_languages",
        &toml,
        6,
        "unknown key `languages` in [schema.kinds.FS]",
    );
    for table in [
        "output",
        "reference",
        "id",
        "scan",
        "citations",
        "fmt.cross_refs",
    ] {
        let toml = format!("{HEAD}[{table}]\n");
        let name = format!("v2_v1_table_{}", table.replace('.', "_"));
        refused(
            &name,
            &toml,
            5,
            &format!("unknown config section `{table}`"),
        );
    }
}

/// §FS-config-v2.schema.measures: the measure names the table, the strength is
/// the key, and the thresholds are v1's note budgets (§FS-config-v2.mapping).
#[test]
fn v2_note_measures_lower_to_the_budgets() {
    let toml = format!(
        "{HEAD}[schema.notes.lines]\nmust = 4\nshould = 2\n\
         [schema.notes.columns]\nmust = 90\n{FS_GOAL}"
    );
    let notes = project("v2_note_measures", &toml).schema.notes;
    assert_eq!((notes.max_lines, notes.suggested_lines), (4, 2));
    assert_eq!(notes.max_columns, 90);
    assert!(!notes.warn_on_suggested, "`should` stays guidance");

    let toml = format!("{HEAD}[schema.notes.lines]\nmust = 4\nwarn = 2\n{FS_GOAL}");
    let notes = project("v2_note_measures_warn", &toml).schema.notes;
    assert_eq!(notes.suggested_lines, 2);
    assert!(notes.warn_on_suggested, "`warn` is v1's opted-in warning");
}

/// §FS-config-v2.schema.measures: a stronger strength takes the looser threshold.
#[test]
fn v2_measure_threshold_ordering_is_refused() {
    let toml = format!("{HEAD}[schema.notes.lines]\nmust = 1\nshould = 3\n");
    let message = "threshold ordering: `must` (1) is stricter than `should` (3)";
    refused("v2_threshold_order", &toml, 6, message);
}

/// §FS-config-v2.reader.3: a strength key is never also a table.
#[test]
fn v2_a_strength_written_as_a_table_is_refused() {
    let toml = format!("{HEAD}[schema.notes.lines.must]\n");
    let message = "`must` is a strength key in [schema.notes.lines], not a table";
    refused("v2_measure_collision", &toml, 5, message);
}

/// §FS-config-v2.rules.citations.default: a positive default is refused at
/// the project and at the kind; v1 still loads one (§FS-config-v2.mapping).
#[test]
fn v2_a_positive_citation_default_is_refused() {
    let message = "`default = \"must\"` is not a v2 citation default";
    let toml = format!("{HEAD}{FS_GOAL}[rules.citations]\ndefault = \"must\"\n");
    refused("v2_project_default_must", &toml, 10, message);
    let toml = format!("{HEAD}{FS_GOAL}[rules.citations.FS]\ndefault = \"must\"\n");
    refused("v2_kind_default_must", &toml, 10, message);
    let toml = format!("{HEAD}{FS_GOAL}[rules.citations]\ndefault = \"warn-not\"\n");
    assert!(
        project("v2_default_warn_not", &toml)
            .rules
            .citations
            .declared
    );
}

/// §FS-config-v2.rules.citations: overlap between two lists is refused at the
/// second, as in v1 (§FS-config.3.9.5.1).
#[test]
fn v2_citation_overlap_is_refused() {
    let toml =
        format!("{HEAD}{FS_GOAL}[rules.citations.FS]\nmust = [\"GOAL\"]\nwarn = [\"GOAL\"]\n");
    refused("v2_citation_overlap", &toml, 11, "GOAL");
}

/// §FS-config-v2.rules.citations: any-of entries lower into the shared
/// disjunction record.
#[test]
fn v2_any_of_entries_lower_to_disjunctions() {
    let toml = format!("{HEAD}{FS_GOAL}[rules.citations.FS]\nmust = [\"GOAL|FS\"]\n");
    let rules = project("v2_any_of", &toml).rules.citations;
    let fs = &rules.per_kind["FS"];
    let kinds: Vec<_> = fs.must[0].targets.iter().map(|t| t.kind.as_str()).collect();
    assert_eq!(kinds, ["GOAL", "FS"]);
}

/// §FS-config-v2.rules.resolution: `warn` is the missing-snapshot warning v1
/// spells `resolve = "should"`; an explicit entry needs `fetch`.
#[test]
fn v2_resolution_lowers_and_requires_fetch() {
    let ticket =
        "[schema.kinds.TICKET]\nfiles = [\"docs/tickets.md\"]\nid_format = \"{kind}-{number}\"\n";
    let toml =
        format!("{HEAD}{ticket}fetch = \"scripts/fetch\"\n[rules.resolution]\nTICKET = \"warn\"\n");
    let resolution = project("v2_resolution_warn", &toml).rules.resolution;
    assert_eq!(resolution.get("TICKET"), Some(&KindResolution::Should));

    let toml = format!("{HEAD}{ticket}[rules.resolution]\nTICKET = \"must\"\n");
    let message = "`[rules.resolution] TICKET` needs `fetch` on [schema.kinds.TICKET]";
    refused("v2_resolution_no_fetch", &toml, 9, message);
}

/// §FS-config-v2.rules.grounding: a row's ladder replaces the project's whole;
/// a row without one takes the project's, and v1's effective pair is kept.
#[test]
fn v2_grounding_ladders_lower_whole() {
    let test_row = "[schema.kinds.test]\ncitable = false\nfolders = [\"tests\"]\n";
    let skill_row = "[schema.kinds.skill]\ncitable = false\nfolders = [\"skills\"]\n";
    let toml = format!(
        "{HEAD}{test_row}{skill_row}[rules.citations.grounding]\nmust = \"file\"\n\
         [rules.citations.test.grounding]\nmust = \"h2\"\n"
    );
    let config = load("v2_ladders", &toml).expect("the v2 file should load");
    let pair = |name: &str| {
        let kind = config
            .kinds
            .iter()
            .find(|kind| kind.kind == name)
            .expect("row");
        config.kind_grounding(kind)
    };
    assert_eq!(pair("test"), (true, 2));
    assert_eq!(pair("skill"), (true, 1));
}

/// §FS-config-v2.rules.grounding: a stronger rung never uses a finer unit.
#[test]
fn v2_a_stronger_finer_rung_is_refused() {
    let toml = format!("{HEAD}[rules.citations.grounding]\nmust = \"h2\"\nwarn = \"file\"\n");
    let message = "grounding ladder: `must` (h2) is finer than `warn` (file)";
    refused("v2_ladder_order", &toml, 6, message);
}

/// §FS-config-v2.presentation: the executed presentation keys lower to the
/// records v1's keys fill.
#[test]
fn v2_presentation_lowers() {
    let toml = format!(
        "{HEAD}{FS_GOAL}[presentation]\ndescription = \"One line\"\ntrigger = \"%%\"\n\
         conversation = \"link\"\n[presentation.kinds.FS]\ntitle = \"What\"\n\
         [presentation.fmt]\nexclude = [\"docs/goals.md\"]\n"
    );
    let presentation = project("v2_presentation", &toml).presentation;
    assert_eq!(presentation.description.as_deref(), Some("One line"));
    assert_eq!(presentation.trigger, "%%");
    assert_eq!(presentation.conversation.as_deref(), Some("link"));
    assert_eq!(
        presentation.kinds.get("FS").map(String::as_str),
        Some("What")
    );
    assert_eq!(presentation.fmt.exclude, ["docs/goals.md"]);
}

/// §FS-config-v2.rollout: a clause the format defines and this grund does not
/// execute is refused at its line, never accepted and ignored.
#[test]
fn v2_unexecuted_clauses_are_refused_at_their_line() {
    let not_yet = "is part of the v2 format but not supported by this grund yet";
    let cases = [
        (
            "fields",
            format!("{HEAD}{FS_GOAL}[schema.kinds.FS.fields.terms]\npresence = \"must\"\n"),
            9,
            "`[schema.kinds.FS.fields.terms]`",
        ),
        (
            "form",
            format!("{HEAD}[schema.kinds.RULE]\nfiles = [\"rules.md\"]\nform = \"rule\"\n"),
            7,
            "`[schema.kinds.RULE] form`",
        ),
        (
            "two_places",
            format!("{HEAD}[schema.kinds.FS]\nfolders = [\"a\", \"b\"]\n"),
            6,
            "`[schema.kinds.FS] folders`",
        ),
        (
            "extensions",
            format!("{HEAD}[schema.sources.extensions]\nmdx = \"markdown\"\n"),
            5,
            "`[schema.sources.extensions]`",
        ),
        (
            "languages",
            "grund_config_version = 2\n[schema.sources]\nlanguages = [\"markdown\", \"rust\"]\n"
                .to_string(),
            3,
            "`[schema.sources] languages`",
        ),
        (
            "exclude",
            format!("{HEAD}exclude = [\"vendor\"]\n"),
            5,
            "`[schema.sources] exclude`",
        ),
        (
            "anchors",
            format!("{HEAD}[presentation.fmt]\nanchors = \"gitlab\"\n"),
            6,
            "`[presentation.fmt] anchors`",
        ),
        (
            "rules",
            format!("{HEAD}[presentation]\nrules = \"home\"\n"),
            6,
            "`[presentation] rules`",
        ),
    ];
    for (name, toml, line, clause) in cases {
        refused(
            &format!("v2_rollout_{name}"),
            &toml,
            line,
            &format!("{clause} {not_yet}"),
        );
    }
}

/// §FS-config-v2.defaults.2: an omitted language set is refused at the version key.
#[test]
fn v2_an_omitted_language_set_is_refused_at_the_version_key() {
    let toml = "grund_config_version = 2\n[schema.kinds.FS]\nfiles = [\"requirements.md\"]\n";
    let message = "v2 needs an explicit [schema.sources] languages";
    refused("v2_languages_omitted", toml, 1, message);
}

/// §FS-config-v2.mapping: the v1 states v2 does not spell keep their meaning
/// in the records (§FS-config.5.2). These hold today and must keep holding.
#[test]
fn v1_historical_states_keep_their_records() {
    let toml = "[reference]\nstrict = false\n[id]\nsection_heading_levels = \"loose\"\n\
                [citations]\ndefault = \"must\"\n";
    let project = project("v1_historical_states", toml);
    assert_eq!(project.version, 1);
    assert!(!project.schema.citation.strict);
    assert!(!project.schema.ids.named_sections);
    assert_eq!(project.schema.ids.format, "{kind}-{number}-{slug}");
    assert_eq!(project.schema.ids.slug_pattern, "[a-z0-9][a-z0-9-]*");
    assert_eq!(project.schema.ids.section_heading_levels, "loose");
    assert_eq!(
        project.rules.citations.global_default,
        Some(CitationLevel::Must)
    );
}

/// §FS-config.5: an omitted version and an explicit `1` both select v1.
#[test]
fn v1_is_selected_by_an_omitted_or_explicit_one() {
    for (name, toml) in [
        ("v1_omitted", ""),
        ("v1_explicit", "grund_config_version = 1\n"),
    ] {
        assert_eq!(project(name, toml).version, 1, "{name}");
    }
}
