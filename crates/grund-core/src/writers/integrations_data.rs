//! The hosts' spelling of `integrations` (§FS-distribution.3.3.6): the names a
//! binding passes parsed into the command's vocabularies, and the typed answer
//! projected into the JSON the Python and Node bindings return
//! (§FS-integrations.1.3.2). It decides nothing the typed operation does not.

use super::integrations_agents::{ConversationRendering, ConversationTarget, IntegrationAgent};
use super::integrations_api::integrations;
use super::integrations_clients::{InstallKind, IntegrationClient};
use super::integrations_output::{
    IntegrationEvent, IntegrationNote, IntegrationsError, IntegrationsFailure, IntegrationsOpts,
    IntegrationsOutput,
};
use crate::model::{Finding, OperationContext, OperationDiagnostic, format_path};
use anyhow::Result;
use serde_json::{Value, json};
use std::sync::Arc;

fn parsed<T>(
    value: Option<&str>,
    parse: impl Fn(&str) -> Option<T>,
    code: &'static str,
    message: &'static str,
) -> Result<Option<T>> {
    value
        .map(|name| parse(name).ok_or_else(|| OperationDiagnostic::new("operation", code, message)))
        .transpose()
        .map_err(Into::into)
}

/// `integrations` for a language host: inspection and installation alike, with
/// no output stream, process argument or cwd mutation (§FS-distribution.3.3.4).
pub(crate) fn integrations_data(
    client: &str,
    write: bool,
    conversation: Option<&str>,
    target: Option<&str>,
    agent: Option<&str>,
) -> Result<Value> {
    let client = (!client.is_empty()).then_some(client);
    let mut opts = IntegrationsOpts::default();
    opts.client = parsed(
        client,
        IntegrationClient::from_name,
        "invalid-client",
        "unknown integration client",
    )?;
    opts.write = write;
    opts.conversation = parsed(
        conversation,
        ConversationRendering::from_name,
        "invalid-preference",
        "unknown conversation preference",
    )?;
    opts.conversation_target = parsed(
        target,
        ConversationTarget::from_name,
        "invalid-preference",
        "unknown conversation target",
    )?;
    opts.agent = parsed(
        agent,
        IntegrationAgent::from_name,
        "invalid-agent",
        "unknown agent",
    )?;
    match integrations(opts) {
        Ok(out) => Ok(output(&out)),
        Err(error) => Err(failure(error, opts.client)),
    }
}

fn output(out: &IntegrationsOutput) -> Value {
    let clients = out
        .clients
        .iter()
        .map(|status| {
            json!({"client":status.client.name(),"detected":status.detected,
            "installed":status.installed,"install_kind":status.install_kind.name(),
            "install":format!("grund integrations {} --write", status.client.name()),
            "config_target":status.config_target})
        })
        .collect::<Vec<_>>();
    let artifact = out.artifact.map(|a| {
        json!({"client":a.client.name(),"snippet":a.snippet,"resolver":a.resolver,
        "package_json":a.package_json,"extension_js":a.extension_js})
    });
    let manual_steps = out
        .manual_steps
        .iter()
        .map(|step| step.snippet)
        .collect::<Vec<_>>();
    json!({"detected":out.detected.iter().map(|c| c.name()).collect::<Vec<_>>(),
    "clients":clients,"artifact":artifact,"events":events(&out.events),
    "manual_steps":manual_steps,"run_cautions":cautions(&out.warnings)})
}

fn events(events: &[IntegrationEvent]) -> Vec<Value> {
    events
        .iter()
        .map(|e| json!({"path":format_path(&e.path),"verb":e.outcome.name(),"note":e.note.as_ref().map(note)}))
        .collect()
}

fn note(note: &IntegrationNote) -> String {
    match note {
        IntegrationNote::UnwiredWezterm { apply_call } => {
            format!("add {apply_call}config) where you build your config")
        }
        IntegrationNote::Guidance {
            rendering: ConversationRendering::Plain,
            ..
        } => "plain".to_owned(),
        IntegrationNote::Guidance {
            target,
            requested,
            overridden,
            ..
        } => format!(
            "link → {}{}",
            target.name(),
            if target != requested {
                "; unverified here"
            } else if *overridden {
                "; agent override"
            } else {
                ""
            }
        ),
        IntegrationNote::AgentHomeMissing { home } => format!("no {home}"),
    }
}

fn cautions(warnings: &[Finding]) -> Vec<Value> {
    warnings
        .iter()
        .map(|w| {
            json!({"severity":w.severity,"channel":null,"code":w.code,"path":w.path,
            "line":w.line,"column":w.column,"message":w.message,"sites":[],"authority":[]})
        })
        .collect()
}

/// The refusal as the bindings' structured failure: a refused combination is
/// one `invalid-preference`, a filesystem failure is located on its target with
/// the IO error beneath, and a run that had begun carries its partial output.
fn failure(error: IntegrationsError, client: Option<IntegrationClient>) -> anyhow::Error {
    let IntegrationsError {
        output,
        failure,
        source,
        began,
    } = error;
    let IntegrationsFailure::Filesystem { path, message } = failure else {
        return OperationDiagnostic::new(
            "operation",
            "invalid-preference",
            "invalid integration preference combination",
        )
        .into();
    };
    let mut diagnostic = OperationDiagnostic::new("filesystem", "io", message);
    diagnostic.path = Some(format_path(&path));
    let error = match source.map(Arc::try_unwrap) {
        Some(Ok(io)) => anyhow::Error::new(io).context(diagnostic),
        _ => diagnostic.into(),
    };
    if !began {
        return error;
    }
    // A manual client's step is reached before anything a run can fail on.
    let manual_steps = client
        .filter(|c| c.kind() == InstallKind::Manual)
        .map(|c| c.artifact_snippet().unwrap_or(""))
        .into_iter()
        .collect::<Vec<_>>();
    let context = OperationContext {
        message: error.to_string(),
        cautions: json!(cautions(&output.warnings)),
        partial_output: json!({"events":events(&output.events), "manual_steps":manual_steps}),
    };
    error.context(context)
}
