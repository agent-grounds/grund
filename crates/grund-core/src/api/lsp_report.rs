//! The report an editor snapshot carries (§FS-lsp.4.1, §AR-lsp.5.1): `grund check`'s
//! decision made for a surface that has no CLI, from the same checker functions, so
//! an editor and a terminal over one tree report one set of diagnostics.
//!
//! Split out of `lsp_snapshot.rs`, which walks the loaded context for navigation
//! records; this file holds what the two scopes of §AR-lsp.5.1.1 do to the report.

use anyhow::Result;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::run::unread_resolution_source_cautions;
use super::scope_cautions::scan_scope_caution;
use crate::checker::{
    ScanScope, check_chapter_rules, check_with_workspace_and_overlays, path_report_scope,
    retain_diagnostics_in_report_scope, sort_diagnostics,
};
use crate::config::Config;
use crate::model::{CheckReport, Diagnostic, TextOverlays};
use crate::resolver::{WorkspaceCheckTarget, WorkspaceContext};
use crate::workspace::{
    absent_only_workspace_caution, absent_optional_member_warnings, scope_is_config_root,
    unlisted_workspace_block_errors,
};

/// §AR-lsp.5.1.1: a snapshot anchored below its config root keeps the two scopes of
/// §FS-check.1.3.6.1, the way `run_check_with_run_warnings` does for `grund check
/// <path>`. The walk is widened to the project's ordinary roots on the config, for
/// the reason that run sets it there — the scanner asks four frames down — and the
/// returned report scope is what the caller narrows the report to once every rule
/// has run. `None`, and an unchanged walk, for a root anchor.
///
/// Only the single-project arm: a config that still declares `[workspace]` here is
/// a non-member folder of the workspace root, which loads every project at its own
/// root and is not narrowed by the anchor at all (`load_resolved_workspace_context`).
pub(super) fn widen_for_path_anchor(
    config: &mut Config,
    path: &Path,
    path_provided: bool,
) -> Result<Option<ScanScope>> {
    if config.workspace_declared || scope_is_config_root(config, path, path_provided) {
        return Ok(None);
    }
    config.set_scan_resolution_wide(true);
    path_report_scope(config, path, path_provided)
}

/// §AR-lsp.5.1.1: the snapshot's report — every rule run over the resolution scope,
/// then narrowed to the anchor by `grund check <path>`'s own filter, with its
/// exemptions (§FS-check.1.3.6.1), and the two cautions that run asks of the report
/// scope rather than the walk. A no-op narrowing for a root anchor.
pub(super) fn editor_report(
    context: &WorkspaceContext,
    overlays: &TextOverlays,
    anchor: &Path,
    report_scope: Option<&ScanScope>,
) -> CheckReport {
    check_workspace_context(context, false, overlays, anchor, report_scope)
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
/// §AR-lsp.5.1.1: `report_scope` is `Some` only on the single-project arm
/// (`widen_for_path_anchor`), where each step `run_check_with_run_warnings` takes after
/// its rules is taken here in its order: the narrowing, then the scope caution asked
/// of the anchor, then the files the wider walk could not read outside it.
fn check_workspace_context(
    context: &WorkspaceContext,
    force_require_grounding: bool,
    overlays: &TextOverlays,
    anchor: &Path,
    report_scope: Option<&ScanScope>,
) -> CheckReport {
    let workspace = context
        .projects
        .iter()
        .map(|project| {
            (
                project.alias.clone(),
                WorkspaceCheckTarget {
                    findings: &project.findings,
                    config: &project.config,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut report = CheckReport::default();
    let rules_complete = context
        .projects
        .iter()
        .all(|project| project.scan_errors.is_empty());
    for project in &context.projects {
        let mut config = project.config.clone();
        // §FS-check.1: the same global default the key sets, per member — an
        // explicit `false` on a `[[kinds]]` row still wins (§FS-config.3.4.8.3).
        if force_require_grounding {
            config.force_require_grounding();
        }
        let mut project_report = if context.workspace_loaded {
            check_with_workspace_and_overlays(
                &project.findings,
                &config,
                // §FS-workspace.8.1: paths inside a message are spelled from the
                // render root, like the anchors beside them.
                context.render_config(),
                Some(&project.alias),
                &workspace,
                overlays,
            )
        } else {
            check_with_workspace_and_overlays(
                &project.findings,
                &config,
                &config,
                None,
                &BTreeMap::new(),
                overlays,
            )
        };
        check_chapter_rules(
            &project.findings,
            &config,
            rules_complete,
            None,
            context
                .workspace_loaded
                .then_some((project.alias.as_str(), &workspace)),
            &mut project_report,
        );
        // §FS-check.1.3.6.1: a file the wider walk could not read outside the anchor
        // leaves the errors; §FS-check.1.3.6.3 says it once the report is narrowed.
        let (inside, unread_outside): (Vec<_>, Vec<_>) = project
            .scan_errors
            .iter()
            .cloned()
            .partition(|(file, _)| report_scope.is_none_or(|scope| scope.contains(file)));
        append_lsp_scan_errors(&mut project_report, inside);
        for channel in [&mut project_report.errors, &mut project_report.warnings] {
            retain_diagnostics_in_report_scope(channel, context.render_config(), report_scope);
        }
        let report_is_silent =
            project_report.errors.is_empty() && project_report.warnings.is_empty();
        report.errors.append(&mut project_report.errors);
        report.warnings.append(&mut project_report.warnings);
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
        retain_diagnostics_in_report_scope(&mut unlisted, context.render_config(), report_scope);
        report.errors.extend(unlisted);
    }
    // §FS-check.4.9, §FS-check.2.2: the announcements and the caution — see this
    // function's docs.
    report
        .warnings
        .extend(absent_optional_member_warnings(context.render_config()));
    report.warnings.extend(absent_only_workspace_caution(
        context.render_config(),
        context.projects.is_empty(),
    ));
    sort_diagnostics(&mut report.errors);
    sort_diagnostics(&mut report.warnings);
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
        .filter(|warning| warning.code != "unlisted-workspace-block")
        .cloned()
        .collect()
}

fn append_lsp_scan_errors(
    report: &mut CheckReport,
    scan_errors: impl IntoIterator<Item = (PathBuf, String)>,
) {
    for (file, message) in scan_errors {
        report.errors.push(Diagnostic {
            code: "io",
            path: Some(file),
            line: None,
            column: None,
            message,
            sites: Vec::new(),
            authority: Vec::new(),
        });
    }
}
