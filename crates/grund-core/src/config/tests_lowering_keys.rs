//! Test module: the key rows of the v1 lowering table (§AR-config.3.3 item 4).
//! For every row of `v1/mapping.rs` `KEYS`, a file that writes the key at a
//! non-default value is lowered beside one that does not, and only the field the
//! row names may differ. The default rows and the façade are
//! `tests_lowering.rs`.

use std::collections::BTreeMap;
use std::path::Path;

use super::v1::mapping::KEYS;
use super::v1::read;
use super::*;
use crate::testing::{test_root, write};

const FS_ROW: &str = "[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\n";

/// Every field of `project` as `path -> value`, at the depth the table names
/// fields. A location is folded into the value it locates, so moving a key's
/// line changes that key's field and nothing beside it.
fn fields(project: &Project) -> BTreeMap<&'static str, String> {
    let schema = &project.schema;
    let notes = &schema.notes;
    let sources = &schema.sources;
    let ids = &schema.ids;
    let grounding = &project.rules.grounding;
    let presentation = &project.presentation;
    let members = &project.workspace;
    let at = |value: &dyn std::fmt::Debug, source: &Option<ConfigLocation>| {
        format!(
            "{value:?} @ {:?}",
            source.as_ref().map(|source| source.line)
        )
    };
    BTreeMap::from([
        ("version", format!("{:?}", project.version)),
        ("name", at(&project.name, &project.name_source)),
        (
            "workspace.declared",
            at(&members.declared, &members.section_source),
        ),
        (
            "workspace.members",
            at(&members.members, &members.members_source),
        ),
        (
            "workspace.optional_members",
            at(&members.optional_members, &members.optional_members_source),
        ),
        (
            "workspace.include_root",
            at(&members.include_root, &members.include_root_source),
        ),
        (
            "schema.citation.marker",
            format!("{:?}", schema.citation.marker),
        ),
        (
            "schema.citation.strict",
            format!("{:?}", schema.citation.strict),
        ),
        (
            "schema.citation.shorthand",
            format!("{:?}", schema.citation.shorthand),
        ),
        ("schema.ids.format", format!("{:?}", ids.format)),
        (
            "schema.ids.section_separator",
            format!("{:?}", ids.section_separator),
        ),
        (
            "schema.ids.number_pattern",
            format!("{:?}", ids.number_pattern),
        ),
        ("schema.ids.slug_pattern", format!("{:?}", ids.slug_pattern)),
        (
            "schema.ids.named_sections",
            format!("{:?}", ids.named_sections),
        ),
        (
            "schema.ids.section_heading_levels",
            format!("{:?}", ids.section_heading_levels),
        ),
        ("schema.sources.include", format!("{:?}", sources.include)),
        ("schema.sources.exclude", format!("{:?}", sources.exclude)),
        (
            "schema.sources.extensions",
            format!("{:?}", sources.extensions),
        ),
        (
            "schema.sources.comment_prefixes",
            format!("{:?}", sources.comment_prefixes),
        ),
        (
            "schema.sources.docstring_python",
            format!("{:?}", sources.docstring_python),
        ),
        (
            "schema.sources.respect_gitignore",
            format!("{:?}", sources.respect_gitignore),
        ),
        (
            "schema.notes.inline_style",
            format!("{:?}", notes.inline_style),
        ),
        (
            "schema.notes.suggested_lines",
            at(&notes.suggested_lines, &notes.suggested_lines_source),
        ),
        (
            "schema.notes.max_lines",
            at(&notes.max_lines, &notes.max_lines_source),
        ),
        (
            "schema.notes.max_columns",
            format!("{:?}", notes.max_columns),
        ),
        ("schema.notes.layout", format!("{:?}", notes.layout)),
        (
            "schema.notes.layout_check",
            format!("{:?}", notes.layout_check),
        ),
        (
            "schema.notes.warn_on_suggested",
            format!("{:?}", notes.warn_on_suggested),
        ),
        ("schema.leads", format!("{:?}", schema.leads)),
        ("schema.rows", format!("{:?}", schema.rows)),
        ("schema.nesting", format!("{:?}", schema.nesting)),
        ("rules.citations", format!("{:?}", project.rules.citations)),
        (
            "rules.grounding.require",
            format!("{:?}", grounding.require),
        ),
        (
            "rules.grounding.level",
            at(&grounding.level, &grounding.level_source),
        ),
        ("rules.grounding.kinds", format!("{:?}", grounding.kinds)),
        (
            "rules.resolution",
            format!("{:?}", project.rules.resolution),
        ),
        (
            "presentation.fmt.exclude",
            format!("{:?}", presentation.fmt.exclude),
        ),
        (
            "presentation.fmt.cross_refs_enabled",
            format!("{:?}", presentation.fmt.cross_refs_enabled),
        ),
        (
            "presentation.fmt.anchor_format",
            format!("{:?}", presentation.fmt.anchor_format),
        ),
        ("presentation.kinds", format!("{:?}", presentation.kinds)),
        (
            "presentation.description",
            format!("{:?}", presentation.description),
        ),
        (
            "presentation.conversation",
            format!("{:?}", presentation.conversation),
        ),
        (
            "presentation.trigger",
            format!("{:?}", presentation.trigger),
        ),
        (
            "presentation.output.format",
            format!("{:?}", presentation.output.format),
        ),
        (
            "presentation.output.relative_paths",
            format!("{:?}", presentation.output.relative_paths),
        ),
        (
            "presentation.output.color",
            format!("{:?}", presentation.output.color),
        ),
    ])
}

/// The fixture for one key row: a file without the key, the same file with it,
/// and the directories either needs to exist (a value home is read off disk).
fn key_case(name: &str) -> (String, String, &'static [&'static str]) {
    let plain = |base: &str, line: &str| (base.to_string(), format!("{base}{line}\n"), &[][..]);
    let kinds = |base: &str, line: &str| plain(&format!("{FS_ROW}{base}"), line);
    match name {
        "grund_config_version" => plain("", "grund_config_version = 1"),
        "project_name" => plain("", "project_name = \"demo\""),
        "project_description" => plain("", "project_description = \"A demo.\""),
        "[reference] conversation" => plain("[reference]\n", "conversation = \"link\""),
        "[reference] grounding_level" => plain(
            "[reference]\nrequire_grounding = true\n",
            "grounding_level = 2",
        ),
        "[reference] inline_note_layout" => plain(
            "[reference]\n",
            "inline_note_layout = \"citation-first-colon\"",
        ),
        "[reference] inline_note_layout_check" => {
            plain("[reference]\n", "inline_note_layout_check = \"warn\"")
        }
        "[reference] inline_note_max_columns" => {
            plain("[reference]\n", "inline_note_max_columns = 80")
        }
        "[reference] inline_note_max_lines" => plain("[reference]\n", "inline_note_max_lines = 5"),
        "[reference] inline_note_suggested_lines" => {
            plain("[reference]\n", "inline_note_suggested_lines = 2")
        }
        "[reference] inline_style" => plain("[reference]\n", "inline_style = \"citation-only\""),
        "[reference] lead_size_warning" => plain(
            "[reference]\n",
            "lead_size_warning = { max = 600, unit = \"words\" }",
        ),
        "[reference] marker" => plain("[reference]\n", "marker = \"@\""),
        "[reference] require_grounding" => plain("[reference]\n", "require_grounding = true"),
        "[reference] shorthand" => plain("[reference]\n", "shorthand = \"accepted\""),
        "[reference] strict" => plain("[reference]\n", "strict = false"),
        "[reference] trigger" => plain("[reference]\n", "trigger = \"%%\""),
        "[reference] warn_on_suggested" => plain("[reference]\n", "warn_on_suggested = true"),
        "[id] format" => plain("[id]\n", "format = \"{kind}-{slug}\""),
        "[id] named_sections" => plain("[id]\n", "named_sections = true"),
        "[id] number_pattern" => plain("[id]\n", "number_pattern = \"[0-9]+\""),
        "[id] section_heading_levels" => plain("[id]\n", "section_heading_levels = \"loose\""),
        "[id] section_separator" => plain("[id]\n", "section_separator = \"_\""),
        "[id] slug_pattern" => plain("[id]\n", "slug_pattern = \"[a-z]+\""),
        "[[kinds]] citable" => kinds("", "citable = false"),
        "[[kinds]] fetch" => (
            format!("{FS_ROW}fetch = \"scripts/a\"\n"),
            format!("{FS_ROW}fetch = \"scripts/b\"\n"),
            &[],
        ),
        "[[kinds]] file" => (
            "[[kinds]]\nkind = \"FS\"\nfile = \"a.md\"\n".to_string(),
            "[[kinds]]\nkind = \"FS\"\nfile = \"b.md\"\n".to_string(),
            &[],
        ),
        "[[kinds]] folder" => (
            FS_ROW.to_string(),
            "[[kinds]]\nkind = \"FS\"\nfolder = \"specs\"\n".to_string(),
            &[],
        ),
        "[[kinds]] format" => kinds("", "format = \"{kind}-{slug}\""),
        "[[kinds]] grounding_level" => kinds("require_grounding = true\n", "grounding_level = 2"),
        "[[kinds]] index" => kinds("", "index = false"),
        "[[kinds]] kind" => (
            FS_ROW.to_string(),
            "[[kinds]]\nkind = \"AR\"\nfolder = \"docs\"\n".to_string(),
            &[],
        ),
        "[[kinds]] require_grounding" => kinds("", "require_grounding = true"),
        "[[kinds]] resolve" => kinds("fetch = \"scripts/a\"\n", "resolve = \"should\""),
        "[[kinds]] rules" => kinds("", "rules = true"),
        "[[kinds]] scan" => {
            let place = "[[kinds]]\nkind = \"notes\"\nfolder = \"notes\"\ncitable = false\n";
            plain(place, "scan = false")
        }
        "[[kinds]] title" => kinds("", "title = \"Specs\""),
        "[[kinds]] value_chapter" => {
            let (base, variant, _) = plain(
                &format!("[id]\nnamed_sections = true\n\n{FS_ROW}"),
                "value_chapter = \"values\"",
            );
            (base, variant, &["docs"])
        }
        "[[kinds]] values" => {
            let (base, variant, _) = kinds("", "values = true");
            (base, variant, &["docs"])
        }
        "[scan] comment_prefixes" => plain("[scan]\n", "comment_prefixes = [\"//\"]"),
        "[scan] docstring_python" => plain("[scan]\n", "docstring_python = false"),
        "[scan] exclude" => plain("[scan]\n", "exclude = [\"vendor\"]"),
        "[scan] extensions" => plain("[scan]\n", "extensions = [\"md\"]"),
        "[scan] include" => plain("[scan]\n", "include = [\"docs\"]"),
        "[scan] respect_gitignore" => plain("[scan]\n", "respect_gitignore = false"),
        "[output] color" => plain("[output]\n", "color = \"never\""),
        "[output] format" => plain("[output]\n", "format = \"json\""),
        "[output] relative_paths" => plain("[output]\n", "relative_paths = false"),
        "[fmt.cross_refs] anchor_format" => {
            plain("[fmt.cross_refs]\n", "anchor_format = \"gitlab\"")
        }
        "[fmt.cross_refs] enabled" => plain("[fmt.cross_refs]\n", "enabled = false"),
        "[workspace] include_root" => plain("[workspace]\n", "include_root = false"),
        "[workspace] members" => plain("[workspace]\n", "members = [\"a\"]"),
        "[workspace] optional_members" => plain("[workspace]\n", "optional_members = [\"b\"]"),
        "[citations] default" => plain("[citations]\n", "default = \"should\""),
        "[citations.<KIND>] default" => plain("[citations.FS]\n", "default = \"should\""),
        "[citations.<KIND>] may" => plain("[citations.FS]\n", "may = [\"AR\"]"),
        "[citations.<KIND>] must" => plain("[citations.FS]\n", "must = [\"AR\"]"),
        "[citations.<KIND>] must-not" => plain("[citations.FS]\n", "must-not = [\"AR\"]"),
        "[citations.<KIND>] should" => plain("[citations.FS]\n", "should = [\"AR\"]"),
        "[citations.<KIND>] should-not" => plain("[citations.FS]\n", "should-not = [\"AR\"]"),
        "[fmt] exclude" => plain("[fmt]\n", "exclude = [\"vendor/**\"]"),
        other => panic!("no lowering case for the mapping.rs row `{other}`"),
    }
}

/// Lower `text`, written to `file` under `root`, with the v1 reader alone.
pub(super) fn lower(root: &Path, file: &str, text: &str) -> Project {
    let path = root.join(file);
    write(&path, text);
    read(&path, Path::new(file), root).unwrap_or_else(|error| panic!("{file}: {error:#}\n{text}"))
}

fn is_under(path: &str, prefix: &str) -> bool {
    path == prefix || path.starts_with(&format!("{prefix}."))
}

/// §AR-config.3.3 item 4: each key, set to a non-default value, changes the
/// field its row names and no other. `grund_config_version` has one value the
/// v1 reader accepts, so writing it changes nothing at all.
#[test]
fn every_key_lowers_into_the_one_field_its_row_names() {
    for (index, (name, path)) in KEYS.iter().enumerate() {
        let root = test_root(&format!("lowering_key_{index}"));
        let (base, variant, dirs) = key_case(name);
        for dir in dirs {
            std::fs::create_dir_all(root.join(dir)).unwrap();
        }
        let before = fields(&lower(&root, "base.toml", &base));
        let after = fields(&lower(&root, "variant.toml", &variant));
        let changed: Vec<_> = before
            .keys()
            .filter(|field| before[*field] != after[*field])
            .copied()
            .collect();
        if *name == "grund_config_version" {
            assert_eq!(changed, Vec::<&str>::new(), "{name}");
            continue;
        }
        assert!(!changed.is_empty(), "{name}: lowering changed nothing");
        for field in changed {
            assert!(
                is_under(field, path),
                "{name}: changed {field}, but mapping.rs lowers it into {path}"
            );
        }
    }
}
