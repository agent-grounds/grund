//! The closed set of rendering-layer clients grund ships an integration for
//! (§FS-integrations.1), and what each one is: the artifact bytes, how `--write`
//! applies them, the `~`-rooted target each lands at, and the line-comment token
//! its host config is written in. The ordering is frozen, because detection and
//! every listing print it (§FS-integrations.2). Every artifact is embedded in the
//! binary, printed on demand, and installed only under `--write` as a managed
//! marked block — the `completions`/`init` ethos (§DF-integrations-command).
//!
//! None of the payloads below cites anything. `--write` installs them into a
//! user's dotfiles, where an ID of this repository names nothing that user has
//! (§REQ-shipped-surfaces.1), so the grounding lives here instead — one doc
//! comment per artifact, saying what each file is answering to, checked like
//! every other citation in this tree (§REQ-shipped-surfaces.2).

use std::path::PathBuf;

/// The resolver every client shells out to, and the one artifact `--write`
/// installs for all of them (§FS-integrations.3.4.2). It takes the clicked token,
/// tells a citation from the `§<ID> path:line` location an agent prints beside
/// one (§FS-integrations.3.1.8), finds the config root by climbing under either
/// discovery name (§FS-config.1), reads `path` and `line` out of `--format
/// json` — including the manifest shape a directory-backed case answers with
/// (§FS-show.2.4.2) — and makes the path absolute, which grund's own reports
/// never do (§FS-errors.4).
pub const GRUND_OPEN_RESOLVER: &str = include_str!("../../assets/integrations/grund-open");

/// iTerm2 keeps its settings in a binary plist, so this one is printed for the
/// user to apply by hand rather than spliced; `--write` still installs the
/// resolver it depends on (§FS-integrations.3.4.2). Its Smart Selection rule
/// stays citation-only because Semantic History already opens a bare
/// `path:line`, which is what the `link` preference makes agents print
/// (§FS-integrations.4.3.2) — the location matcher the other terminals need
/// (§FS-integrations.3.1.8) would only compete with it.
pub(crate) const ITERM2_SNIPPET: &str = include_str!("../../assets/integrations/iterm2.txt");

/// WezTerm: one hyperlink rule for the citation and one for the location beside
/// it, registered location-first so the whole `path:line` wins over an
/// ID-shaped fragment inside it (§FS-integrations.3.1.9). WezTerm has no hover
/// event, so peek opens a split pane (§FS-integrations.3.3.2).
pub(crate) const WEZTERM_SNIPPET: &str = include_str!("../../assets/integrations/wezterm.lua");

/// kitty: the `hints` kitten over the visible screen, matching the location
/// ahead of the citation for the same reason WezTerm registers it first
/// (§FS-integrations.3.1.9), with an overlay window as the read-without-leaving
/// path kitty's missing hover leaves it (§FS-integrations.3.3.2).
pub(crate) const KITTY_SNIPPET: &str = include_str!("../../assets/integrations/kitty.conf");

/// tmux cannot make text clickable at all, so a prefix key hands the copy
/// buffer — citation or location alike, the resolver tells them apart
/// (§FS-integrations.3.1.8) — to `grund-open`, and a second key reads the
/// declaration in a popup over the session (§FS-integrations.3.3.2).
const TMUX_SNIPPET: &str = include_str!("../../assets/integrations/tmux.conf");

pub const VSCODE_PACKAGE_JSON: &str = include_str!("../../assets/integrations/vscode/package.json");

/// VS Code is the one client that can show a declaration on *hover* rather than
/// on click (§FS-integrations.3.3.1), and one `--brief` resolution serves both
/// (§FS-integrations.3.2.1). The provider mirrors grund's own config-root climb
/// (§FS-config.3.6) under both discovery names (§FS-config.1), so a reported
/// path is joined against the root that produced it, and reveals a
/// directory-backed case in the Explorer, having no line to open
/// (§FS-show.2.4.2). Its document and terminal matchers, and the strip before a
/// resolution, follow the citation-token rules every client shares: no hardcoded
/// marker, and the `.<section>` suffix preserved so a subsection click lands on
/// that section rather than the declaration heading (§FS-integrations.3.1.4).
pub const VSCODE_EXTENSION_JS: &str = include_str!("../../assets/integrations/vscode/extension.js");

/// Where `--write` installs the `grund-open` resolver for terminal clients; a
/// single source so the descriptor plan and the writer cannot drift.
pub const RESOLVER_TARGET: &str = "~/.local/bin/grund-open";

/// The rendering-layer clients grund ships an integration for. The set is closed
/// and frozen (§FS-integrations.1.4); the ordering here is the frozen output order
/// used by detection and every listing.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum IntegrationClient {
    Codium,
    Iterm2,
    Kitty,
    Tmux,
    Vscode,
    Wezterm,
}

/// How `--write` applies a client's integration (§FS-integrations.4).
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum InstallKind {
    /// A marked block spliced into the client's text config.
    Block,
    /// An unpacked extension directory.
    Vscode,
    /// Nothing writable: the client stores configuration somewhere no managed
    /// block can live, so `--write` installs the resolver and prints the steps
    /// the user has to apply by hand (§FS-integrations.3.4).
    Manual,
}

impl InstallKind {
    /// Reported in the detection plan so a caller can tell why a `manual`
    /// client never reports `installed` (§FS-integrations.5).
    pub fn name(self) -> &'static str {
        match self {
            InstallKind::Block => "block",
            InstallKind::Vscode => "extension",
            InstallKind::Manual => "manual",
        }
    }
}

impl IntegrationClient {
    /// Frozen order: `codium, iterm2, kitty, tmux, vscode, wezterm`.
    pub const ALL: [IntegrationClient; 6] = [
        IntegrationClient::Codium,
        IntegrationClient::Iterm2,
        IntegrationClient::Kitty,
        IntegrationClient::Tmux,
        IntegrationClient::Vscode,
        IntegrationClient::Wezterm,
    ];

    pub fn name(self) -> &'static str {
        match self {
            IntegrationClient::Codium => "codium",
            IntegrationClient::Iterm2 => "iterm2",
            IntegrationClient::Kitty => "kitty",
            IntegrationClient::Tmux => "tmux",
            IntegrationClient::Vscode => "vscode",
            IntegrationClient::Wezterm => "wezterm",
        }
    }

    pub fn from_name(name: &str) -> Option<IntegrationClient> {
        IntegrationClient::ALL
            .into_iter()
            .find(|client| client.name() == name)
    }

    pub fn is_terminal(self) -> bool {
        !matches!(self, IntegrationClient::Vscode | IntegrationClient::Codium)
    }

    pub fn install_kind(self) -> InstallKind {
        match self {
            IntegrationClient::Iterm2 => InstallKind::Manual,
            IntegrationClient::Vscode | IntegrationClient::Codium => InstallKind::Vscode,
            IntegrationClient::Kitty | IntegrationClient::Tmux | IntegrationClient::Wezterm => {
                InstallKind::Block
            }
        }
    }

    /// The terminal config snippet for a terminal client; `None` for vscode.
    pub fn snippet(self) -> Option<&'static str> {
        match self {
            IntegrationClient::Codium => None,
            IntegrationClient::Iterm2 => Some(ITERM2_SNIPPET),
            IntegrationClient::Kitty => Some(KITTY_SNIPPET),
            IntegrationClient::Tmux => Some(TMUX_SNIPPET),
            IntegrationClient::Wezterm => Some(WEZTERM_SNIPPET),
            IntegrationClient::Vscode => None,
        }
    }

    /// A `~`-rooted hint at where `--write` installs the config, printed verbatim
    /// so it stays byte-stable across machines (§FS-integrations.6).
    pub fn config_target(self) -> &'static str {
        match self {
            // Not a path: iTerm2's rules live in a binary plist, so this names
            // the place a human applies them (§FS-integrations.3.4).
            IntegrationClient::Iterm2 => "Settings > Profiles > Advanced > Smart Selection",
            IntegrationClient::Kitty => "~/.config/kitty/kitty.conf",
            IntegrationClient::Tmux => "~/.tmux.conf",
            IntegrationClient::Wezterm => "~/.config/wezterm/wezterm.lua",
            IntegrationClient::Vscode => "~/.vscode/extensions/grund.grund-terminal-citations",
            // VSCodium is a separate application with its own extensions root;
            // installing the same extension into ~/.vscode would land where it
            // is never loaded (§FS-integrations.3.2.4).
            IntegrationClient::Codium => "~/.vscode-oss/extensions/grund.grund-terminal-citations",
        }
    }

    pub fn install_command(self) -> String {
        format!("grund integrations {} --write", self.name())
    }

    /// The line-comment token of the file `--write` installs into. The managed
    /// block markers are comments in the *host* file's language, so this cannot
    /// be one fixed string: `kitty.conf` and `.tmux.conf` comment with `#`,
    /// while `wezterm.lua` is Lua, where `#` is the length operator and a `#`
    /// marker is a syntax error that costs the user their whole config
    /// (§FS-integrations.4.1.1).
    pub fn comment_prefix(self) -> &'static str {
        match self {
            IntegrationClient::Iterm2 | IntegrationClient::Kitty | IntegrationClient::Tmux => "#",
            IntegrationClient::Codium => "//",
            IntegrationClient::Wezterm => "--",
            // vscode installs unpacked files, not a block inside a host config.
            IntegrationClient::Vscode => "//",
        }
    }

    /// Whether a newly-added block goes at the top of the host file rather than
    /// the bottom. A settings file does not care, but a config that is a
    /// *program* does: an appended block lands after the file's `return`, where
    /// its definitions are unreachable and the helper the user is told to call
    /// is nil. Placing it first also matches how one reads Lua — definitions
    /// above use (§FS-integrations.4.1).
    pub fn prepends_block(self) -> bool {
        matches!(self, IntegrationClient::Wezterm)
    }

    /// Emitted below the managed block when `--write` creates the config file
    /// from scratch, so a fresh install is a *working* config rather than one
    /// the user must finish by hand. Unmanaged: later writes rewrite only the
    /// block and leave this alone (§FS-integrations.4.1.2).
    pub fn fresh_config_scaffold(self) -> Option<&'static str> {
        match self {
            // WezTerm applies hyperlink rules only from the config object the
            // file returns, so the block above defines the helper and this calls
            // it. Without this, a fresh file parses but registers nothing.
            IntegrationClient::Wezterm => Some(
                "\n\
                 -- Your WezTerm configuration. grund manages only the block above;\n\
                 -- everything from here down is yours to edit.\n\
                 local config = wezterm.config_builder()\n\
                 \n\
                 grund_apply_hyperlink_rule(config)\n\
                 \n\
                 return config\n",
            ),
            _ => None,
        }
    }
}

pub fn known_clients_line() -> String {
    format!(
        "known clients: {}",
        IntegrationClient::ALL
            .iter()
            .map(|client| client.name())
            .collect::<Vec<_>>()
            .join(", ")
    )
}

/// Expand a leading `~` in an install-target hint against `$HOME`, resolving
/// `~/.config` through `XDG_CONFIG_HOME` when that is set (§FS-integrations.4).
///
/// The hints themselves are printed verbatim and stay `~`-rooted so reports are
/// byte-stable across machines (§FS-integrations.6); only resolution is
/// environment-dependent. Every client whose configuration lives under
/// `~/.config` — kitty, WezTerm, Zed, and grund's own user config — reads it
/// from `$XDG_CONFIG_HOME` when set, so writing to a hardcoded `~/.config`
/// there lands where the tool never looks, and the failure is silent in exactly
/// the way §FS-integrations.3.2.4 refuses to accept for VSCodium.
pub fn expand_target(target: &str) -> Option<PathBuf> {
    if let Some(rest) = target.strip_prefix("~/.config/") {
        return Some(user_config_base()?.join(rest));
    }
    if let Some(rest) = target.strip_prefix("~/") {
        let home = std::env::var_os("HOME")?;
        return Some(PathBuf::from(home).join(rest));
    }
    Some(PathBuf::from(target))
}

/// `$XDG_CONFIG_HOME`, or `$HOME/.config` when it is unset or empty.
fn user_config_base() -> Option<PathBuf> {
    if let Some(base) = std::env::var_os("XDG_CONFIG_HOME").filter(|value| !value.is_empty()) {
        return Some(PathBuf::from(base));
    }
    Some(PathBuf::from(std::env::var_os("HOME")?).join(".config"))
}
