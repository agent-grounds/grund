//! The one line of look-behind the value-binding recovery pass carries: a
//! literal that closes its line onto a value citation opening the next line of
//! the same run of text is an invalid attempted binding, not prose
//! (§FS-values.3.1.1.1, §AR-scanner.2.3.5).

use super::citation_line::CitationLine;
use super::context::markdown_heading_level;
use super::embedded_value_context::semantic_comment_content;
use super::value_binding_attempts::{
    AttemptedValueTarget, attempted_value_target, split_value_binding_site,
};
use super::value_context::{binding_span_is_inside, value_binding_context};
use crate::model::Catalog;
use crate::workspace::WorkspaceCitationTarget;

/// A literal whose closing backtick ended its line, held for one line so the
/// next can say whether it opens with the literal's citation.
pub(super) struct SplitLiteral {
    line: usize,
    column: usize,
    run: TextRun,
}

/// What makes two consecutive lines one run of text (§FS-values.3.1.1.1): one
/// Markdown paragraph at one blockquote depth, one Python docstring, or one
/// comment block of the shared block walk (§FS-inline-citation-style.1.2).
#[derive(Clone, Copy, PartialEq, Eq)]
enum TextRun {
    Markdown { quote_depth: usize },
    Docstring,
    Comment { block: usize },
}

/// Report the literal held from the line before when this line continues its
/// run and its first text after the continuation prefix is `(` and a
/// marker-prefixed citation aimed at value authority, then hold this line's own
/// literal if its closing backtick ends the line (§FS-values.3.1.1.1).
///
/// The citation stays the ordinary citation the line pass already recorded;
/// this adds the attempt and takes nothing away. A line that neither ends in a
/// backtick nor follows one pays only for its context lookup (§AR-benchmarks),
/// and a line the file pass skips — a fence, a declaration — breaks the
/// adjacency the held literal needs.
pub(super) fn scan_split_binding(
    line: &CitationLine<'_>,
    workspace_targets: &[WorkspaceCitationTarget],
    held: &mut Option<SplitLiteral>,
    findings: &mut Catalog,
) {
    let earlier = held.take();
    let Some(context) = value_binding_context(line) else {
        return;
    };
    let close_tick = closing_backtick(line, context);
    if earlier.is_none() && close_tick.is_none() {
        return;
    }
    let Some((run, text)) = text_run(line, context) else {
        return;
    };
    if let Some(literal) = earlier
        && literal.line + 1 == line.lineno
        && literal.run == run
        && let Some(target) = opening_value_citation(line, text, workspace_targets)
    {
        findings
            .invalid_value_bindings
            .push(split_value_binding_site(
                line.path,
                (literal.line, literal.column),
                target,
            ));
    }
    // A heading's literal ends its paragraph rather than running on into the
    // next line (§FS-values.3.1.1.1); a raw-text HTML block holds none (§FS-check.1.1.5.1).
    if line.is_md && line.may_be_heading && markdown_heading_level(text).is_some() {
        return;
    }
    *held = close_tick.and_then(|close_tick| {
        // Without an opener on this line the backtick closes a multiline
        // literal, as the same-line recovery pass reads it (§FS-values.3.1.1).
        let open_tick = line.scan_line[..close_tick]
            .rfind('`')
            .unwrap_or(close_tick);
        binding_span_is_inside(context, open_tick, close_tick + 1).then_some(SplitLiteral {
            line: line.lineno,
            column: line.column_offset + open_tick + 1,
            run,
        })
    });
}

/// The closing backtick that ends this line's value-binding context with only
/// whitespace after it. The raw line is asked too, because a Python docstring
/// that closes on this line ends its run there and its quotes sit outside the
/// scanned content.
fn closing_backtick(line: &CitationLine<'_>, context: (usize, usize)) -> Option<usize> {
    let content = line.scan_line.get(context.0..context.1)?.trim_end();
    if !content.ends_with('`') {
        return None;
    }
    let after = line.raw_line.get(line.column_offset + context.1..)?;
    after
        .trim()
        .is_empty()
        .then_some(context.0 + content.len() - 1)
}

/// The run this line belongs to and its text after the continuation prefix:
/// the blockquote markers and indentation in Markdown, the indentation in a
/// docstring, and the comment wrapper of §FS-values.2.4.1 in source.
fn text_run<'a>(line: &CitationLine<'a>, context: (usize, usize)) -> Option<(TextRun, &'a str)> {
    let content = line.scan_line.get(context.0..context.1)?;
    if line.is_md {
        let (quote_depth, text) = blockquote(content);
        return Some((TextRun::Markdown { quote_depth }, text));
    }
    if line.docstring.is_docstring() {
        return Some((TextRun::Docstring, content));
    }
    let comment = line.value_comment?;
    let text = semantic_comment_content(content, false, comment.block_comment, line.schema);
    Some((
        TextRun::Comment {
            block: comment.block,
        },
        text,
    ))
}

/// A Markdown line's blockquote depth and its text after the `>` markers.
fn blockquote(line: &str) -> (usize, &str) {
    let mut depth = 0;
    let mut text = line;
    while let Some(after) = text.trim_start_matches([' ', '\t']).strip_prefix('>') {
        depth += 1;
        text = after.strip_prefix(' ').unwrap_or(after);
    }
    (depth, text)
}

/// What a next line's citation aims at, when its first text is `(` immediately
/// followed by the marker: the same target test every other attempt meets, so
/// the checker keeps only the ones aimed at value authority (§FS-values.3.1.1).
fn opening_value_citation(
    line: &CitationLine<'_>,
    text: &str,
    workspace_targets: &[WorkspaceCitationTarget],
) -> Option<AttemptedValueTarget> {
    let text = text.trim_start();
    if !text
        .strip_prefix('(')?
        .starts_with(&line.schema.citation.marker)
    {
        return None;
    }
    attempted_value_target(text, line.schema, line.frame, workspace_targets)
}
