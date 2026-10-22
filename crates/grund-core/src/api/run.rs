//! One `check` run, single-project and workspace (§AR-system.2.9): resolve the
//! config, scan, check, and fold in the cautions and the config findings the run
//! earns (§FS-check.1, §FS-check.2, §FS-workspace.5).
//!
//! The run is what every data-returning check surface shares, and the api is the
//! lowest component that holds it. Process parsing and rendering stay in the
//! CLI, outside the engine (§AR-system.2.9.1, §AR-bindings.2).

use anyhow::Result;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::config_findings::config_diagnostics;
use super::path_report::PathReport;
use super::report::public_run_warnings;
use super::scope_cautions::{scan_scope_caution, workspace_path_caution};
use super::unread_roots::unread_root_warnings;
use crate::checker::{
    CheckWorkspace, check_chapter_rules, check_project, configured_scope, out_of_scope_references,
    out_of_scope_section_headings, parse_ad_hoc, parse_ad_hoc_with_workspace, path_report_scope,
    repair_index_citations, retain_diagnostics_in_report_scope, retain_findings_in_scope,
    sort_diagnostics, workspace_out_of_scope_references, workspace_out_of_scope_section_headings,
};
use crate::config::{Config, Invocation, ProjectRecords, Selection};
use crate::model::{CheckReport, Code, Diagnostic, Finding, Switch};
use crate::resolver::{
    WorkspaceCheckTarget, WorkspaceProject, load_workspace_projects, settled_run_warnings,
};
use crate::scanner::scan_tree;
use crate::workspace::{
    absent_only_workspace_caution, absent_optional_member_warnings, absent_optional_members,
    arm_optional_members, open, unlisted_workspace_block_errors,
};
use crate::writers::{ListedProject, WorkspaceBlocks, expected, listed_projects};

pub(crate) struct CheckRun {
    pub(crate) config: Config,
    pub(crate) report: CheckReport,
    pub(crate) had_scan_errors: bool,
    /// The root of each project the run checked, in check order; the report
    /// gate of `config migrate` compares them (§FS-config-migrate.5.2).
    pub(crate) projects: Vec<PathBuf>,
}

/// One `grund check` run over `path`: resolve the config, scan, check, and fold in
/// the cautions and warnings the run earns for its scope and its config files.
///
/// Why the scope caution is only a warning: it never changes the exit code. The
/// agent-entrypoint check (§FS-check.3.5) runs even when no source file is scanned,
/// so a missing or stale `AGENTS.md` block still reports normally and suppresses
/// both cautions.
#[cfg(test)]
pub(crate) fn run_check(
    path: &Path,
    path_provided: bool,
    force_require_grounding: bool,
    full: bool,
    ad_hoc_sentence: Option<&str>,
) -> Result<CheckRun> {
    let mut ignored_warnings = Vec::new();
    run_check_with_run_warnings(
        path,
        path_provided,
        force_require_grounding,
        full,
        ad_hoc_sentence,
        &mut ignored_warnings,
    )
}

/// Run `check` while preserving the root warnings a later workspace-expansion
/// refusal must not discard (§FS-check.4.10.8). The side channel is
/// returned data, never rendered here (§FS-distribution.3.1).
///
/// §FS-check.1.3.6.1: an explicit path below the config root is this run's **report**
/// scope and not its **resolution** scope, so the single-project arm below walks the
/// project's ordinary roots as well as the path, runs every rule over the whole of
/// that, and drops the diagnostics anchored outside the path before the report is
/// written. Two narrowings, and §AR-resolver.3.3 is why they are kept at visibly
/// different stages: `retain_findings_in_scope` narrows the scan result *before* any
/// rule runs, which is what `--full` additivity needs (§FS-check.1.3.4), while
/// `retain_diagnostics_in_report_scope` narrows the report *after* every rule has
/// run, which is what lets a path-scoped run resolve against the whole project.
/// Four things the path filter leaves alone: a run-level finding, which carries no
/// path; a finding that names a site inside the path whatever its anchor
/// (§FS-check.1.3.6.2); the §FS-check.3.5 agent-entrypoint probe's own finding, which
/// asks about the project root; and the scope cautions, which are asked of the report
/// scope rather than the walk. What it drops it does not forget: a file the wider walk
/// could not read leaves the errors and the exit code but earns a caution
/// (§FS-check.1.3.6.3).
pub(super) fn run_check_with_run_warnings(
    path: &Path,
    path_provided: bool,
    force_require_grounding: bool,
    full: bool,
    ad_hoc_sentence: Option<&str>,
    run_warnings: &mut Vec<Finding>,
) -> Result<CheckRun> {
    // §FS-check.1.3: `--full` cancels `[scan] include` for the walk. It is a
    // per-run flag, never a config key (§DF-check-full-scope.2.5).

    // §FS-check.1.3.6.1: below the root the path is the report scope, so the
    // `Check` selection has `open()` build a run whose walk reads the ordinary roots.

    // §FS-check.1: the flag and `[reference] require_grounding` are one knob; it never
    // turns the key off (§FS-config.3.4.8.3), and reaches every member (§FS-check.3.5).
    let invocation = Invocation::new(path, path_provided, Selection::Check)
        .full(full)
        .require_grounding(force_require_grounding);
    let config = open(&invocation)?;
    // §FS-check.4.10.8: root boundary population has answered
    // everything a failed expansion is allowed to carry. Capture it before a
    // nested member can refuse; questions below the root do not exist yet.
    *run_warnings = public_run_warnings(&config, settled_run_warnings(config.run()));
    // §FS-check.1.3.6.1: a config that still declares `[workspace]` after the
    // member rewrite is the root run's, at the root or below it outside every member,
    // and a path below the root narrows only what that run reports.
    if config.workspace_declared {
        let report_scope = path_report_scope(config.schema(), config.frame(), path, path_provided)?;
        let narrowing = PathReport {
            path,
            path_provided,
            full,
            scope: report_scope.as_ref(),
        };
        let run = run_workspace_check(config, ad_hoc_sentence, &narrowing)?;
        // §FS-check.4.10.8: successful expansion may settle more
        // blocks, so the successful side channel is the complete ordered set.
        *run_warnings = public_run_warnings(&run.config, settled_run_warnings(run.config.run()));
        return Ok(run);
    }
    // §FS-rules.4: ad-hoc grammar/vocabulary refusals happen before scanning.
    let ad_hoc = ad_hoc_sentence
        .map(|sentence| parse_ad_hoc(config.schema(), config.rules(), config.frame(), sentence))
        .transpose()?;

    let (mut findings, mut scan_errors) =
        scan_tree(config.schema(), config.frame(), Some(path), path_provided)?;
    // §FS-check.1.3.6.1: exactly the path, and `None` over the config root.
    let report_scope = path_report_scope(config.schema(), config.frame(), path, path_provided)?;
    // §FS-check.1.3 / §FS-check.3.14: read the out-of-scope tier off the whole
    // `--full` walk first, then narrow the findings back to the configured scope
    // so every other rule reports exactly what a run without the flag reports.

    // §FS-check.1.3.6: and a path scope has no tier of its own — the flag has
    // nothing to cancel there, which was true by accident until §FS-check.1.3.6.1
    // made the walk wider than the report.
    let scope = match report_scope {
        Some(_) => None,
        None => configured_scope(config.schema(), config.frame(), path, path_provided, full)?,
    };
    let mut out_of_scope = out_of_scope_references(
        &findings,
        config.schema(),
        config.frame(),
        &BTreeMap::new(),
        scope.as_ref(),
    );
    out_of_scope.extend(out_of_scope_section_headings(&findings, scope.as_ref()));
    retain_findings_in_scope(&mut findings, config.frame().grammar(), scope.as_ref());
    // §FS-check.1.3.6.1: a file the wider walk could not read is outside the report
    // scope like anything else about it, and it is the exit code as well as a line
    // (§FS-check.2.4) — so it leaves both here, and §FS-check.1.3.6.3 says it below.
    let mut unread_outside_report_scope = Vec::new();
    if let Some(report_scope) = report_scope.as_ref() {
        let (inside, outside): (Vec<_>, Vec<_>) = std::mem::take(&mut scan_errors)
            .into_iter()
            .partition(|(file, _)| report_scope.contains(file));
        scan_errors = inside;
        unread_outside_report_scope = outside;
    }
    // §FS-check.1.3.6.3: the chapter rules are asked whether the **resolution** scope
    // was read in full. The narrowed list would have a run that could not read half
    // its tree assert the absence facts anyway (§FS-rules.4.1).
    let resolution_was_complete = scan_errors.is_empty() && unread_outside_report_scope.is_empty();
    // §AR-checker.1.3: presentation's bytes, rendered before the checker runs.
    let no_workspace = BTreeMap::new();
    // §FS-check.3.5.5: a member's or a standalone project's run leaves the
    // workspace sections out rather than climb to render them.
    let expected = expected(&findings, &config, &no_workspace, WorkspaceBlocks::LeftOut);
    let mut report = check_project(
        config.rules(),
        config.schema(),
        &findings,
        &expected,
        config.frame(),
        &CheckWorkspace::alone(&no_workspace),
    );
    check_chapter_rules(
        &findings,
        config.rules(),
        config.schema(),
        config.frame(),
        resolution_was_complete,
        ad_hoc,
        None,
        &mut report,
    );
    // §FS-check.3.12.2: both direction producers share entry-preserving advice.
    repair_index_citations(
        &findings,
        config.rules(),
        config.schema(),
        &expected,
        config.frame(),
        &mut report,
    );
    // §FS-scan.checks.unread-scan-root: before the narrowing, which drops it where the
    // path does not cover the `grund.toml` holding the row (.scope).
    report
        .warnings
        .extend(unread_root_warnings(&config, &findings));
    let had_scan_errors = append_scan_errors(&mut report, scan_errors);
    // §FS-check.1.3.6.1: every rule has run over the resolution scope, so the report
    // narrows to the path — the second stage, after the rules (§AR-resolver.3.3).
    let narrowing = PathReport {
        path,
        path_provided,
        full,
        scope: report_scope.as_ref(),
    };
    narrowing.narrow(
        &mut report,
        &config,
        unread_outside_report_scope,
        // §FS-check.2.2 / §FS-check.4.5: a walk that read no files, or read them and
        // recognized nothing in them, is almost always a misconfigured scope.
        |report_is_silent| {
            scan_scope_caution(
                &config,
                &findings,
                path,
                path_provided,
                report_is_silent,
                report_scope.as_ref(),
                config.display(),
            )
        },
        // §FS-check.4.3, §FS-check.4.11, §FS-config.4.1: outside `report_is_silent`, so
        // a repository mid-migration keeps the scope caution beside its config pair.
        config_diagnostics(&config, config.display()),
    );
    // §FS-check.3.29.13: the blocks this walk met that no enclosing one lists — one of
    // the report's *errors* since the ramp ended (§FS-check.3.29.14), and located, so it
    // reaches the exit code rather than standing in place of `success` (§FS-check.2.1.3).
    let mut unlisted =
        unlisted_workspace_block_errors(&config, &config, None, &findings.walked_dirs);
    // §FS-check.1.3.6.1: the wider walk met directories the path does not name, and
    // a block out there is not this run's report to make.
    retain_diagnostics_in_report_scope(&mut unlisted, config.frame(), report_scope.as_ref());
    report.errors.extend(unlisted);
    // §FS-check.3.14, after the scope caution above (§FS-check.2.2, §FS-check.4.5):
    // a `--full` run whose *configured* scope read or recognized nothing still earns
    // that caution — the tier says where the citations are, the config was not told.
    report.errors.extend(out_of_scope);
    sort_diagnostics(&mut report.errors);
    // §FS-check.1.4.1: the flag is the gate of the tier and of its caution.
    if full {
        report.armed.arm(Switch::Full);
    }

    let run = CheckRun {
        projects: vec![config.root.clone()],
        config,
        report,
        had_scan_errors,
    };
    *run_warnings = public_run_warnings(&run.config, settled_run_warnings(run.config.run()));
    Ok(run)
}

/// The workspace arm of `grund check`: every member checked in turn, its findings
/// anchored on the workspace root's config, with the per-project cautions and the
/// per-config warnings folded into one report.
///
/// Why the root is skipped in the per-project warning loop at the end: with
/// `include_root = true` the root is itself a `projects` entry, so warning once per
/// project would name the root's directory twice.
///
/// §FS-check.4.9: the announcement of every namespace the run did not read is one
/// located warning each, on stdout, at the `optional_members` entry that made the
/// skip legal. It is what buys the green exit — the exit code says nothing about
/// coverage here, so the report must — and it withholds `success` like any other
/// warning (§FS-check.2.1.3). §FS-workspace.2.2.10: the block those entries left with no
/// project at all is not an error either, but a run with nothing to read still says
/// so beside them (§FS-check.2.2). The LSP builds both from
/// `check_workspace_context`, which is the same decision made from the same place
/// (§FS-lsp.4.1).
///
/// §FS-check.1.3.6.1: `report_scope` is the path of a `check <path>` below
/// the root and outside every member, `None` at the root. Every project is still
/// checked completely, and the report then narrows to the path through the one
/// narrowing the single-project arm goes through ([`PathReport::narrow`]): a finding
/// anchored outside it is dropped unless it names a site inside, a file the run could
/// not read out there leaves the errors for a caution (§FS-check.1.3.6.3), a path
/// that holds nothing to scan says so (§FS-check.2.2), and a `--full` the path made
/// redundant says that (§FS-check.1.3.7).
fn run_workspace_check(
    root_config: Config,
    ad_hoc_sentence: Option<&str>,
    narrowing: &PathReport<'_>,
) -> Result<CheckRun> {
    let (full, report_scope) = (narrowing.full, narrowing.scope);
    let mut root_config = ProjectRecords::of(root_config);
    // §FS-check.3.5: `--require-grounding` reaches every member's run through the
    // invocation it was built from (`Run::members`), as `--full` does.
    let mut projects = load_workspace_projects(&mut root_config)?;
    // §FS-check.1.3.8: `include` is a per-project statement, so each project's
    // walk is widened past its own and tiered against its own configured scope.
    let scopes = projects
        .iter()
        .map(|project| {
            configured_scope(
                project.config.schema(),
                project.config.frame(),
                &project.config.root,
                true,
                full,
            )
        })
        .collect::<Result<Vec<_>>>()?;
    let mut out_of_scope = workspace_out_of_scope_references(&projects, &scopes);
    out_of_scope.extend(workspace_out_of_scope_section_headings(&projects, &scopes));
    for (project, scope) in projects.iter_mut().zip(&scopes) {
        let grammar = project.config.frame().grammar();
        retain_findings_in_scope(&mut project.findings, grammar, scope.as_ref());
    }

    let rules_complete = projects
        .iter()
        .all(|project| project.scan_errors.is_empty());
    let workspace = projects
        .iter()
        .map(|project| {
            (
                project.alias.clone(),
                WorkspaceCheckTarget::of(
                    &project.findings,
                    project.config.schema(),
                    project.config.frame(),
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let ad_hoc = ad_hoc_sentence
        .map(|sentence| {
            parse_ad_hoc_with_workspace(
                root_config.schema(),
                root_config.rules(),
                root_config.frame(),
                sentence,
                &workspace,
            )
        })
        .transpose()?;
    // §FS-check.4.12: a qualified citation's `resolve` policy is its target's rule.
    let check_workspace = CheckWorkspace::new(
        &workspace,
        projects
            .iter()
            .map(|project| (project.alias.as_str(), project.config.rules())),
    );
    let mut report = CheckReport::default();
    let mut had_scan_errors = false;
    let mut unread_outside_report_scope = Vec::new();
    let listed = workspace_block_projects(&root_config, &projects);
    let blocks = listed
        .as_deref()
        .map_or(WorkspaceBlocks::LeftOut, WorkspaceBlocks::Compared);
    for project in &projects {
        let expected = expected(&project.findings, &project.config, &workspace, blocks);
        // §FS-workspace.8.1: the report is rendered from the workspace root,
        // so a path a member's message names is spelled from there too.
        let frame = project
            .config
            .frame()
            .displayed_by(root_config.display())
            .checked_as(Some(&project.alias));
        let mut project_report = check_project(
            project.config.rules(),
            project.config.schema(),
            &project.findings,
            &expected,
            frame,
            &check_workspace,
        );
        check_chapter_rules(
            &project.findings,
            project.config.rules(),
            project.config.schema(),
            frame,
            rules_complete,
            ad_hoc.clone(),
            Some((&project.alias, &workspace)),
            &mut project_report,
        );
        // §FS-check.3.12.2: repair in this project's config vocabulary before merging.
        repair_index_citations(
            &project.findings,
            project.config.rules(),
            project.config.schema(),
            &expected,
            frame,
            &mut project_report,
        );
        // §FS-scan.checks.unread-scan-root.scope: each project in its own `grund.toml`.
        project_report
            .warnings
            .extend(unread_root_warnings(&project.config, &project.findings));
        let project_has_findings =
            !project_report.errors.is_empty() || !project_report.warnings.is_empty();
        report.errors.append(&mut project_report.errors);
        report.warnings.append(&mut project_report.warnings);
        report.suggestions.append(&mut project_report.suggestions);
        report.verified_blocks |= project_report.verified_blocks;
        // §FS-check.1.4.1: a check any project armed is armed in the run.
        report.armed.absorb(&mut project_report.armed);
        // §FS-check.1.3.6.3: as the single-project arm partitions them.
        let (inside, outside): (Vec<_>, Vec<_>) = project
            .scan_errors
            .iter()
            .cloned()
            .partition(|(file, _)| report_scope.is_none_or(|scope| scope.contains(file)));
        unread_outside_report_scope.extend(outside);
        had_scan_errors |= append_scan_errors(&mut report, inside);
        // §FS-check.2.2 / §FS-check.4.5.3 / §FS-workspace.5: the single-project cautions,
        // asked per project — one member's empty scope says nothing about another's.
        // §FS-check.1.3.6.1: a narrowed run asks the path instead, below.
        if report_scope.is_none() {
            report.warnings.extend(scan_scope_caution(
                &project.config,
                &project.findings,
                &project.config.root,
                true,
                project.scan_errors.is_empty() && !project_has_findings,
                None,
                // §FS-workspace.5.3, §FS-config.3.6: the root is named from the report base.
                root_config.display(),
            ));
        }
    }
    // §FS-workspace.2.2, §FS-check.4.9: the caution and the announcements — see
    // this function's docs.
    report.warnings.extend(absent_only_workspace_caution(
        &root_config,
        projects.is_empty(),
    ));
    report.warnings.extend(absent_optional_member_warnings(
        &root_config,
        report.verified_blocks,
    ));
    arm_optional_members(&root_config, &mut report.armed);
    if full {
        report.armed.arm(Switch::Full);
    }
    // §FS-check.3.29.11: per project — the candidates are what *that* walk reached, and
    // the absorbing namespace is its own. Rendered against the workspace root like
    // every other message here (§FS-workspace.8.1).
    for project in &projects {
        report.errors.extend(unlisted_workspace_block_errors(
            &project.config,
            &root_config,
            Some(&project.alias),
            &project.findings.walked_dirs,
        ));
    }
    report.errors.extend(out_of_scope);
    // §FS-check.4.3, §FS-check.4.11.3: the root's config and every member's, each
    // named where that project loaded it — one config, not one scope, and a
    // workspace may mix the two discovery forms (§FS-workspace.2).
    let mut config_warnings =
        config_diagnostics(&root_config, root_config.display()).collect::<Vec<_>>();
    for project in &projects {
        if project.config.root != root_config.root {
            config_warnings.extend(config_diagnostics(&project.config, root_config.display()));
        }
    }
    // §FS-check.1.3.6.1: the root run's report, narrowed to the path as the
    // single-project arm narrows it (§AR-resolver.3.3).
    narrowing.narrow(
        &mut report,
        &root_config,
        unread_outside_report_scope,
        // §FS-check.2.2: asked of the path, which no project's walk read a file under.
        |report_is_silent| {
            workspace_path_caution(
                &root_config,
                projects.iter().map(|project| &project.findings),
                narrowing.path,
                narrowing.path_provided,
                report_is_silent,
                report_scope,
            )
        },
        config_warnings,
    );
    sort_diagnostics(&mut report.errors);
    sort_diagnostics(&mut report.warnings);
    sort_diagnostics(&mut report.suggestions);

    Ok(CheckRun {
        config: root_config.into_facade(),
        report,
        had_scan_errors,
        projects: projects
            .iter()
            .map(|project| project.config.root.clone())
            .collect(),
    })
}

/// §FS-check.3.5.5: the projects a workspace run renders its blocks' workspace
/// sections from, or `None` where it leaves them out — a run at a nested group's
/// root, whose projects carry the group's alias path, and one with an optional
/// member absent anywhere in the tree, the fact `init` keeps the member list on
/// (§FS-init.2.3.4.15.8). The list is the projects already loaded, so the
/// comparison adds no walk.
pub(super) fn workspace_block_projects(
    root_config: &Config,
    projects: &[WorkspaceProject],
) -> Option<Vec<ListedProject>> {
    let outermost = projects
        .iter()
        .all(|project| project.config.workspace_scope_path.is_empty());
    (outermost && absent_optional_members(root_config).is_empty()).then(|| {
        listed_projects(projects.iter().map(|project| {
            (
                project.alias.clone(),
                project.config.root.clone(),
                project.config.project_description.clone(),
            )
        }))
    })
}

fn append_scan_errors(
    report: &mut CheckReport,
    scan_errors: impl IntoIterator<Item = (PathBuf, String)>,
) -> bool {
    let mut had_scan_errors = false;
    for (file, message) in scan_errors {
        had_scan_errors = true;
        // A file that could not be read mid-walk is reported as a CLI-shaped
        // `error: <path>: <reason>` finding (§FS-check.2.4): the walk continued,
        // the findings below are real, but the view of the tree was incomplete.
        report.errors.push(Diagnostic {
            code: Code::IO,
            path: Some(file),
            line: None,
            column: None,
            message,
            sites: Vec::new(),
            authority: Vec::new(),
        });
    }
    had_scan_errors
}
