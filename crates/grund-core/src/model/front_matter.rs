//! A declaration's front-matter field (§FS-fields.front-matter): what the scan
//! read of the block a kind that declares the field gives its file's first
//! declaration, and the section-map entries that lie in it
//! (§FS-fields.front-matter.coordinates). The checker judges the field from
//! this record alone, so it never parses the block a second time.

/// §FS-fields.front-matter.coordinates: what a section-map entry lying in the
/// front matter carries besides its line and title — its slice of the file and
/// the slice's last line. An entry without one is a heading.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FrontMatterSlice {
    /// The coordinate's text, no line ending at its end.
    pub(crate) text: String,
    pub(crate) end_line: usize,
}

/// §FS-fields.front-matter: the field as one declaration of a kind that
/// declares it has it.
#[derive(Clone, Debug)]
pub(crate) struct FrontMatterField {
    pub(crate) block: FrontMatterBlock,
    /// Each heading that claimed the field's coordinate, its path and line,
    /// dropped from the section map (§FS-fields.checks.front-matter-heading).
    pub(crate) headings: Vec<(String, usize)>,
}

/// §FS-fields.front-matter.reading, §FS-fields.front-matter.placement.
#[derive(Clone, Debug)]
pub(crate) enum FrontMatterBlock {
    /// The declaration has no front matter, and why.
    Absent(FrontMatterAbsence),
    /// The block is present, its opening delimiter on line 1, its keys unread.
    Unreadable(FrontMatterUnreadable),
    /// The block's top-level keys, in source order, a repeated key kept.
    Read(Vec<FrontMatterKeySite>),
}

/// §FS-fields.front-matter.findings: why a declaration has no front matter,
/// each with the remedy its `missing-field` gives.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FrontMatterAbsence {
    /// Its file opens with no block.
    NoBlock,
    /// Line 1 is a byte-order mark and then exactly `---` or `+++`, the
    /// delimiter the remedy names.
    ByteOrderMark(&'static str),
    /// A later declaration of its Markdown file.
    NotFirst,
    /// A doc-comment declaration.
    DocComment,
}

/// §FS-fields.checks.unreadable-front-matter: why the keys cannot be read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FrontMatterUnreadable {
    /// The reason and the file line it is at.
    Yaml { reason: String, line: usize },
    /// A `+++` block, which grund does not read yet.
    Toml,
}

/// §FS-fields.front-matter.keys: one top-level key as written.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FrontMatterKeySite {
    /// Its decoded name, or the text a key that is no scalar is written as.
    pub(crate) name: String,
    /// Whether the name is a section name, so the key has a coordinate.
    pub(crate) section: bool,
    pub(crate) line: usize,
    /// Its value where that is a one-line scalar on the key's line
    /// (§FS-fields.front-matter.values).
    pub(crate) one_line: Option<String>,
}
