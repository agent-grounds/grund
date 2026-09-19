//! The user-level grund configuration `--write` records the conversation
//! preference in, and reads back (§FS-integrations.4.3): the keys grund
//! consumes, the targeted scan that finds them under every spelling TOML calls
//! equivalent, and the one-key install that rewrites a value while preserving
//! every unrelated byte.
//!
//! This is a scan, not a TOML parser, and nothing in it fails: the file is
//! machine-local, read by one command, and decides nothing about whether a tree
//! checks clean — the opposite of the closed allow-list the repository config
//! enforces (§FS-config.4.3). Every line grund did not act on is collected as a
//! problem for the caller to report once.

use std::fs;
use std::path::{Path, PathBuf};

use super::integrations_agents::{
    ConversationRendering, ConversationTarget, known_agent, known_agents_list,
};
use super::integrations_clients::expand_target;
use super::integrations_install::BlockOutcome;
use crate::config::strip_comment;

/// The user-level Grund configuration `--write` records the preference in
/// (§FS-integrations.4.3). `~/.config` resolves through `XDG_CONFIG_HOME`.
pub const USER_CONFIG_TARGET: &str = "~/.config/grund/config.toml";

/// The user-level setting spelled exactly as the repository key for the same
/// concept (§FS-config.3.1.3): one name, two scopes.
const CONVERSATION_KEY_PATH: &str = "reference.conversation";

/// How a linked citation addresses its declaration (§FS-integrations.4.3.2). No
/// repository spelling — the scheme is machine state
/// (§DF-conversation-link-target.2.3).
const CONVERSATION_TARGET_KEY_PATH: &str = "reference.conversation_target";

/// The keys grund consumes from the user configuration, for the unused-key
/// warning that names them.
const USER_CONFIG_KEY_PATHS: &str = "`reference.conversation`, `reference.conversation_target`, and `reference.agents.<agent>.conversation_target`";

pub fn read_optional_text(path: &Path) -> Result<String, (PathBuf, String)> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(text),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(err) => Err((path.to_path_buf(), err.to_string())),
    }
}

pub fn user_grund_config_path() -> Option<PathBuf> {
    expand_target(USER_CONFIG_TARGET)
}

/// The dotted name of a `[section]` header line, whitespace-normalized so
/// `[ reference ]` and `[reference]` — the same table in TOML — are the same
/// section here. `[[array]]` headers keep their inner bracket and therefore
/// never compare equal to a real section name.
fn section_header_name(line: &str) -> Option<String> {
    let inner = line.strip_prefix('[')?.strip_suffix(']')?;
    Some(
        inner
            .split('.')
            .map(str::trim)
            .collect::<Vec<_>>()
            .join("."),
    )
}

/// The contents of a TOML basic or literal string, or `None` when the value is
/// not quoted. Both quote styles mean the same thing for a value from a closed
/// enum, and rejecting `'link'` — which TOML accepts — would reject a config the
/// user wrote correctly.
fn unquote_toml_string(value: &str) -> Option<&str> {
    let value = value.trim();
    ['"', '\''].into_iter().find_map(|quote| {
        value
            .strip_prefix(quote)
            .and_then(|inner| inner.strip_suffix(quote))
    })
}

/// The fully-qualified key path of a `key = value` line inside `section`, so the
/// equivalent TOML spellings — `[reference]` + `conversation`, `[reference]` +
/// nothing with a dotted root `reference.conversation`, `[reference.sub]` — all
/// reduce to one name.
fn qualified_key(section: &str, key: &str) -> String {
    if section.is_empty() {
        key.to_string()
    } else {
        format!("{section}.{key}")
    }
}

/// What one scan of the user configuration found (§FS-integrations.4.3).
pub struct UserConfigScan {
    pub preference: Option<ConversationRendering>,
    /// The recorded addressing target, independent of `preference`: an
    /// unreadable target never costs the `plain`/`link` value recorded beside
    /// it (§FS-integrations.4.3.5).
    pub target: Option<ConversationTarget>,
    /// `[reference.agents.<agent>]` partials, in file order — the override
    /// layer merged over `target` per agent (§FS-integrations.4.4). Only known
    /// agents land here; an unknown one is a warning naming the closed set.
    pub agent_targets: Vec<(String, ConversationTarget)>,
    /// `(line, message)` for everything in the file grund did not act on, in
    /// file order. Every message names what is being ignored, so the report says
    /// what the run will do rather than only what is wrong.
    pub problems: Vec<(usize, String)>,
}

/// Scan the user configuration for the single preference grund reads there, and
/// collect everything else it did not act on (§FS-integrations.4.3).
///
/// This is a targeted scan, not a TOML parser, so it must accept every spelling
/// TOML calls equivalent — whitespace inside the section header, a dotted key
/// path, either quote style. A spelling grund merely failed to *see* would be
/// silently reversed to the default and then written back alongside the
/// original, leaving the opposite of what the user asked for.
///
/// Recognizing the right spelling is only half of that, though: nothing tells a
/// reader of this file which keys grund actually consumes, so a typo, a retired
/// spelling, or a repository key set here in the belief that it applies globally
/// all read as "configured" and do nothing.
///
/// **Nothing here fails.** One rule covers the whole file: report it, ignore it,
/// continue on the documented default. This file is machine-local, read by one
/// command, and decides nothing about whether a tree checks clean — the opposite
/// of the closed allow-list the repository config enforces (§FS-config.4.3),
/// where an unknown key means two installs could disagree about a checked tree.
/// A stale line in a personal config is not a reason to refuse to install a
/// terminal integration, and an unparseable value is not more of a reason than
/// an unread key: both mean grund has no preference from this file, which is
/// exactly the state of a machine that never wrote one.
pub fn scan_user_config(text: &str) -> UserConfigScan {
    let mut section = String::new();
    let mut preference = None;
    let mut target = None;
    let mut agent_targets: Vec<(String, ConversationTarget)> = Vec::new();
    let mut problems = Vec::new();
    for (idx, raw_line) in text.lines().enumerate() {
        let line = strip_comment(raw_line).trim();
        if let Some(name) = section_header_name(line) {
            section = name;
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let qualified = qualified_key(&section, key.trim());
        let value = value.trim();
        // First occurrence wins, per key, and it is the one a write rewrites, so
        // a read and a write can never disagree about which line is the setting.
        match qualified.as_str() {
            CONVERSATION_KEY_PATH => {
                if preference.is_some() {
                    problems.push((
                        idx + 1,
                        format!(
                            "ignoring duplicate `{CONVERSATION_KEY_PATH}`; the first one is used"
                        ),
                    ));
                    continue;
                }
                let Some(unquoted) = unquote_toml_string(value) else {
                    problems.push((
                        idx + 1,
                        format!(
                            "ignoring `{CONVERSATION_KEY_PATH} = {value}`: must be a quoted plain | link value"
                        ),
                    ));
                    continue;
                };
                match ConversationRendering::from_name(unquoted) {
                    Some(parsed) => preference = Some(parsed),
                    None => problems.push((
                        idx + 1,
                        format!(
                            "ignoring `{CONVERSATION_KEY_PATH} = {value}`: must be one of plain | link"
                        ),
                    )),
                }
            }
            CONVERSATION_TARGET_KEY_PATH => {
                if target.is_some() {
                    problems.push((
                        idx + 1,
                        format!(
                            "ignoring duplicate `{CONVERSATION_TARGET_KEY_PATH}`; the first one is used"
                        ),
                    ));
                    continue;
                }
                let accepted = ConversationTarget::accepted_list();
                let Some(unquoted) = unquote_toml_string(value) else {
                    problems.push((
                        idx + 1,
                        format!(
                            "ignoring `{CONVERSATION_TARGET_KEY_PATH} = {value}`: must be a quoted {accepted} value"
                        ),
                    ));
                    continue;
                };
                match ConversationTarget::from_name(unquoted) {
                    Some(parsed) => target = Some(parsed),
                    None => problems.push((
                        idx + 1,
                        format!(
                            "ignoring `{CONVERSATION_TARGET_KEY_PATH} = {value}`: must be one of {accepted}"
                        ),
                    )),
                }
            }
            // §FS-integrations.4.4: `[reference.agents.<agent>]` is a partial of
            // the keys above, so it accepts the same names and nothing else.
            _ if section.starts_with("reference.agents.") => {
                let agent = section.trim_start_matches("reference.agents.");
                let Some(known) = known_agent(agent) else {
                    problems.push((
                        idx + 1,
                        format!(
                            "unknown agent `{agent}` in reference.agents; known agents: {}",
                            known_agents_list()
                        ),
                    ));
                    continue;
                };
                if key.trim() != "conversation_target" {
                    problems.push((
                        idx + 1,
                        format!(
                            "unused key `{qualified}`; grund reads only {USER_CONFIG_KEY_PATHS} from this file"
                        ),
                    ));
                    continue;
                }
                if agent_targets.iter().any(|(name, _)| name == known) {
                    problems.push((
                        idx + 1,
                        format!("ignoring duplicate `{qualified}`; the first one is used"),
                    ));
                    continue;
                }
                let accepted = ConversationTarget::accepted_list();
                match unquote_toml_string(value).and_then(ConversationTarget::from_name) {
                    Some(parsed) => agent_targets.push((known.to_string(), parsed)),
                    None => problems.push((
                        idx + 1,
                        format!("ignoring `{qualified} = {value}`: must be one of {accepted}"),
                    )),
                }
            }
            _ => problems.push((
                idx + 1,
                format!(
                    "unused key `{qualified}`; grund reads only {USER_CONFIG_KEY_PATHS} from this file"
                ),
            )),
        }
    }
    UserConfigScan {
        preference,
        target,
        agent_targets,
        problems,
    }
}

/// The `plain`/`link` preference alone, for tests that assert on one key.
#[cfg(test)]
pub(crate) fn conversation_preference(text: &str) -> Option<ConversationRendering> {
    scan_user_config(text).preference
}

/// The recorded addressing target alone, for tests that assert on one key.
#[cfg(test)]
pub(crate) fn conversation_target_preference(text: &str) -> Option<ConversationTarget> {
    scan_user_config(text).target
}

/// `install_reference_key` bound to the `conversation` key, which is what most
/// of the preference-file tests exercise.
#[cfg(test)]
pub(crate) fn install_conversation_preference(
    existing: &str,
    preference: ConversationRendering,
) -> (String, BlockOutcome) {
    install_reference_key(
        existing,
        "reference",
        "conversation",
        preference.name(),
        conversation_preference(existing) == Some(preference),
    )
}

/// Install or replace one `[reference]` line while preserving unrelated bytes.
/// Infallible: every defect in this file is a warning reported at load
/// (§FS-integrations.4.3.5), so there is nothing left here to refuse.
///
/// `table` is the table the key belongs to — `reference`, or one agent's
/// `reference.agents.<agent>` partial (§FS-integrations.4.4) — and `bare_key`
/// the name to write when the key is absent; the two are joined to match an
/// existing line, since the file may spell the key dotted at root.
/// `already_recorded` says the scan already read this exact value.
///
/// Why an already-recorded value is left alone: rewriting an identical value
/// would report `updated` for a no-op and drop whatever comment the user wrote
/// beside it.
pub fn install_reference_key(
    existing: &str,
    table: &str,
    bare_key: &str,
    value: &str,
    already_recorded: bool,
) -> (String, BlockOutcome) {
    let key_path = format!("{table}.{bare_key}");
    // Already recorded: leave the bytes alone — a second `--write` is a no-op
    // reporting `exists` (§FS-integrations.6.1).
    if already_recorded {
        return (existing.to_string(), BlockOutcome::Unchanged);
    }
    let mut offset = 0;
    let mut section = String::new();
    let mut section_stop = None;
    for raw_line in existing.split_inclusive('\n') {
        let stop = offset + raw_line.len();
        let raw = raw_line.trim_end_matches(['\n', '\r']);
        let line = strip_comment(raw).trim();
        if let Some(name) = section_header_name(line) {
            section = name;
            if section == table {
                section_stop = Some(stop);
            }
            offset = stop;
            continue;
        }
        if let Some((key, _)) = line.split_once('=')
            && qualified_key(&section, key.trim()) == key_path
        {
            // Rewrite only the value, keeping the key exactly as written: a
            // dotted `reference.conversation` at root rewritten as a bare
            // `conversation` would land in whatever table precedes it and stop
            // being this setting at all.
            let head = &raw[..raw.find('=').unwrap_or(raw.len()) + 1];
            let replacement = format!("{head} \"{value}\"\n");
            let mut updated = String::with_capacity(existing.len() + replacement.len());
            updated.push_str(&existing[..offset]);
            updated.push_str(&replacement);
            updated.push_str(&existing[stop..]);
            let outcome = if updated == existing {
                BlockOutcome::Unchanged
            } else {
                BlockOutcome::Updated
            };
            return (updated, outcome);
        }
        offset = stop;
    }

    let replacement = format!("{bare_key} = \"{value}\"\n");
    if let Some(insert_at) = section_stop {
        let mut updated = String::with_capacity(existing.len() + replacement.len());
        updated.push_str(&existing[..insert_at]);
        if !existing[..insert_at].ends_with('\n') {
            updated.push('\n');
        }
        updated.push_str(&replacement);
        updated.push_str(&existing[insert_at..]);
        return (updated, BlockOutcome::Updated);
    }

    let mut updated = String::with_capacity(existing.len() + replacement.len() + 18);
    updated.push_str(existing);
    if !existing.is_empty() && !existing.ends_with('\n') {
        updated.push('\n');
    }
    if !existing.is_empty() {
        updated.push('\n');
    }
    updated.push_str(&format!("[{table}]\n"));
    updated.push_str(&replacement);
    (updated, BlockOutcome::Appended)
}
