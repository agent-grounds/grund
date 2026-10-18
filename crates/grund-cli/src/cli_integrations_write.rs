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

/// Whether a run stopped writing the resolver: at the script, or at the
/// directory that holds it.
fn resolver_failed(failure: &IntegrationsFailure) -> bool {
    let IntegrationsFailure::Filesystem { path, .. } = failure else {
        return false;
    };
    let resolver = Path::new(RESOLVER_HOME_SUFFIX);
    path.ends_with(resolver) || resolver.parent().is_some_and(|dir| path.ends_with(dir))
}

fn report_integration_events(events: &[IntegrationEvent]) {
    for event in events {
        eprint!("{}", event_lines(event.outcome.name(), &event.path, event.note.as_ref()));
    }
}

/// One event's lines on stderr: `<verb> <path>`, with what its note adds
/// (§FS-integrations.1.3.2). A note this release does not name annotates a write
/// that already happened, so its event prints the bare line, as a note-less one
/// does (§AR-bindings.2.1).
fn event_lines(verb: &str, path: &Path, note: Option<&IntegrationNote>) -> String {
    let path = path.display();
    match note {
        // Reported rather than silent (§FS-integrations.4.1.3): the install
        // is otherwise indistinguishable from one that works.
        Some(IntegrationNote::UnwiredWezterm { apply_call }) => format!(
            "{verb} {path}\nnote: {path} does not call `{apply_call}config)` on the config it returns, so WezTerm reads the block and registers nothing; add that line where you build your config\n"
        ),
        Some(IntegrationNote::AgentHomeMissing { home }) => format!("{verb} {path} (no {home})\n"),
        Some(IntegrationNote::Guidance {
            rendering,
            target,
            requested,
            overridden,
        }) => format!(
            "{verb} {path} ({})\n",
            describe_guidance(*rendering, *target, *requested, *overridden)
        ),
        // A note this release does not name (§AR-bindings.2.1): the write happened.
        None | Some(_) => format!("{verb} {path}\n"),
    }
}

/// What one agent's block ended up teaching, and why it is not what was asked
/// for when it is not (§FS-integrations.4.4.4). Reported per target, because
/// unreported an override, a gate downgrade, and an unread key are
/// indistinguishable from the outside.
fn describe_guidance(
    rendering: ConversationRendering,
    target: ConversationTarget,
    requested: ConversationTarget,
    overridden: bool,
) -> String {
    if rendering == ConversationRendering::Plain {
        return ConversationRendering::Plain.name().to_string();
    }
    let mut described = format!("{} \u{2192} {}", rendering.name(), target.name());
    if target != requested {
        described.push_str(&format!("; {} unverified here", requested.name()));
    } else if overridden {
        described.push_str("; agent override");
    }
    described
}

/// A run that stopped (§FS-integrations.4.3.14): what it wrote before it, then
/// the target and why, exit `2`. A client whose home cannot be resolved is named
/// by the client rather than by its `~`-rooted target, and a manual client's
/// step is not printed when the resolver it depends on is what failed
/// (§FS-integrations.3.4.2).
fn report_integrations_failure(
    error: &IntegrationsError,
    client: Option<IntegrationClient>,
) -> ExitCode {
    if resolver_failed(&error.failure) {
        let mut output = error.output.clone();
        output.manual_steps.clear();
        report_integrations_write(&output);
    } else {
        report_integrations_write(&error.output);
    }
    match unresolved_home(error, client) {
        Some(client) => eprintln!("error: cannot resolve home directory for {}", client.name()),
        None => eprintln!("error: {}", refusal_message(&error.failure)),
    }
    ExitCode::from(2)
}

/// The named client, when the run stopped because its `~`-rooted target has no
/// home directory to resolve it (§FS-integrations.4.3.14).
fn unresolved_home(
    error: &IntegrationsError,
    client: Option<IntegrationClient>,
) -> Option<IntegrationClient> {
    let IntegrationsFailure::Filesystem { path, .. } = &error.failure else {
        return None;
    };
    client.filter(|client| named_status(&error.output, *client).config_target == path.as_os_str())
}
