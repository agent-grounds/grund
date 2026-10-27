//! A YAML body's lines and the byte scans over them that place a node in the
//! source text (§DA-yaml-value-reader.2.3, §DA-yaml-value-reader.2.4): the
//! span tree reads its events against this, never against the parser's indices.

use yaml_rust2::scanner::Marker;

/// The body without its byte order mark, and where each of its lines starts.
pub(super) struct YamlText<'a> {
    pub(super) body: &'a str,
    pub(super) lines: Vec<usize>,
}

impl<'a> YamlText<'a> {
    pub(super) fn new(body: &'a str) -> Self {
        let mut lines = vec![0];
        lines.extend(body.match_indices('\n').map(|(index, _)| index + 1));
        Self { body, lines }
    }

    /// §DA-yaml-value-reader.2.3: the line's start plus the bytes of its first
    /// `col()` characters, in the body's own offsets.
    pub(super) fn offset(&self, mark: Marker) -> usize {
        let Some(&start) = self.lines.get(mark.line().saturating_sub(1)) else {
            return self.body.len();
        };
        let line = &self.body[start..];
        start
            + line
                .char_indices()
                .nth(mark.col())
                .map_or(line.len(), |(index, _)| index)
    }

    /// Back from `pos` over whitespace, comment lines (less indented than
    /// `content_indent` for a block scalar), and for a plain scalar
    /// (`content_indent` absent) an inline comment and, in flow, the separator
    /// before the next node. Out of flow, the next item's `-` goes as well.
    pub(super) fn trim_back(
        &self,
        mut pos: usize,
        content_indent: Option<usize>,
        in_flow: bool,
    ) -> usize {
        let plain = content_indent.is_none();
        loop {
            let trimmed = self.body[..pos].trim_end();
            pos = if in_flow {
                trimmed
                    .strip_suffix(',')
                    .unwrap_or(trimmed)
                    .trim_end()
                    .len()
            } else {
                trimmed.len()
            };
            let line_start = self.body[..pos].rfind('\n').map_or(0, |index| index + 1);
            let line = &self.body[line_start..pos];
            let indent = line.len() - line.trim_start().len();
            let below = content_indent.is_none_or(|own| indent < own);
            let next_dash = !in_flow && line.trim() == "-";
            if (line.trim_start().starts_with('#') || next_dash) && below {
                pos = line_start;
            } else if plain && let Some(comment) = line.find(" #") {
                pos = line_start + comment;
            } else {
                return pos;
            }
        }
    }

    /// The `|` or `>` that opens the block scalar whose content starts at `at`.
    pub(super) fn block_indicator(&self, at: usize) -> usize {
        let line = self.line_of(at);
        let header = (0..line - 1).rev().find_map(|index| {
            let start = self.lines[index];
            let text = &self.body[start..self.lines[index + 1]];
            (!text.trim().is_empty()).then_some((start, text))
        });
        header
            .and_then(|(start, text)| {
                let mut previous = ' ';
                text.char_indices().find_map(|(index, c)| {
                    let found = matches!(c, '|' | '>') && previous.is_whitespace();
                    previous = c;
                    found.then_some(start + index)
                })
            })
            .unwrap_or(at)
    }

    /// The `-` of a block sequence whose start event sits at or after it.
    pub(super) fn dash_before(&self, at: usize) -> usize {
        let before = self.body[..at].trim_end_matches([' ', '\t']);
        if self.body[at..].starts_with('-') {
            at
        } else if before.ends_with('-') {
            before.len() - 1
        } else {
            at
        }
    }

    pub(super) fn line_of(&self, at: usize) -> usize {
        self.lines.partition_point(|&start| start <= at)
    }
}
