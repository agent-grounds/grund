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
