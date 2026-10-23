//! The report an editor snapshot carries (§FS-lsp.4.1, §AR-lsp.5.1): `grund check`'s
//! decision made for a surface that has no CLI, from the same checker functions, so
//! an editor and a terminal over one tree report one set of diagnostics.
//!
//! Split out of `lsp_snapshot.rs`, which walks the loaded context for navigation
//! records; this file holds what the two scopes of §AR-lsp.5.1.1 do to the report.

use anyhow::Result;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::path_report::unread_resolution_source_cautions;
use super::report::complete_report;
use super::run::workspace_block_projects;
use super::scope_cautions::{scan_scope_caution, workspace_path_caution};
use super::unread_roots::unread_root_warnings;
use crate::checker::{
    CheckWorkspace, ScanScope, check_chapter_rules, check_project, path_report_scope,
    repair_index_citations, retain_diagnostics_in_report_scope,
};
use crate::config::Config;
use crate::model::{CheckReport, Code, Diagnostic};
use crate::resolver::{WorkspaceCheckTarget, WorkspaceContext};
use crate::workspace::{
    absent_only_workspace_caution, absent_optional_member_warnings, unlisted_workspace_block_errors,
};
use crate::writers::{WorkspaceBlocks, expected};

/// §AR-lsp.5.1.1: a snapshot anchored below its config root keeps the two scopes of
/// §FS-check.1.3.6.1, the way `run_check_with_run_warnings` does for `grund check
/// <path>`. The walk was widened to the project's ordinary roots when `open()`
/// built the run under the `Editor` selection, and the report scope returned here
/// is what the caller narrows the report to once every rule has run. `None`, and
/// an unchanged walk, for a root anchor.
///
/// A config that still declares `[workspace]` here is the root's, anchored at a
/// folder below it and outside every member. That run loads every project at its own
/// root (`load_resolved_workspace_context`), so its walk needs no widening, and the
/// anchor bounds only its report, over the root's frame, as `run_workspace_check`
/// bounds `grund check <path>`'s.
pub(super) fn editor_report_scope(
    config: &Config,
    path: &Path,
    path_provided: bool,
) -> Result<Option<ScanScope>> {
    path_report_scope(config.schema(), config.frame(), path, path_provided)
}

/// §AR-lsp.5.1.1: the snapshot's report — every rule run over the resolution scope,
/// then narrowed to the anchor by `grund check <path>`'s own filter, with its
/// exemptions (§FS-check.1.3.6.1), and the two cautions that run asks of the report
/// scope rather than the walk. A no-op narrowing for a root anchor.
pub(super) fn editor_report(
    context: &WorkspaceContext,
    anchor: &Path,
    report_scope: Option<&ScanScope>,
) -> CheckReport {
    check_workspace_context(context, anchor, report_scope)
}

/// §FS-lsp.4.1: the report the editor shows, decided here so that an editor and a
/// terminal over one tree say the same thing — this is `grund check`'s workspace
/// arm (`run_workspace_check`) for a surface that has no CLI.
///
/// §FS-check.4.9.4: the announcement of every namespace the run did not read is a
/// **located** report finding, so it is one of the diagnostics the editor must
/// mirror. It belongs to the run rather than to a project, which is why it is read
/// off the render config outside the loop below and survives a block whose every
/// project was the absent one — and why that config is cloned after the workspace
/// walk (`load_resolved_workspace_context`). §FS-check.2.2: the caution for that
/// same block rides beside it.
///
/// §AR-lsp.5.1.1: `report_scope` is `Some` for an anchor below its config root. On
/// the single-project arm (`widen_for_path_anchor`) each step `run_check_with_run_warnings`
/// takes after its rules is taken here in its order: the narrowing, then the scope
/// caution asked of the anchor, then the files the wider walk could not read outside
/// it. Below a workspace root and outside every member each project is narrowed the
/// same way, and the scope caution is asked once, of the whole report, before those
/// files, as `run_workspace_check` asks it.
fn check_workspace_context(
    context: &WorkspaceContext,
    anchor: &Path,
    report_scope: Option<&ScanScope>,
) -> CheckReport {
    let workspace = context
        .projects
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
    // §FS-check.4.12: a qualified citation's `resolve` policy is its target's rule.
    let check_workspace = CheckWorkspace::new(
        &workspace,
        context
            .projects
            .iter()
            .map(|project| (project.alias.as_str(), project.config.rules())),
    );
    let mut report = CheckReport::default();
    // §FS-check.1.3.6.1: a workspace anchor below its root asks the scope caution once,
    // after the loop, and holds the unread files back for after it (§FS-check.2.2.3.1).
    let narrowed_workspace = context.workspace_loaded && report_scope.is_some();
    let mut unread_outside_report_scope = Vec::new();
    let rules_complete = context
        .projects
        .iter()
        .all(|project| project.scan_errors.is_empty());
    // §FS-check.3.5.5: the workspace sections are compared where `grund check`
    // compares them, from the same projects.
    let listed = context
        .workspace_loaded
        .then(|| workspace_block_projects(context.render_config(), &context.projects))
        .flatten();
    let blocks = listed
        .as_deref()
        .map_or(WorkspaceBlocks::LeftOut, WorkspaceBlocks::Compared);
    for project in &context.projects {
        let config = project.config.clone();
        let no_workspace = BTreeMap::new();
        let expected = expected(
            &project.findings,
            &config,
            if context.workspace_loaded {
                &workspace
            } else {
                &no_workspace
            },
            blocks,
        );
        let mut project_report = if context.workspace_loaded {
            // §AR-checker.1.3: presentation's bytes, rendered before the checker runs.
            // §FS-workspace.8.1: paths inside a message are spelled from the
            // render root, like the anchors beside them.
            let frame = config
                .frame()
                .displayed_by(context.render_config().frame().display)
                .checked_as(Some(&project.alias));
            check_project(
                config.rules(),
                config.schema(),
                &project.findings,
                &expected,
                frame,
                &check_workspace,
            )
        } else {
            check_project(
                config.rules(),
                config.schema(),
                &project.findings,
                &expected,
                config.frame(),
                &CheckWorkspace::alone(&no_workspace),
            )
        };
        check_chapter_rules(
            &project.findings,
            config.rules(),
            config.schema(),
            config.frame(),
            rules_complete,
            None,
            context
                .workspace_loaded
                .then_some((project.alias.as_str(), &workspace)),
            &mut project_report,
        );
        // §FS-check.3.12.2: the editor uses the same repair as both CLI scopes.
        repair_index_citations(
            &project.findings,
            config.rules(),
            config.schema(),
            &expected,
            config.frame(),
            &mut project_report,
        );
        // §FS-lsp.4.1: `grund check`'s warning from its own function
        // (§FS-scan.checks.unread-scan-root).
        project_report
            .warnings
            .extend(unread_root_warnings(&config, &project.findings));
        // §FS-check.1.3.6.1: a file the wider walk could not read outside the anchor
        // leaves the errors; §FS-check.1.3.6.3 says it once the report is narrowed.
        let (inside, unread_outside): (Vec<_>, Vec<_>) = project
            .scan_errors
            .iter()
            .cloned()
            .partition(|(file, _)| report_scope.is_none_or(|scope| scope.contains(file)));
        append_lsp_scan_errors(&mut project_report, inside);
        for channel in [&mut project_report.errors, &mut project_report.warnings] {
            retain_diagnostics_in_report_scope(
                channel,
                context.render_config().frame(),
                report_scope,
            );
        }
        let report_is_silent =
            project_report.errors.is_empty() && project_report.warnings.is_empty();
        report.errors.append(&mut project_report.errors);
        report.warnings.append(&mut project_report.warnings);
        report.verified_blocks |= project_report.verified_blocks;
        if narrowed_workspace {
            unread_outside_report_scope.extend(unread_outside);
            continue;
        }
        // §FS-lsp.4.1: `grund check`'s decision from its own function (§FS-check.2.2,
        // §FS-check.4.5), asked of the anchor and its report scope (§AR-lsp.5.1.1).
        let path = match report_scope {
            Some(_) => anchor,
            // `grund-lsp` anchors at the project root and distributes the
            // diagnostics per file itself (§FS-lsp.4.1).
            None => config.root.as_path(),
        };
        report.warnings.extend(scan_scope_caution(
            &config,
            &project.findings,
            path,
            true,
            report_is_silent,
            report_scope,
            context.render_config().display(),
        ));
        // §FS-check.1.3.6.3, §AR-lsp.5.1.1: after the silence flag, as in `grund check`
        // (§FS-check.2.2.3.1).
        report
            .warnings
            .extend(unread_resolution_source_cautions(unread_outside));
    }
    // §FS-check.3.29.13, §FS-check.3.29.11: per project, the blocks that walk met that
    // no enclosing one lists — located report errors, the decision `run_workspace_check`
    // makes, so an editor and a terminal place one diagnostic at one line (§FS-lsp.4.1).
    for project in &context.projects {
        let mut unlisted = unlisted_workspace_block_errors(
            &project.config,
            context.render_config(),
            context.workspace_loaded.then_some(project.alias.as_str()),
            &project.findings.walked_dirs,
        );
        // §FS-check.1.3.6.1: a block the wider walk met outside the anchor is not this
        // report's to make.
        retain_diagnostics_in_report_scope(
            &mut unlisted,
            context.render_config().frame(),
            report_scope,
        );
        report.errors.extend(unlisted);
    }
    // §FS-check.4.9, §FS-check.2.2: the announcements and the caution — see this
    // function's docs. An announcement is located, so the anchor narrows it as `grund
    // check <path>` does (§FS-check.1.3.6.1).
    let mut announcements =
        absent_optional_member_warnings(context.render_config(), report.verified_blocks);
    retain_diagnostics_in_report_scope(
        &mut announcements,
        context.render_config().frame(),
        report_scope,
    );
    report.warnings.extend(announcements);
    report.warnings.extend(absent_only_workspace_caution(
        context.render_config(),
        context.projects.is_empty(),
    ));
    if narrowed_workspace {
        let report_is_silent = report.errors.is_empty() && report.warnings.is_empty();
        report.warnings.extend(workspace_path_caution(
            context.render_config(),
            context.projects.iter().map(|project| &project.findings),
            anchor,
            true,
            report_is_silent,
            report_scope,
        ));
        report.warnings.extend(unread_resolution_source_cautions(
            unread_outside_report_scope,
        ));
    }
    // §FS-errors.4.1: the editor completes its channels by the same rule as disk checks.
    complete_report(&mut report);
    report
}

/// §FS-lsp.1.1.3, §FS-check.3.29.15: the run's warning channel as the editor sees it —
/// the three `[workspace]` cautions the run settled, without the unlisted block.
///
/// The five walking surfaces that have no report of their own keep that finding on this
/// channel, which is why the context still settles it (§FS-check.3.29.9). The editor
/// takes it off `report` instead, located and as an error (§FS-check.3.29.13), and a copy
/// left here would publish a second, warning-severity squiggle on the same line — the one
/// way the flip can go wrong invisibly, so the drop is a named step rather than a filter
/// buried in the call.
pub(super) fn editor_run_warnings(context: &WorkspaceContext) -> Vec<Diagnostic> {
    context
        .run_warnings
        .iter()
        .filter(|warning| warning.code != Code::UNLISTED_WORKSPACE_BLOCK)
        .cloned()
        .collect()
}

fn append_lsp_scan_errors(
    report: &mut CheckReport,
    scan_errors: impl IntoIterator<Item = (PathBuf, String)>,
) {
    for (file, message) in scan_errors {
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
}
