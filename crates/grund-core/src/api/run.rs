//! One `check` run, single-project and workspace (§AR-system.2.9): resolve the
//! config, scan, check, and fold in the cautions and the config findings the run
//! earns (§FS-check.1, §FS-check.2, §FS-workspace.5).
//!
//! It sat in the deprecated `check` adapter while `check_with_opts` — the whole
//! published `check` — called into it, which had the api reading a renderer
//! (§AR-system.4, §AR-system.2.9). Only `command_check` was the renderer; the run
//! is what every surface shares, and the api is the lowest component that holds
//! it, so `compat/check.rs` now reads it downward and the deprecated path and the
//! embedding surface cannot report different things about one tree.

use anyhow::Result;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::config_findings::config_diagnostics;
use super::scope_cautions::{full_scope_ignored_warning, scan_scope_caution};
use crate::checker::{
    check_chapter_rules, check_findings, check_with_workspace, configured_scope,
    out_of_scope_references, out_of_scope_section_headings, parse_ad_hoc, retain_findings_in_scope,
    sort_diagnostics, workspace_out_of_scope_references, workspace_out_of_scope_section_headings,
};
use crate::config::Config;
use crate::model::{CheckReport, Diagnostic};
use crate::resolver::{WorkspaceCheckTarget, load_workspace_projects};
use crate::scanner::scan_tree;
use crate::workspace::{
    absent_only_workspace_caution, absent_optional_member_warnings, resolve_workspace_config,
    scope_is_config_root, unlisted_workspace_block_warnings,
};

pub(crate) struct CheckRun {
    pub(crate) config: Config,
    pub(crate) report: CheckReport,
    pub(crate) had_scan_errors: bool,
}

/// One `grund check` run over `path`: resolve the config, scan, check, and fold in
/// the cautions and warnings the run earns for its scope and its config files.
///
/// Why the scope caution is only a warning: it never changes the exit code. The
/// agent-entrypoint check (§FS-check.3.5) runs even when no source file is scanned,
/// so a missing or stale `AGENTS.md` block still reports normally and suppresses
/// both cautions.
pub(crate) fn run_check(
    path: &Path,
    path_provided: bool,
    force_require_grounding: bool,
    full: bool,
    ad_hoc_sentence: Option<&str>,
) -> Result<CheckRun> {
    let mut config = resolve_workspace_config(path)?;
    // §FS-rules.4: ad-hoc grammar/vocabulary refusals happen before scanning.
    let ad_hoc = ad_hoc_sentence
        .map(|sentence| parse_ad_hoc(&config, sentence))
        .transpose()?;
    // §FS-check.1.3: `--full` cancels `[scan] include` for the walk. It is a
    // per-run flag, never a config key (§DF-check-full-scope.2.5).
    config.scan_full = full;
    // §FS-check.1: the flag and `[reference] require_grounding` are one knob, so
    // it sets the same global default — it never turns the key off, and a
    // `[[kinds]]` row that says `false` stays exempt under it (§FS-config.3.4.8.3).
    if force_require_grounding {
        config.require_grounding = true;
    }
    if config.workspace_declared && scope_is_config_root(&config, path, path_provided) {
        return run_workspace_check(config, force_require_grounding, full, ad_hoc);
    }

    let (mut findings, scan_errors) = scan_tree(&config, Some(path), path_provided)?;
    // §FS-check.1.3 / §FS-check.3.14: read the out-of-scope tier off the whole
    // `--full` walk first, then narrow the findings back to the configured scope
    // so every other rule reports exactly what a run without the flag reports.
    let scope = configured_scope(&config, path, path_provided, full)?;
    let mut out_of_scope =
        out_of_scope_references(&findings, &config, &BTreeMap::new(), scope.as_ref());
    out_of_scope.extend(out_of_scope_section_headings(&findings, scope.as_ref()));
    retain_findings_in_scope(&mut findings, scope.as_ref());
    let mut report = check_findings(&findings, &config);
    check_chapter_rules(
        &findings,
        &config,
        scan_errors.is_empty(),
        ad_hoc,
        None,
        &mut report,
    );
    let had_scan_errors = append_scan_errors(&mut report, scan_errors);
    // §FS-check.2.2 / §FS-check.4.5: a walk that read no files, or read them and
    // recognized nothing in them, is almost always a misconfigured scope rather
    // than a clean repo — say so on stderr instead of exiting 0 in silence.
    let report_is_silent = report.errors.is_empty() && report.warnings.is_empty();
    report.warnings.extend(scan_scope_caution(
        &config,
        &findings,
        path,
        path_provided,
        report_is_silent,
    ));
    // What the config itself carries (§FS-check.4.3, §FS-check.4.11,
    // §FS-config.4.1), outside `report_is_silent`: a repository mid-migration
    // must not lose the scope caution just because it also has a config pair.
    report.warnings.extend(config_diagnostics(&config));
    // §FS-check.1.3.7, also after the scope caution: `--full` cancels `[scan] include`,
    // and an explicit path other than the config root already bypasses that key — so
    // the flag changed nothing, and the caller who typed it wanted a wider search.
    report.warnings.extend(full_scope_ignored_warning(
        &config,
        path,
        path_provided,
        full,
    ));
    // §FS-check.4.8.13: the blocks this walk met that no enclosing one lists. A report
    // warning, not a line printed past it: that is what stands it in place of
    // `success` (§FS-check.2.1.3) and makes §DF-unlisted-workspace-block.2.1's ramp work.
    report.warnings.extend(unlisted_workspace_block_warnings(
        &config,
        &config,
        None,
        &findings.walked_dirs,
    ));
    // §FS-check.3.14, after the scope caution above (§FS-check.2.2, §FS-check.4.5):
    // a `--full` run whose *configured* scope read or recognized nothing still earns
    // that caution — the tier says where the citations are, the config was not told.
    report.errors.extend(out_of_scope);
    sort_diagnostics(&mut report.errors);

    Ok(CheckRun {
        config,
        report,
        had_scan_errors,
    })
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
fn run_workspace_check(
    mut root_config: Config,
    force_require_grounding: bool,
    full: bool,
    ad_hoc: Option<crate::rules::sentence::ParsedRule>,
) -> Result<CheckRun> {
    let mut projects = load_workspace_projects(&mut root_config)?;
    // §FS-check.3.5: `--require-grounding` propagates to every member's
    // config. The flag only affects checking, not scanning, so applying it
    // after the load is equivalent to setting it before.
    if force_require_grounding {
        for project in &mut projects {
            project.config.require_grounding = true;
        }
    }
    // §FS-check.1.3.8: `include` is a per-project statement, so each project's
    // walk is widened past its own and tiered against its own configured scope.
    let scopes = projects
        .iter()
        .map(|project| configured_scope(&project.config, &project.config.root, true, full))
        .collect::<Result<Vec<_>>>()?;
    let mut out_of_scope = workspace_out_of_scope_references(&projects, &scopes);
    out_of_scope.extend(workspace_out_of_scope_section_headings(&projects, &scopes));
    for (project, scope) in projects.iter_mut().zip(&scopes) {
        retain_findings_in_scope(&mut project.findings, scope.as_ref());
    }

    let rules_complete = projects
        .iter()
        .all(|project| project.scan_errors.is_empty());
    let workspace = projects
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
    let mut had_scan_errors = false;
    for project in &projects {
        let mut project_report = check_with_workspace(
            &project.findings,
            &project.config,
            // §FS-workspace.8.1: the report is rendered from the workspace root,
            // so a path a member's message names is spelled from there too.
            &root_config,
            Some(&project.alias),
            &workspace,
        );
        check_chapter_rules(
            &project.findings,
            &project.config,
            rules_complete,
            ad_hoc.clone(),
            Some((&project.alias, &workspace)),
            &mut project_report,
        );
        let project_has_findings =
            !project_report.errors.is_empty() || !project_report.warnings.is_empty();
        report.errors.append(&mut project_report.errors);
        report.warnings.append(&mut project_report.warnings);
        report.suggestions.append(&mut project_report.suggestions);
        had_scan_errors |= append_scan_errors(&mut report, project.scan_errors.iter().cloned());
        // §FS-check.2.2 / §FS-check.4.5.3 / §FS-workspace.5: the same two cautions as
        // the single-project path, asked per project — one member's empty scope or
        // grammar mismatch says nothing about another's, and each names its own.
        report.warnings.extend(scan_scope_caution(
            &project.config,
            &project.findings,
            &project.config.root,
            true,
            project.scan_errors.is_empty() && !project_has_findings,
        ));
    }
    // §FS-workspace.2.2, §FS-check.4.9: the caution and the announcements — see
    // this function's docs.
    report.warnings.extend(absent_only_workspace_caution(
        &root_config,
        projects.is_empty(),
    ));
    report
        .warnings
        .extend(absent_optional_member_warnings(&root_config));
    // §FS-check.4.3, §FS-check.4.11.3: the root's config and every member's, each
    // named where that project loaded it — one config, not one scope, and a
    // workspace may mix the two discovery forms (§FS-workspace.2).
    report.warnings.extend(config_diagnostics(&root_config));
    for project in &projects {
        if project.config.root != root_config.root {
            report.warnings.extend(config_diagnostics(&project.config));
        }
    }
    // §FS-check.4.8.11: per project — the candidates are what *that* walk reached, and
    // the absorbing namespace is its own. Rendered against the workspace root like
    // every other message here (§FS-workspace.8.1).
    for project in &projects {
        report.warnings.extend(unlisted_workspace_block_warnings(
            &project.config,
            &root_config,
            Some(&project.alias),
            &project.findings.walked_dirs,
        ));
    }
    report.errors.extend(out_of_scope);
    sort_diagnostics(&mut report.errors);
    sort_diagnostics(&mut report.warnings);
    sort_diagnostics(&mut report.suggestions);

    Ok(CheckRun {
        config: root_config,
        report,
        had_scan_errors,
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
            code: "io",
            path: Some(file),
            line: None,
            column: None,
            message,
            sites: Vec::new(),
        });
    }
    had_scan_errors
}
