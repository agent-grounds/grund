//! Test module: the schema view lowered from a `Project` (§AR-config.6.3) — the
//! class of each node, one junction per any-of entry and strength, grounding as a
//! badge, the order of edges, every place of a row, and `everything else`
//! (§FS-schema), in the vocabulary of node, junction and badge (§FS-schema.terms).

use super::*;
use crate::testing::{test_root, write};

const HEAD: &str = "grund_config_version = 2\nproject_name = \"view\"\n[schema.sources]\nlanguages = [\"markdown\"]\n";

fn project(name: &str, body: &str) -> Project {
    let root = test_root(name);
    write(&root.join("grund.toml"), &format!("{HEAD}{body}"));
    match load_config(&root) {
        Ok(config) => config.project().clone(),
        Err(err) => panic!("{name}: the file should load: {err:#}"),
    }
}

fn names(view: &SchemaView) -> Vec<&str> {
    view.nodes.iter().map(|node| node.name.as_str()).collect()
}

fn node<'a>(view: &'a SchemaView, name: &str) -> &'a SchemaNode {
    view.nodes
        .iter()
        .find(|node| node.name == name)
        .expect(name)
}

const KINDS: &str = "[schema.kinds.GOAL]\nfiles = [\"goals.md\"]\n\
                     [schema.kinds.FS]\nfolders = [\"fs\"]\n\
                     [schema.kinds.AR]\nfolders = [\"ar\"]\n\
                     [schema.kinds.test]\ncitable = false\nfolders = [\"tests\"]\n\
                     [schema.kinds.vendor]\ncitable = false\nscan = false\nfolders = [\"vendor\"]\n";

/// §FS-schema.4, §FS-schema.7: every row is one node in row order, its class a
/// word, and the implicit complement comes after every written row.
#[test]
fn every_row_is_one_node_of_its_class() {
    let mut project = project("schema-view-classes", KINDS);
    // §FS-schema.4: a homeless kind is a kind row with no place.
    let mut homeless = project.schema.rows[1].clone();
    homeless.name = "DF".to_string();
    homeless.places.clear();
    project.schema.rows.push(homeless);
    let view = SchemaView::of(&project);
    assert_eq!(
        names(&view),
        ["GOAL", "FS", "AR", "test", "vendor", "DF", "code"]
    );
    let classes: Vec<NodeClass> = view.nodes.iter().map(|node| node.class).collect();
    assert_eq!(
        classes,
        [
            NodeClass::Kind,
            NodeClass::Kind,
            NodeClass::Kind,
            NodeClass::Place,
            NodeClass::Unscanned,
            NodeClass::HomelessKind,
            NodeClass::Complement,
        ]
    );
    assert_eq!(node(&view, "code").places, [Extent::Complement]);
    assert_eq!(node(&view, "FS").places, [Extent::Folder("fs".to_string())]);
}

/// §FS-schema.4: a row with more than one place lists every one in written order.
#[test]
fn a_two_place_row_lists_both_places() {
    let mut project = project("schema-view-two-places", KINDS);
    let second = Place {
        extent: Extent::Folder("specs".to_string()),
        scanned: true,
        source: None,
    };
    project.schema.rows[1].places.push(second);
    let view = SchemaView::of(&project);
    assert_eq!(
        node(&view, "FS").places,
        [
            Extent::Folder("fs".to_string()),
            Extent::Folder("specs".to_string())
        ]
    );
}

/// §FS-schema.5.1: an any-of entry is one edge through one junction, and subjects
/// writing the same entry at the same strength share it; another strength is
/// another junction.
#[test]
fn identical_any_of_entries_at_one_strength_share_a_junction() {
    let rules = "[rules.citations.FS]\nshould = [\"GOAL|AR\"]\n\
                 [rules.citations.AR]\nmust = [\"GOAL|AR\"]\n\
                 [rules.citations.test]\nshould = [\"GOAL|AR\"]\n";
    let view = SchemaView::of(&project(
        "schema-view-junctions",
        &format!("{KINDS}{rules}"),
    ));
    assert_eq!(view.edges.len(), 3);
    let junctions: Vec<Option<usize>> = view.edges.iter().map(|edge| edge.junction).collect();
    // FS's should, AR's must at another strength, then test's should, which
    // reuses FS's junction.
    assert_eq!(junctions, [Some(0), Some(1), Some(0)]);
    assert_eq!(view.junctions.len(), 2);
    let labels: Vec<&str> = view.junctions[0]
        .targets
        .iter()
        .map(|t| t.label.as_str())
        .collect();
    assert_eq!(labels, ["GOAL", "AR"]);
}

/// §FS-schema.6: grounding is a badge on its own node, never an edge; a row's own
/// ladder replaces the project's, and an unscanned row carries none.
#[test]
fn grounding_is_a_badge_and_never_an_edge() {
    let rules = "[rules.citations.grounding]\nmust = \"file\"\n\
                 [rules.citations.test.grounding]\nwarn = \"h2\"\n";
    let view = SchemaView::of(&project("schema-view-badges", &format!("{KINDS}{rules}")));
    assert!(view.edges.is_empty());
    let badge = |name| node(&view, name).grounding.clone();
    let must_file = vec![BadgeRung {
        strength: Strength::Must,
        level: 1,
    }];
    assert_eq!(badge("FS"), must_file);
    assert_eq!(badge("code"), must_file);
    assert_eq!(
        badge("test"),
        [BadgeRung {
            strength: Strength::Warn,
            level: 2
        }]
    );
    assert_eq!(badge("test")[0].unit(), "h2");
    assert!(badge("vendor").is_empty());
    assert!(view.nodes.iter().all(|node| !node.ruled));
}

/// §FS-schema.6: a `may` rung checks nothing, so a ladder of `may` alone is no
/// badge, and a `may` rung beside a `must` adds no rung to it (§REQ-one-answer).
#[test]
fn a_may_rung_is_no_badge() {
    let rules = "[rules.citations.grounding]\nmay = \"file\"\n\
                 [rules.citations.FS.grounding]\nmust = \"h2\"\nmay = \"h3\"\n";
    let view = SchemaView::of(&project("schema-view-may-rung", &format!("{KINDS}{rules}")));
    assert!(node(&view, "GOAL").grounding.is_empty());
    assert!(node(&view, "test").grounding.is_empty());
    assert!(node(&view, "code").grounding.is_empty());
    assert_eq!(
        node(&view, "FS").grounding,
        [BadgeRung {
            strength: Strength::Must,
            level: 2
        }]
    );
}

/// §FS-schema.7, §FS-schema.5.2: edges follow their subject's row, then strength
/// from `must` down with the prohibitions last, then written order — never the
/// order the lists were written in.
#[test]
fn edges_follow_subject_then_strength_then_written_order() {
    let kinds = "[schema.kinds.GOAL]\nfiles = [\"goals.md\"]\n\
                 [schema.kinds.FS]\nfolders = [\"fs\"]\n[schema.kinds.AR]\nfolders = [\"ar\"]\n\
                 [schema.kinds.DF]\nfolders = [\"df\"]\n[schema.kinds.DA]\nfolders = [\"da\"]\n\
                 [schema.kinds.RM]\nfolders = [\"rm\"]\n[schema.kinds.REQ]\nfolders = [\"req\"]\n";
    let rules = "[rules.citations.AR]\nmust = [\"FS\"]\n\
                 [rules.citations.FS]\nshould-not = [\"REQ\"]\nmust-not = [\"RM\"]\n\
                 may = [\"DA\"]\nshould = [\"DF\"]\nwarn = [\"AR\"]\nmust = [\"GOAL\", \"FS\"]\n";
    let view = SchemaView::of(&project("schema-view-order", &format!("{kinds}{rules}")));
    let order: Vec<(&str, &str, &str)> = view
        .edges
        .iter()
        .map(|edge| {
            let subject = view.nodes[edge.subject].name.as_str();
            (subject, edge.level.as_str(), edge.targets[0].label.as_str())
        })
        .collect();
    assert_eq!(
        order,
        [
            ("FS", "must", "GOAL"),
            ("FS", "must", "FS"),
            ("FS", "warn", "AR"),
            ("FS", "should", "DF"),
            ("FS", "may", "DA"),
            ("FS", "must-not", "RM"),
            ("FS", "should-not", "REQ"),
            ("AR", "must", "FS"),
        ]
    );
    assert!(view.edges[1].is_self());
    assert!(!view.edges[0].is_self());
}

/// §FS-schema.5: a subject's `default` and the project's, where not `may`, are
/// shown as `everything else`; a `may` default shows nothing.
#[test]
fn a_default_other_than_may_is_everything_else() {
    let rules = "[rules.citations]\ndefault = \"should-not\"\n\
                 [rules.citations.FS]\ndefault = \"must-not\"\n\
                 [rules.citations.AR]\ndefault = \"may\"\n";
    let view = SchemaView::of(&project("schema-view-defaults", &format!("{KINDS}{rules}")));
    assert_eq!(view.default, Some(CitationLevel::ShouldNot));
    assert_eq!(node(&view, "FS").default, Some(CitationLevel::MustNot));
    assert!(node(&view, "FS").ruled);
    assert_eq!(node(&view, "AR").default, None);
}

/// §FS-schema.4.1: a kind's fields are its slots, and `closed` shows only where
/// it is not `may`.
#[test]
fn a_kind_carries_its_slots_and_closed_strength() {
    let fields = "[schema.kinds.FS.fields.status]\none_of = [\"Draft\", \"Accepted\"]\n\
                  presence = \"should\"\nposition = \"first\"\n";
    let mut project = project("schema-view-fields", &format!("{KINDS}{fields}"));
    let view = SchemaView::of(&project);
    let slots = &node(&view, "FS").slots;
    assert_eq!(slots.len(), 1);
    assert_eq!(slots[0].name.as_deref(), Some("status"));
    assert_eq!(slots[0].presence, Strength::Should);
    assert_eq!(slots[0].placement, Some(Placement::First));
    assert_eq!(
        slots[0].content,
        FieldContent::OneOf(vec!["Draft".to_string(), "Accepted".to_string()])
    );
    assert_eq!(node(&view, "FS").closed, None);
    project.schema.rows[1].kind.as_mut().unwrap().closed = Strength::Warn;
    assert_eq!(
        node(&SchemaView::of(&project), "FS").closed,
        Some(Strength::Warn)
    );
}

fn titled(content: FieldContent, title: Option<&str>) -> SchemaSlot {
    SchemaSlot {
        name: Some("terms".to_string()),
        presence: Strength::Must,
        placement: Some(Placement::First),
        content,
        title: title.map(str::to_string),
    }
}

/// §FS-schema.4.1: a titled field's line ends with `titled` and its title as a
/// code span, after its content, for a prose field and a value chapter alike.
#[test]
fn a_field_line_names_its_title_last() {
    let prose = titled(FieldContent::Prose, Some("Terms"));
    assert_eq!(
        prose.field_line(),
        (
            "terms".to_string(),
            "must · first · titled `Terms`".to_string()
        )
    );
    let values = SchemaSlot {
        presence: Strength::May,
        placement: None,
        ..titled(FieldContent::Values, Some("Constants"))
    };
    assert_eq!(values.field_line().1, "may · values · titled `Constants`");
}

/// §FS-schema.4.1: a field with no title prints the line it printed before
/// titles were shown.
#[test]
fn an_untitled_field_line_keeps_its_bytes() {
    assert_eq!(
        titled(FieldContent::Prose, None).field_line().1,
        "must · first"
    );
}

/// §FS-schema.4.1: the fence is one backtick longer than the longest run inside
/// the title, with a space inside each fence where the title begins or ends with one.
#[test]
fn a_title_with_backticks_gets_a_longer_fence() {
    let line = |title: &str| titled(FieldContent::Prose, Some(title)).field_line().1;
    assert_eq!(
        line("Use ``x`` here"),
        "must · first · titled ```Use ``x`` here```"
    );
    assert_eq!(
        line("`code` form"),
        "must · first · titled `` `code` form ``"
    );
    assert_eq!(line("form `x`"), "must · first · titled `` form `x` ``");
}

/// §FS-schema.4.1: a control character in a title is its TOML basic-string
/// escape, so the line stays one line.
#[test]
fn a_title_with_a_line_break_stays_on_one_line() {
    let line = titled(FieldContent::Prose, Some("Sum\nmary"))
        .field_line()
        .1;
    assert_eq!(line, "must · first · titled `Sum\\nmary`");
}

/// §FS-init.2.3.5.12.2: the block's bullet is the field line with single spaces.
#[test]
fn the_block_bullet_is_the_titled_field_line() {
    let fields = "[schema.kinds.FS.fields.terms]\ntitle = \"Terms\"\n\
                  presence = \"must\"\nposition = \"first\"\n";
    let view = SchemaView::of(&project("schema-view-titled", &format!("{KINDS}{fields}")));
    let fs = node(&view, "FS");
    assert_eq!(fs.slots[0].title.as_deref(), Some("Terms"));
    assert_eq!(
        fs.shape_lines(),
        vec![(0, "field terms must · first · titled `Terms`".to_string())]
    );
}

/// §FS-schema.8: with no config file found, the header names none.
#[test]
fn a_tree_without_a_config_file_names_none_in_the_header() {
    let root = test_root("schema-view-no-config");
    std::fs::create_dir_all(&root).expect("root");
    let config = load_config(&root).expect("defaults load");
    assert_eq!(
        SchemaView::of_config(&config).config_file,
        "defaults, no grund.toml"
    );
    write(&root.join("grund.toml"), HEAD);
    let config = load_config(&root).expect("the file should load");
    assert_eq!(SchemaView::of_config(&config).config_file, "grund.toml");
}
