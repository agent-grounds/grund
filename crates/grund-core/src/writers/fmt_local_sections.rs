//! Canonical expansion and report labels for declaration-local section
//! citations (§FS-fmt.2.4).

use std::path::Path;

use crate::config::Config;
use crate::grammar::render_id;
use crate::model::Findings;

/// Expand only scanner-proven, uniquely owned local section edges. The complete
/// scan is requested lazily on the first persisted marker-plus-digit candidate,
/// exactly as for number-only shorthand; trigger-created `$$2` markers are
/// excluded because §FS-fmt.2.4 adds no typing feature for local paths.
pub(super) fn expand_local_section_citations(
    line: &str,
    path: &Path,
    lineno: usize,
    config: &Config,
    findings: Option<&Findings>,
    trigger_marker_starts: &[usize],
    saw_candidate: &mut bool,
) -> Option<String> {
    if config.marker.is_empty() || !line.contains(&config.marker) {
        return None;
    }
    let Some(findings) = findings else {
        *saw_candidate |= line.match_indices(&config.marker).any(|(start, _)| {
            !trigger_marker_starts.contains(&start)
                && line[start + config.marker.len()..].starts_with(|ch: char| ch.is_ascii_digit())
        });
        return None;
    };
    let sites = findings
        .citations
        .iter()
        .filter(|cite| cite.local_section && cite.file == path && cite.line == lineno)
        .collect::<Vec<_>>();
    if sites.is_empty() {
        return None;
    }
    let mut output = String::with_capacity(line.len());
    let mut cursor = 0;
    let mut changed = false;
    for cite in sites {
        let Some(relative) = line[cursor..].find(&cite.text) else {
            continue;
        };
        let start = cursor + relative;
        let end = start + cite.text.len();
        output.push_str(&line[cursor..start]);
        if cite.shorthand_rewritable {
            output.push_str(&config.marker);
            output.push_str(&render_id(&config.grammar, &cite.id));
            output.push_str(&config.section_separator);
            output.push_str(cite.section.as_deref().unwrap_or_default());
            changed = true;
        } else {
            output.push_str(&line[start..end]);
        }
        cursor = end;
    }
    if !changed {
        return None;
    }
    output.push_str(&line[cursor..]);
    Some(output)
}

/// One report row per local replacement, even when several share a line
/// (§FS-fmt.2.4). This keeps dry-run and write reviewable at token granularity.
pub(super) fn local_section_labels(
    path: &Path,
    lineno: usize,
    config: &Config,
    findings: Option<&Findings>,
) -> Vec<String> {
    findings
        .into_iter()
        .flat_map(|findings| &findings.citations)
        .filter(|cite| {
            cite.local_section
                && cite.shorthand_rewritable
                && cite.file == path
                && cite.line == lineno
        })
        .map(|cite| {
            let canonical = format!(
                "{}{}{}{}",
                config.marker,
                render_id(&config.grammar, &cite.id),
                config.section_separator,
                cite.section.as_deref().unwrap_or_default(),
            );
            format!(
                "local section \u{2192} canonical: {} \u{2192} {canonical}",
                cite.text
            )
        })
        .collect()
}
