//! What it means for an index to **mention** an ID it holds no entry for
//! (§FS-check.3.18.5.1) — the look `check_kind_indexes` words its missing-entry
//! finding on, in a file of its own beside the rule that raises it
//! (§AR-core-module-layout.1). It decides nothing and raises nothing
//! (§FS-check.3.18.6): it answers one question about the index's text.

use std::collections::BTreeSet;

use crate::config::Frame;
use crate::grammar::{MarkdownBlocks, parse_id};
use crate::model::{Declaration, Id};

/// Whether `line` names `file_name` as a **whole** file name
/// (§FS-check.3.18.5.1). The character on either side must be one that cannot
/// continue a file name, so `overview.md` is named by neither `my-overview.md`
/// nor `overview.markdown`, while a path ending in the name still names that
/// file, because `/` cannot continue one either.
fn line_names_file(line: &str, file_name: &str) -> bool {
    if file_name.is_empty() {
        return false;
    }
    // A non-ASCII neighbour reads as a boundary: it is not a byte any file name
    // in a declaration path is built out of, and testing bytes keeps this from
    // slicing into one (§REQ-never-crashes).
    let continues =
        |byte: u8| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b'~');
    let bytes = line.as_bytes();
    let mut cursor = 0;
    while let Some(relative) = line[cursor..].find(file_name) {
        let start = cursor + relative;
        let end = start + file_name.len();
        cursor = end;
        let opens = start == 0 || !continues(bytes[start - 1]);
        let closes = end == bytes.len() || !continues(bytes[end]);
        if opens && closes {
            return true;
        }
    }
    false
}

/// Which of the index's unentered covered IDs its text **mentions**
/// (§FS-check.3.18.5.1) — the evidence §FS-check.3.18's missing-entry finding
/// words itself on, and never a finding of its own (§FS-check.3.18.6).
///
/// One pass over the index's lines. A line inside a fenced block is not the page
/// saying anything, by the one fence reader every Markdown surface shares
/// (§FS-check.1.1.5), and a declaration heading declares the ID rather than
/// mentioning it — the same test the entry-form pass applies (§FS-fmt.6.4).
/// Inline code is deliberately *not* excluded, which is what lets one sentence be
/// true both of a citation the scan recorded and discarded (§FS-check.3.17.4) and
/// of a markerless mention it never recorded at all.
///
/// Whether a 1-indexed line is a declaration heading is `declaration_heading`'s
/// answer, the one `KindIndexEntries` gives the entry rule and the unused carve-out
/// alike (§AR-checker.2.16), off the scan's record of the index (§AR-scanner.2.8.5).
/// The scan records no heading in a raw-text HTML block (§FS-check.1.1.5.1), so a
/// heading-shaped line in one declares nothing and what it names is a mention.
///
/// Called once per index target, and only where that target already owes at
/// least one finding, so an index whose declarations are all entered — every
/// index in a green repository — pays nothing (§GOAL-fast-feedback).
pub(super) fn index_mentions<'a>(
    frame: Frame<'_>,
    marker: &str,
    declaration_heading: impl Fn(usize) -> bool,
    lines: &[&str],
    unentered: &[(&'a Id, &'a Declaration)],
) -> BTreeSet<&'a Id> {
    let mut mentioned: BTreeSet<&Id> = BTreeSet::new();
    // §FS-check.1.1.5.2: the whole index, so a fence opener in its front matter opens nothing.
    let mut blocks = MarkdownBlocks::lines(lines.iter().copied());
    for (index, line) in lines.iter().enumerate() {
        let block = blocks.line(line);
        if block.in_fence() {
            continue;
        }
        if declaration_heading(index + 1) {
            continue;
        }
        // §AR-scanner.2.8.5: the scanner's own token reader, so a longer ID containing
        // this one parses as itself and is not a mention of it, while a namespaced,
        // section-suffixed or markerless occurrence, or a pattern's ID head, names it.
        let tokens = frame.grammar().citation_captures(line, marker);
        for (_, caps) in tokens.captures.iter().chain(&tokens.pattern_heads) {
            let Some(found) = parse_id(caps, frame.grammar()) else {
                continue;
            };
            for (id, _) in unentered {
                if **id == found {
                    mentioned.insert(*id);
                }
            }
        }
        // The half no ID-token search can reach: an index row that names the
        // declaration's file and nothing else — the row in this rule's own
        // reproducer.
        for (id, decl) in unentered {
            if decl
                .file
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| line_names_file(line, name))
            {
                mentioned.insert(*id);
            }
        }
    }
    mentioned
}
