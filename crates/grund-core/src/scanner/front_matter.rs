//! The front-matter pass (§FS-fields.front-matter.reading): on a file holding a
//! declaration whose kind declares the front-matter field, read the block its
//! first declaration owns through the one YAML span tree (§DA-yaml-value-reader)
//! and give its keys their coordinates in that declaration's section map
//! (§FS-fields.front-matter.coordinates). Every other declaration of such a kind
//! is told why it has no front matter (§FS-fields.front-matter.placement).

use super::yaml::{ScalarStyle, YamlKind, YamlNode, YamlStream, parse_yaml};
use crate::config::{FRONT_MATTER, Schema, kind_front_matter};
use crate::grammar::named_section_component;
use crate::model::{
    Catalog, Declaration, FrontMatterAbsence, FrontMatterBlock, FrontMatterField,
    FrontMatterKeySite, FrontMatterSlice, FrontMatterUnreadable, SectionInfo,
};

/// Whether any declaration in `findings` is held to a front-matter field, the
/// one question that decides whether the pass runs at all (§AR-benchmarks).
pub(super) fn holds_front_matter(findings: &Catalog, schema: &Schema) -> bool {
    findings
        .declarations
        .values()
        .flatten()
        .any(|decl| held(schema, decl))
}

/// A stub is held to no field (§FS-fields.identity.name).
fn held(schema: &Schema, decl: &Declaration) -> bool {
    !decl.is_stub && kind_front_matter(schema, &decl.id.kind).is_some()
}

/// §FS-fields.front-matter.reading: read one file's block into its first
/// declaration, and the reason it has none into every other held declaration.
/// `front_matter` is the block's last line, as the record keeps it
/// (§FS-check.1.1.5.2), and `text` the file's text.
pub(super) fn read_front_matter(
    findings: &mut Catalog,
    schema: &Schema,
    text: &str,
    is_md: bool,
    front_matter: Option<usize>,
) {
    // §FS-check.1.1.5.2.1: the file's first declaration, the one starting earliest.
    let first = findings
        .declarations
        .values()
        .flatten()
        .map(|decl| decl.line)
        .min();
    for decl in findings.declarations.values_mut().flatten() {
        if !held(schema, decl) {
            continue;
        }
        let headings = claimed_headings(decl);
        let block = if !is_md {
            FrontMatterBlock::Absent(FrontMatterAbsence::DocComment)
        } else if Some(decl.line) != first {
            FrontMatterBlock::Absent(FrontMatterAbsence::NotFirst)
        } else {
            let (block, entries) = read_block(text, front_matter);
            enter(decl, entries);
            block
        };
        decl.front_matter = Some(FrontMatterField { block, headings });
    }
}

/// §FS-fields.checks.front-matter-heading: each heading whose path is the
/// field's, taken out of the section map, where it would claim a coordinate the
/// block holds; it declares nothing.
fn claimed_headings(decl: &mut Declaration) -> Vec<(String, usize)> {
    let claims = |path: &str| {
        path == FRONT_MATTER
            || path
                .strip_prefix(FRONT_MATTER)
                .is_some_and(|rest| rest.starts_with('.'))
    };
    let mut headings: Vec<(String, usize)> = decl
        .sections
        .iter()
        .filter(|(path, _)| claims(path))
        .map(|(path, info)| (path.clone(), info.line))
        .collect();
    decl.sections.retain(|path, _| !claims(path));
    headings.extend(
        decl.duplicate_sections
            .iter()
            .filter(|(path, _)| claims(path))
            .map(|(path, info)| (path.clone(), info.line)),
    );
    decl.duplicate_sections.retain(|(path, _)| !claims(path));
    headings.sort_by_key(|(_, line)| *line);
    headings
}

/// One coordinate the block gives: its path and its section-map entry.
pub(super) type Entry = (String, SectionInfo);

/// The first declaration's coordinates enter its one section map, a key
/// written twice going to its duplicates (§FS-fields.front-matter.keys).
fn enter(decl: &mut Declaration, entries: Vec<Entry>) {
    for (path, info) in entries {
        if decl.sections.contains_key(&path) {
            decl.duplicate_sections.push((path, info));
        } else {
            decl.sections.insert(path, info);
        }
    }
}

/// §FS-fields.front-matter.reading: the block of a file whose front matter ends
/// at `front_matter`, or why there is none, and the coordinates it gives.
pub(super) fn read_block(
    text: &str,
    front_matter: Option<usize>,
) -> (FrontMatterBlock, Vec<Entry>) {
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    let opener = lines.first().map_or("", |line| bare(line));
    let Some(last) = front_matter else {
        // §FS-fields.front-matter.reading: a mark before a delimiter hides a block.
        let marked = opener.strip_prefix('\u{feff}').and_then(|rest| {
            ["---", "+++"]
                .into_iter()
                .find(|delimiter| *delimiter == rest)
        });
        let absence = match marked {
            Some(delimiter) => FrontMatterAbsence::ByteOrderMark(delimiter),
            None => FrontMatterAbsence::NoBlock,
        };
        return (FrontMatterBlock::Absent(absence), Vec::new());
    };
    let start = lines[0].len();
    let end: usize = lines[..last - 1].iter().map(|line| line.len()).sum();
    let block_end = end + bare(lines[last - 1]).len();
    let whole = (
        FRONT_MATTER.to_string(),
        entry(String::new(), 1, &text[..block_end], last),
    );
    if opener == "+++" {
        let reason = FrontMatterUnreadable::Toml;
        return (FrontMatterBlock::Unreadable(reason), vec![whole]);
    }
    let content = Content {
        text: &text[start..end],
        offset: start,
        file: text,
    };
    match content.keys() {
        Ok(keys) => {
            let mut entries = vec![whole];
            let mut sites = Vec::new();
            for key in keys {
                if key.site.section {
                    let path = format!("{FRONT_MATTER}.{}", key.site.name);
                    let title = key.site.one_line.clone().unwrap_or_default();
                    entries.push((path, entry(title, key.site.line, key.slice, key.end_line)));
                }
                sites.push(key.site);
            }
            (FrontMatterBlock::Read(sites), entries)
        }
        Err(reason) => (FrontMatterBlock::Unreadable(reason), vec![whole]),
    }
}

/// A line without its line ending, LF or CRLF.
fn bare(line: &str) -> &str {
    let line = line.strip_suffix('\n').unwrap_or(line);
    line.strip_suffix('\r').unwrap_or(line)
}

/// §FS-fields.front-matter.coordinates: an entry with no heading, marked by its
/// slice: `heading_level` 0, and no value.
fn entry(title: String, line: usize, slice: &str, end_line: usize) -> SectionInfo {
    SectionInfo {
        title,
        line,
        heading_level: 0,
        value: None,
        value_root: None,
        relative: false,
        front_matter: Some(FrontMatterSlice {
            text: slice.to_string(),
            end_line,
        }),
    }
}

/// The block's content, the lines between its delimiters, where it starts in the
/// file: every span the tree reports is counted in the file from here.
struct Content<'a> {
    text: &'a str,
    offset: usize,
    file: &'a str,
}

/// One top-level key as read, with its coordinate's slice.
struct Key<'a> {
    site: FrontMatterKeySite,
    slice: &'a str,
    end_line: usize,
}

impl<'a> Content<'a> {
    /// The file line of a byte of the content.
    fn line(&self, at: usize) -> usize {
        // The content starts on line 2, after the opening delimiter.
        2 + self.text[..at].matches('\n').count()
    }

    /// §FS-fields.front-matter.reading: one document whose top level is a mapping.
    fn keys(&self) -> Result<Vec<Key<'a>>, FrontMatterUnreadable> {
        let stream = parse_yaml(self.text).map_err(|error| FrontMatterUnreadable::Yaml {
            reason: error.message,
            line: 1 + error.line,
        })?;
        let root = self.root(&stream)?;
        let Some(YamlNode {
            kind: YamlKind::Mapping { entries },
            ..
        }) = root
        else {
            return Ok(Vec::new());
        };
        // §FS-fields.front-matter.coordinates: an entry ends where the next key starts.
        let bounds = entries
            .iter()
            .skip(1)
            .map(|(key, _)| key.span.start)
            .chain([self.text.len()]);
        Ok(entries
            .iter()
            .zip(bounds)
            .map(|((key, value), bound)| self.key(key, value, bound))
            .collect())
    }

    /// The one document's root, `None` for content with nothing in it.
    fn root<'s>(
        &self,
        stream: &'s YamlStream,
    ) -> Result<Option<&'s YamlNode>, FrontMatterUnreadable> {
        let unreadable = |reason: &str, at: usize| FrontMatterUnreadable::Yaml {
            reason: reason.to_string(),
            line: self.line(at),
        };
        let mut documents = stream.documents.iter();
        let Some(first) = documents.next() else {
            return Ok(None);
        };
        if let Some(second) = documents.next() {
            let at = second.explicit_start.unwrap_or(second.root.span.start);
            return Err(unreadable("a second document starts", at));
        }
        let root = &first.root;
        match &root.kind {
            YamlKind::Mapping { .. } => Ok(Some(root)),
            _ if empty(root) => Ok(None),
            _ => Err(unreadable(
                "its top level is not a mapping",
                root.span.start,
            )),
        }
    }

    /// §FS-fields.front-matter.keys, §FS-fields.front-matter.values: one entry
    /// of the top-level mapping, which ends before `bound`.
    fn key(&self, key: &YamlNode, value: &YamlNode, bound: usize) -> Key<'a> {
        let written = &self.text[key.span.start..key.span.end];
        let (name, section) = match &key.kind {
            YamlKind::Scalar { value, .. } => (value.clone(), named_section_component(value)),
            _ => (written.to_string(), false),
        };
        let line = self.line(key.span.start);
        let one_line = match &value.kind {
            _ if empty(value) => Some(String::new()),
            YamlKind::Scalar {
                value: decoded,
                style: ScalarStyle::Plain | ScalarStyle::SingleQuoted | ScalarStyle::DoubleQuoted,
                single_line: true,
            } if self.line(value.span.start) == line => Some(decoded.clone()),
            _ => None,
        };
        // §FS-fields.front-matter.coordinates: from the key to its value's end, or
        // to its colon where it has no value, no line ending at the end.
        let end = if empty(value) {
            self.colon_end(key.span.end, bound).unwrap_or(key.span.end)
        } else {
            value.span.end.max(key.span.end)
        };
        let slice = self.text[key.span.start..end].trim_end();
        let (start, end) = (
            self.offset + key.span.start,
            self.offset + key.span.start + slice.len(),
        );
        Key {
            site: FrontMatterKeySite {
                name,
                section,
                line,
                one_line,
            },
            slice: &self.file[start..end],
            // Counted in the slice, never at a byte stepped back from its end,
            // which a multi-byte last character would split.
            end_line: line + slice.matches('\n').count(),
        }
    }

    /// Just past the colon a value-less key writes between `from` and `bound`, its
    /// entry's end, skipping comments; `None` for a key that writes none (`? k`, `{k}`).
    fn colon_end(&self, from: usize, bound: usize) -> Option<usize> {
        let mut comment = false;
        for (at, ch) in self.text[from..bound].char_indices() {
            match ch {
                '\n' => comment = false,
                _ if comment => {}
                '#' => comment = true,
                ':' => return Some(from + at + 1),
                _ => {}
            }
        }
        None
    }
}

/// A plain scalar with nothing written: an empty value, or empty content.
fn empty(node: &YamlNode) -> bool {
    matches!(
        &node.kind,
        YamlKind::Scalar { value, style: ScalarStyle::Plain, .. } if value.is_empty()
    ) && !node.anchored
        && node.tag.is_none()
}
