//! Pre-write section validation for fetched snapshots (§FS-fetch.3.sections).
//! The ordinary per-file scan owns recognition, canonical relative paths,
//! fences and duplicate records. Shared record helpers own the structural
//! predicates; this caller only selects errors and renders a fetch refusal.

use std::path::Path;

use super::fetch::{FetchFailure, fetch_operational};
use crate::config::Config;
use crate::grammar::render_id;
use crate::model::{Id, remedies};
use crate::scanner::scan_snapshot_text;

/// Judge only the requested declaration in provider memory, under the selected
/// project's configuration, without a tree walk or citation-resolution gate
/// (§FS-fetch.3.sections). `path` supplies the snapshot's file reading and home
/// context; scanner line numbers still refer to the provider's original stdout.
pub(super) fn validate_fetched_sections(
    text: &str,
    requested: &Id,
    path: &Path,
    selected: &Config,
) -> Result<(), FetchFailure> {
    let schema = selected.schema();
    let records = scan_snapshot_text(path, text, schema, selected.frame())
        .map_err(|error| fetch_operational(format!("cannot scan fetch output: {error:#}")))?;
    let owner = render_id(&selected.grammar, requested);
    let separator = &schema.ids.section_separator;
    for declaration in records.declarations.get(requested).into_iter().flatten() {
        // §FS-declarations.checks.section-heading-level: only strict/must is an error.
        if schema.ids.section_heading_levels == "strict" {
            for (section_path, section) in &declaration.sections {
                if let Some(expected) = declaration.section_heading_mismatch(section_path, section)
                {
                    // §FS-fetch.3: output holds one subsection depth, so re-levelling, check's
                    // first repair, would only meet the depth refusal; the path is what can move.
                    return Err(fetch_operational(format!(
                        "fetch output line {}: section {owner}{separator}{section_path} heading \
                         level mismatch: expected {} (level {expected}), found {} (level {}); \
                         fetch output holds one subsection depth, so give the heading a \
                         one-component path or fence it as content",
                        section.line,
                        "#".repeat(expected),
                        "#".repeat(section.heading_level),
                        section.heading_level,
                    )));
                }
            }
        }
        // §FS-declarations.checks.orphan-section: only enabled name-bearing paths need prefixes.
        if schema.ids.named_sections {
            for (section_path, section) in &declaration.sections {
                if let Some(prefix) = declaration.missing_section_prefix(section_path) {
                    return Err(fetch_operational(format!(
                        "fetch output line {}: orphan section {owner}{separator}{section_path}: \
                         missing prefix {owner}{separator}{prefix}; {}",
                        section.line,
                        remedies::orphan_section(),
                    )));
                }
            }
        }
        // §FS-declarations.checks.duplicate-section: errors independently of depth strength.
        if let Some((section_path, lines)) = declaration.section_collisions().into_iter().next() {
            let others = lines[1..]
                .iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            return Err(fetch_operational(format!(
                "fetch output line {}: duplicate section {owner}{separator}{section_path} \
                 (also declared at fetched-output lines {others}); {}",
                lines[0],
                remedies::duplicate_section_headings(),
            )));
        }
    }
    Ok(())
}
