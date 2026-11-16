//! A heading whose complete section path is already written in place, read for
//! the one fault that keeps it from declaring a section: a missing title, or a
//! `_` in a named component (§FS-declarations.checks.unmarked-heading.2.3). This
//! file only reads the text; whether the repair would declare a section of the
//! owning declaration is the scanner's question.

use super::section_paths::{named_mode_section_path, named_section_component};
use crate::model::named_section_path;

/// The fault a path-bearing heading text carries, and the path that repairs it
/// (§FS-declarations.checks.unmarked-heading.2.3).
pub(crate) struct PathFault<'a> {
    /// The authored path, without a trailing `:` or `.`.
    pub(crate) path: &'a str,
    /// The authored path with every `_` replaced by `-`, a section path.
    pub(crate) corrected: String,
    /// The authored title, or `None` when the heading has none.
    pub(crate) title: Option<&'a str>,
    /// Whether some named component of the authored path carries a `_`.
    pub(crate) underscore: bool,
}

impl PathFault<'_> {
    /// The heading that repairs the fault at `level`: the `#` run, the corrected
    /// path, the title syntax that path takes (§FS-config.3.3.1), and the authored
    /// title or `Untitled` (§FS-declarations.checks.unmarked-heading.2.3).
    pub(crate) fn shown_heading(&self, level: usize) -> String {
        format!(
            "{} {}{} {}",
            "#".repeat(level),
            self.corrected,
            if named_section_path(&self.corrected) {
                ":"
            } else {
                "."
            },
            self.title.unwrap_or("Untitled")
        )
    }
}

/// `text` — a heading's rendered text, the `#` run and its space set aside —
/// read as a complete path that lacks only a title or has a `_` in a named
/// component, or `None` when it has neither fault, is a dotted heading, or
/// would not be a section path once its `_`s are hyphens
/// (§FS-declarations.checks.unmarked-heading.2.3).
pub(crate) fn path_fault(text: &str) -> Option<PathFault<'_>> {
    if text.starts_with('.') {
        return None; // §FS-declarations.checks.unmarked-heading.2.1's.
    }
    let text = text.trim();
    let (token, rest) = text
        .split_once([' ', '\t'])
        .map_or((text, ""), |(token, rest)| (token, rest.trim()));
    let (path, title) = if rest.is_empty() {
        // No title: `P:` on a name-bearing path, `P.` on a numeric one, or a bare `P`
        // of two or more components; a lone bare component reads as a title.
        let path = if let Some(path) = token.strip_suffix(':') {
            authored_path(path).filter(|path| named_section_path(path))?
        } else if let Some(path) = token.strip_suffix('.') {
            authored_path(path).filter(|path| !named_section_path(path))?
        } else {
            authored_path(token).filter(|path| path.contains('.'))?
        };
        (path, None)
    } else {
        // A title after `: ` is a fault only where a named component carries a `_`.
        let path = authored_path(token.strip_suffix(':')?)
            .filter(|path| named_section_path(path) && path.contains('_'))?;
        (path, Some(rest))
    };
    let corrected = path.replace('_', "-");
    let numeric = |part: &str| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit());
    // §FS-config.3.2.7: every name of the corrected path is a `[a-z][a-z0-9-]*` handle.
    named_mode_section_path(&corrected, numeric).then_some(PathFault {
        path,
        corrected,
        title,
        underscore: path.contains('_'),
    })
}

/// `path` when it is components joined by `.`, each a number `[0-9]+` or a word
/// `[a-z][a-z0-9_-]*` (§FS-declarations.checks.unmarked-heading.2.3).
fn authored_path(path: &str) -> Option<&str> {
    path.split('.')
        .all(|part| {
            (!part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
                || named_section_component(&part.replace('_', "-"))
        })
        .then_some(path)
}

#[cfg(test)]
mod tests {
    use super::path_fault;

    fn shown(text: &str, level: usize) -> Option<(String, bool, String)> {
        path_fault(text).map(|fault| {
            (
                fault.path.to_string(),
                fault.underscore,
                fault.shown_heading(level),
            )
        })
    }

    #[test]
    fn a_path_with_no_title_is_shown_untitled() {
        let expect = |path: &str, heading: &str| Some((path.into(), false, heading.into()));
        assert_eq!(
            shown("requirements.1", 3),
            expect("requirements.1", "### requirements.1: Untitled")
        );
        assert_eq!(
            shown("requirements.3:", 3),
            expect("requirements.3", "### requirements.3: Untitled")
        );
        assert_eq!(shown("1.", 2), expect("1", "## 1. Untitled"));
        assert_eq!(shown("1.2", 3), expect("1.2", "### 1.2. Untitled"));
    }

    #[test]
    fn an_underscore_is_shown_hyphenated_with_its_title_kept() {
        assert_eq!(
            shown("requirements.size_format: Size format", 3),
            Some((
                "requirements.size_format".into(),
                true,
                "### requirements.size-format: Size format".into()
            ))
        );
        assert_eq!(
            shown("requirements.size_format:", 3),
            Some((
                "requirements.size_format".into(),
                true,
                "### requirements.size-format: Untitled".into()
            ))
        );
    }

    #[test]
    fn a_title_a_dotted_heading_or_a_bad_name_has_no_fault() {
        for text in [
            "1",
            "background",
            "open_questions",
            "Background",
            "requirements.2: Two",
            "2. Second",
            ".2",
            "Notes_here: Notes",
            "_draft: Draft",
            "1:",
            "requirements.",
            "open_questions Open",
        ] {
            assert!(path_fault(text).is_none(), "{text}");
        }
    }
}
