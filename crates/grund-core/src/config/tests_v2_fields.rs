//! Test module: a v2 kind's fields, `closed` and `form` (§FS-fields), held on
//! the records they lower into. Each refusal names the line it is located at
//! (§FS-config.4.3), and each v2 spelling of a v1 contract lowers to the same
//! `Kind` the v1 key does (§FS-fields.forms, §FS-fields.content.values).

use super::*;
use crate::testing::{test_root, write};

const HEAD: &str = "grund_config_version = 2\nproject_name = \"example\"\n[schema.sources]\nlanguages = [\"markdown\"]\n";
const AR: &str = "[schema.kinds.AR]\nfolders = [\"docs\"]\nindex = false\n";

fn load(name: &str, toml: &str) -> anyhow::Result<Config> {
    let root = test_root(name);
    write(&root.join("grund.toml"), toml);
    std::fs::create_dir_all(root.join("docs")).unwrap();
    load_config(&root)
}

/// The lowered kind of row `row`, from a file that must load.
fn kind(name: &str, toml: &str, row: &str) -> Kind {
    let config = match load(name, toml) {
        Ok(config) => config,
        Err(err) => panic!("{name}: the file should load: {err:#}"),
    };
    let project = config.project();
    let found = project.schema.rows.iter().find(|r| r.name == row);
    match found.and_then(|r| r.kind.clone()) {
        Some(kind) => kind,
        None => panic!("{name}: no citable row {row}"),
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

/// §FS-fields.declare.field: a field table with every key loads.
#[test]
fn v2_a_field_with_every_key_loads() {
    let toml = format!(
        "{HEAD}{AR}[schema.kinds.AR.fields.placement]\ntitle = \"Placement\"\n\
         presence = \"must\"\nposition = \"first\"\n\
         [schema.kinds.AR.fields.terms]\ntitle = \"Terms\"\npresence = \"warn\"\nafter = \"placement\"\n\
         [schema.kinds.AR.fields.status]\none_of = [\"Proposed\", \"Accepted\"]\npresence = \"should\"\n\
         position = \"last\"\n"
    );
    kind("v2_fields_every_key", &toml, "AR");
}

/// §FS-fields.declare.closed: `closed` takes a strength and loads.
#[test]
fn v2_closed_takes_a_strength() {
    for strength in ["must", "warn", "should", "may"] {
        let toml = format!(
            "{HEAD}[schema.kinds.AR]\nfolders = [\"docs\"]\nindex = false\nclosed = \"{strength}\"\n"
        );
        kind(&format!("v2_closed_{strength}"), &toml, "AR");
    }
}

/// §FS-fields.order.refusals: every placement refusal, at its key.
#[test]
fn v2_placement_refusals_are_located_at_their_key() {
    let f = |name: &str| format!("[schema.kinds.AR.fields.{name}]\n");
    let cases = [
        (
            "position_and_after",
            format!(
                "{HEAD}{AR}{}position = \"first\"\n{}position = \"last\"\nafter = \"placement\"\n",
                f("placement"),
                f("terms")
            ),
            12,
            "field AR.terms sets both position and after; a field takes one placement",
        ),
        (
            "two_first",
            format!(
                "{HEAD}{AR}{}position = \"first\"\n{}position = \"first\"\n",
                f("placement"),
                f("scope")
            ),
            11,
            "fields AR.placement and AR.scope are both first; only one field can be",
        ),
        (
            "two_last",
            format!(
                "{HEAD}{AR}{}position = \"last\"\n{}position = \"last\"\n",
                f("why"),
                f("scope")
            ),
            11,
            "fields AR.why and AR.scope are both last; only one field can be",
        ),
        (
            "two_after_one",
            format!(
                "{HEAD}{AR}{}position = \"first\"\n{}after = \"placement\"\n{}after = \"placement\"\n",
                f("placement"),
                f("terms"),
                f("scope")
            ),
            13,
            "fields AR.terms and AR.scope both come directly after AR.placement; chain one after the other",
        ),
        (
            "after_unknown",
            format!(
                "{HEAD}{AR}{}position = \"first\"\n{}after = \"plcement\"\n{}content = \"values\"\n",
                f("placement"),
                f("terms"),
                f("values")
            ),
            11,
            "after = \"plcement\" names no field of AR; its fields: placement, terms, values",
        ),
        (
            "after_last",
            format!(
                "{HEAD}{AR}{}position = \"last\"\n{}after = \"why\"\n",
                f("why"),
                f("scope")
            ),
            11,
            "AR.why is last; nothing can come after it",
        ),
        (
            "after_itself",
            format!("{HEAD}{AR}{}after = \"a\"\n", f("a")),
            9,
            "AR.a comes after itself",
        ),
        (
            "two_cycle",
            format!(
                "{HEAD}{AR}{}after = \"b\"\n{}after = \"a\"\n",
                f("a"),
                f("b")
            ),
            11,
            "AR.a and AR.b each come after the other; one of them needs position or no after",
        ),
        (
            "three_cycle",
            format!(
                "{HEAD}{AR}{}after = \"c\"\n{}after = \"a\"\n{}after = \"b\"\n",
                f("a"),
                f("b"),
                f("c")
            ),
            13,
            "AR.a, AR.b and AR.c come after one another in a cycle; one of them needs position or no after",
        ),
    ];
    for (name, toml, line, message) in cases {
        refused(&format!("v2_fields_{name}"), &toml, line, message);
    }
}

/// §FS-fields.content.one-of, §FS-fields.identity.match, §FS-fields.declare.form:
/// scalar, title and value-set refusals.
#[test]
fn v2_scalar_and_title_refusals_are_located_at_their_key() {
    let status = "[schema.kinds.AR.fields.status]\n";
    let cases = [
        (
            "one_of_title",
            format!("{HEAD}{AR}{status}title = \"Status\"\none_of = [\"Accepted\"]\n"),
            10,
            "field AR.status sets one_of and title; a scalar's title is its value",
        ),
        (
            "one_of_content",
            format!("{HEAD}{AR}{status}content = \"prose\"\none_of = [\"Accepted\"]\n"),
            10,
            "field AR.status sets one_of and content; a scalar's content is its title",
        ),
        (
            "one_of_empty",
            format!("{HEAD}{AR}{status}one_of = []\n"),
            9,
            "field AR.status has an empty one_of; a scalar needs at least one value",
        ),
        (
            "one_of_repeated",
            format!("{HEAD}{AR}{status}one_of = [\"Accepted\", \"Accepted\"]\n"),
            9,
            "field AR.status lists \"Accepted\" twice in one_of",
        ),
        (
            "one_of_whitespace",
            format!("{HEAD}{AR}{status}one_of = [\" Accepted\"]\n"),
            9,
            "field AR.status has one_of entry \" Accepted\", which is empty or has surrounding whitespace",
        ),
        (
            "title_empty",
            format!("{HEAD}{AR}[schema.kinds.AR.fields.terms]\ntitle = \"\"\n"),
            9,
            "field AR.terms has title \"\", which is empty or has surrounding whitespace",
        ),
        (
            "presence_unknown",
            format!("{HEAD}{AR}[schema.kinds.AR.fields.terms]\npresence = \"maybe\"\n"),
            9,
            "unknown [schema.kinds.AR.fields.terms] presence `maybe` (expected must, warn, should, or may)",
        ),
        (
            "position_unknown",
            format!("{HEAD}{AR}[schema.kinds.AR.fields.terms]\nposition = \"middle\"\n"),
            9,
            "unknown [schema.kinds.AR.fields.terms] position `middle` (expected first or last)",
        ),
        (
            "content_unknown",
            format!("{HEAD}{AR}[schema.kinds.AR.fields.terms]\ncontent = \"table\"\n"),
            9,
            "unknown [schema.kinds.AR.fields.terms] content `table` (expected prose or values)",
        ),
        (
            "form_unknown",
            format!("{HEAD}{AR}form = \"table\"\n"),
            8,
            "unknown [schema.kinds.AR] form `table` (expected prose, value, or rule)",
        ),
        (
            "field_key_unknown",
            format!("{HEAD}{AR}[schema.kinds.AR.fields.terms]\nrequired = true\n"),
            9,
            "unknown key `required` in [schema.kinds.AR.fields.terms]",
        ),
    ];
    for (name, toml, line, message) in cases {
        refused(&format!("v2_fields_{name}"), &toml, line, message);
    }
}

/// §FS-fields.forms: a kind with no authored body takes no fields and cannot be closed.
#[test]
fn v2_a_kind_with_no_body_takes_no_fields() {
    let cases = [
        (
            "value_fields",
            format!(
                "{HEAD}[schema.kinds.CONST]\nfolders = [\"docs\"]\nindex = false\nform = \"value\"\n[schema.kinds.CONST.fields.status]\npresence = \"must\"\n"
            ),
            9,
            "CONST is form = \"value\": its children are numbered components, so it takes no fields",
        ),
        (
            "value_closed",
            format!(
                "{HEAD}[schema.kinds.CONST]\nfolders = [\"docs\"]\nindex = false\nform = \"value\"\nclosed = \"must\"\n"
            ),
            9,
            "CONST is form = \"value\": its children are numbered components, so it cannot be closed",
        ),
        (
            "not_citable",
            format!(
                "{HEAD}[schema.kinds.NOTES]\nfolders = [\"docs\"]\ncitable = false\n[schema.kinds.NOTES.fields.status]\npresence = \"must\"\n"
            ),
            8,
            "NOTES is not citable: it declares nothing, so it takes no fields",
        ),
        (
            "fetched",
            format!(
                "{HEAD}[schema.kinds.TICKET]\nid_format = \"{{kind}}-{{number}}\"\nfetch = \"scripts/fetch\"\nclosed = \"must\"\n"
            ),
            8,
            "TICKET is fetched: its declarations are a snapshot, so it cannot be closed",
        ),
    ];
    for (name, toml, line, message) in cases {
        refused(&format!("v2_fields_{name}"), &toml, line, message);
    }
}

/// §FS-fields.content.values: a second values field is v1's one-chapter refusal.
#[test]
fn v2_a_second_values_field_is_refused() {
    let toml = format!(
        "{HEAD}{AR}[schema.kinds.AR.fields.values]\ncontent = \"values\"\n\
         [schema.kinds.AR.fields.limits]\ncontent = \"values\"\n"
    );
    let err = match load("v2_fields_two_values", &toml) {
        Ok(_) => panic!("two values fields should be refused"),
        Err(err) => format!("{err:#}"),
    };
    assert!(
        err.contains("grund.toml:11:"),
        "expected line 11, got: {err}"
    );
}

const V1_HEAD: &str = "grund_config_version = 1\nproject_name = \"example\"\n\
                       [id]\nformat = \"{kind}-{slug}\"\nnamed_sections = true\n";

/// §FS-fields.content.values, §FS-fields.v1: a v2 values field and v1's
/// `value_chapter` lower to the same `Kind`.
#[test]
fn v2_a_values_field_lowers_to_the_v1_value_chapter() {
    let v1 = format!(
        "{V1_HEAD}[[kinds]]\nkind = \"AR\"\nfolder = \"docs\"\nindex = false\nvalue_chapter = \"values\"\n"
    );
    let v2 = format!("{HEAD}{AR}[schema.kinds.AR.fields.values]\ncontent = \"values\"\n");
    assert_eq!(
        kind("v2_fields_values_v2", &v2, "AR"),
        kind("v2_fields_values_v1", &v1, "AR")
    );
}

/// §FS-fields.record, §FS-fields.forms: both readers lower into one record, and
/// `form = "rule"` with a values field is v1's `rules = true`
/// with `value_chapter`, and `form = "value"` is v1's `values = true`.
#[test]
fn v2_forms_lower_to_the_v1_keys() {
    let v1_rule = format!(
        "{V1_HEAD}[[kinds]]\nkind = \"RULE\"\nfolder = \"docs\"\nindex = false\nrules = true\nvalue_chapter = \"values\"\n"
    );
    let v2_rule = format!(
        "{HEAD}[schema.kinds.RULE]\nfolders = [\"docs\"]\nindex = false\nform = \"rule\"\n\
         [schema.kinds.RULE.fields.values]\ncontent = \"values\"\n"
    );
    assert_eq!(
        kind("v2_fields_rule_v2", &v2_rule, "RULE"),
        kind("v2_fields_rule_v1", &v1_rule, "RULE")
    );
    let v1_value = format!(
        "{V1_HEAD}[[kinds]]\nkind = \"CONST\"\nfolder = \"docs\"\nindex = false\nvalues = true\n"
    );
    let v2_value = format!(
        "{HEAD}[schema.kinds.CONST]\nfolders = [\"docs\"]\nindex = false\nform = \"value\"\n"
    );
    assert_eq!(
        kind("v2_fields_value_v2", &v2_value, "CONST"),
        kind("v2_fields_value_v1", &v1_value, "CONST")
    );
}
