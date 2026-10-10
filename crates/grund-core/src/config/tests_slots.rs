//! Test module: a kind's slots, derived from its form (§AR-config.6.2).

use super::*;

fn kind(form: Form) -> Kind {
    Kind {
        id_format: None,
        form,
        index: KindIndex::Default,
        origin: Origin::Local,
    }
}

/// §AR-config.6.2: a prose kind has no slot.
#[test]
fn a_prose_kind_yields_no_slot() {
    assert_eq!(kind(Form::Prose).slots().count(), 0);
}

/// §AR-config.6.2: a value kind with a chapter yields that chapter, named, holding values.
#[test]
fn a_value_kind_with_a_chapter_yields_one_named_values_slot() {
    let kind = kind(Form::Value {
        chapter: Some("values".to_string()),
    });
    assert_eq!(
        kind.slots().collect::<Vec<_>>(),
        vec![Slot {
            handle: Handle::Named("values"),
            presence: Presence::May,
            content: Content::Values,
        }]
    );
}

/// §AR-config.6.2: a value kind with no chapter holds its values in numbered children.
#[test]
fn a_value_kind_without_a_chapter_yields_one_numbered_values_slot() {
    let kind = kind(Form::Value { chapter: None });
    assert_eq!(
        kind.slots().collect::<Vec<_>>(),
        vec![Slot {
            handle: Handle::Numbered,
            presence: Presence::May,
            content: Content::Values,
        }]
    );
}

/// §AR-config.6.2: a rule kind has no slot, even one that names a value chapter.
#[test]
fn a_rule_kind_yields_no_slot() {
    for value_chapter in [None, Some("values".to_string())] {
        assert_eq!(kind(Form::Rule { value_chapter }).slots().count(), 0);
    }
}
