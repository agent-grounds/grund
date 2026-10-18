//! Test module: what a version-2 refusal appends after its text
//! (§FS-config-v2.reader.1.keys, §FS-config-v2.reader.1.v1-tables). Each case
//! writes a file with one mistake on a known line and holds the whole refusal
//! from `grund.toml:<line>: ` to its end, so a hint that is missing, reordered or
//! reworded fails, and so does one appended where nothing may be. The `v1_*`
//! case holds the v1 reader's bytes, which do not move (§FS-config.5.2).

use super::*;
use crate::testing::{test_root, write};

/// The envelope and a Markdown-only source line, so a case reads no source file
/// (§FS-config-v2.schema.sources.languages).
const HEAD: &str = "grund_config_version = 2\nproject_name = \"example\"\n[schema.sources]\nlanguages = [\"markdown\"]\n";
const ROW: &str = "[schema.kinds.FS]\nfiles = [\"requirements.md\"]\n";

/// The refusal a file draws, as the CLI prints it.
fn refusal(name: &str, toml: &str) -> String {
    let root = test_root(name);
    write(&root.join("grund.toml"), toml);
    match load_config(&root) {
        Ok(_) => panic!("{name}: the file should be refused:\n{toml}"),
        Err(err) => format!("{err:#}"),
    }
}

/// The 1-based line `toml` writes `text` on.
fn line_of(toml: &str, text: &str) -> usize {
    toml.lines()
        .position(|line| line == text)
        .unwrap_or_else(|| panic!("`{text}` is not a line of:\n{toml}"))
        + 1
}

/// The refusal is `grund.toml:<line>: <message>` and ends there.
fn refused_exactly(name: &str, toml: &str, offending: &str, message: &str) {
    let err = refusal(name, toml);
    let expected = format!("grund.toml:{}: {message}", line_of(toml, offending));
    assert!(
        err.ends_with(&expected),
        "{name}: expected the refusal to end with\n  {expected}\ngot\n  {err}"
    );
}

/// One unknown key: the file, the offending line, and the whole message.
struct KeyCase {
    name: &'static str,
    toml: String,
    offending: &'static str,
    message: &'static str,
}

fn key_case(
    name: &'static str,
    before: &str,
    offending: &'static str,
    message: &'static str,
) -> KeyCase {
    KeyCase {
        name,
        toml: format!("{before}{offending}\n"),
        offending,
        message,
    }
}

/// Every v2 table that reaches the unknown-key refusal, each with the keys its
/// own point lists, in that point's order (§FS-config-v2.reader.1.keys).
fn key_cases() -> Vec<KeyCase> {
    vec![
        key_case(
            "hint_envelope",
            "grund_config_version = 2\n",
            "project = \"example\"",
            "unknown key `project` at the top level (expected grund_config_version or project_name)",
        ),
        key_case(
            "hint_schema",
            &format!("{HEAD}[schema]\n"),
            "strict = false",
            "unknown key `strict` in [schema] (expected marker, shorthand, id_format, slug_pattern, number_pattern, section_separator, or heading_depth)",
        ),
        key_case(
            "hint_sources",
            HEAD,
            "include = [\"src\"]",
            "unknown key `include` in [schema.sources] (expected languages, declarations, exclude, or respect_gitignore)",
        ),
        key_case(
            "hint_definition",
            &format!("{HEAD}[schema.sources.definitions.notes]\n"),
            "prefixes = [\"%%\"]",
            "unknown key `prefixes` in [schema.sources.definitions.notes] (expected class, comment_prefixes, doc_comments, or docstrings)",
        ),
        key_case(
            "hint_notes_lines",
            &format!("{HEAD}[schema.notes.lines]\n"),
            "max = 4",
            "unknown key `max` in [schema.notes.lines] (expected must, warn, should, or may)",
        ),
        key_case(
            "hint_notes_columns",
            &format!("{HEAD}[schema.notes.columns]\n"),
            "max = 90",
            "unknown key `max` in [schema.notes.columns] (expected must, warn, should, or may)",
        ),
        key_case(
            "hint_notes_text",
            &format!("{HEAD}[schema.notes.text]\n"),
            "warn = true",
            "unknown key `warn` in [schema.notes.text] (expected must)",
        ),
        key_case(
            "hint_notes_layout",
            &format!("{HEAD}[schema.notes.layout]\n"),
            "layout = \"any\"",
            "unknown key `layout` in [schema.notes.layout] (expected must, warn, should, or may)",
        ),
        key_case(
            "hint_leads_words",
            &format!("{HEAD}[schema.leads.words]\n"),
            "max = 60",
            "unknown key `max` in [schema.leads.words] (expected must, warn, should, or may)",
        ),
        key_case(
            "hint_row",
            &format!("{HEAD}{ROW}"),
            "folder = \"docs\"",
            "unknown key `folder` in [schema.kinds.FS] (expected files, folders, citable, scan, index, id_format, fetch, form, or closed)",
        ),
        key_case(
            "hint_field",
            &format!("{HEAD}{ROW}[schema.kinds.FS.fields.terms]\n"),
            "required = true",
            "unknown key `required` in [schema.kinds.FS.fields.terms] (expected title, presence, position, after, content, or one_of)",
        ),
        key_case(
            "hint_citations",
            &format!("{HEAD}{ROW}[rules.citations]\n"),
            "must = [\"FS\"]",
            "unknown key `must` in [rules.citations] (expected default)",
        ),
        key_case(
            "hint_citation_kind",
            &format!("{HEAD}{ROW}[rules.citations.FS]\n"),
            "error = [\"FS\"]",
            "unknown key `error` in [rules.citations.FS] (expected must, warn, should, may, must-not, warn-not, should-not, or default)",
        ),
        key_case(
            "hint_project_ladder",
            &format!("{HEAD}{ROW}[rules.citations.grounding]\n"),
            "level = \"file\"",
            "unknown key `level` in [rules.citations.grounding] (expected must, warn, should, or may)",
        ),
        key_case(
            "hint_row_ladder",
            &format!("{HEAD}{ROW}[rules.citations.FS.grounding]\n"),
            "level = \"file\"",
            "unknown key `level` in [rules.citations.FS.grounding] (expected must, warn, should, or may)",
        ),
        key_case(
            "hint_resolution",
            &format!("{HEAD}{ROW}[rules.resolution]\n"),
            "\"FS\" = \"must\"",
            "unknown key `\"FS\"` in [rules.resolution] (expected a kind name)",
        ),
        key_case(
            "hint_extensions",
            &format!("{HEAD}[schema.sources.extensions]\n"),
            "md.x = \"markdown\"",
            "unknown key `md.x` in [schema.sources.extensions] (expected a suffix)",
        ),
        key_case(
            "hint_presentation",
            &format!("{HEAD}[presentation]\n"),
            "project_description = \"x\"",
            "unknown key `project_description` in [presentation] (expected description, trigger, conversation, or rules)",
        ),
        key_case(
            "hint_presentation_kind",
            &format!("{HEAD}{ROW}[presentation.kinds.FS]\n"),
            "name = \"Specs\"",
            "unknown key `name` in [presentation.kinds.FS] (expected title)",
        ),
        key_case(
            "hint_presentation_fmt",
            &format!("{HEAD}[presentation.fmt]\n"),
            "enabled = true",
            "unknown key `enabled` in [presentation.fmt] (expected anchors, links, or exclude)",
        ),
        key_case(
            "hint_workspace",
            &format!("{HEAD}[workspace]\n"),
            "member = [\"a\"]",
            "unknown key `member` in [workspace] (expected members, optional_members, or include_root)",
        ),
    ]
}

/// §FS-config-v2.reader.1.keys: an unknown key's refusal keeps its text and
/// appends the keys its table admits, in the order the table's point lists them.
#[test]
#[ignore = "red until #818 appends the admitted keys to an unknown key; implement removes this"]
fn v2_an_unknown_key_names_the_keys_its_table_admits() {
    for case in key_cases() {
        refused_exactly(case.name, &case.toml, case.offending, case.message);
    }
}

/// §FS-config-v2.reader.1.keys: a parent written as a header of its own admits
/// no keys and says it holds only tables.
#[test]
#[ignore = "red until #818 appends the admitted keys to an unknown key; implement removes this"]
fn v2_an_unknown_key_in_a_parent_says_it_holds_only_tables() {
    for parent in [
        "schema.notes",
        "schema.leads",
        "schema.kinds",
        "schema.kinds.FS.fields",
        "schema.sources.definitions",
        "rules",
        "presentation.kinds",
    ] {
        let toml = format!("{HEAD}{ROW}[{parent}]\nbogus = 1\n");
        let name = format!("hint_parent_{}", parent.replace('.', "_"));
        let message = format!("unknown key `bogus` in [{parent}] (it holds only tables)");
        refused_exactly(&name, &toml, "bogus = 1", &message);
    }
}

/// The keys an `(expected …)` hint names, read back out of a refusal.
fn hinted_keys(name: &str, err: &str) -> Vec<String> {
    let Some((_, hint)) = err.rsplit_once(" (expected ") else {
        panic!("{name}: the refusal names no admitted key: {err}");
    };
    let hint = hint.strip_suffix(')').unwrap_or(hint);
    hint.replace(", or ", ", ")
        .replace(" or ", ", ")
        .split(", ")
        .map(str::to_string)
        .collect()
}

/// §FS-config-v2.reader.1.keys: the hint is the table's admitted keys and
/// nothing else, so each key it names is one the table reads: written there, it
/// may be refused for its value, never as an unknown key.
#[test]
#[ignore = "red until #818 appends the admitted keys to an unknown key; implement removes this"]
fn v2_every_key_a_hint_names_is_one_its_table_reads() {
    for case in key_cases() {
        if matches!(case.name, "hint_resolution" | "hint_extensions") {
            continue; // A name, not a closed key list.
        }
        let err = refusal(case.name, &case.toml);
        for key in hinted_keys(case.name, &err) {
            let toml = case.toml.replace(case.offending, &format!("{key} = \"x\""));
            let name = format!("{}_{}", case.name, key.replace('-', "_"));
            let root = test_root(&name);
            write(&root.join("grund.toml"), &toml);
            if let Err(err) = load_config(&root) {
                let err = format!("{err:#}");
                assert!(
                    !err.contains(&format!("unknown key `{key}`")),
                    "{name}: the hint names `{key}`, which its table refuses as unknown: {err}"
                );
            }
        }
    }
}

/// §FS-config-v2.reader.1.v1-tables: a v1 table is refused at its header and
/// names, as one set, the v2 tables its settings are written in.
#[test]
#[ignore = "red until #818 names a v1 table's v2 tables at its header; implement removes this"]
fn v2_a_v1_table_names_the_v2_tables_its_settings_are_written_in() {
    let homes = [
        (
            "reference",
            "[schema], [schema.notes.*], [schema.leads.words], [rules.citations.grounding], [presentation]",
        ),
        ("id", "[schema]"),
        (
            "scan",
            "[schema.sources], [schema.sources.extensions], [schema.sources.definitions.*], [schema.kinds.code]",
        ),
        ("citations", "[rules.citations]"),
        ("citations.FS", "[rules.citations.FS]"),
        ("fmt", "[presentation.fmt]"),
        ("fmt.cross_refs", "[presentation.fmt]"),
        ("output", "run flags, such as --format and --path-base"),
    ];
    for (table, home) in homes {
        let toml = format!("{HEAD}[{table}]\nkey = 1\n");
        let name = format!("hint_v1_{}", table.replace('.', "_"));
        let message = format!("unknown config section `{table}` (a v1 table; in v2: {home})");
        refused_exactly(&name, &toml, &format!("[{table}]"), &message);
    }
    let toml = format!("{HEAD}[[kinds]]\nkind = \"FS\"\n");
    refused_exactly(
        "hint_v1_kinds",
        &toml,
        "[[kinds]]",
        "unknown config section `[[kinds]]` (v2 writes no arrays of tables) (a v1 table; in v2: [schema.kinds.<NAME>])",
    );
}

/// §FS-config-v2.reader.1.v1-tables: an unknown table that is no v1 spelling
/// has no home to name, so its refusal keeps reader.1's text with nothing after.
#[test]
fn v2_an_unknown_table_that_is_no_v1_spelling_appends_nothing() {
    for table in ["schema.leads.lines", "project", "fmt.other"] {
        let toml = format!("{HEAD}[{table}]\n");
        let name = format!("hint_none_{}", table.replace('.', "_"));
        let message = format!("unknown config section `{table}`");
        refused_exactly(&name, &toml, &format!("[{table}]"), &message);
    }
    let toml = format!("{HEAD}[[rows]]\n");
    refused_exactly(
        "hint_none_rows",
        &toml,
        "[[rows]]",
        "unknown config section `[[rows]]` (v2 writes no arrays of tables)",
    );
}

/// §FS-config.5.2: the v1 reader writes the same unknown-section text and
/// appends nothing, v2's tables included, and its unknown key keeps its bytes.
#[test]
fn v1_an_unknown_section_keeps_its_bytes() {
    for table in [
        "schema",
        "schema.sources",
        "rules",
        "presentation",
        "project",
    ] {
        let toml = format!("grund_config_version = 1\n[{table}]\n");
        let name = format!("hint_v1_reader_{}", table.replace('.', "_"));
        let message = format!("unknown config section `{table}`");
        refused_exactly(&name, &toml, &format!("[{table}]"), &message);
    }
    refused_exactly(
        "hint_v1_reader_key",
        "grund_config_version = 1\n[reference]\nstrcit = true\n",
        "strcit = true",
        "unknown config key `strcit`",
    );
}
