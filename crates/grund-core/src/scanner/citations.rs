use std::collections::BTreeSet;

use super::file_pass::CitationLine;
use crate::grammar::{
    QUALIFIED_CITATION_PREFIX, never_rewrite_context_in, parse_id, parse_longest_id_prefix,
    parse_loose_qualified_id_prefix, qualified_suppressed_in_source,
};
use crate::model::{Citation, Findings, LegacyCitationCandidate};
use crate::workspace::WorkspaceCitationTarget;

/// Whether `fmt` may rewrite the citation whose marker starts at `marker_start` —
/// a **`scan_line`** offset, which is what every pass below holds — on the line
/// they are scanning (§FS-fmt.2.3, §FS-check.3.13.1). One place asks it, so the
/// qualified pass, the unqualified one and the shorthand pass can never reach
/// different verdicts about one site; the recorded column stays a raw-file column
/// either way (§AR-scanner.2.6.10).
fn scanned_citation_rewritable(line: &CitationLine<'_>, marker_start: usize) -> bool {
    !never_rewrite_context_in(
        line.docstring,
        line.raw_line,
        line.is_md,
        line.column_offset + marker_start,
    )
}

/// §FS-workspace.5.2: a member-local scan must still recognize marker-qualified
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
/// already spoken for (§AR-scanner.2.6.1.1).
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
/// (§FS-workspace.1.2, §AR-workspace.2). Runs inline during `scan_file` in
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
            // §FS-fmt.2.3 / §FS-check.3.13.1: a qualified shorthand is rewritable
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
/// (§FS-check.1.1.1, §FS-config.3.2.6). The remainder of the already-read line is
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

/// §AR-scanner.2.6: collect number-only shorthand citations — `§FS-042` for
/// `§FS-042-user-login` — under a `[id] format` that carries both `{number}` and
/// `{slug}`. Three gates in order, each cheap enough to run per line: the repo
/// must have a shorthand at all, the line must contain the marker, and the token
/// must not already be claimed by the full-ID pass (§DF-number-only-citation-shorthand.2.6).
///
/// The marker is required unconditionally — there is no bare branch even under
/// `strict = false`, because `KIND-NNN` carries no slug to make an accidental
/// match unlikely and occurs constantly as issue keys and part numbers
/// (§DF-number-only-citation-shorthand.2.4).
///
/// Testing `claimed_markers` before the regex is also what keeps the pass cheap on
/// a well-formed tree, where every marker is claimed and no shorthand pattern is
/// ever run.
///
/// A qualified marker belongs to the pass that claimed it — the workspace one,
/// which claims every `§<alias>/...` on the line, or the loose fallback outside it,
/// which records each token it parsed. Without the record the shorthand pattern
/// matched the same token a second time and it became two identical citations: a
/// duplicated row in `cover` and a diagnostic `check` printed twice. Skipping
/// unconditionally instead would delete the citation wherever the loose parser
/// declines a shape this project's `[id] format` accepts. The qualified form also
/// collides with a path, so a marked qualified token inside inline code or a string
/// literal is not a citation at all — the same carve-out the other passes apply.
///
/// Qualified `§<alias>/FS-042` is left to the workspace pass, which parses the ID
/// tail with the *target* project's grammar — the citing project's shorthand
/// shape would be the wrong one to apply across a namespace boundary. Outside
/// workspace mode there is no such pass to defer to unconditionally, so the
/// deferral is by record: `qualified_claimed` holds the markers a qualified pass
/// actually emitted at, and only those are skipped.
pub(super) fn scan_shorthand_citations(
    line: &CitationLine<'_>,
    workspace_mode: bool,
    claimed_markers: &[usize],
    qualified_claimed: &BTreeSet<usize>,
    findings: &mut Findings,
) {
    if line.config.marker.is_empty() {
        return;
    }
    for (marker_start, _) in line.scan_line.match_indices(&line.config.marker) {
        // §DF-number-only-citation-shorthand.2.6: the full-ID pass owns every token
        // it can claim, and `claimed_markers` is the record of what it claimed on
        // this line — tested before the regex (§GOAL-fast-feedback).
        if claimed_markers.contains(&marker_start) {
            continue;
        }
        let token_start = marker_start + line.config.marker.len();
        let Some(rest) = line.scan_line.get(token_start..) else {
            continue;
        };
        let Some(shorthand) = line.config.grammar.shorthand_for(rest) else {
            continue;
        };
        let Some(caps) = shorthand.prefix_re().captures(rest) else {
            continue;
        };
        let match_end = caps.get(0).map_or(0, |found| found.end());
        if line.config.grammar.has_reserved_named_tail(rest, match_end) {
            continue;
        }
        // §DF-number-only-citation-shorthand.2.6: the pattern is anchored only at
        // the start, so without this the `FS-042` inside the rejected full ID
        // `§FS-042-User-Login` would be reported as a token the file does not hold.
        if !line.config.grammar.id_token_ends_cleanly(rest, match_end) {
            continue;
        }
        // §FS-fmt.2.4.1.1: the token ended, which does not make it a citation.
        let numeric_run =
            line.config
                .grammar
                .shorthand_sits_in_numeric_run(&line.config.marker, rest, match_end);
        // §AR-scanner.2.6.1.1: a qualified marker a qualified pass already claimed —
        // the workspace one, or the loose fallback (§FS-workspace.5.2) — belongs to
        // that pass alone (§REQ-no-missed-citation.1, §AR-scanner.2.3.2).
        let namespace = caps.name("namespace").map(|m| m.as_str().to_string());
        if namespace.is_some()
            && (workspace_mode
                || qualified_claimed.contains(&marker_start)
                || qualified_suppressed_in_source(line.scan_line, line.is_md, marker_start))
        {
            continue;
        }
        let Some(id) = parse_id(&caps, &line.config.grammar) else {
            continue;
        };
        let token_end = token_start + match_end;
        findings.citations.push(Citation {
            namespace,
            id,
            section: caps.name("sec").map(|m| m.as_str().to_string()),
            file: line.path.to_path_buf(),
            line: line.lineno,
            column: line.column_offset + marker_start + 1,
            has_marker: true,
            shorthand: true,
            // §FS-check.3.13.1: still a citation here — it resolves, it counts, it
            // grounds its file — but `fmt` may not rewrite it (§FS-fmt.2.3), so the
            // checker withholds the "write the canonical form" error.
            shorthand_rewritable: scanned_citation_rewritable(line, marker_start),
            numeric_run,
            text: line.scan_line[marker_start..token_end].to_string(),
            inline_site: line.inline_sites.get(&line.lineno).cloned(),
            // §AR-scanner.2.4: classified in the post-pass in `scan_file`.
            source_kind: String::new(),
            enclosing_declaration: None,
        });
    }
}
