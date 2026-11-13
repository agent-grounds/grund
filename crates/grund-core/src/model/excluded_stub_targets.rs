//! The rule that refuses a stub's target (§FS-declarations.stubs.target.3), saved
//! in its broken verdict before pairing (§AR-scanner.4.6), so the checker renders
//! the reason without asking the walk's rules again (§AR-checker.2.5). The same rule keeps
//! the walk from a folder home's `<ID>.md`, which `fetch` refuses and `check` names
//! (§FS-fetch.4.2.1, §FS-check.3.1.3.1), in the same words: an ignore file spelled by the
//! report's one spelling of a path, which each reader hands in (§AR-config.1.7, §FS-cli.3.4).

use std::path::Path;

/// Which of the walk's rules refuses a stub's target
/// (§FS-declarations.stubs.target.3).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ExcludingRule {
    /// A directory on the path that this `[scan] exclude` entry names, as written.
    Exclude(String),
    /// A v2 `[schema.sources] exclude` pattern matching a directory on the path or
    /// the target itself, as written (§FS-config-v2.schema.sources.exclude.stubs).
    Pattern(String),
    /// An ignore file under `respect_gitignore`, as a path the report spells
    /// (§FS-cli.3.4), and the line of it that matched, as written.
    IgnoreFile {
        file: std::path::PathBuf,
        pattern: String,
    },
}

impl ExcludingRule {
    /// Why the walk does not read the path, as the broken-stub line and the unread-home line
    /// both say it (§FS-declarations.stubs.target.3, §FS-fetch.4.2.1), an ignore file named by
    /// `spell`.
    pub(crate) fn reason(&self, spell: &impl Fn(&Path) -> String) -> String {
        match self {
            ExcludingRule::Exclude(entry) => format!("[scan] exclude names {entry}"),
            ExcludingRule::Pattern(pattern) => {
                format!("[schema.sources] exclude matches it by {pattern}")
            }
            ExcludingRule::IgnoreFile { file, pattern } => {
                format!("{} ignores it by {pattern}", spell(file))
            }
        }
    }

    /// The repair at the path itself, whose name `fetch` does not choose (§FS-fetch.4.2.1), an
    /// ignore file named by `spell`.
    pub(crate) fn lift(&self, spell: &impl Fn(&Path) -> String) -> String {
        match self {
            ExcludingRule::Exclude(entry) => format!("remove {entry} from [scan] exclude"),
            ExcludingRule::Pattern(pattern) => {
                format!("remove {pattern} from [schema.sources] exclude")
            }
            ExcludingRule::IgnoreFile { file, .. } => {
                format!("stop ignoring it in {}", spell(file))
            }
        }
    }
}
