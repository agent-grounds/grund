// The one answer to "is this Markdown line structure". A fence is one context
// that answers no; a raw-text HTML block is another, and a file's front matter is
// the third. Keeping all three in one state machine is what stops a reader from
// knowing one and not the others.

use super::fence::{MarkdownFence, markdown_fence_delimiter};

/// What a Markdown line is to a reader asking whether it can be structure
/// (§FS-check.1.1.5, §FS-check.1.1.5.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MarkdownLine {
    /// Ordinary text: a heading here is a heading.
    Text,
    /// A fence's opening or closing delimiter line.
    FenceDelimiter,
    /// A line inside a fence: neither structure nor citations.
    Fenced,
    /// A line of a raw-text HTML block, its opener and closer included: never
    /// structure, but its citations stay live (§FS-check.1.1.5.1).
    RawHtml,
    /// A line of the file's front matter, its delimiter lines included: never
    /// structure and never rewritten, but its citations stay live
    /// (§FS-check.1.1.5.2).
    FrontMatter,
}

impl MarkdownLine {
    /// The fence's delimiter or its content: the context in which nothing is
    /// read at all (§FS-check.1.1.5).
    pub(crate) fn in_fence(self) -> bool {
        matches!(self, Self::FenceDelimiter | Self::Fenced)
    }

    /// Whether a heading-shaped line here is a heading (§FS-check.1.1.5.1).
    pub(crate) fn may_be_heading(self) -> bool {
        self == Self::Text
    }

    /// Whether `fmt` may rewrite a citation on this line: never in a fence, and
    /// never in front matter, where a link is no valid YAML or TOML value
    /// (§FS-check.1.1.5.2, §FS-citations.zones).
    pub(crate) fn rewritable(self) -> bool {
        matches!(self, Self::Text | Self::RawHtml)
    }
}

/// The last line, 1-based, of the front matter that `lines` — a Markdown file's
/// lines as `str::lines` yields them — opens with, or `None` where it opens with
/// none (§FS-check.1.1.5.2). Line 1 must be exactly `---` or `+++`, a trailing
/// `\r` being the line ending; the block closes at the next line exactly the same,
/// or exactly `...` for `---`. A block that never closes is no front matter,
/// which is why the extent needs the whole file rather than a line at a time.
pub(crate) fn front_matter_extent<'a>(lines: impl IntoIterator<Item = &'a str>) -> Option<usize> {
    let exact = |line: &'a str| line.strip_suffix('\r').unwrap_or(line);
    let mut lines = lines.into_iter().map(exact);
    let opener = lines.next()?;
    if opener != "---" && opener != "+++" {
        return None;
    }
    let closes = |line: &str| line == opener || opener == "---" && line == "...";
    lines.position(closes).map(|index| index + 2)
}

/// The tag names that open a CommonMark raw-text HTML block, its type 1
/// (§FS-check.1.1.5.1). Other HTML blocks keep the existing reading.
const RAW_TEXT_TAGS: [&str; 4] = ["pre", "script", "style", "textarea"];

/// The front matter, fence and raw-text HTML block state of a Markdown file read
/// line by line (§FS-check.1.1.5, §FS-check.1.1.5.1, §FS-check.1.1.5.2). Every
/// reader that asks whether a Markdown line is a heading walks the file through
/// this one machine, so the contexts never nest differently on two surfaces. A
/// reader of a whole file starts it with [`MarkdownBlocks::file`]; a reader of a
/// declaration body, which starts at its heading and so below any front matter,
/// starts it with `default()`.
#[derive(Clone, Copy, Default)]
pub(crate) struct MarkdownBlocks {
    fence: Option<MarkdownFence>,
    /// The tag of the open raw-text block, which only its own end tag closes.
    raw: Option<&'static str>,
    /// The last line of the front matter, 0 where there is none.
    front_matter: usize,
    /// How many lines have been read.
    read: usize,
}

impl MarkdownBlocks {
    /// The machine for a whole Markdown file, its front matter found once
    /// (§FS-check.1.1.5.2).
    pub(crate) fn file(text: &str) -> Self {
        Self::lines(text.lines())
    }

    /// [`MarkdownBlocks::file`] for a file already split into its lines.
    pub(crate) fn lines<'a>(lines: impl IntoIterator<Item = &'a str>) -> Self {
        Self {
            front_matter: front_matter_extent(lines).unwrap_or(0),
            ..Self::default()
        }
    }

    /// The last line, 1-based, of the file's front matter (§FS-check.1.1.5.2).
    pub(crate) fn front_matter(&self) -> Option<usize> {
        (self.front_matter > 0).then_some(self.front_matter)
    }

    /// Classify `line` and advance past it. Whichever context is open owns the
    /// line: inside a fence an HTML opener is fence content, and inside a
    /// raw-text block a fence opener is block content (§FS-check.1.1.5.1). Front
    /// matter is decided first, from line 1, so inside it neither opens
    /// (§FS-check.1.1.5.2).
    pub(crate) fn line(&mut self, line: &str) -> MarkdownLine {
        self.read += 1;
        if self.read <= self.front_matter {
            return MarkdownLine::FrontMatter;
        }
        if let Some(tag) = self.raw {
            if contains_end_tag(line, tag) {
                self.raw = None;
            }
            return MarkdownLine::RawHtml;
        }
        let was_fenced = self.fence.is_some();
        if markdown_fence_delimiter(&mut self.fence, line) {
            return MarkdownLine::FenceDelimiter;
        }
        if was_fenced {
            return MarkdownLine::Fenced;
        }
        if let Some(tag) = raw_text_opener(line) {
            // The opening line may close the block itself.
            if !contains_end_tag(line, tag) {
                self.raw = Some(tag);
            }
            return MarkdownLine::RawHtml;
        }
        MarkdownLine::Text
    }
}

/// The raw-text tag `line` opens a block with: at most three leading spaces,
/// `<`, one of the tag names case-insensitively, then whitespace, `>` or the end
/// of the line (§FS-check.1.1.5.1).
fn raw_text_opener(line: &str) -> Option<&'static str> {
    let bytes = line.as_bytes();
    let indent = bytes.iter().take_while(|byte| **byte == b' ').count();
    if indent > 3 {
        return None;
    }
    let rest = bytes[indent..].strip_prefix(b"<")?;
    RAW_TEXT_TAGS.into_iter().find(|tag| {
        rest.len() >= tag.len()
            && rest[..tag.len()].eq_ignore_ascii_case(tag.as_bytes())
            && rest
                .get(tag.len())
                .is_none_or(|byte| byte.is_ascii_whitespace() || *byte == b'>')
    })
}

/// Whether `line` holds the end tag `</tag>` anywhere, case-insensitively
/// (§FS-check.1.1.5.1).
fn contains_end_tag(line: &str, tag: &str) -> bool {
    let needle = tag.len() + 3;
    line.as_bytes().windows(needle).any(|window| {
        window.starts_with(b"</")
            && window[2..needle - 1].eq_ignore_ascii_case(tag.as_bytes())
            && window[needle - 1] == b'>'
    })
}
