//! The per-file scan (§AR-scanner.2): one line-by-line pass over one file's
//! text, producing every declaration, section, citation and value candidate in
//! it. Named for what it is rather than for the component, because the component
//! is the directory now (§AR-core-module-layout.1): `walk.rs` is the directory
//! traversal of §AR-scanner.1 and this is the state machine it hands each file
//! to, and the two meet only at the file list one passes the other.
//!
//! The eight pieces of mutable state advance together per line, which is why the
//! loop is not broken apart; what is not that state has left for a sibling, and
//! `docs/file-size-human-exceptions.toml` records what is still to go.

use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::Arc;

use super::after_pass::AfterPass;
use super::citation_line::CitationLine;
use super::citations::{
    claim_citation_tokens, scan_escaped_citations, scan_legacy_citation_candidates,
    scan_local_section_candidates, scan_shorthand_citations,
};
use super::conflict_blocks::MarkerLines;
use super::context::{
    assign_declaration_bodies, in_some_home, inline_citation_sites, markdown_heading_level,
    resolve_citation_owners, retain_in_body_sections,
};
use super::declaration_line::{close_declaration, stub_and_title};
use super::embedded_value_context::{
    EMBEDDED_VALUE_MARKER, MISPLACED_MARKER, authored_heading_level,
    embedded_value_marker_for_line, invalid_embedded_marker, push_invalid_embedded_marker,
};
use super::extents::assign_extents;
use super::qualified_citations::{
    scan_fallback_qualified_citations, scan_workspace_qualified_pass,
};
use super::record::{CommentFences, HeadingLine, bare_block_line, declaration_eligible, zone_of};
use super::relative_sections::HeldHeadings;
use super::section_record::record_section_heading;
use super::tree::heading_level_for_line;
use super::units::record_file_structure;
use super::unmarked_headings::assign_unmarked_heading_owners;
use super::value_binding_split::scan_split_binding;
use super::value_context::recognized_source_value_contexts;
use super::values::scan_value_bindings;
use crate::config::{Frame, Schema};
use crate::grammar::{
    DocstringContent, MarkdownBlocks, MarkdownLine, MarkdownLinks, PythonDocstringScanState,
    atx_heading_text, declaration_captures, near_miss_heading, parse_id, section_path,
    source_scan_line,
};
use crate::model::{
    Catalog, Citation, Declaration, DeclarationSource, FileRecord, FileText, HeadingKind, Id,
    NearMissHeading, UnmarkedHeadingCandidate,
};
use crate::workspace::WorkspaceCitationTarget;

/// The per-file scan (§AR-scanner.2): line by line, find declaration headings
/// (§AR-scanner.2.1 — in Markdown or in a code/`"""` doc-comment, §AR-scanner.4),
/// nested section headings (§AR-scanner.2.2), and `<ID>[.<section>]` citations
/// (§AR-scanner.2.3, §FS-check.1.1) — skipping fenced code blocks; outside
/// Markdown, bare ID-shaped tokens inside string literals (§FS-fmt.2.3.1); in
/// Markdown, bare ID-shaped tokens inside a link destination (§FS-check.1.1.4,
/// §FS-fmt.2.3.4); and any bare token at all under `[reference] strict`
/// (§FS-config.3.1).
///
/// The shared full-ID matcher admits explicit-marker starts and bare-token
/// boundaries (§FS-check.1.1.10); whether a match is qualified
/// (marker + `<alias>/<ID>`) or unqualified (marker + `<ID>`) is determined by
/// whether the `<namespace>` capture fired (§AR-workspace.3.1). The alias
/// prefix is only honoured when the marker precedes it — an unmarked
/// `<alias>/<ID>` in prose is text, never a qualified citation
/// (§FS-workspace.1.3, §AR-workspace.3.1).
///
/// In workspace mode the caller passes a non-empty `workspace_targets` so a
/// `§<alias>/<ID>` token parses with the target project's grammar inline —
/// one disk read for both unqualified and qualified citations
/// (§AR-workspace.5.1). An empty slice falls back to the loose qualified
/// parser used by member-local scans (§FS-workspace.5.2).
///
/// One file's text into `findings`, the text being what the caller read through
/// `sources`, a buffer where the run holds one (§AR-system.2.13): declarations, their section
/// headings, citations, and the headings that came close to being declarations.
///
/// `classify` turns on citing-side classification — declaration body ranges and each
/// citation's source kind. `grund check` asks for it; the read-only commands turn it
/// off, and a project without direction rules pays nothing.
///
/// The near-miss test runs in this pass rather than in a second read of the tree
/// because re-deriving the line, the position rules and the fence/docstring state
/// later costs a whole extra pass over every file for a list most runs find empty.
///
/// Section paths first land in the primary or duplicate map, then the body-span
/// post-pass narrows both maps and records rejected headings for §FS-declarations.checks.section-outside-declaration.
///
/// `claimed_markers` is what keeps the shorthand pattern from running at all on a
/// line whose markers are already accounted for.
///
/// Text section headings also require the spans, because the shared coordinate
/// catalog is body-local (§FS-declarations.section.map). `scan_one_file` gives this call a fresh
/// `Catalog`, so `findings` holds exactly this file's records.
pub(super) fn scan_file_text(
    path: &Path,
    file_text: FileText,
    schema: &Schema,
    frame: Frame<'_>,
    findings: &mut Catalog,
    workspace_targets: &[WorkspaceCitationTarget],
) -> Result<()> {
    let marker = &schema.citation.marker;
    // §FS-config-v2.schema.sources: a v2 file is read as its language is, hosts and all.
    let reading = frame.compiled.reading(schema, path);
    let grammar = reading.lexical.grammar;
    let (is_md, is_py) = (reading.markdown, reading.python);
    let docstring_python = reading.lexical.docstring_python;
    // §AR-scanner.2.8.1: the record keeps the text as read, and the pass reads it there.
    let mut record = FileRecord::new(file_text, is_md);
    let shared = record.text.clone();
    let text = shared.as_str();
    // §FS-languages.multi-line-string: what each line begins inside, read off the whole file.
    let literals = reading.literals(text);
    // §FS-citations.wrap: the links a Markdown line's zones are read in, read only
    // when a citation's rewrite is asked of them.
    let links = MarkdownLinks::new(if is_md { text } else { "" });
    // §AR-scanner.2.8.1: the comment blocks, walked once, are what the inline-note sites read.
    let (inline_sites, inline_block_lines) = if is_md {
        Default::default()
    } else {
        inline_citation_sites(&reading, &literals, &mut record, schema, workspace_targets)
    };
    // §FS-declarations.stubs.shape: a stub is a link-titled heading in a kind's home only.
    let stub_home = is_md && in_some_home(path, schema, frame);
    let mut markdown_blocks = MarkdownBlocks::default();
    let mut comment_fences = CommentFences::default();
    let mut py_docstring = PythonDocstringScanState::default();
    let mut current: Option<Declaration> = None;
    // A marker enables value authority in any scanned document (§FS-values.1),
    // and a declared chapter is recognized in the same source doc-comments
    // (§FS-values.2.5); unopted trees gain no extra read (§FS-values.9).
    let value_kind = |schema: &Schema| schema.kinds().any(|(_, kind)| kind.has_values());
    let scan_values = text.contains(EMBEDDED_VALUE_MARKER)
        || value_kind(schema)
        || workspace_targets
            .iter()
            .any(|target| value_kind(&target.schema));
    let has_binding_candidate = text.contains('`') && text.contains(marker);
    // §AR-scanner.2.3.2: reuse comment spans for qualified sites
    // too, including block interiors that have no comment prefix on this line.
    let has_qualified_candidate = !marker.is_empty() && text.contains(marker) && text.contains('/');
    let value_line_contexts = ((scan_values || has_binding_candidate || has_qualified_candidate)
        && !is_md)
        .then(|| recognized_source_value_contexts(&text, &reading));
    // §AR-scanner.2.4.2: citing-side classification feeds citation-direction and chapter
    // rules, so the caller asks for it: `check`, the LSP, `cover` and `refs` pass `true`;
    // `list`, `show`, `fmt`, ID completion, `sizes` and `batch` pass `false`.
    let classify = frame.run.scope.classify_citation_sources;
    // §AR-scanner.2.2.7 / §FS-declarations.checks.unmarked-heading.1: declaration and section
    // recognition happen in this same fence-aware pass. Plain ATX headings wait until body spans
    // are known before becoming reportable candidates.
    let mut held_headings = HeldHeadings::default();
    let mut total_lines = 0usize;
    // §FS-values.3.1.1.1: a literal that closed the line before, held for one line.
    let mut split = None;
    // §AR-scanner.2.9: conflict marker lines, read off every raw line.
    let mut markers = MarkerLines::default();

    for (idx, line) in text.lines().enumerate() {
        let lineno = idx + 1;
        total_lines = lineno;
        // §FS-declarations.body.conflict.markers: before the fence skip, which would hide it.
        markers.read(lineno, line);
        let block = if is_md {
            markdown_blocks.line(line)
        } else {
            MarkdownLine::Text
        };
        if block.in_fence() {
            continue;
        }
        // §FS-check.1.1.5.1: a raw-text HTML block line is never a heading, so it
        // declares nothing and opens no section, but its citations stay live below.
        let heading_position = block.may_be_heading();
        let trimmed = line.trim_start();
        let scan = source_scan_line(line, is_py, docstring_python, &mut py_docstring);
        record.zones.extend(zone_of(lineno, &scan));
        if !is_md {
            comment_fences.read(&mut record, lineno);
        }
        let heading = HeadingLine {
            text,
            line,
            lineno,
            scan: &scan,
            is_md,
        };
        let scan_line = scan.text.as_ref();
        let source_value_context = value_line_contexts
            .as_ref()
            .and_then(|contexts| contexts.get(idx).copied().flatten());
        let embedded_marker = embedded_value_marker_for_line(
            scan_line,
            is_md,
            scan.in_py_docstring,
            source_value_context,
        );

        // §FS-declarations.line.block-comment: a bare line of a closed `/* … */` block is in
        // declaration position, read as a docstring line is, where the comment reading finds none.
        let bare_block = !is_md && !scan.in_py_docstring && bare_block_line(&record, lineno);
        // §FS-config-v2.schema.sources.declarations.gate: one answer, before every recognizer.
        let eligible = declaration_eligible(&record, &reading, lineno);
        let mut declared = false;
        if heading_position
            && eligible
            && let Some((caps, bare)) =
                declaration_captures(grammar, scan_line, scan.in_py_docstring, is_md)
                    .map(|caps| (caps, false))
                    .or_else(|| {
                        bare_block
                            .then(|| declaration_captures(grammar, scan_line, true, is_md))
                            .flatten()
                            .map(|caps| (caps, true))
                    })
            && let Some(id) = parse_id(&caps, grammar)
        {
            close_declaration(&mut current, findings);
            // §FS-declarations.line.configured-slug: the matched delimiter belongs to the tail.
            let tail = &scan_line[caps.name("id").unwrap().end()..];
            let (is_stub, defined_in, title) = stub_and_title(tail, stub_home);
            let heading_level =
                heading_level_for_line(scan_line, is_md || scan.in_py_docstring || bare, &caps);
            record.headings.push(heading.record(
                &record,
                heading_level,
                None,
                HeadingKind::Declaration,
            ));
            current = Some(Declaration {
                authored_id: Some(caps.name("id").unwrap().as_str().to_string()),
                id,
                file: path.to_path_buf(),
                line: lineno,
                heading_level,
                sections: BTreeMap::new(),
                duplicate_sections: Vec::new(),
                is_stub,
                defined_in,
                // §AR-scanner.4.6: recorded after the walk, once every declaration is in.
                stub_resolution: None,
                e2e_case: None,
                title,
                // §AR-scanner.2.4.1: real body span is assigned in the post-pass
                // below once every declaration and (for Markdown) every heading
                // on the file is known. Default to the single declaration line.
                body_start: lineno,
                body_end: lineno,
                body_has_content: false,
                source: DeclarationSource::Text,
                value_valid: None,
            });
            if let Some(marker_start) = embedded_marker {
                push_invalid_embedded_marker(
                    findings,
                    current.as_ref().map(|decl| decl.id.clone()),
                    path,
                    lineno,
                    scan.column_offset,
                    marker_start,
                    MISPLACED_MARKER,
                );
            }
            declared = true;
        }

        // §FS-declarations.checks.declaration-near-miss.1: the line was not a declaration. Ask the
        // near-miss pattern whether it looked like one, here rather than in a second read of the
        // tree — the scan has the line, the position rules and the fence/docstring state.
        if heading_position
            && !declared
            && eligible
            && let Some(((text, format, kind), bare)) =
                near_miss_heading(grammar, scan_line, scan.in_py_docstring, is_md)
                    .map(|miss| (miss, false))
                    .or_else(|| {
                        bare_block
                            .then(|| near_miss_heading(grammar, scan_line, true, is_md))
                            .flatten()
                            .map(|miss| (miss, true))
                    })
        {
            close_declaration(&mut current, findings);
            findings.near_miss_headings.push(NearMissHeading {
                file: path.to_path_buf(),
                line: lineno,
                text: text.to_string(),
                format: format.to_string(),
            });
            let token_end = scan_line
                .find(text)
                .map(|start| start + text.len())
                .unwrap_or(0);
            let (is_stub, defined_in, title) = stub_and_title(&scan_line[token_end..], stub_home);
            let heading_level = if is_md || scan.in_py_docstring || bare {
                scan_line
                    .trim_start()
                    .chars()
                    .take_while(|ch| *ch == '#')
                    .count()
                    .max(1)
            } else {
                1
            };
            record.headings.push(heading.record(
                &record,
                heading_level,
                None,
                HeadingKind::Declaration,
            ));
            // §FS-config.3.2.5 / §AR-scanner.2.1: retain a rejected declaration-position
            // token as an ordinary catalog declaration, with exact spelling and all
            // body/section state from this same file pass.
            current = Some(Declaration {
                authored_id: Some(text.to_string()),
                id: Id::legacy(kind.to_string(), text),
                file: path.to_path_buf(),
                line: lineno,
                heading_level,
                sections: BTreeMap::new(),
                duplicate_sections: Vec::new(),
                is_stub,
                defined_in,
                stub_resolution: None,
                e2e_case: None,
                title,
                body_start: lineno,
                body_end: lineno,
                body_has_content: false,
                source: DeclarationSource::Text,
                value_valid: None,
            });
            // §FS-declarations.checks.declaration-near-miss.5: its title is read below.
            declared = true;
        }
        // §AR-scanner.2.3: a declaration's heading is read for its citations below,
        // never again for structure.
        let structure = heading_position && !declared;

        // §FS-languages.fence.comment-block: a fence in a comment block hides its headings only.
        let heading_structure = structure && !record.in_comment_fence(lineno);
        let section_caps = heading_structure
            .then(|| grammar.section_re.captures(scan_line))
            .flatten();
        let recognized_section = section_caps.as_ref().and_then(section_path).is_some();
        // §FS-languages.comment-wrapper.headings: a bare block line is read as written, so a
        // bare `## 1. Sub` is as deep as its own `#` run, its declaration's section `.1`.
        let markdown_depth = is_md || scan.in_py_docstring || bare_block;
        // §AR-scanner.2.8.1: every heading the loop meets, settled once the bodies are known.
        if let Some(caps) = section_caps.as_ref()
            && let Some(sec) = section_path(caps)
        {
            let level = heading_level_for_line(scan_line, markdown_depth, caps);
            let path = Some(sec.to_string());
            record
                .headings
                .push(heading.record(&record, level, path, HeadingKind::Rejected));
        } else if is_md
            && structure
            && let Some(level) = markdown_heading_level(trimmed)
        {
            record
                .headings
                .push(heading.record(&record, level, None, HeadingKind::Plain));
        }
        let mut embedded_marker_attached = false;
        if let Some(caps) = section_caps
            && let Some(decl) = current.as_mut()
            && let Some(sec) = section_path(&caps)
        {
            embedded_marker_attached = record_section_heading(
                decl,
                &caps,
                sec,
                scan_line,
                &scan,
                markdown_depth,
                is_md,
                lineno,
                embedded_marker,
                schema,
                path,
                findings,
            );
        }
        if is_md
            && structure
            && !recognized_section
            && let Some(heading_level) = markdown_heading_level(line)
        {
            let heading = line.trim_end().trim_start().to_string();
            let title = atx_heading_text(trimmed, heading_level).to_string();
            let column = line.find('#').unwrap_or(0) + 1;
            // §FS-declarations.section.relative.form: its parent is known once the file is.
            held_headings.push(UnmarkedHeadingCandidate {
                file: path.to_path_buf(),
                line: lineno,
                column,
                heading,
                heading_level,
                title,
            });
        }
        // §FS-languages.fence.comment-block: a fenced marker is an illustration, as in Markdown.
        if embedded_marker.is_some()
            && !embedded_marker_attached
            && heading_structure
            && authored_heading_level(
                scan_line,
                is_md || scan.in_py_docstring,
                source_value_context.is_some_and(|context| context.block_comment),
                reading.lexical,
            )
            .is_some()
        {
            let site = invalid_embedded_marker(
                current.as_ref().map(|decl| decl.id.clone()),
                path,
                lineno,
                scan.column_offset + embedded_marker.unwrap() + 1,
                MISPLACED_MARKER,
            );
            // §FS-values.2.4: a relative heading's marker is the parent pass's to attach.
            findings
                .invalid_value_declarations
                .extend(held_headings.hold_marker(lineno, site));
        }

        let citation_line = CitationLine {
            scan_line,
            raw_line: line,
            docstring: DocstringContent::of(&scan, line, literals.entry(idx)),
            links: links.line(idx),
            column_offset: scan.column_offset,
            lineno,
            path,
            schema,
            frame,
            lexical: reading.lexical,
            source_forms: reading.source_forms(),
            is_md,
            may_be_heading: heading_position,
            declares: current
                .as_ref()
                .filter(|decl| decl.line == lineno)
                .map(|decl| &decl.id),
            value_comment: source_value_context,
            inline_sites: &inline_sites,
            inline_block_lines: &inline_block_lines,
        };
        let workspace_mode = !workspace_targets.is_empty();
        let citation_start = findings.citations.len();
        let mut qualified_marker_starts = BTreeSet::new();
        // §AR-scanner.2.6.1: every marker the full-ID pattern matched at, whether or
        // not this pass emitted a citation there. The shorthand pass skips these
        // (§DF-number-only-citation-shorthand.2.6), so the full ID always wins.
        let mut claimed_markers: Vec<usize> = Vec::new();
        // §FS-check.1.1.10: explicit-marker and bare starts share the same policy gates.
        // §FS-check.1.1.11: a pattern is recorded and claims its marker, so no pass reads a prefix.
        for (offset, caps) in claim_citation_tokens(
            &citation_line,
            workspace_mode,
            &mut claimed_markers,
            findings,
        ) {
            let Some(full) = caps.get(0) else { continue };
            let token_start = offset + full.start();
            let token_end = offset + full.end();
            let namespace = caps.name("namespace").map(|m| m.as_str().to_string());
            let has_marker = scan_line[..token_start].ends_with(marker);
            if has_marker {
                claimed_markers.push(token_start - marker.len());
            }
            // §FS-check.1.1.9 / §AR-scanner.2.3.1: the never-rewrite zones are asked
            // before the unmarked-`alias/ID` rule and the strict gate, so the escape
            // is what exempts `<§>ID` and `<§>alias/ID` alike, in both modes.
            let bare_zone = !has_marker && citation_line.bare_token_in_zone(token_start, marker);
            if bare_zone {
                continue;
            }
            // In workspace mode, the qualified branch is parsed below with the
            // target's grammar — let that pass own every `§<alias>/...` hit so
            // we never emit one with the citing project's grammar.
            if workspace_mode && namespace.is_some() {
                continue;
            }
            // §FS-workspace.1.3, §AR-workspace.3.1: an unmarked `alias/ID` is text,
            // not a citation. The slash is part of the visual token; we do not
            // fall back to recognising the trailing ID as a bare citation.
            if namespace.is_some() && !has_marker {
                continue;
            }
            if schema.citation.strict && !has_marker {
                continue;
            }
            // In an opted-in repository an unmarked name tail is prose as one
            // whole token, even in compatibility scanning mode (§FS-check.1.1.2).
            if !has_marker && grammar.is_named_section(caps.name("sec").map(|sec| sec.as_str())) {
                continue;
            }
            let Some(id) = parse_id(&caps, grammar) else {
                continue;
            };
            let start = if has_marker {
                token_start.saturating_sub(marker.len())
            } else {
                token_start
            };
            if namespace.is_some() && has_marker && citation_line.qualified_suppressed(start) {
                continue;
            }
            if citation_line.declares == Some(&id) {
                continue;
            }
            let text = scan_line[start..token_end].to_string();
            if namespace.is_some() && has_marker {
                qualified_marker_starts.insert(start);
            }
            findings.citations.push(Citation {
                namespace,
                id,
                section: caps.name("sec").map(|m| m.as_str().to_string()),
                file: path.to_path_buf(),
                line: lineno,
                column: scan.column_offset + start + 1,
                has_marker,
                shorthand: false,
                local_section: false,
                shorthand_rewritable: true,
                numeric_run: false,
                text,
                inline_site: inline_sites.get(&lineno).cloned(),
                // §AR-scanner.2.4: classified in the post-pass below.
                source_kind: String::new(),
                enclosing_declaration: None,
                enclosing_section: None,
            });
        }
        if workspace_mode {
            scan_workspace_qualified_pass(&citation_line, workspace_targets, findings);
        } else {
            // §AR-scanner.2.6.1.1: the fallback records what it claimed into the same
            // set, so the shorthand pass below can tell a qualified marker that
            // already became a citation from one no qualified pass could parse.
            scan_fallback_qualified_citations(
                &citation_line,
                &mut qualified_marker_starts,
                findings,
            );
        }
        scan_shorthand_citations(
            &citation_line,
            workspace_mode,
            &claimed_markers,
            &qualified_marker_starts,
            findings,
        );
        scan_local_section_candidates(
            &citation_line,
            &claimed_markers,
            &qualified_marker_starts,
            findings,
        );
        scan_legacy_citation_candidates(&citation_line, findings);
        scan_escaped_citations(&citation_line, findings);
        // Exact candidates stay in this pass. The checker activates them only
        // for whole or marked authority, keeping unmarked prose inert across
        // workspaces (§FS-values.3.1, §FS-values.7, §FS-values.9).
        scan_value_bindings(&citation_line, workspace_targets, citation_start, findings);
        // §FS-values.3.1.1.1: a split binding needs both a backtick and the marker.
        if has_binding_candidate {
            scan_split_binding(&citation_line, workspace_targets, &mut split, findings);
        }
    }

    close_declaration(&mut current, findings);
    record.total_lines = total_lines;
    held_headings.hold_doc_comment(findings, &mut record, path);
    held_headings.resolve(findings, &mut record, grammar.named_sections, schema);
    // §AR-scanner.2.8.2: every declaration's and section's extents, on every run.
    assign_extents(&mut record, findings);

    // §AR-scanner.2.4: now that every declaration and (for Markdown) every heading
    // on the file is known, fix each declaration's body span and classify each
    // citation's citing side — off the hot path either way (§AR-benchmarks).
    let due = AfterPass::of(findings, schema, scan_values);
    let has_unmarked_headings = !held_headings.unmarked.is_empty();
    let has_local_section_candidates = !findings.local_section_citation_candidates.is_empty();
    if classify
        // §FS-cover.6.2: a `cover --lines` request reads the body spans too.
        || !frame.run.scope.owner_lines.is_empty()
        || due.text_sections
        || due.value_declarations
        || due.embedded_roots
        || due.declared_chapters
        || has_local_section_candidates
        // §AR-scanner.2.9: a block's owners are read off the body spans.
        || markers.any()
    {
        assign_declaration_bodies(findings, &record);
    }
    // §FS-declarations.body.conflict.where, §FS-declarations.body.conflict.owners.
    markers.record(path, &record, grammar, findings);
    if due.text_sections {
        retain_in_body_sections(findings);
    }
    if has_unmarked_headings {
        assign_unmarked_heading_owners(findings, held_headings.unmarked, &record);
    }
    due.validate_values(
        path,
        text,
        &reading,
        schema,
        frame,
        value_line_contexts.as_deref(),
        findings,
    );
    resolve_citation_owners(findings, schema, frame, path, &record, classify);
    // §AR-scanner.2.7.1: the headings and doc-comment blocks a grounding unit finer
    // than the file is cut out of — recorded only where the file's own row asks
    // for one, so a level-1 tree pays nothing (§FS-config.3.4.8.2).
    record_file_structure(path, &record, schema, frame, findings);
    findings.files.insert(path.to_path_buf(), Arc::new(record));
    Ok(())
}
