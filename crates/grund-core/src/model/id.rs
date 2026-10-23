//! Numeric identity and authored spelling are separate (§FS-config.3.2.8).

/// Numeric identity under the effective grammar (§FS-config.3.2.8). Authored
/// spelling belongs to the declaration or citation context, so the public
/// record keeps its three fields (§FS-distribution.3.1.2).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub struct Id {
    pub kind: String,
    pub num: Option<u32>,
    pub slug: Option<String>,
}

const LEGACY_ID_SENTINEL: char = '\0';

impl Id {
    /// Read numeric identity with the existing u32 acceptance boundary
    /// (§FS-config.3.2.8).
    pub(crate) fn parsed(kind: String, number: Option<&str>, slug: Option<String>) -> Option<Self> {
        Some(Self {
            kind,
            num: number.map(str::parse).transpose().ok()?,
            slug,
        })
    }

    /// Preserve an off-grammar declaration's exact spelling without widening
    /// the configured authoring grammar (§FS-config.3.2.5).
    pub(crate) fn legacy(kind: String, spelling: &str) -> Self {
        Self {
            kind,
            num: None,
            slug: Some(format!("{LEGACY_ID_SENTINEL}{spelling}")),
        }
    }

    pub(crate) fn legacy_spelling(&self) -> Option<&str> {
        self.slug
            .as_deref()
            .and_then(|slug| slug.strip_prefix(LEGACY_ID_SENTINEL))
    }
}
