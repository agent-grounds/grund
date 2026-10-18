//! The integrations names the CLI once assembled `--write` from, kept for the
//! two minors §REQ-backwards-compatibility.2 owes them (§FS-distribution.3.1.3).
//! Each forwards to the crate-private original the engine itself calls, so the
//! crate builds clean while an embedder still naming one is warned; the whole
//! module goes on the release its notes name, as the four aliases did.

use std::path::{Path, PathBuf};

use super::integrations_agents::{ConversationRendering, ConversationTarget};
use super::integrations_clients::{InstallKind, IntegrationClient};
use super::{integrations_agents, integrations_clients, integrations_detect};
use super::{integrations_install, integrations_user_config};

#[deprecated(
    since = "0.19.0",
    note = "read IntegrationNote::Guidance, which names the agent; is removed in 0.21.0"
)]
pub type GlobalAgentTarget = integrations_agents::GlobalAgentTarget;
#[deprecated(
    since = "0.19.0",
    note = "the gate is applied by integrations; is removed in 0.21.0"
)]
pub type LinkSupport = integrations_agents::LinkSupport;
#[deprecated(
    since = "0.19.0",
    note = "read IntegrationOutcome on IntegrationEvent; is removed in 0.21.0"
)]
pub type BlockOutcome = integrations_install::BlockOutcome;
#[deprecated(
    since = "0.19.0",
    note = "integrations reads the user configuration; is removed in 0.21.0"
)]
pub type UserConfigScan = integrations_user_config::UserConfigScan;

#[deprecated(
    since = "0.19.0",
    note = "the agents are IntegrationAgent::ALL; is removed in 0.21.0"
)]
pub const GLOBAL_AGENT_INSTRUCTION_TARGETS: [integrations_agents::GlobalAgentTarget; 6] =
    integrations_agents::GLOBAL_AGENT_INSTRUCTION_TARGETS;
#[deprecated(
    since = "0.19.0",
    note = "read IntegrationArtifact::resolver; is removed in 0.21.0"
)]
pub const GRUND_OPEN_RESOLVER: &str = integrations_clients::GRUND_OPEN_RESOLVER;
#[deprecated(
    since = "0.19.0",
    note = "read IntegrationArtifact::resolver_target; is removed in 0.21.0"
)]
pub const RESOLVER_TARGET: &str = integrations_clients::RESOLVER_TARGET;
#[deprecated(
    since = "0.19.0",
    note = "read IntegrationArtifact::package_json; is removed in 0.21.0"
)]
pub const VSCODE_PACKAGE_JSON: &str = integrations_clients::VSCODE_PACKAGE_JSON;
#[deprecated(
    since = "0.19.0",
    note = "read IntegrationArtifact::extension_js; is removed in 0.21.0"
)]
pub const VSCODE_EXTENSION_JS: &str = integrations_clients::VSCODE_EXTENSION_JS;
#[deprecated(
    since = "0.19.0",
    note = "read IntegrationNote::UnwiredWezterm; is removed in 0.21.0"
)]
pub const WEZTERM_APPLY_CALL: &str = integrations_install::WEZTERM_APPLY_CALL;
#[deprecated(
    since = "0.19.0",
    note = "integrations reads the user configuration; is removed in 0.21.0"
)]
pub const USER_CONFIG_TARGET: &str = integrations_user_config::USER_CONFIG_TARGET;
#[deprecated(
    since = "0.19.0",
    note = "integrations writes the block at its version; is removed in 0.21.0"
)]
pub const INTEGRATIONS_BLOCK_VERSION: u32 = crate::grammar::INTEGRATIONS_BLOCK_VERSION;

#[deprecated(
    since = "0.19.0",
    note = "set IntegrationsOpts::agent; is removed in 0.21.0"
)]
pub fn agent_override_table(agent: &str) -> String {
    integrations_agents::agent_override_table(agent)
}

#[deprecated(
    since = "0.19.0",
    note = "list IntegrationAgent::ALL by name; is removed in 0.21.0"
)]
pub fn known_agents_list() -> String {
    integrations_agents::known_agents_list()
}

#[deprecated(
    since = "0.19.0",
    note = "use IntegrationAgent::from_name; is removed in 0.21.0"
)]
pub fn known_agent(name: &str) -> Option<&'static str> {
    integrations_agents::known_agent(name)
}

#[deprecated(
    since = "0.19.0",
    note = "list IntegrationClient::ALL by name; is removed in 0.21.0"
)]
pub fn known_clients_line() -> String {
    integrations_clients::known_clients_line()
}

#[deprecated(
    since = "0.19.0",
    note = "read the paths on IntegrationEvent; is removed in 0.21.0"
)]
pub fn expand_target(target: &str) -> Option<PathBuf> {
    integrations_clients::expand_target(target)
}

#[deprecated(
    since = "0.19.0",
    note = "read IntegrationsOutput::detected; is removed in 0.21.0"
)]
pub fn detect_clients() -> Vec<IntegrationClient> {
    integrations_detect::detect_clients()
}

#[deprecated(
    since = "0.19.0",
    note = "use IntegrationOutcome::name; is removed in 0.21.0"
)]
pub fn block_outcome_verb(outcome: integrations_install::BlockOutcome) -> &'static str {
    integrations_install::block_outcome_verb(outcome)
}

#[deprecated(
    since = "0.19.0",
    note = "integrations reports one IntegrationEvent per target; is removed in 0.21.0"
)]
pub fn merge_outcomes(
    first: integrations_install::BlockOutcome,
    second: integrations_install::BlockOutcome,
) -> integrations_install::BlockOutcome {
    integrations_install::merge_outcomes(first, second)
}

#[deprecated(
    since = "0.19.0",
    note = "call integrations with write set; is removed in 0.21.0"
)]
pub fn install_managed_block(
    comment: &str,
    prepend: bool,
    existing: &str,
    snippet: &str,
) -> Result<(String, integrations_install::BlockOutcome), String> {
    integrations_install::install_managed_block(comment, prepend, existing, snippet)
}

#[deprecated(
    since = "0.19.0",
    note = "call integrations with write set; is removed in 0.21.0"
)]
pub fn install_agent_guidance_block(
    existing: &str,
    preference: ConversationRendering,
    target: ConversationTarget,
) -> Result<(String, integrations_install::BlockOutcome), String> {
    integrations_install::install_agent_guidance_block(existing, preference, target)
}

#[deprecated(
    since = "0.19.0",
    note = "read IntegrationNote::UnwiredWezterm; is removed in 0.21.0"
)]
pub fn needs_wezterm_wiring(client: IntegrationClient, text: &str) -> bool {
    integrations_install::needs_wezterm_wiring(client, text)
}

#[deprecated(
    since = "0.19.0",
    note = "call integrations with write set; is removed in 0.21.0"
)]
pub fn write_resolver_script() -> Result<Option<PathBuf>, (PathBuf, String)> {
    integrations_install::write_resolver_script()
}

#[deprecated(
    since = "0.19.0",
    note = "read IntegrationClientStatus::installed; is removed in 0.21.0"
)]
pub fn integration_is_current(client: IntegrationClient) -> bool {
    integrations_install::integration_is_current(client)
}

#[deprecated(
    since = "0.19.0",
    note = "read IntegrationClientStatus::installed; is removed in 0.21.0"
)]
pub fn vscode_integration_is_current(dir: &Path) -> bool {
    integrations_install::vscode_integration_is_current(dir)
}

#[deprecated(
    since = "0.19.0",
    note = "integrations reads the user configuration; is removed in 0.21.0"
)]
pub fn read_optional_text(path: &Path) -> Result<String, (PathBuf, String)> {
    integrations_user_config::read_optional_text(path)
}

#[deprecated(
    since = "0.19.0",
    note = "integrations reads the user configuration; is removed in 0.21.0"
)]
pub fn user_grund_config_path() -> Option<PathBuf> {
    integrations_user_config::user_grund_config_path()
}

#[deprecated(
    since = "0.19.0",
    note = "read IntegrationsOutput::warnings; is removed in 0.21.0"
)]
pub fn scan_user_config(text: &str) -> integrations_user_config::UserConfigScan {
    integrations_user_config::scan_user_config(text)
}

#[deprecated(
    since = "0.19.0",
    note = "set IntegrationsOpts::conversation_target; is removed in 0.21.0"
)]
pub fn install_reference_key(
    existing: &str,
    table: &str,
    bare_key: &str,
    value: &str,
    already_recorded: bool,
) -> (String, integrations_install::BlockOutcome) {
    integrations_user_config::install_reference_key(
        existing,
        table,
        bare_key,
        value,
        already_recorded,
    )
}

impl IntegrationClient {
    #[deprecated(
        since = "0.19.0",
        note = "read IntegrationArtifact::resolver; is removed in 0.21.0"
    )]
    pub fn is_terminal(self) -> bool {
        self.is_terminal_client()
    }

    #[deprecated(
        since = "0.19.0",
        note = "read IntegrationClientStatus::install_kind; is removed in 0.21.0"
    )]
    pub fn install_kind(self) -> InstallKind {
        self.kind()
    }

    #[deprecated(
        since = "0.19.0",
        note = "read IntegrationArtifact::snippet; is removed in 0.21.0"
    )]
    pub fn snippet(self) -> Option<&'static str> {
        self.artifact_snippet()
    }

    #[deprecated(
        since = "0.19.0",
        note = "read IntegrationClientStatus::config_target; is removed in 0.21.0"
    )]
    pub fn config_target(self) -> &'static str {
        self.target_hint()
    }

    #[deprecated(
        since = "0.19.0",
        note = "the install command is the frontend's own; is removed in 0.21.0"
    )]
    pub fn install_command(self) -> String {
        self.cli_install_command()
    }

    #[deprecated(
        since = "0.19.0",
        note = "call integrations with write set; is removed in 0.21.0"
    )]
    pub fn comment_prefix(self) -> &'static str {
        self.comment()
    }

    #[deprecated(
        since = "0.19.0",
        note = "call integrations with write set; is removed in 0.21.0"
    )]
    pub fn prepends_block(self) -> bool {
        self.prepends()
    }

    #[deprecated(
        since = "0.19.0",
        note = "call integrations with write set; is removed in 0.21.0"
    )]
    pub fn fresh_config_scaffold(self) -> Option<&'static str> {
        self.scaffold()
    }
}

impl ConversationTarget {
    #[deprecated(
        since = "0.19.0",
        note = "list ConversationTarget::ALL by name; is removed in 0.21.0"
    )]
    pub fn accepted_list() -> String {
        Self::accepted_names()
    }
}
