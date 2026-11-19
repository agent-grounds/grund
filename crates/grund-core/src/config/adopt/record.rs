//! The one typed record `grund config adopt-builtins` returns (§AR-config.10): the plan, the
//! candidate, the listed comments and what the gate found (§FS-config-adopt-builtins.5.2). It
//! renders nothing; `grund-cli` prints it.

use crate::model::{Finding, remedies};

/// What a run found at the config it was pointed at.
#[derive(Clone, Debug)]
pub enum AdoptOutcome {
    /// §FS-config-adopt-builtins.9: no suffix is adopted, so nothing is written, compared or
    /// headed; `kept` are the suffixes in play the plan keeps.
    NothingToAdopt {
        config: String,
        kept: Vec<KeptSuffix>,
    },
    /// A plan with at least one adopted suffix, previewed or written, or a refused symbolic link.
    Adopted(Box<AdoptReport>),
}

/// §FS-config-adopt-builtins.5.2: the report on one version-2 config.
#[derive(Clone, Debug, Default)]
pub struct AdoptReport {
    /// The config's path from the report base, `grund.toml` by default.
    pub config: String,
    /// Whether the run was asked to write (`--write`).
    pub write: bool,
    /// Whether the run may drop the listed comments (`--drop-comments`).
    pub drop_comments: bool,
    /// Whether the config is a symbolic link, refused before the plan
    /// (§FS-config-adopt-builtins.8.1).
    pub symlink: bool,
    /// Each adopted suffix, in entry order (§FS-config-adopt-builtins.3.1).
    pub adopted: Vec<AdoptedSuffix>,
    /// Each adopted row that adds suffixes, in the candidate's `languages` order
    /// (§FS-config-adopt-builtins.3.3).
    pub added: Vec<AddedSuffixes>,
    /// Each kept suffix, in entry order (§FS-config-adopt-builtins.3.2).
    pub kept: Vec<KeptSuffix>,
    /// The comments on lines adoption removes, in file order (§FS-config-adopt-builtins.7).
    pub comments: Vec<AdoptComment>,
    /// The error the candidate does not load with (§FS-config-adopt-builtins.6.1).
    pub load_error: Option<String>,
    /// The readings that differ from the plan: the `differs:` heading, then `- `/`+ ` lines,
    /// empty when the candidate reads as planned (§FS-config-adopt-builtins.6.2).
    pub readings: Vec<String>,
    /// The other records that differ: the `differs:` heading, then `- `/`+ ` lines, empty when
    /// they are equal (§FS-config-adopt-builtins.6.3).
    pub records: Vec<String>,
    /// What `grund check --suggestions` reports before and after
    /// (§FS-config-adopt-builtins.6.4); `None` when the candidate does not read as planned.
    pub compared: Option<Compared>,
    /// The candidate's `languages`, in order (§FS-config-adopt-builtins.4.1).
    pub languages: Vec<String>,
    /// The candidate text (§FS-config-adopt-builtins.5.1).
    pub candidate: String,
    /// Whether the candidate replaced the config (§FS-config-adopt-builtins.8.2).
    pub wrote: bool,
}

impl AdoptReport {
    /// §FS-config-adopt-builtins.10: whether the run is refused, exit `1`.
    pub fn refused(&self) -> bool {
        self.symlink
            || self.load_error.is_some()
            || !self.readings.is_empty()
            || !self.records.is_empty()
            || (!self.comments.is_empty() && !self.drop_comments)
    }

    /// §FS-config-adopt-builtins.10: the exit, `1` when refused and `0` otherwise.
    pub fn exit(&self) -> u8 {
        u8::from(self.refused())
    }

    /// §FS-config-adopt-builtins.5.1: the candidate for stdout, only from a preview whose
    /// `--write` would write.
    pub fn stdout(&self) -> Option<&str> {
        (!self.write && !self.refused()).then_some(self.candidate.as_str())
    }
}

/// §FS-config-adopt-builtins.3.1: one adopted suffix and what reads differently.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdoptedSuffix {
    pub suffix: String,
    pub definition: String,
    pub row: String,
    /// Whether the row is opt-in, so admitted only because `languages` lists it.
    pub opt_in: bool,
    /// Each `reads differently:` line's text, in the order of §FS-config-adopt-builtins.5.3.
    pub differences: Vec<String>,
}

/// §FS-config-adopt-builtins.3.3: the suffixes an adopted row starts to read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddedSuffixes {
    pub row: String,
    pub suffixes: Vec<String>,
}

/// §FS-config-adopt-builtins.3.2: one kept suffix and why.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeptSuffix {
    pub suffix: String,
    pub definition: String,
    pub why: KeptWhy,
}

/// Why a suffix in play is kept (§FS-config-adopt-builtins.3.2).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KeptWhy {
    /// No built-in row claims it.
    NoRow,
    /// The row that claims it reads it with another class.
    Class { row: String, class: String },
}

/// §FS-config-adopt-builtins.7: one comment on a line adoption removes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdoptComment {
    pub first: usize,
    pub last: usize,
    /// `above <key or header>`, `after <key>` or `inside [<table>]`.
    pub place: String,
}

/// §FS-config-adopt-builtins.6.4: the two `check --suggestions` runs.
#[derive(Clone, Debug, Default)]
pub struct Compared {
    pub before: u8,
    pub after: u8,
    /// The findings only the file's run reports, each with its channel, in `check`'s order.
    pub removed: Vec<(&'static str, Finding)>,
    /// The findings only the candidate's run reports, each with its channel, in `check`'s order.
    pub added: Vec<(&'static str, Finding)>,
}

/// §FS-config-adopt-builtins.10: what keeps the command from reaching a verdict, exit `2`; the
/// file is unchanged in every case.
#[derive(Debug)]
pub enum AdoptFailure {
    /// No config up the walk from `path`.
    NoConfig { path: String },
    /// The config does not load, as `config validate` says it.
    Load(anyhow::Error),
    /// The config is version 1, which migration moves first.
    Version1 { config: String },
    /// The comparison cannot be computed (§FS-config-adopt-builtins.6.4).
    Compare(anyhow::Error),
    /// The write failed at `reason` (§FS-config-adopt-builtins.8.1).
    Write { config: String, reason: String },
    /// The config changed after the plan was built (§FS-config-adopt-builtins.8.1).
    Changed { config: String },
}

impl AdoptFailure {
    /// §FS-config-adopt-builtins.10: every failure is exit `2`, with the file unchanged.
    pub fn exit(&self) -> u8 {
        2
    }
}

/// The failure's message, after `error: ` (§FS-config-adopt-builtins.8.1,
/// §FS-config-adopt-builtins.10).
impl std::fmt::Display for AdoptFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoConfig { path } => {
                write!(
                    f,
                    "no grund.toml at {path} or above it; grund init writes one"
                )
            }
            Self::Load(error) | Self::Compare(error) => write!(f, "{error:#}"),
            // §AR-config.9.1: the remedy is `config-version-1`'s Fix, its slot "it first".
            Self::Version1 { config } => write!(
                f,
                "{config} is version 1; {} --write",
                remedies::config_version_1("it first")
            ),
            Self::Write { config, reason } => write!(
                f,
                "{config}: could not write the adopted config ({reason}); {config} is unchanged, \
                 run grund config adopt-builtins again"
            ),
            Self::Changed { config } => write!(
                f,
                "{config} changed while its built-in languages were being adopted; nothing \
                 written, run grund config adopt-builtins again"
            ),
        }
    }
}
