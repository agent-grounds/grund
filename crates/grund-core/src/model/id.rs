//! Numeric identity and authored spelling are separate (§FS-config.3.2.8).

use std::cmp::Ordering;

/// A parsed ID under the effective grammar (§FS-config.3.2). Report rendering
/// preserves its authored number; allocation supplies its own width
/// (§FS-config.3.2.8). There is no `Display` implementation because rendering
/// requires the project's grammar.
#[derive(Debug, Clone)]
pub struct Id {
    pub kind: String,
    pub num: Option<u32>,
    pub slug: Option<String>,
    /// Authored numeric digits, absent on a synthetic or newly allocated ID.
    /// Presentation only: equality and ordering use `num` (§FS-config.3.2.8).
    pub number_spelling: Option<String>,
}

const LEGACY_ID_SENTINEL: char = '\0';

impl Id {
    /// Read numeric value and spelling together, retaining the existing u32
    /// acceptance boundary (§FS-config.3.2.8).
    pub(crate) fn parsed(kind: String, number: Option<&str>, slug: Option<String>) -> Option<Self> {
        Some(Self {
            kind,
            num: number.map(str::parse).transpose().ok()?,
            slug,
            number_spelling: number.map(str::to_string),
        })
    }

    /// Preserve an off-grammar declaration's exact spelling without widening
    /// the configured authoring grammar (§FS-config.3.2.5).
    pub(crate) fn legacy(kind: String, spelling: &str) -> Self {
        Self {
            kind,
            num: None,
            slug: Some(format!("{LEGACY_ID_SENTINEL}{spelling}")),
            number_spelling: None,
        }
    }

    pub(crate) fn legacy_spelling(&self) -> Option<&str> {
        self.slug
            .as_deref()
            .and_then(|slug| slug.strip_prefix(LEGACY_ID_SENTINEL))
    }

    /// Numeric identity and its established sort order ignore presentation
    /// metadata (§FS-config.3.2.8, §FS-declarations.checks.duplicate).
    fn identity(&self) -> (&str, Option<u32>, &Option<String>) {
        (&self.kind, self.num, &self.slug)
    }
}

/// Source digits do not affect numeric equality (§FS-config.3.2.8).
impl PartialEq for Id {
    fn eq(&self, other: &Self) -> bool {
        self.identity() == other.identity()
    }
}

/// Equality remains total across source spellings (§FS-config.3.2.8).
impl Eq for Id {}

/// All spellings share the established total order (§FS-config.3.2.8).
impl PartialOrd for Id {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Keep numeric ordering independent of authored width (§FS-config.3.2.8).
impl Ord for Id {
    fn cmp(&self, other: &Self) -> Ordering {
        self.identity().cmp(&other.identity())
    }
}
