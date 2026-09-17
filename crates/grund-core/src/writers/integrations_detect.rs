//! Which rendering-layer clients apply to the ambient environment
//! (§FS-integrations.2). Reads only the named environment variables and answers
//! in the frozen client order, so one environment always yields one list — the
//! detection half of the component, apart from the client table it answers over
//! and from the install that acts on it (§AR-core-module-layout.1).

use super::integrations_clients::IntegrationClient;

/// Detect which clients apply from the ambient environment (§FS-integrations.2).
/// Reads only the named variables; results are returned in the frozen client
/// order, deduplicated, so a given environment always yields the same list.
///
/// Why VSCodium is recognized by path and not by variable: VS Code and VSCodium
/// set an identical `VSCODE_*` environment, so presence alone cannot tell them
/// apart. What differs is where those variables point — VSCodium's helper paths
/// live under its own application directory. When that shows, VSCodium is marked
/// *as well*: the extensions roots differ, and installing into the wrong one is
/// silent.
pub(crate) fn detect_clients() -> Vec<IntegrationClient> {
    let has = |name: &str| std::env::var_os(name).is_some_and(|value| !value.is_empty());
    // `IntegrationClient` variants are declared in the same order as `ALL`, so the
    // discriminant is the index into this presence table.
    let mut matched = [false; 6];
    let mut mark = |client: IntegrationClient| matched[client as usize] = true;
    if has("WEZTERM_EXECUTABLE") {
        mark(IntegrationClient::Wezterm);
    }
    if has("KITTY_WINDOW_ID") {
        mark(IntegrationClient::Kitty);
    }
    match std::env::var("TERM_PROGRAM").ok().as_deref() {
        Some("WezTerm") => mark(IntegrationClient::Wezterm),
        Some("iTerm.app") => mark(IntegrationClient::Iterm2),
        Some("tmux") => mark(IntegrationClient::Tmux),
        Some("vscode") => mark(IntegrationClient::Vscode),
        _ => {}
    }
    // §FS-integrations.3.2: presence alone cannot tell VS Code from VSCodium, so
    // a value naming VSCodium's own application directory marks VSCodium as well.
    let vscode_vars: Vec<String> = std::env::vars_os()
        .filter(|(key, _)| {
            key.to_str()
                .is_some_and(|key| key == "VSCODE_PID" || key.starts_with("VSCODE_"))
        })
        .filter_map(|(_, value)| value.to_str().map(str::to_ascii_lowercase))
        .collect();
    if !vscode_vars.is_empty() {
        mark(IntegrationClient::Vscode);
    }
    if vscode_vars.iter().any(|value| value_names_codium(value)) {
        mark(IntegrationClient::Codium);
    }
    if has("TMUX") {
        mark(IntegrationClient::Tmux);
    }
    IntegrationClient::ALL
        .into_iter()
        .enumerate()
        .filter(|(idx, _)| matched[*idx])
        .map(|(_, client)| client)
        .collect()
}

/// Whether a (lowercased) `VSCODE_*` value *names* VSCodium's application
/// directory (§FS-integrations.2) — a path segment that is VSCodium's, not a
/// substring anywhere, so a workspace under `~/codium-notes` does not mark the
/// client. Segment equality plus a `vscodium` infix covers the packagings:
/// `/usr/share/codium`, `VSCodium.app`, `vscodium-bin`, and the
/// `com.vscodium.codium` Flatpak id.
pub(crate) fn value_names_codium(value: &str) -> bool {
    value
        .split(['/', '\\'])
        .any(|segment| segment == "codium" || segment.contains("vscodium"))
}
