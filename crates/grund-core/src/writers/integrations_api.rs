//! Data-returning managed integration orchestration (§FS-distribution.3.1,
//! §FS-distribution.3.3.6), using the existing owned-block splices and probes.

use super::*;
use crate::model::{OperationDiagnostic, format_path};
use anyhow::Result;
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn located(path: &Path, message: String) -> OperationDiagnostic {
    let mut e = OperationDiagnostic::new("filesystem", "io", message);
    e.path = Some(format_path(path));
    e
}
pub(super) fn tuple_error((path, message): (PathBuf, String)) -> anyhow::Error {
    located(&path, message).into()
}

pub(super) fn save(path: &Path, text: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            let diagnostic = located(parent, e.to_string());
            anyhow::Error::new(e).context(diagnostic)
        })?;
    }
    fs::write(path, text).map_err(|e| {
        let diagnostic = located(path, e.to_string());
        anyhow::Error::new(e).context(diagnostic)
    })?;
    Ok(())
}

pub(super) fn event(path: &Path, verb: &str, note: Option<String>) -> Value {
    json!({"path":format_path(path),"verb":verb,"note":note})
}

/// Both inspection and installation are engine operations. No output stream,
/// process arguments, or cwd mutation is involved (§FS-distribution.3.3.4).
pub(crate) fn integrations_data(
    client: &str,
    write: bool,
    conversation: Option<&str>,
    target: Option<&str>,
    agent: Option<&str>,
) -> Result<Value> {
    let client = if client.is_empty() {
        None
    } else {
        Some(IntegrationClient::from_name(client).ok_or_else(|| {
            OperationDiagnostic::new("operation", "invalid-client", "unknown integration client")
        })?)
    };
    let conversation = conversation
        .map(|s| {
            ConversationRendering::from_name(s).ok_or_else(|| {
                OperationDiagnostic::new(
                    "operation",
                    "invalid-preference",
                    "unknown conversation preference",
                )
            })
        })
        .transpose()?;
    let target = target
        .map(|s| {
            ConversationTarget::from_name(s).ok_or_else(|| {
                OperationDiagnostic::new(
                    "operation",
                    "invalid-preference",
                    "unknown conversation target",
                )
            })
        })
        .transpose()?;
    let agent = agent
        .map(|s| {
            known_agent(s).ok_or_else(|| {
                OperationDiagnostic::new("operation", "invalid-agent", "unknown agent")
            })
        })
        .transpose()?;
    if (!write && (conversation.is_some() || target.is_some() || agent.is_some()))
        || (agent.is_some() && target.is_none())
        || (write && client.is_none() && conversation.is_none() && target.is_none())
    {
        return Err(OperationDiagnostic::new(
            "operation",
            "invalid-preference",
            "invalid integration preference combination",
        )
        .into());
    }
    let detected = detect_clients();
    let clients = IntegrationClient::ALL
        .iter()
        .map(|c| {
            json!({"client":c.name(),
        "detected":detected.contains(c),"installed":integration_is_current(*c),
        "install_kind":c.install_kind().name(),"install":c.install_command(),
        "config_target":c.config_target()})
        })
        .collect::<Vec<_>>();
    let artifact = client.map(|c| {
        json!({"client":c.name(),"snippet":c.snippet(),
        "resolver":if c.is_terminal(){Some(GRUND_OPEN_RESOLVER)}else{None},
        "package_json":if c.is_terminal(){None}else{Some(VSCODE_PACKAGE_JSON)},
        "extension_js":if c.is_terminal(){None}else{Some(VSCODE_EXTENSION_JS)}})
    });
    let mut events = Vec::new();
    let mut manual_steps = Vec::new();
    let mut cautions = Vec::new();
    if write {
        // §FS-integrations.4.3.7: read user configuration before any artifact write.
        let guidance = super::integrations_guidance::load()?;
        cautions = guidance.cautions.clone();
        let result: Result<()> = (|| {
            if let Some(client) = client {
                install(client, &mut events, &mut manual_steps)?;
            }
            super::integrations_guidance::install(
                guidance,
                conversation,
                target,
                agent,
                &mut events,
            )?;
            Ok(())
        })();
        if let Err(error) = result {
            let context = crate::model::OperationContext {
                message: error.to_string(),
                cautions: json!(cautions),
                partial_output: json!({"events":events, "manual_steps":manual_steps}),
            };
            return Err(error.context(context));
        }
    }
    Ok(
        json!({"detected":detected.iter().map(|c|c.name()).collect::<Vec<_>>(),"clients":clients,
        "artifact":artifact,"events":events,"manual_steps":manual_steps,"run_cautions":cautions}),
    )
}

fn install(
    client: IntegrationClient,
    events: &mut Vec<Value>,
    manual_steps: &mut Vec<String>,
) -> Result<()> {
    match client.install_kind() {
        InstallKind::Manual => manual_steps.push(client.snippet().unwrap_or("").to_owned()),
        InstallKind::Block => {
            let path = expand_target(client.config_target()).ok_or_else(|| {
                located(
                    Path::new(client.config_target()),
                    "cannot resolve home directory".into(),
                )
            })?;
            let existing = read_optional_text(&path).map_err(tuple_error)?;
            let fresh = existing.is_empty();
            let (mut updated, outcome) = install_managed_block(
                client.comment_prefix(),
                client.prepends_block(),
                &existing,
                client.snippet().unwrap_or(""),
            )
            .map_err(|e| located(&path, e))?;
            if fresh && let Some(scaffold) = client.fresh_config_scaffold() {
                updated.push_str(scaffold);
            }
            if outcome != BlockOutcome::Unchanged {
                save(&path, &updated)?;
            }
            let note = needs_wezterm_wiring(client, &updated)
                .then(|| format!("add {WEZTERM_APPLY_CALL}config) where you build your config"));
            events.push(event(&path, block_outcome_verb(outcome), note));
        }
        InstallKind::Vscode => {
            let dir = expand_target(client.config_target()).ok_or_else(|| {
                located(
                    Path::new(client.config_target()),
                    "cannot resolve home directory".into(),
                )
            })?;
            let exists = dir.join(".grund-version").is_file();
            if vscode_integration_is_current(&dir) {
                events.push(event(&dir, "exists", None));
                return Ok(());
            }
            for (name, body) in [
                ("package.json", VSCODE_PACKAGE_JSON.to_owned()),
                ("extension.js", VSCODE_EXTENSION_JS.to_owned()),
                (
                    ".grund-version",
                    crate::grammar::INTEGRATIONS_BLOCK_VERSION.to_string(),
                ),
            ] {
                save(&dir.join(name), &body)?;
            }
            events.push(event(&dir, if exists { "updated" } else { "wrote" }, None));
            return Ok(());
        }
    }
    if let Some(path) = write_resolver_script().map_err(tuple_error)? {
        events.push(event(&path, "wrote", None));
    }
    Ok(())
}
