use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use super::line_owners::{LineOwners, resolve_requested_lines};
use super::unmarked_headings::markdown_declaration_body_end;
use crate::config::{Frame, Schema};
use crate::grammar::{
    AliasGrammar, CommentBlockKind, DocCommentRule, block_declares_id, block_is_doc_comment,
    comment_blocks, doc_comment_rule, first_content_line, inline_note_verdicts,
};
use crate::model::{Catalog, InlineCitationSite, SectionHeadingOutsideDeclaration};
use crate::model::{scanned_decl_relative_path, scanned_path_key, sort_path_key};
use crate::workspace::WorkspaceCitationTarget;

/// Narrow the shared coordinate catalog to each declaration's body and retain
/// every rejected heading as one check site (§FS-show.2.1.2.1, §FS-declarations.checks.section-outside-declaration).
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

/// Assign every declaration in `findings` its body line span (§AR-scanner.2.4.1).
/// In Markdown the body runs until the next heading at the same or higher level;
/// in a source file it is bounded by the comment/docstring block the declaration
/// opens, capped before the next declaration sharing that block.
pub(super) fn assign_declaration_bodies(
    findings: &mut Catalog,
    is_md: bool,
    is_py: bool,
    schema: &Schema,
    frame: Frame<'_>,
    text: &str,
    md_headings: &[(usize, usize)],
    total_lines: usize,
) {
    // The lines (sorted) at which this file declares — used to cap a source
    // declaration's body before the next declaration in the same comment block.
    let mut decl_lines: Vec<usize> = findings
        .declarations
        .values()
        .flatten()
        .map(|decl| decl.line)
        .collect();
    decl_lines.sort_unstable();

    let code_blocks = (!is_md).then(|| comment_block_ranges(text, is_py, schema, frame));

    for decl in findings.declarations.values_mut().flatten() {
        decl.body_start = decl.line;
        if decl.is_stub {
            decl.body_end = decl.line;
            decl.body_has_content = false;
            continue;
        }
        if is_md {
            decl.body_end = markdown_declaration_body_end(decl, md_headings, total_lines);
        } else {
            let block_end = code_blocks
                .as_ref()
                .and_then(|blocks| {
                    blocks
                        .iter()
                        .find(|(start, end)| *start <= decl.line && decl.line <= *end)
                        .map(|(_, end)| *end)
                })
                .unwrap_or(decl.line);
            let next_decl_cap = decl_lines
                .iter()
                .copied()
                .find(|line| *line > decl.line)
                .filter(|line| *line <= block_end)
                .map(|line| line - 1);
            decl.body_end = next_decl_cap.unwrap_or(block_end).max(decl.line);
        }
        decl.body_has_content = text
            .lines()
            .skip(decl.line)
            .take(decl.body_end.saturating_sub(decl.line))
            .any(|line| !line.trim().is_empty());
    }
}

/// The 1-indexed inclusive line spans of every comment / docstring block in a
/// source file (§AR-scanner.2.4.1) — the shared block walk of `comment_block.rs`
/// without the declares-an-ID filtering, so a declaration's body can be bounded
/// by the block that hosts it.
fn comment_block_ranges(
    text: &str,
    is_py: bool,
    schema: &Schema,
    frame: Frame<'_>,
) -> Vec<(usize, usize)> {
    let lines = text.lines().collect::<Vec<_>>();
    comment_blocks(&lines, is_py, frame.compiled.lexical(schema))
        .into_iter()
        .map(|(start, end, _)| (start + 1, end + 1))
        .collect()
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
    md_headings: &[(usize, usize)],
    total_lines: usize,
    classify: bool,
) {
    if !frame.run.scope.owner_lines.is_empty() {
        resolve_requested_lines(
            findings,
            path,
            md_headings,
            total_lines,
            &frame.run.scope.owner_lines,
        );
    }
    let has_local_candidates = !findings.local_section_citation_candidates.is_empty();
    if classify || has_local_candidates {
        classify_citation_sources(findings, schema, frame, path, md_headings);
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
    md_headings: &[(usize, usize)],
) {
    let owners = LineOwners::new(findings, md_headings);
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

/// The kind whose configured home (`[[kinds]] folder` / `file`, §FS-config.3.4)
/// uniquely contains `path` — step 2 of §AR-scanner.2.4.2. `None` when no home or
/// more than one home matches, so the citation falls through to `code`.
pub(crate) fn file_home_kind(path: &Path, schema: &Schema, frame: Frame<'_>) -> Option<String> {
    // Walked file paths are canonicalized against the scan root (`scan_roots`
    // resolves an explicit scope), while `config.root` is the configured,
    // possibly-symlinked root — so on macOS a temp dir resolves through
    // `/private` and on Windows through a `\\?\` verbatim prefix, and a plain
    // `strip_prefix(config.root)` misses. Reuse the checker's reverse-home
    // helper, which strips against the physical root first, then the configured
    // one, so the lookup is identical to the declaration-home lookup.
    let physical_root =
        fs::canonicalize(frame.root()).unwrap_or_else(|_| frame.root().to_path_buf());
    let relative = scanned_decl_relative_path(path, frame.root(), &physical_root)?;
    let relative = relative.as_ref();
    let mut matched: Option<&str> = None;
    for row in &schema.rows {
        let hit = match (row.file(), row.folder()) {
            (Some(file), _) => relative == scanned_path_key(Path::new(file)).as_path(),
            (_, Some(folder)) => relative.starts_with(scanned_path_key(Path::new(folder))),
            _ => false,
        };
        if hit {
            if matched.is_some_and(|prev| prev != row.name.as_str()) {
                return None;
            }
            matched = Some(row.name.as_str());
        }
    }
    matched.map(str::to_string)
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
/// (§FS-inline-citation-style.1). Markdown prose is deliberately out of scope,
/// and so is a *doc comment*: documentation is not a note about a clause, so
/// nothing the inline citation style says applies to one
/// (§FS-inline-citation-style.1.1, §AR-scanner.4).
pub(super) fn inline_citation_sites(
    path: &Path,
    text: &str,
    is_md: bool,
    is_py: bool,
    schema: &Schema,
    frame: Frame<'_>,
    workspace_targets: &[WorkspaceCitationTarget],
) -> (
    BTreeMap<usize, InlineCitationSite>,
    BTreeMap<usize, std::sync::Arc<[String]>>,
) {
    let mut sites = BTreeMap::new();
    let mut site_lines = BTreeMap::new();
    if is_md {
        return (sites, site_lines);
    }
    let lines = text.lines().collect::<Vec<_>>();
    // §FS-workspace.1.2: the lexical half of the run's citation targets, taken once
    // per file — which project's grammar reads a qualified token's ID tail is all
    // the tokenizer below needs of a target (§AR-system.2.1).
    let alias_grammars = lexical_targets(workspace_targets);
    // §FS-inline-citation-style.1.1.2: which blocks this file's language calls
    // documentation. Read once from the extension, then applied to each block
    // below with one comparison (§AR-scanner.4).
    let doc_rule = doc_comment_rule(path);
    // Only a position language asks where the file's leading comment is, so the
    // scan for it is taken only there and the marker recognizers ignore the flag
    // it feeds (§GOAL-fast-feedback).
    let leading_limit = match doc_rule {
        DocCommentRule::Position(_) => first_content_line(&lines),
        _ => 0,
    };
    for (start, end, kind) in comment_blocks(&lines, is_py, frame.compiled.lexical(schema)) {
        let block = &lines[start..=end];
        // §FS-inline-citation-style.1.1: a doc comment hosts no site — the same
        // skip a block that *declares* an ID already earned, and for the same
        // reason: its shape is not this spec's to govern.
        let is_doc_comment = block_is_doc_comment(
            doc_rule,
            &kind,
            block,
            lines.get(end + 1).copied(),
            start <= leading_limit,
        );
        if !is_doc_comment
            && !block_declares_id(
                block,
                matches!(kind, CommentBlockKind::PythonDocstring),
                frame.compiled.lexical(schema),
            )
        {
            // §FS-inline-citation-style.3.3: both verdicts are taken here, while
            // the block's lines are in hand, so the checker never re-reads one
            // (§AR-scanner.3.2).
            let (has_note, layout_violations) = inline_note_verdicts(
                block,
                start + 1,
                frame.compiled.lexical(schema),
                &alias_grammars,
            );
            let site = InlineCitationSite {
                first_line: start + 1,
                last_line: end + 1,
                // §FS-inline-citation-style.2.3.2: a column is one character, not one
                // byte — `é` and `§` cost one each (§DF-note-columns-are-characters).
                max_columns: block
                    .iter()
                    .map(|line| line.chars().count())
                    .max()
                    .unwrap_or(0),
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
