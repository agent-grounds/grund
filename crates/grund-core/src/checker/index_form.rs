//! How one citation of an indexed ID sits in its kind's index — the entry form
//! the kind-index invariant reads (§FS-check.3.18.5, §FS-check.3.17.1,
//! §DF-index-entry-form.2.1), in a file of its own beside `index`
//! (§AR-checker.2.16, §AR-core-module-layout.1).

use crate::grammar::{
    CitationLink, LineLinks, citation_link, is_inside_inline_code, markdown_never_rewrite_context,
};

/// How one citation of an indexed ID sits in the index file — the entry's form
/// (§FS-check.3.18.5, §FS-check.3.17.1, §DF-index-entry-form.2.1).
#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum IndexCitationForm {
    /// A link a reader can follow, which is what the entry has to be
    /// (§FS-citations.wrap.shape): wrapped as `[§<ID>…](<target>)`, the form
    /// `grund fmt --cross-refs` writes in a kind's index (§FS-fmt.6.1.1), or in the
    /// text of a link its author wrote whose destination is not empty
    /// (§FS-citations.wrap.enclosing).
    Link,
    /// A recognized citation of the ID, unwrapped, **and one the
    /// cross-reference pass would wrap on its next `--write`**. §FS-check.3.17.4's
    /// finding, and the only form that earns it.
    Bare,
    /// Everything else: a citation `fmt` declines to wrap, so it is neither an
    /// entry nor a §FS-check.3.17.5 finding, and the ID falls to §FS-check.3.18's
    /// error (§DF-index-entry-form.2.3).
    Ignored,
}

/// Classify every occurrence of `text` on `line` and keep the strongest form.
/// Reading the line rather than trusting a recorded column is what makes this
/// agree with `fmt` on a line that carries the same citation twice.
///
/// The two verdicts are deliberately asymmetric, because they ask different
/// questions. `Link` asks whether the entry *is* the link a reader can follow,
/// and any wrap satisfies that — including a hand-written one around an
/// unmarked token, which `fmt` would leave alone but which is a correct entry
/// as it stands — and so does a link its author wrote around the citation and
/// more text, wherever it goes. `Bare` asks whether `grund fmt --write` would
/// turn this occurrence into that link, and only an occurrence the
/// cross-reference pass actually reaches may answer yes: marker-prefixed
/// (§FS-fmt.6.5), outside §FS-fmt.2.3's never-rewrite zones, and outside an
/// enclosing link (§FS-check.3.17.4). Anything else is `Ignored` —
/// §FS-check.3.17.4 names `grund fmt --write` as its fix, and an error whose
/// named fix the tool declines to perform is an error a repository can never
/// clear (§DF-index-entry-form.2.3), the same trap `shorthand_rewritable` keeps
/// §FS-check.3.13.1 out of.
///
/// `line` is Markdown: §FS-config.3.4 requires `index` to name a `.md` file and
/// `kind_index_targets` drops anything else, which is what lets the never-rewrite
/// test below be asked in its Markdown form.
///
/// Where the marker is looked for: it may be part of the recorded citation text or
/// sit just before this occurrence of it, which is the same token either way. An
/// empty marker (permitted outside strict mode) matches both tests, exactly as it
/// makes the formatter's own test vacuous.
pub(super) fn index_citation_form(
    line: &str,
    links: LineLinks<'_>,
    text: &str,
    marker: &str,
) -> IndexCitationForm {
    let mut form = IndexCitationForm::Ignored;
    let mut cursor = 0;
    while let Some(relative) = line[cursor..].find(text) {
        let start = cursor + relative;
        let end = start + text.len();
        // Past the whole match, never `start + 1`: a citation begins with the
        // marker, which is multi-byte under the default `§`, and slicing one byte
        // into it would panic (§REQ-never-crashes).
        cursor = end;
        match index_citation_form_at(line, links, start, end, marker) {
            IndexCitationForm::Link => return IndexCitationForm::Link,
            IndexCitationForm::Bare => form = IndexCitationForm::Bare,
            IndexCitationForm::Ignored => {}
        }
    }
    form
}

/// The same entry-form judgment for one exact occurrence (§FS-check.3.18.5),
/// also used by §FS-check.3.12.2's repair so another link on the line cannot
/// turn an ordinary citation into a required marked entry. `links` places `line`
/// in the index, so a reference or a link spanning lines is read as `fmt` reads it.
pub(super) fn index_citation_form_at(
    line: &str,
    links: LineLinks<'_>,
    start: usize,
    end: usize,
    marker: &str,
) -> IndexCitationForm {
    match citation_link(links, line, start, end) {
        // §FS-check.3.17.4: the formatter's own bracket test, with a destination
        // closed by its balancing `)` and holding something before it.
        CitationLink::Wrap(wrap) if wrap.close > wrap.url_start => {
            if is_inside_inline_code(line, start - 1) {
                return IndexCitationForm::Ignored;
            }
            return IndexCitationForm::Link;
        }
        // §FS-check.3.18.5: the author's link is the entry where it goes somewhere,
        // its destination never compared. `fmt` leaves it either way, so one that
        // goes nowhere is no entry and the ID falls to §FS-check.3.18.
        CitationLink::Enclosing { entry } => {
            if entry {
                return IndexCitationForm::Link;
            }
            return IndexCitationForm::Ignored;
        }
        // §FS-citations.checks.image-index-entry: a citation-only image is no entry,
        // and `fmt` leaves it, so the ID falls to §FS-check.3.18 as a mention.
        CitationLink::Image(_) => return IndexCitationForm::Ignored,
        CitationLink::Wrap(_) | CitationLink::Unwrapped => {}
    }
    // §FS-fmt.6.5: `--cross-refs` wraps marker-prefixed citations only, and
    // without `--marker` a bare token stays bare — so a bare token is not an
    // entry `fmt` can repair.
    let marked = line[start..end].starts_with(marker) || line[..start].ends_with(marker);
    // §FS-fmt.2.3, through the one predicate the scanner and the rewrite
    // already share: an inline-code illustration and a Markdown link
    // destination are zones `fmt` never writes in.
    if !marked || markdown_never_rewrite_context(links, line, start) {
        return IndexCitationForm::Ignored;
    }
    IndexCitationForm::Bare
}

/// What an index says about one ID: whether any citation of it is a full link,
/// and where the first bare one sits (§FS-check.3.17.6 anchors there).
#[derive(Default)]
pub(super) struct IndexEntryState {
    pub(super) linked: bool,
    pub(super) first_bare: Option<(usize, usize)>,
}

impl IndexEntryState {
    pub(super) fn record(&mut self, form: IndexCitationForm, line: usize, column: usize) {
        match form {
            IndexCitationForm::Link => self.linked = true,
            IndexCitationForm::Bare => {
                let here = (line, column);
                if self.first_bare.is_none_or(|seen| here < seen) {
                    self.first_bare = Some(here);
                }
            }
            IndexCitationForm::Ignored => {}
        }
    }
}
