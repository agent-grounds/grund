//! The section grammar asked of one path, with no regex over a line: whether a
//! component is a handle, whether a path is a named-mode path or puts a name under
//! a number, and whether one coordinate lies beneath another (§FS-config.3.3.1,
//! §FS-config.3.3.4). The patterns the line regexes compile stay in `compiled.rs`;
//! the component pattern is shared from there, so the two cannot drift.

use once_cell::sync::Lazy;
use regex::Regex;

use super::compiled::NAMED_COMPONENT_PATTERN;

static NAMED_COMPONENT_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(&format!(r"\A{NAMED_COMPONENT_PATTERN}\z")).unwrap());

/// Whether `part` is one `[a-z][a-z0-9-]*` section handle (§FS-config.3.3.1): the
/// component grammar the section regexes compile, asked of one component. The
/// handles `value_chapter` and the values fields name are held to it too
/// (§FS-config.3.2.7, §FS-config.3.4.13), and so is a rule's chapter path.
pub(crate) fn named_section_component(part: &str) -> bool {
    NAMED_COMPONENT_RE.is_match(part)
}

/// The first component of a chapter path no subject can name: an all-digit one,
/// or an empty one, which is no number but no name either
/// (§FS-rule-refusals.sentences.chapter-path). A section holding one is no rule
/// unit (§FS-rules.2), so the facts adapter and the subject parser ask this.
pub(crate) fn unnamed_component(path: &str) -> Option<&str> {
    path.split('.')
        .find(|part| part.bytes().all(|byte| byte.is_ascii_digit()))
}

/// Whether a `.`-joined path is a named-mode section path (§FS-config.3.3.1): every
/// component a handle or a number, a name under a number included. What counts as
/// a number is the caller's question — a value binding admits only positive ones
/// (§FS-values.3.1) — so it is passed in.
pub(crate) fn named_mode_section_path(path: &str, numeric: impl Fn(&str) -> bool) -> bool {
    path.split('.')
        .all(|part| numeric(part) || named_section_component(part))
}

/// The path of a heading's rendered text, the `#` run and its space set aside, when
/// it reads as a complete-path heading whose path puts a name under a number
/// (§FS-config.3.3.1): the path, its colon, then the title every section heading
/// carries. What makes a Markdown file worth a lock pass
/// (§FS-section-lock.holds.entry).
pub(crate) fn mixed_heading_path(text: &str) -> Option<&str> {
    let (path, title) = text.split_once(':')?;
    let number = |part: &str| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit());
    (title.starts_with([' ', '\t'])
        && !title.trim().is_empty()
        && crate::model::name_under_number(path)
        && named_mode_section_path(path, number))
    .then_some(path)
}

/// Whether a binding's cited section path is a value-binding shape at all: a
/// valid root path — every component numeric, or a named prefix — optionally
/// followed by one positive numeric immediate coordinate (§FS-values.3.1). A value
/// root is never under a number, so a mixed path such as `2.values` binds nothing
/// (§FS-values.2.5). A path with no coordinate is the root itself
/// (§FS-values.3.1.2), and a bare ID has no path to check. Which of the two
/// readings a path takes, and whether its root *is* a root, is the checker's
/// question (§FS-values.5.1); this is only the grammar.
pub(crate) fn value_binding_section_shape_is_valid(section: &str) -> bool {
    named_mode_section_path(section, crate::model::positive_numeric_component)
        && !crate::model::name_under_number(section)
}

/// Whether the coordinate `candidate` is the one asked for or lies beneath it
/// — the one definition of *beneath* the scheme has (§FS-refs.2). The
/// comparison is on **path components**, never on characters: `candidate` is a
/// descendant only when it continues `requested` across `separator`, so `3.1`
/// reaches `3.1.2` at every depth and never the sibling `3.10`, and
/// `checks.duplicate` reaches `checks.duplicate.1` and never
/// `checks.duplicate-section`. A string prefix is wrong by an eightfold here,
/// silently, which is the error a subtree query exists to remove.
///
/// `separator` is the component boundary of the caller's coordinate space at
/// the depth it asks about, because those two differ (§FS-config.3.3.4): a
/// recorded section path is dotted at every depth whatever the outer
/// `[id] section_separator` is, while a bare declaration enters its section
/// tree through that outer separator.
///
/// It lives in the grammar component because it is a rule about the section
/// grammar and needs no `Config`, no file and no frontend (§AR-system.2.1),
/// and because its two readers — `grund refs --descendants` (§FS-refs.1) and
/// the LSP's declaration-side title (§FS-lsp.1.3.1) — both sit above it
/// (§AR-system.4). One definition, so the terminal and the editor cannot
/// disagree about what a subtree holds (§FS-lsp.4).
pub(crate) fn path_at_or_under(candidate: &str, requested: &str, separator: &str) -> bool {
    candidate == requested
        || candidate
            .strip_prefix(requested)
            .is_some_and(|tail| tail.starts_with(separator))
}
