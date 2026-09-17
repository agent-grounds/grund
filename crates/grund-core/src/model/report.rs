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
/// than the first one on the line (§FS-lsp.1.1); it is `None` for line-anchored
/// findings.
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
/// (§FS-errors.4, §FS-non-goals.9). `suggestions` is the third, non-severity
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
