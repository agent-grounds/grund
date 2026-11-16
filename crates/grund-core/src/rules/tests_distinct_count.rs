//! What a `distinct` outbound count counts (§FS-rules.3.2.1, §FS-rules.5.2): the different
//! declarations owning the targets of the sites the plain count reads, so a declaration cited
//! bare, at a numbered section and at a named chapter counts once; its row
//! (§FS-rules.7.3) and its counted sites, one per citation and not one per declaration
//! (§FS-rules.7.3.1); and its deduplication key, which carries the counting mode
//! (§FS-rules.6). Issue #1367 is the report: one `SRC` cited three times satisfied
//! `must cite at least 3 SRC`, and no sentence could ask for three different ones.
//!
//! Each rule is parsed from its sentence, so every test that names `distinct` is red until the
//! sentence front end accepts it, with `unknown kind "distinct SRC"`. The plain counts are the
//! controls, green today and after.

use super::RuleAnchor;
use super::engine::evaluate;
use super::facts::{Completeness, FactHeader, NodeKey, NodeMeta, RuleFacts, SiteKey, SiteMeta};
use super::sentence::{ParsedRule, RuleVocabulary, parse_rule};
use crate::config::{Config, KindTable};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

fn key(value: &str) -> NodeKey {
    NodeKey(value.into())
}

fn anchor(path: &str, line: usize, column: Option<usize>) -> RuleAnchor {
    RuleAnchor {
        path: path.into(),
        line,
        column,
    }
}

/// A snapshot built one fact at a time; a node's key is its label.
struct Facts(RuleFacts);

impl Facts {
    fn new() -> Self {
        Self(RuleFacts {
            header: FactHeader {
                schema: 1,
                project: "distinct-count".into(),
                producer: "test".into(),
                completeness: Completeness::Complete,
            },
            decl: Vec::new(),
            chapter: Vec::new(),
            contains: Vec::new(),
            cites: Vec::new(),
            site_in: Vec::new(),
            nodes: BTreeMap::new(),
            sites: BTreeMap::new(),
        })
    }

    fn declare(&mut self, id: &str, kind: &str) -> &mut Self {
        self.0.decl.push((key(id), kind.into()));
        let path = format!("docs/{id}.md");
        let meta = NodeMeta {
            label: id.into(),
            anchor: anchor(&path, 1, None),
        };
        self.0.nodes.insert(key(id), meta);
        self
    }

    /// A named chapter `handle` directly under declaration `id`.
    fn chapter(&mut self, id: &str, handle: &str) -> &mut Self {
        let node = format!("{id}.{handle}");
        let path = self.0.nodes[&key(id)].anchor.path.clone();
        self.0
            .chapter
            .push((key(&node), handle.into(), Some(handle.into())));
        self.0.contains.push((key(id), key(&node)));
        let meta = NodeMeta {
            label: node.clone(),
            anchor: anchor(&path, 3, None),
        };
        self.0.nodes.insert(key(&node), meta);
        self
    }

    /// A citation written as `written` in `from`'s file at `line`, resolving to unit `target`.
    fn cite(&mut self, from: &str, target: &str, written: &str, line: usize) -> &mut Self {
        let site = SiteKey(format!("site-{}", self.0.cites.len()));
        let path = self.0.nodes[&key(from)].anchor.path.clone();
        self.0.cites.push((site.clone(), key(from), key(target)));
        self.0.site_in.push((site.clone(), key(from)));
        let meta = SiteMeta {
            label: format!("\u{a7}{written}"),
            anchor: anchor(&path, line, Some(1)),
        };
        self.0.sites.insert(site, meta);
        self
    }
}

/// `SRC-x` has a named `outcome` chapter; a numbered section citation resolves to its
/// declaration (§FS-rules.5.1). `COUNTRY-a` cites `SRC-x` bare, at `.2` and at `.outcome`;
/// `COUNTRY-b` cites `SRC-y` and `DOC-d`; `COUNTRY-c` cites nothing; `SRC-z` cites itself
/// bare and at its `outcome` chapter, and `SRC-y` once.
fn facts() -> RuleFacts {
    let mut facts = Facts::new();
    facts
        .declare("SRC-x", "SRC")
        .chapter("SRC-x", "outcome")
        .declare("SRC-y", "SRC")
        .declare("SRC-z", "SRC")
        .chapter("SRC-z", "outcome")
        .declare("DOC-d", "DOC")
        .declare("COUNTRY-a", "COUNTRY")
        .declare("COUNTRY-b", "COUNTRY")
        .declare("COUNTRY-c", "COUNTRY");
    facts
        .cite("COUNTRY-a", "SRC-x", "SRC-x", 5)
        .cite("COUNTRY-a", "SRC-x", "SRC-x.2", 6)
        .cite("COUNTRY-a", "SRC-x.outcome", "SRC-x.outcome", 7)
        .cite("COUNTRY-b", "SRC-y", "SRC-y", 5)
        .cite("COUNTRY-b", "DOC-d", "DOC-d", 6)
        .cite("SRC-z", "SRC-z", "SRC-z", 5)
        .cite("SRC-z", "SRC-z.outcome", "SRC-z.outcome", 6)
        .cite("SRC-z", "SRC-y", "SRC-y", 7);
    facts.0
}

fn vocabulary() -> RuleVocabulary {
    let mut config = Config::default_for(PathBuf::from("distinct-count"));
    config.id_format = "{kind}-{slug}".into();
    config.slug_pattern = "[a-z][a-z0-9-]*".into();
    config.named_sections = true;
    config.rebuild_grammar().unwrap();
    let kinds: BTreeSet<String> = ["COUNTRY", "DOC", "SRC"].map(String::from).into();
    RuleVocabulary {
        fields: Default::default(),
        objects: KindTable::of_citable(&kinds),
        kinds,
        named_sections: true,
        named_kinds: Default::default(),
        non_citable: Default::default(),
        id_grammars: vec![config.grammar],
        section_separators: vec![".".into()],
    }
}

/// The rule a sentence states, from declaration `origin`.
fn rule(origin: &str, title: &str) -> ParsedRule {
    let anchor = anchor(&format!("docs/rules/{origin}.md"), 1, None);
    parse_rule(title, origin.into(), anchor, &vocabulary())
        .unwrap_or_else(|error| panic!("{title}: {}", error.message))
        .0
}

/// Each finding as `code: message`, beside the citations its count read as `line written`.
fn findings(rules: &[ParsedRule]) -> Vec<(String, Vec<String>)> {
    evaluate(rules, &[], &facts())
        .into_iter()
        .map(|(diagnostic, counted)| {
            let counted = counted
                .iter()
                .map(|site| format!("{} {}", site.anchor.line, site.label))
                .collect();
            let message = format!("{}: {}", diagnostic.code.as_str(), diagnostic.message);
            (message, counted)
        })
        .collect()
}

const REMEDY: &str = "add or remove those citations until the count fits";

fn row(message: &str) -> String {
    format!("citation-cardinality: {message}; {REMEDY}")
}

fn sites(entries: &[&str]) -> Vec<String> {
    entries.iter().map(|entry| entry.to_string()).collect()
}

/// The findings `rules` give about `subject` alone.
fn about(rules: &[ParsedRule], subject: &str) -> Vec<(String, Vec<String>)> {
    let prefix = format!("citation-cardinality: {subject} ");
    findings(rules)
        .into_iter()
        .filter(|(message, _)| message.starts_with(&prefix))
        .collect()
}

/// §FS-rules.3.2.1, §FS-rules.7.3.1: `SRC-x` cited bare, at `.2` and at `.outcome` is one
/// declaration, and the row lists all three sites, so its list adds up to the sites; a subject
/// that cites no `SRC` is a zero count, which names no site.
#[test]
#[ignore = "red until #1367 counts distinct cited declarations; implement removes this"]
fn a_declaration_its_section_and_its_chapter_count_once() {
    let distinct = rule("RULE-d", "Each COUNTRY must cite at least 2 distinct SRC.");
    assert_eq!(
        findings(&[distinct]),
        [
            (
                row("COUNTRY-a cites 1 distinct SRC; RULE-d requires at least 2"),
                sites(&["5 \u{a7}SRC-x", "6 \u{a7}SRC-x.2", "7 \u{a7}SRC-x.outcome"])
            ),
            (
                row("COUNTRY-b cites 1 distinct SRC; RULE-d requires at least 2"),
                sites(&["5 \u{a7}SRC-y"])
            ),
            (
                row("COUNTRY-c cites 0 distinct SRC; RULE-d requires at least 2"),
                Vec::new()
            ),
        ]
    );
}

/// §FS-rules.3.2.1: one `distinct` covers the whole `or` list, so a `SRC` and a `DOC` are two.
#[test]
#[ignore = "red until #1367 counts distinct cited declarations; implement removes this"]
fn an_or_list_counts_across_its_kinds() {
    let distinct = rule(
        "RULE-d",
        "Each COUNTRY must cite exactly one distinct SRC or DOC.",
    );
    assert_eq!(about(std::slice::from_ref(&distinct), "COUNTRY-a"), []);
    assert_eq!(
        about(&[distinct], "COUNTRY-b"),
        [(
            row("COUNTRY-b cites 2 distinct DOC or SRC; RULE-d requires exactly one"),
            sites(&["5 \u{a7}SRC-y", "6 \u{a7}DOC-d"])
        )]
    );
}

/// §FS-rules.3.2.1: a subject in its own target set counts itself once, whether it cites itself
/// bare or at a chapter, beside the other declaration it cites.
#[test]
#[ignore = "red until #1367 counts distinct cited declarations; implement removes this"]
fn a_subject_in_its_own_target_set_counts_itself_once() {
    let exactly = rule("RULE-d", "Each SRC must cite exactly 2 distinct SRC.");
    assert_eq!(about(&[exactly], "SRC-z"), []);
    let at_most = rule("RULE-d", "Each SRC must cite at most 1 distinct SRC.");
    assert_eq!(
        findings(&[at_most]),
        [(
            row("SRC-z cites 2 distinct SRC; RULE-d requires at most 1"),
            sites(&["5 \u{a7}SRC-z", "6 \u{a7}SRC-z.outcome", "7 \u{a7}SRC-y"])
        )]
    );
}

/// §FS-rules.7.3, §FS-rules.7.3.1: `exactly one` and `at most 1` are bounds the form takes; a
/// zero count is `citation-cardinality` too, and names no site.
#[test]
#[ignore = "red until #1367 counts distinct cited declarations; implement removes this"]
fn exactly_one_and_at_most_one_bound_the_count() {
    assert_eq!(
        findings(&[rule(
            "RULE-d",
            "Each COUNTRY must cite exactly one distinct SRC."
        )]),
        [(
            row("COUNTRY-c cites 0 distinct SRC; RULE-d requires exactly one"),
            Vec::new()
        )]
    );
    assert_eq!(
        findings(&[rule(
            "RULE-m",
            "Each COUNTRY must cite at most 1 distinct SRC."
        )]),
        []
    );
}

/// §FS-rules.6: two distinct rules of one meaning are one finding naming both, while a distinct
/// rule and a plain rule of the same count are two constraints and two findings.
#[test]
#[ignore = "red until #1367 counts distinct cited declarations; implement removes this"]
fn the_counting_mode_is_part_of_the_deduplication_key() {
    let twin = [
        rule("RULE-a", "Each COUNTRY must cite at least 4 distinct SRC."),
        rule("RULE-b", "Each COUNTRY must cite at least 4 distinct SRC."),
    ];
    let merged = about(&twin, "COUNTRY-a");
    assert_eq!(merged.len(), 1, "{merged:?}");
    let message = &merged[0].0;
    assert!(message.contains("cites 1 distinct SRC"), "{message}");
    assert!(
        message.contains("RULE-a") && message.contains("RULE-b"),
        "{message}"
    );

    let both = [
        rule("RULE-a", "Each COUNTRY must cite at least 4 distinct SRC."),
        rule("RULE-p", "Each COUNTRY must cite at least 4 SRC."),
    ];
    let mut messages: Vec<String> = about(&both, "COUNTRY-a")
        .into_iter()
        .map(|(m, _)| m)
        .collect();
    messages.sort();
    assert_eq!(
        messages,
        [
            row("COUNTRY-a cites 1 distinct SRC; RULE-a requires at least 4"),
            row("COUNTRY-a cites SRC 3 times; RULE-p requires at least 4"),
        ]
    );
}

/// §FS-rules.3.2, §FS-rules.7.3: the plain count beside them keeps counting sites, with its row
/// and its counted citations as #1366 gives them. A control, green today.
#[test]
fn the_plain_count_still_counts_sites() {
    assert_eq!(
        about(
            &[rule("RULE-p", "Each COUNTRY must cite at least 4 SRC.")],
            "COUNTRY-a"
        ),
        [(
            row("COUNTRY-a cites SRC 3 times; RULE-p requires at least 4"),
            sites(&["5 \u{a7}SRC-x", "6 \u{a7}SRC-x.2", "7 \u{a7}SRC-x.outcome"])
        )]
    );
    assert_eq!(
        about(
            &[rule("RULE-p", "Each COUNTRY must cite at least 3 SRC.")],
            "COUNTRY-a"
        ),
        []
    );
}
