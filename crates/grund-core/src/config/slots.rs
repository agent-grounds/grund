//! §AR-config.6.2: a kind's shape as a sequence of slots, derived from its
//! `form` and its fields rather than stored apart, so there is no second
//! hand-maintained model of what a declaration of the kind holds
//! (§FS-fields.record).

use super::fields::{FieldContent, FrontMatterShape, Placement};
use super::project::Strength;
use super::rows::{Form, Kind};

/// One slot of a kind's shape (§AR-config.6.2, §FS-fields.record).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Slot<'a> {
    pub handle: Handle<'a>,
    /// The display name the slot's chapter must show (§FS-fields.identity.match).
    pub title: Option<&'a str>,
    /// Where the slot stands; data on the slot, never a reordering (§FS-fields.order).
    pub placement: Option<&'a Placement>,
    /// The configured strength (§FS-fields.strengths.table). A form says
    /// nothing about it, so a slot derived from one is `May`, the weakest.
    pub presence: Strength,
    pub content: Content<'a>,
}

/// How a slot is addressed inside a declaration: by a named section, or by the
/// declaration's own numbered children (§FS-values.2.5, §FS-values.2.1).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Handle<'a> {
    Named(&'a str),
    Numbered,
}

/// What a slot holds: prose, value roots, or one of a closed set of titles.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Content<'a> {
    Prose,
    Values,
    OneOf(&'a [String]),
    /// The file's front matter, its keys the slot's sections (§FS-fields.front-matter.record).
    FrontMatter(&'a FrontMatterShape),
}

impl Kind {
    /// §AR-config.6.2, §FS-fields.record: the kind's slots in declared order.
    /// A `Value` kind yields its numbered children, holding values; every
    /// field yields a `Named` slot, in header order.
    pub fn slots(&self) -> impl Iterator<Item = Slot<'_>> {
        let numbered = (self.form == Form::Value).then_some(Slot {
            handle: Handle::Numbered,
            title: None,
            placement: None,
            presence: Strength::May,
            content: Content::Values,
        });
        let fields = self.fields.iter().map(|field| Slot {
            handle: Handle::Named(&field.name),
            title: field.title.as_deref(),
            placement: field.placement.as_ref(),
            presence: field.presence,
            content: match &field.content {
                FieldContent::Prose => Content::Prose,
                FieldContent::Values => Content::Values,
                FieldContent::OneOf(values) => Content::OneOf(values),
                FieldContent::FrontMatter(shape) => Content::FrontMatter(shape),
            },
        });
        numbered.into_iter().chain(fields)
    }
}
