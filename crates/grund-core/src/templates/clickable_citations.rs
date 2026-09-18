//! Which local-conversation rendering one entrypoint teaches, and the
//! `### Clickable citations` section it renders into (§FS-init.2.3.6,
//! §FS-init.2.3.4.17).
//!
//! A function of the config and the entrypoint path and nothing else, which is
//! what lets `init` write the section and `check` re-render it and compare
//! bytes: the render *is* the hash (§FS-check.3.5, §AR-system.2.11).

use std::path::Path;

use crate::config::Config;

/// Render the `### Clickable citations` section (§FS-init.2.3.6): the fixed
/// repository-web convention always, plus the config-derived local-conversation
/// sentence when the repo commits the `link` opinion (§FS-init.2.3.4.17,
/// §DF-repo-conversation-opinion). Without the opinion, local conversation
/// rendering belongs to user-level instructions installed by
/// `grund integrations --write` (§FS-integrations.4.3).
///
/// Why the marker is interpolated rather than left as a `{MARKER}` placeholder:
/// this section is spliced into the template *after* that placeholder is
/// expanded, so a placeholder in this string would survive into the written
/// block.
///
/// Why the committed local-conversation form is always the `file` target: it is
/// composed at write time from the repository root the agent already holds, and
/// embeds nothing about any machine, so two installs render byte-identical
/// files. The per-agent gate is what picks between the two forms — instructing
/// the link form to a renderer that shows the destination in place of the label
/// would erase the citation itself.
///
/// Why the deference clause is there: it is the §DF-repo-conversation-opinion.2.3
/// precedence. This committed opinion is the no-knowledge fallback, and a
/// machine whose user-level block states a rendering knows something about its
/// own surface that the repository cannot — its choice wins.
pub(crate) fn clickable_citations_section(config: &Config, surface: ConversationSurface) -> String {
    // The wording is fixed; the marker is the repository's own
    // (§FS-init.2.3.6), interpolated here rather than left as a `{MARKER}`
    // placeholder.
    let marker = config.marker.as_str();
    let mut section = format!(
        "### Clickable citations\n\nOn repository web surfaces, link `{marker}<ID>` to the PR branch in PR bodies, the reviewed commit in reviews, an exact commit for permalinks, and the default branch otherwise; fall back to plain when unsure."
    );
    if config.conversation.as_deref() == Some("link") {
        // §DF-conversation-link-target: the committed form is always the `file`
        // target (§FS-non-goals.13); which of the two forms is rendered is the
        // per-agent gate (§DF-conversation-link-target.2.4).
        let local = match surface {
            ConversationSurface::Linked => format!(
                " In local conversations, render `{marker}<ID>` as a Markdown link whose visible text is the citation itself and whose target is `file://<absolute path>#L<line>` for its declaration; fall back to the bare citation when unsure."
            ),
            ConversationSurface::Plain => format!(
                " In local conversations, follow `{marker}<ID>` with its declaration location as plain `path:line` text; fall back to the bare citation when unsure."
            ),
        };
        section.push_str(&local);
        section.push_str(
            " If a user-level grund block states a local-conversation rendering, follow that instead: that machine knows what its surface can open.",
        );
    }
    section
}

/// Which local-conversation form one entrypoint file teaches
/// (§FS-init.2.3.4.17). A pure function of the target path, so the generated
/// block stays reproducible (§FS-non-goals.13).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ConversationSurface {
    /// The Claude entrypoints, whose renderer is verified to honor an
    /// absolute-URI Markdown link (§DF-neural-link-generation, rows 12–14).
    Linked,
    /// Every other entrypoint: the location travels as plain `path:line` text
    /// until a click-test says more.
    Plain,
}

impl ConversationSurface {
    /// `CLAUDE.md` at the repository root or under `.claude/` — the two paths
    /// the Claude entrypoint family occupies (§FS-init.2.3).
    pub(crate) fn for_entrypoint(path: &Path) -> Self {
        match path.file_name().and_then(|name| name.to_str()) {
            Some("CLAUDE.md") => Self::Linked,
            _ => Self::Plain,
        }
    }
}
