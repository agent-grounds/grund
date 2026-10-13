//! The relative section heading as text (§FS-declarations.section.relative.form):
//! one `.` and one component, then the title syntax of the complete-path heading
//! it stands for. What its parent is, and so whether that title syntax fits, is
//! the scanner's question; this file only reads and rewrites the line.

use once_cell::sync::Lazy;
use regex::Regex;

/// `.` then one component, a handle or a number, then an optional `.` or `:`,
/// then the space and the title every section heading carries.
static RELATIVE_HEADING_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\A\.(?P<component>[a-z][a-z0-9-]*|[0-9]+)(?P<punct>[.:])?[ \t]+\S").unwrap()
});

/// A heading's text read as the relative form: its component and the
/// punctuation after it (§FS-declarations.section.relative.form).
pub(crate) struct RelativeForm<'a> {
    pub(crate) component: &'a str,
    pub(crate) punct: Option<char>,
}

impl RelativeForm<'_> {
    /// Whether the punctuation is the title syntax the complete-path heading of
    /// `canonical` would use: the colon on a name-bearing path, an optional
    /// period on a purely numeric one (§FS-config.3.3.1).
    pub(crate) fn fits(&self, numeric: bool) -> bool {
        match self.punct {
            Some(':') => !numeric,
            Some(_) | None => numeric,
        }
    }
}

/// `text` — a heading's rendered text, the `#` run and its space set aside — read
/// as a relative heading, or `None` when it is not one.
pub(crate) fn relative_heading_form(text: &str) -> Option<RelativeForm<'_>> {
    let caps = RELATIVE_HEADING_RE.captures(text)?;
    Some(RelativeForm {
        component: caps.name("component")?.as_str(),
        punct: caps.name("punct").and_then(|m| m.as_str().chars().next()),
    })
}

/// Whether any line of `text` is an ATX heading whose text reads as the relative
/// form, wherever it sits: what makes a tree worth a scan for a writer that has
/// no record of it. Whether the heading declares a section is the scan's answer.
pub(crate) fn may_hold_relative_heading(text: &str) -> bool {
    text.lines().any(|line| {
        let line = line.trim_start_matches(' ');
        let after = line.trim_start_matches('#');
        after.len() < line.len()
            && after.starts_with([' ', '\t'])
            && relative_heading_form(after.trim_start_matches([' ', '\t'])).is_some()
    })
}

/// The heading line with its relative path replaced by `canonical`, its depth
/// and everything after the path kept: `### .1: Username` at `format.1` is
/// `### format.1: Username` (§FS-declarations.section.relative.shown). A line
/// that is not the relative form comes back unchanged.
pub(crate) fn expand_relative_heading(line: &str, canonical: &str) -> String {
    let after_hashes = line.trim_start().trim_start_matches('#');
    let text = after_hashes.trim_start_matches([' ', '\t']);
    let Some(form) = relative_heading_form(text) else {
        return line.to_string();
    };
    let dot = line.len() - text.len();
    let tail = &line[dot + 1 + form.component.len()..];
    format!("{}{canonical}{tail}", &line[..dot])
}

/// The heading line with its path, `written` as the line spells it, replaced by
/// `.` and `below`, the components it has beneath an enclosing section; its `#`
/// run and everything after the path kept: `#### 1.1.1. Latency` and `#### .1.
/// Latency`, both at `1.1.1`, are `#### .1. Latency` beneath `1.1`
/// (§FS-section-lock.holds.fingerprint). The inverse of
/// [`expand_relative_heading`]. A line that does not spell `written` comes back
/// unchanged.
pub(crate) fn relative_to_ancestor(line: &str, written: &str, below: &str) -> String {
    let after_hashes = line.trim_start().trim_start_matches('#');
    let text = after_hashes.trim_start_matches([' ', '\t']);
    match text.strip_prefix(written) {
        Some(tail) if !written.is_empty() => {
            format!("{}.{below}{tail}", &line[..line.len() - text.len()])
        }
        _ => line.to_string(),
    }
}

/// The relative heading an unmarked dotted heading would be written as under the
/// parent of `suggested` (§FS-declarations.checks.unmarked-heading.2.1): the `#`
/// run, `.` and the last component, the title syntax the suggested heading uses,
/// and the authored title less its leading dotted token, or `Untitled`.
pub(crate) fn relative_heading_hint(level: usize, suggested: &str, title: &str) -> String {
    let component = suggested.rsplit('.').next().unwrap_or(suggested);
    let named = suggested
        .split('.')
        .any(|part| part.bytes().any(|byte| byte.is_ascii_lowercase()));
    let rest = title
        .trim_start()
        .split_once([' ', '\t'])
        .map_or("", |(_, rest)| rest.trim_start());
    format!(
        "{} .{component}{} {}",
        "#".repeat(level),
        if named { ":" } else { "." },
        if rest.is_empty() { "Untitled" } else { rest }
    )
}

#[cfg(test)]
mod tests {
    use super::{
        expand_relative_heading, may_hold_relative_heading, relative_heading_form,
        relative_heading_hint, relative_to_ancestor,
    };

    #[test]
    fn a_path_is_written_relative_to_an_ancestor_keeping_the_rest() {
        assert_eq!(
            relative_to_ancestor("#### 1.1.1. Latency", "1.1.1", "1"),
            "#### .1. Latency"
        );
        assert_eq!(
            relative_to_ancestor("#### .1. Latency", ".1", "1"),
            "#### .1. Latency"
        );
        assert_eq!(
            relative_to_ancestor("##### .1. P95", ".1", "1.1"),
            "##### .1.1. P95"
        );
        assert_eq!(
            relative_to_ancestor(
                "#### 1.goals.latency: Latency",
                "1.goals.latency",
                "latency"
            ),
            "#### .latency: Latency"
        );
        assert_eq!(
            relative_to_ancestor("#### Other", "1.1.1", "1"),
            "#### Other"
        );
    }

    #[test]
    fn only_a_relative_heading_asks_for_a_scan() {
        assert!(may_hold_relative_heading("# FS-a: A\n\n### .1: Username\n"));
        assert!(!may_hold_relative_heading(
            "# FS-a: A\n\n## 1. One\n.1 is prose\n"
        ));
        assert!(!may_hold_relative_heading("#.1 not a heading\n"));
        assert!(!may_hold_relative_heading("## ...and then\n"));
    }

    #[test]
    fn expansion_keeps_the_depth_and_the_title_syntax() {
        assert_eq!(
            expand_relative_heading("### .1: Username", "format.1"),
            "### format.1: Username"
        );
        assert_eq!(
            expand_relative_heading("### .1. Scope", "1.1"),
            "### 1.1. Scope"
        );
        assert_eq!(
            expand_relative_heading("### .2 Terms", "1.2"),
            "### 1.2 Terms"
        );
        assert_eq!(expand_relative_heading("## Notes", "notes"), "## Notes");
    }

    #[test]
    fn the_form_is_one_component_then_a_title() {
        assert!(relative_heading_form(".format: Formatting").is_some());
        assert!(relative_heading_form(".1 Scope").is_some());
        assert!(relative_heading_form(".format.user: Two components").is_none());
        assert!(relative_heading_form(".1").is_none());
        assert!(relative_heading_form("...and then").is_none());
    }

    #[test]
    fn the_hint_writes_the_last_component() {
        assert_eq!(
            relative_heading_hint(3, "format.1", "format.1 Username"),
            "### .1: Username"
        );
        assert_eq!(relative_heading_hint(3, "1.2", "1.2"), "### .2. Untitled");
    }
}
