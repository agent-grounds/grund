//! One line's place among its document's links, as `markdown_links.rs` reads
//! them (§FS-citations.wrap): the spans no pass rewrites, the text of each link
//! that encloses, and each image's description, and the one reading of a
//! citation's span every caller asks of them (§FS-citations.wrap.enclosing,
//! §FS-citations.zones.authored-markup).

use super::fmt_cross_refs::CrossRefWrap;

/// Where the settled spans fall on one line as a caller holds it.
#[derive(Default, Debug)]
pub(crate) struct LineScan {
    /// Bytes no pass rewrites: a reference key, a raw HTML tag, an autolink and a
    /// definition's destination and title.
    pub(super) zones: Vec<(usize, usize)>,
    /// The text of each enclosing link on the line, and whether it is an entry.
    pub(super) texts: Vec<(usize, usize, bool)>,
    /// Each image's description, and its destination.
    pub(super) images: Vec<(usize, usize, CrossRefWrap)>,
}

/// How a span on a line sits in the document's links (§FS-citations.wrap).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Enclosure {
    /// In the text of a link its author wrote, or in bytes no pass rewrites;
    /// `entry` where that link goes somewhere (§FS-citations.wrap.enclosing).
    Link { entry: bool },
    /// The whole description of an image (§FS-citations.wrap.enclosing.images).
    Image(CrossRefWrap),
    /// Neither.
    None,
}

impl LineScan {
    /// §FS-citations.zones.authored-markup, §FS-citations.zones.link-destination:
    /// whether the byte at `pos` is one no pass rewrites.
    pub(crate) fn in_zone(&self, pos: usize) -> bool {
        self.zones
            .iter()
            .any(|&(start, end)| start <= pos && pos < end)
    }

    /// The reading of the citation spanning `start..end` that is not a wrap: the
    /// whole description of an image, inside bytes no pass rewrites, in the text
    /// of the innermost enclosing link, or none. A collapsed or shortcut
    /// reference's text is a key and a link's text at once, so a zone that is
    /// exactly a link's text goes where that link goes; any other zone goes nowhere.
    pub(crate) fn enclosure(&self, start: usize, end: usize) -> Enclosure {
        if let Some(&(_, _, wrap)) = self
            .images
            .iter()
            .find(|&&(open, close, _)| open == start && close == end)
        {
            return Enclosure::Image(wrap);
        }
        if let Some(&(zone_start, zone_end)) = self
            .zones
            .iter()
            .find(|&&(open, close)| open <= start && start < close)
        {
            let entry = self.texts.contains(&(zone_start, zone_end, true));
            return Enclosure::Link { entry };
        }
        self.texts
            .iter()
            .filter(|&&(open, close, _)| open <= start && end <= close)
            .max_by_key(|&&(open, close, _)| (open, std::cmp::Reverse(close)))
            .map_or(Enclosure::None, |&(_, _, entry)| Enclosure::Link { entry })
    }
}
