//! Install one rendering-layer integration the way an embedder does: ask
//! `grund_core::integrations` once and read its answer
//! (§FS-distribution.3.1.3). Each event is printed as its outcome verb and the
//! path it touched (§FS-integrations.1.3.2), and its note, if it has one, on
//! stderr; a refusal is printed as one `error:` line and exit `2`
//! (§FS-integrations.1.3.3).
//!
//! ```sh
//! cargo run -p grund-rust-api-example -- kitty --write
//! ```

use std::process::ExitCode;

use grund_core::{IntegrationClient, IntegrationNote, IntegrationsOpts, integrations};

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(client) = args
        .next()
        .as_deref()
        .and_then(IntegrationClient::from_name)
    else {
        eprintln!("usage: grund-rust-api-example <client> [--write]");
        return ExitCode::from(2);
    };
    let mut opts = IntegrationsOpts::default();
    opts.client = Some(client);
    opts.write = args.any(|arg| arg == "--write");
    match integrations(opts) {
        Ok(out) => {
            for event in &out.events {
                println!("{} {}", event.outcome.name(), event.path.display());
                if let Some(note) = &event.note {
                    eprintln!("note: {}: {}", event.path.display(), describe(note));
                }
            }
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(2)
        }
    }
}

/// What a note says. `IntegrationNote` is `#[non_exhaustive]`: a later
/// grund-core may add a note, so the match ends in a wildcard arm
/// (§FS-distribution.3.1.2).
fn describe(note: &IntegrationNote) -> String {
    match note {
        IntegrationNote::UnwiredWezterm { apply_call } => {
            format!("the config never calls `{apply_call}config)`, so the block does nothing")
        }
        IntegrationNote::Guidance {
            rendering, target, ..
        } => format!("guidance as {} for {}", rendering.name(), target.name()),
        IntegrationNote::AgentHomeMissing { home } => format!("{home} is absent, so no agent here"),
        _ => "a note this example does not know".to_string(),
    }
}
