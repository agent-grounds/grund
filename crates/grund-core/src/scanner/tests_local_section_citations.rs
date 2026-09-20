//! Scanner ownership and grammar boundaries for declaration-local numeric citations
//! (§FS-check.1.1.8, §AR-scanner.2.3, §AR-scanner.2.4).

use super::*;
use crate::model::{Citation, Findings, Id};
use crate::testing::{numbered_config, test_root, write};

fn local_citations(findings: &Findings) -> Vec<&Citation> {
    findings
        .citations
        .iter()
        .filter(|citation| citation.text.starts_with('@') && !citation.text.starts_with("@FS-"))
        .collect()
}

#[test]
fn local_numeric_paths_use_marker_boundaries_and_markdown_body_ownership() {
    let root = test_root("local_numeric_paths_use_marker_boundaries_and_markdown_body_ownership");
    let path = root.join("docs/functional-spec/FS-001-alpha.md");
    write(
        &path,
        concat!(
            "# FS-001-alpha: Alpha\n\n",
            "Owned @2 and @2.1; unmarked 2.1; unsupported @2.goals and @2abc.\n",
            "Malformed @2..1 and @2... and @2..goals.\n",
            "Full @FS-001-alpha.2 and escaped <@>2.\n\n",
            "```text\n@2\n```\n\n",
            "## 2. Target\n\n### 2.1 Child\n\n",
            "# Plain chapter\n\nOwnerless @2.\n",
        ),
    );
    let mut config = numbered_config(root);
    config.marker = "@".into();
    config.rebuild_grammar().expect("configured marker grammar");
    let mut scans = Vec::new();
    for strict in [true, false] {
        config.strict = strict;
        let (findings, errors) = scan_tree(&config, Some(&path), true).expect("scan fixture");
        assert!(errors.is_empty(), "unexpected scan errors: {errors:?}");
        scans.push(findings);
    }
    let local = local_citations(&scans[0]);
    assert_eq!(
        local
            .iter()
            .map(|citation| citation.text.as_str())
            .collect::<Vec<_>>(),
        ["@2", "@2.1"],
        "only complete, owned, marked numeric paths become local edges"
    );
    let owner = Id {
        kind: "FS".into(),
        num: Some(1),
        slug: Some("alpha".into()),
    };
    assert!(local.iter().all(|citation| citation.id == owner));
    assert_eq!(
        local
            .iter()
            .map(|citation| citation.section.as_deref())
            .collect::<Vec<_>>(),
        [Some("2"), Some("2.1")]
    );
    assert_eq!(
        scans[0]
            .local_section_citation_candidates
            .iter()
            .map(|candidate| (candidate.text.as_str(), candidate.section.as_deref()))
            .collect::<Vec<_>>(),
        [
            ("@2.goals", None),
            ("@2abc", None),
            ("@2..1", None),
            ("@2...", None),
            ("@2..goals", None),
            ("@2", Some("2")),
        ],
        "unsupported tokens stay whole; the ownerless numeric candidate keeps its syntax"
    );
    assert_eq!(
        scans[0]
            .citations
            .iter()
            .filter(|citation| citation.text == "@FS-001-alpha.2")
            .count(),
        1,
        "full-ID precedence remains intact"
    );
    assert_eq!(
        local_citations(&scans[1])
            .iter()
            .map(|citation| (citation.text.as_str(), citation.section.as_deref()))
            .collect::<Vec<_>>(),
        [("@2", Some("2")), ("@2.1", Some("2.1"))],
        "strict mode does not change marker-gated local recognition"
    );
}

#[test]
fn source_local_paths_use_comment_boundaries_and_nearest_preceding_declaration() {
    let root =
        test_root("source_local_paths_use_comment_boundaries_and_nearest_preceding_declaration");
    let path = root.join("src/pair.rs");
    write(
        &path,
        concat!(
            "/// AR-001-first: First\n",
            "/// First owns \u{a7}1.\n",
            "///\n",
            "/// FS-002-second: Second\n",
            "/// Second owns \u{a7}2.1.\n",
            "///\n",
            "/// ## 2. Target\n",
            "/// ### 2.1 Child\n",
            "pub struct Pair;\n\n",
            "// Outside the doc-comment \u{a7}2.1.\n",
        ),
    );
    let config = numbered_config(root);

    let (findings, errors) = scan_tree(&config, Some(&path), true).expect("scan source fixture");
    assert!(errors.is_empty(), "unexpected scan errors: {errors:?}");
    let local = findings
        .citations
        .iter()
        .filter(|citation| citation.text.starts_with('§') && citation.id.num.is_some())
        .collect::<Vec<_>>();
    assert_eq!(
        local.len(),
        2,
        "the comment after the doc block has no owner"
    );
    assert_eq!(local[0].id.kind, "AR");
    assert_eq!(local[0].id.num, Some(1));
    assert_eq!(local[0].section.as_deref(), Some("1"));
    assert_eq!(local[1].id.kind, "FS");
    assert_eq!(local[1].id.num, Some(2));
    assert_eq!(local[1].section.as_deref(), Some("2.1"));
}
