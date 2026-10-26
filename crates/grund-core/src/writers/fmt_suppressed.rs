//! A line of a suppressed scope (§FS-fmt.2.5): the answer the walk in
//! `fmt_rewrite.rs` gets when every rewrite is off, which is the line as it
//! stands unless the index carve-out (§FS-fmt.2.5.3) has to reach it.

use std::path::Path;

use super::fmt_anchor_row::anchor_detail;
use super::fmt_links::wrap_markdown_links_with_targets;
use super::fmt_rewrite::FmtLineOpts;
use super::fmt_row::LineRewrite;
use crate::config::Config;
use crate::grammar::LineLinks;

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
pub(super) fn suppressed_line(
    line: &str,
    links: LineLinks<'_>,
    path: &Path,
    config: &Config,
    is_md: bool,
    opts: &FmtLineOpts<'_>,
) -> (String, LineRewrite) {
    let unchanged = || (line.to_string(), LineRewrite::default());
    if !is_md {
        return unchanged();
    }
    let (Some(entry_ids), Some(findings)) = (opts.index_entry_ids, opts.findings) else {
        return unchanged();
    };
    let wrapped = wrap_markdown_links_with_targets(
        line,
        links,
        path,
        config,
        findings,
        opts.workspace,
        Some(entry_ids),
        opts.shorthand_targets,
        opts.settled,
    );
    if wrapped.line == line {
        return unchanged();
    }
    // §FS-fmt.3.5.1: the index link is accounted for like any other.
    let mut rewrite = LineRewrite::default();
    rewrite.counts.links_written = wrapped.written();
    rewrite.anchor_detail = anchor_detail(&wrapped.repointed);
    (wrapped.line, rewrite)
}
