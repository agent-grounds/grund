//! The user-level agent-instruction surfaces `grund integrations --write`
//! synchronizes (§FS-integrations.4.3): which agents have a file-backed global
//! instruction file, what each renderer is verified to do with a linked
//! citation (§DF-conversation-link-target.2.4), and the two closed enums the
//! preference is spelled in — the `plain`/`link` rendering and the target a
//! link addresses its declaration through (§FS-config.3.1).
//!
//! The instruction texts are here rather than in a template: they are
//! user-global and written once, before grund knows which repositories will be
//! opened, so they name no marker (§FS-integrations.3.1). The repository
//! entrypoint carries the syntax; this block carries the policy.

/// How much of the linked conversation form one agent's renderer is *verified*
/// to honor (§DF-conversation-link-target.2.4). Anything unverified resolves to
/// `ConversationTarget::Path`, the form that surface already had — the gate can
/// hold a target where it is, never make one worse.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum LinkSupport {
    /// Every target, local schemes included (matrix rows 12–14).
    Every,
    /// `file:` and web URLs dispatch, editor schemes do not; labels survive
    /// either way (matrix row 16, Pi).
    FileAndWeb,
    /// Web URLs only; a local destination is rendered in place of the link
    /// label, erasing the citation (matrix rows 3 and 15, Codex).
    WebOnly,
    /// No click-test either way, so the citation keeps its plain location.
    Unverified,
}

impl LinkSupport {
    pub fn resolve(self, target: ConversationTarget) -> ConversationTarget {
        match (self, target) {
            (Self::Every, _) => target,
            (Self::FileAndWeb, ConversationTarget::File | ConversationTarget::Web) => target,
            (Self::WebOnly, ConversationTarget::Web) => target,
            _ => ConversationTarget::Path,
        }
    }
}

/// One agent's file-backed global instruction surface (§FS-integrations.4.3).
/// `home` is the directory whose presence says the user actually runs that
/// agent: `--write` installs a *rendering layer*, and provisioning the config
/// tree of five agents the machine does not have is not part of that.
pub struct GlobalAgentTarget {
    /// The agent's name — the key `[reference.agents.<agent>]` is written under
    /// and the value `--agent` accepts (§FS-integrations.4.4).
    pub agent: &'static str,
    /// `~`-rooted instruction file, printed verbatim so reports stay stable.
    pub file: &'static str,
    /// `~`-rooted directory that shows the agent is in use.
    pub home: &'static str,
    /// What this agent's renderer is verified to do with a linked citation.
    pub link_support: LinkSupport,
}

/// The file-backed global instruction surfaces for every agent grund supports
/// end-to-end (§FS-integrations.4.3). Keep this superset aligned with the
/// repository entrypoints in §FS-init.2.1.
pub const GLOBAL_AGENT_INSTRUCTION_TARGETS: [GlobalAgentTarget; 6] = [
    GlobalAgentTarget {
        agent: "codex",
        file: "~/.codex/AGENTS.md",
        home: "~/.codex",
        link_support: LinkSupport::WebOnly,
    },
    GlobalAgentTarget {
        agent: "claude",
        file: "~/.claude/CLAUDE.md",
        home: "~/.claude",
        link_support: LinkSupport::Every,
    },
    GlobalAgentTarget {
        agent: "gemini",
        file: "~/.gemini/GEMINI.md",
        home: "~/.gemini",
        link_support: LinkSupport::Unverified,
    },
    GlobalAgentTarget {
        agent: "copilot",
        file: "~/.copilot/copilot-instructions.md",
        home: "~/.copilot",
        link_support: LinkSupport::Unverified,
    },
    GlobalAgentTarget {
        agent: "zed",
        file: "~/.config/zed/AGENTS.md",
        home: "~/.config/zed",
        link_support: LinkSupport::Unverified,
    },
    GlobalAgentTarget {
        agent: "pi",
        file: "~/.pi/agent/AGENTS.md",
        home: "~/.pi",
        link_support: LinkSupport::FileAndWeb,
    },
];

/// The table one agent's overrides live under (§FS-integrations.4.4). The
/// partial is merged over the machine-wide keys, so only the names above are
/// accepted inside it.
pub fn agent_override_table(agent: &str) -> String {
    format!("reference.agents.{agent}")
}

/// `codex | claude | …` for the errors that list the accepted set.
pub fn known_agents_list() -> String {
    GLOBAL_AGENT_INSTRUCTION_TARGETS
        .iter()
        .map(|target| target.agent)
        .collect::<Vec<_>>()
        .join(", ")
}

/// The canonical spelling of a known agent name, or `None` (§FS-integrations.4.4).
pub fn known_agent(name: &str) -> Option<&'static str> {
    GLOBAL_AGENT_INSTRUCTION_TARGETS
        .iter()
        .map(|target| target.agent)
        .find(|known| *known == name)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConversationRendering {
    Plain,
    Link,
}

impl ConversationRendering {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "plain" => Some(Self::Plain),
            "link" => Some(Self::Link),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Plain => "plain",
            Self::Link => "link",
        }
    }

    /// §FS-integrations.4.3: self-scoping — the texts apply only inside grund
    /// repositories, so in any other repo their footprint is one inert sentence.
    /// The precedence sentence appears only in `plain`: repository `link` against
    /// user `plain` is the only possible conflict, and the machine wins it
    /// (§DF-repo-conversation-opinion.2.3) — `plain` is only ever recorded by a
    /// `--write` that installed a rendering layer, knowledge no repository has.
    ///
    /// These texts name no marker, for the same reason the matchers and the
    /// resolver do not hardcode one (§FS-integrations.3.1): they are user-global
    /// and written once, before grund knows which repositories will be opened,
    /// and `[reference] marker` is per-repo. There is nothing to interpolate at
    /// install time, so these carry the policy and the repository entrypoint —
    /// which does render its own marker — carries the syntax.
    pub(crate) fn instruction(self, target: ConversationTarget) -> String {
        match self {
            Self::Plain => "In repositories with a `grund.toml` (at the root or under `.agents/`): write citations bare in local conversations — the marker and ID alone, nothing appended; `grund integrations` makes them clickable. Follow this even when repository instructions ask for linked citations — that repository sentence defers to this block, and the installed rendering layer already resolves bare citations. Elsewhere, ignore this.".to_string(),
            // The target is the one value interpolated here, and legitimately
            // so: unlike the marker it *is* machine state, which is what a
            // user-global file is for (§FS-integrations.4.3).
            Self::Link => match target.uri_phrase() {
                None => "In repositories with a `grund.toml` (at the root or under `.agents/`): follow each citation with its declaration location as plain `path:line` text in local conversations; fall back to the bare citation when unsure. Elsewhere, ignore this.".to_string(),
                Some(phrase) => format!(
                    "In repositories with a `grund.toml` (at the root or under `.agents/`): in local conversations render each citation as a Markdown link whose visible text is the citation itself and whose target is {phrase}; fall back to the bare citation when unsure. Elsewhere, ignore this."
                ),
            },
        }
    }
}

/// How a linked citation addresses its declaration (§FS-config.3.1,
/// §DF-conversation-link-target.2.2). A closed enum: each value names one fixed
/// template an agent fills from the declaration's absolute path and line.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ConversationTarget {
    /// `file://<abs>#L<line>` — the default, and the only local form that
    /// presumes nothing about the machine beyond a handler for `file:`.
    #[default]
    File,
    /// No URI: the location travels as plain `path:line` text. The pre-2026-08-11
    /// form, kept as the opt-out and used as the gate's fallback.
    Path,
    /// The forge blob URL for the current ref, per the repository-web rule.
    Web,
    Vscode,
    Vscodium,
    Cursor,
}

impl ConversationTarget {
    /// Accepted values, in the order the error message lists them.
    pub(crate) const ALL: [ConversationTarget; 6] = [
        ConversationTarget::File,
        ConversationTarget::Path,
        ConversationTarget::Web,
        ConversationTarget::Vscode,
        ConversationTarget::Vscodium,
        ConversationTarget::Cursor,
    ];

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|target| target.name() == name)
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Path => "path",
            Self::Web => "web",
            Self::Vscode => "vscode",
            Self::Vscodium => "vscodium",
            Self::Cursor => "cursor",
        }
    }

    /// `file | path | web | …` for the error that lists the accepted set.
    pub fn accepted_list() -> String {
        Self::ALL
            .iter()
            .map(|target| target.name())
            .collect::<Vec<_>>()
            .join(" | ")
    }

    /// The template clause the instruction block names, or `None` for `Path`,
    /// which carries no URI and takes the plain-location sentence instead.
    fn uri_phrase(self) -> Option<&'static str> {
        match self {
            Self::Path => None,
            Self::File => Some("`file://<absolute path>#L<line>` for the declaration"),
            Self::Web => Some("the declaration's forge URL at the current commit"),
            Self::Vscode => Some("`vscode://file<absolute path>:<line>` for the declaration"),
            Self::Vscodium => Some("`vscodium://file<absolute path>:<line>` for the declaration"),
            Self::Cursor => Some("`cursor://file<absolute path>:<line>` for the declaration"),
        }
    }
}
