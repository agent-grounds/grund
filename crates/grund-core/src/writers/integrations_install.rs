//! What `--write` puts on disk for one client, and what it finds already there
//! (§FS-integrations.4): the managed-block splice for a terminal config and for
//! the user-level agent instructions, the `grund-open` resolver with its
//! executable bit, and the byte-current probes a detection plan reports
//! `installed` from (§FS-integrations.5).
//!
//! Finding a block and reading its version is `grammar/managed_block.rs`'s
//! (§AR-system.2.1); what is left here is the half that decides — where a block
//! goes when the host file has none, whether the file was created from scratch,
//! and which verb the run has earned.

use std::fs;
use std::path::{Path, PathBuf};

use super::integrations_agents::{ConversationRendering, ConversationTarget};
use super::integrations_clients::{
    GRUND_OPEN_RESOLVER, InstallKind, IntegrationClient, RESOLVER_TARGET, VSCODE_EXTENSION_JS,
    VSCODE_PACKAGE_JSON, expand_target,
};
use crate::grammar::{
    AGENT_GUIDANCE_BLOCK_VERSION, INTEGRATIONS_BLOCK_VERSION, agent_guidance_markers,
    find_agent_guidance_block, find_managed_block, integrations_block_markers,
};

/// Result of splicing a managed block into a dotfile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BlockOutcome {
    Appended,
    Updated,
    Unchanged,
}

pub(crate) fn block_outcome_verb(outcome: BlockOutcome) -> &'static str {
    match outcome {
        BlockOutcome::Appended => "appended",
        BlockOutcome::Updated => "updated",
        BlockOutcome::Unchanged => "exists",
    }
}

/// One file, two managed keys: `exists` only when neither line moved, and an
/// append anywhere makes the whole write an append (§FS-integrations.6).
pub(crate) fn merge_outcomes(first: BlockOutcome, second: BlockOutcome) -> BlockOutcome {
    match (first, second) {
        (BlockOutcome::Unchanged, other) | (other, BlockOutcome::Unchanged) => other,
        (BlockOutcome::Appended, _) | (_, BlockOutcome::Appended) => BlockOutcome::Appended,
        _ => BlockOutcome::Updated,
    }
}

/// Splice the managed integrations block carrying `snippet` into `existing`
/// (§FS-integrations.4.1): replace the bytes between the current-version markers
/// when a block is present, else append after a blank-line separator. Everything
/// outside the block is preserved. Returns the new text and what changed. A block
/// whose version is newer than this binary understands is an error.
pub(crate) fn install_managed_block(
    comment: &str,
    prepend: bool,
    existing: &str,
    snippet: &str,
) -> Result<(String, BlockOutcome), String> {
    let (begin, end) = integrations_block_markers(comment, INTEGRATIONS_BLOCK_VERSION);
    let block = format!("{begin}\n{}\n{end}\n", snippet.trim_end_matches('\n'));
    if let Some(span) = find_managed_block(comment, existing)? {
        let mut updated = String::with_capacity(existing.len());
        updated.push_str(&existing[..span.start]);
        updated.push_str(&block);
        updated.push_str(&existing[span.stop..]);
        let outcome = if updated == existing {
            BlockOutcome::Unchanged
        } else {
            BlockOutcome::Updated
        };
        Ok((updated, outcome))
    } else if prepend {
        let mut prepended = String::with_capacity(existing.len() + block.len() + 2);
        prepended.push_str(&block);
        if !existing.is_empty() {
            prepended.push('\n');
            prepended.push_str(existing);
        }
        Ok((prepended, BlockOutcome::Appended))
    } else {
        let mut appended = String::with_capacity(existing.len() + block.len() + 2);
        appended.push_str(existing);
        if !existing.is_empty() && !existing.ends_with('\n') {
            appended.push('\n');
        }
        if !existing.is_empty() {
            appended.push('\n');
        }
        appended.push_str(&block);
        Ok((appended, BlockOutcome::Appended))
    }
}

pub(crate) fn install_agent_guidance_block(
    existing: &str,
    preference: ConversationRendering,
    target: ConversationTarget,
) -> Result<(String, BlockOutcome), String> {
    let (begin, end) = agent_guidance_markers(AGENT_GUIDANCE_BLOCK_VERSION);
    let block = format!(
        "{begin}\n## Grund citation rendering\n\n{}\n{end}\n",
        preference.instruction(target)
    );
    if let Some(span) = find_agent_guidance_block(existing)? {
        let mut updated = String::with_capacity(existing.len() + block.len());
        updated.push_str(&existing[..span.start]);
        updated.push_str(&block);
        updated.push_str(&existing[span.stop..]);
        let outcome = if updated == existing {
            BlockOutcome::Unchanged
        } else {
            BlockOutcome::Updated
        };
        Ok((updated, outcome))
    } else {
        let mut appended = String::with_capacity(existing.len() + block.len() + 2);
        appended.push_str(existing);
        if !existing.is_empty() && !existing.ends_with('\n') {
            appended.push('\n');
        }
        if !existing.is_empty() {
            appended.push('\n');
        }
        appended.push_str(&block);
        Ok((appended, BlockOutcome::Appended))
    }
}

/// The call a WezTerm config must make on the object it returns for the managed
/// block to do anything (§FS-integrations.4.1).
pub(crate) const WEZTERM_APPLY_CALL: &str = "grund_apply_hyperlink_rule(";

/// Whether this write leaves the user one manual step short of a working
/// integration (§FS-integrations.4.1). WezTerm applies hyperlink rules only from
/// the config object the user's own Lua returns, and no installer can safely
/// rewrite the function that builds it — so the block can be perfectly installed
/// and every click still inert, which reports exactly like a broken install.
///
/// The search is the unmanaged remainder alone, never the whole file: the block
/// defines the helper and names it in its own comments, so a whole-file test
/// would match on every config and report nothing. A file grund scaffolded from
/// scratch calls the helper below the block and is therefore already wired.
pub(crate) fn needs_wezterm_wiring(client: IntegrationClient, text: &str) -> bool {
    if client != IntegrationClient::Wezterm {
        return false;
    }
    let outside = match find_managed_block(client.comment_prefix(), text) {
        Ok(Some(span)) => format!("{}{}", &text[..span.start], &text[span.stop..]),
        // No block, or a block this binary cannot read: the caller has already
        // failed on the latter, and the former cannot be wired either way.
        _ => text.to_string(),
    };
    !outside.contains(WEZTERM_APPLY_CALL)
}

/// Install `grund-open` to `~/.local/bin` when absent or out of date. Returns the
/// path when written, `None` when already current.
pub(crate) fn write_resolver_script() -> Result<Option<PathBuf>, (PathBuf, String)> {
    let Some(path) = expand_target(RESOLVER_TARGET) else {
        return Err((
            PathBuf::from(RESOLVER_TARGET),
            "cannot resolve home directory".to_string(),
        ));
    };
    if fs::read_to_string(&path).is_ok_and(|current| current == GRUND_OPEN_RESOLVER) {
        if is_executable(&path) {
            return Ok(None);
        }
        // Content already matches, but a copy restored by a dotfile manager (or
        // written under a +x-stripping umask) may not be executable — clicking a
        // citation would then fail with "permission denied". Ensure the bit is
        // set before reporting the resolver as current.
        set_executable(&path).map_err(|err| (path.clone(), err.to_string()))?;
        return Ok(None);
    }
    if let Some(parent) = path.parent()
        && let Err(err) = fs::create_dir_all(parent)
    {
        return Err((parent.to_path_buf(), err.to_string()));
    }
    fs::write(&path, GRUND_OPEN_RESOLVER).map_err(|err| (path.clone(), err.to_string()))?;
    set_executable(&path).map_err(|err| (path.clone(), err.to_string()))?;
    Ok(Some(path))
}

#[cfg(unix)]
pub(crate) fn set_executable(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let metadata = fs::metadata(path)?;
    let mut perms = metadata.permissions();
    perms.set_mode(perms.mode() | 0o111);
    fs::set_permissions(path, perms)
}

#[cfg(not(unix))]
pub(crate) fn set_executable(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path).is_ok_and(|metadata| metadata.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

/// Whether every grund-owned artifact for a client is present and byte-current
/// (§FS-integrations.5). Only the client's fixed target paths are read.
pub(crate) fn integration_is_current(client: IntegrationClient) -> bool {
    match client.install_kind() {
        InstallKind::Block => {
            terminal_integration_is_current(client, client.snippet().unwrap_or(""))
        }
        InstallKind::Vscode => expand_target(client.config_target())
            .is_some_and(|dir| vscode_integration_is_current(&dir)),
        // Applied by hand in a binary plist we never read: grund cannot know,
        // and guessing "installed" would be worse than reporting nothing.
        InstallKind::Manual => false,
    }
}

fn terminal_integration_is_current(client: IntegrationClient, snippet: &str) -> bool {
    let Some(config_path) = expand_target(client.config_target()) else {
        return false;
    };
    let Ok(existing) = fs::read_to_string(config_path) else {
        return false;
    };
    let Ok(Some(span)) = find_managed_block(client.comment_prefix(), &existing) else {
        return false;
    };
    let (begin, end) =
        integrations_block_markers(client.comment_prefix(), INTEGRATIONS_BLOCK_VERSION);
    let expected = format!("{begin}\n{}\n{end}\n", snippet.trim_end_matches('\n'));
    if span.version != INTEGRATIONS_BLOCK_VERSION || existing[span.start..span.stop] != expected {
        return false;
    }
    let Some(resolver_path) = expand_target(RESOLVER_TARGET) else {
        return false;
    };
    fs::read_to_string(&resolver_path).is_ok_and(|text| text == GRUND_OPEN_RESOLVER)
        && is_executable(&resolver_path)
}

pub(crate) fn vscode_integration_is_current(dir: &Path) -> bool {
    fs::read_to_string(dir.join(".grund-version"))
        .is_ok_and(|text| text == INTEGRATIONS_BLOCK_VERSION.to_string())
        && fs::read_to_string(dir.join("package.json"))
            .is_ok_and(|text| text == VSCODE_PACKAGE_JSON)
        && fs::read_to_string(dir.join("extension.js"))
            .is_ok_and(|text| text == VSCODE_EXTENSION_JS)
}
