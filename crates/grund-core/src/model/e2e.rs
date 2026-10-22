use std::path::PathBuf;

/// Scanner/catalog records for executable scenario declarations
/// (§AR-scanner.6, §AR-core-module-layout.1).

/// An `e2e/cases/<name>/` directory treated as an `E2E-<name>` declaration
/// (§AR-scanner.6.1) — its `command.args`, `expected.exit`, and fixture file list
/// are what `grund E2E-<name>` renders (§FS-show.2.4).
#[derive(Debug, Clone)]
pub struct E2eCase {
    pub dir: PathBuf,
    /// The words of `command.args`, empty when the file is absent (§AR-scanner.6.2).
    pub args: Vec<String>,
    /// Whether `command.args` is present at all, so an absent manifest (implicit
    /// `grund check`) stays apart from one that holds no words (§AR-scanner.6.2).
    pub has_command_args: bool,
    pub expected_exit: i32,
    pub fixtures: Vec<PathBuf>,
    pub spec_refs: Vec<E2eSpecRef>,
}

impl E2eCase {
    /// The invocation the text form prints on its first line: `grund check` when
    /// the case has no `command.args`, the bare `grund` when it holds no words,
    /// `grund <words>` otherwise (§FS-show.2.4, §AR-scanner.6.2).
    pub fn invocation(&self) -> String {
        if !self.has_command_args {
            "grund check".to_string()
        } else if self.args.is_empty() {
            "grund".to_string()
        } else {
            format!("grund {}", self.args.join(" "))
        }
    }
}

/// A non-empty `spec.refs` manifest line from an E2E case (§AR-scanner.6.3).
/// It is evidence for E2E citation-direction obligations (§FS-config.3.9), not a
/// normal citation site, so it does not produce dangling-ref findings: an E2E
/// case grounds in the *layer* a `spec.refs` entry names, and entries are
/// deliberately allowed to use idealized, not-locally-resolvable IDs, so only
/// `kind` (plus `namespace`) is retained.
#[derive(Debug, Clone)]
pub struct E2eSpecRef {
    pub namespace: Option<String>,
    pub kind: String,
}
