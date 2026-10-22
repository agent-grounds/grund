//! Companion entrypoint writes and their partial failures (§FS-init.2.3.4.17.2,
//! §FS-distribution.3.2.2.1).
use super::init::verb_wrote;
use super::init_block::{AgentsUpdateResult, MembersSection, update_agents_block};
use super::init_output::{InitError, InitEvent};
use crate::model::format_path;
use crate::scanner::InitCompanionAgentEntrypoint;
use crate::templates::ConversationSurface;
use std::fs;
use std::path::Path;

pub(super) struct CompanionWrite<'a, F> {
    pub target: &'a Path,
    pub companions: Vec<InitCompanionAgentEntrypoint>,
    pub agents_block: &'a str,
    pub claude_block: Option<&'a str>,
    pub add_chapter_rules: F,
    pub dry_run: bool,
    /// §FS-init.2.3.4.15.8: an optional member is absent, so each existing block
    /// keeps its own member list.
    pub keep_members: bool,
    pub diagnostic: &'a mut Option<anyhow::Error>,
    pub events: &'a mut Vec<InitEvent>,
    /// What each companion did with its member list, in report order.
    pub members: &'a mut Vec<(String, MembersSection)>,
    pub workflow_entrypoint: &'a mut Option<String>,
    pub any_change: &'a mut bool,
}
pub(super) fn write_companions<F>(args: CompanionWrite<'_, F>) -> Result<(), InitError>
where
    F: Fn(String, &Path) -> String,
{
    let CompanionWrite {
        target,
        companions,
        agents_block,
        claude_block,
        add_chapter_rules,
        dry_run,
        keep_members,
        diagnostic,
        events,
        members,
        workflow_entrypoint,
        any_change,
    } = args;
    for entrypoint in companions {
        let path_ref = entrypoint.path();
        // The Claude entrypoints teach the linked form; every other companion
        // gets the plain-location block (§FS-init.2.3.4.17.2).
        let entrypoint_block = match ConversationSurface::for_entrypoint(path_ref) {
            ConversationSurface::Linked => claude_block.as_deref().unwrap_or(&agents_block),
            ConversationSurface::Plain => &agents_block,
        };
        let entrypoint_block = add_chapter_rules(entrypoint_block.to_string(), path_ref);
        let rel = path_ref
            .strip_prefix(&target)
            .unwrap_or(path_ref)
            .to_path_buf();
        let rel = format_path(&rel);
        if workflow_entrypoint.is_none() {
            *workflow_entrypoint = Some(rel.clone());
        }
        match entrypoint {
            InitCompanionAgentEntrypoint::Existing(path) => {
                match update_agents_block(&path, &entrypoint_block, &rel, dry_run, keep_members) {
                    Ok((result, section)) => {
                        members.push((rel.clone(), section));
                        *any_change |= result != AgentsUpdateResult::Unchanged;
                        let verb = result.verb(dry_run);
                        events.push(InitEvent { verb, path: rel });
                    }
                    Err(err) => {
                        let (source, message) = update_failure(&path, &err);
                        *diagnostic = Some(source);
                        return Err(InitError::with_events(std::mem::take(events), message));
                    }
                }
            }
            InitCompanionAgentEntrypoint::MissingAlias(path) => {
                if !dry_run
                    && let Some(parent) = path.parent()
                    && let Err(err) = fs::create_dir_all(parent)
                {
                    // §FS-distribution.3.3.2: retain source I/O data before string projection.
                    *diagnostic = Some(
                        crate::model::OperationDiagnostic::filesystem(
                            &parent,
                            &err,
                            format!("create {}: {err}", parent.display()),
                        )
                        .into(),
                    );
                    return Err(InitError::with_events(
                        std::mem::take(events),
                        format!("create {}: {err}", parent.display()),
                    ));
                }
                if !dry_run && let Err(err) = fs::write(&path, entrypoint_block) {
                    // §FS-distribution.3.3.2: retain source I/O data before string projection.
                    *diagnostic = Some(
                        crate::model::OperationDiagnostic::filesystem(
                            &path,
                            &err,
                            format!("write {}: {err}", path.display()),
                        )
                        .into(),
                    );
                    return Err(InitError::with_events(
                        std::mem::take(events),
                        format!("write {}: {err}", path.display()),
                    ));
                }
                members.push((rel.clone(), MembersSection::New));
                events.push(InitEvent {
                    verb: verb_wrote(dry_run),
                    path: rel,
                });
                *any_change = true;
            }
        }
        #[cfg(feature = "test-binding-writes")]
        if !dry_run && events.last().is_some_and(InitEvent::is_change) {
            if let Err(error) = super::binding_write_fault::after_write() {
                let path = target.join(&events.last().unwrap().path);
                *diagnostic = Some(
                    crate::model::OperationDiagnostic::filesystem(&path, &error, error.to_string())
                        .into(),
                );
                return Err(InitError::with_events(
                    std::mem::take(events),
                    error.to_string(),
                ));
            }
        }
    }

    Ok(())
}

/// The diagnostic and message for an entrypoint whose block could not be read or
/// spliced — one wording whether the write pass meets it or the pre-write pass
/// does first (§FS-init.1.2.6).
pub(super) fn update_failure(path: &Path, err: &anyhow::Error) -> (anyhow::Error, String) {
    // Forward slashes on every platform, like report paths (§FS-errors.2.2) —
    // Windows must not leak backslashes into the message.
    let message = format!("update {}: {err}", format_path(path));
    // §FS-distribution.3.3.2: classify while the original source is held.
    let io = err.downcast_ref::<std::io::Error>();
    let mut source = crate::model::OperationDiagnostic::new(
        if io.is_some() {
            "filesystem"
        } else {
            "operation"
        },
        "init",
        message.clone(),
    );
    source.path = Some(format_path(path));
    if let Some(io) = io {
        source.details = serde_json::json!({"os_error":io.raw_os_error()});
    }
    (source.into(), message)
}
