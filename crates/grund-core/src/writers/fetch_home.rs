//! Reading a snapshot home for the explicit materializer: which declarations it holds at the
//! home's depth, the bytes each one owns, and the refusal of a declaration inside them
//! (§FS-fetch.4, §FS-fetch.5, §FS-fetch.6.nested). `fetch_write.rs` decides from these and writes.

use std::path::{Path, PathBuf};

use super::fetch::{FetchFailure, fetch_operational};
use crate::grammar::{
    Grammar, MarkdownBlocks, markdown_line_records, near_miss_heading, parse_id_arg,
    parse_longest_id_prefix,
};
use crate::model::{Id, format_path, relative_from_base};
use crate::scanner::markdown_heading_level;

/// The ID a fetch writes, with the two spellings its refusal needs (§FS-fetch.6.nested):
/// the ID as the command named it, and the base its paths are reported from (§FS-errors.terms).
pub(super) struct Requested<'a> {
    pub(super) id: &'a Id,
    pub(super) raw: &'a str,
    pub(super) report_base: &'a Path,
}

#[derive(Clone)]
pub(super) struct SnapshotDeclaration {
    pub(super) id: Id,
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) path: Option<PathBuf>,
}

fn home_text(bytes: &[u8]) -> std::result::Result<&str, FetchFailure> {
    std::str::from_utf8(bytes).map_err(|_| fetch_operational("snapshot home is not UTF-8 Markdown"))
}

/// A line of `split_inclusive('\n')` without its ending, as the scanner reads it.
fn line_content(line: &str) -> &str {
    let without_newline = line.strip_suffix('\n').unwrap_or(line);
    without_newline
        .strip_suffix('\r')
        .unwrap_or(without_newline)
}

/// §FS-fetch.6.nested: what fetch owns ends at another declaration, so refuse where the bytes it
/// would replace hold, after the heading line, a line the next scan records. The blocks are read
/// from the top of the file and `<n>` is a line of that file. A line only partly in the range is
/// the next heading's, whose indentation alone falls inside it.
pub(super) fn refuse_nested_declaration(
    bytes: &[u8],
    declaration: &SnapshotDeclaration,
    path: &Path,
    grammar: &Grammar,
    requested: &Requested<'_>,
    way_out: &str,
) -> std::result::Result<(), FetchFailure> {
    let text = home_text(bytes)?;
    let mut blocks = MarkdownBlocks::file(text);
    let mut offset = 0usize;
    for (index, line) in text.split_inclusive('\n').enumerate() {
        let content = line_content(line);
        let block = blocks.line(content);
        if offset > declaration.start
            && offset + line.len() <= declaration.end
            && block.may_be_heading()
            && let Some(nested) = markdown_line_records(grammar, content)
        {
            return Err(fetch_operational(format!(
                "{}:{}: fetching `{}` would delete `{nested}`, declared inside it; \
                 move `{nested}` {way_out} or delete it, then fetch again",
                format_path(&relative_from_base(requested.report_base, path)),
                index + 1,
                requested.raw,
            )));
        }
        offset += line.len();
    }
    Ok(())
}

pub(super) fn declarations_at_depth(
    bytes: &[u8],
    grammar: &Grammar,
    depth: usize,
) -> std::result::Result<Vec<SnapshotDeclaration>, FetchFailure> {
    let text = home_text(bytes)?;
    let mut headings = Vec::<(usize, Option<Id>)>::new();
    let mut offset = 0usize;
    // §FS-check.1.1.5.2: the whole snapshot home, so its front matter holds no heading.
    let mut blocks = MarkdownBlocks::file(text);
    for line in text.split_inclusive('\n') {
        let content = line_content(line);
        let trimmed = content.trim_start();
        // §FS-check.1.1.5.1, §FS-check.1.1.5.2: neither a fence, a raw-text HTML block
        // nor front matter holds a heading.
        if !blocks.line(content).may_be_heading() {
            offset += line.len();
            continue;
        }
        let hashes = trimmed.bytes().take_while(|byte| *byte == b'#').count();
        if markdown_heading_level(trimmed) == Some(depth) {
            let start = offset + content.len() - trimmed.len();
            let tail = trimmed[hashes..].trim_start();
            let id = match tail.split_once(':') {
                Some((raw, title)) => match parse_id_arg(raw.trim(), grammar) {
                    Ok((id, None)) if !title.trim().is_empty() => Some(id),
                    Ok((_, None)) => {
                        return Err(fetch_operational(format!(
                            "snapshot home has malformed declaration heading `{}`",
                            tail.trim()
                        )));
                    }
                    _ if near_miss_heading(grammar, trimmed, false, true).is_some() => {
                        return Err(fetch_operational(format!(
                            "snapshot home has malformed declaration heading `{}`",
                            tail.trim()
                        )));
                    }
                    _ => None,
                },
                None if parse_longest_id_prefix(tail, grammar).is_some() => {
                    return Err(fetch_operational(format!(
                        "snapshot home has malformed declaration heading `{}`",
                        tail.trim()
                    )));
                }
                None => None,
            };
            headings.push((start, id));
        }
        offset += line.len();
    }
    Ok(headings
        .iter()
        .enumerate()
        .filter_map(|(index, (start, id))| {
            let id = id.as_ref()?;
            // The blank separator before the next sibling is outside the
            // declaration-local replacement and therefore remains byte-for-byte.
            let end = headings.get(index + 1).map_or(bytes.len(), |next| {
                let end = next.0;
                if bytes[..end].ends_with(b"\r\n\r\n") {
                    end - 2
                } else if bytes[..end].ends_with(b"\n\n") {
                    end - 1
                } else {
                    end
                }
            });
            Some(SnapshotDeclaration {
                id: id.clone(),
                start: *start,
                end,
                path: None,
            })
        })
        .collect())
}
