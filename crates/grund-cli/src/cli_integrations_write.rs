/// The `--write` half of `grund integrations` (§FS-integrations.4): how one run's
/// answer is reported. The run itself — the client's install, the resolver and
/// the user-level citation guidance synchronized across every agent this machine
/// has (§FS-integrations.4.3.8) — is the engine's `integrations`
/// (§FS-distribution.3.1.3), and this file only renders what it returns
/// (§AR-bindings.3): the user configuration's warnings, then one line per target
/// in the order the run touched them, on stderr, and a manual client's snippet
/// on stdout.
///
/// Where a manual client's step is reported: after the resolver its snippet
/// depends on, and before the guidance (§FS-integrations.3.4.2).
fn report_integrations_write(out: &IntegrationsOutput) {
    for warning in &out.warnings {
        eprintln!(
            "warning: {}:{}: {}",
            warning.path.as_deref().unwrap_or(""),
            warning.line.unwrap_or(0),
            warning.message
        );
    }
    let resolver = RESOLVER_HOME_SUFFIX;
    let split = out
        .events
        .iter()
        .position(|event| event.path.ends_with(resolver))
        .map_or(0, |at| at + 1);
    let split = if out.manual_steps.is_empty() { out.events.len() } else { split };
    report_integration_events(&out.events[..split]);
    for step in &out.manual_steps {
        let target = named_status(out, step.client).config_target;
        eprintln!("manual {} ({target})", step.client.name());
        println!("{}", step.snippet);
    }
    report_integration_events(&out.events[split..]);
}

/// The resolver's path below the home directory, which names its event.
const RESOLVER_HOME_SUFFIX: &str = ".local/bin/grund-open";

fn report_integration_events(events: &[IntegrationEvent]) {
    for event in events {
        let (verb, path) = (event.outcome.name(), event.path.display());
        match &event.note {
            None => eprintln!("{verb} {path}"),
            // Reported rather than silent (§FS-integrations.4.1.3): the install
            // is otherwise indistinguishable from one that works.
            Some(IntegrationNote::UnwiredWezterm { apply_call }) => {
                eprintln!("{verb} {path}");
                eprintln!(
                    "note: {path} does not call `{apply_call}config)` on the config it returns, so WezTerm reads the block and registers nothing; add that line where you build your config"
                );
            }
            Some(IntegrationNote::AgentHomeMissing { home }) => {
                eprintln!("{verb} {path} (no {home})")
            }
            Some(note) => eprintln!("{verb} {path} ({})", describe_guidance(note)),
        }
    }
}

/// What one agent's block ended up teaching, and why it is not what was asked
/// for when it is not (§FS-integrations.4.4.4). Reported per target, because
/// unreported an override, a gate downgrade, and an unread key are
/// indistinguishable from the outside.
fn describe_guidance(note: &IntegrationNote) -> String {
    let IntegrationNote::Guidance {
        rendering,
        target,
        requested,
        overridden,
    } = note
    else {
        return String::new();
    };
    if *rendering == ConversationRendering::Plain {
        return ConversationRendering::Plain.name().to_string();
    }
    let mut described = format!("{} \u{2192} {}", rendering.name(), target.name());
    if target != requested {
        described.push_str(&format!("; {} unverified here", requested.name()));
    } else if *overridden {
        described.push_str("; agent override");
    }
    described
}

/// A run that stopped (§FS-integrations.4.3.14): what it wrote before it, then
/// the target and why, exit `2`. A client whose home cannot be resolved is named
/// by the client rather than by its `~`-rooted target.
fn report_integrations_failure(
    error: &IntegrationsError,
    client: Option<IntegrationClient>,
) -> ExitCode {
    report_integrations_write(&error.output);
    match &error.failure {
        IntegrationsFailure::Filesystem { path, .. }
            if client.is_some_and(|client| {
                named_status(&error.output, client).config_target == path.as_os_str()
            }) =>
        {
            let name = client.map_or("", IntegrationClient::name);
            eprintln!("error: cannot resolve home directory for {name}");
        }
        IntegrationsFailure::Filesystem { path, message } => {
            eprintln!("error: {}: {message}", path.display());
        }
        refusal => eprintln!("error: {}", refusal_message(refusal)),
    }
    ExitCode::from(2)
}
