//! A stub whose target the walk excludes or ignores
//! (§FS-declarations.checks.excluded-stub-target): recorded by the scanner where it
//! reaches the stub's verdict (§AR-scanner.4.6), so the checker reports it without
//! asking the walk's rules again. Nothing else reads it: until grund 0.19.0 the stub
//! keeps the verdict its target's name, extension and text give it.

use std::path::PathBuf;

use super::Id;

/// One stub that pairs with a target the walk would not read: the stub's line, its
/// ID, the link's destination as written, and the rule that refuses the target.
pub(crate) struct ExcludedStubTargetNote {
    pub(crate) file: PathBuf,
    pub(crate) line: usize,
    pub(crate) id: Id,
    pub(crate) target: PathBuf,
    pub(crate) rule: ExcludingRule,
}

/// Which of the walk's rules refuses a stub's target
/// (§FS-declarations.stubs.target.3).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ExcludingRule {
    /// A directory on the path that this `[scan] exclude` entry names, as written.
    Exclude(String),
    /// An ignore file under `respect_gitignore`: its path from the config root, or
    /// its absolute path outside it, and the line of it that matched, as written.
    IgnoreFile { file: String, pattern: String },
}
