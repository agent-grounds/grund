/// The deprecated `main_entry()` adapter for `grund integrations`
/// (§AR-bindings.2): the argument parsing, the entry point both CLI frontends
/// call, and the printing forms — detection, the machine-shaped plans, and a
/// client's artifact read before installing (§FS-integrations). The engine
/// renders here only because the published CLI still imports this one function
/// (§AR-system.2.9); everything it decides over lives in `writers/` — the
/// client set it prints, the detection it reports, and the installs the
/// `--write` half beside this file carries out.
///
/// The user-facing setup guide, named by detection (§FS-integrations.2). It
/// carries what `--write` cannot do for the caller: the prerequisites that fail
/// silently, and the per-client manual step.
use anyhow::Result;
use std::process::ExitCode;

use super::integrations_write::{
    load_user_config, write_integration, write_user_citation_guidance_command,
};
use crate::model::json_escape;
use crate::writers::{
    ConversationRendering, ConversationTarget, GRUND_OPEN_RESOLVER, IntegrationClient,
    RESOLVER_TARGET, VSCODE_EXTENSION_JS, VSCODE_PACKAGE_JSON, detect_clients,
    integration_is_current, known_agent, known_agents_list, known_clients_line,
};

pub(crate) const SETUP_GUIDE_URL: &str =
    "https://github.com/agent-grounds/grund/blob/main/docs/user-facing/clickable-citations.md";

/// Parsed `grund integrations` invocation.
pub(crate) struct IntegrationsInvocation {
    pub(crate) client: Option<IntegrationClient>,
    pub(crate) write: bool,
    json: bool,
    pub(crate) conversation: Option<ConversationRendering>,
    pub(crate) conversation_target: Option<ConversationTarget>,
    /// `--agent <name>`: scope `conversation_target` to one agent instead of
    /// the machine (§FS-integrations.4.4).
    pub(crate) agent: Option<&'static str>,
}

/// Parse args, or return an error `ExitCode` after printing a CLI-level message.
pub(crate) fn parse_integrations_args(args: &[String]) -> Result<IntegrationsInvocation, ExitCode> {
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
    let conversation = match conversation.as_deref() {
        None => None,
        Some(value) => match ConversationRendering::from_name(value) {
            Some(value) => Some(value),
            None => {
                eprintln!("error: --conversation must be one of plain | link");
                return Err(ExitCode::from(2));
            }
        },
    };
    let conversation_target = match conversation_target.as_deref() {
        None => None,
        Some(value) => match ConversationTarget::from_name(value) {
            Some(value) => Some(value),
            None => {
                eprintln!(
                    "error: --conversation-target must be one of {}",
                    ConversationTarget::accepted_list()
                );
                return Err(ExitCode::from(2));
            }
        },
    };
    // §FS-integrations.4.4: `--agent` scopes `--conversation-target` and
    // nothing else, so an agent with no target is an error rather than a
    // silent no-op.
    let agent = match agent.as_deref() {
        None => None,
        Some(value) => match known_agent(value) {
            Some(known) => Some(known),
            None => {
                eprintln!(
                    "error: unknown agent `{value}`; known agents: {}",
                    known_agents_list()
                );
                return Err(ExitCode::from(2));
            }
        },
    };
    if agent.is_some() && !write {
        eprintln!("error: --agent requires --write");
        return Err(ExitCode::from(2));
    }
    if agent.is_some() && conversation_target.is_none() {
        eprintln!("error: --agent requires --conversation-target");
        return Err(ExitCode::from(2));
    }
    if conversation.is_some() && !write {
        eprintln!("error: --conversation requires --write");
        return Err(ExitCode::from(2));
    }
    if conversation_target.is_some() && !write {
        eprintln!("error: --conversation-target requires --write");
        return Err(ExitCode::from(2));
    }
    // Either conversation flag is enough to make the clientless form
    // unambiguous: it updates the preference and the instruction blocks and
    // installs no arbitrary client (§FS-integrations.1).
    if write && client.is_none() && conversation.is_none() && conversation_target.is_none() {
        eprintln!("error: integrations --write requires a client or --conversation");
        eprintln!("{}", known_clients_line());
        return Err(ExitCode::from(2));
    }
    if write && json {
        // `--write` reports what it changed on stderr; there is no JSON install
        // plan. Reject rather than run the side effect and silently drop the
        // format the caller asked for.
        eprintln!("error: integrations --write does not support --format json");
        return Err(ExitCode::from(2));
    }
    Ok(IntegrationsInvocation {
        client,
        write,
        json,
        conversation,
        conversation_target,
        agent,
    })
}

/// The `grund integrations` entry point, called from both CLI frontends
/// (§FS-integrations). Prints by default; writes only under `--write`.
///
/// Why the user configuration is read before the first artifact is installed:
/// its warnings are then reported once, and a file grund cannot parse fails the
/// command outright rather than after a client's config and the resolver are
/// already on disk.
pub fn run_integrations(args: &[String]) -> ExitCode {
    let invocation = match parse_integrations_args(args) {
        Ok(invocation) => invocation,
        Err(code) => return code,
    };
    // §FS-integrations.4.3: `--write` reads the user configuration exactly once,
    // before any artifact is installed.
    if invocation.write {
        let user_config = match load_user_config() {
            Ok(config) => config,
            Err((path, message)) => {
                eprintln!("error: {}: {message}", path.display());
                return ExitCode::from(2);
            }
        };
        return match invocation.client {
            Some(client) => write_integration(
                client,
                invocation.conversation,
                invocation.conversation_target,
                invocation.agent,
                user_config,
            ),
            None => write_user_citation_guidance_command(
                invocation.conversation,
                invocation.conversation_target,
                invocation.agent,
                user_config,
            ),
        };
    }
    match invocation.client {
        None => print_detection(invocation.json),
        Some(client) if invocation.json => {
            print!("{}", client_descriptor_json(client));
            ExitCode::SUCCESS
        }
        Some(client) => {
            print_client_artifact(client);
            ExitCode::SUCCESS
        }
    }
}

/// No-client detection print (§FS-integrations.2). Environment-dependent, so it
/// is never goldened; exit is always `0`.
fn print_detection(json: bool) -> ExitCode {
    let detected = detect_clients();
    if json {
        print!("{}", detection_plan_json(&detected));
        return ExitCode::SUCCESS;
    }
    if detected.is_empty() {
        println!("No supported terminal or editor detected. Available integrations:");
        for client in IntegrationClient::ALL {
            println!("  {:<8} {}", client.name(), client.install_command());
        }
    } else {
        println!("Detected integrations for this environment:");
        for client in detected {
            println!("  {:<8} {}", client.name(), client.install_command());
        }
    }
    println!();
    println!("Run `grund integrations <client>` to preview one before installing.");
    // The prerequisites and the per-client manual step live in the guide, and
    // this is the command a user reaches first (§FS-integrations.2).
    println!("Setup guide: {SETUP_GUIDE_URL}");
    ExitCode::SUCCESS
}

/// The machine-shaped detection plan (§FS-integrations.5): detected clients in
/// frozen order, then every client with whether it was detected and its install.
pub(crate) fn detection_plan_json(detected: &[IntegrationClient]) -> String {
    let detected_names = detected
        .iter()
        .map(|client| format!("\"{}\"", client.name()))
        .collect::<Vec<_>>()
        .join(",");
    let clients = IntegrationClient::ALL
        .iter()
        .map(|client| {
            // `install_kind` is what lets a caller tell a manual client's
            // "not knowable" from a real "not installed" (§FS-integrations.3.4).
            format!(
                "{{\"client\":\"{}\",\"detected\":{},\"installed\":{},\"install_kind\":\"{}\",\"install\":\"{}\"}}",
                client.name(),
                detected.contains(client),
                integration_is_current(*client),
                client.install_kind().name(),
                json_escape(&client.install_command()),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("{{\"detected\":[{detected_names}],\"clients\":[{clients}]}}\n")
}

/// One JSON object describing a client's artifact and its `--write` targets,
/// without printing the artifact bytes (§FS-integrations.5).
pub(crate) fn client_descriptor_json(client: IntegrationClient) -> String {
    let kind = if client.is_terminal() {
        "terminal"
    } else {
        "editor"
    };
    let resolver = if client.is_terminal() {
        format!(",\"resolver_target\":\"{}\"", json_escape(RESOLVER_TARGET))
    } else {
        String::new()
    };
    format!(
        "{{\"client\":\"{}\",\"kind\":\"{}\",\"install\":\"{}\",\"install_kind\":\"{}\",\"config_target\":\"{}\"{}}}\n",
        client.name(),
        kind,
        json_escape(&client.install_command()),
        client.install_kind().name(),
        json_escape(client.config_target()),
        resolver,
    )
}

/// Print a client's artifact for a human to read before installing
/// (§FS-integrations.3). Deterministic and environment-independent.
fn print_client_artifact(client: IntegrationClient) {
    match client.snippet() {
        Some(snippet) => {
            println!("# grund {} citation integration", client.name());
            println!("# Install with: {}", client.install_command());
            println!("#");
            println!("# 1. Terminal config — add to {}:", client.config_target());
            println!();
            print!("{snippet}");
            println!();
            println!(
                "# 2. Resolver — install grund-open to a directory on PATH (e.g. ~/.local/bin):"
            );
            println!();
            print!("{GRUND_OPEN_RESOLVER}");
        }
        None => {
            // vscode: print the unpacked extension source.
            println!("# grund {} terminal-citations extension", client.name());
            println!("# Install with: {}", client.install_command());
            println!("#");
            println!("# package.json:");
            println!();
            print!("{VSCODE_PACKAGE_JSON}");
            println!();
            println!("# extension.js:");
            println!();
            print!("{VSCODE_EXTENSION_JS}");
        }
    }
}
