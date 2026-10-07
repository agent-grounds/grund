//! Rendered heading text for section and declaration anchors (§FS-fmt.6.2.2,
//! §DF-declaration-anchor). Reducing links and HTML outside inline code is a
//! lexical fact shared by readers and writers (§AR-system.2.1).

use once_cell::sync::Lazy;
use regex::Regex;

/// An inline Markdown link `[text](url)` — used to reduce a heading to the text a
/// renderer would slugify (the destination URL is not part of that text), so an
/// anchor stays correct even when a citation in a section heading has been wrapped
/// by `grund fmt --cross-refs` (§DF-github-anchor-fidelity, §FS-fmt.6.2.2).
static MD_INLINE_LINK: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[([^\]]*)\]\([^)]*\)").unwrap());
/// An HTML-tag-shaped span `<…>` — a renderer drops it from a heading's text
/// (`## RM-read: grund <ID>` slugs as `rm-read-grund`), so it must be removed
/// before slugging the heading (§DF-github-anchor-fidelity, §FS-fmt.6.2.2).
static HTML_TAG: Lazy<Regex> = Lazy::new(|| Regex::new(r"<[^>]*>").unwrap());

/// Reduce a heading's source text to the text content a Markdown renderer would
/// slugify: an inline link `[text](url)` shows as `text`, an HTML-tag span `<…>`
/// is dropped. Used by both the section-anchor and declaration-anchor paths
/// (§DF-github-anchor-fidelity, §DF-declaration-anchor).
///
/// Neither reduction applies **inside an inline code span**, because a renderer
/// resolves code spans first and everything within one is literal text. GitHub
/// slugs `### 8.1 \`grund <alias>/<ID>\`` as `81-grund-aliasid`, not
/// `81-grund-`: the angle brackets are content there, not a tag. Stripping them
/// anyway produced an anchor that resolves nowhere, and every heading that
/// documents a placeholder-carrying command has that shape.
pub(crate) fn reduce_heading_text(text: &str) -> String {
    let mut out = String::new();
    for (segment, is_code) in inline_code_segments(text) {
        if is_code {
            out.push_str(segment);
        } else {
            out.push_str(&HTML_TAG.replace_all(&MD_INLINE_LINK.replace_all(segment, "$1"), ""));
        }
    }
    out
}

/// Split heading text into its non-code and inline-code segments, flagged
/// `true` for code. A span opens on a run of *n* backticks and closes on the
/// next run of exactly *n* (CommonMark); a run that never closes is ordinary
/// text, which is why the scan continues past it rather than swallowing the
/// rest of the heading.
fn inline_code_segments(text: &str) -> Vec<(&str, bool)> {
    let bytes = text.as_bytes();
    let mut segments = Vec::new();
    let mut plain_start = 0;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'`' {
            i += 1;
            continue;
        }
        let open = i;
        while i < bytes.len() && bytes[i] == b'`' {
            i += 1;
        }
        let fence = i - open;
        let mut scan = i;
        let close = loop {
            while scan < bytes.len() && bytes[scan] != b'`' {
                scan += 1;
            }
            if scan >= bytes.len() {
                break None;
            }
            let run = scan;
            while scan < bytes.len() && bytes[scan] == b'`' {
                scan += 1;
            }
            if scan - run == fence {
                break Some(scan);
            }
        };
        if let Some(stop) = close {
            if plain_start < open {
                segments.push((&text[plain_start..open], false));
            }
            segments.push((&text[open..stop], true));
            plain_start = stop;
            i = stop;
        }
    }
    if plain_start < text.len() {
        segments.push((&text[plain_start..], false));
    }
    segments
}
