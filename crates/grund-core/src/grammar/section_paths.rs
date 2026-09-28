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
