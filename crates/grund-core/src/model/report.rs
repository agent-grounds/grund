//! The report a run produces, in both its shapes (§AR-system.2.2): the internal
//! `CheckReport` the rules fill, and the published `Report` an embedder reads
//! (§FS-errors.2.1, §FS-errors.5). The published three came down out of the
//! api's contract file when §AR-system.2.9 became a module, because the editor
//! snapshot of §FS-lsp carries a `Report` and the show refusal of §FS-errors.5.2
//! carries a `FindingSite`, and both of those are the queries' (§AR-system.4).
//! Only the records are here; the conversion between them needs a `Config` to
//! spell a path with and stays in `api/report.rs`.

use std::path::PathBuf;

/// A secondary location attached to a diagnostic — e.g. the other declaration in a
/// duplicate pair, or the citation that pointed at a missing section (§FS-errors.2.1).
#[derive(Clone)]
pub(crate) struct Site {
    pub(crate) path: PathBuf,
    pub(crate) line: usize,
}

/// One finding in the located-finding shape of §FS-errors.2.1: a fixed `code`, the
/// `path:line` it occurred at, the message text, and any cross-reference `sites`.
/// `column` is the 1-based start column of the offending token when the finding
/// concerns a specific citation, so a consumer can anchor on that token rather
/// than the first one on the line (§FS-lsp.1.1.1); it is `None` for line-anchored
/// findings.
#[derive(Clone)]
pub(crate) struct Diagnostic {
    pub(crate) code: &'static str,
    pub(crate) path: Option<PathBuf>,
    pub(crate) line: Option<usize>,
    pub(crate) column: Option<usize>,
    pub(crate) message: String,
    pub(crate) sites: Vec<Site>,
}

/// The outcome of `check`: errors and warnings, kept apart so the exit code keys
/// off errors only (§FS-check.2, §FS-check.4) and the printed order is fixed
/// (§FS-errors.4.1, §FS-non-goals.9). `suggestions` is the third, non-severity
/// advisory channel (§FS-check.2.3, §DF-citation-directions.2.3): the
/// `should` / `should-not` citation-direction findings, withheld from the
/// default run and surfaced only under `--suggestions`. It never affects the
/// exit code.
#[derive(Default)]
pub(crate) struct CheckReport {
    pub(crate) errors: Vec<Diagnostic>,
    pub(crate) warnings: Vec<Diagnostic>,
    pub(crate) suggestions: Vec<Diagnostic>,
}

/// A secondary location attached to a published [`Finding`] (§FS-errors.2.1,
/// §FS-errors.5): the twin of [`Site`] with its path already spelled the way the
/// report spells it, because a consumer of the published shape has no `Config`
/// to render one with.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FindingSite {
    pub path: String,
    pub line: usize,
}

/// One published finding: the located-finding shape of §FS-errors.2.1 with its
/// paths rendered — the twin of [`Diagnostic`], and what `check` returns to an
/// embedder rather than prints (§AR-bindings.2).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Finding {
    pub severity: &'static str,
    pub code: &'static str,
    pub path: Option<String>,
    pub line: Option<usize>,
    /// 1-based start column of the offending citation when the finding concerns
    /// a specific token, so an LSP can anchor on that token rather than the first
    /// citation on the line (§FS-lsp.1.1.1). `None` for line-anchored findings.
    pub column: Option<usize>,
    pub message: String,
    pub sites: Vec<FindingSite>,
}

/// The published outcome of `check` — the twin of [`CheckReport`], kept in the
/// same three channels so the exit code still keys off errors alone
/// (§FS-check.2, §FS-check.2.3).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Report {
    pub errors: Vec<Finding>,
    pub warnings: Vec<Finding>,
    /// The non-severity advisory channel (§FS-check.2.3): citation-direction
    /// `should` / `should-not` findings, populated only when
    /// `CheckOpts::include_suggestions` is set. Never affects the exit code.
    pub suggestions: Vec<Finding>,
}
