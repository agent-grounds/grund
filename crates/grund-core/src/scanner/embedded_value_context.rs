//! Host-context recognition and heading normalization for embedded values
//! (§FS-values.2.4, §FS-values.9).

use std::cell::OnceCell;
use std::collections::BTreeMap;
use std::path::Path;

use super::value_context::SourceValueLineContext;
use crate::config::FileReading;
use crate::grammar::named_mode_section_path;
use crate::grammar::{
    LexicalSettings, PythonDocstringScanState, is_lone_block_marker, longest_form_at, prefix_forms,
    source_scan_line,
};
use crate::model::{
    Catalog, Declaration, DeclarationSource, Id, InvalidValueSite, SectionInfo, ValueRootOrigin,
};

pub(crate) const EMBEDDED_VALUE_MARKER: &str = "<!-- grund:value -->";

/// The finding a marker on a heading that is no numeric section earns (§FS-values.2.4).
pub(super) const MISPLACED_MARKER: &str =
    "embedded value marker must be on a citable numeric section heading";

/// Return the marker's byte offset only for the one authored suffix that grants
/// authority: one ASCII separator space, exact lowercase marker bytes, then
/// optional trailing whitespace (§FS-values.2.4). Lookalikes stay prose.
pub(crate) fn exact_embedded_value_marker(line: &str) -> Option<usize> {
    let trimmed = line.trim_end_matches([' ', '\t']);
    // `strip_suffix` proves the byte boundary as well as the suffix. Computing
    // an arbitrary byte offset and slicing there can land inside any preceding
    // non-ASCII character (§REQ-never-crashes, §FS-values.9).
    let before = trimmed.strip_suffix(EMBEDDED_VALUE_MARKER)?;
    let marker_start = before.len();
    let title = before.strip_suffix(' ')?;
    if title.ends_with(char::is_whitespace) || title.is_empty() {
        return None;
    }
    Some(marker_start)
}

/// A marked root's semantic title: `text` without its tag and the space before
/// it (§FS-values.2.4.2). Text that does not end in the exact tag is returned
/// as it is.
pub(crate) fn without_value_tag(text: &str) -> &str {
    exact_embedded_value_marker(text).map_or(text, |start| text[..start].trim_end())
}

fn without_source_block_close(line: &str, block_comment: bool) -> &str {
    if !block_comment {
        return line;
    }
    let trimmed = line.trim_end_matches([' ', '\t']);
    trimmed
        .strip_suffix("*/")
        .map(|before| before.trim_end_matches([' ', '\t']))
        .unwrap_or(line)
}

/// Enroll a marker only after the host line has been classified. Markdown and
/// enabled Python docstrings already provide semantic content; every other
/// source form must put the marker bytes inside the recognized comment span.
pub(super) fn embedded_value_marker_for_line(
    line: &str,
    markdown: bool,
    in_py_docstring: bool,
    source_context: Option<SourceValueLineContext>,
) -> Option<usize> {
    if markdown || in_py_docstring {
        return exact_embedded_value_marker(line);
    }
    let context = source_context?;
    let semantic_line = without_source_block_close(line, context.block_comment);
    let marker = exact_embedded_value_marker(semantic_line)?;
    context
        .contains(marker, marker + EMBEDDED_VALUE_MARKER.len())
        .then_some(marker)
}

/// Every line of a file with its source wrapper resolved, as the two value
/// validators read it: the semantic text, its authored column offset, whether
/// it is inside an enabled Python docstring, and whether it is inside a block
/// comment (§FS-values.2.4.1).
pub(super) fn normalized_value_lines(
    text: &str,
    reading: &FileReading<'_>,
    source_contexts: Option<&[Option<SourceValueLineContext>]>,
) -> Vec<ValueLine> {
    let mut py_docstring = PythonDocstringScanState::default();
    text.lines()
        .enumerate()
        .map(|(index, line)| {
            let scan = source_scan_line(
                line,
                reading.python,
                reading.lexical.docstring_python,
                &mut py_docstring,
            );
            (
                scan.text.to_string(),
                scan.column_offset,
                scan.in_py_docstring,
                source_contexts
                    .and_then(|contexts| contexts.get(index).copied().flatten())
                    .is_some_and(|context| context.block_comment),
            )
        })
        .collect()
}

/// One line as [`normalized_value_lines`] reads it.
pub(super) type ValueLine = (String, usize, bool, bool);

/// A file's lines as the value validators read them (§FS-values.2.4.1), normalized on
/// first use, so a pass that only sometimes meets a value root pays only when it does.
pub(super) struct ValueLines<'a> {
    text: &'a str,
    pub(super) reading: &'a FileReading<'a>,
    contexts: Option<&'a [Option<SourceValueLineContext>]>,
    lines: OnceCell<Vec<ValueLine>>,
}

impl<'a> ValueLines<'a> {
    pub(super) fn new(
        text: &'a str,
        reading: &'a FileReading<'a>,
        contexts: Option<&'a [Option<SourceValueLineContext>]>,
    ) -> Self {
        let lines = OnceCell::new();
        Self {
            text,
            reading,
            contexts,
            lines,
        }
    }

    pub(super) fn lines(&self) -> &[ValueLine] {
        self.lines
            .get_or_init(|| normalized_value_lines(self.text, self.reading, self.contexts))
    }
}

/// What the physical cursor of the value root `root` reads (§FS-values.2.4.3): each line
/// of the root's subtree with content, from the line after the root heading to the line
/// before the next heading no deeper than it, or `end`, with the path the line is read at.
/// That is the path the record holds for a relative heading (`relative`, §FS-values.2.4),
/// else the heading's authored coordinate, title or none, name-bearing only under a
/// chapter root (§FS-values.2.5). The root's own check walks it, and so does the repair
/// shown under the root (§FS-declarations.checks.unmarked-heading), so both count the
/// same components.
pub(super) fn root_cursor<'n>(
    normalized: &'n [ValueLine],
    root: &SectionInfo,
    end: usize,
    relative: &'n BTreeMap<usize, String>,
    is_md: bool,
    lexical: LexicalSettings<'n>,
) -> impl Iterator<Item = (usize, &'n ValueLine, Option<String>)> + use<'n> {
    let (root_line, root_level) = (root.line, root.heading_level);
    // §FS-values.2.5: only a chapter root's path carries names, so a marked root's
    // subtree keeps the numeric-only grammar it had before the key existed (§FS-values.9).
    let chapter_root = root
        .value_root
        .as_ref()
        .is_some_and(|root| matches!(root.origin, ValueRootOrigin::Chapter));
    let subtree_end = ((root_line + 1)..=end)
        .find(|line_no| {
            normalized
                .get(line_no.saturating_sub(1))
                .and_then(|(line, _, docstring, block_comment)| {
                    authored_heading_level(line, is_md || *docstring, *block_comment, lexical)
                })
                .is_some_and(|level| level <= root_level)
        })
        .map(|line| line - 1)
        .unwrap_or(end);
    (root_line.saturating_add(1)..=subtree_end).filter_map(move |line_no| {
        let entry = normalized.get(line_no - 1)?;
        let (line, _, docstring, block_comment) = entry;
        let markdown = is_md || *docstring;
        let content = semantic_comment_content(line, markdown, *block_comment, lexical);
        if content.is_empty() || is_lone_block_marker(content) {
            return None;
        }
        let child = relative.get(&line_no).cloned().or_else(|| {
            authored_heading_path(line, markdown, *block_comment, chapter_root, lexical)
        });
        Some((line_no, entry, child))
    })
}

pub(super) fn push_invalid_embedded_marker(
    findings: &mut Catalog,
    id: Option<Id>,
    path: &Path,
    line: usize,
    column_offset: usize,
    marker_start: usize,
    message: &str,
) {
    let column = column_offset + marker_start + 1;
    let site = invalid_embedded_marker(id, path, line, column, message);
    findings.invalid_value_declarations.push(site);
}

/// The `invalid-value-declaration` a marker at `column` of `line` earns.
pub(super) fn invalid_embedded_marker(
    id: Option<Id>,
    path: &Path,
    line: usize,
    column: usize,
    message: &str,
) -> InvalidValueSite {
    InvalidValueSite {
        id,
        file: path.to_path_buf(),
        line,
        column: Some(column),
        message: message.to_string(),
        source: DeclarationSource::Text,
        binding_namespace: None,
        binding_section: None,
        binding_citation: None,
    }
}

/// Whether `path` is an immediate numeric component of the value root at `root`:
/// `root`, a dot, and one all-digit component, the only heading a root's physical
/// cursor advances on (§FS-values.2.4.3). A grandchild or a named child is not one.
pub(super) fn immediate_numeric_component(root: &str, path: &str) -> bool {
    path.strip_prefix(root)
        .and_then(|rest| rest.strip_prefix('.'))
        .is_some_and(|tail| !tail.is_empty() && tail.bytes().all(|byte| byte.is_ascii_digit()))
}

/// The line of each heading of `decl` written in the relative form, with the
/// canonical path the record holds for it, a duplicate's included (§FS-values.2.4).
pub(super) fn relative_heading_paths(decl: &Declaration) -> BTreeMap<usize, String> {
    let duplicates = decl
        .duplicate_sections
        .iter()
        .map(|(path, info)| (path, info));
    decl.sections
        .iter()
        .chain(duplicates)
        .filter(|(_, info)| info.relative)
        .map(|(path, info)| (info.line, path.clone()))
        .collect()
}

pub(super) fn component_without_block_close(component: &str, block_comment: bool) -> &str {
    without_source_block_close(component, block_comment)
}

/// Heading depth after the same configured wrapper accepted by declaration and
/// section scanning has been removed (§FS-values.2.4.1). This deliberately does
/// not decide whether the heading is citable; it also identifies plain/named
/// headings that are forbidden inside a strict embedded root.
pub(super) fn authored_heading_level(
    line: &str,
    markdown: bool,
    block_comment: bool,
    lexical: LexicalSettings<'_>,
) -> Option<usize> {
    // Strip a source wrapper—including `#`—before authored heading hashes;
    // Python docstrings arrive as Markdown after quote normalization
    // (§FS-values.2.4.1).
    let content = if markdown {
        line.trim_start()
    } else {
        semantic_comment_content(line, false, block_comment, lexical).trim_start()
    };
    let level = content.bytes().take_while(|byte| *byte == b'#').count();
    (level > 0
        && content[level..]
            .chars()
            .next()
            .is_none_or(char::is_whitespace))
    .then_some(level)
}

/// A heading coordinate before title validation. This recognizes the physical
/// child slot even when the title is absent, so the component's own error does
/// not manufacture a second zero-component error at its root. `named` asks for
/// the name-bearing coordinate form as well, because a chapter root's own path
/// carries it (§FS-values.2.5); whether the *tail* below a root is numeric
/// stays the caller's question (§FS-values.2.4.3). Only a chapter's own level
/// and a chapter root's subtree pass it: marker authority governs everywhere
/// else and reads exactly the numeric-only grammar it always did, so a project
/// that declares no chapter is unaffected (§FS-values.2.5, §FS-values.9).
pub(super) fn authored_heading_path(
    line: &str,
    markdown: bool,
    block_comment: bool,
    named: bool,
    lexical: LexicalSettings<'_>,
) -> Option<String> {
    let content = if markdown {
        line.trim_start()
    } else {
        semantic_comment_content(line, false, block_comment, lexical).trim_start()
    };
    let level = content.bytes().take_while(|byte| *byte == b'#').count();
    let rest = content.get(level..)?;
    if level == 0 || !rest.chars().next().is_some_and(char::is_whitespace) {
        return None;
    }
    let token = rest.trim_start().split_whitespace().next()?;
    // §FS-config.3.3.1: a numeric heading's full stop is optional punctuation
    // and a name-bearing one's colon is mandatory; neither is part of the path.
    if let Some(coordinate) = (named && lexical.grammar.named_sections)
        .then(|| token.strip_suffix(':'))
        .flatten()
    {
        let well_formed = named_mode_section_path(coordinate, |part| {
            !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit())
        });
        return well_formed.then(|| coordinate.to_string());
    }
    let coordinate = token.strip_suffix('.').unwrap_or(token);
    coordinate
        .split('.')
        .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| coordinate.to_string())
}

pub(super) fn semantic_comment_content<'a>(
    line: &'a str,
    markdown: bool,
    block_comment: bool,
    lexical: LexicalSettings<'_>,
) -> &'a str {
    let mut content = line.trim();
    if markdown {
        return content;
    }
    if is_lone_block_marker(content) {
        return "";
    }
    // §FS-languages.prefixes.forms: the marker is the longest form of one of the
    // file's hosts (§FS-languages.prefixes), so `;;` and `-- |` go whole as `///` does.
    let prefixes = lexical
        .comment_prefixes
        .iter()
        .map(|prefix| prefix_forms(prefix))
        .collect::<Vec<_>>();
    if let Some(len) = longest_form_at(content, &prefixes) {
        content = content[len..].trim();
    }
    without_source_block_close(content, block_comment).trim()
}
