//! What a count hands the checker beside its finding (§AR-rules.5, §FS-rules.7.3.1): one entry per
//! citation the count read, the fact's own `SiteMeta` with its path, line, column and the citation
//! as written, and nothing beside a zero count or any other finding.

use super::super::facts::{
    Completeness, FactHeader, NodeKey, NodeMeta, RuleFacts, SiteKey, SiteMeta,
};
use super::super::{
    Cardinality, ParsedRule, RuleAnchor, RuleLevel, RulePolarity, RuleRelation, RuleSubject,
    RuleTargets, TargetMode,
};
use super::evaluate;
use std::collections::BTreeMap;

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

/// A snapshot built one fact at a time.
struct Facts(RuleFacts);

impl Facts {
    fn new() -> Self {
        Self(RuleFacts {
            header: FactHeader {
                schema: 1,
                project: "test-project".into(),
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

    fn declare(&mut self, node: &str, kind: &str, label: &str, path: &str) -> &mut Self {
        self.0.decl.push((key(node), kind.into()));
        self.0.nodes.insert(
            key(node),
            NodeMeta {
                label: label.into(),
                anchor: anchor(path, 1, None),
            },
        );
        self
    }

    fn chapter(&mut self, node: &str, parent: &str, label: &str, coordinate: &str, line: usize) {
        let path = self.0.nodes[&key(parent)].anchor.path.clone();
        self.0
            .chapter
            .push((key(node), coordinate.into(), Some(coordinate.into())));
        self.0.contains.push((key(parent), key(node)));
        self.0.nodes.insert(
            key(node),
            NodeMeta {
                label: label.into(),
                anchor: anchor(&path, line, None),
            },
        );
    }

    /// A citation of `target` written at `line:column` of `from`'s file, inside every unit of
    /// `units` (the declaration first).
    fn cite(&mut self, units: &[&str], target: &str, line: usize, column: usize, written: &str) {
        let site = SiteKey(format!("site-{}", self.0.cites.len()));
        let path = self.0.nodes[&key(units[0])].anchor.path.clone();
        let immediate = units.last().expect("a unit");
        self.0
            .cites
            .push((site.clone(), key(immediate), key(target)));
        for unit in units {
            self.0.site_in.push((site.clone(), key(unit)));
        }
        self.0.sites.insert(
            site,
            SiteMeta {
                label: written.into(),
                anchor: anchor(&path, line, Some(column)),
            },
        );
    }
}

fn rule(subject: RuleSubject, relation: RuleRelation, mode: TargetMode, kind: &str) -> ParsedRule {
    ParsedRule {
        origin: "RULE-count".into(),
        anchor: anchor("docs/rules/RULE-count.md", 1, None),
        subject,
        level: RuleLevel::Required,
        polarity: RulePolarity::Positive,
        relation,
        targets: RuleTargets::Kinds {
            values: vec![kind.into()],
            mode,
        },
        cardinality: Cardinality {
            minimum: None,
            maximum: Some(1),
        },
    }
}

fn counted(path: &str, line: usize, column: usize, written: &str) -> SiteMeta {
    SiteMeta {
        label: format!("\u{a7}{written}"),
        anchor: anchor(path, line, Some(column)),
    }
}

/// Each finding's message and what its count read.
fn findings(rule: ParsedRule, facts: &RuleFacts) -> Vec<(String, Vec<SiteMeta>)> {
    evaluate(&[rule], &[], facts)
        .into_iter()
        .map(|(diagnostic, counted)| (diagnostic.message, counted))
        .collect()
}

/// §FS-rules.7.3.1, §FS-rules.7.4: an inbound count reads every citing citation, with no cap,
/// each in the file it is written in.
#[test]
fn twelve_citers_are_twelve_counted_citations() {
    let mut facts = Facts::new();
    facts.declare("req", "REQ", "REQ-a", "docs/req.md");
    let mut expected = Vec::new();
    for n in 1..=12 {
        let (node, path) = (format!("fs{n}"), format!("docs/fs/FS-{n:02}.md"));
        facts.declare(&node, "FS", &format!("FS-{n:02}"), &path);
        facts.cite(&[&node], "req", 3, 7, "\u{a7}REQ-a");
        expected.push(counted(&path, 3, 7, "REQ-a"));
    }
    let subject = RuleSubject::Kind("REQ".into(), None);
    let found = findings(
        rule(
            subject,
            RuleRelation::BeCitedBy,
            TargetMode::Aggregate,
            "FS",
        ),
        &facts.0,
    );
    assert_eq!(found.len(), 1);
    assert!(found[0].0.starts_with("REQ-a is cited by FS 12 times"));
    assert_eq!(found[0].1, expected);
}

/// §FS-rules.7.3.1: an under-count above zero names what it read, and a zero count reads nothing.
#[test]
fn an_under_count_above_zero_names_its_citations_and_zero_names_none() {
    let mut facts = Facts::new();
    facts
        .declare("req", "REQ", "REQ-a", "docs/req.md")
        .declare("one", "FS", "FS-one", "docs/one.md")
        .declare("none", "FS", "FS-none", "docs/none.md");
    facts.cite(&["one"], "req", 4, 2, "\u{a7}REQ-a");
    let mut two = rule(
        RuleSubject::Kind("FS".into(), None),
        RuleRelation::Cite,
        TargetMode::Aggregate,
        "REQ",
    );
    two.cardinality = Cardinality {
        minimum: Some(2),
        maximum: Some(2),
    };
    let found = findings(two, &facts.0);
    assert_eq!(
        found,
        vec![
            (
                "FS-one cites REQ 1 times; RULE-count requires exactly 2; add or remove those citations until the count fits".into(),
                vec![counted("docs/one.md", 4, 2, "REQ-a")],
            ),
            (
                "FS-none cites REQ 0 times; RULE-count requires exactly 2; add or remove those citations until the count fits".into(),
                Vec::new(),
            ),
        ]
    );
}

/// §FS-rules.7.3.1: a chapter subject's count reads only the citations written inside it.
#[test]
fn a_chapter_subject_reads_only_the_citations_inside_it() {
    let mut facts = Facts::new();
    facts.declare("req", "REQ", "REQ-a", "docs/req.md").declare(
        "fs",
        "FS",
        "FS-one",
        "docs/one.md",
    );
    facts.chapter("needs", "fs", "FS-one.needs", "needs", 5);
    facts.cite(&["fs"], "req", 3, 1, "\u{a7}REQ-a");
    facts.cite(&["fs", "needs"], "req", 7, 1, "\u{a7}REQ-a");
    facts.cite(&["fs", "needs"], "req", 8, 1, "\u{a7}REQ-a");
    let subject = RuleSubject::ChapterOfKind {
        kind: "FS".into(),
        name: "needs".into(),
        whose: None,
    };
    let found = findings(
        rule(subject, RuleRelation::Cite, TargetMode::Aggregate, "REQ"),
        &facts.0,
    );
    assert_eq!(found.len(), 1);
    assert_eq!(
        found[0].1,
        vec![
            counted("docs/one.md", 7, 1, "REQ-a"),
            counted("docs/one.md", 8, 1, "REQ-a"),
        ]
    );
}

/// §FS-rules.7.3.1: a `cite each` row reads only its own target's citations, a citation of that
/// target's section among them (§FS-rules.5.1), and not another target's.
#[test]
fn a_cite_each_row_reads_only_its_own_targets_citations() {
    let mut facts = Facts::new();
    facts
        .declare("a", "REQ", "REQ-a", "docs/a.md")
        .declare("b", "REQ", "REQ-b", "docs/b.md")
        .declare("fs", "FS", "FS-one", "docs/one.md");
    facts.chapter("a1", "a", "REQ-a.1", "1", 3);
    facts.cite(&["fs"], "a", 3, 1, "\u{a7}REQ-a");
    facts.cite(&["fs"], "b", 4, 1, "\u{a7}REQ-b");
    facts.cite(&["fs"], "a1", 5, 9, "\u{a7}REQ-a.1");
    let found = findings(
        rule(
            RuleSubject::Kind("FS".into(), None),
            RuleRelation::Cite,
            TargetMode::PerTarget,
            "REQ",
        ),
        &facts.0,
    );
    assert_eq!(found.len(), 1);
    assert!(found[0].0.starts_with("FS-one cites REQ-a 2 times"));
    assert_eq!(
        found[0].1,
        vec![
            counted("docs/one.md", 3, 1, "REQ-a"),
            counted("docs/one.md", 5, 9, "REQ-a.1"),
        ]
    );
}
