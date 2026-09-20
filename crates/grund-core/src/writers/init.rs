use std::fs;
use std::path::PathBuf;

use super::init_block::{
    AgentsUpdateResult, update_agents_block, write_or_update_canonical_agent_entrypoint,
};
pub(crate) use super::init_guidance::init_fs_home;
use super::init_guidance::{InitNext, docs_scaffold};
use super::init_notes::{duplicate_agent_entrypoint_notes, shadowed_claude_entrypoint_note};
use super::init_plan::{InitAgentEntrypointSelection, selected_init_agent_entrypoints};
use super::init_render::{agents_workspace_members_section, init_pending_effective_config};
use super::init_target::{refuse_init_global_instruction_paths, refuse_init_target};
use crate::checker::configured_rule_sentences;
use crate::config::{Config, config_file_in, display_path};
use crate::model::{Diagnostic, Finding, FindingSite, format_path};
use crate::scanner::{
    CANONICAL_AGENT_ENTRYPOINT, CanonicalSurfaceReach, InitCompanionAgentEntrypoint,
    effective_scope_reads_any_file, scan_tree,
};
use crate::templates::{
    ConversationSurface, render_agents_append_block, render_agents_md_from_block, render_grund_toml,
};
use crate::workspace::populate_workspace_boundary;

#[derive(Clone)]
pub struct InitOpts {
    pub target: PathBuf,
    /// Explicit generated project identity. When absent, `init` uses the
    /// target-local configured name before the basename (§FS-init.2.3.8).
    pub name: Option<String>,
    /// `--description` — pending one-line `project_description` for a freshly
    /// written config (§FS-init.1, §DF-workspace-member-descriptions).
    pub description: Option<String>,
    pub docs: bool,
    pub force: bool,
    pub dry_run: bool,
    /// `--check` — the `--dry-run` preview taken as a verdict (§FS-init.1):
    /// writes nothing, reports what `--dry-run` reports, and leaves the caller
    /// to exit `1` when any reported event is a change (§FS-init.4.1). It implies
    /// `dry_run` inside `init` rather than opening a second path through it.
    pub check: bool,
    /// `--no-vcs` — scaffold into a target no version-control marker covers
    /// (§FS-init.1.2.3). Lifts that rule and only that one; it is not `--force`,
    /// which decides whether files `init` owns get overwritten (§FS-init.3).
    pub no_vcs: bool,
    pub agent_selection: InitAgentEntrypointSelection,
}

impl Default for InitOpts {
    fn default() -> Self {
        Self {
            target: PathBuf::from("."),
            name: None,
            description: None,
            docs: false,
            force: false,
            dry_run: false,
            check: false,
            no_vcs: false,
            agent_selection: InitAgentEntrypointSelection::default(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InitEvent {
    pub verb: &'static str,
    pub path: String,
}

impl InitEvent {
    /// Whether this event reports work rather than a path that was already
    /// current. Every verb but `exists` is a change — `wrote`/`appended`/
    /// `updated` and their `would-` forms alike. The one definition of the
    /// predicate: it suppresses the `next:` block (§FS-init.2.2.2) and it decides
    /// the `--check` exit code (§FS-init.4.1), which is why those two agree.
    pub fn is_change(&self) -> bool {
        self.verb != "exists"
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InitOutput {
    pub events: Vec<InitEvent>,
    /// Located validation findings discovered before any write (§FS-rules.4).
    /// They are ordinary report rows and make `init` exit 1, not operational
    /// failures that would exit 2.
    pub errors: Vec<Finding>,
    /// Things the run could not do that the caller would otherwise have to
    /// notice for itself (§FS-init.2.3.4.17.4). Reported, never fatal.
    pub notes: Vec<String>,
    pub next: Option<InitNext>,
    /// The run's warning channel (§FS-distribution.3.1): the `[workspace]`
    /// cautions the walk-up settled — §FS-check.4.7's absorbed scan,
    /// §FS-check.4.10's unread opted-out block and §FS-workspace.6.1.7.5's
    /// undecidable ancestor claim. `init` expands the outermost workspace above
    /// its target to teach the alias set, so it resolves a block's member
    /// boundary like every other walking command and owes the reader the same
    /// lines (§FS-check.2.1.1).
    pub warnings: Vec<Finding>,
}

impl InitOutput {
    /// Whether the run reported anything left to do — the verdict `--check`
    /// draws from the report it just printed (§FS-init.4.1). Notes and the
    /// `next:` block are deliberately not consulted: a note is a report, not a
    /// finding.
    pub fn has_pending_changes(&self) -> bool {
        self.events.iter().any(InitEvent::is_change)
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InitError {
    pub output: InitOutput,
    pub message: String,
}

impl InitError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            output: InitOutput::default(),
            message: message.into(),
        }
    }

    fn with_events(events: Vec<InitEvent>, message: impl Into<String>) -> Self {
        Self {
            output: InitOutput {
                events,
                ..InitOutput::default()
            },
            message: message.into(),
        }
    }
}

impl std::fmt::Display for InitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for InitError {}

/// Add v11's conditional chapter-rule section while leaving the v10 bytes
/// untouched for every project without a rule kind (§FS-rules.9).
fn render_chapter_rules(
    mut block: String,
    rule_kind_enabled: bool,
    rows: &[(String, String)],
) -> String {
    if !rule_kind_enabled {
        return block;
    }
    block = block.replacen(
        "Grounding with grund (v10)",
        "Grounding with grund (v11)",
        1,
    );
    let mut section = String::from(
        "### Chapter rules\n\n`must`/`must not` are `grund check` errors; `should`/`should not` are suggestions (`grund check --suggestions`).\n\n",
    );
    for (origin, sentence) in rows {
        section.push_str(&format!("- {sentence} §{origin}\n"));
    }
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
/// Why the managed block is rendered once and reused for both surfaces: the
/// workspace-members walk-up is non-trivial I/O for a large workspace and
/// produces byte-identical output each time. The selected entrypoint plan
/// determines whether a missing self `AGENTS.md` should be treated as
/// about-to-exist; companion-only init must not link to a missing canonical
/// entrypoint. Two surfaces at most: the local-conversation sentence differs
/// between the Claude entrypoints and everything else, and nothing else in the
/// block does. The linked variant is rendered only when a Claude entrypoint is
/// actually selected, so the common run still walks the workspace once.
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
        init_pending_effective_config(&target, name.as_deref(), description.as_deref())
            .map_err(|err| InitError::new(err.to_string()))?;
    // §FS-init.2.2.2: the guidance probe consumes the exact §AR-workspace.6
    // boundary used by scanner commands. Keep this best-effort: the existing
    // workspace renderer owns init's diagnostics and error-tolerant behavior.
    let _ = populate_workspace_boundary(&mut init_config);
    let reach = CanonicalSurfaceReach::for_config(&init_config);
    // §FS-rules.4 / §FS-init.2.3.5: validate scanned rule declarations before
    // any entrypoint write, then reuse their exact titles in managed guidance.
    let rule_kind_enabled = init_config.kinds.iter().any(|kind| kind.rules);
    let rule_rows = if rule_kind_enabled {
        let (findings, errors) = scan_tree(&init_config, Some(&target), true)
            .map_err(|err| InitError::new(err.to_string()))?;
        if let Some((path, message)) = errors.first() {
            return Err(InitError::new(format!("{}: {message}", path.display())));
        }
        match configured_rule_sentences(&findings, &init_config) {
            Ok(rows) => rows,
            Err(diagnostic) => {
                return Ok(InitOutput {
                    errors: vec![init_finding(&init_config, diagnostic)],
                    ..InitOutput::default()
                });
            }
        }
    } else {
        Vec::new()
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

    // §FS-init.2.3.4.15, §FS-check.4.8: walked once and handed to both surfaces —
    // the section does not vary by surface, and this walk is where every block is
    // asked whether its members swallowed its scan, once per run.
    let (workspace_members, run_warnings) = agents_workspace_members_section(
        &resolved_name,
        &init_config,
        &target,
        agent_entrypoints.canonical,
    );
    // Render the managed block once and reuse it for both surfaces: the two
    // surfaces differ in one sentence only (§FS-init.2.3.4.17.2).
    let render_block = |surface| {
        let block =
            render_agents_append_block(&resolved_name, &init_config, &workspace_members, surface);
        render_chapter_rules(block, rule_kind_enabled, &rule_rows)
    };
    let agents_block = render_block(ConversationSurface::Plain);
    let claude_block = agent_entrypoints
        .companions
        .iter()
        .any(|entrypoint| {
            ConversationSurface::for_entrypoint(entrypoint.path()) == ConversationSurface::Linked
        })
        .then(|| render_block(ConversationSurface::Linked));
    let agents_contents = render_agents_md_from_block(&resolved_name, &agents_block);
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
            &agents_block,
            force,
            dry_run,
        ) {
            Ok(event) => {
                any_change |= event.is_change();
                events.push(event);
            }
            Err(message) => return Err(InitError::with_events(events, message)),
        }
        workflow_entrypoint = Some(CANONICAL_AGENT_ENTRYPOINT.to_string());
    }

    for entrypoint in agent_entrypoints.companions {
        let path_ref = entrypoint.path();
        // The Claude entrypoints teach the linked form; every other companion
        // gets the plain-location block (§FS-init.2.3.4.17.2).
        let entrypoint_block = match ConversationSurface::for_entrypoint(path_ref) {
            ConversationSurface::Linked => claude_block.as_deref().unwrap_or(&agents_block),
            ConversationSurface::Plain => &agents_block,
        };
        let rel = path_ref
            .strip_prefix(&target)
            .unwrap_or(path_ref)
            .to_path_buf();
        let rel = format_path(&rel);
        if workflow_entrypoint.is_none() {
            workflow_entrypoint = Some(rel.clone());
        }
        match entrypoint {
            InitCompanionAgentEntrypoint::Existing(path) => {
                match update_agents_block(&path, entrypoint_block, &rel, dry_run) {
                    Ok(AgentsUpdateResult::Appended) => {
                        events.push(InitEvent {
                            verb: verb_appended(dry_run),
                            path: rel,
                        });
                        any_change = true;
                    }
                    Ok(AgentsUpdateResult::Updated) => {
                        events.push(InitEvent {
                            verb: verb_updated(dry_run),
                            path: rel,
                        });
                        any_change = true;
                    }
                    Ok(AgentsUpdateResult::Unchanged) => events.push(InitEvent {
                        verb: "exists",
                        path: rel,
                    }),
                    Err(err) => {
                        return Err(InitError::with_events(
                            events,
                            // Forward slashes on every platform, like report
                            // paths (§FS-errors.2.2) — Windows must not leak
                            // backslashes into the message.
                            format!("update {}: {err}", format_path(&path)),
                        ));
                    }
                }
            }
            InitCompanionAgentEntrypoint::MissingAlias(path) => {
                if !dry_run
                    && let Some(parent) = path.parent()
                    && let Err(err) = fs::create_dir_all(parent)
                {
                    return Err(InitError::with_events(
                        events,
                        format!("create {}: {err}", parent.display()),
                    ));
                }
                if !dry_run && let Err(err) = fs::write(&path, entrypoint_block) {
                    return Err(InitError::with_events(
                        events,
                        format!("write {}: {err}", path.display()),
                    ));
                }
                events.push(InitEvent {
                    verb: verb_wrote(dry_run),
                    path: rel,
                });
                any_change = true;
            }
        }
    }

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
            return Err(InitError::with_events(
                events,
                format!("write {}: {err}", config_dest.display()),
            ));
        }
        events.push(InitEvent {
            verb: verb_wrote(dry_run),
            path: config_rel.to_string(),
        });
        any_change = true;
    }

    let fs_home = init_fs_home(&init_config);
    let files: Vec<(String, String)> = if docs {
        docs_scaffold(&fs_home)
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
            return Err(InitError::with_events(
                events,
                format!("create {}: {err}", parent.display()),
            ));
        }
        if !dry_run && let Err(err) = fs::write(&dest, contents) {
            return Err(InitError::with_events(
                events,
                format!("write {}: {err}", dest.display()),
            ));
        }
        events.push(InitEvent {
            verb: verb_wrote(dry_run),
            path: rel.clone(),
        });
        any_change = true;
    }

    let next = any_change.then(|| {
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
                return Err(InitError::with_events(
                    events,
                    format!("inspect {}: {message}", path.display()),
                ));
            }
        }
    }
    Ok(InitOutput {
        events,
        errors: Vec::new(),
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
    }
}

/// Stderr verb for a newly written file. `--dry-run` reports `would-write `
/// instead of `wrote `; otherwise the verbs match a real run (§FS-init.2.2).
pub(super) fn verb_wrote(dry_run: bool) -> &'static str {
    if dry_run { "would-write" } else { "wrote" }
}

pub(super) fn verb_appended(dry_run: bool) -> &'static str {
    if dry_run { "would-append" } else { "appended" }
}

pub(super) fn verb_updated(dry_run: bool) -> &'static str {
    if dry_run { "would-update" } else { "updated" }
}
