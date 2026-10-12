//! The template one `grund init` run lays (§FS-init.1.6): which built-in template
//! its `--template` names, whether `--scaffold` can be honoured, the note an
//! explicit template earns over a config the target already has, and the starter
//! files `--scaffold` writes.
//!
//! What each template *holds* is data and lives in `templates/assets.rs`
//! (§AR-system.2.11); this file is the run's half — every decision here is about
//! one invocation, which is why it sits beside the run rather than the payload
//! (§AR-core-module-layout.1).

use std::fs;
use std::path::Path;

use super::init_output::{InitError, InitEvent, verb_wrote};
use crate::sources::path_exists;
use crate::templates::Template;

/// The template a run lays, settled from its `--template` and `--scaffold`
/// before anything is read or written. An unknown name and `none` beside
/// `--scaffold` are refused here, so a refused run writes nothing
/// (§FS-init.1.6.3, §FS-init.1.6.4). `None` is the default template
/// (§FS-init.1.6.1).
pub(super) fn resolve_template(name: Option<&str>, scaffold: bool) -> Result<Template, String> {
    let template = match name {
        None => Template::default(),
        Some(name) => Template::from_name(name).ok_or_else(|| {
            let built_in = Template::ALL.map(Template::name);
            format!(
                "unknown template `{name}` — the built-in templates are {}; a template from a path is not supported yet",
                built_in.join(" and ")
            )
        })?,
    };
    if scaffold && !template.has_starter_files() {
        return Err(format!(
            "--scaffold has nothing to lay: the {} template has no starter files",
            template.name()
        ));
    }
    Ok(template)
}

/// The note an explicitly named template earns when the target already has a
/// config, which `init` never writes over (§FS-init.1.6.5): the flag must not
/// silently do nothing. `existing` is the config as the report names it. The
/// wording holds for a preview and a write alike, because neither writes it.
pub(super) fn template_config_note(template: Template, existing: &str) -> String {
    format!(
        "{existing} exists, so the {} template's grund.toml is not written — init never overwrites a config",
        template.name()
    )
}

/// One run's starter-file writes (§FS-init.2.1.3): the files `--scaffold` lays,
/// each at its path under `target`.
pub(super) struct StarterWrite<'a> {
    pub target: &'a Path,
    pub files: &'a [(String, String)],
    pub force: bool,
    pub dry_run: bool,
    pub diagnostic: &'a mut Option<anyhow::Error>,
    pub events: &'a mut Vec<InitEvent>,
}

/// Lay `files`: an existing one is reported `exists ` and left alone unless
/// `--force` (§FS-init.3.3, §FS-init.3.4), a missing one is written with its
/// directories. Returns whether any was written — or, under `--dry-run`, would
/// have been. A failed write keeps the events reported so far and the source
/// I/O error beside the message (§FS-distribution.3.3.2).
pub(super) fn lay_starter_files(args: StarterWrite<'_>) -> Result<bool, InitError> {
    let StarterWrite {
        target,
        files,
        force,
        dry_run,
        diagnostic,
        events,
    } = args;
    let mut any_change = false;
    let mut fail = |path: &Path, err: &std::io::Error, message: String| {
        *diagnostic =
            Some(crate::model::OperationDiagnostic::filesystem(path, err, message.clone()).into());
        message
    };
    for (rel, contents) in files {
        let dest = target.join(rel);
        if !force && path_exists(&dest) {
            events.push(InitEvent {
                verb: "exists",
                path: rel.clone(),
            });
            continue;
        }
        if !dry_run
            && let Some(parent) = dest.parent()
            && let Err(err) = fs::create_dir_all(parent)
        {
            let message = fail(parent, &err, format!("create {}: {err}", parent.display()));
            return Err(InitError::with_events(std::mem::take(events), message));
        }
        if !dry_run && let Err(err) = fs::write(&dest, contents) {
            let message = fail(&dest, &err, format!("write {}: {err}", dest.display()));
            return Err(InitError::with_events(std::mem::take(events), message));
        }
        events.push(InitEvent {
            verb: verb_wrote(dry_run),
            path: rel.clone(),
        });
        #[cfg(feature = "test-binding-writes")]
        if !dry_run {
            if let Err(error) = super::binding_write_fault::after_write() {
                let message = fail(&dest, &error, error.to_string());
                return Err(InitError::with_events(std::mem::take(events), message));
            }
        }
        any_change = true;
    }
    Ok(any_change)
}
