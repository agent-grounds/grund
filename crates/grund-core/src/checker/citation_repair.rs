//! Applicable repairs for required marked index entries (§FS-check.3.12.2).
//! Both direction producers finish here, after rule deduplication, so repair
//! selection changes only message text and never a verdict or its authority.

use std::collections::BTreeMap;

use super::citation_prohibitions::citation_site_level;
use super::index::marked_index_entries;
use crate::config::{Frame, Rules, Schema};
use crate::model::remedies;
use crate::model::{Catalog, CheckReport, Code, Expected, citation_direction_repair};

/// §FS-check.3.12.2: replace only the ordinary repair at actual entry sites,
/// retaining any trailing compatibility clause for either direction producer.
/// Call after both passes, before narrowing or merging project reports; a
/// suggestion has no repair and is left byte-for-byte unchanged.
pub(crate) fn repair_index_citations(
    findings: &Catalog,
    rules: &Rules,
    schema: &Schema,
    expected: &Expected,
    frame: Frame<'_>,
    report: &mut CheckReport,
) {
    if !report
        .errors
        .iter()
        .chain(&report.warnings)
        .any(|finding| finding.code == Code::FORBIDDEN_CITATION)
    {
        return;
    }
    let entries = marked_index_entries(findings, schema, frame, &expected.index_targets)
        .into_iter()
        .map(|cite| ((cite.file.as_path(), cite.line, cite.column), cite))
        .collect::<BTreeMap<_, _>>();
    for finding in report.errors.iter_mut().chain(&mut report.warnings) {
        if finding.code != Code::FORBIDDEN_CITATION {
            continue;
        }
        let (Some(path), Some(line), Some(column)) =
            (finding.path.as_deref(), finding.line, finding.column)
        else {
            continue;
        };
        let Some(cite) = entries.get(&(path, line, column)) else {
            continue;
        };
        let Some((prefix, trailing)) = finding.message.split_once(&citation_direction_repair())
        else {
            continue;
        };
        // §FS-errors.5.5.1: each repair is a case of forbidden-citation's Fix.
        let repair = if !finding.authority.is_empty() {
            remedies::forbidden_citation_index_rule(finding.authority.join(", "))
        } else {
            let Some((level, matched)) = citation_site_level(rules, cite) else {
                continue;
            };
            let table = if frame.version == 2 {
                "rules.citations"
            } else {
                "citations"
            };
            let table = format!("[{table}.{}]", cite.source_kind);
            if matched.is_empty() {
                remedies::forbidden_citation_index_may(&cite.id.kind, &table)
            } else {
                remedies::forbidden_citation_index_prohibition(level.as_str(), &table)
            }
        };
        finding.message = format!("{prefix} — {repair}{trailing}");
    }
}
