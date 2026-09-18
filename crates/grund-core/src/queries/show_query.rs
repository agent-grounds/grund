//! What a show query is asked and what it refuses with (§AR-system.2.7): the
//! options a caller passes (§FS-distribution.3.1) and the typed refusal the
//! resolver raises (§FS-errors.5).
//!
//! These sat in the api's contract file while `queries/show.rs` raised the
//! refusal and `queries/batch.rs` read the options upward (§AR-system.4). A
//! record belongs to the component that produces it, and both of these are
//! produced here: the mode selects which slice of a declaration body the query
//! returns (§FS-show.2), and the refusal is raised by the slicer itself. They
//! stay `pub` — an embedder constructs a `ShowOpts` and matches a
//! `ShowQueryError` exactly where it did before (§AR-core-module-layout.2).

use std::path::PathBuf;

use crate::model::{FindingSite, ShowRenderMode};

/// Options for programmatic declaration reads through `show`
/// (§FS-distribution.3.0, §FS-distribution.3.1).
#[derive(Clone)]
pub struct ShowOpts {
    pub path: PathBuf,
    pub section: Option<String>,
    pub mode: ShowMode,
    pub format: ShowFormat,
}

impl Default for ShowOpts {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            section: None,
            mode: ShowMode::Lead,
            format: ShowFormat::Text,
        }
    }
}

/// How much of a declaration body the query returns (§FS-show.2).
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum ShowMode {
    Brief,
    Lead,
    Toc,
    Full,
}

impl ShowMode {
    pub(crate) fn render_mode(self) -> ShowRenderMode {
        match self {
            ShowMode::Brief => ShowRenderMode::Brief,
            ShowMode::Lead => ShowRenderMode::Default,
            ShowMode::Toc => ShowRenderMode::Toc,
            ShowMode::Full => ShowRenderMode::Full,
        }
    }
}

/// Which shape the answer carries (§FS-show.3): the plain body, the Markdown
/// form that keeps its cross-reference links, or the JSON object.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum ShowFormat {
    Text,
    Markdown,
    Json,
}

/// A failed ID query whose message names sites the JSON diagnostic can also
/// carry (§FS-errors.5): the two-homes `ambiguous` refusal and the
/// `ambiguous-section` refusal. Raised from `queries/show.rs` and downcast by
/// both printers, so `sites` never needs a second parse of `message`.
/// `Display` is `message` verbatim — the text form is unchanged by this type.
#[derive(Clone, Debug)]
pub struct ShowQueryError {
    pub code: &'static str,
    pub message: String,
    pub sites: Vec<FindingSite>,
}

impl std::fmt::Display for ShowQueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ShowQueryError {}
