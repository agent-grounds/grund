//! The excluded-stub-target warning (§FS-declarations.checks.excluded-stub-target): a
//! stub that pairs with a target the walk would not read, for a `[scan] exclude` entry
//! or an ignore file, reported at the stub, since grund 0.19.0 reports it as
//! `broken-stub`. The scanner notes the stub where it reaches its verdict
//! (§AR-scanner.1.13); this reports the note.

use crate::config::Frame;
use crate::grammar::render_id;
use crate::model::{Catalog, CheckReport, Diagnostic, ExcludingRule, format_path};

/// §FS-declarations.checks.excluded-stub-target: one warning per noted stub. It leaves
/// the exit status where the rest of the run put it (§FS-check.2), and the stub's
/// verdict does not move until 0.19.0.
pub(super) fn check_excluded_stub_targets(
    findings: &Catalog,
    frame: Frame<'_>,
    report: &mut CheckReport,
) {
    report
        .warnings
        .extend(findings.excluded_stub_target_notes.iter().map(|note| {
            let id = render_id(frame.grammar(), &note.id);
            let target = format_path(&note.target);
            let (because, remedy) = match &note.rule {
                ExcludingRule::Exclude(entry) => (
                    format!("[scan] exclude names {entry}"),
                    format!(
                        "move the target out of {entry}, or take {entry} out of [scan] exclude"
                    ),
                ),
                ExcludingRule::Pattern(pattern) => (
                    format!("[schema.sources] exclude matches it by {pattern}"),
                    format!(
                        "move the target out of what {pattern} matches, or take {pattern} out of \
                         [schema.sources] exclude"
                    ),
                ),
                ExcludingRule::IgnoreFile { file, pattern } => (
                    format!("{file} ignores it by {pattern}"),
                    format!(
                        "move the target out of what {file} ignores, or stop ignoring it there"
                    ),
                ),
            };
            Diagnostic {
                code: "excluded-stub-target",
                path: Some(note.file.clone()),
                line: Some(note.line),
                column: None,
                message: format!(
                    "{id} is a stub to {target}, which the scan does not read because {because}, \
                     and grund 0.19.0 reports this stub as broken-stub — {remedy}, to keep the stub"
                ),
                sites: Vec::new(),
                authority: Vec::new(),
            }
        }));
}
