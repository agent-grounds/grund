//! The grammar component (§AR-system.2.1): the lexical facts every other
//! component shares — the ID grammar compiled from `[id] format` and its
//! near-miss detection, comment-line and comment-block recognition,
//! fenced-block boundaries, the number-only shorthand, inline-note layout, and
//! the never-rewrite predicates (§FS-fmt.2.3). It consumes text and knows no
//! file, no rule and no frontend.
//!
//! The module boundary is what §AR-system.4 asks for: an item another component
//! reads is re-exported below, and everything else is the component's own
//! (§AR-core-module-layout.1). The submodules are the former `grammar*`,
//! `markdown_fence`, `comment_line`, `comment_block`, `shorthand*`,
//! `inline_note_layout`, `never_rewrite` and ID-grammar category files, one per
//! lexical fact §AR-system.2.1 names. Five items the flat layout parked in
//! scanner files came down here when §AR-system.2.5 became a module: the
//! off-grammar `LegacyGrammar`, the `{kind}` literal reader of the `[id] format`
//! template, the member-local fallback ID parser, the qualified-citation
//! suppression rule, and the two shorthand questions asked of a catalog. One
//! item went the other way when §AR-system.2.6 became a module: the inline
//! citation style rule, which is the checker's (§AR-checker.2.14). What stays
//! here is the classifier both stages read.
//!
//! Five items came down out of the writers when §AR-system.2.8 became a module.
//! `anchors.rs` is the whole of the former `fmt_link_anchors.rs` plus the
//! section-heading text the scanner read upward to fill a stored section title:
//! deriving an anchor from heading text is a renderer's slugger reproduced
//! byte-for-byte (§DF-github-anchor-fidelity), which is lexical and no part of a
//! writer's plan. The ID renderer and its qualified form joined `ids.rs`, which
//! also took the record of one marked citation found on a Markdown line
//! (§AR-system.4).

mod anchors;
mod comment_block;
mod comment_line;
mod compiled;
mod fence;
mod id_format;
mod id_rules;
mod ids;
mod inline_note_layout;
mod near_miss;
mod never_rewrite;
mod shorthand;
mod shorthand_targets;
mod source_line;

pub use compiled::Grammar;

// What the other components read, still through the crate root while they are
// flat (§AR-system.4). The finalize task narrows this as each caller moves into
// a module of its own.
pub(crate) use anchors::{anchor_slug, section_anchor_text};
pub(crate) use comment_block::{
    CommentBlockKind, DocCommentRule, block_declares_id, block_is_doc_comment, comment_blocks,
    doc_comment_rule, first_content_line,
};
pub(crate) use comment_line::{comment_strip_prefixes, strip_comment_tokens};
pub(crate) use compiled::{
    AGENTS_BLOCK_BEGIN, AGENTS_BLOCK_END, AGENTS_BLOCK_H2, AGENTS_SECTION_BOUNDARY,
    QUALIFIED_CITATION_PREFIX, STUB_LINK_HEADING, reduce_heading_text, section_path,
};
pub(crate) use fence::markdown_fence_delimiter;
pub(crate) use id_format::{
    id_shape, id_token_end_at, literal_after_kind_placeholder, parse_longest_id_prefix,
};
pub(crate) use id_rules::{id_grammar_key_slash_error, id_grammar_literal_slash_error};
pub(crate) use ids::{
    MarkdownLineCitation, parse_id, parse_id_arg, parse_loose_qualified_id_prefix, render_id,
    render_qualified_id,
};
pub(crate) use inline_note_layout::{
    BlockCitations, CITATION_RUN_SEPARATOR, LayoutChannel, block_has_inline_note_memoized,
    inline_layout_violations, inline_note_layout_sentence, inline_note_verdicts, layout_channel,
    layout_pass_enabled, line_says_something,
};
pub(crate) use near_miss::{declaration_captures, declaration_id_on_line, near_miss_heading};
pub(crate) use never_rewrite::{
    DocstringContent, DocstringCursor, bare_token_in_never_rewrite_zone, is_escaped,
    is_inside_inline_code, is_inside_markdown_link_destination, never_rewrite_context,
    never_rewrite_context_in, qualified_suppressed_in_source, scanned_citation_rewritable,
    string_literal_in,
};
pub(crate) use shorthand::{
    IdArgError, ShorthandIndex, ShorthandIndexes, expand_shorthand_citations_with_origins,
    parse_id_arg_with_shorthand, report_shorthand_citation, resolve_qualified_shorthand_citations,
    resolve_shorthand_citations, scan_shorthand_citations, shorthand_candidates, shorthand_names,
    shorthand_token_expansion,
};
pub(crate) use shorthand_targets::ShorthandTargets;
pub(crate) use source_line::{PythonDocstringScanState, source_scan_line};

// What only the crate's own test modules read (§AR-core-module-layout.1): the
// GitHub slugger, the layout classifier's steps, the citation tokenizer, and
// the single-line shorthand expansion.
#[cfg(test)]
pub(crate) use anchors::anchor_slug_github;
#[cfg(test)]
pub(crate) use comment_line::{comment_content_range, line_citation_ranges};
#[cfg(test)]
pub(crate) use inline_note_layout::{
    InlineNoteLayout, block_has_inline_note, content_conforms, line_layout_view,
};
#[cfg(test)]
pub(crate) use shorthand::expand_shorthand_citations;
