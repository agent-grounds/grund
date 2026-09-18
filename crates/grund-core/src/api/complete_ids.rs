//! The published ID-completion contract (§AR-system.2.9): the candidates a shell
//! frontend offers for a partly typed ID or alias path (§FS-completions,
//! §FS-workspace.8.4).
//!
//! Config and scan errors are returned to the caller rather than printed, so a
//! frontend decides whether to show them mid-completion (§AR-bindings.2).

use anyhow::Result;
use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::config::Config;
use crate::grammar::render_id;
use crate::model::{Finding, Findings};
use crate::resolver::load_workspace_context;

use super::report::context_run_warnings;

#[derive(Clone)]
pub struct CompleteIdsOpts {
    pub path: PathBuf,
    pub path_provided: bool,
    pub prefix: String,
    pub sections: bool,
}

impl Default for CompleteIdsOpts {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            path_provided: false,
            prefix: String::new(),
            sections: false,
        }
    }
}

/// Dynamic ID completion candidates for shell frontends. Config or scan errors
/// are returned to the caller so command frontends can decide whether to hide
/// them during tab completion.
///
/// A prefix that lands mid-path completes both: the deeper alias paths still to
/// be typed, and the IDs of the project already named.
pub fn complete_ids(opts: CompleteIdsOpts) -> Result<Vec<String>> {
    complete_ids_with_run_warnings(opts).1
}

/// [`complete_ids`] for a frontend that also renders the run's `[workspace]`
/// warnings (§FS-check.4.7, §FS-check.4.10, §FS-workspace.6.1). The candidate
/// list is a bare `Vec<String>`, so this is the only channel they have.
#[doc(hidden)]
pub fn complete_ids_with_run_warnings(
    opts: CompleteIdsOpts,
) -> (Vec<Finding>, Result<Vec<String>>) {
    let mut run_warnings = Vec::new();
    let candidates = complete_ids_run(opts, &mut run_warnings);
    (run_warnings, candidates)
}

fn complete_ids_run(opts: CompleteIdsOpts, run_warnings: &mut Vec<Finding>) -> Result<Vec<String>> {
    let context = load_workspace_context(&opts.path, opts.path_provided)?;
    *run_warnings = context_run_warnings(&context);
    let current_config = context
        .current_project()
        .map(|project| &project.config)
        .unwrap_or_else(|| context.render_config());
    let mut candidates = BTreeSet::new();
    // §FS-workspace.8.4: an alias path may itself carry slashes
    // (§FS-workspace.6.1), so the split is at the *last* `/` — the left names a
    // project, the right is its ID-prefix.
    if let Some((alias_prefix, id_prefix)) = opts.prefix.rsplit_once('/') {
        if !context.workspace_loaded {
            return Ok(Vec::new());
        }
        for alias in context.aliases() {
            let continuation = format!("{alias}/");
            // The exact prefix is withheld: re-offering what is already on the
            // line stalls the shell instead of advancing it.
            if continuation != opts.prefix && continuation.starts_with(&opts.prefix) {
                candidates.insert(continuation);
            }
        }
        if let Some(project) = context.project_by_alias(alias_prefix) {
            let complete_sections =
                opts.sections || id_prefix.contains(&project.config.section_separator);
            add_complete_id_candidates(
                &mut candidates,
                Some(alias_prefix),
                &project.config,
                &project.findings,
                complete_sections,
            );
        }
    } else {
        let complete_sections =
            opts.sections || opts.prefix.contains(&current_config.section_separator);
        if let Some(current_project) = context.current_project() {
            add_complete_id_candidates(
                &mut candidates,
                None,
                current_config,
                &current_project.findings,
                complete_sections,
            );
        }
        if context.workspace_loaded {
            for alias in context.aliases() {
                candidates.insert(format!("{alias}/"));
            }
        }
    }
    Ok(candidates
        .into_iter()
        .filter(|candidate| candidate.starts_with(&opts.prefix))
        .collect())
}

fn add_complete_id_candidates(
    candidates: &mut BTreeSet<String>,
    alias: Option<&str>,
    config: &Config,
    findings: &Findings,
    include_sections: bool,
) {
    let qualifier = alias.map(|alias| format!("{alias}/")).unwrap_or_default();
    for (id, decls) in &findings.declarations {
        let rendered = render_id(&config.grammar, id);
        if include_sections {
            for decl in decls {
                for section in decl.sections.keys() {
                    candidates.insert(format!(
                        "{}{}{}{}",
                        qualifier, rendered, config.section_separator, section
                    ));
                }
            }
        } else {
            candidates.insert(format!("{qualifier}{rendered}"));
        }
    }
}
