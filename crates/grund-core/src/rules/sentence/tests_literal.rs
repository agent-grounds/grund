//! Literal matching and semantic identity agree without changing display
//! spelling (§FS-config.3.2.8, §FS-rules.6) or exact subjects (§AR-rules.2).

use super::DeclarationSelector;
use crate::grammar::{Grammar, GrammarKind};
use std::cmp::Ordering;

fn grammar(format: &str) -> Grammar {
    let kinds = ["ADR", "GOAL"].map(|name| GrammarKind {
        name: name.into(),
        format: None,
    });
    Grammar::build(format, &kinds, "[0-9]+", "[a-z][a-z0-9-]*", ".", true, &[]).unwrap()
}

fn assert_identity_agreement(selectors: &[DeclarationSelector]) {
    for (index, left) in selectors.iter().enumerate() {
        for (other, right) in selectors.iter().enumerate() {
            let equivalent = index == other || (index < 3 && other < 3);
            assert_eq!(left == right, equivalent, "{left} vs {right}");
            assert_eq!(left.cmp(right) == Ordering::Equal, equivalent);
            assert_eq!(left.partial_cmp(right), Some(left.cmp(right)));
            assert_eq!(left.cmp(right), right.cmp(left).reverse());
            assert_eq!(left.matches(&right.to_string()), equivalent);
        }
    }
}

#[test]
fn numeric_equality_ordering_and_matching_share_the_grammar_selected_identity() {
    for (format, labels) in [
        (
            "{kind}-x{number}v2-{slug}",
            [
                "ADR-x1v2-alpha",
                "ADR-x001v2-alpha",
                "ADR-x0001v2-alpha",
                "ADR-x0002v2-alpha",
                "ADR-x0001v2-beta",
                "GOAL-x0001v2-alpha",
            ],
        ),
        (
            "{number}@{kind}@{slug}",
            [
                "1@ADR@alpha",
                "001@ADR@alpha",
                "0001@ADR@alpha",
                "0002@ADR@alpha",
                "0001@ADR@beta",
                "0001@GOAL@alpha",
            ],
        ),
    ] {
        let grammar = grammar(format);
        let selectors = labels.map(|label| DeclarationSelector::parsed(label.into(), &grammar));
        for (label, selector) in labels.iter().zip(&selectors) {
            assert_eq!(selector.to_string(), *label);
        }
        assert_identity_agreement(&selectors);
    }
}

#[test]
fn nonnumeric_and_producer_neutral_subjects_keep_exact_identity() {
    let grammar = grammar("{kind}-{slug}");
    let selectors = [
        DeclarationSelector::parsed("ADR-alpha".into(), &grammar),
        DeclarationSelector::from("ADR-alpha"),
        DeclarationSelector::from("ADR-alpha"),
        DeclarationSelector::parsed("ADR-beta".into(), &grammar),
        DeclarationSelector::from("ADR-001"),
        DeclarationSelector::from("ADR-0001"),
    ];
    assert_identity_agreement(&selectors);
}
