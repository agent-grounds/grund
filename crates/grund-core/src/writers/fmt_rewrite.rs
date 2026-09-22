//! Citation normalization (§FS-fmt.2): one pass over a document that rewrites
//! triggers to markers, marks bare citations, expands shorthands, and keeps
//! cross-reference links current. The modes share the traversal — each line is
//! asked every question once — so what lives here is the walk and the per-line
//! decisions inside it. The link construction §FS-fmt.6 needs is
//! `fmt_links.rs`, the two suppressed scopes of §FS-fmt.2.5 are
//! `grammar/fmt_suppress.rs` — recognizing them is lexical, and the editor's
//! on-type rule reads the same two records (§FS-lsp.1.4.3). The command surface
//! lives in `grund-cli`; this engine file only returns edits
//! (§AR-system.2.9.1).
//!
//! Named for the rewrite rather than for the category, because the category is
//! the `writers/` directory now (§AR-core-module-layout.1). Tree and scope
//! orchestration, including §FS-fmt.6.6's auto-enable pair, lives in the
//! cohesive `fmt_tree.rs` sibling (§AR-core-module-layout.1.5).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::fmt_links::wrap_markdown_links_with_targets;
use super::fmt_local_sections::{expand_local_section_citations, local_section_labels};
use crate::config::Config;
use crate::grammar::{
    DocstringContent, DocstringCursor, FmtDirectives, declaration_id_on_line, id_token_end_at,
    is_inside_inline_code, is_inside_markdown_link_destination, markdown_fence_delimiter,
    string_literal_in,
};
use crate::model::{Findings, Id};
use crate::resolver::{
    ShorthandTargets, WorkspaceContext, expand_shorthand_citations_with_origins,
};

/// One file's rewritten lines plus what the walk needs to decide afterwards:
/// whether anything changed, and whether a shorthand expansion was wanted but
/// could not be performed for lack of the declaration set (§FS-fmt.2.4.5).
pub(super) struct RewrittenFile {
    pub(super) lines: Vec<String>,
    pub(super) changed: bool,
    pub(super) saw_shorthand_candidate: bool,
}

/// Apply `fmt_line` to every rewritable line of one file, appending each changed
/// line to `changes`. Fenced blocks and declaration headings are passed through
/// untouched (§FS-fmt.2.3), and so is every line of a suppressed scope
/// (§FS-fmt.2.5) — the `grund:fmt` region state is carried here beside the fence
/// state, because both are facts about where in the file the reader has got to.
pub(super) fn rewrite_file(
    original: &str,
    path: &Path,
    config: &Config,
    is_md: bool,
    opts: &FmtLineOpts<'_>,
    changes: &mut Vec<(PathBuf, usize, String)>,
) -> RewrittenFile {
    let mut markdown_fence = None;
    let mut lines = Vec::new();
    let mut changed = false;
    let mut saw_shorthand_candidate = false;
    // §FS-fmt.2.5.2.1: every file starts with the rewrite on — a region never
    // carries across files.
    let mut directives = FmtDirectives::new(config.lexical(), is_md);
    // §FS-fmt.2.3.1.1: `fmt` judges a docstring line on its content, so it carries
    // the scanner's docstring state — advanced on *every* line, including the ones
    // passed through untouched, or one unvisited `"""` desynchronizes the file.
    let is_py = path.extension().and_then(|e| e.to_str()) == Some("py");
    let mut docstrings = DocstringCursor::new(is_py, config.docstring_python);
    for (idx, line) in original.lines().enumerate() {
        if is_md && markdown_fence_delimiter(&mut markdown_fence, line) {
            lines.push(line.to_string());
            continue;
        }
        // The state this line *starts* in, kept so each rewrite stage can re-derive
        // the span from the line it is about to change (see `DocstringCursor`).
        let entry = docstrings;
        let docstring = docstrings.advance(line);
        // §FS-fmt.2.5.2: a directive inside a fenced block is an illustration, so
        // the fence is asked first and toggles nothing; the directive line itself
        // is passed through whatever state it leaves behind.
        if markdown_fence.is_some() {
            lines.push(line.to_string());
            continue;
        }
        if directives.consume(line, docstring) {
            lines.push(line.to_string());
            continue;
        }
        if declaration_id_on_line(
            &config.grammar,
            docstring.text_of(line),
            docstring.is_docstring(),
            is_md,
        )
        .is_some()
        {
            lines.push(line.to_string());
            continue;
        }
        // §FS-fmt.2.5: the file's own `[fmt] exclude` verdict, or the region the
        // directives above have opened. Either one leaves only the index carve-out.
        let suppressed = opts.excluded || !directives.rewriting();
        let (new_line, label) = fmt_line_at(
            line,
            entry,
            path,
            idx + 1,
            config,
            is_md,
            opts,
            suppressed,
            &mut saw_shorthand_candidate,
        );
        if new_line != line {
            let local_labels = local_section_labels(path, idx + 1, config, opts.findings);
            if local_labels.is_empty() {
                changes.push((path.to_path_buf(), idx + 1, label));
            } else {
                changes.extend(
                    local_labels
                        .into_iter()
                        .map(|label| (path.to_path_buf(), idx + 1, label)),
                );
            }
            changed = true;
        }
        lines.push(new_line);
    }
    RewrittenFile {
        lines,
        changed,
        saw_shorthand_candidate,
    }
}

/// The rewrites `fmt_line` runs and their inputs — grouped so `fmt_line` has
/// one logical "what to rewrite" parameter instead of three flags plus two
/// optional findings handles.
pub(crate) struct FmtLineOpts<'a> {
    pub(crate) add_marker: bool,
    pub(crate) cross_refs: bool,
    /// §FS-fmt.2.5.1: this file is named by `[fmt] exclude`, so every line of it
    /// is suppressed. The per-region directives (§FS-fmt.2.5.2) are the other half
    /// of the same verdict and are read line by line in `rewrite_file`.
    pub(crate) excluded: bool,
    /// §FS-fmt.6.1.2: when this file is a kind's index the always-linkify carve-out
    /// may have to reach, the IDs that index owes an entry for — the only citations
    /// the pass wraps where the ordinary one is off. `None` for every other file,
    /// and read on a line only where that line's ordinary pass is off: under
    /// `[fmt.cross_refs] enabled = false`, or inside a suppressed scope
    /// (§FS-fmt.2.5.3). Elsewhere `cross_refs` already means "wrap what this file
    /// has" and the carve-out has nothing to add.
    pub(crate) index_entry_ids: Option<&'a BTreeSet<Id>>,
    pub(crate) findings: Option<&'a Findings>,
    pub(crate) workspace: Option<&'a WorkspaceContext>,
    /// The declaration indexes the shorthand rewrite resolves against, built once
    /// per walk (§FS-fmt.2.4.5). Separate from `findings` because a qualified
    /// shorthand reads another project's declarations entirely.
    pub(crate) shorthand_targets: &'a ShorthandTargets<'a>,
}

/// Apply the `fmt` rewrites to one line, in order: trigger→marker (§FS-fmt.2.1),
/// then optionally bare→marker (§FS-fmt.2.2), then shorthand→canonical
/// (§FS-fmt.2.4), then optionally Markdown-link wrapping (§FS-fmt.6) —
/// returning the new line plus a label naming the most significant rewrite that
/// fired.
///
/// The shorthand pass runs after the trigger pass and reads its output, so a
/// typed `$$FS-042` is marked and then expanded within the one call — which is
/// how §FS-fmt.2.4.3's "in one step" holds without the trigger pass needing to
/// know about declarations.
///
/// Why each stage takes ownership of the previous stage's line:
/// `expand_shorthand_citations` returns `None` for "unchanged" exactly so the
/// common line can be moved through untouched.
///
/// Why a shorthand expansion names the text it wrote in the label: the other
/// three rewrites move markup around an unchanged ID token, so `grund check` can
/// still see a mistake in them; this one writes the slug *into* the token, and a
/// wrong one is a well-formed citation of the wrong declaration.
#[allow(clippy::too_many_arguments)]
fn fmt_line_at(
    line: &str,
    docstrings: DocstringCursor,
    path: &Path,
    lineno: usize,
    config: &Config,
    is_md: bool,
    opts: &FmtLineOpts<'_>,
    suppressed: bool,
    saw_shorthand_candidate: &mut bool,
) -> (String, String) {
    if suppressed {
        return suppressed_line(line, path, config, is_md, opts);
    }
    let triggered = replace_trigger(line, docstrings.peek(line), config, is_md);
    let trigger_changed = triggered.line != line;
    let mut trigger_marker_starts = triggered.marker_starts;
    let marked = if opts.add_marker {
        add_markers(
            &triggered.line,
            docstrings.peek(&triggered.line),
            config,
            is_md,
            &mut trigger_marker_starts,
        )
    } else {
        triggered.line
    };
    let marker_changed = opts.add_marker && marked != line && !trigger_changed;
    let local_expansion = expand_local_section_citations(
        &marked,
        path,
        lineno,
        config,
        opts.findings,
        &trigger_marker_starts,
        saw_shorthand_candidate,
    );
    let local_changed = local_expansion.is_some();
    let marked = local_expansion.unwrap_or(marked);
    // Each stage below takes ownership of the previous stage's line rather than
    // cloning it: `fmt` touches every line of every scanned file, so one avoidable
    // allocation per line is a measurable share of the command (§GOAL-fast-feedback).
    let mut expansions = Vec::new();
    let expansion = expand_shorthand_citations_with_origins(
        &marked,
        docstrings.peek(&marked),
        config,
        is_md,
        opts.shorthand_targets,
        &trigger_marker_starts,
        saw_shorthand_candidate,
        &mut expansions,
    );
    let shorthand_changed = expansion.is_some();
    let mut final_line = expansion.unwrap_or(marked);
    let mut link_changed = false;
    // §FS-fmt.6.1.2: the carve-out narrows the pass to the index's own entries only
    // where the ordinary pass is off; where it runs, it already wraps the page.
    let entry_ids = if opts.cross_refs {
        None
    } else {
        opts.index_entry_ids
    };
    if (opts.cross_refs || entry_ids.is_some())
        && is_md
        && let Some(findings) = opts.findings
    {
        let wrapped = wrap_markdown_links_with_targets(
            &final_line,
            path,
            config,
            findings,
            opts.workspace,
            entry_ids,
            opts.shorthand_targets,
        );
        link_changed = wrapped != final_line;
        final_line = wrapped;
    }
    let label = if trigger_changed {
        "trigger \u{2192} marker"
    } else if marker_changed {
        "bare \u{2192} marker"
    } else if local_changed {
        "local section \u{2192} canonical"
    } else if shorthand_changed {
        "shorthand \u{2192} canonical"
    } else if link_changed {
        "markdown link"
    } else {
        ""
    };
    // §FS-fmt.3.6: whichever label won, a line that expanded a shorthand also names
    // the text it will write — the one rewrite no later pass can question
    // (§DF-shorthand-numeric-run.2.7).
    let label = if expansions.is_empty() {
        label.to_string()
    } else {
        format!(
            "{label}: {}",
            expansions
                .iter()
                .map(|(written, canonical)| format!("{written} \u{2192} {canonical}"))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    (final_line, label)
}

/// Test/editor-facing single-line entry point. A standalone line has no scan
/// coordinate, so declaration-local rewrites are intentionally unavailable;
/// the tree formatter calls `fmt_line_at` with the real line number.
#[cfg(test)]
pub(crate) fn fmt_line(
    line: &str,
    docstrings: DocstringCursor,
    path: &Path,
    config: &Config,
    is_md: bool,
    opts: &FmtLineOpts<'_>,
    suppressed: bool,
    saw_shorthand_candidate: &mut bool,
) -> (String, String) {
    fmt_line_at(
        line,
        docstrings,
        path,
        0,
        config,
        is_md,
        opts,
        suppressed,
        saw_shorthand_candidate,
    )
}

/// One line of a suppressed scope (§FS-fmt.2.5): an excluded file, or a
/// `grund:fmt off` region. Every rewrite is off here — the trigger, the marker
/// upgrade, the shorthand expansion, and the ordinary wrap alike — so the line
/// keeps its bytes and the report says nothing about it.
///
/// The one survivor is the always-linkify carve-out (§FS-fmt.2.5.3): a kind's
/// index entries are wrapped in a suppressed scope exactly as they are under
/// `[fmt.cross_refs] enabled = false`, because a suppression that could leave an
/// index in a state §FS-check.3.17 reports and `fmt` refuses to repair would
/// reopen the hole §DF-index-always-linkified closed. `index_entry_ids` is
/// `None` for every file that is not a configured index, which is the ordinary
/// case and returns the line untouched.
fn suppressed_line(
    line: &str,
    path: &Path,
    config: &Config,
    is_md: bool,
    opts: &FmtLineOpts<'_>,
) -> (String, String) {
    let unchanged = || (line.to_string(), String::new());
    if !is_md {
        return unchanged();
    }
    let (Some(entry_ids), Some(findings)) = (opts.index_entry_ids, opts.findings) else {
        return unchanged();
    };
    let wrapped = wrap_markdown_links_with_targets(
        line,
        path,
        config,
        findings,
        opts.workspace,
        Some(entry_ids),
        opts.shorthand_targets,
    );
    if wrapped == line {
        return unchanged();
    }
    (wrapped, "markdown link".to_string())
}

/// Rewrite each `$$<ID>` trigger to `§<ID>` — but only where `$$` is immediately
/// followed by a real ID-shaped token, and never inside a string literal in source
/// code or Markdown link destinations (§FS-fmt.2.1, §FS-fmt.2.3.1,
/// §DF-reference-marker).
struct TriggerReplacement {
    line: String,
    /// Byte offsets in `line` where §FS-fmt.2.1 produced a marker. Keeping
    /// these offsets is what lets §FS-fmt.2.4.3 distinguish authoring sugar from
    /// persisted marker shorthand candidate by candidate.
    marker_starts: Vec<usize>,
}

fn replace_trigger(
    line: &str,
    docstring: DocstringContent<'_>,
    config: &Config,
    is_md: bool,
) -> TriggerReplacement {
    let mut output = String::new();
    let mut marker_starts = Vec::new();
    let mut cursor = 0;
    while let Some(relative) = line[cursor..].find(&config.trigger) {
        let start = cursor + relative;
        let after = start + config.trigger.len();
        if id_token_end_at(line, after, &config.grammar).is_some()
            && (is_md || !string_literal_in(docstring, line, start))
            && (!is_md || !is_inside_inline_code(line, start))
            && (!is_md || !is_inside_markdown_link_destination(line, start))
        {
            output.push_str(&line[cursor..start]);
            marker_starts.push(output.len());
            output.push_str(&config.marker);
            cursor = after;
            continue;
        }
        output.push_str(&line[cursor..after]);
        cursor = after;
    }
    output.push_str(&line[cursor..]);
    TriggerReplacement {
        line: output,
        marker_starts,
    }
}

/// Prefix `§` onto bare ID-shaped tokens that lack it — the `--marker` upgrade
/// (§FS-fmt.2.2) — skipping tokens already marked, Markdown inline-code examples,
/// Markdown link destinations, and source-code string literals (§FS-fmt.2.3).
fn add_markers(
    line: &str,
    docstring: DocstringContent<'_>,
    config: &Config,
    is_md: bool,
    trigger_marker_starts: &mut [usize],
) -> String {
    let mut output = String::new();
    let mut cursor = 0;
    let mut inserted_at = Vec::new();
    for caps in config.grammar.citation_re.captures_iter(line) {
        let Some(found) = caps.get(0) else { continue };
        // §FS-workspace.1.3: a `path/ID` token without a marker is text, not a
        // citation — `fmt --marker` must not auto-promote it to `§path/ID`.
        if caps.name("namespace").is_some() {
            continue;
        }
        if line[..found.start()].ends_with(&config.marker) {
            continue;
        }
        // §FS-fmt.2.3 / §FS-check.1.1.2: an unmarked named coordinate is one
        // prose token under the opt-in and is never promoted by `fmt --marker`.
        if config.grammar.has_reserved_named_tail(line, found.end())
            || config
                .grammar
                .is_named_section(caps.name("sec").map(|sec| sec.as_str()))
        {
            continue;
        }
        if is_md && is_inside_inline_code(line, found.start()) {
            continue;
        }
        if is_md && is_inside_markdown_link_destination(line, found.start()) {
            continue;
        }
        if !is_md && string_literal_in(docstring, line, found.start()) {
            continue;
        }
        output.push_str(&line[cursor..found.start()]);
        inserted_at.push(found.start());
        output.push_str(&config.marker);
        output.push_str(found.as_str());
        cursor = found.end();
    }
    output.push_str(&line[cursor..]);
    // §FS-fmt.2.4.3: the shorthand pass reads the rewritten line, so carry each
    // trigger-origin marker past any earlier bare-citation marker insertions.
    for start in trigger_marker_starts {
        *start += inserted_at
            .iter()
            .filter(|inserted| **inserted < *start)
            .count()
            * config.marker.len();
    }
    output
}
