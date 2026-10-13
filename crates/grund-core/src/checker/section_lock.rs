//! The section-lock rule family (§FS-section-lock.checks): what `check` says
//! about the lock against the tree, read off the one judgement `fmt --write`
//! writes from, so a finding and the write that clears it agree on every entry.
//! The lock files are read by the scan, beside the walk, so no rule here reads
//! a file (§AR-checker.placement).

use std::collections::BTreeMap;

use crate::config::{Frame, Schema};
use crate::model::{CheckReport, Diagnostic, name_under_number};
use crate::resolver::{EntryVerdict, WorkspaceCheckTarget, judge_section_locks};
use crate::rules::engine::UNRECORDED_MIXED_CLAUSE;

/// §FS-section-lock.checks: a lock line grund cannot read, a cited locked
/// address the lock does not hold, and a cited section whose recorded text now
/// sits at another address. None fires in a tree with no lock file and no
/// citation of a locked address. A citation into another project is judged
/// against that project in `targets` (§FS-section-lock.file.members).
pub(super) fn check_section_locks<'a>(
    findings: &'a crate::model::Catalog,
    schema: &Schema,
    frame: Frame<'_>,
    targets: &BTreeMap<String, WorkspaceCheckTarget<'a>>,
    report: &mut CheckReport,
) {
    for judgement in judge_section_locks(findings, schema, frame, targets) {
        // §FS-section-lock.checks: named as the report names every path.
        let lock = frame.display_path(&judgement.path);
        let coordinate = |section: &str| judgement.cited(section);
        // §FS-section-lock.checks.invalid-section-lock
        for line in &judgement.lines.invalid {
            report.errors.push(located(
                "invalid-section-lock",
                judgement.path.clone(),
                *line,
                "section lock line is not <coordinate> <fingerprint>; run grund fmt --write \
                 to rebuild the file"
                    .to_string(),
            ));
        }
        // §FS-section-lock.checks.unrecorded-section-citation: at each citation's site,
        // naming the releases for a mixed address, which grund before 0.18.0 skipped.
        for (section, citations, _) in &judgement.unrecorded {
            let clause = match name_under_number(section) {
                true => UNRECORDED_MIXED_CLAUSE,
                false => "",
            };
            for citation in citations {
                report.errors.push(located(
                    "unrecorded-section-citation",
                    citation.file.clone(),
                    citation.line,
                    format!(
                        "{} is cited but has no entry in {lock}; run grund fmt --write{clause}",
                        coordinate(section)
                    ),
                ));
            }
        }
        // §FS-section-lock.checks.section-address-changed: at the heading holding it now.
        let Some(decl) = judgement.decl else {
            continue;
        };
        for (entry, verdict) in &judgement.verdicts {
            let (EntryVerdict::Moved { to }, Some(info)) =
                (verdict, decl.sections.get(&entry.section))
            else {
                continue;
            };
            let (at, to) = (coordinate(&entry.section), coordinate(to));
            report.errors.push(located(
                "section-address-changed",
                decl.file.clone(),
                info.line,
                format!(
                    "{at} is cited, and what {lock} recorded for it now sits at {to}; move it \
                     back, cite {to} instead, or leave a pointer to it at {at}; if {to} is a \
                     copy, edit the copy or cite it from {at}"
                ),
            ));
        }
    }
}

fn located(
    code: &'static str,
    path: std::path::PathBuf,
    line: usize,
    message: String,
) -> Diagnostic {
    Diagnostic {
        code,
        path: Some(path),
        line: Some(line),
        column: None,
        message,
        sites: Vec::new(),
        authority: Vec::new(),
    }
}
