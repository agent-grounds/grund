use std::fs;
use std::path::Path;

use super::init_block::write_or_update_canonical_agent_entrypoint;
pub(crate) use super::init_guidance::init_fs_home;
use super::init_guidance::{InitNext, docs_scaffold_for_config};
use super::init_notes::{duplicate_agent_entrypoint_notes, shadowed_claude_entrypoint_note};
use super::init_plan::selected_init_agent_entrypoints;
use super::init_render::{agents_workspace_members_section, init_pending_effective_config};
use super::init_target::{refuse_init_global_instruction_paths, refuse_init_target};
use crate::checker::{
    chapter_rules_section, configured_rule_sentences, declared_workspace_vocabulary,
};
use crate::config::{Config, config_file_in, display_path};
use crate::model::{Catalog, Diagnostic, Finding, FindingSite, format_path};
use crate::scanner::{
    CANONICAL_AGENT_ENTRYPOINT, CanonicalSurfaceReach, effective_scope_reads_any_file, scan_tree,
};
use crate::templates::{
    ConversationSurface, render_agents_append_block, render_agents_md_from_block, render_grund_toml,
};
use crate::workspace::populate_workspace_boundary;

pub use super::init_output::{InitError, InitEvent, InitOpts, InitOutput};

/// Insert the shared, entrypoint-relative chapter-rule section while leaving
/// non-rule projects untouched (§FS-init.2.3.5.10).
fn render_chapter_rules(mut block: String, section: Option<&str>) -> String {
    let Some(section) = section else {
        return block;
    };
    block = block.replacen(
        "Grounding with grund (v14)",
        "Grounding with grund (v15)",
        1,
    );
    let insertion = block
        .find("\n### Clickable citations")
        .or_else(|| block.find("\n<!-- END GRUND MANAGED BLOCK -->"));
    if let Some(index) = insertion {
        block.insert_str(index, &format!("\n{section}"));
    } else {
        block.push_str(&format!("\n{section}"));
    }
    block
}

/// Scaffold a grund setup into `opts.target`: the agent-instruction
/// entrypoints, `grund.toml`, and — with `--docs` — the documentation stubs.
///
/// Why the effective config is read before the entrypoint plan: one selection
/// rule depends on it. A companion symlinked to `AGENTS.md` leaves its agent
/// covered unless that key makes the canonical file unable to carry that
/// agent's form.
///
/// Why the base block is rendered once and reused for both surfaces: the
/// workspace-members walk-up is non-trivial I/O for a large workspace and
/// produces byte-identical output each time. The selected entrypoint plan
/// determines whether a missing self `AGENTS.md` should be treated as
/// about-to-exist; companion-only init must not link to a missing canonical
/// entrypoint. Two base surfaces at most: the local-conversation sentence differs
/// between the Claude entrypoints and everything else. The chapter-rule section
/// is inserted per entrypoint so its citation destinations are relative to that
/// file (§FS-init.2.3.5.10). The linked base is rendered only when a Claude
/// entrypoint is actually selected, so the common run still walks the workspace once.
///
/// Why the duplicate-entrypoint notes and the Claude companions are computed
/// before the companion loop: the loop consumes the plan. `init` creates one
/// entrypoint per agent, but a repository that already carries two keeps both,
/// and this run is where that shows; and the entrypoint the symlink note names
/// is the one this run makes current, when it makes one current.
///
/// What `init` will and will not overwrite: `grund.toml` is the project's
/// configuration — the surface a repo customizes (kinds, marker, scan scope,
/// …). `init` writes the canonical template only when the target has **no**
/// config under either discovery name; an existing one is never overwritten,
/// not even with `--force`, and is reported under the name it was found at so a
/// repo on the `.agents/` form never grows the redundant pair. `--force`
/// targets the things `init` owns end to end — the managed agent-instructions
/// block and the `--docs` scaffold stubs — not the user's settings.
///
/// Why the shadowed-entrypoint note is emitted: the committed `link` opinion is
/// rendered per entrypoint, and a Claude entrypoint that is a symlink to
/// `AGENTS.md` is the canonical file — which every other agent reads too, so it
/// must keep the plain form.
pub fn init(opts: InitOpts) -> std::result::Result<InitOutput, InitError> {
    init_run(opts, &mut None, &mut Vec::new())
}

/// Keep failure sources beside partial output without changing InitError
/// (§FS-distribution.3.1, §FS-distribution.3.3.2).
pub(crate) fn init_with_diagnostics(
    opts: InitOpts,
) -> (
    std::result::Result<InitOutput, InitError>,
    Option<anyhow::Error>,
    Vec<Finding>,
) {
    let mut diagnostic = None;
    let mut cautions = Vec::new();
    let result = init_run(opts, &mut diagnostic, &mut cautions);
    (result, diagnostic, cautions)
}

fn init_run(
    opts: InitOpts,
    diagnostic: &mut Option<anyhow::Error>,
    cautions: &mut Vec<Finding>,
) -> std::result::Result<InitOutput, InitError> {
    let InitOpts {
        target,
        name,
        description,
        docs,
        force,
        dry_run,
        check,
        no_vcs,
        agent_selection,
    } = opts;
    // §FS-init.1: `--check` is the `--dry-run` run taken as a verdict, so it
    // suppresses writes through that same flag rather than a second code path —
    // which is what makes the two reports identical by construction.
    let dry_run = dry_run || check;
    // §FS-init.1: `--description` mirrors the config-side single-line rule
    // (§FS-config.3) — reject line breaks before any file is touched.
    if let Some(description) = &description
        && (description.contains('\n') || description.contains('\r'))
    {
        return Err(InitError::new(
            "--description must be a single line".to_string(),
        ));
    }
    // §FS-init.1, §FS-init.1.2: what `<path>` is, and whether it may be
    // scaffolded at all. The companion rule that needs the entrypoint plan runs
    // below, where that plan first exists; both are ahead of every write.
    if let Some(message) = refuse_init_target(&target, no_vcs) {
        return Err(InitError::new(message));
    }

    // §FS-init.2.3.8: render agent instructions against the config `init` leaves
    // in place; select the name from explicit flag, target config, then basename.

    // §FS-init.2.1.1.1, §FS-init.2.3.4.17: do both before the entrypoint plan,
    // so each renderer consumes the same identity and effective grammar.
    let (mut init_config, resolved_name) =
        init_pending_effective_config(&target, name.as_deref(), description.as_deref()).map_err(
            |err| {
                let message = err.to_string();
                *diagnostic = Some(err);
                InitError::new(message)
            },
        )?;
    // §FS-init.2.2.2: the guidance probe consumes the exact §AR-workspace.6
    // boundary used by scanner commands. Keep this best-effort: the existing
    // workspace renderer owns init's diagnostics and error-tolerant behavior.
    let _ = populate_workspace_boundary(&mut init_config);
    let reach = CanonicalSurfaceReach::for_config(&init_config);
    // §FS-rules.4 / §FS-init.2.3.5: validate scanned rule declarations before
    // any entrypoint write, then reuse their exact titles in managed guidance.
    let rule_kind_enabled = init_config.kinds.iter().any(|kind| kind.rules);
    let (rule_rows, rule_errors, rule_findings) = if rule_kind_enabled {
        let (findings, errors) = scan_tree(&init_config, Some(&target), true).map_err(|err| {
            let message = err.to_string();
            *diagnostic = Some(err);
            InitError::new(message)
        })?;
        if let Some((path, message)) = errors.first() {
            // §FS-distribution.3.3.2: scanner errors carry a source path.
            let mut source = crate::model::OperationDiagnostic::new(
                "filesystem",
                "io",
                format!("{}: {message}", path.display()),
            );
            source.path = Some(format_path(path));
            *diagnostic = Some(source.into());
            return Err(InitError::new(format!("{}: {message}", path.display())));
        }
        // §FS-rules.4.1: the workspace this project's own config declares, which
        // is the one `check` in this same directory resolves against — never the
        // one climbed to above, which is teaching rather than judging.
        let vocab = declared_workspace_vocabulary(&init_config);
        match configured_rule_sentences(&findings, &init_config, &vocab) {
            Ok(rules) => (
                rules.rows,
                rules
                    .unverifiable
                    .into_iter()
                    .map(|diagnostic| init_finding(&init_config, diagnostic))
                    .collect::<Vec<_>>(),
                findings,
            ),
            // §FS-rules.4: an invalid rule, and only an invalid rule, withholds
            // the write — the managed block stays byte-for-byte untouched and
            // the run exits nonzero.
            Err(diagnostic) => {
                return Ok(InitOutput {
                    errors: vec![init_finding(&init_config, diagnostic)],
                    ..InitOutput::default()
                });
            }
        }
    } else {
        (Vec::new(), Vec::new(), Catalog::default())
    };

    let agent_entrypoints = match selected_init_agent_entrypoints(&target, &agent_selection, reach)
    {
        Ok(entrypoints) => entrypoints,
        Err((path, message)) => {
            return Err(InitError::new(format!(
                "inspect {}: {message}",
                path.display()
            )));
        }
    };

    // §FS-init.1.2.2: the planned entrypoint paths are known now, so check them
    // against the user-global instruction files `grund integrations --write`
    // owns (§FS-integrations.4.3.8) before any of them is written.
    if let Some(message) =
        refuse_init_global_instruction_paths(&agent_entrypoints.planned_paths(&target))
    {
        return Err(InitError::new(message));
    }

    // §FS-init.2.3.4.15, §FS-check.3.29: walked once and handed to both surfaces —
    // the section does not vary by surface, and this walk is where every block is
    // asked whether its members swallowed its scan, once per run.
    let (workspace_members, run_warnings) = agents_workspace_members_section(
        &resolved_name,
        &init_config,
        &target,
        agent_entrypoints.canonical,
    );
    // §FS-distribution.3.3.2: later write failures retain these run cautions.
    *cautions = run_warnings.clone();
    // Render the base once per conversation surface (§FS-init.2.3.4.17.2).
    // §FS-init.2.3.5.10: the shared rule renderer adds destinations per file.
    let render_block = |surface| {
        render_agents_append_block(
            &resolved_name,
            init_config.project(),
            init_config.compiled(),
            &workspace_members,
            surface,
        )
    };
    let add_chapter_rules = |block: String, path: &Path| {
        // §FS-init.2.3.5.10: a pending entrypoint cannot be canonicalized yet;
        // give the resolver a path under the loaded config's root instead.
        let path = init_config
            .root
            .join(path.strip_prefix(&target).unwrap_or(path));
        let section = rule_kind_enabled
            .then(|| chapter_rules_section(&init_config, &path, &rule_findings, &rule_rows));
        render_chapter_rules(block, section.as_deref())
    };
    let agents_block = render_block(ConversationSurface::Plain);
    let claude_block = agent_entrypoints
        .companions
        .iter()
        .any(|entrypoint| {
            ConversationSurface::for_entrypoint(entrypoint.path()) == ConversationSurface::Linked
        })
        .then(|| render_block(ConversationSurface::Linked));
    let canonical_block = add_chapter_rules(
        agents_block.clone(),
        &target.join(CANONICAL_AGENT_ENTRYPOINT),
    );
    let agents_contents = render_agents_md_from_block(&resolved_name, &canonical_block);
    // §FS-init.2.1.1, §FS-init.2.3.4.17.4: both computed before the companion loop
    // consumes the plan.
    let claude_companions = agent_entrypoints.companions_of_claude(&target);
    let mut notes = duplicate_agent_entrypoint_notes(&target, &agent_entrypoints, reach, dry_run);
    let mut workflow_entrypoint = None;
    // Track whether any path changed (or, under --dry-run, *would* change).
    // The `next:` block is suppressed when every reported path is `exists `,
    // since the user already has a complete grund setup (§FS-init.2.2.2).
    let mut any_change = false;
    let mut events = Vec::new();
    if agent_entrypoints.canonical {
        match write_or_update_canonical_agent_entrypoint(
            &target,
            CANONICAL_AGENT_ENTRYPOINT,
            &agents_contents,
            &canonical_block,
            force,
            dry_run,
        ) {
            Ok(event) => {
                any_change |= event.is_change();
                events.push(event);
                #[cfg(feature = "test-binding-writes")]
                if !dry_run && events.last().is_some_and(InitEvent::is_change) {
                    if let Err(error) = super::binding_write_fault::after_write() {
                        *diagnostic = Some(
                            crate::model::OperationDiagnostic::filesystem(
                                &target.join(CANONICAL_AGENT_ENTRYPOINT),
                                &error,
                                error.to_string(),
                            )
                            .into(),
                        );
                        return Err(InitError::with_events(events, error.to_string()));
                    }
                }
            }
            Err(message) => return Err(InitError::with_events(events, message)),
        }
        workflow_entrypoint = Some(CANONICAL_AGENT_ENTRYPOINT.to_string());
    }

    super::init_companions::write_companions(super::init_companions::CompanionWrite {
        target: &target,
        companions: agent_entrypoints.companions,
        agents_block: &agents_block,
        claude_block: claude_block.as_deref(),
        add_chapter_rules,
        dry_run,
        diagnostic,
        events: &mut events,
        workflow_entrypoint: &mut workflow_entrypoint,
        any_change: &mut any_change,
    })?;

    // `grund.toml` is the project's configuration (§GOAL-configurable): written
    // only when the target has none (§FS-config.1), never overwritten, reported
    // under the name found (§FS-init.2.4.1, §FS-check.4.3, §FS-init.3).
    if let Some(existing) = config_file_in(&target) {
        let rel = existing
            .strip_prefix(&target)
            .unwrap_or(&existing)
            .to_path_buf();
        events.push(InitEvent {
            verb: "exists",
            path: format_path(&rel),
        });
    } else {
        // §DF-config-file-location.2.3: the bare, root-visible form is the one
        // `init` generates, so the default a new project meets is the one the
        // rest of the ecosystem uses.
        let config_rel = "grund.toml";
        // No `create_dir_all`: the destination's parent is `target`, already
        // verified to be an existing directory above.
        let config_dest = target.join(config_rel);
        if !dry_run
            && let Err(err) = fs::write(
                &config_dest,
                render_grund_toml(&resolved_name, description.as_deref()),
            )
        {
            // §FS-distribution.3.3.2: retain source I/O data before string projection.
            *diagnostic = Some(
                crate::model::OperationDiagnostic::filesystem(
                    &config_dest,
                    &err,
                    format!("write {}: {err}", config_dest.display()),
                )
                .into(),
            );
            return Err(InitError::with_events(
                events,
                format!("write {}: {err}", config_dest.display()),
            ));
        }
        events.push(InitEvent {
            verb: verb_wrote(dry_run),
            path: config_rel.to_string(),
        });
        #[cfg(feature = "test-binding-writes")]
        if !dry_run {
            if let Err(error) = super::binding_write_fault::after_write() {
                *diagnostic = Some(
                    crate::model::OperationDiagnostic::filesystem(
                        &config_dest,
                        &error,
                        error.to_string(),
                    )
                    .into(),
                );
                return Err(InitError::with_events(events, error.to_string()));
            }
        }
        any_change = true;
    }

    let fs_home = init_fs_home(&init_config);
    let files: Vec<(String, String)> = if docs {
        docs_scaffold_for_config(&fs_home, &init_config)
    } else {
        Vec::new()
    };
    for (rel, contents) in &files {
        let dest = target.join(rel);
        if !force && dest.exists() {
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
                events,
                format!("create {}: {err}", parent.display()),
            ));
        }
        if !dry_run && let Err(err) = fs::write(&dest, contents) {
            // §FS-distribution.3.3.2: retain source I/O data before string projection.
            *diagnostic = Some(
                crate::model::OperationDiagnostic::filesystem(
                    &dest,
                    &err,
                    format!("write {}: {err}", dest.display()),
                )
                .into(),
            );
            return Err(InitError::with_events(
                events,
                format!("write {}: {err}", dest.display()),
            ));
        }
        events.push(InitEvent {
            verb: verb_wrote(dry_run),
            path: rel.clone(),
        });
        #[cfg(feature = "test-binding-writes")]
        if !dry_run {
            if let Err(error) = super::binding_write_fault::after_write() {
                *diagnostic = Some(
                    crate::model::OperationDiagnostic::filesystem(&dest, &error, error.to_string())
                        .into(),
                );
                return Err(InitError::with_events(events, error.to_string()));
            }
        }
        any_change = true;
    }

    // §FS-init.2.2.2.1: only `exists`/`updated` lines, no `--docs`, and the
    // effective FS home on disk — the refresh `grund check` sends people down.
    let refresh_of_complete_setup = !docs
        && events
            .iter()
            .all(|event| !event.is_change() || event.verb == verb_updated(dry_run))
        && target.join(fs_home.path()).exists();
    let next = (any_change && !refresh_of_complete_setup).then(|| {
        // §FS-init.2.2.2: only no-`--docs` guidance asks this question. The probe
        // uses the effective config selected above and exits on its first file.
        let scan_reads_file = !docs && effective_scope_reads_any_file(&init_config);
        InitNext {
            docs,
            entrypoint: workflow_entrypoint
                .unwrap_or_else(|| CANONICAL_AGENT_ENTRYPOINT.to_string()),
            fs_home,
            scan_reads_file,
        }
    });
    // §FS-init.2.3.4.17.4: silence here would read as the committed `link` opinion
    // simply not working.
    if reach == CanonicalSurfaceReach::PlainEntrypointsOnly {
        match shadowed_claude_entrypoint_note(&target, &claude_companions, dry_run) {
            Ok(Some(note)) => notes.push(note),
            Ok(None) => {}
            // The same hard failure the selection gives for a path it cannot
            // inspect: this note is the only place the state is visible, so
            // dropping it on an unreadable link would report a clean run.
            Err((path, message)) => {
                // §FS-distribution.3.3.2: inspect failures keep their known path.
                let mut source = crate::model::OperationDiagnostic::new(
                    "filesystem",
                    "io",
                    format!("inspect {}: {message}", path.display()),
                );
                source.path = Some(format_path(&path));
                *diagnostic = Some(source.into());
                return Err(InitError::with_events(
                    events,
                    format!("inspect {}: {message}", path.display()),
                ));
            }
        }
    }
    Ok(InitOutput {
        events,
        // §FS-rules.4.1.2: it is the write that is not withheld, not the failure
        // that is forgiven — these errors still carry the run to exit 1.
        errors: rule_errors,
        notes,
        next,
        warnings: run_warnings,
    })
}

fn init_finding(config: &Config, diagnostic: Diagnostic) -> Finding {
    Finding {
        severity: "error",
        code: diagnostic.code,
        path: diagnostic.path.map(|path| display_path(config, &path)),
        line: diagnostic.line,
        column: diagnostic.column,
        message: diagnostic.message,
        sites: diagnostic
            .sites
            .into_iter()
            .map(|site| FindingSite {
                path: display_path(config, &site.path),
                line: site.line,
            })
            .collect(),
        authority: Vec::new(),
    }
}

pub(super) use super::init_output::{verb_appended, verb_updated, verb_wrote};
