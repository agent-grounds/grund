//! `integrations`, the one engine operation behind every form of
//! `grund integrations` (§FS-integrations.1.3.1, §FS-distribution.3.1.3): it
//! refuses, detects, describes and, under `write`, installs, using the
//! owned-block splices and probes beside it, and answers with the records of
//! `integrations_output.rs`. No output stream, process argument or cwd mutation
//! is involved (§FS-distribution.3.3.4).

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::integrations_clients::{
    GRUND_OPEN_RESOLVER, InstallKind, IntegrationClient, RESOLVER_TARGET, VSCODE_EXTENSION_JS,
    VSCODE_PACKAGE_JSON, expand_target,
};
use super::integrations_detect::detect_clients;
use super::integrations_guidance;
use super::integrations_install::{
    BlockOutcome, WEZTERM_APPLY_CALL, install_managed_block, integration_is_current,
    needs_wezterm_wiring, vscode_integration_is_current, write_resolver_script,
};
use super::integrations_output::{
    IntegrationArtifact, IntegrationClientStatus, IntegrationEvent, IntegrationManualStep,
    IntegrationNote, IntegrationOutcome, IntegrationsError, IntegrationsFailure, IntegrationsOpts,
    IntegrationsOutput,
};
use super::integrations_user_config::read_optional_text;

/// What stopped a write: the target, the message a report names it with, and
/// the filesystem's own error where there is one (§FS-integrations.4.3.14).
pub(super) struct Fault {
    pub(super) path: PathBuf,
    pub(super) message: String,
    pub(super) source: Option<std::io::Error>,
}

impl Fault {
    pub(super) fn at(path: &Path, message: impl Into<String>) -> Self {
        Self {
            path: path.to_path_buf(),
            message: message.into(),
            source: None,
        }
    }

    fn io(path: &Path, error: std::io::Error) -> Self {
        Self {
            path: path.to_path_buf(),
            message: error.to_string(),
            source: Some(error),
        }
    }

    fn into_error(self, output: IntegrationsOutput, began: bool) -> IntegrationsError {
        IntegrationsError {
            output,
            failure: IntegrationsFailure::Filesystem {
                path: self.path,
                message: self.message,
            },
            source: self.source.map(Arc::new),
            began,
        }
    }
}

impl From<(PathBuf, String)> for Fault {
    fn from((path, message): (PathBuf, String)) -> Self {
        Self {
            path,
            message,
            source: None,
        }
    }
}

/// The `~`-rooted target as a path, or the fault that names it unresolved.
pub(super) fn expand(target: &'static str) -> Result<PathBuf, Fault> {
    expand_target(target)
        .ok_or_else(|| Fault::at(Path::new(target), "cannot resolve home directory"))
}

pub(super) fn save(path: &Path, text: &str) -> Result<(), Fault> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| Fault::io(parent, error))?;
    }
    fs::write(path, text).map_err(|error| Fault::io(path, error))
}

pub(super) fn outcome(outcome: BlockOutcome) -> IntegrationOutcome {
    match outcome {
        BlockOutcome::Appended => IntegrationOutcome::Appended,
        BlockOutcome::Updated => IntegrationOutcome::Updated,
        BlockOutcome::Unchanged => IntegrationOutcome::Unchanged,
    }
}

pub(super) fn event(path: PathBuf, outcome: IntegrationOutcome) -> IntegrationEvent {
    IntegrationEvent {
        path,
        outcome,
        note: None,
    }
}

/// Answer `grund integrations` as data (§FS-integrations.1.3.1): refuse a
/// combination §FS-integrations.6.2 refuses before reading anything, read the
/// user configuration before any client artifact is written, and plan every
/// guidance target before writing the first, so a run fails at the point and
/// with the partial writes of §FS-integrations.4.3.14.
pub fn integrations(opts: IntegrationsOpts) -> Result<IntegrationsOutput, IntegrationsError> {
    if let Some(failure) = opts.refusal() {
        return Err(IntegrationsError::refused(failure));
    }
    let detected = detect_clients();
    let mut out = IntegrationsOutput {
        clients: IntegrationClient::ALL
            .iter()
            .map(|&client| IntegrationClientStatus {
                client,
                install_kind: client.kind(),
                detected: detected.contains(&client),
                installed: integration_is_current(client),
                config_target: client.target_hint(),
            })
            .collect(),
        detected,
        artifact: opts.client.map(artifact),
        ..IntegrationsOutput::default()
    };
    if !opts.write {
        return Ok(out);
    }
    // §FS-integrations.4.3.7: read user configuration before any artifact write.
    let guidance = match integrations_guidance::load() {
        Ok(guidance) => guidance,
        Err(fault) => return Err(fault.into_error(out, false)),
    };
    out.warnings = guidance.cautions.clone();
    let written = match opts.client {
        Some(client) => install(client, &mut out),
        None => Ok(()),
    }
    .and_then(|()| integrations_guidance::install(guidance, &opts, &mut out.events));
    match written {
        Ok(()) => Ok(out),
        Err(fault) => Err(fault.into_error(out, true)),
    }
}

fn artifact(client: IntegrationClient) -> IntegrationArtifact {
    let terminal = client.is_terminal_client();
    IntegrationArtifact {
        client,
        snippet: client.artifact_snippet(),
        resolver: terminal.then_some(GRUND_OPEN_RESOLVER),
        resolver_target: terminal.then_some(RESOLVER_TARGET),
        package_json: (!terminal).then_some(VSCODE_PACKAGE_JSON),
        extension_js: (!terminal).then_some(VSCODE_EXTENSION_JS),
    }
}

/// The client half of a `write` (§FS-integrations.4.1, §FS-integrations.4.2,
/// §FS-integrations.3.4.2): its config or extension, then the resolver a
/// terminal client needs, then the step a manual client leaves to the user.
fn install(client: IntegrationClient, out: &mut IntegrationsOutput) -> Result<(), Fault> {
    match client.kind() {
        InstallKind::Manual => {}
        InstallKind::Block => {
            let path = expand(client.target_hint())?;
            let existing = read_optional_text(&path)?;
            let fresh = existing.is_empty();
            let (mut updated, spliced) = install_managed_block(
                client.comment(),
                client.prepends(),
                &existing,
                client.artifact_snippet().unwrap_or(""),
            )
            .map_err(|message| Fault::at(&path, message))?;
            // A file created from scratch gets the client's starter config
            // below the block, so the install is usable without hand-editing.
            if fresh && let Some(scaffold) = client.scaffold() {
                updated.push_str(scaffold);
            }
            if spliced != BlockOutcome::Unchanged {
                save(&path, &updated)?;
            }
            // §FS-integrations.4.1.3: reported rather than silent.
            let note =
                needs_wezterm_wiring(client, &updated).then_some(IntegrationNote::UnwiredWezterm {
                    apply_call: WEZTERM_APPLY_CALL,
                });
            out.events.push(IntegrationEvent {
                path,
                outcome: outcome(spliced),
                note,
            });
        }
        InstallKind::Vscode => {
            let dir = expand(client.target_hint())?;
            if vscode_integration_is_current(&dir) {
                out.events.push(event(dir, IntegrationOutcome::Unchanged));
                return Ok(());
            }
            let existed = dir.join(".grund-version").is_file();
            let version = crate::grammar::INTEGRATIONS_BLOCK_VERSION.to_string();
            for (name, body) in [
                ("package.json", VSCODE_PACKAGE_JSON),
                ("extension.js", VSCODE_EXTENSION_JS),
                (".grund-version", version.as_str()),
            ] {
                save(&dir.join(name), body)?;
            }
            let written = if existed {
                IntegrationOutcome::Updated
            } else {
                IntegrationOutcome::Wrote
            };
            out.events.push(event(dir, written));
            return Ok(());
        }
    }
    if let Some(path) = write_resolver_script()? {
        out.events.push(event(path, IntegrationOutcome::Wrote));
    }
    if client.kind() == InstallKind::Manual {
        out.manual_steps.push(IntegrationManualStep {
            client,
            snippet: client.artifact_snippet().unwrap_or(""),
        });
    }
    Ok(())
}
