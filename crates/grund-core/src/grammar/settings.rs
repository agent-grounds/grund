//! What config hands this component (§AR-system.2.1): the citable `[[kinds]]`
//! rows the ID patterns are compiled from, the scalar `grund.toml` keys a
//! lexical reader consults beside those patterns, and the per-alias grammar a
//! qualified citation's ID tail is read with (§FS-config.3.2, §FS-workspace.1).
//!
//! Records rather than lookups, and the reason this component names no `Config`
//! (§AR-system.4). Every value here is config's decision — it owns the keys, it
//! validates them, and it builds one of these out of what it holds — and the
//! grammar only reads it. The `Grammar` travels *inside*
//! [`LexicalSettings`] rather than beside it, so a reader cannot pair one
//! project's compiled patterns with another project's keys.

use super::compiled::Grammar;

/// One citable `[[kinds]]` row as the ID grammar reads it (§FS-config.3.4): the
/// name that is the literal prefix of every ID in the kind, and the row's
/// `format` override where it carries one (§FS-config.3.2).
///
/// Only the *citable* rows reach here. A non-citable kind declares no IDs, so
/// its name never tokenizes and never enters a pattern — config applies that
/// filter when it builds the list, which is why nothing below reads a `citable`
/// flag (§AR-scanner.2.1).
pub(crate) struct GrammarKind {
    pub(crate) name: String,
    pub(crate) format: Option<String>,
}

/// The compiled grammar plus the scalar keys a lexical reader consults beside it
/// (§FS-config.3.1, §FS-config.3.4.8): the citation marker, `[reference]
/// strict`, the comment prefixes a comment line opens with, whether Python
/// docstrings are scanned, and the three `[reference]` keys that say what an
/// inline note may look like (§FS-inline-citation-style.2.2,
/// §FS-inline-citation-style.3.3).
///
/// A **borrowed view**, built on demand from the live `Config` rather than
/// copied out of it, so a reader here sees exactly the value config holds at the
/// moment it asks — the property that makes handing the grammar a record instead
/// of a `Config` a pure re-spelling of who may read whom (§AR-core-module-layout.2).
#[derive(Clone, Copy)]
pub(crate) struct LexicalSettings<'a> {
    pub(crate) grammar: &'a Grammar,
    pub(crate) marker: &'a str,
    pub(crate) strict: bool,
    pub(crate) comment_prefixes: &'a [String],
    pub(crate) docstring_python: bool,
    pub(crate) inline_style: &'a str,
    pub(crate) inline_note_layout: &'a str,
    pub(crate) inline_note_layout_check: &'a str,
}

/// One project of the run whose grammar parses a qualified citation's ID tail
/// (§FS-workspace.1, §AR-workspace.2): the alias the citation writes, and the
/// compiled grammar that tail is read with — a workspace may mix `[id] format`s,
/// and the *target's* shape is the right one to apply across a namespace
/// boundary.
///
/// The lexical half of the run's citation targets and nothing more. Whoever
/// iterates the loaded projects — the scanner — picks the grammar out of each
/// one and hands the list down (§AR-system.2.10).
#[derive(Clone, Copy)]
pub(crate) struct AliasGrammar<'a> {
    pub(crate) alias: &'a str,
    pub(crate) grammar: &'a Grammar,
}
