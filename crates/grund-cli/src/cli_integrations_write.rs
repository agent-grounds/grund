/// The `--write` half of `grund integrations` (§FS-integrations.4): what one
/// install reports, and how the user-level citation guidance is synchronized
/// across every agent whose instruction file this machine has
/// (§FS-integrations.4.3). Splitting the reporting from the splices and probes
/// it reports on is what lets the whole command sit in the frontend without a
/// size exception (§AR-core-module-layout.3); the splices are the engine's, in
/// `writers/integrations_install.rs`, and return data rather than printing
/// (§AR-bindings.3, §FS-distribution.3.1).
///
/// Apply an integration to disk under `--write` (§FS-integrations.4). Reports on
/// stderr; exit `0` on success, `2` on a newer-block or IO error.
use std::fs;

fn write_integration(
    client: IntegrationClient,
    conversation: Option<ConversationRendering>,
    conversation_target: Option<ConversationTarget>,
    agent: Option<&'static str>,
    user_config: UserConfig,
) -> ExitCode {
    let integration_status = match client.install_kind() {
        InstallKind::Block => write_terminal_integration(client, client.snippet().unwrap_or("")),
        InstallKind::Vscode => write_vscode_integration(client),
        InstallKind::Manual => write_manual_integration(client),
    };
    if integration_status != ExitCode::SUCCESS {
        return integration_status;
    }
    write_user_citation_guidance_command(conversation, conversation_target, agent, user_config)
}

fn write_user_citation_guidance_command(
    conversation: Option<ConversationRendering>,
    conversation_target: Option<ConversationTarget>,
    agent: Option<&'static str>,
    user_config: UserConfig,
) -> ExitCode {
    match write_user_citation_guidance(conversation, conversation_target, agent, user_config) {
        Ok(()) => ExitCode::SUCCESS,
        Err((path, message)) => {
            eprintln!("error: {}: {message}", path.display());
            ExitCode::from(2)
        }
    }
}

/// `--write` for a client with no writable configuration: install the resolver
/// the manual steps depend on, then print those steps (§FS-integrations.3.4).
/// Reported as `manual` rather than a block verb, so a script can tell that a
/// human still has to act.
fn write_manual_integration(client: IntegrationClient) -> ExitCode {
    match write_resolver_script() {
        Ok(Some(path)) => eprintln!("wrote {}", path.display()),
        Ok(None) => {}
        Err((path, message)) => {
            eprintln!("error: {}: {message}", path.display());
            return ExitCode::from(2);
        }
    }
    eprintln!("manual {} ({})", client.name(), client.config_target());
    if let Some(snippet) = client.snippet() {
        println!("{snippet}");
    }
    ExitCode::SUCCESS
}

fn write_terminal_integration(client: IntegrationClient, snippet: &str) -> ExitCode {
    let Some(config_path) = expand_target(client.config_target()) else {
        eprintln!("error: cannot resolve home directory for {}", client.name());
        return ExitCode::from(2);
    };
    let existing = match fs::read_to_string(&config_path) {
        Ok(text) => text,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(err) => {
            eprintln!("error: {}: {err}", config_path.display());
            return ExitCode::from(2);
        }
    };
    let fresh = existing.is_empty();
    let (mut updated, outcome) = match install_managed_block(
        client.comment_prefix(),
        client.prepends_block(),
        &existing,
        snippet,
    ) {
        Ok(result) => result,
        Err(message) => {
            eprintln!("error: {}: {message}", config_path.display());
            return ExitCode::from(2);
        }
    };
    // A file created from scratch gets the client's starter config appended
    // below the block, so the install is usable without hand-editing.
    if fresh && let Some(scaffold) = client.fresh_config_scaffold() {
        updated.push_str(scaffold);
    }
    if outcome != BlockOutcome::Unchanged {
        if let Some(parent) = config_path.parent()
            && let Err(err) = fs::create_dir_all(parent)
        {
            eprintln!("error: {}: {err}", parent.display());
            return ExitCode::from(2);
        }
        if let Err(err) = fs::write(&config_path, &updated) {
            eprintln!("error: {}: {err}", config_path.display());
            return ExitCode::from(2);
        }
    }
    eprintln!("{} {}", block_outcome_verb(outcome), config_path.display());
    // Reported rather than silent (§FS-integrations.4.1): the install is
    // otherwise indistinguishable from one that works.
    if needs_wezterm_wiring(client, &updated) {
        eprintln!(
            "note: {} does not call `{WEZTERM_APPLY_CALL}config)` on the config it returns, so WezTerm reads the block and registers nothing; add that line where you build your config",
            config_path.display()
        );
    }
    match write_resolver_script() {
        Ok(Some(path)) => eprintln!("wrote {}", path.display()),
        Ok(None) => {}
        Err((path, message)) => {
            eprintln!("error: {}: {message}", path.display());
            return ExitCode::from(2);
        }
    }
    ExitCode::SUCCESS
}

/// Materialize the unpacked VS Code extension into the extensions directory
/// (§FS-integrations.4.2). Overwrites only the grund-owned files.
fn write_vscode_integration(client: IntegrationClient) -> ExitCode {
    let Some(dir) = expand_target(client.config_target()) else {
        eprintln!("error: cannot resolve home directory for {}", client.name());
        return ExitCode::from(2);
    };
    let marker = dir.join(".grund-version");
    let current = fs::read_to_string(&marker).ok();
    if vscode_integration_is_current(&dir) {
        eprintln!("exists {}", dir.display());
        return ExitCode::SUCCESS;
    }
    if let Err(err) = fs::create_dir_all(&dir) {
        eprintln!("error: {}: {err}", dir.display());
        return ExitCode::from(2);
    }
    let files = [
        ("package.json", VSCODE_PACKAGE_JSON),
        ("extension.js", VSCODE_EXTENSION_JS),
        (".grund-version", &*format!("{INTEGRATIONS_BLOCK_VERSION}")),
    ];
    for (name, contents) in files {
        let path = dir.join(name);
        if let Err(err) = fs::write(&path, contents) {
            eprintln!("error: {}: {err}", path.display());
            return ExitCode::from(2);
        }
    }
    let verb = if current.is_some() {
        "updated"
    } else {
        "wrote"
    };
    eprintln!("{verb} {}", dir.display());
    ExitCode::SUCCESS
}

/// What `--write` will do to one user-guidance file, decided before any of them
/// is touched (§FS-integrations.4.3).
enum GuidancePlan {
    /// Write these bytes, reporting the block outcome's verb.
    Write(String, BlockOutcome),
    /// The agent is not in use on this machine — report why and write nothing.
    /// Named so the report says which directory would make it apply.
    Skip(&'static str),
    /// An instruction file, reported with the form it received
    /// (§FS-integrations.4.4).
    WriteAgent(String, BlockOutcome, EffectiveForm),
}

/// What one agent's block ended up teaching, and why it is not what was asked
/// for when it is not (§FS-integrations.4.4). Reported per target, because
/// unreported an override, a gate downgrade, and an unread key are
/// indistinguishable from the outside.
struct EffectiveForm {
    rendering: ConversationRendering,
    target: ConversationTarget,
    requested: ConversationTarget,
    overridden: bool,
}

impl EffectiveForm {
    fn describe(&self) -> String {
        if self.rendering == ConversationRendering::Plain {
            return ConversationRendering::Plain.name().to_string();
        }
        let mut note = format!("{} \u{2192} {}", self.rendering.name(), self.target.name());
        if self.target != self.requested {
            note.push_str(&format!("; {} unverified here", self.requested.name()));
        } else if self.overridden {
            note.push_str("; agent override");
        }
        note
    }
}

/// The user configuration `--write` reads, loaded once per invocation so its
/// warnings are reported exactly once (§FS-integrations.4.3).
struct UserConfig {
    path: PathBuf,
    text: String,
    preference: Option<ConversationRendering>,
    target: Option<ConversationTarget>,
    agent_targets: Vec<(String, ConversationTarget)>,
}

/// Read and report on the user configuration without writing anything. Every
/// line grund did not act on is warned about here, at the one point the file is
/// read: nothing else in this file has any effect, and a setting that silently
/// does nothing is indistinguishable from one that works. Only failing to reach
/// the file is an error; its contents never are (§FS-integrations.4.3).
fn load_user_config() -> Result<UserConfig, (PathBuf, String)> {
    let path = user_grund_config_path().ok_or_else(|| {
        (
            PathBuf::from(USER_CONFIG_TARGET),
            "cannot resolve user configuration directory".to_string(),
        )
    })?;
    let text = read_optional_text(&path)?;
    let scan = scan_user_config(&text);
    for (line, message) in &scan.problems {
        eprintln!("warning: {}:{line}: {message}", path.display());
    }
    Ok(UserConfig {
        path,
        text,
        preference: scan.preference,
        target: scan.target,
        agent_targets: scan.agent_targets,
    })
}

/// Persist the machine-local conversation preference and synchronize it into
/// global agent instructions (§FS-integrations.4.3). All files are planned
/// before the first write so malformed managed blocks fail without touching any
/// of these user-guidance targets.
///
/// Why a per-agent override cannot widen the form: the override moves the
/// request, never the verdict, so no key can instruct a form recorded as erasing
/// the citation on that surface.
fn write_user_citation_guidance(
    requested: Option<ConversationRendering>,
    requested_target: Option<ConversationTarget>,
    scoped_agent: Option<&'static str>,
    user_config: UserConfig,
) -> Result<(), (PathBuf, String)> {
    let UserConfig {
        path: config_path,
        text: config_existing,
        preference: stored,
        target: stored_target,
        mut agent_targets,
    } = user_config;
    let effective = requested.or(stored).unwrap_or(ConversationRendering::Plain);
    // A scoped write changes one agent's partial and leaves the base exactly as
    // it was — that is the whole point of the flag (§FS-integrations.4.4).
    let machine_target = if scoped_agent.is_some() {
        stored_target.unwrap_or_default()
    } else {
        requested_target.or(stored_target).unwrap_or_default()
    };
    // Both keys are recorded, and both are recorded even when inert: a machine
    // that set a target under `plain` keeps it when it later switches to `link`
    // (§FS-integrations.1).
    let (config_updated, conversation_outcome) = install_reference_key(
        &config_existing,
        "reference",
        "conversation",
        effective.name(),
        stored == Some(effective),
    );
    let (config_updated, target_outcome) = install_reference_key(
        &config_updated,
        "reference",
        "conversation_target",
        machine_target.name(),
        stored_target == Some(machine_target),
    );
    let mut config_outcome = merge_outcomes(conversation_outcome, target_outcome);
    let mut config_updated = config_updated;
    if let (Some(agent), Some(target)) = (scoped_agent, requested_target) {
        let stored_for_agent = agent_targets
            .iter()
            .find(|(name, _)| name == agent)
            .map(|(_, value)| *value);
        let (next, outcome) = install_reference_key(
            &config_updated,
            &agent_override_table(agent),
            "conversation_target",
            target.name(),
            stored_for_agent == Some(target),
        );
        config_updated = next;
        config_outcome = merge_outcomes(config_outcome, outcome);
        match agent_targets.iter_mut().find(|(name, _)| name == agent) {
            Some(entry) => entry.1 = target,
            None => agent_targets.push((agent.to_string(), target)),
        }
    }

    let mut plans = vec![(
        config_path,
        GuidancePlan::Write(config_updated, config_outcome),
    )];
    for target in GLOBAL_AGENT_INSTRUCTION_TARGETS {
        let path = expand_target(target.file).ok_or_else(|| {
            (
                PathBuf::from(target.file),
                "cannot resolve home directory".to_string(),
            )
        })?;
        // In use when the agent's own directory exists, or when the instruction
        // file already does — the latter keeps a target grund wrote earlier (or
        // the user maintains by hand) synchronized even if the directory check
        // would no longer select it.
        let in_use = expand_target(target.home).is_some_and(|home| home.is_dir()) || path.is_file();
        if !in_use {
            plans.push((path, GuidancePlan::Skip(target.home)));
            continue;
        }
        // §FS-integrations.4.4: the agent's own partial replaces the base, then
        // §DF-conversation-link-target.2.4 gates the result.
        let overridden = agent_targets
            .iter()
            .find(|(name, _)| name == target.agent)
            .map(|(_, value)| *value);
        let requested_for_agent = overridden.unwrap_or(machine_target);
        let gated = target.link_support.resolve(requested_for_agent);
        let form = EffectiveForm {
            rendering: effective,
            target: gated,
            requested: requested_for_agent,
            overridden: overridden.is_some(),
        };
        let existing = read_optional_text(&path)?;
        let (updated, outcome) = install_agent_guidance_block(&existing, effective, gated)
            .map_err(|message| (path.clone(), message))?;
        plans.push((path, GuidancePlan::WriteAgent(updated, outcome, form)));
    }

    for (path, plan) in plans {
        match plan {
            GuidancePlan::Write(updated, outcome) => {
                if outcome != BlockOutcome::Unchanged {
                    if let Some(parent) = path.parent() {
                        fs::create_dir_all(parent)
                            .map_err(|err| (parent.to_path_buf(), err.to_string()))?;
                    }
                    fs::write(&path, updated).map_err(|err| (path.clone(), err.to_string()))?;
                }
                eprintln!("{} {}", block_outcome_verb(outcome), path.display());
            }
            GuidancePlan::WriteAgent(updated, outcome, form) => {
                if outcome != BlockOutcome::Unchanged {
                    if let Some(parent) = path.parent() {
                        fs::create_dir_all(parent)
                            .map_err(|err| (parent.to_path_buf(), err.to_string()))?;
                    }
                    fs::write(&path, updated).map_err(|err| (path.clone(), err.to_string()))?;
                }
                eprintln!(
                    "{} {} ({})",
                    block_outcome_verb(outcome),
                    path.display(),
                    form.describe()
                );
            }
            GuidancePlan::Skip(home) => {
                eprintln!("skipped {} (no {home})", path.display());
            }
        }
    }
    Ok(())
}
