//! The rule that refuses a stub's target (§FS-declarations.stubs.target.3), saved
//! in its broken verdict before pairing (§AR-scanner.4.6), so the checker renders
//! the reason without asking the walk's rules again (§AR-checker.2.5).

/// Which of the walk's rules refuses a stub's target
/// (§FS-declarations.stubs.target.3).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ExcludingRule {
    /// A directory on the path that this `[scan] exclude` entry names, as written.
    Exclude(String),
    /// A v2 `[schema.sources] exclude` pattern matching a directory on the path or
    /// the target itself, as written (§FS-config-v2.schema.sources.exclude.stubs).
    Pattern(String),
    /// An ignore file under `respect_gitignore`: its path from the config root, or
    /// its absolute path outside it, and the line of it that matched, as written.
    IgnoreFile { file: String, pattern: String },
}
