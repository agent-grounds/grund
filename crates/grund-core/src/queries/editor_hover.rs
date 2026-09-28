use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::editor_snapshot::{LspCitation, LspSnapshot};
use crate::grammar::path_at_or_under;

/// How much of the tree leans on one declaration-side title: citation sites and
/// the distinct files those sites live in (§FS-lsp.1.2.5).
///
/// The declaration-side title hover (§FS-lsp.1.2): which citations belong to a
/// title, how many sites and files that is, and the exact Markdown body the LSP
/// hands the editor.
///
/// Split out of the api's contract for the reason the on-type rule was: the
/// public items in
/// this file are part of the embedding contract §AR-core-module-layout.2 keeps
/// there, but what they carry is a behavior with its own invariant — one
/// definition of "is cited by this title", shared by the hover count and the
/// reference list so the two can never disagree (§FS-lsp.1.3.1) — and that
/// invariant is what a reader comes here for.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LspUsage {
    pub sites: usize,
    pub files: usize,
}

/// Whether a citation belongs to the declaration-side title `title_query_id`:
/// the title's own ID, or one of its deeper sections (§FS-lsp.1.3.1).
/// A bare declaration enters its section tree through the configured outer
/// separator; an existing section path always enters descendants through `.`
/// (§FS-config.3.3.4).
///
/// Both query IDs are already namespace-qualified by the snapshot, so a
/// member's own `§<ID>` and a sibling's `§<alias>/<ID>` compare equal here for
/// the same declaration — which is what makes the count agree with `grund refs`
/// across a workspace (§FS-lsp.1.2.5, §FS-workspace.8.2).
pub fn citation_under_title(
    title_query_id: &str,
    citation_query_id: &str,
    section_separator: &str,
) -> bool {
    let local_title = title_query_id.rsplit('/').next().unwrap_or(title_query_id);
    let descendant_separator = if local_title.contains(section_separator) {
        "."
    } else {
        section_separator
    };
    path_at_or_under(citation_query_id, title_query_id, descendant_separator)
}

impl LspSnapshot {
    /// Every citation site that belongs to the declaration-side title
    /// `query_id`, in snapshot order — the set `textDocument/references`
    /// returns for that title (§FS-lsp.1.3.1), which on a whole-ID title is by
    /// definition the set `grund refs <ID>` reports (§FS-refs.2).
    pub fn title_citations(&self, query_id: &str, section_separator: &str) -> Vec<&LspCitation> {
        self.citations
            .iter()
            .filter(|citation| {
                citation_under_title(query_id, &citation.query_id, section_separator)
            })
            .collect()
    }

    /// The counts §FS-lsp.1.2.5 shows on a declaration-side title, over exactly
    /// the sites `title_citations` lists. Read from the snapshot rather than
    /// from a fresh `refs` query, which would re-scan the tree per hover
    /// (§AR-lsp.5.2).
    pub fn title_usage(&self, query_id: &str, section_separator: &str) -> LspUsage {
        let sites = self.title_citations(query_id, section_separator);
        usage_over_paths(sites.iter().map(|citation| citation.path.as_path()))
    }
}

/// The pair both transports report over one citation set: its members are the
/// sites, the distinct paths among them the files (§FS-refs.3.4,
/// §FS-lsp.1.2.5). One fold, called by the hover's `title_usage` and by
/// `grund refs --total`, so the terminal and the editor can never disagree
/// about a number (§FS-lsp.4).
///
/// The paths are taken generically because the two callers hold different
/// types for the same thing — a `RefHit::path` is a `String` and an
/// `LspCitation::path` a `PathBuf` — and the fold is over the set, not over
/// either representation of it.
pub fn usage_over_paths<P: AsRef<Path>>(paths: impl IntoIterator<Item = P>) -> LspUsage {
    let mut sites = 0usize;
    let mut files: BTreeSet<PathBuf> = BTreeSet::new();
    for path in paths {
        sites += 1;
        files.insert(path.as_ref().to_path_buf());
    }
    LspUsage {
        sites,
        files: files.len(),
    }
}

/// The declaration-title hover body (§FS-lsp.1.2.4): the whole title as inline
/// code, then the usage clause. One line, and the same bytes for the same
/// snapshot — the wording lives here rather than in the transport so a second
/// frontend cannot re-word it (§FS-lsp.4).
pub fn lsp_title_hover_body(title: &str, usage: LspUsage) -> String {
    format!("{} — {}", markdown_code_span(title), usage_clause(usage))
}

/// Append literal target-kind metadata after the existing successful hover,
/// preserving all preview bytes and the original title helper (§FS-lsp.1.2.8).
/// Call after preview linkification so title text is never interpreted as a citation.
pub fn lsp_hover_with_kind_title(body: &str, kind_title: Option<&str>) -> String {
    match kind_title {
        Some(title) => format!("{body}\n\nKind: {}", markdown_code_span(title)),
        None => body.to_string(),
    }
}

/// `text` as a CommonMark code span, verbatim (§FS-lsp.1.2.4). A backslash does
/// not escape a backtick inside a code span, so a title carrying one — plenty
/// of section headings do, `2.1.2 Section map (--toc)` among them — is fenced
/// with a run one longer than the longest run inside it, and padded with one
/// space at each end when it starts or ends with a backtick (CommonMark strips
/// exactly one leading and one trailing space when both are present).
fn markdown_code_span(text: &str) -> String {
    let mut longest_run = 0usize;
    let mut run = 0usize;
    for ch in text.chars() {
        run = if ch == '`' { run + 1 } else { 0 };
        longest_run = longest_run.max(run);
    }
    let fence = "`".repeat(longest_run + 1);
    let pad = if text.starts_with('`') || text.ends_with('`') {
        " "
    } else {
        ""
    };
    format!("{fence}{pad}{text}{pad}{fence}")
}

/// `cited at <n> site(s) across <m> file(s)`, or `not cited` at zero
/// (§FS-lsp.1.2.4). Only the two nouns inflect: the preposition is `across` at
/// every count, so a skimmed hover changes only in its digits.
///
/// Public because `grund refs --total` prints this clause and no other
/// (§FS-refs.3.4): the wording lives here once, so a terminal and an editor
/// render one sentence rather than two that drift (§FS-lsp.1.2.5).
pub fn usage_clause(usage: LspUsage) -> String {
    if usage.sites == 0 {
        return "not cited".to_string();
    }
    format!(
        "cited at {} site{} across {} file{}",
        usage.sites,
        plural_s(usage.sites),
        usage.files,
        plural_s(usage.files)
    )
}

fn plural_s(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}
