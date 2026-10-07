//! Canonical expansion of declaration-local section citations, and the
//! details their report row names (§FS-fmt.2.4, §FS-fmt.3.6.1).

use std::path::Path;

use crate::config::Config;
use crate::grammar::render_id;
use crate::model::{Catalog, Citation};
use crate::resolver::section_resolves;

/// Expand only scanner-proven, uniquely owned local section edges whose cited
/// section resolves against that owner (§FS-fmt.2.4.6). The complete
/// scan is requested lazily on the first persisted marker-plus-digit candidate,
/// exactly as for number-only shorthand; trigger-created `$$2` markers are
/// excluded because §FS-fmt.2.4 adds no typing feature for local paths.
///
/// Each expansion is recorded in `expansions` as its byte offset in the returned
/// line, the text it replaced, and the canonical text written — the details its
/// line's one report row names (§FS-fmt.3.6.1). Recording them as the line is
/// built is what keeps the preview to exactly the sites the write expands
/// (§FS-fmt.7.3).
#[allow(clippy::too_many_arguments)]
pub(super) fn expand_local_section_citations(
    line: &str,
    path: &Path,
    lineno: usize,
    config: &Config,
    findings: Option<&Catalog>,
    trigger_marker_starts: &[usize],
    saw_candidate: &mut bool,
    expansions: &mut Vec<(usize, String, String)>,
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
        // §FS-fmt.2.4.6: a refused site is copied through byte-identical.
        if expandable(cite, findings, config) {
            let written_start = output.len();
            output.push_str(&config.marker);
            output.push_str(&render_id(&config.grammar, &cite.id));
            output.push_str(&config.section_separator);
            output.push_str(cite.section.as_deref().unwrap_or_default());
            expansions.push((
                written_start,
                cite.text.clone(),
                output[written_start..].to_string(),
            ));
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

/// Whether the write expands this owned local site: a text the writer rules
/// permit, and a cited section its owner records a heading for (§FS-fmt.2.4.6),
/// asked of the lookup `check` withholds the command on (§FS-check.3.24.1), which
/// reads a stub's sections from its target (§FS-check.3.2.1).
fn expandable(cite: &Citation, findings: &Catalog, config: &Config) -> bool {
    cite.shorthand_rewritable
        && section_resolves(
            findings,
            config,
            &cite.id,
            cite.section.as_deref().unwrap_or_default(),
        )
}
