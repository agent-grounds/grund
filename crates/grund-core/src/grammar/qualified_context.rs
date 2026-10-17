//! Qualified source citations use comment context (§FS-citations.recognized.contexts,
//! §AR-scanner.2.3.2), independently of formatter and bare-token quote toggles.

use super::multi_line_forms::{Language, Skip};
use super::never_rewrite::{is_escaped, qualified_suppressed_in_source};
use super::source_line::prefix_forms;

/// The resolved source language and comment hosts, shared by every qualified
/// detection pass (§FS-citations.recognized.contexts). A caller already walking
/// a comment block can supply its content range instead of rediscovering it.
#[derive(Clone, Copy)]
pub(crate) struct QualifiedSource<'a> {
    pub(crate) forms: Option<&'a str>,
    pub(crate) comment_prefixes: &'a [String],
    pub(crate) comment_range: Option<(usize, usize)>,
}

impl QualifiedSource<'_> {
    /// §FS-citations.recognized.contexts: carried literals take precedence over
    /// comment-looking bytes; outside comments the existing exclusion is retained.
    pub(crate) fn suppressed(&self, line: &str, entry: usize, is_md: bool, pos: usize) -> bool {
        if is_md {
            return false;
        }
        if pos < entry {
            return true;
        }
        let comment = self
            .comment_range
            .filter(|(start, end)| entry <= *start && *start <= pos && pos < *end)
            .map(|(start, end)| {
                let rest = line[start..end].trim_start();
                let prefix = self
                    .comment_prefixes
                    .iter()
                    .filter_map(|prefix| prefix_forms(prefix).len_at(rest))
                    .max()
                    .unwrap_or(0);
                end - rest.len() + prefix
            })
            .filter(|start| *start <= pos)
            .or_else(|| self.comment_start(line, entry, pos));
        if let Some(start) = comment {
            return comment_quotes_open(&line[start..], pos - start);
        }
        qualified_suppressed_in_source(line, entry, false, pos)
    }

    /// Find only a supported comment before the site, using the existing bounded
    /// language reading for Rust characters/lifetimes (§FS-citations.recognized.contexts).
    /// This does not change the multi-line form search or its quote policy.
    fn comment_start(&self, line: &str, entry: usize, pos: usize) -> Option<usize> {
        let bytes = line.as_bytes();
        let language = self.forms.and_then(Language::of);
        let mut quote = None;
        let mut i = entry;
        while i < pos && i < bytes.len() {
            if let Some(open) = quote {
                if bytes[i] == open && !is_escaped(bytes, i) {
                    quote = None;
                }
            } else {
                let comment_len = line.get(i..).and_then(|rest| {
                    self.comment_prefixes
                        .iter()
                        .filter_map(|prefix| prefix_forms(prefix).len_at(rest))
                        .max()
                });
                if let Some(len) = comment_len
                    && language.is_none_or(|language| language.comment_at(bytes, i))
                {
                    // A closed block leaves the rest of the line in source context.
                    if bytes[i..].starts_with(b"/*")
                        && let Some(close) = line[i + len..].find("*/")
                        && i + len + close + 2 <= pos
                    {
                        i += len + close + 2;
                        continue;
                    }
                    return Some(i + len);
                }
                if language == Some(Language::Rust)
                    && let Some(Skip::To(end)) = Language::Rust.skip_at(bytes, i)
                {
                    i = end;
                    continue;
                }
                // A Haskell identifier's primes are not character-literal openers.
                let prime = matches!(self.forms, Some("hs" | "lhs"))
                    && bytes[i] == b'\''
                    && follows_word(line, i);
                if matches!(bytes[i], b'\'' | b'"' | b'`') && !prime && !is_escaped(bytes, i) {
                    quote = Some(bytes[i]);
                }
            }
            i += 1;
        }
        None
    }
}

/// Quoted comment data and inline code remain excluded, but an apostrophe after
/// a prose word opens no literal (§FS-citations.recognized.contexts). A single
/// quote that opened quoted data still closes it, including after a word.
fn comment_quotes_open(line: &str, pos: usize) -> bool {
    let mut quote = None;
    for (i, ch) in line.char_indices().take_while(|(i, _)| *i < pos) {
        if is_escaped(line.as_bytes(), i) {
            continue;
        }
        if let Some(open) = quote {
            if ch == open {
                quote = None;
            }
        } else if matches!(ch, '\'' | '"' | '`') && (ch != '\'' || !follows_word(line, i)) {
            quote = Some(ch);
        }
    }
    // The inline-code reader also handles multi-backtick spans and their closers.
    quote.is_some() || super::never_rewrite::is_inside_inline_code(line, pos)
}

/// Word-adjacent apostrophes/primes are prose or identifier punctuation at the
/// supported comment sites (§FS-citations.recognized.contexts).
fn follows_word(line: &str, pos: usize) -> bool {
    line.get(..pos)
        .and_then(|before| before.chars().next_back())
        .is_some_and(|ch| ch.is_alphanumeric() || ch == '_' || ch == '\'')
}
