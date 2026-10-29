//! The section-lock rule family (§FS-section-lock.checks): what `check` says
//! about the lock against the tree, read off the one judgement `fmt --write`
//! writes from, so a finding and the write that clears it agree on every entry.
//! The lock files are read by the scan, beside the walk, so no rule here reads
//! a file (§AR-checker.placement).

use crate::config::Frame;
use crate::model::remedies;
use crate::model::{CheckReport, Code, Diagnostic, Site};
use crate::resolver::{EntryVerdict, LockJudgement};

/// §FS-section-lock.checks: a lock line grund cannot read, a cited locked
/// address the lock does not hold, and a cited section whose recorded text now
/// sits at another address, read off `judgements`, the run's one judgement of
/// its locks. None fires in a tree with no lock file and no citation of a locked
/// address.
pub(super) fn check_section_locks(
    judgements: &[LockJudgement<'_>],
    frame: Frame<'_>,
    report: &mut CheckReport,
) {
    for judgement in judgements {
        // §FS-section-lock.checks: named as the report names every path.
        let lock = frame.display_path(&judgement.path);
        let coordinate = |section: &str| judgement.cited(section);
        // §FS-section-lock.checks.invalid-section-lock
        for line in &judgement.lines.invalid {
            report.errors.push(located(
                Code::INVALID_SECTION_LOCK,
                judgement.path.clone(),
                *line,
                // §FS-section-lock.checks.remedy: the row's Fix, which takes the run by reference.
                format!(
                    "section lock line is not <coordinate> <fingerprint>; {}",
                    remedies::invalid_section_lock()
                ),
            ));
        }
        // §FS-section-lock.checks.unrecorded-section-citation: at each citation's site.
        for (section, citations, _) in &judgement.unrecorded {
            for citation in citations {
                report.errors.push(located(
                    Code::UNRECORDED_SECTION_CITATION,
                    citation.file.clone(),
                    citation.line,
                    // §FS-section-lock.checks.remedy: the run that writes a lock, with its scope.
                    format!(
                        "{} is cited but has no entry in {lock}; {}",
                        coordinate(section),
                        remedies::unrecorded_section_citation()
                    ),
                ));
            }
        }
        // §FS-section-lock.checks.section-address-changed: at the heading holding it now.
        let Some(decl) = judgement.decl else {
            continue;
        };
        for (entry, verdict) in &judgement.verdicts {
            let (EntryVerdict::Moved { to, sites }, Some(info)) =
                (verdict, decl.sections.get(&entry.section))
            else {
                continue;
            };
            let (at, to) = (coordinate(&entry.section), coordinate(to));
            let mut finding = located(
                Code::SECTION_ADDRESS_CHANGED,
                decl.file.clone(),
                info.line,
                format!(
                    "{at} is cited, and what {lock} recorded for it now sits at {to}; {}",
                    remedies::section_address_changed(&to, &at)
                ),
            );
            // §FS-section-lock.checks.section-address-changed: the finding's own words
            // after the remedy name each line still citing `at`; `sites` holds the same.
            if !sites.is_empty() {
                let named = sites
                    .iter()
                    .map(|(path, line)| format!("{}:{line}", frame.display_path(path)))
                    .collect::<Vec<_>>();
                finding
                    .message
                    .push_str(&format!("; {at} is cited at {}", named.join(", ")));
                finding.sites = sites
                    .iter()
                    .map(|(path, line)| Site {
                        path: path.clone(),
                        line: *line,
                    })
                    .collect();
            }
            report.errors.push(finding);
        }
    }
}

fn located(code: Code, path: std::path::PathBuf, line: usize, message: String) -> Diagnostic {
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
