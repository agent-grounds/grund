//! §AR-config.6.2: a kind's shape as a sequence of slots, derived from its
//! `form` rather than stored, so there is no second hand-maintained model of
//! what a declaration of the kind holds.

use super::rows::{Form, Kind};

/// One slot of a kind's shape (§AR-config.6.2).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Slot<'a> {
    pub handle: Handle<'a>,
    pub presence: Presence,
    pub content: Content<'a>,
}

/// How a slot is addressed inside a declaration: by a named section, or by the
/// declaration's own numbered children (§FS-values.2.5, §FS-values.2.1).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Handle<'a> {
    Named(&'a str),
    Numbered,
}

/// How strongly a declaration of the kind is held to carry the slot. A form
/// says nothing about it, so a slot derived from one is `May`, the weakest.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Presence {
    Must,
    Should,
    May,
}

/// What a slot holds: prose, value roots, or one of a closed set of titles.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Content<'a> {
    Prose,
    Values,
    OneOf(&'a [String]),
}

impl Kind {
    /// §AR-config.6.2: the kind's slots in declared order. A `Prose` or `Rule`
    /// kind yields none, and a `Value` kind yields one — its value chapter as a
    /// `Named` handle with `Values` content, or `Numbered` when it has none.
    pub fn slots(&self) -> impl Iterator<Item = Slot<'_>> {
        let value = match &self.form {
            Form::Value { chapter } => Some(Slot {
                handle: chapter.as_deref().map_or(Handle::Numbered, Handle::Named),
                presence: Presence::May,
                content: Content::Values,
            }),
            Form::Prose | Form::Rule { .. } => None,
        };
        value.into_iter()
    }
}
