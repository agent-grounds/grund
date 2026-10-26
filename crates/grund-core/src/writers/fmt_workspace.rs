use anyhow::Result;

use super::fmt_error::{FmtConflictAbort, FmtScanAbort};
use super::fmt_section_lock::{lock_targets, section_lock_changes};
use super::fmt_settle::settle_project_headings;
use super::fmt_tree::{FmtRunOpts, FmtTreeOutcome, auto_cross_refs_for_scope, plan_tree};
use crate::config::Config;
use crate::resolver::{SettledHeadings, WorkspaceContext};

/// Run one formatter pass across every project in a workspace-root scope.
///
/// A write-capable caller first invokes this with `perform_writes = false`.
/// That lets every project discover whether it needs the whole declaration set
/// before any earlier project can be changed. If any project refuses, scan
/// errors from both strict and ordinary project walks become one strict abort
/// in the root-then-members order established by `WorkspaceContext`
/// (§FS-fmt.3.3). A conflict block refuses the same way, every project's blocks
/// in that order, and an unreadable path anywhere in the run wins over it,
/// because a scan that could not read a path cannot say what the path holds
/// (§FS-fmt.3.3).
///
/// Every project is walked before any project's locks are judged, and every
/// lock judged before anything is written: a citation into another project is
/// judged against that project as this run leaves it, and the root's walk comes
/// before its members' (§FS-section-lock.write.same-run, §FS-fmt.7.4.3).
pub(crate) fn fmt_workspace_projects(
    context: &WorkspaceContext,
    render: &Config,
    add_marker: bool,
    explicit_cross_refs: bool,
    perform_writes: bool,
) -> Result<FmtTreeOutcome> {
    let mut outcome = FmtTreeOutcome {
        changes: Vec::new(),
        scan_errors: Vec::new(),
        refused_writes: Vec::new(),
    };
    // §FS-fmt.6.3.3: a qualified link names a heading in another project as this
    // run leaves it, so every project's headings are settled before any is walked.
    let mut settled = SettledHeadings::default();
    for project in &context.projects {
        if let Some(findings) = project.complete_findings() {
            settled.extend(settle_project_headings(
                &project.config,
                add_marker,
                findings.findings(),
                context,
            )?);
        }
    }

    // One slot per project: its plan, or the refusal it met (§FS-fmt.3.3).
    let mut plans = Vec::new();
    let mut refusals: Vec<Option<anyhow::Error>> = Vec::new();
    for project in &context.projects {
        let auto_cross_refs =
            auto_cross_refs_for_scope(&project.config, Some(&project.config.root), true)?;
        let run_opts = FmtRunOpts {
            add_marker,
            cross_refs: explicit_cross_refs || auto_cross_refs,
            write: perform_writes,
            render,
            workspace: Some(context),
            precomputed_findings: project.complete_findings(),
            // §FS-fmt.6.1.3: check previews the same index carve-out write applies.
            index_cross_refs: true,
            settled: Some(&settled),
        };
        match plan_tree(&project.config, Some(&project.config.root), true, &run_opts) {
            Ok(plan) => {
                plans.push(Some(plan));
                refusals.push(None);
            }
            Err(error) => {
                plans.push(None);
                refusals.push(Some(refusal(error)?));
            }
        }
    }
    // §FS-section-lock.write.same-run: each project's locks against every tree the walks leave.
    let locks = {
        let mut judged = Vec::new();
        for ((project, plan), refused) in context.projects.iter().zip(&plans).zip(&mut refusals) {
            let tree = plan
                .as_ref()
                .map(|plan| plan.judged(&project.config, render));
            judged.push(match tree.transpose() {
                Ok(tree) => tree.flatten(),
                Err(error) => {
                    *refused = Some(refusal(error)?);
                    None
                }
            });
        }
        let targets = lock_targets(Some(context), &judged);
        let changes = |tree: &Option<_>| match tree {
            Some(tree) => section_lock_changes(tree, &targets),
            None => Vec::new(),
        };
        judged.iter().map(changes).collect::<Vec<_>>()
    };

    let mut preflight_scan_errors = Vec::new();
    let mut strict_abort = false;
    let mut conflicts = Vec::new();
    for (plan, refused) in plans.iter().zip(&refusals) {
        match (refused, plan) {
            (Some(error), _) => {
                if let Some(abort) = error.downcast_ref::<FmtScanAbort>() {
                    strict_abort = true;
                    preflight_scan_errors.extend(abort.scan_errors.iter().cloned());
                } else if let Some(abort) = error.downcast_ref::<FmtConflictAbort>() {
                    conflicts.extend(abort.conflicts.iter().cloned());
                }
            }
            (None, Some(plan)) => {
                preflight_scan_errors.extend(plan.outcome.scan_errors.iter().cloned())
            }
            (None, None) => {}
        }
    }
    if strict_abort || !conflicts.is_empty() && !preflight_scan_errors.is_empty() {
        return Err(FmtScanAbort {
            scan_errors: preflight_scan_errors,
        }
        .into());
    }
    if !conflicts.is_empty() {
        return Err(FmtConflictAbort { conflicts }.into());
    }

    let planned = context.projects.iter().zip(plans).zip(locks);
    for ((project, plan), locks) in planned {
        let Some(plan) = plan else { continue };
        match plan.write(&project.config, render, locks, perform_writes) {
            Ok(mut walked) => {
                outcome.changes.append(&mut walked.changes);
                outcome.scan_errors.append(&mut walked.scan_errors);
                outcome.refused_writes.append(&mut walked.refused_writes);
            }
            Err(mut error) => {
                // §FS-distribution.3.2.2.1: retain prior projects in a late writer failure;
                // the preflight wrote none of theirs (§FS-fmt.7.4.3).
                if perform_writes
                    && let Some(diagnostic) = error.downcast_mut::<crate::OperationDiagnostic>()
                {
                    let partial = &mut diagnostic.partial_output;
                    if partial.is_object() {
                        let preceding = serde_json::json!({
                            "changes":outcome.changes.iter().map(|row|
                                row.partial_json(render)).collect::<Vec<_>>(),
                            "scan_errors":outcome.scan_errors.iter().map(|e|
                                serde_json::json!({"path":e.path,"message":e.message})).collect::<Vec<_>>(),
                            "refused_writes":outcome.refused_writes});
                        for key in ["changes", "scan_errors", "refused_writes"] {
                            let mut ordered =
                                preceding[key].as_array().cloned().unwrap_or_default();
                            ordered.extend(partial[key].as_array().into_iter().flatten().cloned());
                            partial[key] = serde_json::json!(ordered);
                        }
                    }
                }
                return Err(error);
            }
        }
    }
    Ok(outcome)
}

/// A refusal one project met, kept for the run's one refusal where it is a scan
/// or a conflict abort (§FS-fmt.3.3); anything else ends the run at once.
fn refusal(error: anyhow::Error) -> Result<anyhow::Error> {
    if error.downcast_ref::<FmtScanAbort>().is_some()
        || error.downcast_ref::<FmtConflictAbort>().is_some()
    {
        Ok(error)
    } else {
        Err(error)
    }
}
