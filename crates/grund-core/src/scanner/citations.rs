use std::collections::BTreeSet;

use super::citation_line::CitationLine;
use crate::grammar::{
    CandidateReading, QUALIFIED_CITATION_PREFIX, never_rewrite_context_in, parse_id,
    parse_longest_id_prefix, qualified_suppressed_in_source,
};
use crate::model::{
    Catalog, Citation, GlobCitation, LegacyCitationCandidate, LocalSectionCitationCandidate,
};

/// Whether `fmt` may rewrite the citation whose marker starts at `marker_start` —
/// a **`scan_line`** offset, which is what every pass below holds — on the line
/// they are scanning (§FS-fmt.2.3, §FS-check.3.13.1). One place asks it, so the
/// qualified pass, the unqualified one and the shorthand pass can never reach
/// different verdicts about one site; the recorded column stays a raw-file column
/// either way (§AR-scanner.2.6.10).
pub(super) fn scanned_citation_rewritable(line: &CitationLine<'_>, marker_start: usize) -> bool {
    !never_rewrite_context_in(
        line.docstring,
        line.raw_line,
        line.is_md,
        line.column_offset + marker_start,
    )
}

/// Retain one whole configured-marker-plus-digit token until declaration body
/// spans are available (§FS-check.1.1.8, §AR-scanner.2.3). Full citations and
/// number-only ID shorthand have already claimed their marker offsets. A
/// digit-starting tail is kept whole for the unsupported-form verdict rather
/// than truncating a plausible numeric prefix into an edge.
pub(super) fn scan_local_section_candidates(
    line: &CitationLine<'_>,
    claimed_markers: &[usize],
    qualified_claimed: &BTreeSet<usize>,
    findings: &mut Catalog,
) {
    if line.schema.citation.marker.is_empty() {
        return;
    }
    for (marker_start, _) in line.scan_line.match_indices(&line.schema.citation.marker) {
        if claimed_markers.contains(&marker_start) || qualified_claimed.contains(&marker_start) {
            continue;
        }
        let column = line.column_offset + marker_start + 1;
        if findings.citations.iter().any(|citation| {
            citation.file == line.path && citation.line == line.lineno && citation.column == column
        }) {
            // The shorthand pass runs first and may itself begin with a digit
            // under a configured ID format; it owns that marker just as the
            // full-ID pass does (§FS-check.1.1.8).
            continue;
        }
        let token_start = marker_start + line.schema.citation.marker.len();
        let Some(rest) = line.scan_line.get(token_start..) else {
            continue;
        };
        if !rest.starts_with(|ch: char| ch.is_ascii_digit()) {
            continue;
        }
        // Scan the maximal token before deciding whether its dotted components
        // are supported (§FS-check.1.1.8). This keeps empty components in
        // `<§>2..1` and `<§>2...` from being truncated into an edge to section 2.
        let mut token_len = rest
            .char_indices()
            .find_map(|(offset, ch)| {
                (!(ch.is_alphanumeric() || matches!(ch, '.' | '_' | '-'))).then_some(offset)
            })
            .unwrap_or(rest.len());
        let token = &rest[..token_len];
        let trailing_dots = token.bytes().rev().take_while(|byte| *byte == b'.').count();
        // One terminal dot is sentence punctuation. A repeated terminal run is
        // itself malformed dotted syntax and stays inside the whole-token
        // unsupported verdict (§FS-check.3.24).
        if trailing_dots == 1 {
            token_len -= 1;
        }
        let tail = &rest[..token_len];
        let supported = tail
            .split('.')
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()));
        findings
            .local_section_citation_candidates
            .push(LocalSectionCitationCandidate {
                text: line.scan_line[marker_start..token_start + token_len].to_string(),
                section: supported.then(|| tail.to_string()),
                file: line.path.to_path_buf(),
                line: line.lineno,
                column,
                rewritable: scanned_citation_rewritable(line, marker_start),
                inline_site: line.inline_sites.get(&line.lineno).cloned(),
                source_kind: String::new(),
                enclosing_declaration: None,
                enclosing_section: None,
            });
    }
}

/// Retain marker-prefixed tokens that the configured grammar may have rejected;
/// catalog reconciliation promotes only exact declaration-backed spellings
/// (§FS-check.1.1.1, §FS-config.3.2.6). The remainder of the already-read line is
/// enough to defer token/section precedence without a second file read.
pub(super) fn scan_legacy_citation_candidates(line: &CitationLine<'_>, findings: &mut Catalog) {
    if line.schema.citation.marker.is_empty()
        || !line.scan_line.contains(&line.schema.citation.marker)
    {
        return;
    }
    for (marker_start, _) in line.scan_line.match_indices(&line.schema.citation.marker) {
        let column = line.column_offset + marker_start + 1;
        // §FS-check.1.1.11: a pattern's marker yields no catalog-compatible prefix either.
        if glob_claimed(line, marker_start, findings) {
            continue;
        }
        let token_start = marker_start + line.schema.citation.marker.len();
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
                enclosing_section: None,
            });
    }
}

/// §FS-check.1.1.11 / §AR-scanner.2.3.6: the line's full-ID captures, after
/// recording each marked candidate they read as a pattern — at its marker's
/// column and as written after the marker, for §FS-check.checks.glob-citation —
/// and claiming its marker, so no later pass on the line reads a prefix there.
/// Outside a workspace this grammar also reads a marked qualified token, so a
/// pattern after its `<alias>/` yields no prefix capture either; the fallback
/// pass reports its alias, which no run without a workspace knows. In a
/// workspace the target's grammar reads the tail in the qualified pass.
pub(super) fn claim_citation_tokens<'a>(
    line: &CitationLine<'a>,
    workspace_mode: bool,
    claimed_markers: &mut Vec<usize>,
    findings: &mut Catalog,
) -> Vec<(usize, regex::Captures<'a>)> {
    let marker = line.schema.citation.marker.as_str();
    let grammar = line.frame.grammar();
    let mut tokens = grammar.citation_captures(line.scan_line, marker);
    for span in &tokens.patterns {
        claimed_markers.push(span.start);
        record_glob_citation(line, span.clone(), findings);
    }
    if !workspace_mode {
        tokens.captures.retain(|(offset, caps)| {
            let (Some(full), Some(alias)) = (caps.get(0), caps.name("namespace")) else {
                return true;
            };
            let token_start = offset + full.start();
            let Some(marker_start) = token_start.checked_sub(marker.len()) else {
                return true;
            };
            if !line.scan_line[..token_start].ends_with(marker)
                || qualified_suppressed_in_source(line.scan_line, line.is_md, marker_start)
            {
                return true;
            }
            let tail_start = offset + alias.end() + 1;
            let prefix_len = offset + full.end() - tail_start;
            if !matches!(
                grammar.read_candidate(&line.scan_line[tail_start..], prefix_len),
                CandidateReading::Pattern(_)
            ) {
                return true;
            }
            // Outside a workspace every alias is unknown, so the fallback pass reports
            // the alias for the whole token rather than a pattern (§FS-check.3.8).
            claimed_markers.push(marker_start);
            false
        });
    }
    tokens.captures
}

/// Whether a pattern was already recorded at the marker at `marker_start`
/// (§FS-check.1.1.11), so a later pass on the line reads no prefix there.
pub(super) fn glob_claimed(
    line: &CitationLine<'_>,
    marker_start: usize,
    findings: &Catalog,
) -> bool {
    let column = line.column_offset + marker_start + 1;
    findings.glob_citations.iter().any(|pattern| {
        pattern.line == line.lineno && pattern.column == column && pattern.file == line.path
    })
}

/// §FS-check.checks.glob-citation: record the pattern spanning `span`, from its
/// marker to its end, at the marker's column and as written after the marker.
pub(super) fn record_glob_citation(
    line: &CitationLine<'_>,
    span: std::ops::Range<usize>,
    findings: &mut Catalog,
) {
    let marker = line.schema.citation.marker.as_str();
    findings.glob_citations.push(GlobCitation {
        file: line.path.to_path_buf(),
        line: line.lineno,
        column: line.column_offset + span.start + 1,
        token: line.scan_line[span.start + marker.len()..span.end].to_string(),
    });
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
pub(super) fn scan_escaped_citations(line: &CitationLine<'_>, findings: &mut Catalog) {
    if line.schema.citation.marker.is_empty() {
        return;
    }
    let escape = format!("<{}>", line.schema.citation.marker);
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
        let Some(parsed) = parse_longest_id_prefix(id_rest, line.frame.grammar()) else {
            continue;
        };
        // §FS-check.1.1.11: an escaped pattern illustrates a pattern, never its prefix.
        if let CandidateReading::Pattern(_) =
            line.frame.grammar().read_candidate(id_rest, parsed.len)
        {
            continue;
        }
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
            local_section: false,
            // An escape is check-inert; nothing rewrites it (§AR-scanner.2.5), so
            // the run question — which only ever gates a rewrite — never arises.
            shorthand_rewritable: false,
            numeric_run: false,
            text: line.scan_line[escape_start..token_end].to_string(),
            inline_site: None,
            source_kind: String::new(),
            enclosing_declaration: None,
            enclosing_section: None,
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
    findings: &mut Catalog,
) {
    if line.schema.citation.marker.is_empty() {
        return;
    }
    for (marker_start, _) in line.scan_line.match_indices(&line.schema.citation.marker) {
        // §DF-number-only-citation-shorthand.2.6: the full-ID pass owns every token
        // it can claim, and `claimed_markers` is the record of what it claimed on
        // this line — tested before the regex (§GOAL-fast-feedback).
        if claimed_markers.contains(&marker_start) {
            continue;
        }
        let token_start = marker_start + line.schema.citation.marker.len();
        let Some(rest) = line.scan_line.get(token_start..) else {
            continue;
        };
        let Some(shorthand) = line.frame.grammar().shorthand_for(rest) else {
            continue;
        };
        let Some(caps) = shorthand.prefix_re().captures(rest) else {
            continue;
        };
        let match_end = caps.get(0).map_or(0, |found| found.end());
        if line
            .frame
            .grammar()
            .has_reserved_named_tail(rest, match_end)
        {
            continue;
        }
        // §DF-number-only-citation-shorthand.2.6: the pattern is anchored only at
        // the start, so without this the `FS-042` inside the rejected full ID
        // `§FS-042-User-Login` would be reported as a token the file does not hold.
        if !line.frame.grammar().id_token_ends_cleanly(rest, match_end) {
            continue;
        }
        // §FS-fmt.2.4.1.1: the token ended, which does not make it a citation.
        let numeric_run = line.frame.grammar().shorthand_sits_in_numeric_run(
            &line.schema.citation.marker,
            rest,
            match_end,
        );
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
        let Some(id) = parse_id(&caps, line.frame.grammar()) else {
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
            local_section: false,
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
            enclosing_section: None,
        });
    }
}
