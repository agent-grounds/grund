//! The YAML span tree and the value subset read over it (§DA-yaml-value-reader.2.2,
//! §DA-yaml-value-reader.2.3, §FS-values.2.2.3): spans come from each event's
//! line and character column, so text before a node never shifts it, and the
//! subset names each construct it refuses at its own site.

use super::json::JsonNode;
use super::value_yaml_subset::yaml_value_root;
use super::yaml::{ScalarStyle, YamlKind, YamlNode, parse_yaml};

fn root(text: &str) -> YamlNode {
    let mut stream = parse_yaml(text).expect("readable YAML");
    stream.documents.remove(0).root
}

fn slice<'a>(text: &'a str, node: &YamlNode) -> &'a str {
    &text[node.span.start..node.span.end]
}

fn entries(node: &YamlNode) -> &[(YamlNode, YamlNode)] {
    let YamlKind::Mapping { entries } = &node.kind else {
        panic!("expected a mapping, got {node:?}");
    };
    entries
}

fn items(node: &YamlNode) -> &[YamlNode] {
    let YamlKind::Sequence { items } = &node.kind else {
        panic!("expected a sequence, got {node:?}");
    };
    items
}

/// What the subset makes of `text`: each member's decoded key and, per
/// element, its kind and source slice, or the refusal at its slice.
fn subset(text: &str) -> Vec<(String, Vec<(String, String, usize)>)> {
    let stream = parse_yaml(text).expect("readable YAML");
    let JsonNode::Object(members, _) = yaml_value_root(text, &stream) else {
        panic!("expected an object");
    };
    members
        .into_iter()
        .map(|member| {
            let elements = match member.value {
                JsonNode::Array(elements, _) => elements.iter().map(|e| node(text, e)).collect(),
                other => vec![node(text, &other)],
            };
            (member.key.decoded, elements)
        })
        .collect()
}

fn node(text: &str, node: &JsonNode) -> (String, String, usize) {
    let (kind, span) = match node {
        JsonNode::String(string) => (format!("string {}", string.decoded), string.span),
        JsonNode::Number(raw, span) => (format!("number {raw}"), *span),
        JsonNode::Null(span) => ("null".to_string(), *span),
        JsonNode::Bool(span) => ("bool".to_string(), *span),
        JsonNode::Refused(span, reason) => (reason.clone(), *span),
        other => (format!("{other:?}"), other.span()),
    };
    (kind, text[span.start..span.end].to_string(), span.start)
}

fn line_of(text: &str, at: usize) -> usize {
    text[..at].matches('\n').count() + 1
}

#[test]
fn scalar_spans_cover_their_own_bytes() {
    let text = "CONST-a: [plain, 'it''s', \"q\\\"d\"]\nCONST-b:\n  - 1.5 # a comment\n";
    let tree = root(text);
    let entries = entries(&tree);
    let styles = items(&entries[0].1)
        .iter()
        .map(|item| (slice(text, item), item.kind_style()))
        .collect::<Vec<_>>();
    assert_eq!(
        styles,
        [
            ("plain", ScalarStyle::Plain),
            ("'it''s'", ScalarStyle::SingleQuoted),
            ("\"q\\\"d\"", ScalarStyle::DoubleQuoted),
        ]
    );
    assert_eq!(slice(text, &entries[0].1), "[plain, 'it''s', \"q\\\"d\"]");
    assert_eq!(slice(text, &items(&entries[1].1)[0]), "1.5");
    assert_eq!(slice(text, &entries[1].0), "CONST-b");
}

#[test]
fn a_repeated_key_is_kept_in_order() {
    let text = "CONST-a: [1]\nCONST-b: [2]\nCONST-a: [3]\n";
    let keys = entries(&root(text))
        .iter()
        .map(|(key, _)| slice(text, key).to_string())
        .collect::<Vec<_>>();
    assert_eq!(keys, ["CONST-a", "CONST-b", "CONST-a"]);
}

/// The tree keeps an anchor, a tag and an alias as data; refusing them is the
/// subset's business, never the tree's.
#[test]
fn anchors_tags_and_aliases_are_data_in_the_tree() {
    let text = "CONST-a: &first [1]\nCONST-b: !!seq [2]\nCONST-c: *first\n";
    let tree = root(text);
    let entries = entries(&tree);
    assert!(entries[0].1.anchored);
    assert_eq!(entries[1].1.tag.as_deref(), Some("tag:yaml.org,2002:seq"));
    assert!(matches!(entries[2].1.kind, YamlKind::Alias));
    assert_eq!(slice(text, &entries[2].1), "*first");
}

#[test]
fn a_block_scalar_spans_its_indicator_to_its_last_content_line() {
    let text = "CONST-a:\n  - |\n    one\n    two\n\n  # a comment\n  - 2\n";
    let tree = root(text);
    let items = items(&entries(&tree)[0].1);
    assert_eq!(slice(text, &items[0]), "|\n    one\n    two");
    assert_eq!(slice(text, &items[1]), "2");
}

/// §DA-yaml-value-reader.2.3: non-ASCII text before a node shifts no span,
/// whether it sits in a comment, a quoted element or a block scalar.
#[test]
fn non_ascii_text_before_a_node_shifts_no_span() {
    for (text, expected) in [
        ("# Größe — café\nCONST-a: [1]\n", "[1]"),
        ("CONST-a: ['Größe', \"café\", 7]\n", "7"),
        ("CONST-a:\n  - >\n    Größe — café\n  - 7\n", "7"),
    ] {
        let tree = root(text);
        let value = &entries(&tree)[0].1;
        let last = items(value).last().expect("an element");
        let found = if expected == "[1]" { value } else { last };
        assert_eq!(slice(text, found), expected, "in {text:?}");
    }
}

#[test]
fn a_tab_in_indentation_is_a_parse_error_at_its_line() {
    let error = parse_yaml("CONST-a:\n\t- 1\n").expect_err("a tab indents nothing");
    assert_eq!(error.line, 2);
}

/// §FS-values.2.2.3: each construct the subset refuses is named at its own
/// site, never at the root.
#[test]
fn the_subset_refuses_each_construct_at_its_line() {
    let text = concat!(
        "CONST-anchor: &a [1]\n",
        "CONST-alias: *a\n",
        "CONST-tag: !!seq [1]\n",
        "<<: [1]\n",
        "CONST-block:\n  - |\n    text\n",
        "CONST-multi:\n  - two\n    lines\n",
        "CONST-escape: [\"\\x41\"]\n",
        "CONST-nested: [[1]]\n",
    );
    let refusals = subset(text)
        .into_iter()
        .map(|(_, elements)| {
            let (reason, _, start) = elements.into_iter().next().expect("one finding");
            (reason, line_of(text, start))
        })
        .collect::<Vec<_>>();
    let refuses = |construct: &str| format!("a YAML value source refuses {construct}");
    assert_eq!(
        refusals,
        [
            (refuses("an anchor"), 1),
            (refuses("an alias"), 2),
            (refuses("a tag"), 3),
            (refuses("a merge key"), 4),
            (refuses("a block scalar"), 6),
            (refuses("a multi-line scalar"), 9),
            (refuses("the escape `\\x`"), 11),
            (refuses("a nested collection"), 12),
        ]
    );
}

#[test]
fn a_second_document_is_refused_from_its_marker() {
    let text = "CONST-a: [1]\n---\nCONST-b: [2]\n";
    let stream = parse_yaml(text).expect("readable YAML");
    let JsonNode::Refused(span, reason) = yaml_value_root(text, &stream) else {
        panic!("expected a refusal");
    };
    assert_eq!(reason, "a YAML value source refuses a second document");
    assert_eq!(line_of(text, span.start), 2);
}

/// §FS-values.2.2.3: a component's kind comes from how it is written: quoted is
/// a string, plain is a number only in JSON number grammar.
#[test]
fn a_scalar_takes_its_kind_from_how_it_is_written() {
    let text = concat!(
        "CONST-a: [1.10, 0o17, yes, '1', \"a\\u0041\\n\", 'it''s', -2e3]\n",
        "CONST-b: [true, null, ~, ]\n",
        "CONST-c:\n- \n- x\n",
    );
    let members = subset(text);
    let kinds = |index: usize| {
        members[index]
            .1
            .iter()
            .map(|(kind, slice, _)| format!("{kind} @ {slice}"))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        kinds(0),
        [
            "number 1.10 @ 1.10",
            "string 0o17 @ 0o17",
            "string yes @ yes",
            "string 1 @ '1'",
            "string aA\n @ \"a\\u0041\\n\"",
            "string it's @ 'it''s'",
            "number -2e3 @ -2e3",
        ]
    );
    assert_eq!(kinds(1), ["bool @ true", "null @ null", "null @ ~"]);
    // An empty element is located from its own `-`.
    assert_eq!(kinds(2), ["null @ -", "string x @ x"]);
}

#[test]
fn quoted_keys_and_a_sequence_at_the_keys_indent_read_like_any_other() {
    let text = "'CONST-a':\n- 1\n\"CONST-b\": [2]\n";
    let members = subset(text);
    assert_eq!(
        members
            .iter()
            .map(|(key, elements)| (key.as_str(), elements[0].1.as_str()))
            .collect::<Vec<_>>(),
        [("CONST-a", "1"), ("CONST-b", "2")]
    );
}

/// A BOM is stripped before parsing and counted in every span; CRLF line ends
/// stay out of a scalar's slice.
#[test]
fn a_bom_and_crlf_line_ends_keep_spans_exact() {
    let text = "\u{feff}CONST-a:\r\n  - 1\r\n  - 'two'\r\n";
    let members = subset(text);
    assert_eq!(members[0].0, "CONST-a");
    let elements = members[0]
        .1
        .iter()
        .map(|(kind, slice, _)| (kind.as_str(), slice.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(elements, [("number 1", "1"), ("string two", "'two'")]);
}

/// The member's slice, which `show` prints, runs from its key to its last
/// element, comments between elements included and a trailing one left out.
#[test]
fn a_member_slice_keeps_comments_between_its_elements() {
    let text = "CONST-a:\n  - 1 # first\n  # between\n  - 2 # last\nCONST-b: [3]\n";
    let stream = parse_yaml(text).expect("readable YAML");
    let JsonNode::Object(members, _) = yaml_value_root(text, &stream) else {
        panic!("expected an object");
    };
    assert_eq!(
        &text[members[0].span.start..members[0].span.end],
        "CONST-a:\n  - 1 # first\n  # between\n  - 2"
    );
}

/// The tree's one nesting bound (§DA-yaml-value-reader.2.2): 128 nested collections
/// read, whatever holds them.
#[test]
fn collections_nested_128_deep_read() {
    let text = format!("{}x\n", "- ".repeat(128));
    assert!(
        parse_yaml(&text).is_ok(),
        "128 nested sequences are readable"
    );
}

/// A mapping and the 128 sequences under it are 129 collections, refused at the
/// line of the one that crosses the bound, before the tree recurses into it.
#[test]
fn a_collection_nested_129_deep_is_refused_at_its_line() {
    let text = format!("k: v\nnested:\n  {}x\n", "- ".repeat(128));
    let error = parse_yaml(&text)
        .err()
        .expect("a collection nested 129 deep is refused");
    assert_eq!(
        (error.line, error.message.as_str()),
        (3, "collections are nested more than 128 deep")
    );
}

/// The author's reproducer: 10000 levels ran the tree out of stack (§REQ-never-crashes.1).
#[test]
#[ignore = "red until #1048 bounds the span tree's nesting; implement removes this"]
fn ten_thousand_nested_collections_are_refused_not_a_stack_overflow() {
    let text = format!("CONST-deep:\n  {}x\n", "- ".repeat(10_000));
    let error = parse_yaml(&text)
        .err()
        .expect("a collection nested past the bound is refused");
    assert_eq!(
        (error.line, error.message.as_str()),
        (2, "collections are nested more than 128 deep")
    );
}

impl YamlNode {
    fn kind_style(&self) -> ScalarStyle {
        let YamlKind::Scalar { style, .. } = &self.kind else {
            panic!("expected a scalar");
        };
        *style
    }
}
