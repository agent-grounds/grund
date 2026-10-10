//! The marker-qualified citation passes (§FS-workspace.1.2, §AR-scanner.2.3):
//! a `§<alias>/<ID>` token read with the target project's grammar when the run
//! loaded that project, and with the fallback shape when it did not
//! (§FS-workspace.1.2.1, §FS-workspace.5.2).

use std::collections::BTreeSet;

use super::citation_line::CitationLine;
use super::citations::{glob_claimed, record_glob_citation, scanned_citation_rewritable};
use crate::grammar::{
    CandidateReading, QUALIFIED_CITATION_PREFIX, parse_longest_id_prefix,
    parse_qualified_id_prefix, qualified_suppressed_in_source, read_loose_candidate,
};
use crate::model::{Catalog, Citation};
use crate::workspace::WorkspaceCitationTarget;

/// §FS-workspace.5.2: a member-local scan must still recognize marker-qualified
/// citations before the member's own ID grammar is applied. Without this
/// fallback, `§root/FS-root` in a default member can disappear just because the
/// root uses `{kind}-{slug}`.
///
/// The fallback reads the ID tail with the conventional `KIND[-NUM]-SLUG`
/// shape, not the citing or any target project's configured `[id] format`:
/// member-local scans have no workspace catalogue, so the target's grammar is
/// unreachable here. That shape decides how the tail is *read* and never
/// whether the citation is *recognized* (§FS-workspace.5.2) — the unknown
/// thing is the alias, which needs no tail grammar, so a tail outside the
/// shape (lowercase kinds, slug-only grammars that don't split on `-`/`_`,
/// kinds with characters outside `[A-Z0-9]`) is reported here rather than
/// falling through to the workspace-root run. Workspace-root and
/// workspace-aware paths use the target's actual grammar via
/// `scan_workspace_qualified_pass`, which is the one place the tail itself is
/// judged — for a known alias; an unknown one has no target there either, and
/// is read with this same shape (§FS-workspace.1.2.1).
///
/// `qualified_claimed` carries the marker offsets a qualified citation already
/// exists at — the full-ID pass's on entry, this pass's own on return. The
/// shorthand pass reads the union to decide whether a qualified marker is
/// already spoken for (§AR-scanner.2.6.1.1).
pub(super) fn scan_fallback_qualified_citations(
    line: &CitationLine<'_>,
    qualified_claimed: &mut BTreeSet<usize>,
    findings: &mut Catalog,
) {
    if line.schema.citation.marker.is_empty() {
        return;
    }
    for (marker_start, _) in line.scan_line.match_indices(&line.schema.citation.marker) {
        if qualified_claimed.contains(&marker_start) || glob_claimed(line, marker_start, findings) {
            continue;
        }
        if qualified_suppressed_in_source(line.scan_line, line.is_md, marker_start) {
            continue;
        }
        let token_start = marker_start + line.schema.citation.marker.len();
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
        if push_fallback_qualified_citation(line, marker_start, alias, id_start, findings) {
            qualified_claimed.insert(marker_start);
        }
    }
}

/// Push the qualified citation at `marker_start` whose tail, starting at
/// `id_start`, is read with the loose `KIND[-NUM]-SLUG` shape rather than any
/// project's grammar. Both runs that have no target to read a tail with use it:
/// a run that loads no workspace (§FS-workspace.5.2), and a workspace run whose
/// alias names no loaded project (§FS-workspace.1.2.1). Returns whether the
/// marker was claimed — by a citation, or by a pattern read whole.
fn push_fallback_qualified_citation(
    line: &CitationLine<'_>,
    marker_start: usize,
    alias: &str,
    id_start: usize,
    findings: &mut Catalog,
) -> bool {
    let Some(id_rest) = line.scan_line.get(id_start..) else {
        return false;
    };
    // §FS-workspace.5.2: the tail grammar does not decide recognition — a
    // tail outside `KIND[-NUM]-SLUG` is an `unknown project alias` error
    // all the same, at its own site.
    let parsed = parse_qualified_id_prefix(id_rest);
    // §FS-check.1.1.11: a pattern is consumed whole, never read as its prefix.
    let prefix_len = parsed.as_ref().map_or(0, |(_, _, len)| *len);
    if let CandidateReading::Pattern(len) = read_loose_candidate(id_rest, prefix_len) {
        record_glob_citation(line, marker_start..id_start + len, findings);
        return true;
    }
    let Some((id, section, id_len)) = parsed else {
        return false;
    };
    let token_end = id_start + id_len;
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
        local_section: false,
        shorthand_rewritable: true,
        numeric_run: false,
        text: line.scan_line[marker_start..token_end].to_string(),
        inline_site: line.inline_sites.get(&line.lineno).cloned(),
        // §AR-scanner.2.4: classified in the post-pass in `scan_file`.
        source_kind: String::new(),
        enclosing_declaration: None,
        enclosing_section: None,
    });
    true
}

/// One line's worth of marker-qualified workspace citations: a `§<alias>/<ID>`
/// token whose ID tail parses with the target project's grammar
/// (§FS-workspace.1.2, §AR-workspace.2), or whose alias names no loaded project
/// and whose tail is read with the fallback shape (§FS-workspace.1.2.1). Runs
/// inline during `scan_file` in workspace mode so the file is read once, not twice.
pub(super) fn scan_workspace_qualified_pass(
    line: &CitationLine<'_>,
    targets: &[WorkspaceCitationTarget],
    findings: &mut Catalog,
) {
    if line.schema.citation.marker.is_empty() || targets.is_empty() {
        return;
    }
    for (marker_start, _) in line.scan_line.match_indices(&line.schema.citation.marker) {
        if qualified_suppressed_in_source(line.scan_line, line.is_md, marker_start) {
            continue;
        }
        let token_start = marker_start + line.schema.citation.marker.len();
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
        // §FS-workspace.1.2.1: an alias naming no loaded project has no target, so
        // no loaded grammar reads its tail — the fallback shape does, whatever kind
        // it names, and the resolver reports the alias at this site.
        let Some(target) = targets.iter().find(|target| target.alias == alias) else {
            push_fallback_qualified_citation(line, marker_start, alias, id_start, findings);
            continue;
        };
        let Some(id_rest) = line.scan_line.get(id_start..) else {
            continue;
        };
        // §FS-fmt.2.4.1 asks the numeric-run question with the *target's* number
        // shape, the same grammar that claimed the token.
        let target_grammar = &target.compiled.grammar;
        let parsed = parse_longest_id_prefix(id_rest, target_grammar);
        // §FS-check.1.1.11: the target's grammar reads the tail whole, so a
        // pattern is recorded at the marker and no prefix becomes an edge.
        let prefix_len = parsed.as_ref().map_or(0, |parsed| parsed.len);
        let parsed = match target_grammar.read_candidate(id_rest, prefix_len) {
            CandidateReading::Ordinary => parsed,
            CandidateReading::Address(len) => {
                parse_longest_id_prefix(&id_rest[..len], target_grammar)
                    .filter(|whole| whole.len == len)
            }
            CandidateReading::Pattern(len) => {
                record_glob_citation(line, marker_start..id_start + len, findings);
                continue;
            }
        };
        let Some(parsed) = parsed else {
            continue;
        };
        if target_grammar.has_reserved_named_tail(id_rest, parsed.len) {
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
            local_section: false,
            // §FS-fmt.2.4.1: the marker is the citing project's, the number shape
            // the target's — the same split the rewrite itself uses.
            numeric_run: parsed.shorthand
                && target_grammar.shorthand_sits_in_numeric_run(
                    &line.schema.citation.marker,
                    id_rest,
                    parsed.len,
                ),
            text: line.scan_line[marker_start..token_end].to_string(),
            inline_site: line.inline_sites.get(&line.lineno).cloned(),
            // §AR-scanner.2.4: classified in the post-pass in `scan_file`.
            source_kind: String::new(),
            enclosing_declaration: None,
            enclosing_section: None,
        });
    }
}
