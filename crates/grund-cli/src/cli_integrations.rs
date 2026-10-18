/// `grund integrations` (§FS-integrations.1): the argument parsing, the command
/// the dispatcher calls, and the printing forms — detection, the machine-shaped
/// plans, and a client's artifact read before installing (§FS-integrations).
/// Rendering is the frontend's, like every other subcommand's (§AR-bindings.3);
/// everything it renders is the answer of the engine's one `integrations`
/// operation (§FS-distribution.3.1.3), and the `--write` report beside this file
/// renders the same answer.
///
/// The user-facing setup guide, named by detection (§FS-integrations.2.1). It
/// carries what `--write` cannot do for the caller: the prerequisites that fail
/// silently, and the per-client manual step.
const SETUP_GUIDE_URL: &str =
    "https://github.com/agent-grounds/grund/blob/main/docs/user-facing/clickable-citations.md";

/// Parsed `grund integrations` invocation: the engine's options, and the format
/// that is the CLI's own.
struct IntegrationsInvocation {
    opts: IntegrationsOpts,
    json: bool,
}

/// `known clients: …`, the line the client errors close on.
fn known_clients_line() -> String {
    let names = IntegrationClient::ALL.map(IntegrationClient::name);
    format!("known clients: {}", names.join(", "))
}

/// `grund integrations <client> --write`: a command line, so the CLI's own
/// (§FS-integrations.1.3.2).
fn install_command(client: IntegrationClient) -> String {
    format!("grund integrations {} --write", client.name())
}

/// Parse args, or return an error `ExitCode` after printing a CLI-level message.
fn parse_integrations_args(args: &[String]) -> Result<IntegrationsInvocation, ExitCode> {
    let mut client = None;
    let mut write = false;
    let mut format: Option<String> = None;
    let mut conversation: Option<String> = None;
    let mut conversation_target: Option<String> = None;
    let mut agent: Option<String> = None;
    let mut idx = 0;
    while idx < args.len() {
        match args[idx].as_str() {
            "--write" => write = true,
            "--conversation" => {
                idx += 1;
                if idx >= args.len() {
                    eprintln!("error: --conversation requires a value");
                    return Err(ExitCode::from(2));
                }
                conversation = Some(args[idx].clone());
            }
            other if other.starts_with("--conversation=") => {
                conversation = Some(other.trim_start_matches("--conversation=").to_string());
            }
            "--agent" => {
                idx += 1;
                if idx >= args.len() {
                    eprintln!("error: --agent requires a value");
                    return Err(ExitCode::from(2));
                }
                agent = Some(args[idx].clone());
            }
            other if other.starts_with("--agent=") => {
                agent = Some(other.trim_start_matches("--agent=").to_string());
            }
            "--conversation-target" => {
                idx += 1;
                if idx >= args.len() {
                    eprintln!("error: --conversation-target requires a value");
                    return Err(ExitCode::from(2));
                }
                conversation_target = Some(args[idx].clone());
            }
            other if other.starts_with("--conversation-target=") => {
                conversation_target = Some(
                    other
                        .trim_start_matches("--conversation-target=")
                        .to_string(),
                );
            }
            "--format" => {
                idx += 1;
                if idx >= args.len() {
                    eprintln!("error: --format requires a value");
                    return Err(ExitCode::from(2));
                }
                format = Some(args[idx].clone());
            }
            other if other.starts_with("--format=") => {
                format = Some(other.trim_start_matches("--format=").to_string());
            }
            other if other.starts_with('-') => {
                eprintln!("error: unknown flag `{other}`");
                return Err(ExitCode::from(2));
            }
            other => {
                if client.is_some() {
                    eprintln!("error: integrations takes at most one client argument");
                    return Err(ExitCode::from(2));
                }
                match IntegrationClient::from_name(other) {
                    Some(parsed) => client = Some(parsed),
                    None => {
                        eprintln!("error: unknown integration client `{other}`");
                        eprintln!("{}", known_clients_line());
                        return Err(ExitCode::from(2));
                    }
                }
            }
        }
        idx += 1;
    }
    let json = match format.as_deref() {
        None | Some("text") => false,
        Some("json") => true,
        Some(other) => {
            eprintln!("error: unsupported integrations format `{other}`");
            return Err(ExitCode::from(2));
        }
    };
    let mut opts = IntegrationsOpts::default();
    opts.client = client;
    opts.write = write;
    opts.conversation = match conversation.as_deref() {
        None => None,
        Some(value) => match ConversationRendering::from_name(value) {
            Some(value) => Some(value),
            None => {
                eprintln!("error: --conversation must be one of plain | link");
                return Err(ExitCode::from(2));
            }
        },
    };
    opts.conversation_target = match conversation_target.as_deref() {
        None => None,
        Some(value) => match ConversationTarget::from_name(value) {
            Some(value) => Some(value),
            None => {
                let accepted = ConversationTarget::ALL.map(ConversationTarget::name);
                eprintln!(
                    "error: --conversation-target must be one of {}",
                    accepted.join(" | ")
                );
                return Err(ExitCode::from(2));
            }
        },
    };
    opts.agent = match agent.as_deref() {
        None => None,
        Some(value) => match IntegrationAgent::from_name(value) {
            Some(known) => Some(known),
            None => {
                let known = IntegrationAgent::ALL.map(IntegrationAgent::name);
                eprintln!(
                    "error: unknown agent `{value}`; known agents: {}",
                    known.join(", ")
                );
                return Err(ExitCode::from(2));
            }
        },
    };
    // §FS-integrations.6.2: the engine refuses these combinations too, and in
    // this order; the CLI asks first so it can name its own flags.
    if let Some(refusal) = opts.refusal() {
        eprintln!("error: {}", refusal_message(&refusal));
        if matches!(refusal, IntegrationsFailure::WriteWithoutSubject) {
            eprintln!("{}", known_clients_line());
        }
        return Err(ExitCode::from(2));
    }
    if write && json {
        // `--write` reports what it changed on stderr; there is no JSON install
        // plan. Reject rather than run the side effect and silently drop the
        // format the caller asked for.
        eprintln!("error: integrations --write does not support --format json");
        return Err(ExitCode::from(2));
    }
    Ok(IntegrationsInvocation { opts, json })
}

/// Why a run was refused, in the CLI's words: a refused combination in its flag
/// names (§FS-integrations.6.2), a target that could not be read or written as
/// its path and the filesystem's message, and a refusal this release does not
/// name in core's own words, which the caller reports as an operational failure
/// (§AR-bindings.2.1).
fn refusal_message(refusal: &IntegrationsFailure) -> String {
    match refusal {
        IntegrationsFailure::AgentWithoutWrite => "--agent requires --write".into(),
        IntegrationsFailure::AgentWithoutTarget => {
            "--agent requires --conversation-target".into()
        }
        IntegrationsFailure::ConversationWithoutWrite => {
            "--conversation requires --write".into()
        }
        IntegrationsFailure::ConversationTargetWithoutWrite => {
            "--conversation-target requires --write".into()
        }
        IntegrationsFailure::WriteWithoutSubject => {
            "integrations --write requires a client or --conversation".into()
        }
        IntegrationsFailure::Filesystem { path, message } => {
            format!("{}: {message}", path.display())
        }
        unnamed => unnamed.to_string(),
    }
}

/// The `grund integrations` command (§FS-integrations). Prints by default;
/// writes only under `--write`, and then reports what the engine's run did
/// (§FS-integrations.4).
fn command_integrations(args: &[String]) -> ExitCode {
    let IntegrationsInvocation { opts, json } = match parse_integrations_args(args) {
        Ok(invocation) => invocation,
        Err(code) => return code,
    };
    let out = match integrations(opts) {
        Ok(out) => out,
        Err(error) => return report_integrations_failure(&error, opts.client),
    };
    if opts.write {
        report_integrations_write(&out);
        return ExitCode::SUCCESS;
    }
    match out.artifact {
        None => print_detection(&out, json),
        Some(_) if json => print!("{}", client_descriptor_json(&out)),
        Some(artifact) => print_client_artifact(&artifact, &out),
    }
    ExitCode::SUCCESS
}

/// No-client detection print (§FS-integrations.2.2). Environment-dependent, so it
/// is never goldened; exit is always `0`.
fn print_detection(out: &IntegrationsOutput, json: bool) {
    if json {
        print!("{}", detection_plan_json(out));
        return;
    }
    if out.detected.is_empty() {
        println!("No supported terminal or editor detected. Available integrations:");
        for client in IntegrationClient::ALL {
            println!("  {:<8} {}", client.name(), install_command(client));
        }
    } else {
        println!("Detected integrations for this environment:");
        for client in &out.detected {
            println!("  {:<8} {}", client.name(), install_command(*client));
        }
    }
    println!();
    println!("Run `grund integrations <client>` to preview one before installing.");
    // The prerequisites and the per-client manual step live in the guide, and
    // this is the command a user reaches first (§FS-integrations.2.1).
    println!("Setup guide: {SETUP_GUIDE_URL}");
}

/// The machine-shaped detection plan (§FS-integrations.5): detected clients in
/// frozen order, then every client with whether it was detected and its install.
fn detection_plan_json(out: &IntegrationsOutput) -> String {
    let detected_names = out
        .detected
        .iter()
        .map(|client| format!("\"{}\"", client.name()))
        .collect::<Vec<_>>()
        .join(",");
    let clients = out
        .clients
        .iter()
        .map(|status| {
            // `install_kind` is what lets a caller tell a manual client's
            // "not knowable" from a real "not installed" (§FS-integrations.3.4.2).
            format!(
                "{{\"client\":\"{}\",\"detected\":{},\"installed\":{},\"install_kind\":\"{}\",\"install\":\"{}\"}}",
                status.client.name(),
                status.detected,
                status.installed,
                status.install_kind.name(),
                json_escape(&install_command(status.client)),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("{{\"detected\":[{detected_names}],\"clients\":[{clients}]}}\n")
}

/// The named client's detection status.
fn named_status(out: &IntegrationsOutput, client: IntegrationClient) -> &IntegrationClientStatus {
    out.clients
        .iter()
        .find(|status| status.client == client)
        .expect("every client has a status")
}

/// One JSON object describing a client's artifact and its `--write` targets,
/// without printing the artifact bytes (§FS-integrations.5.1).
fn client_descriptor_json(out: &IntegrationsOutput) -> String {
    let artifact = out.artifact.expect("a named client");
    let status = named_status(out, artifact.client);
    let kind = if artifact.resolver.is_some() {
        "terminal"
    } else {
        "editor"
    };
    let resolver = artifact
        .resolver_target
        .map(|target| format!(",\"resolver_target\":\"{}\"", json_escape(target)))
        .unwrap_or_default();
    format!(
        "{{\"client\":\"{}\",\"kind\":\"{}\",\"install\":\"{}\",\"install_kind\":\"{}\",\"config_target\":\"{}\"{}}}\n",
        artifact.client.name(),
        kind,
        json_escape(&install_command(artifact.client)),
        status.install_kind.name(),
        json_escape(status.config_target),
        resolver,
    )
}

/// Print a client's artifact for a human to read before installing
/// (§FS-integrations.3). Deterministic and environment-independent.
fn print_client_artifact(artifact: &IntegrationArtifact, out: &IntegrationsOutput) {
    let client = artifact.client;
    match artifact.snippet {
        Some(snippet) => {
            println!("# grund {} citation integration", client.name());
            println!("# Install with: {}", install_command(client));
            println!("#");
            println!(
                "# 1. Terminal config — add to {}:",
                named_status(out, client).config_target
            );
            println!();
            print!("{snippet}");
            println!();
            println!(
                "# 2. Resolver — install grund-open to a directory on PATH (e.g. ~/.local/bin):"
            );
            println!();
            print!("{}", artifact.resolver.unwrap_or(""));
        }
        None => {
            // vscode: print the unpacked extension source.
            println!("# grund {} terminal-citations extension", client.name());
            println!("# Install with: {}", install_command(client));
            println!("#");
            println!("# package.json:");
            println!();
            print!("{}", artifact.package_json.unwrap_or(""));
            println!();
            println!("# extension.js:");
            println!();
            print!("{}", artifact.extension_js.unwrap_or(""));
        }
    }
}
