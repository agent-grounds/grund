//! What `integrations` is asked and what it answers (§FS-integrations.1.3.1):
//! the options in the command's closed vocabularies, the output of facts a
//! frontend renders, and the refusal that keeps what a run wrote before it
//! failed (§FS-integrations.1.3.3). Nothing here is a line of output
//! (§DA-engine-renders-nothing): the CLI's text, its JSON plan and the hosts'
//! JSON are three spellings of these records.

use std::path::PathBuf;
use std::sync::Arc;

use super::integrations_agents::{ConversationRendering, ConversationTarget, IntegrationAgent};
use super::integrations_clients::{InstallKind, IntegrationClient};
use crate::model::Finding;

/// `grund integrations`'s arguments in their closed vocabularies
/// (§FS-integrations.1.3.1). Built from `Default` with its fields assigned
/// (§FS-distribution.3.1.2).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct IntegrationsOpts {
    /// The client to describe or, under `write`, install.
    pub client: Option<IntegrationClient>,
    /// Apply the integration and synchronize the agent guidance
    /// (§FS-integrations.4); without it nothing is written.
    pub write: bool,
    /// `--conversation`: overrides the stored preference (§FS-integrations.1.1).
    pub conversation: Option<ConversationRendering>,
    /// `--conversation-target` (§FS-integrations.1.2).
    pub conversation_target: Option<ConversationTarget>,
    /// `--agent`: scopes `conversation_target` to one agent (§FS-integrations.4.4.2).
    pub agent: Option<IntegrationAgent>,
}

impl IntegrationsOpts {
    /// The combination §FS-integrations.6.2 refuses, checked in the order the
    /// CLI reports it, or `None`. `integrations` refuses with the same answer
    /// before it reads anything; a frontend with refusals of its own after these
    /// asks first (§FS-integrations.1.3.1).
    #[doc(hidden)]
    pub fn refusal(&self) -> Option<IntegrationsFailure> {
        if self.agent.is_some() && !self.write {
            Some(IntegrationsFailure::AgentWithoutWrite)
        } else if self.agent.is_some() && self.conversation_target.is_none() {
            Some(IntegrationsFailure::AgentWithoutTarget)
        } else if self.conversation.is_some() && !self.write {
            Some(IntegrationsFailure::ConversationWithoutWrite)
        } else if self.conversation_target.is_some() && !self.write {
            Some(IntegrationsFailure::ConversationTargetWithoutWrite)
        } else if self.write
            && self.client.is_none()
            && self.conversation.is_none()
            && self.conversation_target.is_none()
        {
            Some(IntegrationsFailure::WriteWithoutSubject)
        } else {
            None
        }
    }
}

/// The answer (§FS-integrations.1.3.2): only facts, never a line of output.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct IntegrationsOutput {
    /// The clients the environment shows, in the frozen client order
    /// (§FS-integrations.2).
    pub detected: Vec<IntegrationClient>,
    /// One status per client, in the frozen client order.
    pub clients: Vec<IntegrationClientStatus>,
    /// The named client's artifact (§FS-integrations.3).
    pub artifact: Option<IntegrationArtifact>,
    /// One per target a `write` touched, in the order it touched them.
    pub events: Vec<IntegrationEvent>,
    /// One per client the user must set up by hand (§FS-integrations.3.4.2).
    pub manual_steps: Vec<IntegrationManualStep>,
    /// The user configuration's cautions (§FS-integrations.4.3.5).
    pub warnings: Vec<Finding>,
}

/// One client as detection reports it (§FS-integrations.5). It carries no
/// install command: that is the CLI's own argv (§FS-integrations.1.3.2).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct IntegrationClientStatus {
    pub client: IntegrationClient,
    pub install_kind: InstallKind,
    pub detected: bool,
    /// Every grund-owned artifact present and byte-current; never for a manual
    /// client, whose configuration grund cannot read.
    pub installed: bool,
    /// The `~`-rooted place `write` installs into, printed verbatim
    /// (§FS-integrations.6).
    pub config_target: &'static str,
}

/// What a client's integration consists of, as content (§FS-integrations.3).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct IntegrationArtifact {
    pub client: IntegrationClient,
    /// The config snippet; `None` for the editor clients.
    pub snippet: Option<&'static str>,
    /// The `grund-open` resolver, for the terminal clients.
    pub resolver: Option<&'static str>,
    /// Where `write` installs the resolver, `~`-rooted.
    pub resolver_target: Option<&'static str>,
    /// The editor extension's `package.json`, for the editor clients.
    pub package_json: Option<&'static str>,
    /// The editor extension's `extension.js`, for the editor clients.
    pub extension_js: Option<&'static str>,
}

/// One target a `write` touched (§FS-integrations.1.3.2).
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct IntegrationEvent {
    pub path: PathBuf,
    pub outcome: IntegrationOutcome,
    pub note: Option<IntegrationNote>,
}

/// What happened to one target (§FS-integrations.4.1.6).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum IntegrationOutcome {
    Wrote,
    Appended,
    Updated,
    Unchanged,
    Skipped,
}

impl IntegrationOutcome {
    /// The verb a report spells the outcome with: `exists` for `Unchanged`
    /// (§FS-integrations.4.1.6).
    pub fn name(self) -> &'static str {
        match self {
            Self::Wrote => "wrote",
            Self::Appended => "appended",
            Self::Updated => "updated",
            Self::Unchanged => "exists",
            Self::Skipped => "skipped",
        }
    }
}

/// What a report says beside one event (§FS-integrations.1.3.2).
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum IntegrationNote {
    /// The WezTerm config does not make this call on the config it returns, so
    /// the block registers nothing (§FS-integrations.4.1.3).
    UnwiredWezterm { apply_call: &'static str },
    /// The form an agent's block was written in (§FS-integrations.4.4.4):
    /// `target` is the one the gate let through, `requested` the one asked for,
    /// and `overridden` whether the agent's own override chose it.
    Guidance {
        rendering: ConversationRendering,
        target: ConversationTarget,
        requested: ConversationTarget,
        overridden: bool,
    },
    /// The agent's `~`-rooted home is absent, so it is not in use here
    /// (§FS-integrations.4.3.9).
    AgentHomeMissing { home: &'static str },
}

/// A client the user must set up by hand, with the snippet to apply
/// (§FS-integrations.3.4.2).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct IntegrationManualStep {
    pub client: IntegrationClient,
    pub snippet: &'static str,
}

/// Why `integrations` refused (§FS-integrations.1.3.3, §FS-integrations.6.2).
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum IntegrationsFailure {
    /// `agent` without `write`.
    AgentWithoutWrite,
    /// `agent` without `conversation_target`.
    AgentWithoutTarget,
    /// `conversation` without `write`.
    ConversationWithoutWrite,
    /// `conversation_target` without `write`.
    ConversationTargetWithoutWrite,
    /// `write` with neither a client nor a conversation option.
    WriteWithoutSubject,
    /// A target could not be read or written. `path` is the `~`-rooted target
    /// itself when no home directory resolves it.
    Filesystem { path: PathBuf, message: String },
}

impl std::fmt::Display for IntegrationsFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AgentWithoutWrite => f.write_str("an agent requires write"),
            Self::AgentWithoutTarget => f.write_str("an agent requires a conversation target"),
            Self::ConversationWithoutWrite => f.write_str("a conversation requires write"),
            Self::ConversationTargetWithoutWrite => {
                f.write_str("a conversation target requires write")
            }
            Self::WriteWithoutSubject => f.write_str("write requires a client or a conversation"),
            Self::Filesystem { path, message } => write!(f, "{}: {message}", path.display()),
        }
    }
}

/// A refusal with what the run had done before it (§FS-integrations.1.3.3):
/// `output` holds the events written, the manual steps reached and the
/// cautions read, and the filesystem's own error is the `source`.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct IntegrationsError {
    pub output: IntegrationsOutput,
    pub failure: IntegrationsFailure,
    pub(super) source: Option<Arc<std::io::Error>>,
    /// Whether the run had begun installing, which is when a partial output
    /// exists to report (§FS-distribution.3.3.6).
    pub(super) began: bool,
}

impl IntegrationsError {
    pub(super) fn refused(failure: IntegrationsFailure) -> Self {
        Self {
            output: IntegrationsOutput::default(),
            failure,
            source: None,
            began: false,
        }
    }
}

impl std::fmt::Display for IntegrationsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.failure.fmt(f)
    }
}

impl std::error::Error for IntegrationsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_deref()
            .map(|io| io as &(dyn std::error::Error + 'static))
    }
}
