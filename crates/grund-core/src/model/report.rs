//! The report a run produces, in both its shapes (§AR-system.2.2): the internal
//! `CheckReport` the rules fill, and the published `Report` an embedder reads
//! (§FS-errors.2.1, §FS-errors.5). The published three came down out of the
//! api's contract file when §AR-system.2.9 became a module, because the editor
//! snapshot of §FS-lsp carries a `Report` and the show refusal of §FS-errors.5.2
//! carries a `FindingSite`, and both of those are the queries' (§AR-system.4).
//! Only the records are here; the conversion between them needs a `Config` to
//! spell a path with and stays in `api/report.rs`.

use std::path::PathBuf;

use super::armed::Armed;
use super::codes::Code;

/// A secondary location attached to a diagnostic — e.g. the other declaration in a
/// duplicate pair, or the citation that pointed at a missing section (§FS-errors.2.1).
#[derive(Clone)]
pub(crate) struct Site {
    pub(crate) path: PathBuf,
    pub(crate) line: usize,
}

/// §FS-distribution.4.2.3: the landed clause of the release guard's closed
/// vocabulary, which every finding whose warning ramp closed in `0.16.0` ends
/// with — `unreached-declaration` (§FS-rules.checks.unreached-declaration),
/// `unmarked-heading` (§FS-declarations.checks.unmarked-heading.5),
/// `declaration-near-miss` (§FS-declarations.checks.declaration-near-miss.5) and
/// the absorbed scan (§FS-check.3.30.1). One literal, so the four end in the same
/// bytes and the guard reads the floor out of one string.
pub(crate) const LANDED_CLAUSE: &str = "; this became an error in grund 0.16.0";

/// One finding in the located-finding shape of §FS-errors.2.1: a fixed `code`, the
/// `path:line` it occurred at, the message text, and any cross-reference `sites`.
/// `column` is the 1-based start column of the offending token when the finding
/// concerns a specific citation, so a consumer can anchor on that token rather
/// than the first one on the line (§FS-lsp.1.1.1); it is `None` for line-anchored
/// findings. `authority` is the bytewise-sorted set of rule origins that authored
/// the finding and is empty where no chapter rule did (§FS-rules.7.6) — the set
/// the message tail is joined from, so text and JSON cannot disagree about who
/// said it (§FS-rules.6).
#[derive(Clone)]
pub(crate) struct Diagnostic {
    pub(crate) code: Code,
    pub(crate) path: Option<PathBuf>,
    pub(crate) line: Option<usize>,
    pub(crate) column: Option<usize>,
    pub(crate) message: String,
    pub(crate) sites: Vec<Site>,
    pub(crate) authority: Vec<String>,
}

/// The run's own cautions that travel as diagnostics without being check
/// findings. None renders as one of `check`'s JSON objects — each keeps its text
/// on every surface — so none has a §FS-errors.5.5 row or enters the selector
/// vocabulary, and these are what an editor tags a published diagnostic with and
/// what a test selects one by.
impl Code {
    /// §FS-check.4.10.9: a `[workspace]` block whose own files no scan read.
    pub(crate) const UNREAD_WORKSPACE_BLOCK: Code = Code("unread-workspace-block");
    /// §FS-workspace.6.1.7: an ancestor's claim on a block that could not be read.
    pub(crate) const UNDECIDABLE_WORKSPACE_CLAIM: Code = Code("undecidable-workspace-claim");

    /// The check findings about a config line that a terminal still prints CLI-level,
    /// the line in their text (§FS-errors.2.2.1) as well as in `line` (§FS-errors.5.2.3):
    /// the version-2 window's warnings (§FS-config-v2.checks.config-not-toml,
    /// §FS-config-v2.checks.config-multiline-string).
    const CLI_LEVEL_AT_A_LINE: [Code; 2] = [Code::CONFIG_MULTILINE_STRING, Code::CONFIG_NOT_TOML];
}

/// The outcome of `check`: errors and warnings, kept apart so the exit code keys
/// off errors only (§FS-check.2, §FS-check.4) and the printed order is fixed
/// (§FS-errors.4.1, §FS-non-goals.9). `suggestions` is the third, non-severity
/// advisory channel (§FS-check.2.3, §DF-citation-directions.2.3): the
/// `should` / `should-not` citation-direction findings, withheld from the
/// default run and surfaced only under `--suggestions`. It never affects the
/// exit code.
///
/// `armed` is not a channel: it is what the run's gates turned on, which only
/// the CLI's selection reads (§FS-check.1.4.1). Nor is `verified_blocks`: it
/// says the run found a managed block to compare, which the absent-member
/// warning reads (§FS-check.4.9.1).
#[derive(Default)]
pub(crate) struct CheckReport {
    pub(crate) errors: Vec<Diagnostic>,
    pub(crate) warnings: Vec<Diagnostic>,
    pub(crate) suggestions: Vec<Diagnostic>,
    pub(crate) armed: Armed,
    pub(crate) verified_blocks: bool,
}

/// A secondary location attached to a published [`Finding`] (§FS-errors.2.1,
/// §FS-errors.5): the twin of [`Site`] with its path already spelled the way the
/// report spells it, because a consumer of the published shape has no `Config`
/// to render one with.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct FindingSite {
    pub path: String,
    pub line: usize,
}

/// One published finding: the located-finding shape of §FS-errors.2.1 with its
/// paths rendered — the twin of [`Diagnostic`], and what `check` returns to an
/// embedder rather than prints (§AR-bindings.2).
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
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
    /// The rule origins that authored this finding, bytewise sorted, empty where
    /// no chapter rule did — rendered as JSON's last key, `null` when empty
    /// (§FS-errors.5.1, §FS-output-shapes.1). A declared rule's ID, `"--rule"`
    /// for a `check --rule` trial sentence, or both where they reached the same
    /// meaning (§FS-rules.6). It is the field `check --only-rule` queries
    /// (§FS-rules.8).
    pub authority: Vec<String>,
}

impl Finding {
    /// Whether a terminal prints this finding as a CLI-level message on stderr
    /// (§FS-errors.2.2) rather than as a located one on stdout (§FS-errors.2.1): one
    /// with no line, and one of the few whose line a config file holds and whose
    /// message already names it (§FS-errors.5.2.3). Asked of the finding, so text and
    /// JSON put it on the same stream.
    pub fn is_cli_level(&self) -> bool {
        self.line.is_none() || Code::CLI_LEVEL_AT_A_LINE.contains(&Code(self.code))
    }
}

/// The published outcome of `check` — the twin of [`CheckReport`], kept in the
/// same three channels so the exit code still keys off errors alone
/// (§FS-check.2, §FS-check.2.3).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct Report {
    pub errors: Vec<Finding>,
    pub warnings: Vec<Finding>,
    /// The non-severity advisory channel (§FS-check.2.3): citation-direction
    /// `should` / `should-not` findings, populated only when
    /// `CheckOpts::include_suggestions` is set. Never affects the exit code.
    pub suggestions: Vec<Finding>,
}
