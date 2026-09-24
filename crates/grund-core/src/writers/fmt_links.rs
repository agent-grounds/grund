//! The cross-reference link pass (§FS-fmt.6): which citations on a Markdown
//! line get a link, and the splice that writes one without disturbing the bytes
//! around it. `resolver/link_targets.rs` computes the URL each one points at,
//! `fmt_shorthand_links.rs` resolves an accepted shorthand into a linkable
//! citation, and the record of one citation found on a line is
//! `grammar/ids.rs`'s (§AR-system.2.1).
//!
//! The inverse is not here. Undoing the wrap is recognizing it — a `[` before a
//! marker-prefixed citation token and `](…)` after it, nothing resolved — so
//! `flatten_cross_ref_links` is `grammar/fmt_cross_refs.rs`, with the
//! citation-shaped-label predicate it and this pass used to share, which the
//! `show` query, the measured point body and the deprecated adapter all read
//! downward instead of sideways (§DF-show-cross-ref-flattening, §AR-system.4).

use std::collections::BTreeSet;
use std::path::Path;

use super::fmt_shorthand_links::{accepted_shorthand_link, collect_local_accepted_shorthand_links};
use super::fmt_value_bindings::markdown_citation_is_value_binding;
use crate::config::Config;
use crate::grammar::{
    MarkdownLineCitation, QUALIFIED_CITATION_PREFIX, is_inside_inline_code, parse_id,
    parse_longest_id_prefix,
};
use crate::model::{Findings, Id};
use crate::resolver::{
    ShorthandTargets, WorkspaceContext, markdown_link_target, markdown_link_target_with_root,
};
use crate::scanner::{
    collect_local_legacy_markdown_citations, legacy_catalog_ids, match_legacy_tail,
};

/// Wrap each `§<ID>[.<section>]` citation on this Markdown line as `[§<ID>…](url)`
/// — the `--cross-refs` rewrite (§FS-fmt.6.2): re-derive an existing wrapper's URL,
/// skip citations in inline code (§FS-fmt.6.4), and emit nothing when the target
/// does not resolve (§FS-fmt.6.4).
///
/// `workspace` is `None` for single-project runs (and for member-local
/// runs in a workspace — §FS-workspace.8.5.1). When `Some`, a qualified
/// `§<alias>/<ID>` resolves against the named project's findings, with
/// the relative path crossing the workspace and the anchor computed
/// against the target project's config (§FS-workspace.8.5).
#[cfg(test)]
pub(crate) fn wrap_markdown_links(
    line: &str,
    path: &Path,
    config: &Config,
    findings: &Findings,
    workspace: Option<&WorkspaceContext>,
    only_ids: Option<&BTreeSet<Id>>,
) -> String {
    let targets = ShorthandTargets::new(config, Some(findings), workspace);
    wrap_markdown_links_with_targets(line, path, config, findings, workspace, only_ids, &targets)
}

/// The production §FS-fmt.6 wrapper pass, sharing §FS-fmt.2.4.5's already-built
/// target indexes so accepted shorthand adds no per-citation catalog scan.
pub(super) fn wrap_markdown_links_with_targets(
    line: &str,
    path: &Path,
    config: &Config,
    findings: &Findings,
    workspace: Option<&WorkspaceContext>,
    only_ids: Option<&BTreeSet<Id>>,
    shorthand_targets: &ShorthandTargets<'_>,
) -> String {
    let mut output = String::new();
    let mut cursor = 0;
    for citation in markdown_link_citations(line, config, findings, workspace, shorthand_targets) {
        if citation.marker_start < cursor {
            continue;
        }
        // §FS-fmt.6.1.2: the always-linkify carve-out reaches this file for its
        // index entries (§FS-check.3.18) and writes nothing else — a qualified
        // citation is never an entry, and an ID the index does not owe is prose.
        if let Some(only_ids) = only_ids
            && (citation.namespace.is_some() || !only_ids.contains(&citation.id))
        {
            continue;
        }
        // §FS-values.8: wrapping the citation would destroy the sole accepted
        // authored binding form, so a recognized value binding keeps its bytes.
        if markdown_citation_is_value_binding(line, &citation, config, findings, workspace) {
            continue;
        }
        // §FS-workspace.8.5: a qualified `§<alias>/<ID>` resolves against the
        // named project in workspace mode; a member-local run has no workspace
        // context and leaves it untouched — no wrap created, none stripped.
        let target = match citation.namespace.as_deref() {
            Some(namespace) => {
                let Some(workspace) = workspace else { continue };
                let Some(target_project) = workspace.project_by_alias(namespace) else {
                    continue;
                };
                markdown_link_target_with_root(
                    path,
                    &citation.id,
                    citation.section.as_deref(),
                    &target_project.config,
                    &target_project.findings,
                    Some(&workspace.render_root),
                )
            }
            None => markdown_link_target(
                path,
                &citation.id,
                citation.section.as_deref(),
                config,
                findings,
            ),
        };
        let Some(target) = target else {
            continue;
        };
        let marked_end = citation.token_end;
        let marker_start = citation.marker_start;
        let already_wrapped = marker_start > 0 && line.as_bytes()[marker_start - 1] == b'[';
        if already_wrapped && line[marked_end..].starts_with("](") {
            let url_start = marked_end + 2;
            if let Some(close_rel) = line[url_start..].find(')') {
                let close = url_start + close_rel;
                output.push_str(&line[cursor..url_start]);
                output.push_str(&target);
                cursor = close;
                continue;
            }
        }
        output.push_str(&line[cursor..marker_start]);
        let citation = &line[marker_start..marked_end];
        output.push('[');
        output.push_str(citation);
        output.push_str("](");
        output.push_str(&target);
        output.push(')');
        cursor = marked_end;
    }
    output.push_str(&line[cursor..]);
    output
}

fn markdown_link_citations(
    line: &str,
    config: &Config,
    findings: &Findings,
    workspace: Option<&WorkspaceContext>,
    shorthand_targets: &ShorthandTargets<'_>,
) -> Vec<MarkdownLineCitation> {
    let mut citations = Vec::new();
    if let Some(workspace) = workspace {
        collect_workspace_markdown_link_citations(
            line,
            config,
            workspace,
            shorthand_targets,
            &mut citations,
        );
    }
    for caps in config.grammar.citation_re.captures_iter(line) {
        let Some(full) = caps.get(0) else { continue };
        if config.grammar.has_reserved_named_tail(line, full.end()) {
            continue;
        }
        let marker_start = full.start().saturating_sub(config.marker.len());
        if !line[..full.start()].ends_with(&config.marker) {
            continue;
        }
        if is_inside_inline_code(line, marker_start) {
            continue;
        }
        let Some(id) = parse_id(&caps, &config.grammar) else {
            continue;
        };
        citations.push(MarkdownLineCitation {
            marker_start,
            token_end: full.end(),
            namespace: caps.name("namespace").map(|m| m.as_str().to_string()),
            id,
            section: caps.name("sec").map(|m| m.as_str().to_string()),
        });
    }
    collect_local_accepted_shorthand_links(
        line,
        config,
        shorthand_targets.local.as_ref(),
        &mut citations,
    );
    collect_local_legacy_markdown_citations(line, config, findings, &mut citations);
    citations.sort_by(|a, b| {
        (a.marker_start, std::cmp::Reverse(a.token_end))
            .cmp(&(b.marker_start, std::cmp::Reverse(b.token_end)))
    });
    citations.dedup_by(|a, b| a.marker_start == b.marker_start && a.token_end == b.token_end);
    citations
}

fn collect_workspace_markdown_link_citations(
    line: &str,
    config: &Config,
    workspace: &WorkspaceContext,
    shorthand_targets: &ShorthandTargets<'_>,
    out: &mut Vec<MarkdownLineCitation>,
) {
    if config.marker.is_empty() {
        return;
    }
    for (marker_start, _) in line.match_indices(&config.marker) {
        if is_inside_inline_code(line, marker_start) {
            continue;
        }
        let token_start = marker_start + config.marker.len();
        let Some(rest) = line.get(token_start..) else {
            continue;
        };
        let Some(prefix) = QUALIFIED_CITATION_PREFIX.captures(rest) else {
            continue;
        };
        let Some(alias) = prefix.name("namespace").map(|m| m.as_str()) else {
            continue;
        };
        let Some(target_project) = workspace.project_by_alias(alias) else {
            continue;
        };
        let target_index = shorthand_targets.by_alias.get(alias);
        let id_start = token_start + prefix.get(0).unwrap().end();
        let Some(id_rest) = line.get(id_start..) else {
            continue;
        };
        let parsed_prefix = parse_longest_id_prefix(id_rest, &target_project.config.grammar)
            .filter(|parsed| {
                !target_project
                    .config
                    .grammar
                    .has_reserved_named_tail(id_rest, parsed.len)
            });
        // §FS-workspace.8.5 / §FS-fmt.6.2: resolve a parsed shorthand through the
        // target's policy/index before lookup; its slugless parse cannot name a home.
        // Full IDs have already won longest-prefix parsing and remain byte-identical.
        let parsed = match parsed_prefix {
            Some(parsed) if parsed.shorthand => accepted_shorthand_link(
                id_rest,
                config,
                &target_project.config,
                target_index.map(|target| &target.index),
            ),
            Some(parsed) => Some((parsed.id, parsed.section, parsed.len)),
            None => {
                let catalog = legacy_catalog_ids(&target_project.findings.declarations);
                match_legacy_tail(id_rest, &target_project.config, &catalog).or_else(|| {
                    accepted_shorthand_link(
                        id_rest,
                        config,
                        &target_project.config,
                        target_index.map(|target| &target.index),
                    )
                })
            }
        };
        let Some((id, section, len)) = parsed else {
            continue;
        };
        out.push(MarkdownLineCitation {
            marker_start,
            token_end: id_start + len,
            namespace: Some(alias.to_string()),
            id,
            section,
        });
    }
}
