use std::collections::BTreeMap;
use std::path::Path;

use super::extents::body_has_content;
use super::line_owners::{LineOwners, resolve_requested_lines};
use super::record::walk_blocks;
use crate::config::{FileReading, Frame, Owners, Schema};
use crate::grammar::{AliasGrammar, MultiLineStrings, block_declares_id, inline_note_verdicts};
use crate::model::sort_path_key;
use crate::model::{
    BlockForm, Catalog, FileRecord, InlineCitationSite, SectionHeadingOutsideDeclaration,
};
use crate::workspace::WorkspaceCitationTarget;

/// Narrow the shared coordinate catalog to each declaration's body and retain
/// every rejected heading as one check site (§FS-declarations.section.map, §FS-declarations.checks.section-outside-declaration).
///
/// The line scan's current declaration runs farther than the body in Markdown,
/// source comments, docstrings, and stubs. Applying the already-computed span
/// here makes every map reader agree without a surface-local guard. Rejected
/// duplicate claimants become outside-heading sites too, never stale
/// `duplicate-section` collisions.
pub(super) fn retain_in_body_sections(findings: &mut Catalog) {
    let mut rejected = Vec::new();
    for decl in findings.declarations.values_mut().flatten() {
        let body = decl.body_start..=decl.body_end;
        decl.sections.retain(|path, info| {
            let retained = body.contains(&info.line);
            if !retained {
                rejected.push(SectionHeadingOutsideDeclaration {
                    file: decl.file.clone(),
                    line: info.line,
                    path: path.clone(),
                });
            }
            retained
        });
        decl.duplicate_sections.retain(|(path, info)| {
            let retained = body.contains(&info.line);
            if !retained {
                rejected.push(SectionHeadingOutsideDeclaration {
                    file: decl.file.clone(),
                    line: info.line,
                    path: path.clone(),
                });
            }
            retained
        });
    }
    rejected.sort_by(|left, right| {
        (sort_path_key(&left.file), left.line).cmp(&(sort_path_key(&right.file), right.line))
    });
    rejected.dedup_by(|left, right| left.file == right.file && left.line == right.line);
    findings
        .section_headings_outside_declarations
        .extend(rejected);
}

pub(crate) fn section_path_is_numeric(path: &str) -> bool {
    path.split('.')
        .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
}

/// The level of a Markdown ATX heading line (`#` count), or `None` when the line
/// is not a heading (§FS-declarations.checks.unmarked-heading.1). ATX syntax permits at most three leading
/// ASCII spaces and one through six `#`s followed by an ASCII space/tab or EOL.
pub(crate) fn markdown_heading_level(line: &str) -> Option<usize> {
    let indentation = line.bytes().take_while(|byte| *byte == b' ').count();
    if indentation > 3 {
        return None;
    }
    let heading = &line[indentation..];
    let hashes = heading.bytes().take_while(|byte| *byte == b'#').count();
    if !(1..=6).contains(&hashes) {
        return None;
    }
    match heading.as_bytes().get(hashes).copied() {
        None => Some(hashes),
        Some(b' ' | b'\t') => Some(hashes),
        _ => None,
    }
}

/// Publish every declaration's body line span on the catalog (§AR-scanner.2.4.1),
/// as the record's body rule answered it (§FS-declarations.body,
/// §AR-scanner.2.8.2).
pub(super) fn assign_declaration_bodies(findings: &mut Catalog, record: &FileRecord) {
    for decl in findings.declarations.values_mut().flatten() {
        decl.body_start = decl.line;
        decl.body_end = record
            .heading_at(decl.line)
            .map_or(decl.line, |heading| heading.end.max(decl.line));
        decl.body_has_content = !decl.is_stub && body_has_content(record, decl.line, decl.body_end);
    }
}

/// Settle who owns each citation site once body spans are known: classify the
/// citing side, then promote the local candidates that gained a unique owner
/// (§AR-scanner.2.4.2, §AR-scanner.2.4). Promotion reads what classification
/// wrote, so the order is fixed here rather than left to the caller. The lines a
/// `cover --lines` run asks about are settled here too, by the same lookup, while
/// the heading stack is still held (§FS-cover.6.2, §AR-scanner.2.4.4).
pub(super) fn resolve_citation_owners(
    findings: &mut Catalog,
    schema: &Schema,
    frame: Frame<'_>,
    path: &Path,
    record: &FileRecord,
    classify: bool,
) {
    if !frame.run.scope.owner_lines.is_empty() {
        resolve_requested_lines(findings, path, record, &frame.run.scope.owner_lines);
    }
    let has_local_candidates = !findings.local_section_citation_candidates.is_empty();
    if classify || has_local_candidates {
        classify_citation_sources(findings, schema, frame, path, record);
    }
    if has_local_candidates {
        promote_local_section_citations(findings);
    }
}

/// Classify each citation's citing side by the three-step fallback of
/// §AR-scanner.2.4.2: the enclosing declaration's kind (nearest preceding
/// declaration whose body contains the site), else the file's unique kind home,
/// else the homeless kind — `code`, or whatever the project named it
/// (§FS-config.3.9.2.2).
fn classify_citation_sources(
    findings: &mut Catalog,
    schema: &Schema,
    frame: Frame<'_>,
    path: &Path,
    record: &FileRecord,
) {
    let owners = LineOwners::new(findings, record);
    // §FS-config.3.9.2.2: step 3 of the fallback is the homeless kind, whose name
    // is `code` only where the project did not name it something truer.
    let unowned_kind =
        file_home_kind(path, schema, frame).unwrap_or_else(|| schema.complement_name().to_string());
    let settle = |line: usize| match owners.owner_at(line) {
        Some((id, section)) => (id.kind.clone(), Some(id.clone()), section),
        None => (unowned_kind.clone(), None, None),
    };
    for cite in &mut findings.citations {
        (
            cite.source_kind,
            cite.enclosing_declaration,
            cite.enclosing_section,
        ) = settle(cite.line);
    }
    for candidate in &mut findings.legacy_citation_candidates {
        (
            candidate.source_kind,
            candidate.enclosing_declaration,
            candidate.enclosing_section,
        ) = settle(candidate.line);
    }
    for candidate in &mut findings.local_section_citation_candidates {
        (
            candidate.source_kind,
            candidate.enclosing_declaration,
            candidate.enclosing_section,
        ) = settle(candidate.line);
    }
}

/// Promote supported declaration-local paths with one body owner into the
/// ordinary citation graph while preserving their authored token
/// (§AR-scanner.2.4, §DF-declaration-local-section-shorthand.2.4). Unsupported
/// and ownerless records remain diagnostic-only, so no consumer can infer a
/// target that the body rule did not supply.
fn promote_local_section_citations(findings: &mut Catalog) {
    let candidates = std::mem::take(&mut findings.local_section_citation_candidates);
    for candidate in candidates {
        let Some(section) = candidate.section.clone() else {
            findings.local_section_citation_candidates.push(candidate);
            continue;
        };
        let Some(owner) = candidate.enclosing_declaration.clone() else {
            findings.local_section_citation_candidates.push(candidate);
            continue;
        };
        findings.citations.push(crate::model::Citation {
            namespace: None,
            id: owner.clone(),
            section: Some(section),
            file: candidate.file,
            line: candidate.line,
            column: candidate.column,
            has_marker: true,
            shorthand: false,
            local_section: true,
            shorthand_rewritable: candidate.rewritable,
            numeric_run: false,
            text: candidate.text,
            inline_site: candidate.inline_site,
            source_kind: candidate.source_kind,
            enclosing_declaration: Some(owner),
            enclosing_section: candidate.enclosing_section,
        });
    }
    findings.citations.sort_by(|left, right| {
        (sort_path_key(&left.file), left.line, left.column).cmp(&(
            sort_path_key(&right.file),
            right.line,
            right.column,
        ))
    });
}

/// The kind of the row that owns `path` — step 2 of §AR-scanner.2.4.2 — by the
/// one ownership rule every per-file question asks (§AR-scanner.4.7,
/// §FS-config-v2.schema.places.ownership). `None` when the complement owns it,
/// so the citation falls through to `code`.
pub(crate) fn file_home_kind(path: &Path, schema: &Schema, frame: Frame<'_>) -> Option<String> {
    let owner = Owners::new(schema, frame).of(path)?;
    Some(owner.row.name.clone())
}

/// Whether any kind's home holds `path`: the home test of a stub
/// (§FS-declarations.stubs.shape), read off the same places as `file_home_kind`
/// (§AR-scanner.4.7).
pub(crate) fn in_some_home(path: &Path, schema: &Schema, frame: Frame<'_>) -> bool {
    Owners::new(schema, frame).in_some_home(path)
}

/// The lexical half of the run's citation targets (§FS-workspace.1.2): which
/// project's compiled grammar reads a qualified token's ID tail, and nothing
/// else about the target.
///
/// The scanner is what iterates the loaded projects, so the scanner is what
/// picks the grammar out of each one — a tokenizer handed a whole `Config` per
/// target would be the grammar reading two components above it
/// (§AR-system.2.1, §AR-system.4).
fn lexical_targets(targets: &[WorkspaceCitationTarget]) -> Vec<AliasGrammar<'_>> {
    targets
        .iter()
        .map(|target| AliasGrammar {
            alias: &target.alias,
            grammar: &target.compiled.grammar,
        })
        .collect()
}

/// Locate the source-comment blocks that can host inline citation sites
/// (§FS-inline-citation-style.1), from the blocks walked once into the record
/// (§AR-scanner.2.8.1). Markdown prose is deliberately out of scope, and so is a
/// *doc comment*: documentation is not a note about a clause, so nothing the
/// inline citation style says applies to one (§FS-inline-citation-style.1.1,
/// §AR-scanner.4).
pub(super) fn inline_citation_sites(
    reading: &FileReading<'_>,
    literals: &MultiLineStrings,
    record: &mut FileRecord,
    schema: &Schema,
    workspace_targets: &[WorkspaceCitationTarget],
) -> (
    BTreeMap<usize, InlineCitationSite>,
    BTreeMap<usize, std::sync::Arc<[String]>>,
) {
    let text = record.text.clone();
    let lines = text.as_str().lines().collect::<Vec<_>>();
    record.blocks = walk_blocks(&lines, reading);
    let blocks = &record.blocks;
    let mut sites = BTreeMap::new();
    let mut site_lines = BTreeMap::new();
    if blocks.is_empty() {
        return (sites, site_lines);
    }
    // §FS-workspace.1.2: the lexical half of the run's citation targets, taken once
    // per file — which project's grammar reads a qualified token's ID tail is all
    // the tokenizer below needs of a target (§AR-system.2.1).
    let alias_grammars = lexical_targets(workspace_targets);
    let lexical = reading.lexical;
    for recorded in blocks {
        let (start, end) = (recorded.start - 1, recorded.end - 1);
        let block = &lines[start..=end];
        // §FS-inline-citation-style.1.1: a doc comment hosts no site — the same
        // skip a block that *declares* an ID already earned, and for the same
        // reason: its shape is not this spec's to govern.

        // §FS-config-v2.schema.sources.declarations.prose: a block the gate keeps from
        // declaring is prose, so it is measured as any note is.
        if !recorded.doc
            && !(recorded.declares
                && block_declares_id(
                    block,
                    recorded.form == BlockForm::Docstring,
                    recorded.form == BlockForm::Block && recorded.closed,
                    lexical,
                ))
        {
            // §FS-inline-citation-style.3.3: both verdicts are taken here, while
            // the block's lines are in hand, so the checker never re-reads one
            // (§AR-scanner.3.2).
            let (has_note, layout_violations) = inline_note_verdicts(
                block,
                // §FS-languages.multi-line-string: each line read in the state it begins in.
                literals.starting_at(start),
                start + 1,
                lexical,
                &alias_grammars,
            );
            // §AR-scanner.3.1: retain each width once, deriving the maximum from
            // the same Unicode scalar counts (§FS-inline-citation-style.2.3.2).
            let line_columns: std::sync::Arc<[usize]> =
                block.iter().map(|line| line.chars().count()).collect();
            let site = InlineCitationSite {
                first_line: start + 1,
                last_line: end + 1,
                max_columns: line_columns.iter().copied().max().unwrap_or(0),
                line_columns,
                has_note,
                layout_violations,
            };
            // Only a marker-prefixed rejected candidate can need post-catalog
            // reconciliation. Ordinary comment blocks retain no source copy.
            let block_lines = (!schema.citation.marker.is_empty()
                && block
                    .iter()
                    .any(|line| line.contains(&schema.citation.marker)))
            .then(|| {
                std::sync::Arc::<[String]>::from(
                    block
                        .iter()
                        .map(|line| (*line).to_string())
                        .collect::<Vec<_>>(),
                )
            });
            for line in (start + 1)..=(end + 1) {
                sites.insert(line, site.clone());
                if let Some(block_lines) = &block_lines {
                    site_lines.insert(line, std::sync::Arc::clone(block_lines));
                }
            }
        }
    }
    (sites, site_lines)
}
