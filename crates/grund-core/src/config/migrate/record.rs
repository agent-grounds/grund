//! The one typed record `grund config migrate --to 2` returns (§AR-config.8):
//! everything the complete report says (§FS-config-migrate.2.2), and the
//! eligible candidate (§FS-config-migrate.2.1). It renders nothing;
//! `grund-cli` selects compact success (§FS-config-migrate.6.5) or complete
//! output without changing this record or migration policy.

/// What a run found at the config it was pointed at.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MigrateOutcome {
    /// The config is already version 2: nothing to migrate, nothing touched
    /// (§FS-config-migrate.6.6). `config` is its path from the report base.
    AlreadyV2 { config: String },
    /// A version-1 config, previewed or written.
    Migrated(Box<MigrateReport>),
}

/// §FS-config-migrate.2.2: the report on one version-1 config.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MigrateReport {
    /// The config's path from the report base, `grund.toml` by default.
    pub config: String,
    /// Whether the run was asked to write (`--write`).
    pub write: bool,
    /// Whether the run may drop the listed comments (`--drop-comments`).
    pub drop_comments: bool,
    /// What version 2 cannot spell, in file order (§FS-config-migrate.3.1).
    pub refusals: Vec<MigrateRefusal>,
    /// The comments no single key carries (§FS-config-migrate.4.3).
    pub comments: Vec<ListedComment>,
    /// The settings dropped or written differently (§FS-config-migrate.3.2).
    pub notices: Vec<MigrateNotice>,
    /// Each authored key and where it went, in file order; empty when refused.
    pub mapped: Vec<MappedKey>,
    /// Each v1 default written out, as `<v2 key> = <value>`, in the
    /// candidate's order (§FS-config-migrate.3.3).
    pub written: Vec<String>,
    /// The workspace members still on version 1, each by its path from the
    /// root (§FS-config-migrate.8).
    pub members: Vec<String>,
    /// Both gates' result, `None` when a refusal kept them from running
    /// (§FS-config-migrate.5.4).
    pub gates: Option<GateResult>,
    /// Whether the complement gained places, which the next `grund init` adds
    /// to the Project map (§FS-config-migrate.5.3).
    pub code_row_note: bool,
    /// The version-2 file, without the listed comments, when there is one
    /// (§FS-config-migrate.2.1).
    pub candidate: Option<String>,
    /// The number of lines the version-1 file had.
    pub v1_lines: usize,
    /// Whether the candidate replaced the config (§FS-config-migrate.6.5).
    pub wrote: bool,
}

impl MigrateReport {
    /// §FS-config-migrate.2.3: whether the run is refused, exit `1`: a
    /// refusal, a difference at either gate, or a listed comment the run may
    /// not drop.
    pub fn refused(&self) -> bool {
        !self.refusals.is_empty()
            || !matches!(self.gates, Some(GateResult::Equal))
            || (!self.comments.is_empty() && !self.drop_comments)
    }

    /// §FS-config-migrate.2.1: the candidate eligible for stdout, unless nothing
    /// may be written for a cause other than a listed comment, or a `--write`
    /// refuses. This is eligibility, not CLI presentation: the CLI omits even
    /// an eligible candidate on compact successful writes (§FS-config-migrate.6.5).
    pub fn stdout(&self) -> Option<&str> {
        let blocked = !self.refusals.is_empty() || !matches!(self.gates, Some(GateResult::Equal));
        if blocked || (self.write && self.refused()) {
            return None;
        }
        self.candidate.as_deref()
    }
}

/// Where a refusal points (§FS-config-migrate.2.2).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RefusalAnchor {
    /// The line of the key it is about: `<config>:<line>: …`.
    Line(usize),
    /// A key the file omits: `<config>: …`.
    Unset,
    /// The file itself, as a symbolic link: `<config> …`.
    File,
}

/// §FS-config-migrate.3.1: one refusal, its detail lines and its repair.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MigrateRefusal {
    pub anchor: RefusalAnchor,
    pub what: String,
    /// Indented detail lines, each already spelled (`would stop reading: …`).
    pub details: Vec<String>,
    pub repair: String,
}

/// §FS-config-migrate.4.3: a comment no single key carries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListedComment {
    pub first: usize,
    pub last: usize,
    /// Attachment (`above <v1 key>`, `inside <v1 key>` or `standing alone`)
    /// followed by its local recovery guidance (§FS-config-migrate.2.2).
    pub place: String,
}

/// §FS-config-migrate.3.2: one setting dropped, or written differently.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MigrateNotice {
    pub line: usize,
    /// `dropped` when nothing is written for it, else `changed`.
    pub dropped: bool,
    pub key: String,
    pub why: String,
}

/// §FS-config-migrate.2.2: one authored key and its v2 spelling.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MappedKey {
    pub line: usize,
    pub key: String,
    pub spelling: String,
}

/// §FS-config-migrate.5.4: what the two gates found.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GateResult {
    /// Both passed.
    Equal,
    /// The first gate that differed, and its differing lines, each already
    /// prefixed `- ` for version 1 or `+ ` for the candidate.
    Differs { gate: Gate, lines: Vec<String> },
}

/// One of the two gates (§FS-config-migrate.5).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Gate {
    Records,
    Report,
}

/// §FS-config-migrate.2.3: what keeps the command from running at all, exit `2`;
/// the original is untouched in every case.
#[derive(Debug)]
pub enum MigrateFailure {
    /// No config up the walk from `path` (§FS-config-migrate.7).
    NoConfig { path: String },
    /// The config does not load, as `config validate` says it
    /// (§FS-config-migrate.7).
    Load(anyhow::Error),
    /// The write failed at `reason` (§FS-config-migrate.6.2).
    Write { config: String, reason: String },
    /// The config changed while the run read it (§FS-config-migrate.6.3).
    Changed { config: String },
}
