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
//! lexical fact §AR-system.2.1 names.

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
pub(crate) use id_format::{id_shape, id_token_end_at, parse_longest_id_prefix};
pub(crate) use id_rules::{id_grammar_key_slash_error, id_grammar_literal_slash_error};
pub(crate) use ids::{parse_id, parse_id_arg};
pub(crate) use inline_note_layout::{
    BlockCitations, block_has_inline_note_memoized, check_inline_citation_style,
    inline_layout_violations, inline_note_layout_sentence, inline_note_verdicts,
    layout_pass_enabled, line_says_something,
};
pub(crate) use near_miss::{declaration_captures, declaration_id_on_line, near_miss_heading};
pub(crate) use never_rewrite::{
    DocstringContent, DocstringCursor, bare_token_in_never_rewrite_zone, is_inside_inline_code,
    is_inside_markdown_link_destination, is_inside_string_literal, never_rewrite_context,
    never_rewrite_context_in, scanned_citation_rewritable, string_literal_in,
};
pub(crate) use shorthand::{
    IdArgError, ParsedId, ShorthandIndex, ShorthandIndexes,
    expand_shorthand_citations_with_origins, parse_id_arg_with_shorthand,
    report_shorthand_citation, resolve_qualified_shorthand_citations, resolve_shorthand_citations,
    scan_shorthand_citations, shorthand_candidates, shorthand_names, shorthand_token_expansion,
};
pub(crate) use shorthand_targets::ShorthandTargets;
pub(crate) use source_line::{PythonDocstringScanState, source_scan_line};

// What only the crate's own test modules read (§AR-core-module-layout.1): the
// layout classifier's steps, the citation tokenizer, and the single-line
// shorthand expansion the rewrite cases drive directly.
#[cfg(test)]
pub(crate) use comment_line::{comment_content_range, line_citation_ranges};
#[cfg(test)]
pub(crate) use inline_note_layout::{
    InlineNoteLayout, block_has_inline_note, content_conforms, layout_channel, line_layout_view,
};
#[cfg(test)]
pub(crate) use shorthand::expand_shorthand_citations;
