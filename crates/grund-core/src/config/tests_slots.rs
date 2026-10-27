//! Test module: a kind's slots, derived from its form and its fields
//! (§AR-config.6.2, §FS-fields.record).

use super::rows::Origin;
use super::*;

fn kind(form: Form, fields: Vec<Field>) -> Kind {
    Kind {
        id_format: None,
        form,
        index: KindIndex::Default,
        origin: Origin::Local,
        fields,
        closed: Strength::May,
    }
}

/// §AR-config.6.2: a prose or rule kind with no field has no slot.
#[test]
fn a_kind_with_no_field_yields_no_slot() {
    for form in [Form::Prose, Form::Rule] {
        assert_eq!(kind(form, Vec::new()).slots().count(), 0);
    }
}

/// §AR-config.6.2: a value kind holds its values in numbered children.
#[test]
fn a_value_kind_yields_one_numbered_values_slot() {
    assert_eq!(
        kind(Form::Value, Vec::new()).slots().collect::<Vec<_>>(),
        vec![Slot {
            handle: Handle::Numbered,
            title: None,
            placement: None,
            presence: Strength::May,
            content: Content::Values,
        }]
    );
}

/// §FS-fields.v1: v1's value chapter is a `may` values field, on a prose or a
/// rule kind alike (§DF-kind-fields.2.4).
#[test]
fn a_value_chapter_yields_one_named_values_slot() {
    for form in [Form::Prose, Form::Rule] {
        let kind = kind(form, vec![Field::value_chapter("values".to_string())]);
        assert_eq!(
            kind.slots().collect::<Vec<_>>(),
            vec![Slot {
                handle: Handle::Named("values"),
                title: None,
                placement: None,
                presence: Strength::May,
                content: Content::Values,
            }]
        );
    }
}

/// §FS-fields.record: every field is a slot carrying its title, placement,
/// configured strength and content, in header order — placement is data on
/// the slot, never a reordering.
#[test]
fn fields_yield_slots_in_header_order_with_their_shape() {
    let statuses = vec!["Proposed".to_string(), "Accepted".to_string()];
    let kind = kind(
        Form::Prose,
        vec![
            Field {
                name: "terms".to_string(),
                title: Some("Terms".to_string()),
                presence: Strength::Warn,
                placement: Some(Placement::After("placement".to_string())),
                content: FieldContent::Prose,
            },
            Field {
                name: "placement".to_string(),
                title: None,
                presence: Strength::Must,
                placement: Some(Placement::First),
                content: FieldContent::Prose,
            },
            Field {
                name: "status".to_string(),
                title: None,
                presence: Strength::Should,
                placement: Some(Placement::Last),
                content: FieldContent::OneOf(statuses.clone()),
            },
        ],
    );
    assert_eq!(
        kind.slots().collect::<Vec<_>>(),
        vec![
            Slot {
                handle: Handle::Named("terms"),
                title: Some("Terms"),
                placement: Some(&Placement::After("placement".to_string())),
                presence: Strength::Warn,
                content: Content::Prose,
            },
            Slot {
                handle: Handle::Named("placement"),
                title: None,
                placement: Some(&Placement::First),
                presence: Strength::Must,
                content: Content::Prose,
            },
            Slot {
                handle: Handle::Named("status"),
                title: None,
                placement: Some(&Placement::Last),
                presence: Strength::Should,
                content: Content::OneOf(&statuses),
            },
        ]
    );
}
