//! The form table of §FS-languages.multi-line-string.forms: which languages carry
//! a string across a line break, what opens each form and what it waits for to
//! close. `multi_line_string.rs` walks a file with it; this file only recognizes,
//! one opener at a time, so the table reads as the spec's table does.

use super::never_rewrite::is_escaped;
use super::source_line::is_python_string_prefix;

/// One row family of the form table (§FS-languages.multi-line-string.forms).
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Language {
    Rust,
    /// `java` `kt` `kts` `scala` `swift`, and `dart`, which also writes `'''`.
    TripleQuoted {
        single_too: bool,
    },
    CSharp,
    Cpp,
    Go,
    /// `js` `jsx` `mjs` `cjs` `ts` `tsx`.
    Template,
    Python,
    Shell,
    Php,
}

impl Language {
    pub(super) fn of(extension: &str) -> Option<Self> {
        Some(match extension {
            "rs" => Self::Rust,
            "java" | "kt" | "kts" | "scala" | "swift" => Self::TripleQuoted { single_too: false },
            "dart" => Self::TripleQuoted { single_too: true },
            "cs" => Self::CSharp,
            "cpp" | "cc" | "cxx" | "h" | "hpp" | "hh" | "hxx" => Self::Cpp,
            "go" => Self::Go,
            "js" | "jsx" | "mjs" | "cjs" | "ts" | "tsx" => Self::Template,
            "py" => Self::Python,
            "sh" | "bash" | "zsh" => Self::Shell,
            "php" => Self::Php,
            _ => return None,
        })
    }

    /// Whether this language writes `/* … */` blocks, whose lines open nothing
    /// (§FS-languages.multi-line-string.bound).
    pub(super) fn has_block_comments(self) -> bool {
        !matches!(self, Self::Python | Self::Shell)
    }

    /// Whether a line comment opens at byte `i` of `line`, where the per-line
    /// toggle has no quote open (§FS-languages.multi-line-string.bound). The
    /// language's own line comment, not the configured prefix list, which names
    /// `#` for every file and would read Rust's `r#"` as a comment. A mid-line
    /// `/*` ends the search as well: what follows it is comment, not code.
    pub(super) fn comment_at(self, line: &[u8], i: usize) -> bool {
        let rest = &line[i..];
        match self {
            Self::Python => rest.starts_with(b"#"),
            // `$#` and `${#x}` are not comments: a shell `#` opens one at a word start.
            Self::Shell => rest.starts_with(b"#") && (i == 0 || line[i - 1].is_ascii_whitespace()),
            Self::Php => {
                rest.starts_with(b"#") || rest.starts_with(b"//") || rest.starts_with(b"/*")
            }
            _ => rest.starts_with(b"//") || rest.starts_with(b"/*"),
        }
    }

    /// Whether the opener search's per-line toggle reads `byte` as a quote: a
    /// Rust `'` it never does (§FS-languages.multi-line-string.bound).
    pub(super) fn is_quote(self, byte: u8) -> bool {
        matches!(byte, b'"' | b'`') || (byte == b'\'' && self != Self::Rust)
    }

    /// A quote-like byte at `i` that this language reads otherwise than the
    /// per-line toggle does, where no quote is open
    /// (§FS-languages.multi-line-string.bound): a Rust character literal or
    /// lifetime, a JavaScript or TypeScript regular-expression literal.
    pub(super) fn skip_at(self, line: &[u8], i: usize) -> Option<Skip> {
        let end = match (self, line[i]) {
            (Self::Rust, b'\'') => rust_char_end(line, i).unwrap_or(i + 1),
            (Self::Template, b'/') if regex_may_start(&line[..i]) => match regex_end(line, i) {
                Some(end) => end,
                None => return Some(Skip::Line),
            },
            _ => return None,
        };
        // A misread skip must not pass what the search would have read: a quote
        // toggles, a comment opener ends the search.
        let passes = (i + 1..end)
            .any(|j| (self.is_quote(line[j]) && !is_escaped(line, j)) || self.comment_at(line, j));
        Some(if passes { Skip::Line } else { Skip::To(end) })
    }

    /// Whether this language's forms are heredocs, whose body starts on the next
    /// line and of which only the first on a line is carried
    /// (§FS-languages.multi-line-string.forms).
    pub(super) fn heredoc(self) -> bool {
        matches!(self, Self::Shell | Self::Php)
    }

    /// The form whose opener begins at byte `i` of `line`, as the end of the opener
    /// and the closer it waits for (§FS-languages.multi-line-string.forms).
    pub(super) fn opener_at(self, line: &[u8], i: usize) -> Option<(usize, Closer)> {
        let rest = &line[i..];
        // §FS-languages.multi-line-string.forms: a prefix is a whole word.
        let word_start = i == 0 || !is_identifier_byte(line[i - 1]);
        match self {
            Self::Rust => rust_opener(line, i, word_start),
            Self::TripleQuoted { single_too } => {
                let quote = if rest.starts_with(b"\"\"\"") {
                    b"\"\"\""
                } else if single_too && rest.starts_with(b"'''") {
                    b"'''"
                } else {
                    return None;
                };
                Some((i + 3, text_closer(quote, false)))
            }
            Self::CSharp => csharp_opener(line, i),
            Self::Cpp => word_start.then(|| cpp_opener(line, i)).flatten(),
            Self::Go => (line[i] == b'`').then(|| (i + 1, text_closer(b"`", false))),
            Self::Template => (line[i] == b'`').then_some((i + 1, Closer::Template)),
            Self::Python => python_opener(line, i, word_start),
            Self::Shell => shell_opener(line, i),
            Self::Php => php_opener(line, i),
        }
    }
}

/// What the opener search does with a byte [`Language::skip_at`] claims.
pub(super) enum Skip {
    /// Resume at this column, with nothing between opened or toggled.
    To(usize),
    /// Open no form more on this line: a regular expression with no closing `/`,
    /// or a skip that would pass a quote or a comment opener.
    Line,
}

/// How one recognized form ends (§FS-languages.multi-line-string.forms).
#[derive(Clone, PartialEq, Eq)]
pub(super) enum Closer {
    /// The next occurrence of `needle`; where `escapable`, not one preceded by an
    /// odd run of `\`.
    Text { needle: Vec<u8>, escapable: bool },
    /// C#'s raw string: the next run of exactly this many `"`.
    QuoteRun(usize),
    /// C#'s verbatim string: a `"` not immediately followed by another.
    Verbatim,
    /// A JavaScript or TypeScript template literal, with its interpolations.
    Template,
    /// A shell heredoc: a line that is exactly the word, after `<<-` with its
    /// leading tabs dropped first.
    Heredoc { word: String, strip_tabs: bool },
    /// A PHP heredoc or nowdoc: a line whose first non-whitespace text is the
    /// identifier, not followed by an identifier character.
    PhpHeredoc { word: String },
}

fn text_closer(needle: &[u8], escapable: bool) -> Closer {
    Closer::Text {
        needle: needle.to_vec(),
        escapable,
    }
}

pub(super) fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// The length of the run of `"` starting at `i`.
pub(super) fn quote_run(line: &[u8], i: usize) -> usize {
    line[i..].iter().take_while(|byte| **byte == b'"').count()
}

/// An identifier starting at `i` — the heredoc word — and its end.
fn identifier_at(line: &[u8], i: usize) -> Option<usize> {
    let first = *line.get(i)?;
    if !first.is_ascii_alphabetic() && first != b'_' {
        return None;
    }
    Some(
        i + 1
            + line[i + 1..]
                .iter()
                .take_while(|b| is_identifier_byte(**b))
                .count(),
    )
}

/// Rust: a raw string `r`/`br`/`cr` + *n* `#` + `"`, or an ordinary `"`. A `'`
/// opens nothing (§FS-languages.multi-line-string.forms).
fn rust_opener(line: &[u8], i: usize, word_start: bool) -> Option<(usize, Closer)> {
    if line[i] == b'"' {
        return Some((i + 1, text_closer(b"\"", true)));
    }
    if !word_start {
        return None;
    }
    let mut j = i;
    if matches!(line[j], b'b' | b'c') {
        j += 1;
    }
    if line.get(j) != Some(&b'r') {
        return None;
    }
    j += 1;
    let hashes = line[j..].iter().take_while(|byte| **byte == b'#').count();
    j += hashes;
    if line.get(j) != Some(&b'"') {
        return None;
    }
    let mut needle = vec![b'"'];
    needle.extend(std::iter::repeat_n(b'#', hashes));
    Some((j + 1, text_closer(&needle, false)))
}

/// The end of the Rust character or byte literal whose `'` is at `i` — `'x'`,
/// `'\n'`, `'\''`, `'"'`, `'\u{1F600}'` — or `None` for a lifetime, a label
/// or a stray apostrophe (§FS-languages.multi-line-string.bound).
fn rust_char_end(line: &[u8], i: usize) -> Option<usize> {
    let rest = line.get(i + 1..)?;
    let close = if *rest.first()? == b'\\' {
        // `\` and the byte it escapes, then the rest of a `\x7f` or `\u{…}`.
        2 + rest
            .get(2..)?
            .iter()
            .take(10)
            .position(|byte| *byte == b'\'')?
    } else {
        let width = match rest[0] {
            b'\'' => return None,
            0x00..=0x7f => 1,
            0xc0..=0xdf => 2,
            0xe0..=0xef => 3,
            _ => 4,
        };
        (rest.get(width) == Some(&b'\'')).then_some(width)?
    };
    Some(i + 1 + close + 1)
}

/// The words after which a JavaScript `/` begins a regular expression rather than
/// a division (§FS-languages.multi-line-string.bound).
const REGEX_KEYWORDS: [&[u8]; 14] = [
    b"return",
    b"typeof",
    b"instanceof",
    b"in",
    b"of",
    b"new",
    b"delete",
    b"void",
    b"throw",
    b"case",
    b"do",
    b"else",
    b"yield",
    b"await",
];

/// Whether a `/` after `before` is in expression position: the start of the line,
/// after an operator or opening punctuation, or after one of [`REGEX_KEYWORDS`].
fn regex_may_start(before: &[u8]) -> bool {
    let before = before.trim_ascii_end();
    let Some(&last) = before.last() else {
        return true;
    };
    if b"(,=:[!&|?{};+-*%<>~^".contains(&last) {
        return true;
    }
    let word = before.len()
        - before
            .iter()
            .rev()
            .take_while(|byte| is_identifier_byte(**byte))
            .count();
    REGEX_KEYWORDS.contains(&&before[word..])
}

/// The end of the regular-expression literal whose `/` is at `i`: its closing `/`
/// on the same line, neither escaped nor inside a `[…]` class.
fn regex_end(line: &[u8], i: usize) -> Option<usize> {
    let mut class = false;
    let mut j = i + 1;
    while j < line.len() {
        match line[j] {
            b'\\' => j += 1,
            b'[' => class = true,
            b']' => class = false,
            b'/' if !class => return Some(j + 1),
            _ => {}
        }
        j += 1;
    }
    None
}

/// C#: a verbatim `@"`, `$@"` or `@$"`, or a raw run of three or more `"` after
/// any number of `$`.
fn csharp_opener(line: &[u8], i: usize) -> Option<(usize, Closer)> {
    let rest = &line[i..];
    for verbatim in [&b"@\""[..], b"$@\"", b"@$\""] {
        if rest.starts_with(verbatim) {
            return Some((i + verbatim.len(), Closer::Verbatim));
        }
    }
    let quotes = i + rest.iter().take_while(|byte| **byte == b'$').count();
    let run = quote_run(line, quotes.min(line.len()));
    (run >= 3).then_some((quotes + run, Closer::QuoteRun(run)))
}

/// C++: `R"delim(` with an optional `u8`, `u`, `U` or `L` before it; the
/// delimiter is at most 16 characters, none of them space, `(`, `)` or `\`.
fn cpp_opener(line: &[u8], i: usize) -> Option<(usize, Closer)> {
    let rest = &line[i..];
    let prefix = if rest.starts_with(b"u8") {
        2
    } else if matches!(rest[0], b'u' | b'U' | b'L') {
        1
    } else {
        0
    };
    if !rest[prefix..].starts_with(b"R\"") {
        return None;
    }
    let delimiter_start = i + prefix + 2;
    let delimiter_len = line[delimiter_start..]
        .iter()
        .take(17)
        .take_while(|byte| !matches!(byte, b' ' | b'(' | b')' | b'\\'))
        .count();
    let open = delimiter_start + delimiter_len;
    if delimiter_len > 16 || line.get(open) != Some(&b'(') {
        return None;
    }
    let mut needle = vec![b')'];
    needle.extend_from_slice(&line[delimiter_start..open]);
    needle.push(b'"');
    Some((open + 1, text_closer(&needle, false)))
}

/// Python: `"""` or `'''`, optionally prefixed, that is not the first
/// non-whitespace text on its line — a line-initial one is the docstring rule's.
fn python_opener(line: &[u8], i: usize, word_start: bool) -> Option<(usize, Closer)> {
    if !line[..i].iter().any(|byte| !byte.is_ascii_whitespace()) {
        return None;
    }
    let prefix = if word_start {
        line[i..]
            .iter()
            .take(2)
            .take_while(|byte| byte.is_ascii_alphabetic())
            .count()
    } else {
        0
    };
    // The prefixes the assigned-data opener reads, by the one list it reads them from.
    if !is_python_string_prefix(&String::from_utf8_lossy(&line[i..i + prefix])) {
        return None;
    }
    let quote_start = i + prefix;
    let quote = &line[quote_start..];
    let quote = if quote.starts_with(b"\"\"\"") {
        b"\"\"\""
    } else if quote.starts_with(b"'''") {
        b"'''"
    } else {
        return None;
    };
    Some((quote_start + 3, text_closer(quote, true)))
}

/// Shell: `<<` or `<<-`, optional blanks, then a word bare or quoted; a `<<` next
/// to another `<` is a here-string or PHP's opener, and one inside `(( … ))` a
/// shift, not a heredoc.
fn shell_opener(line: &[u8], i: usize) -> Option<(usize, Closer)> {
    if !line[i..].starts_with(b"<<")
        || (i > 0 && line[i - 1] == b'<')
        || line.get(i + 2) == Some(&b'<')
        || in_arithmetic(&line[..i])
    {
        return None;
    }
    let strip_tabs = line.get(i + 2) == Some(&b'-');
    let (word, end) = heredoc_word(line, i + 2 + usize::from(strip_tabs))?;
    Some((end, Closer::Heredoc { word, strip_tabs }))
}

/// Whether `before` leaves a shell arithmetic `(( … ))` open, where `<<` is a shift
/// (§FS-languages.multi-line-string.bound).
fn in_arithmetic(before: &[u8]) -> bool {
    let count = |pair: &[u8]| before.windows(2).filter(|window| *window == pair).count();
    count(b"((") > count(b"))")
}

/// PHP: `<<<`, optional blanks, then an identifier bare or quoted.
fn php_opener(line: &[u8], i: usize) -> Option<(usize, Closer)> {
    if !line[i..].starts_with(b"<<<") {
        return None;
    }
    let (word, end) = heredoc_word(line, i + 3)?;
    Some((end, Closer::PhpHeredoc { word }))
}

/// The word a heredoc opener names from `from` on, after optional spaces or tabs:
/// an identifier, bare or wrapped in `'…'` or `"…"`.
fn heredoc_word(line: &[u8], from: usize) -> Option<(String, usize)> {
    let start = from
        + line[from..]
            .iter()
            .take_while(|byte| matches!(byte, b' ' | b'\t'))
            .count();
    let quote = line
        .get(start)
        .copied()
        .filter(|b| matches!(b, b'\'' | b'"'));
    let word_start = start + usize::from(quote.is_some());
    let word_end = identifier_at(line, word_start)?;
    let end = match quote {
        Some(quote) if line.get(word_end) == Some(&quote) => word_end + 1,
        Some(_) => return None,
        None => word_end,
    };
    let word = String::from_utf8_lossy(&line[word_start..word_end]).into_owned();
    Some((word, end))
}
