//! The cross-reference wrapper read backwards (§FS-fmt.6.2,
//! §DF-show-cross-ref-flattening): whether a `[…](…)` label on a line is a
//! citation the formatter could have wrapped, and the flattening that undoes
//! exactly the wrap `writers/fmt_links.rs` writes.
//!
//! Lexical, and here rather than beside the wrapper it inverts because
//! recognizing the shape is all either side does: the wrap is a `[` immediately
//! before a marker-prefixed citation token and `](…)` immediately after it, and
//! nothing about it is resolved — a dangling citation flattens the same as a
//! live one, and `grund check` still reports it. Three components read the
//! flattening for an answer that must agree with the formatter line for line —
//! the `show` query, the point body a size is measured on and the deprecated
//! adapter (§FS-show.3.2, §FS-list.3.4) — and while it sat in the writers each
//! of them read a sibling or a component above it (§AR-system.4).
//!
//! The label predicate is private, which it could not be before. It sat in
//! `scanner/legacy.rs`, with the off-grammar catalog steps whose spellings it
//! accepts (§FS-config.3.2), and the flattening read it from there across two
//! component boundaries; now that the only caller is the line pass below, one
//! file holds both and nothing outside names either.

use super::compiled::{Grammar, QUALIFIED_CITATION_PREFIX};
use super::never_rewrite::is_inside_inline_code;
use super::settings::LexicalSettings;
use super::shorthand::parse_id_arg_with_shorthand;

/// Whether `label` is a citation-shaped wrapper label the `--cross-refs` pass
/// can emit (§FS-show.3.2): the configured grammar's own spelling, its
/// number-only shorthand, or a persisted off-grammar one, qualified or not
/// (§FS-config.3.2). An ordinary link whose label merely starts with the marker
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

/// Flatten `grund fmt --cross-refs` link wrappers before an ID query
/// prints it in `text` / `json` (§FS-show.3.2, §DF-show-cross-ref-flattening):
/// `[§[alias/]<ID>.<section>](path#anchor)` → `§[alias/]<ID>.<section>`. The inverse of
/// `wrap_markdown_links` (§FS-fmt.6.2) — the wrap shape is a `[` immediately
/// before a marker-prefixed citation token and `](…)` immediately after it,
/// exactly what `grund fmt --cross-refs` emits and re-derives (§FS-fmt.6.3); that
/// is the only thing flattened. Ordinary Markdown links, an unwrapped citation,
/// a citation inside an inline-code span (illustrative, like `fmt` itself —
/// §FS-fmt.6.4), and `--format md` output (kept verbatim by the caller) are all
/// left untouched. Purely textual: the citation is never resolved, so a dangling
/// one is flattened just the same and `grund check` still reports it.
pub(crate) fn flatten_cross_ref_links(body: &str, settings: LexicalSettings<'_>) -> String {
    if !body.contains("](") {
        return body.to_string();
    }
    let mut out = String::with_capacity(body.len());
    for line in body.split_inclusive('\n') {
        out.push_str(&flatten_cross_ref_links_line(line, settings));
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
        // §FS-show.3.2: require a citation-shaped label the formatter can emit,
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
        let rest = &line[cite_end + 2..];
        let Some(close_rel) = rest.find(')') else {
            continue;
        };
        let close = cite_end + 2 + close_rel; // index of the `)`
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
