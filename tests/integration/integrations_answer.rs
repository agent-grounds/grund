//! §FS-integrations.1.3.2, §FS-integrations.1.3.3, §AR-bindings.3 — what the
//! engine's `integrations` answers, read as data rather than as the bytes a
//! frontend renders: a manual client's `write` lists its step beside the
//! resolver it wrote, and a run the resolver stops still carries that step.
//!
//! The run reads `HOME` and `XDG_CONFIG_HOME`, so this binary holds one test
//! and points both at a sandbox before anything reads them.
#![cfg(unix)]

#[path = "binaries.rs"]
mod binaries;

use std::fs;
use std::path::{Path, PathBuf};

use grund_core::{
    InstallKind, IntegrationArtifact, IntegrationClient, IntegrationManualStep, IntegrationOutcome,
    IntegrationsFailure, IntegrationsOpts, integrations,
};

fn sandbox(case: &str) -> PathBuf {
    let root = binaries::repo_root()
        .join("target/integration-work/integrations-answer")
        .join(case);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("home")).expect("create sandbox HOME");
    fs::create_dir_all(root.join("xdg")).expect("create sandbox XDG_CONFIG_HOME");
    // SAFETY: the only test in this binary, so no other thread reads the
    // environment while it changes.
    unsafe {
        std::env::set_var("HOME", root.join("home"));
        std::env::set_var("XDG_CONFIG_HOME", root.join("xdg"));
    }
    root
}

fn iterm2_write() -> IntegrationsOpts {
    let mut opts = IntegrationsOpts::default();
    opts.client = Some(IntegrationClient::Iterm2);
    opts.write = true;
    opts
}

/// The one step names the client and the snippet its artifact carries, read
/// field by field: a promised record is read, never built (§FS-distribution.3.1.2).
fn assert_iterm2_step(steps: &[IntegrationManualStep], artifact: Option<IntegrationArtifact>) {
    let snippet = artifact.and_then(|artifact| artifact.snippet);
    let [step] = steps else {
        panic!("one manual step, not {steps:?}");
    };
    assert_eq!(step.client, IntegrationClient::Iterm2);
    assert_eq!(
        step.snippet,
        snippet.expect("iterm2's artifact carries its snippet")
    );
}

#[test]
fn a_manual_clients_step_is_in_the_answer_and_in_the_refusal() {
    // §FS-integrations.1.3.2: facts only — every client's status, the named
    // client's artifact, the events in order, and the step left to the user.
    let root = sandbox("written");
    let out = integrations(iterm2_write()).expect("iterm2 --write");
    assert_eq!(out.clients.len(), IntegrationClient::ALL.len());
    let status = out
        .clients
        .iter()
        .find(|status| status.client == IntegrationClient::Iterm2)
        .expect("iterm2's status");
    assert_eq!(status.install_kind, InstallKind::Manual);
    assert!(!status.installed, "a manual client never claims installed");
    assert_eq!(
        out.artifact.map(|a| a.client),
        Some(IntegrationClient::Iterm2)
    );
    let resolver = root.join("home/.local/bin/grund-open");
    let events: Vec<(&Path, IntegrationOutcome)> = out
        .events
        .iter()
        .map(|event| (event.path.as_path(), event.outcome))
        .collect();
    // The client half comes first; then the guidance, which here finds no
    // agent home and skips every instruction target.
    assert_eq!(events[0], (resolver.as_path(), IntegrationOutcome::Wrote));
    assert!(
        events[2..]
            .iter()
            .all(|(_, outcome)| *outcome == IntegrationOutcome::Skipped),
        "{events:?}"
    );
    assert_iterm2_step(&out.manual_steps, out.artifact);

    // §FS-integrations.1.3.3: the resolver's directory is a file, so the run
    // stops there; the refusal names it with the filesystem's message, and
    // still carries the step the run reached before it.
    let root = sandbox("resolver-fails");
    fs::create_dir_all(root.join("home/.local")).expect("create .local");
    fs::write(root.join("home/.local/bin"), "not a directory\n").expect("block .local/bin");
    let error = integrations(iterm2_write()).expect_err("the resolver cannot be written");
    let IntegrationsFailure::Filesystem { path, message } = &error.failure else {
        panic!("not a filesystem failure: {:?}", error.failure);
    };
    assert_eq!(path, &root.join("home/.local/bin"));
    assert!(!message.is_empty(), "the filesystem's message is kept");
    assert!(error.output.events.is_empty(), "nothing was written");
    assert_iterm2_step(&error.output.manual_steps, error.output.artifact);
}
