use std::path::PathBuf;

use super::records::Id;

/// One section-like heading rejected from a declaration's shared coordinate
/// maps because it lies beyond that declaration's body (§AR-scanner.2.4.1).
#[non_exhaustive]
#[derive(Clone)]
pub struct SectionHeadingOutsideDeclaration {
    pub file: PathBuf,
    pub line: usize,
    pub path: String,
}

/// One body-owned Markdown ATX heading that does not participate in the
/// knowledge graph, together with the scanner's deterministic repair guidance
/// (§FS-declarations.checks.unmarked-heading, §AR-scanner.2.2.7).
#[non_exhaustive]
#[derive(Clone)]
pub struct UnmarkedHeading {
    pub file: PathBuf,
    pub line: usize,
    /// One-based byte column of the first `#`, for the complete-heading LSP
    /// range required by §FS-lsp.1.1.
    pub column: usize,
    pub heading: String,
    pub heading_level: usize,
    pub title: String,
    pub owner: Id,
    /// The concrete owner's heading line, so field guidance uses that site's
    /// depth even for duplicate IDs (§FS-declarations.checks.unmarked-heading).
    pub(crate) owner_line: usize,
    pub suggested_path: String,
    /// For a dotted heading whose suggested coordinate has a usable parent, the
    /// relative heading that declares it (§FS-declarations.checks.unmarked-heading).
    pub relative_hint: Option<RelativeHint>,
    /// For a heading whose complete path lacks only a title, has a `_`, or is
    /// purely numeric before a `:`, and whose repair would declare a section,
    /// that fault and the repairing heading (§FS-declarations.checks.unmarked-heading).
    pub(crate) fault_hint: Option<PathFaultHint>,
}

/// The clause an unmarked heading whose path is already in place appends: the
/// authored path, which faults it has, and the heading that repairs them with
/// the path kept (§FS-declarations.checks.unmarked-heading).
#[derive(Clone)]
pub(crate) struct PathFaultHint {
    pub(crate) path: String,
    pub(crate) missing_title: bool,
    pub(crate) underscore: bool,
    /// A purely numeric path written with the `:` a name-bearing path takes.
    pub(crate) colon: bool,
    pub(crate) heading: String,
    /// The path the shown heading declares, which a later heading of the same
    /// body is not shown again.
    pub(crate) corrected: String,
}

/// The clause a dotted unmarked heading's message appends: the parent's path
/// (empty for the declaration) and the relative heading written under it
/// (§FS-declarations.checks.unmarked-heading).
#[non_exhaustive]
#[derive(Clone)]
pub struct RelativeHint {
    pub parent: String,
    pub heading: String,
}

/// A well-formed relative heading that declares nothing because no usable
/// parent sits one level above it (§FS-declarations.checks.relative-section-parent).
/// `written` is the `#` run, a space and the `.<component>`; `under` the path of
/// the nearest usable heading shallower than the parent it needed, empty for
/// the declaration. `column` and `heading` are the authored heading, the span an
/// editor underlines (§FS-lsp.1.1).
#[non_exhaustive]
#[derive(Clone)]
pub struct RelativeSectionParent {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub heading: String,
    pub owner: Id,
    pub written: String,
    pub needed_level: usize,
    pub under: String,
}

/// A fence-filtered Markdown heading awaiting body ownership and coordinate
/// assignment after all declarations and sections in its file are known
/// (§AR-scanner.2.2.7).
pub(crate) struct UnmarkedHeadingCandidate {
    pub(crate) file: PathBuf,
    pub(crate) line: usize,
    pub(crate) column: usize,
    pub(crate) heading: String,
    pub(crate) heading_level: usize,
    pub(crate) title: String,
}

/// One heading that opens with a configured kind and the literal an ID puts
/// after it, without parsing as an ID (§FS-declarations.checks.declaration-near-miss). `text` is the token as
/// written, so the finding can quote it back beside `format`, the candidate
/// kind's effective template that it missed.
#[non_exhaustive]
#[derive(Clone)]
pub struct NearMissHeading {
    pub file: PathBuf,
    pub line: usize,
    pub text: String,
    pub format: String,
}
