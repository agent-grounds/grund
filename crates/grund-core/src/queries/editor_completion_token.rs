//! Active partial-token recognition uses the loaded target's grammar and the
//! existing qualified citation prefix (§FS-lsp.1.6.1, §FS-lsp.1.6.3).

use super::{CompletionProject, LspCompletionContext};
use crate::grammar::{QUALIFIED_CITATION_PREFIX, parse_longest_id_prefix};

pub(super) struct ActiveToken<'a> {
    pub start: usize,
    pub end: usize,
    pub introducer: &'a str,
    pub qualification: &'a str,
    pub prefix: &'a str,
    pub target: &'a CompletionProject,
}

/// Prefer the last active token, then the longest complete introducer ending
/// at its ID start. Token continuation is grammar-owned (§FS-lsp.1.6.3).
pub(super) fn active_token<'a>(
    context: &'a LspCompletionContext,
    owner: &'a CompletionProject,
    line: &'a str,
    cursor: usize,
) -> Option<ActiveToken<'a>> {
    let mut starts = Vec::new();
    for introducer in [&owner.config.marker, &owner.config.trigger] {
        if introducer.is_empty() {
            continue;
        }
        for (start, _) in line[..cursor].char_indices() {
            if line[start..cursor].starts_with(introducer) {
                starts.push((start, introducer.as_str()));
            }
        }
    }
    starts.sort_by(|(a, ap), (b, bp)| (b + bp.len(), bp.len()).cmp(&(a + ap.len(), ap.len())));
    for (start, introducer) in starts {
        let id_start = start + introducer.len();
        let raw = &line[id_start..cursor];
        // §FS-lsp.1.6.1: an escaped marker itself is also protected while
        // the cursor is still between the marker and the closing angle.
        if line[..start].ends_with('<') && line[id_start..].starts_with('>') {
            return None;
        }
        let (target, qualification, prefix) = if raw.contains('/') {
            if !context.qualified {
                return None;
            }
            let caps = QUALIFIED_CITATION_PREFIX.captures(raw)?;
            let alias = caps.name("namespace")?.as_str();
            let target = context
                .projects
                .iter()
                .find(|project| project.alias == alias)?;
            let end = caps.get(0)?.end();
            (target, &raw[..end], &raw[end..])
        } else {
            (owner, "", raw)
        };
        let grammar = &target.config.grammar;
        // §FS-lsp.1.6.3: section requests are not declared-ID requests. A
        // separator may itself be an ID-format literal, so parse it first.
        if parse_longest_id_prefix(prefix, grammar).is_some_and(|parsed| parsed.section.is_some()) {
            return None;
        }
        let tail_start = id_start + qualification.len();
        let tail = &line[tail_start..];
        let bound = tail
            .char_indices()
            .find(|(offset, _)| {
                [&owner.config.marker, &owner.config.trigger]
                    .iter()
                    .any(|p| !p.is_empty() && tail[*offset..].starts_with(p.as_str()))
            })
            .map_or(tail.len(), |(offset, _)| offset);
        let tail = &tail[..bound];
        let mut end = tail_start;
        for (offset, ch) in tail.char_indices() {
            let rest = &tail[offset..];
            if [&owner.config.marker, &owner.config.trigger]
                .iter()
                .any(|p| !p.is_empty() && rest.starts_with(p.as_str()))
            {
                break;
            }
            if grammar.id_token_continues_with(ch) || ch == '-' {
                end = tail_start + offset + ch.len_utf8();
            } else {
                break;
            }
        }
        // §FS-lsp.1.6.3: compiled regexes also recognize component punctuation
        // absent from format literals. Partial prefixes inherit the actual
        // catalog spellings; the old suffix uses the existing full-ID parser.
        if let Some(parsed) = parse_longest_id_prefix(tail, grammar) {
            end = end.max(tail_start + parsed.len);
        }
        for candidate in &target.candidates {
            let shared = tail
                .chars()
                .zip(candidate.id.chars())
                .take_while(|(a, b)| a == b)
                .map(|(ch, _)| ch.len_utf8())
                .sum::<usize>();
            end = end.max(tail_start + shared);
        }
        // §FS-lsp.1.6.3: also remove a real section on a retained legacy ID,
        // while leaving punctuation followed by prose outside the edit.
        if let Some(section) =
            tail[end - tail_start..].strip_prefix(&target.config.section_separator)
        {
            let len = section
                .char_indices()
                .map(|(offset, _)| offset)
                .chain(std::iter::once(section.len()))
                .filter(|offset| *offset > 0)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .find(|offset| grammar.is_section_path(&section[..*offset]));
            if let Some(len) = len {
                end += target.config.section_separator.len() + len;
            }
        }
        // §FS-lsp.1.6.3: terminal sentence punctuation remains outside the
        // token; a section separator continues it only with following text.
        while !target.config.section_separator.is_empty()
            && end > cursor
            && line[..end].ends_with(&target.config.section_separator)
        {
            end -= target.config.section_separator.len();
        }
        if end < cursor {
            continue;
        }
        return Some(ActiveToken {
            start,
            end,
            introducer,
            qualification,
            prefix,
            target,
        });
    }
    None
}
