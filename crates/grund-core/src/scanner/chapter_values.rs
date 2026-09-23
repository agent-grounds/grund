//! The declared value chapter's own level (§FS-values.2.5). Enrollment made
//! each named direct child a root and `embedded_values.rs` validates each root's
//! subtree; what is left is the one level in between, which holds nothing but
//! those roots. A chapter is strict whether or not it managed to hold a single
//! root, so this runs off the kind's configuration rather than off what
//! enrollment found (§AR-scanner.2.2.8).

use std::path::Path;

use super::embedded_value_context::{
    authored_heading_level, authored_heading_path, normalized_value_lines, semantic_comment_content,
};
use super::section_record::section_is_chapter_value_root;
use super::value_context::SourceValueLineContext;
use crate::config::{Config, kind_value_chapter};
use crate::model::{Findings, InvalidValueSite, paths_same_location};

/// What a declared chapter may not hold. One message for every shape, because
/// the rule is one rule — the chapter holds value roots and nothing else — and
/// the location is what tells the author which line broke it (§FS-check.3.20).
const CHAPTER_CONTENT: &str = "a declared value chapter holds only named value-root headings";

/// Report every line directly inside a declared chapter that is not one of its
/// named value-root headings (§FS-values.2.5). A root's own subtree belongs to
/// `validate_embedded_value_roots` and is skipped here, so a malformed component
/// is reported once, by the point that knows which coordinate was expected
/// (§FS-values.2.4.3).
pub(super) fn validate_declared_value_chapters(
    path: &Path,
    text: &str,
    is_md: bool,
    is_py: bool,
    config: &Config,
    source_contexts: Option<&[Option<SourceValueLineContext>]>,
    findings: &mut Findings,
) {
    let normalized = normalized_value_lines(text, is_py, config, source_contexts);
    let mut invalid = Vec::new();
    for decl in findings
        .declarations
        .values()
        .flatten()
        .filter(|decl| paths_same_location(&decl.file, path))
    {
        let Some(chapter) = kind_value_chapter(config, &decl.id.kind) else {
            continue;
        };
        let Some(chapter_info) = decl.sections.get(chapter) else {
            continue;
        };
        let chapter_level = chapter_info.heading_level;
        let mut skip_through = chapter_info.line;
        for line_no in chapter_info.line.saturating_add(1)..=decl.body_end {
            let Some((line, _, docstring, block_comment)) = normalized.get(line_no - 1) else {
                continue;
            };
            let markdown = is_md || *docstring;
            let level = authored_heading_level(line, markdown, *block_comment, config);
            // The chapter ends at the next heading of its own depth or above;
            // everything from there is another chapter's business.
            if level.is_some_and(|found| found <= chapter_level) {
                break;
            }
            if line_no <= skip_through {
                continue;
            }
            if level == Some(chapter_level + 1) {
                // A direct child owns its own subtree either way: a valid root's
                // is the component run, and a condemned one is already named
                // here, so its contents are not reported twice.
                skip_through = child_subtree_end(
                    &normalized,
                    line_no,
                    chapter_level + 1,
                    decl.body_end,
                    is_md,
                    config,
                );
                if authored_heading_path(line, markdown, *block_comment, true, config).is_some_and(
                    |found| section_is_chapter_value_root(config, &decl.id.kind, &found),
                ) {
                    continue;
                }
            } else {
                let content = semantic_comment_content(line, markdown, *block_comment, config);
                if content.is_empty() || matches!(content, "/*" | "/**" | "/*!" | "*" | "*/") {
                    continue;
                }
            }
            invalid.push(InvalidValueSite {
                id: Some(decl.id.clone()),
                file: decl.file.clone(),
                line: line_no,
                column: None,
                message: CHAPTER_CONTENT.to_string(),
                source: decl.source.clone(),
                binding_namespace: None,
                binding_section: None,
            });
        }
    }
    findings.invalid_value_declarations.extend(invalid);
}

/// The last line of the subtree a heading of `level` opened at `line_no`.
fn child_subtree_end(
    normalized: &[(String, usize, bool, bool)],
    line_no: usize,
    level: usize,
    body_end: usize,
    is_md: bool,
    config: &Config,
) -> usize {
    ((line_no + 1)..=body_end)
        .find(|candidate| {
            normalized
                .get(candidate.saturating_sub(1))
                .and_then(|(line, _, docstring, block_comment)| {
                    authored_heading_level(line, is_md || *docstring, *block_comment, config)
                })
                .is_some_and(|found| found <= level)
        })
        .map(|line| line - 1)
        .unwrap_or(body_end)
}
