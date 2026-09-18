//! Persisted off-grammar compatibility (§FS-config.3.2, §FS-check.4.6): a
//! declaration or citation whose exact spelling predates the configured
//! `[id] format`, reconciled against the catalog the same tree scan produced.
//!
//! Named `legacy` for the spellings it matches, not for the deprecated frontend
//! `compat` now means (§AR-system.2.9). Deliberately a **catalog** operation: the
//! authoring grammar stays strict and no unbacked token becomes a citation, so
//! everything here runs after the walk rather than inside a line pass.

use anyhow::anyhow;
use std::collections::BTreeMap;

use super::legacy_inline::reconcile_promoted_inline_site;
use crate::config::Config;
use crate::grammar::{
    IdArgError, MarkdownLineCitation, QUALIFIED_CITATION_PREFIX, is_inside_inline_code,
    parse_id_arg, parse_id_arg_with_shorthand, render_id, shorthand_candidates, shorthand_names,
};
use crate::model::sort_path_key;
use crate::model::{Citation, Declaration, Findings, Id, LegacyCitationCandidate};

/// Resolve a query through the canonical grammar first, then combine exact
/// catalog compatibility with number shorthand (§FS-config.3.2, §FS-show.1).
pub(crate) fn resolve_id_arg(
    raw: &str,
    config: &Config,
    findings: &Findings,
) -> std::result::Result<(Id, Option<String>), IdArgError> {
    if let Ok(parsed) = parse_id_arg(raw, &config.grammar) {
        return Ok(parsed);
    }
    let legacy_catalog = legacy_catalog_ids(&findings.declarations);
    let legacy = match_legacy_tail(raw, config, &legacy_catalog);
    let parsed = parse_id_arg_with_shorthand(raw, &config.grammar);
    let shorthand = parsed.as_ref().ok().filter(|parsed| parsed.shorthand);
    let mut targets = Vec::<(Id, Option<String>)>::new();
    if let Some((id, section, consumed)) = legacy
        && consumed == raw.len()
    {
        targets.push((id, section));
    }
    if let Some(parsed) = shorthand {
        for candidate in shorthand_candidates(&parsed.id, &findings.declarations) {
            targets.push((candidate.clone(), parsed.section.clone()));
        }
    }
    targets.sort();
    targets.dedup();
    match targets.as_slice() {
        [(id, section)] => Ok((id.clone(), section.clone())),
        [] => match parsed {
            Ok(parsed) => Ok((parsed.id, parsed.section)),
            Err(err) => Err(IdArgError::Unparsable(err)),
        },
        many => Err(IdArgError::Ambiguous(anyhow!(
            "ambiguous ID: {raw} (matches {})",
            many.iter()
                .map(|(id, _)| render_id(config, id))
                .collect::<Vec<_>>()
                .join(", ")
        ))),
    }
}

/// Reconcile deferred marker-prefixed candidates against the catalog produced
/// by the same tree scan (§FS-config.3.2, §FS-check.1.1). This is deliberately
/// a catalog operation: the authoring regex stays strict, and no unbacked token
/// can become a citation.
pub(super) fn promote_local_legacy_citations(config: &Config, findings: &mut Findings) {
    let catalog = legacy_catalog_ids(&findings.declarations);
    let configured_catalog = configured_catalog_ids(&findings.declarations);
    let candidates = std::mem::take(&mut findings.legacy_citation_candidates);
    for candidate in candidates {
        if candidate.namespace.is_some() {
            findings.legacy_citation_candidates.push(candidate);
            continue;
        }
        promote_legacy_candidate(
            config,
            config,
            &configured_catalog,
            &catalog,
            candidate,
            &mut findings.citations,
        );
    }
    sort_citations(&mut findings.citations);
}

pub(crate) fn legacy_catalog_ids(declarations: &BTreeMap<Id, Vec<Declaration>>) -> Vec<Id> {
    declarations
        .keys()
        .filter(|id| id.legacy_spelling().is_some())
        .cloned()
        .collect()
}

/// The complement of [`legacy_catalog_ids`]: every ID the configured grammar
/// spells. `pub(crate)` for one reader — the qualified half of this
/// reconciliation, which takes the whole loaded project set and is therefore
/// `resolver/legacy_promotion.rs` (§AR-resolver.placement).
pub(crate) fn configured_catalog_ids(declarations: &BTreeMap<Id, Vec<Declaration>>) -> Vec<Id> {
    declarations
        .keys()
        .filter(|id| id.legacy_spelling().is_none())
        .cloned()
        .collect()
}

pub(crate) fn formatter_wrapper_label_is_citation(label: &str, config: &Config) -> bool {
    let tail = match QUALIFIED_CITATION_PREFIX.captures(label) {
        Some(prefix) => &label[prefix.get(0).expect("qualified prefix match").end()..],
        None => label,
    };
    if tail.is_empty() || tail.contains('/') {
        return false;
    }
    parse_id_arg_with_shorthand(tail, &config.grammar).is_ok()
        || config.grammar.legacy_kind_and_format(tail).is_some()
}

/// One deferred candidate reconciled against one target catalog — the step both
/// halves of this pass share, the local one above and the qualified one in
/// `resolver/legacy_promotion.rs` (§AR-resolver.placement).
pub(crate) fn promote_legacy_candidate(
    source_config: &Config,
    target_config: &Config,
    configured_catalog: &[Id],
    catalog: &[Id],
    candidate: LegacyCitationCandidate,
    citations: &mut Vec<Citation>,
) {
    let Some((id, section, consumed)) = match_legacy_tail(&candidate.tail, target_config, catalog)
    else {
        return;
    };
    let qualified = candidate
        .namespace
        .as_deref()
        .map(|alias| format!("{alias}/"))
        .unwrap_or_default();
    let text = format!(
        "{}{}{}",
        source_config.marker,
        qualified,
        &candidate.tail[..consumed]
    );
    let same_marker = |citation: &Citation| {
        citation.file == candidate.file
            && citation.line == candidate.line
            && citation.column == candidate.column
    };
    // §FS-config.3.2 / §FS-check.1.1: a configured parse reaching past the
    // shorter legacy target owns its marker. A shorthand owns it when its
    // configured target set is non-empty; `ShorthandIndex` adds the legacy target.
    if citations.iter().any(|citation| {
        same_marker(citation)
            && ((!citation.shorthand && citation.text.len() > text.len())
                || configured_catalog
                    .iter()
                    .any(|id| shorthand_names(id, &citation.id)))
    }) {
        return;
    }
    // With no configured interpretation left, exact catalog compatibility is
    // the one target and replaces the rejected/empty shorthand parse.
    citations.retain(|citation| !same_marker(citation));
    let site = candidate.inline_site.clone();
    let block_lines = candidate.inline_block_lines.clone();
    let file = candidate.file.clone();
    citations.push(Citation {
        namespace: candidate.namespace,
        id,
        section,
        file: candidate.file,
        line: candidate.line,
        column: candidate.column,
        has_marker: true,
        shorthand: false,
        shorthand_rewritable: true,
        numeric_run: false,
        text,
        inline_site: candidate.inline_site,
        source_kind: candidate.source_kind,
        enclosing_declaration: candidate.enclosing_declaration,
    });
    if let (Some(site), Some(block_lines)) = (site, block_lines) {
        reconcile_promoted_inline_site(source_config, citations, &file, site, &block_lines);
    }
}

/// Apply the exact-ID-before-section precedence from §FS-config.3.2 to one
/// deferred line tail. Multiple section-prefix interpretations stay unresolved
/// instead of selecting a declaration by iteration order.
pub(crate) fn match_legacy_tail(
    tail: &str,
    config: &Config,
    catalog: &[Id],
) -> Option<(Id, Option<String>, usize)> {
    let mut exact = catalog
        .iter()
        .filter_map(|id| {
            let spelling = id.legacy_spelling()?;
            let rest = tail.strip_prefix(spelling)?;
            exact_token_boundary(rest, config).then_some((id, spelling.len()))
        })
        .collect::<Vec<_>>();
    exact.sort_by_key(|(_, len)| std::cmp::Reverse(*len));
    if let Some((id, len)) = exact.first() {
        return Some(((*id).clone(), None, *len));
    }

    let mut sections = catalog
        .iter()
        .filter_map(|id| {
            let spelling = id.legacy_spelling()?;
            let rest = tail.strip_prefix(spelling)?;
            let rest = rest.strip_prefix(&config.section_separator)?;
            let (section, section_len) = longest_section_prefix(rest, config)?;
            Some((
                id.clone(),
                section,
                spelling.len() + config.section_separator.len() + section_len,
            ))
        })
        .collect::<Vec<_>>();
    sections.sort_by(|a, b| (a.0.clone(), &a.1).cmp(&(b.0.clone(), &b.1)));
    sections.dedup_by(|a, b| a.0 == b.0 && a.1 == b.1);
    match sections.as_slice() {
        [(id, section, consumed)] => Some((id.clone(), Some(section.to_string()), *consumed)),
        _ => None,
    }
}

fn exact_token_boundary(rest: &str, config: &Config) -> bool {
    if rest.is_empty() {
        return true;
    }
    if let Some(after_separator) = rest.strip_prefix(&config.section_separator)
        && longest_section_prefix(after_separator, config).is_some()
    {
        return false;
    }
    rest.chars().next().is_some_and(|ch| {
        ch.is_whitespace()
            || matches!(
                ch,
                '.' | ',' | ';' | ':' | '!' | '?' | ')' | ']' | '}' | '>' | '`' | '\'' | '"'
            )
    })
}

fn longest_section_prefix<'a>(tail: &'a str, config: &Config) -> Option<(&'a str, usize)> {
    tail.char_indices()
        .map(|(index, ch)| index + ch.len_utf8())
        .filter_map(|end| {
            let section = &tail[..end];
            if !config.grammar.is_section_path(section) {
                return None;
            }
            let rest = &tail[end..];
            (rest.is_empty()
                || rest.chars().next().is_some_and(|ch| {
                    ch.is_whitespace()
                        || matches!(
                            ch,
                            '.' | ','
                                | ';'
                                | ':'
                                | '!'
                                | '?'
                                | ')'
                                | ']'
                                | '}'
                                | '>'
                                | '`'
                                | '\''
                                | '"'
                        )
                }))
            .then_some((section, end))
        })
        .last()
}

/// The one order citations are kept in, applied after either half of this pass
/// promotes into the list (§FS-errors.4).
pub(crate) fn sort_citations(citations: &mut [Citation]) {
    citations.sort_by(|a, b| {
        (sort_path_key(&a.file), a.line, a.column).cmp(&(sort_path_key(&b.file), b.line, b.column))
    });
}

/// §FS-fmt.6 / §FS-config.3.2: feed Markdown wrapping from the same exact
/// declaration-backed boundary as scanner promotion, never a relaxed parser.
pub(crate) fn collect_local_legacy_markdown_citations(
    line: &str,
    config: &Config,
    findings: &Findings,
    citations: &mut Vec<MarkdownLineCitation>,
) {
    let catalog = legacy_catalog_ids(&findings.declarations);
    for (marker_start, _) in line.match_indices(&config.marker) {
        if is_inside_inline_code(line, marker_start) {
            continue;
        }
        let token_start = marker_start + config.marker.len();
        let Some(rest) = line.get(token_start..) else {
            continue;
        };
        if QUALIFIED_CITATION_PREFIX.is_match(rest) {
            continue;
        }
        let Some((id, section, consumed)) = match_legacy_tail(rest, config, &catalog) else {
            continue;
        };
        // §FS-config.3.2 / §FS-fmt.6: do not linkify a shorthand-shaped legacy
        // ID when conforming declarations share its number. The scanner reports
        // the combined target set; formatting leaves the same bytes untouched.
        if parse_id_arg_with_shorthand(&rest[..consumed], &config.grammar).is_ok_and(|parsed| {
            parsed.shorthand && !shorthand_candidates(&parsed.id, &findings.declarations).is_empty()
        }) {
            continue;
        }
        citations.push(MarkdownLineCitation {
            marker_start,
            token_end: token_start + consumed,
            namespace: None,
            id,
            section,
        });
    }
}
