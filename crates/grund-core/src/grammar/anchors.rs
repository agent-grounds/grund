//! The anchor a heading gets (§FS-fmt.6.7, §DF-md-link-anchor-strategy): the
//! heading text a section anchor is built from, and the slug each configured
//! `anchor_format` profile derives from that text. Both are pure functions of
//! text — no file, no config record, no findings — which is what makes them
//! lexical facts and puts them in this component rather than in the formatter
//! that emits the link (§AR-system.2.1).
//!
//! They were the `fmt` category's while the crate was flat, and the scanner read
//! the first one upward to fill a section's stored title (§AR-scanner.2.2). Both
//! came down when §AR-system.2.8 became a module, so the scanner reads it
//! downward and the resolver's link target (`resolver/link_targets.rs`)
//! reads the second one the same way. Reproducing a renderer's slugger
//! byte-for-byte is the whole of §DF-github-anchor-fidelity, and nothing about
//! it is a writer's plan.

use unicode_normalization::UnicodeNormalization;

use super::compiled::reduce_heading_text;

/// Slugify a heading into a fragment anchor, dispatching on the configured
/// `[fmt.cross_refs] anchor_format` profile (github / gitlab / mkdocs / pandoc) —
/// §FS-fmt.6.7, §DF-md-link-anchor-strategy.
pub(crate) fn anchor_slug(text: &str, profile: &str) -> String {
    match profile {
        "pandoc" => anchor_slug_pandoc(text),
        "mkdocs" => anchor_slug_mkdocs(text),
        "gitlab" => anchor_slug_gitlab(text),
        _ => anchor_slug_github(text),
    }
}

/// Reproduce GitHub's `github-slugger` byte-for-byte: lowercase the text, delete
/// every character that is not a letter, digit, `_`, or `-` (each deletion in
/// place, so the neighbours close up), then turn each remaining space into one
/// `-`. It does **not** collapse runs of `-` and does **not** trim trailing ones —
/// `## A — B` → `#a--b`, `` ## 6. Watch mode (`--watch`) `` → `#6-watch-mode---watch`.
/// Matching that exactly is the whole point of the `github` profile: the emitted
/// `#fragment` navigates only if it is the slug GitHub itself renders
/// (§DF-github-anchor-fidelity, correcting the "collapse consecutive `-`" wording
/// in §DF-md-link-anchor-strategy.2.3).
pub(crate) fn anchor_slug_github(text: &str) -> String {
    let mut out = String::new();
    for ch in text.chars().flat_map(char::to_lowercase) {
        if ch.is_alphanumeric() || ch == '_' || ch == '-' {
            out.push(ch);
        } else if ch == ' ' {
            out.push('-');
        }
        // anything else (`.`, brackets, backticks, em dash, tabs, …) is dropped
    }
    out
}

fn anchor_slug_gitlab(text: &str) -> String {
    // "Similar to GitHub with minor Unicode-handling differences"
    // (§DF-md-link-anchor-strategy.2.3); identical for the ASCII headings grund's own
    // specs use, so it rides the github slugger (§DF-github-anchor-fidelity).
    anchor_slug_github(text)
}

// Python-Markdown's TOC slugger: lowercase, drop everything that isn't a word
// char, whitespace, or `-`, then collapse each run of whitespace-and-`-` to one
// `-` (`re.sub(r'[-\s]+', sep, value)`). The keep-set includes `-`, unlike a naive
// "alnum + `_`" filter — `# FS-1-x: Y` slugs to `#fs-1-x-y`, not `#fs1x-y`.
fn anchor_slug_mkdocs(text: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in text.nfkd() {
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() || lower == '_' {
            out.push(lower);
            last_dash = false;
        } else if (lower.is_ascii_whitespace() || lower == '-') && !last_dash && !out.is_empty() {
            out.push('-');
            last_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

fn anchor_slug_pandoc(text: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in text.nfkd() {
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() || lower == '_' || lower == '-' || lower == '.' {
            out.push(lower);
            last_dash = lower == '-';
        } else if lower.is_ascii_whitespace() && !last_dash && !out.is_empty() {
            out.push('-');
            last_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

/// The heading text a section anchor is built from — `<number> <title>` taken
/// straight off the heading line, since anchors are derived from heading text, not
/// stored (§DF-md-link-anchor-strategy). The title is reduced to its rendered form
/// (`reduce_heading_text`: `[§FS-<x>.1](path)` → `§FS-<x>.1`, `<ID>` dropped) so
/// the anchor is stable whether or not a citation in this heading has been wrapped
/// by `grund fmt --cross-refs` (§DF-github-anchor-fidelity).
pub(crate) fn section_anchor_text(line: &str, section: &str) -> String {
    let trimmed = line.trim_start();
    // §FS-fmt.6.2: named anchors derive from the complete rendered heading, so
    // its explicit colon reaches the renderer (`goals: Scope`). Numeric paths
    // retain their historical normalized stored text byte for byte.
    if section
        .as_bytes()
        .first()
        .is_some_and(u8::is_ascii_lowercase)
    {
        return reduce_heading_text(trimmed.trim_start_matches('#').trim_start());
    }
    let heading = trimmed
        .trim_start_matches('#')
        .trim_start()
        .trim_start_matches(section)
        .trim_start_matches('.')
        .trim_start();
    format!(
        "{} {}",
        section.replace('.', ""),
        reduce_heading_text(heading)
    )
    .trim()
    .to_string()
}
