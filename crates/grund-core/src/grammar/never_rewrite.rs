use super::multi_line_string::MultiLineStrings;
use super::source_line::{PythonDocstringScanState, SourceScanLine, source_scan_line};

/// The never-rewrite predicates shared by the scanner (§AR-scanner.2.3), `fmt`
/// (§FS-fmt.2.3), and the LSP on-type path (§FS-lsp.1.4): where a citation-shaped
/// token sits in a context none of the three may touch — a string literal, an
/// inline-code span, or a Markdown link destination.
///
/// Whether a citation at `marker_start` sits in one of the contexts `grund fmt`
/// must never rewrite (§FS-fmt.2.3): an illustration in inline code, a Markdown
/// link destination, or a runtime string literal.
///
/// One predicate, read by both the rewrite and the scanner, because the two have
/// to agree: `fmt` is forbidden to canonicalize a shorthand here, so
/// §FS-check.3.13.1 must not demand it here either — an error whose named fix the
/// tool refuses to perform is an error a repository can never clear.
///
/// This is the Markdown half, which a raw line answers whole. A source line is
/// asked through its line view, `never_rewrite_context_in`, because whether a
/// column is inside a string depends on the lines above it
/// (§FS-languages.multi-line-string).
pub(crate) fn markdown_never_rewrite_context(line: &str, marker_start: usize) -> bool {
    is_inside_inline_code(line, marker_start)
        || is_inside_markdown_link_destination(line, marker_start)
}

/// Where a Python docstring's **content** and any assigned-data span sit on one
/// raw source line (§AR-scanner.4.4, §FS-check.1.1.3.1). The docstring view is
/// empty on every other line; the assigned-data span separately protects only
/// its own raw bytes from scanner-aligned rewrites. It also carries the state
/// the line **begins** in: how many of its leading bytes are inside a string
/// literal an earlier line opened (§FS-languages.multi-line-string), so every
/// reader that asks "inside a string?" asks it of the line it is actually on.
///
/// §FS-fmt.2.3.1.1: a docstring's delimiters are doc-comment syntax, not quotes, so
/// the never-rewrite walk runs over this slice rather than the raw line and a
/// docstring line is judged exactly like a `#` comment line. This is the one view
/// the scanner, `fmt`, and the LSP on-type path all hold, which is what keeps
/// §AR-scanner.2.6.10's "one predicate serving both" true now that the predicate has
/// two texts to choose between.
#[derive(Clone, Copy, Default)]
pub(crate) struct DocstringContent<'a> {
    span: Option<(usize, &'a str)>,
    assigned_data_span: Option<(usize, usize)>,
    /// The raw-line bytes before this offset are inside a multi-line literal
    /// (§FS-languages.multi-line-string); `0` on a line that begins in code.
    literal_entry: usize,
}

impl<'a> DocstringContent<'a> {
    /// The content span of a line the scanner has already normalized
    /// (§AR-scanner.4.4). `SourceScanLine::text` is a slice of the raw line starting
    /// at `column_offset`, which is exactly the pair this view is. `literal_entry` is
    /// the line's entry in the file's `MultiLineStrings`.
    pub(crate) fn of(scan: &SourceScanLine<'a>, raw_line: &'a str, literal_entry: usize) -> Self {
        Self {
            span: scan.in_py_docstring.then(|| {
                let start = scan.column_offset;
                (start, &raw_line[start..start + scan.text.len()])
            }),
            assigned_data_span: scan.assigned_data_span,
            literal_entry,
        }
    }

    /// The literal entry state on the scanner's normalized line, which starts
    /// `column_offset` bytes into the raw one (§AR-scanner.4.4,
    /// §FS-languages.multi-line-string).
    pub(crate) fn literal_entry_from(self, column_offset: usize) -> usize {
        self.literal_entry.saturating_sub(column_offset)
    }

    /// The text a whole-line reader sees here: the docstring's content on a
    /// docstring line, the raw line everywhere else (§AR-scanner.4.4).
    pub(crate) fn text_of(self, raw_line: &'a str) -> &'a str {
        self.span.map_or(raw_line, |(_, text)| text)
    }

    /// Whether this line is docstring content at all.
    pub(crate) fn is_docstring(self) -> bool {
        self.span.is_some()
    }

    /// Whether this line contains bytes belonging to an assigned triple-quoted
    /// data span (§FS-check.1.1.3.1).
    pub(crate) fn has_assigned_data(self) -> bool {
        self.assigned_data_span.is_some()
    }

    /// The text a never-rewrite question at raw-line offset `pos` is asked of, with
    /// the literal entry state and `pos` translated into it. A position outside the
    /// content — the delimiter itself, or the code after a one-line docstring
    /// closes — is judged on the raw line, the only text that describes it.
    fn view(self, raw_line: &'a str, pos: usize) -> (&'a str, usize, usize) {
        match self.span {
            Some((offset, text)) if pos >= offset && pos < offset + text.len() => {
                (text, self.literal_entry_from(offset), pos - offset)
            }
            _ => (raw_line, self.literal_entry, pos),
        }
    }

    fn is_assigned_data(self, pos: usize) -> bool {
        self.assigned_data_span
            .is_some_and(|(start, end)| start <= pos && pos < end)
    }
}

/// The never-rewrite question for a line whose docstring content `docstring`
/// describes, at the raw-line offset `marker_start` (§FS-fmt.2.3.1.1). Every surface
/// that asks "may `fmt` rewrite here?" goes through this, so all three reach one
/// verdict per site.
pub(crate) fn never_rewrite_context_in(
    docstring: DocstringContent<'_>,
    line: &str,
    is_md: bool,
    marker_start: usize,
) -> bool {
    if docstring.is_assigned_data(marker_start) {
        return true;
    }
    let (text, entry, pos) = docstring.view(line, marker_start);
    if is_md {
        markdown_never_rewrite_context(text, pos)
    } else {
        is_inside_string_literal(text, entry, pos)
    }
}

/// `is_inside_string_literal` asked of the same view (§FS-fmt.2.3.1.1) — what the
/// two `fmt` passes with no Markdown branch of their own, `replace_trigger` and
/// `add_markers`, use.
pub(crate) fn string_literal_in(docstring: DocstringContent<'_>, line: &str, pos: usize) -> bool {
    if docstring.is_assigned_data(pos) {
        return true;
    }
    let (text, entry, pos) = docstring.view(line, pos);
    is_inside_string_literal(text, entry, pos)
}

/// The Python-docstring state a line **begins** in, so a caller that rewrites the
/// line can ask where its docstring content sits *in the line it is about to
/// change* (§FS-fmt.2.3.1.1).
///
/// `fmt` re-derives the span per rewrite stage instead of carrying it, because a
/// stage before it may have changed the line's length: `--marker` splices a `§`
/// into the docstring's content, and every offset after the splice moves. The
/// content's *start* never moves — only indentation and the delimiter precede it —
/// so replaying `source_scan_line` from the entry state is exact.
///
/// The cursor also reads the file's multi-line string literals
/// (§FS-languages.multi-line-string) off the pre-pass `literals`, one line per
/// `advance`. A splice never moves a closer: nothing before a line's closer is
/// rewritten, because all of it is inside the literal.
#[derive(Clone, Copy)]
pub(crate) struct DocstringCursor<'l> {
    /// `true` only for a `.py` file in a project that scans docstrings; otherwise
    /// the docstring half of every view is empty and no work is done for it.
    scanning: bool,
    state: PythonDocstringScanState,
    literals: &'l MultiLineStrings,
    /// The 0-based index of the line the next `advance` reads.
    line: usize,
}

impl Default for DocstringCursor<'_> {
    fn default() -> Self {
        Self::new(false, false, MultiLineStrings::NONE)
    }
}

impl<'l> DocstringCursor<'l> {
    pub(crate) fn new(is_py: bool, docstring_python: bool, literals: &'l MultiLineStrings) -> Self {
        Self {
            scanning: is_py && docstring_python,
            state: PythonDocstringScanState::default(),
            literals,
            line: 0,
        }
    }

    /// Advance over one raw line, returning its content view.
    pub(crate) fn advance<'a>(&mut self, line: &'a str) -> DocstringContent<'a> {
        let literal_entry = self.literals.entry(self.line);
        self.line += 1;
        if !self.scanning {
            return DocstringContent {
                literal_entry,
                ..DocstringContent::default()
            };
        }
        let scan = source_scan_line(line, true, true, &mut self.state);
        DocstringContent::of(&scan, line, literal_entry)
    }

    /// The content view of `line` read from this cursor's *current* state, leaving
    /// the cursor where it is — the per-stage replay `fmt` needs.
    pub(crate) fn peek<'a>(self, line: &'a str) -> DocstringContent<'a> {
        let mut probe = self;
        probe.advance(line)
    }
}

/// §FS-check.1.1: a **bare** (unmarked) citation is suppressed in the same
/// never-rewrite zones — a source-file string literal, or, in a Markdown file, a
/// link destination — because a marker-prefixed citation is the only kind
/// `grund fmt` ever promotes to canonical form there (§FS-fmt.2.3). Recognizing a
/// bare token as a citation in a zone `fmt` will never touch would leave `check`
/// demanding an edit the formatter refuses to make. Markdown inline-code spans
/// are prose formatting rather than a rewrite hazard, so a bare token there stays
/// a citation off strict mode — this predicate is narrower than
/// `never_rewrite_context_in`, which also withholds the marked case from code.
///
/// §AR-scanner.2.3.1: the third zone, the escape position of §FS-check.1.1.9, is
/// the one that depends on neither the strict mode nor the host language, so it is
/// asked of Markdown and source alike rather than in either branch. `entry` is the
/// line's literal entry state (§FS-languages.multi-line-string).
pub(crate) fn bare_token_in_never_rewrite_zone(
    line: &str,
    entry: usize,
    is_md: bool,
    pos: usize,
    marker: &str,
) -> bool {
    if in_escape_position(line, pos, marker) {
        return true;
    }
    if is_md {
        is_inside_markdown_link_destination(line, pos)
    } else {
        is_inside_string_literal(line, entry, pos)
    }
}

/// §FS-check.1.1.9: whether the token starting at `pos` sits in an **escape
/// position** — the bytes immediately before it are the configured `marker`
/// wrapped in `<` and `>`. `<§>FS-user-login` illustrates a citation's shape, so
/// no pass may read it as one: `check` would demand the edit the file already
/// carries, and `fmt --marker` would splice a marker inside the brackets and turn
/// the illustration into a live citation (§FS-fmt.2.3).
///
/// The escape is spelled with the *configured* marker, which is why this asks for
/// the marker rather than for a literal `<§>`: where the marker is `@`, `<@>ID` is
/// the escape and `<§>ID` an ordinary bare token. Asked of the token's own start
/// column, so `<§> ID` with a space between is not an escape of anything.
pub(crate) fn in_escape_position(line: &str, pos: usize, marker: &str) -> bool {
    if marker.is_empty() {
        return false;
    }
    let Some(before) = line.get(..pos) else {
        return false;
    };
    before
        .strip_suffix('>')
        .and_then(|before| before.strip_suffix(marker))
        .is_some_and(|before| before.ends_with('<'))
}

/// Whether byte offset `pos` falls inside a `'…'`, `"…"`, or `` `…` `` literal on
/// this line — the source-code exclusion that keeps an ID printed in a string
/// from being treated as a citation by the scanner or rewritten by `fmt`
/// (§FS-fmt.2.3.1).
///
/// `entry` is the state the line begins in (§FS-languages.multi-line-string): its
/// first `entry` bytes are inside a literal an earlier line opened, and the
/// per-line toggle starts neutral after them. `0` is a line that begins in code.
pub(crate) fn is_inside_string_literal(line: &str, entry: usize, pos: usize) -> bool {
    after_literal(line, entry, pos).is_none_or(|(tail, pos)| toggled_open(tail, pos))
}

/// The part of `line` after its carried literal closes, with `pos` translated into
/// it — `None` while `pos` is still inside that literal
/// (§FS-languages.multi-line-string).
fn after_literal(line: &str, entry: usize, pos: usize) -> Option<(&str, usize)> {
    if pos < entry {
        return None;
    }
    Some((line.get(entry..)?, pos - entry))
}

/// The per-line quote toggle of §FS-fmt.2.3.1, from a neutral start.
fn toggled_open(line: &str, pos: usize) -> bool {
    let bytes = line.as_bytes();
    let mut single = false;
    let mut double = false;
    let mut backtick = false;
    let mut i = 0;
    while i < pos && i < bytes.len() {
        match bytes[i] {
            b'\'' if !double && !backtick && !is_escaped(bytes, i) => single = !single,
            b'"' if !single && !backtick && !is_escaped(bytes, i) => double = !double,
            b'`' if !single && !double && !is_escaped(bytes, i) => backtick = !backtick,
            _ => {}
        }
        i += 1;
    }
    single || double || backtick
}

/// Whether byte offset `pos` falls inside a `` `…` `` inline-code span in Markdown
/// — citations there are illustrative, not real, so `fmt` leaves them alone
/// (§FS-fmt.2.3, §FS-fmt.6.4).
///
/// Where the span begins and ends is §FS-fmt.2.3.5: it opens at a run of *n*
/// backticks whose first backtick is not escaped, closes at the next run of
/// exactly *n*, reads a backslash inside it as literal, and leaves the rest of
/// the line code when it does not close. The opener's first backtick is outside
/// the span and every later byte up to the end of the closing run is inside it.
pub(crate) fn is_inside_inline_code(line: &str, pos: usize) -> bool {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < pos && i < bytes.len() {
        if bytes[i] != b'`' || is_escaped(bytes, i) {
            i += 1;
            continue;
        }
        let open = i;
        while i < bytes.len() && bytes[i] == b'`' {
            i += 1;
        }
        match closing_run_end(bytes, i, i - open) {
            Some(end) if pos < end => return true,
            Some(end) => i = end,
            None => return true,
        }
    }
    false
}

/// The end of the first run of exactly `len` backticks at or after `from`, the
/// run that closes a span opened by `len` (§FS-fmt.2.3.5). No backslash is read:
/// inside a span it escapes nothing.
fn closing_run_end(bytes: &[u8], from: usize, len: usize) -> Option<usize> {
    let mut i = from;
    while i < bytes.len() {
        if bytes[i] != b'`' {
            i += 1;
            continue;
        }
        let run = i;
        while i < bytes.len() && bytes[i] == b'`' {
            i += 1;
        }
        if i - run == len {
            return Some(i);
        }
    }
    None
}

/// Whether byte offset `pos` falls inside the destination part of an inline
/// Markdown link (`[text](destination)`). URLs are presentation syntax, not
/// citations, so `fmt --marker` must not rewrite ID-shaped file names there
/// (§FS-fmt.2.3.4). Every destination on the line is read with the one extent of
/// `link_destination_close` (§FS-citations.zones.link-destination).
pub(crate) fn is_inside_markdown_link_destination(line: &str, pos: usize) -> bool {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b']' && bytes[i + 1] == b'(' && !is_escaped(bytes, i) {
            let start = i + 2;
            match link_destination_close(bytes, start) {
                Some(close) if pos >= start && pos < close => return true,
                Some(close) => i = close,
                None => return pos >= start,
            }
        }
        i += 1;
    }
    false
}

/// §FS-citations.zones.link-destination: where the destination that begins at
/// `start`, just after an unescaped `](`, ends — the index of the `)` that
/// balances its `(`. Each unescaped `(` inside opens one more level and each
/// unescaped `)` closes one; an escaped one counts for neither. `None` when the
/// `(` never balances on the line, so the destination runs to the end of it.
///
/// The one extent of a destination: the never-rewrite zone above and the wrap
/// test of §FS-citations.wrap.recognized both read it, so no pass of `fmt` can
/// end a destination somewhere `check` does not.
pub(crate) fn link_destination_close(bytes: &[u8], start: usize) -> Option<usize> {
    let mut depth = 1usize;
    for (j, &byte) in bytes.iter().enumerate().skip(start) {
        match byte {
            b'(' if !is_escaped(bytes, j) => depth += 1,
            b')' if !is_escaped(bytes, j) => {
                depth -= 1;
                if depth == 0 {
                    return Some(j);
                }
            }
            _ => {}
        }
    }
    None
}

/// Whether the byte at `pos` is backslash-escaped — an odd run of `\` in front
/// of it (§FS-fmt.2.3).
///
/// The one lexical fact the three predicates above are all written in terms of:
/// a quote, a backtick or a bracket that is escaped opens nothing. Shared with
/// the `grund.toml` reader, which asks the same question of a `"` before
/// deciding whether a `#` starts a comment (§FS-config.3).
pub(crate) fn is_escaped(bytes: &[u8], pos: usize) -> bool {
    let mut count = 0;
    let mut j = pos;
    while j > 0 && bytes[j - 1] == b'\\' {
        count += 1;
        j -= 1;
    }
    count % 2 == 1
}

/// §AR-scanner.2.3.2: the qualified `alias/ID` form collides with a path, module
/// reference, or URL, so in a **source** file a marked qualified citation whose
/// start column falls inside an inline-code span or a string literal is not a
/// citation (the same path-collision caution as §AR-workspace.3.1). Markdown has
/// no string literals and its inline code is prose formatting, so a marked
/// qualified citation there is always a citation. This is the raw exclusion
/// outside supported comments; `QualifiedSource` supplies their contextual
/// distinction (§FS-citations.recognized.contexts) to every detection pass.
/// Both halves are read after the line's carried literal closes
/// (`entry`, §FS-languages.multi-line-string), so a template's closing `` ` ``
/// opens no inline-code span.
pub(crate) fn qualified_suppressed_in_source(
    scan_line: &str,
    entry: usize,
    is_md: bool,
    pos: usize,
) -> bool {
    !is_md
        && after_literal(scan_line, entry, pos)
            .is_none_or(|(tail, pos)| is_inside_inline_code(tail, pos) || toggled_open(tail, pos))
}
