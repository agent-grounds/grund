//! Test module: a slot's shape line (§FS-schema.4.1) — the heading to write as a code
//! span, its presence and placement in check's words, and what the chapter holds — for
//! every kind of slot: prose titled and untitled, each placement, a scalar, a value
//! chapter, a front-matter field with its keys and its `closed` (§FS-schema.4.1.3), a
//! value kind in each form its home holds and fetched (§FS-schema.4.1.4), a kind's
//! `closed`, the code-span fences and the placeholders (§FS-schema.4.1.2).

use super::schema_slot::kind_closed_line;
use super::*;
use crate::model::ValueFormat;
use crate::testing::{test_root, write};

fn slot(content: FieldContent, title: Option<&str>) -> SchemaSlot {
    SchemaSlot {
        name: Some("terms".to_string()),
        presence: Strength::Must,
        placement: Some(Placement::First),
        content,
        title: title.map(str::to_string),
        files: None,
    }
}

/// The one line a slot that nests nothing prints.
fn text(slot: &SchemaSlot) -> String {
    let lines = slot.shape_lines();
    assert_eq!(lines.len(), 1, "{lines:?}");
    lines[0].text.clone()
}

/// §FS-schema.4.1: a prose field is its heading, then its presence and placement.
#[test]
fn a_prose_field_shows_its_heading_presence_and_placement() {
    let titled = slot(FieldContent::Prose, Some("Terms"));
    assert_eq!(
        titled.shape_lines(),
        vec![ShapeLine {
            depth: 0,
            key: "field terms".to_string(),
            text: "`## terms: Terms` (must, first)".to_string(),
        }]
    );
    // §FS-schema.4.1.2: an untitled field's title is a placeholder.
    let untitled = SchemaSlot {
        presence: Strength::May,
        placement: None,
        ..slot(FieldContent::Prose, None)
    };
    assert_eq!(text(&untitled), "`## terms: <title>` (may)");
}

/// §FS-schema.4.1, §FS-fields.order.after: `last`, and `after` in check's words.
#[test]
fn each_placement_is_written_in_checks_words() {
    let placed = |placement| SchemaSlot {
        placement: Some(placement),
        ..slot(FieldContent::Prose, Some("Terms"))
    };
    assert_eq!(
        text(&placed(Placement::Last)),
        "`## terms: Terms` (must, last)"
    );
    assert_eq!(
        text(&placed(Placement::After("reason".to_string()))),
        "`## terms: Terms` (must, directly after reason)"
    );
}

/// §FS-schema.4.1, §FS-fields.content.one-of: a scalar's title is its values'
/// placeholder, and the line says the title is the value.
#[test]
fn a_scalar_shows_its_values_as_the_title() {
    let values = vec!["Low".to_string(), "High".to_string()];
    assert_eq!(
        text(&slot(FieldContent::OneOf(values), None)),
        "`## terms: <one of Low, High>` (must, first): the title is the value"
    );
}

/// §FS-schema.4.1, §FS-fields.content.values: a value chapter holds named roots
/// under its own name, each with numbered parts.
#[test]
fn a_value_chapter_shows_its_roots_and_parts() {
    let values = SchemaSlot {
        placement: None,
        ..slot(FieldContent::Values, Some("Constants"))
    };
    assert_eq!(
        text(&values),
        "`## terms: Constants` (must): only values, each `### terms.<name>: <title>` with parts \
         `#### .1: <number or word>`; a unit is a part of its own"
    );
}

/// §FS-schema.4.1.3: front matter is no heading; its keys and its own `closed`
/// nest one level under it, each as the line to write.
#[test]
fn front_matter_shows_the_block_its_keys_and_its_closed() {
    let shape = FrontMatterShape {
        closed: Strength::Must,
        keys: vec![
            FrontMatterKey {
                name: "status".to_string(),
                presence: Strength::Must,
                one_of: Some(vec!["proposed".to_string(), "accepted".to_string()]),
            },
            FrontMatterKey {
                name: "date".to_string(),
                presence: Strength::May,
                one_of: None,
            },
        ],
    };
    let front = SchemaSlot {
        name: Some("front-matter".to_string()),
        placement: None,
        ..slot(FieldContent::FrontMatter(shape.clone()), None)
    };
    let lines: Vec<(usize, String, String)> = (front.shape_lines().into_iter())
        .map(|line| (line.depth, line.key, line.text))
        .collect();
    let expected = [
        (
            0,
            "field front-matter",
            "`---` YAML or `+++` TOML front matter at the top of the file (must)",
        ),
        (
            1,
            "key status",
            "`status: <one of proposed, accepted>` (must)",
        ),
        (1, "key date", "`date: <value>` (may)"),
        (1, "closed", "no other key (must)"),
    ];
    let expected: Vec<(usize, String, String)> = (expected.into_iter())
        .map(|(depth, key, text)| (depth, key.to_string(), text.to_string()))
        .collect();
    assert_eq!(lines, expected);
    // An open block prints no `closed` line of its own.
    let open = SchemaSlot {
        content: FieldContent::FrontMatter(FrontMatterShape {
            closed: Strength::May,
            ..shape
        }),
        ..front
    };
    assert_eq!(open.shape_lines().len(), 3);
}

/// §FS-schema.4.1: a kind's `closed` names what it still admits.
#[test]
fn a_kind_closed_line_names_what_it_still_admits() {
    assert_eq!(
        kind_closed_line(Strength::Warn),
        ShapeLine {
            depth: 0,
            key: "closed".to_string(),
            text: "no other chapter except numbered ones (warn)".to_string(),
        }
    );
}

fn numbered(files: ValueFiles) -> SchemaSlot {
    SchemaSlot {
        name: None,
        presence: Strength::May,
        placement: None,
        content: FieldContent::Values,
        title: None,
        files: Some(files),
    }
}

fn home(place: &str, folder: bool, forms: &[ValueForm]) -> ValueHome {
    ValueHome {
        place: place.to_string(),
        folder,
        forms: forms.to_vec(),
    }
}

/// §FS-schema.4.1.4: a file home shows its one form and where, then the citation
/// part 1 answers to.
#[test]
fn a_value_file_home_shows_its_form_and_the_first_part() {
    let json = ValueForm::Read(ValueFormat::Json);
    let line = numbered(ValueFiles {
        fetched: false,
        homes: vec![home("config/constants.json", false, &[json])],
    })
    .shape_lines();
    assert_eq!(line[0].key, "field <number>");
    assert_eq!(
        line[0].text,
        "`{\"<ID>\": [<number or word>, …], …}` in `config/constants.json`: part 1 is \
         `\u{a7}<ID>.1`; a unit is a part of its own"
    );
    let markdown = numbered(ValueFiles {
        fetched: false,
        homes: vec![home("terms.md", false, &[ValueForm::Markdown])],
    });
    assert_eq!(
        text(&markdown),
        "`# <ID>: <title>` with only `## 1. <number or word>`, `## 2. …` under it in \
         `terms.md`: part 1 is `\u{a7}<ID>.1`; a unit is a part of its own"
    );
}

/// §FS-schema.4.1.4: a folder home names each form, joined by `, ` and by `, or `
/// before the last.
#[test]
fn a_value_folder_home_names_every_form_it_holds() {
    let forms = [
        ValueForm::Markdown,
        ValueForm::Read(ValueFormat::Json),
        ValueForm::Read(ValueFormat::Yaml),
    ];
    let folder = numbered(ValueFiles {
        fetched: false,
        homes: vec![home("consts/", true, &forms)],
    });
    assert_eq!(
        text(&folder),
        "`# <ID>: <title>` with only `## 1. <number or word>`, `## 2. …` under it in a \
         Markdown file in `consts/`, `{\"<ID>\": [<number or word>, …], …}` in a JSON file in \
         `consts/`, or `<ID>: [<number or word>, …]` in a YAML file in `consts/`: part 1 is \
         `\u{a7}<ID>.1`; a unit is a part of its own"
    );
}

/// §FS-schema.4.1.4: a fetched kind's line teaches no form to write by hand.
#[test]
fn a_fetched_value_kind_names_the_file_grund_fetch_writes() {
    let fetched = numbered(ValueFiles {
        fetched: true,
        homes: vec![home("values/limits.md", false, &[])],
    });
    assert_eq!(
        text(&fetched),
        "written by `grund fetch` into `values/limits.md`: part 1 is `\u{a7}<ID>.1`"
    );
}

/// §FS-schema.4.1.2: the fence is one backtick longer than the longest run inside
/// the span, with a space inside each fence where it begins or ends with one.
#[test]
fn a_title_with_backticks_gets_a_longer_fence() {
    let line = |title: &str| text(&slot(FieldContent::Prose, Some(title)));
    assert_eq!(
        line("Use ``x`` here"),
        "```## terms: Use ``x`` here``` (must, first)"
    );
    assert_eq!(line("form `x`"), "`` ## terms: form `x` `` (must, first)");
}

/// §FS-schema.4.1.2: a control character in a title or a value is its TOML
/// basic-string escape, so the line stays one line.
#[test]
fn a_title_with_a_line_break_stays_on_one_line() {
    assert_eq!(
        text(&slot(FieldContent::Prose, Some("Sum\nmary"))),
        "`## terms: Sum\\nmary` (must, first)"
    );
    let values = vec!["A\tB".to_string()];
    assert_eq!(
        text(&slot(FieldContent::OneOf(values), None)),
        "`## terms: <one of A\\tB>` (must, first): the title is the value"
    );
}

const VALUE_KINDS: &str = "grund_config_version = 2\nproject_name = \"view\"\n\
                           [schema.sources]\nlanguages = [\"markdown\"]\n\
                           [schema.kinds.J]\nfiles = [\"c.json\"]\nform = \"value\"\n\
                           [schema.kinds.Y]\nfiles = [\"c.yaml\"]\nform = \"value\"\n\
                           [schema.kinds.M]\nfolders = [\"consts\"]\nindex = false\n\
                           form = \"value\"\n\
                           [schema.kinds.F]\nfiles = [\"f.md\"]\nform = \"value\"\n\
                           fetch = \"scripts/fetch\"\n";

/// §FS-schema.4.1.4: the view asks each home's forms of the rule the scan reads a
/// value home by, and a fetched kind its one home.
#[test]
fn the_view_reads_each_value_homes_forms_by_the_scans_rule() {
    let root = test_root("schema-slot-value-homes");
    write(&root.join("grund.toml"), VALUE_KINDS);
    // A value kind's home must exist (§FS-values.2.2).
    write(&root.join("c.json"), "{}\n");
    write(&root.join("c.yaml"), "{}\n");
    write(&root.join("consts/m.md"), "# M-a: A\n\n## 1. 1\n");
    write(&root.join("f.md"), "# F-a: A\n\n## 1. 1\n");
    let config = load_config(&root).expect("the file should load");
    let view = SchemaView::of(config.project());
    let files = |kind: &str| {
        let node = view
            .nodes
            .iter()
            .find(|node| node.name == kind)
            .expect(kind);
        node.slots[0].files.clone().expect("a value kind's files")
    };
    let json = ValueForm::Read(ValueFormat::Json);
    let yaml = ValueForm::Read(ValueFormat::Yaml);
    assert_eq!(files("J").homes, vec![home("c.json", false, &[json])]);
    assert_eq!(files("Y").homes, vec![home("c.yaml", false, &[yaml])]);
    let folder = [ValueForm::Markdown, json, yaml];
    assert_eq!(files("M").homes, vec![home("consts/", true, &folder)]);
    let fetched = files("F");
    assert!(fetched.fetched);
    assert_eq!(fetched.homes, vec![home("f.md", false, &[])]);
}

/// §FS-init.2.3.5.12.2: the block's bullet is the shape line, which the view
/// prints after the slot's key.
#[test]
fn the_block_bullet_is_the_shape_line() {
    let root = test_root("schema-slot-block-bullet");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 2\nproject_name = \"view\"\n[schema.kinds.FS]\n\
         folders = [\"fs\"]\n[schema.kinds.FS.fields.terms]\ntitle = \"Terms\"\n\
         presence = \"must\"\nposition = \"first\"\n",
    );
    let config = load_config(&root).expect("the file should load");
    let view = SchemaView::of(config.project());
    let fs = view
        .nodes
        .iter()
        .find(|node| node.name == "FS")
        .expect("FS");
    assert_eq!(
        fs.shape_lines(),
        vec![ShapeLine {
            depth: 0,
            key: "field terms".to_string(),
            text: "`## terms: Terms` (must, first)".to_string(),
        }]
    );
}
