// The one answer to "is this Markdown line structure". A fence is one context
// that answers no; a raw-text HTML block is another, and a file's front matter is
// the third. Keeping all three in one state machine is what stops a reader from
// knowing one and not the others. Whether a fence's citations are read is asked
// here too, so no reader learns a fence's tag on its own.

use std::collections::BTreeSet;

use super::fence::{MarkdownFence, delimiter_run, markdown_fence_delimiter};

/// A fenced or raw-text block a Markdown text opens and does not close.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OpenBlock {
    /// 1-indexed line the block opens on.
    pub(crate) line: usize,
    /// A raw-text HTML block (`<pre>`, …) rather than a fence (§FS-check.1.1.5.1).
    pub(crate) raw: bool,
}

/// §FS-fetch.admission.4: the block a whole Markdown text leaves open at its end, read line by
/// line by the same machine the scan reads every Markdown file with (§FS-check.1.1.5.1), its
/// front matter included (§FS-check.1.1.5.2). `fetch` asks it of its output rather than of the
/// scan's record (§AR-scanner.2.8.2).
pub(crate) fn open_block(text: &str) -> Option<OpenBlock> {
    let mut blocks = MarkdownBlocks::file(text);
    for line in text.lines() {
        blocks.line(line);
    }
    blocks.open_at()
}

/// The lines, 1-based, inside a fence `fenced` selects in a whole Markdown file's
/// `lines`: where a citation is read but no `fmt` pass writes a link
/// (§FS-languages.fence.language). Empty, and nothing walked, where `fenced` is.
pub(crate) fn selected_fence_lines(lines: &[&str], fenced: &[String]) -> BTreeSet<usize> {
    if fenced.is_empty() {
        return BTreeSet::new();
    }
    let mut blocks = MarkdownBlocks::lines(lines.iter().copied()).selecting(fenced);
    (1..)
        .zip(lines)
        .filter(|(_, line)| blocks.line(line) == MarkdownLine::Selected)
        .map(|(number, _)| number)
        .collect()
}

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
    /// A line inside a fence `[schema.sources] fenced` selects: still a fence for
    /// everything but citations, so never structure and never rewritten, but its
    /// citations are live (§FS-languages.fence.language).
    Selected,
    /// A line of a raw-text HTML block, its opener and closer included: never
    /// structure, but its citations stay live (§FS-check.1.1.5.1).
    RawHtml,
    /// A line of the file's front matter, its delimiter lines included: never
    /// structure and never rewritten, but its citations stay live
    /// (§FS-check.1.1.5.2).
    FrontMatter,
}

impl MarkdownLine {
    /// The fence's delimiter or its content, a selected fence's included: the
    /// context no writer rewrites and no structure reader reads (§FS-check.1.1.5,
    /// §FS-languages.fence.language).
    pub(crate) fn in_fence(self) -> bool {
        matches!(self, Self::FenceDelimiter | Self::Fenced | Self::Selected)
    }

    /// Whether a citation on this line is read: everywhere but an unselected fence
    /// and every fence's delimiter lines (§FS-languages.fence.language).
    pub(crate) fn citations_live(self) -> bool {
        !matches!(self, Self::FenceDelimiter | Self::Fenced)
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
/// starts it with `default()`. A reader that reads citations names the fence tags
/// the project selects with [`MarkdownBlocks::selecting`]; every other reader selects
/// none, so to it a selected fence is a fence like any other (§FS-languages.fence.language).
#[derive(Clone, Copy, Default)]
pub(crate) struct MarkdownBlocks<'a> {
    fence: Option<MarkdownFence>,
    /// `[schema.sources] fenced`: the tags whose fences read their citations.
    fenced: &'a [String],
    /// Whether the open fence's tag is one of `fenced`.
    selected: bool,
    /// The tag of the open raw-text block, which only its own end tag closes.
    raw: Option<&'static str>,
    /// The last line of the front matter, 0 where there is none.
    front_matter: usize,
    /// How many lines have been read.
    read: usize,
    /// The line, 1-based, the open fence or raw-text block began on, 0 where none is open.
    opened: usize,
}

impl<'a> MarkdownBlocks<'a> {
    /// The machine for a whole Markdown file, its front matter found once
    /// (§FS-check.1.1.5.2).
    pub(crate) fn file(text: &str) -> Self {
        Self::lines(text.lines())
    }

    /// [`MarkdownBlocks::file`] for a file already split into its lines.
    pub(crate) fn lines<'l>(lines: impl IntoIterator<Item = &'l str>) -> Self {
        Self {
            front_matter: front_matter_extent(lines).unwrap_or(0),
            ..Self::default()
        }
    }

    /// The same machine reading the fences tagged with one of `fenced` as selected
    /// (§FS-config-v2.schema.sources.fenced, §FS-languages.fence.language).
    pub(crate) fn selecting(self, fenced: &'a [String]) -> Self {
        Self { fenced, ..self }
    }

    /// The last line, 1-based, of the file's front matter (§FS-check.1.1.5.2).
    pub(crate) fn front_matter(&self) -> Option<usize> {
        (self.front_matter > 0).then_some(self.front_matter)
    }

    /// The line the fence or raw-text block still open after the last line read began on, and
    /// whether it is a raw-text block.
    fn open_at(&self) -> Option<OpenBlock> {
        let raw = self.raw.is_some();
        (self.fence.is_some() || raw).then_some(OpenBlock {
            line: self.opened,
            raw,
        })
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
            if !was_fenced {
                self.opened = self.read;
                // §FS-languages.fence.language: the opener's tag, compared literally.
                self.selected = fence_tag(line)
                    .is_some_and(|tag| self.fenced.iter().any(|selected| selected == tag));
            }
            return MarkdownLine::FenceDelimiter;
        }
        if was_fenced {
            return if self.selected {
                MarkdownLine::Selected
            } else {
                MarkdownLine::Fenced
            };
        }
        if let Some(tag) = raw_text_opener(line) {
            // The opening line may close the block itself.
            if !contains_end_tag(line, tag) {
                self.raw = Some(tag);
                self.opened = self.read;
            }
            return MarkdownLine::RawHtml;
        }
        MarkdownLine::Text
    }
}

/// The tag of the fence `line` opens: the first word of its info string, the text after the
/// opening run with surrounding whitespace stripped, up to the first space or tab; `None` where
/// the info string is empty (§FS-languages.fence.language). Asked only of a line the fence
/// reader has just read as an opener, so it reads the same run.
fn fence_tag(line: &str) -> Option<&str> {
    let (_, _, info) = delimiter_run(line)?;
    info.split([' ', '\t']).find(|word| !word.is_empty())
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
