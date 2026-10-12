use std::path::PathBuf;

use super::records::Id;

/// One section-like heading rejected from a declaration's shared coordinate
/// maps because it lies beyond that declaration's body (§FS-declarations.section.map).
pub struct SectionHeadingOutsideDeclaration {
    pub file: PathBuf,
    pub line: usize,
    pub path: String,
}

/// One body-owned Markdown ATX heading that does not participate in the
/// knowledge graph, together with the scanner's deterministic repair guidance
/// (§FS-declarations.checks.unmarked-heading, §AR-scanner.2.2.7).
pub struct UnmarkedHeading {
    pub file: PathBuf,
    pub line: usize,
    /// One-based byte column of the first `#`, for the complete-heading LSP
    /// range required by §FS-lsp.1.1.1.
    pub column: usize,
    pub heading: String,
    pub heading_level: usize,
    pub title: String,
    pub owner: Id,
    pub suggested_path: String,
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
/// after it, without parsing as an ID (§FS-declarations.checks.declaration-near-miss.1). `text` is the token as
/// written, so the finding can quote it back beside `format`, the candidate
/// kind's effective template that it missed.
pub struct NearMissHeading {
    pub file: PathBuf,
    pub line: usize,
    pub text: String,
    pub format: String,
}

/// A declaration heading of the stub shape that the two stub tests of
/// §FS-declarations.stubs.shape read differently, noted by the scanner where it
/// read the heading (§AR-scanner.4.7) and reported as `stub-shape`
/// (§FS-declarations.checks.stub-shape). Nothing else reads it: the heading is
/// read as the `docs` test reads it until 0.19.0.
pub(crate) struct StubShapeNote {
    pub(crate) file: PathBuf,
    pub(crate) line: usize,
    pub(crate) id: Id,
    pub(crate) reading: StubShapeReading,
}

/// Which way the two tests disagree on one heading.
pub(crate) enum StubShapeReading {
    /// A stub today only because a directory on its path is named `docs`; outside
    /// every kind's home, so a declaration from 0.19.0.
    StubOnlyByDocs,
    /// A declaration titled with a link today; in a kind's home, so from 0.19.0 a
    /// stub to `path`, the link's destination as written.
    StubByHome { path: String },
}
