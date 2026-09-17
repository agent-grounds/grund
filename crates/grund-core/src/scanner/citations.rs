use std::collections::BTreeSet;

use super::file_pass::CitationLine;
use crate::grammar::{
    QUALIFIED_CITATION_PREFIX, parse_longest_id_prefix, parse_loose_qualified_id_prefix,
    qualified_suppressed_in_source, scanned_citation_rewritable,
};
use crate::model::{Citation, Findings, LegacyCitationCandidate};
use crate::workspace::WorkspaceCitationTarget;

/// §FS-workspace.5: a member-local scan must still recognize marker-qualified
/// citations before the member's own ID grammar is applied. Without this
/// fallback, `§root/FS-root` in a default member can disappear just because the
/// root uses `{kind}-{slug}`.
///
/// The fallback parses the ID tail with the conventional `KIND[-NUM]-SLUG`
/// shape (`parse_loose_qualified_id_prefix`), not the citing or any target
/// project's configured `[id] format`. Member-local scans have no workspace
/// catalogue, so the target's grammar is unreachable here. The tradeoff:
/// non-default ID grammars (lowercase kinds, slug-only shapes that don't
/// separate on `-`/`_`, kinds with characters outside `[A-Z0-9]`) won't be
/// recognised as qualified citations at member scope and will fall through
/// to be diagnosed at the workspace-root run instead. Workspace-root and
/// workspace-aware paths use the target's actual grammar via
/// `scan_workspace_qualified_pass`.
///
/// `qualified_claimed` carries the marker offsets a qualified citation already
/// exists at — the full-ID pass's on entry, this pass's own on return. The
/// shorthand pass reads the union to decide whether a qualified marker is
/// already spoken for (§AR-scanner.2.6).
pub(super) fn scan_fallback_qualified_citations(
    line: &CitationLine<'_>,
    qualified_claimed: &mut BTreeSet<usize>,
    findings: &mut Findings,
) {
    if line.config.marker.is_empty() {
        return;
    }
    for (marker_start, _) in line.scan_line.match_indices(&line.config.marker) {
        if qualified_claimed.contains(&marker_start) {
            continue;
        }
        if qualified_suppressed_in_source(line.scan_line, line.is_md, marker_start) {
            continue;
        }
        let token_start = marker_start + line.config.marker.len();
        let Some(rest) = line.scan_line.get(token_start..) else {
            continue;
        };
        let Some(prefix) = QUALIFIED_CITATION_PREFIX.captures(rest) else {
            continue;
        };
        let Some(alias) = prefix.name("namespace").map(|m| m.as_str()) else {
            continue;
        };
        let id_start = token_start + prefix.get(0).unwrap().end();
        let Some(id_rest) = line.scan_line.get(id_start..) else {
            continue;
        };
        let Some((id, section, id_len)) = parse_loose_qualified_id_prefix(id_rest) else {
            continue;
        };
        let token_end = id_start + id_len;
        qualified_claimed.insert(marker_start);
        findings.citations.push(Citation {
            namespace: Some(alias.to_string()),
            id,
            section,
            file: line.path.to_path_buf(),
            line: line.lineno,
            column: line.column_offset + marker_start + 1,
            has_marker: true,
            // The loose parser has no target grammar to derive a shorthand from,
            // so a fallback-parsed qualified citation is never one (§AR-scanner.2.6).
            shorthand: false,
            shorthand_rewritable: true,
            numeric_run: false,
            text: line.scan_line[marker_start..token_end].to_string(),
            inline_site: line.inline_sites.get(&line.lineno).cloned(),
            // §AR-scanner.2.4: classified in the post-pass in `scan_file`.
            source_kind: String::new(),
            enclosing_declaration: None,
        });
    }
}

/// One line's worth of marker-qualified workspace citations: a `§<alias>/<ID>`
/// token whose ID tail parses with the target project's grammar
/// (§FS-workspace.1, §AR-workspace.2). Runs inline during `scan_file` in
/// workspace mode so the file is read once, not twice.
pub(super) fn scan_workspace_qualified_pass(
    line: &CitationLine<'_>,
    targets: &[WorkspaceCitationTarget],
    findings: &mut Findings,
) {
    if line.config.marker.is_empty() || targets.is_empty() {
        return;
    }
    for (marker_start, _) in line.scan_line.match_indices(&line.config.marker) {
        if qualified_suppressed_in_source(line.scan_line, line.is_md, marker_start) {
            continue;
        }
        let token_start = marker_start + line.config.marker.len();
        let Some(rest) = line.scan_line.get(token_start..) else {
            continue;
        };
        let Some(prefix) = QUALIFIED_CITATION_PREFIX.captures(rest) else {
            continue;
        };
        let Some(alias) = prefix.name("namespace").map(|m| m.as_str()) else {
            continue;
        };
        let id_start = token_start + prefix.get(0).unwrap().end();
        let Some(id_rest) = line.scan_line.get(id_start..) else {
            continue;
        };
        // The winning target is kept, not just its parse: §FS-fmt.2.4.1 asks the
        // numeric-run question with the *target's* number shape, the same grammar
        // that claimed the token.
        let parsed = match targets.iter().find(|target| target.alias == alias) {
            Some(target) => parse_longest_id_prefix(id_rest, &target.config.grammar)
                .map(|parsed| (parsed, &target.config)),
            None => targets.iter().find_map(|target| {
                parse_longest_id_prefix(id_rest, &target.config.grammar)
                    .map(|parsed| (parsed, &target.config))
            }),
        };
        let Some((parsed, target_config)) = parsed else {
            continue;
        };
        if target_config
            .grammar
            .has_reserved_named_tail(id_rest, parsed.len)
        {
            continue;
        }
        let token_end = id_start + parsed.len;
        findings.citations.push(Citation {
            namespace: Some(alias.to_string()),
            id: parsed.id,
            section: parsed.section,
            file: line.path.to_path_buf(),
            line: line.lineno,
            column: line.column_offset + marker_start + 1,
            has_marker: true,
            // §FS-fmt.2.3 / §FS-check.3.13: a qualified shorthand is rewritable
            // wherever an unqualified one is — the workspace pass reaches the aliased
            // project's declarations, so `fmt` can name the canonical form here too.
            shorthand_rewritable: scanned_citation_rewritable(line, marker_start),
            shorthand: parsed.shorthand,
            // §FS-fmt.2.4.1: the marker is the citing project's, the number shape
            // the target's — the same split the rewrite itself uses.
            numeric_run: parsed.shorthand
                && target_config.grammar.shorthand_sits_in_numeric_run(
                    &line.config.marker,
                    id_rest,
                    parsed.len,
                ),
            text: line.scan_line[marker_start..token_end].to_string(),
            inline_site: line.inline_sites.get(&line.lineno).cloned(),
            // §AR-scanner.2.4: classified in the post-pass in `scan_file`.
            source_kind: String::new(),
            enclosing_declaration: None,
        });
    }
}

/// Retain marker-prefixed tokens that the configured grammar may have rejected;
/// catalog reconciliation promotes only exact declaration-backed spellings
/// (§FS-check.1.1, §FS-config.3.2). The remainder of the already-read line is
/// enough to defer token/section precedence without a second file read.
pub(super) fn scan_legacy_citation_candidates(line: &CitationLine<'_>, findings: &mut Findings) {
    if line.config.marker.is_empty() || !line.scan_line.contains(&line.config.marker) {
        return;
    }
    for (marker_start, _) in line.scan_line.match_indices(&line.config.marker) {
        let column = line.column_offset + marker_start + 1;
        let token_start = marker_start + line.config.marker.len();
        let Some(rest) = line.scan_line.get(token_start..) else {
            continue;
        };
        let (namespace, tail) = match QUALIFIED_CITATION_PREFIX.captures(rest) {
            Some(prefix) => {
                if qualified_suppressed_in_source(line.scan_line, line.is_md, marker_start) {
                    continue;
                }
                let Some(alias) = prefix.name("namespace") else {
                    continue;
                };
                let end = prefix.get(0).unwrap().end();
                (Some(alias.as_str().to_string()), &rest[end..])
            }
            None => (None, rest),
        };
        findings
            .legacy_citation_candidates
            .push(LegacyCitationCandidate {
                namespace,
                tail: tail.to_string(),
                file: line.path.to_path_buf(),
                line: line.lineno,
                column,
                inline_site: line.inline_sites.get(&line.lineno).cloned(),
                inline_block_lines: line.inline_block_lines.get(&line.lineno).cloned(),
                source_kind: String::new(),
                enclosing_declaration: None,
            });
    }
}

/// §AR-scanner.2.5: collect `<§>`-escaped citation illustrations. The literal
/// `<§>[alias/]ID[.section]` is deliberately *not* a live citation — the `<` and
/// `>` around the marker mean `§` is not immediately followed by an ID, so no
/// detection pass matches it (§AR-workspace.3.1). We record the shape anyway,
/// into a check-inert list, so the checker can flag one whose ID resolves to a
/// real declaration (§FS-check.4.2): an escape of a *real* ID is a likely
/// bracketed live citation, not an intended illustration. IDs are parsed with
/// the citing project's grammar; a cross-namespace target with an exotic grammar
/// may be missed, which only ever costs a suggestion, never a false error.
pub(super) fn scan_escaped_citations(line: &CitationLine<'_>, findings: &mut Findings) {
    if line.config.marker.is_empty() {
        return;
    }
    let escape = format!("<{}>", line.config.marker);
    if !line.scan_line.contains(&escape) {
        return;
    }
    for (escape_start, _) in line.scan_line.match_indices(&escape) {
        let token_start = escape_start + escape.len();
        let Some(rest) = line.scan_line.get(token_start..) else {
            continue;
        };
        let (namespace, id_rest, alias_len) =
            match QUALIFIED_CITATION_PREFIX.captures(rest).and_then(|p| {
                p.name("namespace")
                    .map(|m| (m.as_str().to_string(), p.get(0).unwrap().end()))
            }) {
                Some((alias, alias_len)) => {
                    let Some(id_rest) = rest.get(alias_len..) else {
                        continue;
                    };
                    (Some(alias), id_rest, alias_len)
                }
                None => (None, rest, 0),
            };
        let Some(parsed) = parse_longest_id_prefix(id_rest, &line.config.grammar) else {
            continue;
        };
        let token_end = token_start + alias_len + parsed.len;
        findings.escaped_citations.push(Citation {
            namespace,
            id: parsed.id,
            section: parsed.section,
            file: line.path.to_path_buf(),
            line: line.lineno,
            column: line.column_offset + escape_start + 1,
            has_marker: false,
            shorthand: parsed.shorthand,
            // An escape is check-inert; nothing rewrites it (§AR-scanner.2.5), so
            // the run question — which only ever gates a rewrite — never arises.
            shorthand_rewritable: false,
            numeric_run: false,
            text: line.scan_line[escape_start..token_end].to_string(),
            inline_site: None,
            source_kind: String::new(),
            enclosing_declaration: None,
        });
    }
}
