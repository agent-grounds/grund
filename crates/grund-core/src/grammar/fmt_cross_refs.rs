//! The cross-reference wrapper read backwards (§FS-fmt.6.2,
//! §DF-show-cross-ref-flattening): whether a `[…](…)` label on a line is a
//! citation the formatter could have wrapped, and the flattening that undoes
//! exactly the wrap `writers/fmt_links.rs` writes. Beside the wrap test sits the
//! reading every caller asks of a citation's brackets, whether it is a wrap, in a
//! link its author wrote, or unwrapped (§FS-citations.wrap).
//!
//! Lexical, and here rather than beside the wrapper it inverts because
//! recognizing the shape is all either side does: the wrap is a `[` immediately
//! before a marker-prefixed citation token and `](…)` immediately after it, and
//! nothing about it is resolved — a dangling citation flattens the same as a
//! live one, and `grund check` still reports it. Four consumers read the
//! flattening for an answer that must agree with the formatter line for line —
//! single and batch `show`, the point body a size is measured on and the
//! deprecated adapter (§DF-show-cross-ref-flattening.2.1, §FS-list.3.4.1) — and
//! while it sat in the writers each of them read a sibling or a component above
//! it (§AR-system.4).
//!
//! The label predicate is private, which it could not be before. It sat in
//! `scanner/legacy.rs`, with the off-grammar catalog steps whose spellings it
//! accepts (§FS-config.3.2.5), and the flattening read it from there across two
//! component boundaries; now that the only caller is the line pass below, one
//! file holds both and nothing outside names either.

use super::compiled::{Grammar, QUALIFIED_CITATION_PREFIX};
use super::markdown_blocks::MarkdownBlocks;
use super::never_rewrite::{
    inline_code_span_end, is_escaped, is_inside_inline_code, link_destination_close,
};
use super::settings::LexicalSettings;
use super::shorthand::parse_id_arg_with_shorthand;

/// Whether `label` is a citation-shaped wrapper label the `--cross-refs` pass
/// can emit (§FS-show.3.2.1): the configured grammar's own spelling, its
/// number-only shorthand, or a persisted off-grammar one, qualified or not
/// (§FS-config.3.2.5). An ordinary link whose label merely starts with the marker
/// is not one.
fn formatter_wrapper_label_is_citation(label: &str, grammar: &Grammar) -> bool {
    let tail = match QUALIFIED_CITATION_PREFIX.captures(label) {
        Some(prefix) => &label[prefix.get(0).expect("qualified prefix match").end()..],
        None => label,
    };
    if tail.is_empty() || tail.contains('/') {
        return false;
    }
    parse_id_arg_with_shorthand(tail, grammar).is_ok()
        || grammar.legacy_kind_and_format(tail).is_some()
}

/// A cross-reference wrap around one citation, as its brackets show it
/// (§FS-citations.wrap.recognized): the destination opens at `url_start` and
/// ends at `close`, the `)` that balances its `(` by the one extent of
/// §FS-citations.zones.link-destination.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CrossRefWrap {
    pub(crate) url_start: usize,
    pub(crate) close: usize,
}

/// The one bracket test of §FS-citations.wrap.recognized: the citation spanning
/// `start..end` is wrapped when the byte before it is `[`, the text after it
/// begins `](`, and that destination closes on the line. A `[§<ID>](` that never
/// closes is no wrap, whatever later `)` stands on the line. Asked first by
/// `citation_link`, the reading every caller asks, so none of them can recognize
/// a wrap the others do not.
pub(super) fn cross_ref_wrap(line: &str, start: usize, end: usize) -> Option<CrossRefWrap> {
    if start == 0 || line.as_bytes().get(start - 1) != Some(&b'[') {
        return None;
    }
    if !line.get(end..)?.starts_with("](") {
        return None;
    }
    let url_start = end + 2;
    let close = link_destination_close(line.as_bytes(), url_start)?;
    Some(CrossRefWrap { url_start, close })
}

/// How one citation on a Markdown line sits among the brackets around it: the
/// three readings of §FS-citations.wrap.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CitationLink {
    /// A wrap `fmt` keeps re-deriving (§FS-citations.wrap.recognized).
    Wrap(CrossRefWrap),
    /// In the text of an enclosing link its author wrote, whose destination opens
    /// at `url_start` and ends at `close`, the `)` that balances its `(`; the two
    /// are equal where it is empty (§FS-citations.wrap.enclosing).
    Enclosing { url_start: usize, close: usize },
    /// Neither, so the link pass wraps it fresh wherever a zone does not forbid it.
    Unwrapped,
}

/// The three-way reading of §FS-citations.wrap for the citation spanning
/// `start..end`, the one `fmt`'s link pass, `check`'s kind-index entry form and
/// its external enrollment, and `show`'s flatten all ask, so none of them can
/// take a link the others leave to its author. The wrap test runs first: a
/// citation whose own brackets make a wrap is one, whatever brackets stand around
/// it (§FS-citations.wrap.enclosing.wrap-wins).
pub(crate) fn citation_link(line: &str, start: usize, end: usize) -> CitationLink {
    if let Some(wrap) = cross_ref_wrap(line, start, end) {
        return CitationLink::Wrap(wrap);
    }
    match enclosing_link(line.as_bytes(), start, end) {
        Some((url_start, close)) => CitationLink::Enclosing { url_start, close },
        None => CitationLink::Unwrapped,
    }
}

/// §FS-citations.wrap.enclosing.shape: the destination `(url_start, close)` of the
/// innermost link whose text holds `start..end`, read on the one line. A bracket
/// that is backslash-escaped or inside an inline-code span counts for nothing, and
/// every destination is passed over whole, by the one extent of
/// §FS-citations.zones.link-destination. A citation in a code span or in a
/// destination is in no link's text, and a destination that never closes makes no
/// link and leaves the rest of the line a destination.
fn enclosing_link(bytes: &[u8], start: usize, end: usize) -> Option<(usize, usize)> {
    if start > end || end > bytes.len() {
        return None;
    }
    // The openers still open where the citation starts, innermost last, each
    // `true` when it opens an image's description.
    let mut openers: Vec<bool> = Vec::new();
    let mut i = 0;
    while i < start {
        match bytes[i] {
            b'`' if !is_escaped(bytes, i) => {
                let span_end = inline_code_span_end(bytes, i)?;
                if span_end > start {
                    return None;
                }
                i = span_end;
                continue;
            }
            b'[' if !is_escaped(bytes, i) => openers.push(opens_image(bytes, i)),
            b']' if !is_escaped(bytes, i) => {
                openers.pop();
                if bytes.get(i + 1) == Some(&b'(') {
                    let close = link_destination_close(bytes, i + 2)?;
                    if close > start {
                        return None;
                    }
                    i = close + 1;
                    continue;
                }
            }
            _ => {}
        }
        i += 1;
    }
    // Walk on to the `]` that closes each opener, innermost first: the first that
    // is a link's, `](` and a destination that closes, encloses the citation.
    let mut nested = 0usize;
    let mut i = end;
    while i < bytes.len() && !openers.is_empty() {
        match bytes[i] {
            b'`' if !is_escaped(bytes, i) => {
                i = inline_code_span_end(bytes, i)?;
                continue;
            }
            b'[' if !is_escaped(bytes, i) => nested += 1,
            b']' if !is_escaped(bytes, i) => {
                let destination = match bytes.get(i + 1) {
                    Some(b'(') => Some(link_destination_close(bytes, i + 2)?),
                    _ => None,
                };
                if nested > 0 {
                    nested -= 1;
                } else if let (Some(close), Some(false)) = (destination, openers.pop()) {
                    return Some((i + 2, close));
                }
                if let Some(close) = destination {
                    i = close + 1;
                    continue;
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// §FS-citations.wrap.enclosing.images: a `[` right after an unescaped `!` opens
/// an image's description, which balances like any bracket and never encloses.
fn opens_image(bytes: &[u8], open: usize) -> bool {
    open > 0 && bytes[open - 1] == b'!' && !is_escaped(bytes, open - 1)
}

/// Flatten `grund fmt --cross-refs` link wrappers before an ID query
/// prints it in `text` / `json` (§FS-show.3.2, §DF-show-cross-ref-flattening):
/// `[§[alias/]<ID>.<section>](path#anchor)` → `§[alias/]<ID>.<section>`. The inverse of
/// `wrap_markdown_links` (§FS-fmt.6.2) — the wrap shape is a `[` immediately
/// before a marker-prefixed citation token and `](…)` immediately after it,
/// exactly what `grund fmt --cross-refs` emits and re-derives (§FS-fmt.6.3.1); that
/// is the only thing flattened. Ordinary Markdown links, an unwrapped citation,
/// a citation inside an inline-code span (illustrative, like `fmt` itself —
/// §FS-fmt.6.4), and `--format md` output (kept verbatim by the caller) are all
/// left untouched. In a Markdown body, fence delimiters and their contents also
/// remain authored until the shared delimiter grammar closes the fence
/// (§FS-show.2.5, §FS-show.3.2.1, §FS-show.3.2.2). Purely textual: the citation
/// is never resolved, so a dangling one is flattened just the same and `grund
/// check` still reports it.
pub(crate) fn flatten_cross_ref_links(
    body: &str,
    settings: LexicalSettings<'_>,
    markdown_body: bool,
) -> String {
    if !body.contains("](") {
        return body.to_string();
    }
    let mut out = String::with_capacity(body.len());
    let mut markdown_blocks = MarkdownBlocks::default();
    for line in body.split_inclusive('\n') {
        // A fence delimiter or fenced content is verbatim (§FS-check.1.1.5); a
        // raw-text HTML block is flattened like prose, and a fence opener inside
        // one opens nothing (§FS-check.1.1.5.1).
        let fence_line = line.strip_suffix('\n').unwrap_or(line);
        let fence_line = fence_line.strip_suffix('\r').unwrap_or(fence_line);
        if markdown_body && markdown_blocks.line(fence_line).in_fence() {
            out.push_str(line);
        } else {
            out.push_str(&flatten_cross_ref_links_line(line, settings));
        }
    }
    out
}

fn flatten_cross_ref_links_line(line: &str, settings: LexicalSettings<'_>) -> String {
    let marker = settings.marker;
    if marker.is_empty() {
        return line.to_string();
    }
    let mut output = String::new();
    let mut cursor = 0usize;
    let wrapper_start = format!("[{marker}");
    for (bracket_pos, _) in line.match_indices(&wrapper_start) {
        let marker_start = bracket_pos + 1;
        let label_start = marker_start + marker.len();
        let Some(label_close_rel) = line[label_start..].find("](") else {
            continue;
        };
        let cite_end = label_start + label_close_rel;
        let token = &line[label_start..cite_end];
        // §FS-show.3.2.1: require a citation-shaped label the formatter can emit,
        // including persisted and qualified legacy spellings but excluding an
        // ordinary link whose label merely starts with the marker.
        if token.is_empty()
            || token
                .chars()
                .any(|ch| ch.is_whitespace() || matches!(ch, '[' | ']' | '(' | ')' | '`'))
            || !formatter_wrapper_label_is_citation(token, settings.grammar)
        {
            continue;
        }
        // A citation shown inside `` `…` `` is an illustration, not a citation —
        // leave it exactly as written, the same call `grund fmt --cross-refs` makes.
        if is_inside_inline_code(line, bracket_pos) {
            continue;
        }
        // §FS-show.3.2.1: the reading `fmt` asks, and only a wrap collapses, whole
        // to the balancing `)`; a wrap that never closes is printed as written.
        let CitationLink::Wrap(CrossRefWrap { close, .. }) =
            citation_link(line, marker_start, cite_end)
        else {
            continue;
        };
        if bracket_pos < cursor {
            continue;
        }
        output.push_str(&line[cursor..bracket_pos]);
        output.push_str(&line[marker_start..cite_end]); // §[alias/]<ID>[.<section>]
        cursor = close + 1;
    }
    output.push_str(&line[cursor..]);
    output
}
