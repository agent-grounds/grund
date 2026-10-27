//! The grammar component (§AR-system.2.1): the lexical facts every other
//! component shares — the ID grammar compiled from `[id] format` and its
//! near-miss detection, comment-line and comment-block recognition,
//! fenced-block and raw-text HTML block boundaries, the number-only shorthand, inline-note layout, the
//! never-rewrite predicates and the formatter's own syntax — its cross-reference
//! wrapper and its suppression directives (§FS-fmt.2.3, §FS-fmt.2.5). It
//! consumes text and knows no
//! file, no rule and no frontend — and no `Config` either: what config decides
//! reaches here as the compiled `Grammar` and the `LexicalSettings` record
//! beside it, both built above and only read here (`settings.rs`,
//! §AR-system.2.1, §AR-system.4).
//!
//! The module boundary is what §AR-system.4 asks for: an item another component
//! reads is re-exported below, and everything else is the component's own
//! (§AR-core-module-layout.1.1). The submodules are the former `grammar*`,
//! `markdown_fence`, `comment_line`, `comment_block`, `shorthand`,
//! `inline_note_layout`, `never_rewrite` and ID-grammar category files, one per
//! lexical fact §AR-system.2.1 names. Four items the flat layout parked in
//! scanner files came down here when §AR-system.2.5 became a module: the
//! off-grammar `LegacyGrammar`, the `{kind}` literal reader of the `[id] format`
//! template, the member-local fallback ID parser, and the qualified-citation
//! suppression rule. Three went the other way. The inline citation style rule is
//! the checker's (§AR-checker.2.14), and so is the finding a shorthand site
//! earns (§FS-check.3.13) — what stays here is the classifier both stages read.
//! And the shorthand asked of a whole run's catalog is the resolver's
//! (§AR-resolver.4): recognizing the shape is lexical, resolving it against
//! every loaded project's declarations is not.
//!
//! Five items came down out of the writers when §AR-system.2.8 became a module.
//! `anchors.rs` is the whole of the former `fmt_link_anchors.rs` plus the
//! section-heading text the scanner read upward to fill a stored section title:
//! deriving an anchor from heading text is a renderer's slugger reproduced
//! byte-for-byte (§DF-github-anchor-fidelity), which is lexical and no part of a
//! writer's plan. The ID renderer and its qualified form joined `ids.rs`, which
//! also took the record of one marked citation found on a Markdown line
//! (§AR-system.4).
//!
//! Two more came down when §AR-system.2.11 became a component and the
//! formatter's own syntax stopped being the formatter's alone.
//! `fmt_cross_refs.rs` is the `--cross-refs` wrapper read backwards
//! (§DF-show-cross-ref-flattening) with the label predicate that decides
//! whether a `[…](…)` label is a citation at all, which the scanner had parked
//! in `legacy.rs` for it and which is private now that both are one file, and
//! the one reading of a citation's brackets every caller asks
//! (§FS-citations.wrap); `fmt_suppress.rs` is the two scopes §FS-fmt.2.5 takes
//! out of a rewrite's reach. Recognizing a wrapper and recognizing a directive
//! are both lexical, and while they sat above, the editor's on-type rule and
//! three readers of a flattened body reached sideways or upward for them
//! (§FS-lsp.1.4, §AR-system.4).
//!
//! `managed_block.rs` came down with the rest of §AR-system.2.8: finding the
//! block grund owns inside somebody else's document and reading the `vN` version
//! it carries is lexical, and it was implemented twice up there — once for the
//! agent entrypoint (§FS-init.2.3) and once for a client dotfile
//! (§FS-integrations.4.1) — while the checker's agent-entrypoint rule read the
//! first copy upward (§FS-check.3.5). Both copies are here now, so all three
//! callers read one module downward; what differs between them is in that file's
//! own doc comment.
//!
//! `section_paths.rs` is the newest, and came here for the same reason: whether
//! one section coordinate lies beneath another is a fact about the section
//! grammar and its component boundary (§FS-config.3.3.4), and both readers of
//! it sit above — `grund refs --descendants` in the api (§FS-refs.2) and the
//! declaration-side title in the editor queries (§FS-lsp.1.3.1). Written twice
//! it would be written wrong once, so it is written here. The predicates over one
//! path's components joined it from `compiled.rs` when a name under a number
//! became a section (§FS-config.3.3.1): they ask the same grammar, of a path
//! rather than of a line.

mod anchors;
mod block_sections;
mod comment_block;
mod comment_line;
mod compiled;
mod coordinate_glob;
mod coordinate_glob_match;
mod coordinate_glob_read;
mod coordinate_glob_tokens;
mod fence;
mod field_values;
mod fmt_cross_refs;
mod fmt_suppress;
mod glob_candidate;
mod heading_text;
mod id_format;
mod id_rules;
mod ids;
mod inline_note_layout;
mod language_table;
mod line_scan;
mod link_walk;
mod managed_block;
mod markdown_blocks;
mod markdown_links;
mod markdown_paragraphs;
mod multi_line_forms;
mod multi_line_string;
mod near_miss;
mod never_rewrite;
mod qualified_context;
mod relative_heading;
mod section_paths;
mod settings;
mod shorthand;
mod source_line;

pub use compiled::Grammar;

// What the other components read, each by this module's path (§AR-system.4):
// the whole of what crosses this boundary, and the only thing outside the
// directory that can name any of it.
pub(crate) use anchors::{anchor_slug, section_anchor_text, section_display_name};
pub(crate) use block_sections::{WORKSPACE_MEMBERS, WORKSPACE_SECTIONS, block_sections};
pub(crate) use comment_block::{
    CommentBlockKind, DocCommentRule, block_declares_id, block_is_doc_comment, comment_blocks,
    declaration_id_on_source_line, first_content_line, is_bare_block_line,
};
pub(crate) use comment_line::{comment_envelope, comment_strip_prefixes, strip_block_closer};
pub(crate) use compiled::{
    QUALIFIED_CITATION_PREFIX, STUB_LINK_HEADING, is_valid_project_alias, section_path,
};
pub(crate) use field_values::{direct_section_value, field_value};
pub(crate) use language_table::{DocProfile, classified_extensions, extension_rule};
// The bare fence reader serves the record's doc-comment headings and the template
// tests, which look for a fenced example rather than for a heading (§FS-check.1.1.5).
pub(crate) use coordinate_glob::{CoordinateGlob, GlobQualifier, GlobRefusal, GlobStage};
pub(crate) use coordinate_glob_match::GlobMatcher;
pub(crate) use fence::{MarkdownFence, markdown_fence_delimiter};
pub(crate) use fmt_cross_refs::{CitationLink, citation_link, flatten_cross_ref_links};
pub(crate) use fmt_suppress::{FMT_DIRECTIVE, FmtDirectives, FmtExcluded};
pub(crate) use glob_candidate::{CandidateReading, GlobOperator, read_loose_candidate};
pub(crate) use heading_text::{atx_heading_text, reduce_heading_text, without_closing_sequence};
pub(crate) use id_format::{
    id_shape, id_token_end_at, literal_after_kind_placeholder, parse_longest_id_prefix,
};
pub(crate) use id_rules::{
    id_grammar_key_slash_error, id_grammar_literal_slash_error, id_grammar_v2_slash_error,
};
pub(crate) use ids::{
    MarkdownLineCitation, parse_id, parse_id_arg, parse_qualified_id_prefix, render_id,
    split_alias_path,
};
pub(crate) use markdown_links::{LineLinks, MarkdownLinks};
pub(crate) use markdown_paragraphs::blockquote_markers;
mod spelling;
pub(crate) use inline_note_layout::{
    BlockCitations, CITATION_RUN_SEPARATOR, LayoutChannel, block_has_inline_note_memoized,
    inline_layout_violations, inline_note_layout_sentence, inline_note_verdicts, layout_channel,
    layout_pass_enabled, line_says_something,
};
pub(crate) use managed_block::{
    AGENT_GUIDANCE_BLOCK_VERSION, AGENTS_BLOCK_BASE_VERSION, AGENTS_BLOCK_END,
    AGENTS_BLOCK_VERSION, AgentsBlock, AgentsBlockLookup, AgentsBlockMember,
    agent_guidance_markers, agents_block_heading, find_agent_guidance_block, find_agents_block,
    find_managed_block, integrations_block_markers,
};
pub(crate) use markdown_blocks::{MarkdownBlocks, MarkdownLine, front_matter_extent};
pub(crate) use multi_line_string::MultiLineStrings;
pub(crate) use spelling::{
    BindingSpellings, render_catalog_id, render_citation_id, render_declaration_id, render_input_id,
};
// The version a managed integrations block is written and read at: the command
// that installs one stamps it, and that command is the CLI's
// (§FS-integrations.4.2, §AR-bindings.3).
pub use managed_block::INTEGRATIONS_BLOCK_VERSION;
// §AR-scanner.2.8.5: outside the grammar a declaration line is read off the file
// record, so only the grammar's own tests still parse one through this.
#[cfg(test)]
pub(crate) use near_miss::declaration_id_on_line;
pub(crate) use near_miss::{declaration_captures, near_miss_heading};
#[cfg(test)]
pub(crate) use never_rewrite::is_inside_markdown_link_destination;
pub(crate) use never_rewrite::{
    DocstringContent, DocstringCursor, bare_token_in_never_rewrite_zone, in_escape_position,
    is_escaped, is_inside_inline_code, markdown_never_rewrite_context, never_rewrite_context_in,
    string_literal_in,
};
pub(crate) use qualified_context::QualifiedSource;
pub(crate) use relative_heading::{
    expand_relative_heading, may_hold_relative_heading, relative_heading_form,
    relative_heading_hint, relative_to_ancestor,
};
pub(crate) use section_paths::{
    mixed_heading_path, named_mode_section_path, named_section_component, path_at_or_under,
    unnamed_component, value_binding_section_shape_is_valid,
};
pub(crate) use settings::{AliasGrammar, GrammarKind, LexicalSettings, require_kinds};
pub(crate) use shorthand::{
    IdArgError, ParsedId, ShorthandIndex, parse_id_arg_with_shorthand, resolve_shorthand_citations,
    resolve_shorthands, shorthand_candidates, shorthand_names,
};
pub(crate) use source_line::{
    PythonDocstringScanState, SourceScanLine, is_lone_block_marker, longest_form_at, prefix_forms,
    source_scan_line,
};

// What another component's tests read (§AR-core-module-layout.1.1): the GitHub
// slugger, which the scanner's file-pass cases assert their anchors against.
// The layout classifier's steps went beside their own cases, below.
#[cfg(test)]
pub(crate) use anchors::anchor_slug_github;

// The cases that pin this component, one module per behaviour area
// (§AR-core-module-layout.1.3).
#[cfg(test)]
mod tests_block_comment_opener;
#[cfg(test)]
mod tests_block_comment_state;
#[cfg(test)]
mod tests_comment_block;
#[cfg(test)]
mod tests_comment_block_position;
#[cfg(test)]
mod tests_coordinate_glob;
#[cfg(test)]
mod tests_declaration_line;
#[cfg(test)]
mod tests_doc_profiles;
#[cfg(test)]
mod tests_fmt_cross_refs;
#[cfg(test)]
mod tests_fmt_suppression;
#[cfg(test)]
mod tests_inline_code_span;
#[cfg(test)]
mod tests_inline_note_layout;
#[cfg(test)]
mod tests_line_prefix_forms;
#[cfg(test)]
mod tests_managed_block_label;
#[cfg(test)]
mod tests_markdown_blocks;
#[cfg(test)]
mod tests_markdown_paragraphs;
#[cfg(test)]
mod tests_multi_line_string;
#[cfg(test)]
mod tests_never_rewrite;
#[cfg(test)]
mod tests_section_paths;
#[cfg(test)]
mod tests_word_character_marker;
