//! A kind's declared fields (§FS-fields.declare.field): the named chapters a
//! declaration of the kind carries, where each stands, what its heading may
//! say, and how strongly it is held. Both readers lower into this one record
//! (§FS-fields.record): v2 from `[schema.kinds.<KIND>.fields.<name>]`, v1 from
//! `value_chapter` alone (§FS-fields.v1).

use super::project::Strength;

/// §FS-fields.declare.field: one field of a kind, in header order.
#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    /// The section name the field is matched by (§FS-fields.identity.name).
    pub name: String,
    /// The display name its chapter must show, byte for byte
    /// (§FS-fields.identity.match); `None` admits any.
    pub title: Option<String>,
    /// How strongly a declaration is held to it (§FS-fields.strengths.table).
    pub presence: Strength,
    pub placement: Option<Placement>,
    pub content: FieldContent,
}

/// §FS-fields.order: where a field stands among a declaration's direct sections.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Placement {
    First,
    Last,
    /// Directly after the named field (§FS-fields.order.after).
    After(String),
}

/// §FS-fields.content: what a field's chapter holds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FieldContent {
    Prose,
    /// The value chapter of §FS-values.2.5 (§FS-fields.content.values).
    Values,
    /// A scalar: its display name is one of these (§FS-fields.content.one-of).
    OneOf(Vec<String>),
    /// The file's front matter rather than a chapter, its keys the field's
    /// sections (§FS-fields.front-matter, §FS-fields.front-matter.record).
    FrontMatter(FrontMatterShape),
}

/// §FS-fields.front-matter: the one field name with a meaning of its own, on a
/// kind that declares it (§FS-fields.declare.field).
pub const FRONT_MATTER: &str = "front-matter";

/// §FS-fields.front-matter.record: what the front-matter slot carries besides
/// its presence: its own `closed`, and its keys in header order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrontMatterShape {
    /// Whether a key the field does not declare is reported (§FS-fields.front-matter.config).
    pub closed: Strength,
    pub keys: Vec<FrontMatterKey>,
}

/// §FS-fields.front-matter.config: one `[…fields.front-matter.keys.<key>]`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrontMatterKey {
    /// The top-level key it declares, a section name (§FS-fields.front-matter.keys).
    pub name: String,
    /// Whether the block must have the key (§FS-fields.front-matter.findings).
    pub presence: Strength,
    /// The values its one-line value may take, `None` for any value
    /// (§FS-fields.front-matter.values).
    pub one_of: Option<Vec<String>>,
}

impl Field {
    /// §FS-fields.v1: what v1's `value_chapter = "<name>"` lowers to — a `may`
    /// values field and nothing else.
    pub(crate) fn value_chapter(name: String) -> Self {
        Self {
            name,
            title: None,
            presence: Strength::May,
            placement: None,
            content: FieldContent::Values,
        }
    }

    /// §FS-fields.front-matter: the field's front-matter shape, where it is the
    /// front-matter field.
    pub fn front_matter(&self) -> Option<&FrontMatterShape> {
        match &self.content {
            FieldContent::FrontMatter(shape) => Some(shape),
            _ => None,
        }
    }

    /// §FS-fields.rules: the name a chapter-presence rule would count this
    /// field's chapter by — its `title`, or its name where it has none.
    pub(crate) fn expressed_name(&self) -> &str {
        self.title.as_deref().unwrap_or(&self.name)
    }
}
