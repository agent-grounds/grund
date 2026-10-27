//! The compact fact model a manifest pins once per input: each declaration,
//! section, citation, finding and refusal, whose positions, bodies and
//! measurements are read from the input bytes alone, importing nothing that
//! answers (§REQ-one-answer.proof.evidence.closed). Bodies follow
//! §FS-list.3.4.1 and §FS-show.2.1.2, units §FS-list.3.4.2, and the flattened
//! cross-reference wrapper §FS-show.3.2.1.

use serde_json::{Value, json};
use std::collections::BTreeMap;

pub struct Decl {
    pub id: String,
    pub kind: String,
    pub project: Value,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub text: String,
    pub facts: Value,
    pub lead: Option<String>,
    pub full: Option<String>,
    pub toc: Option<String>,
    /// Each stub that pairs with this declaration: file, line, column, heading.
    pub via: Vec<(String, usize, usize, String)>,
}

pub struct Section {
    pub owner: usize,
    pub path: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub text: String,
    pub raw: String,
    pub parent: Option<usize>,
    pub end: usize,
    /// §FS-fields.front-matter.coordinates: a front-matter coordinate, above
    /// its declaration's heading, read as its slice and under no heading.
    pub front_matter: bool,
    pub facts: Value,
    pub lead: String,
    pub full: String,
    pub toc: String,
}

pub struct Cite {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub text: String,
    pub id: String,
    pub section: Value,
    pub facts: Value,
}

pub struct Model {
    pub facts: Value,
    pub files: BTreeMap<String, String>,
    pub decls: Vec<Decl>,
    pub sections: Vec<Section>,
    pub cites: Vec<Cite>,
}

fn field<'a>(value: &'a Value, name: &str) -> Result<&'a str, String> {
    value[name]
        .as_str()
        .ok_or_else(|| format!("compact fact lacks {name}: {value}"))
}

/// The 1-based line and byte column (§FS-inline-citation-style: the scanner's
/// start column is a byte offset) of the `nth` occurrence of `token` in
/// `text`, at or after line `from`.
pub fn locate(
    text: &str,
    token: &str,
    nth: usize,
    from: usize,
) -> Result<(usize, usize, usize), String> {
    let mut seen = 0;
    let mut offset = 0;
    for (index, line) in text.split_inclusive('\n').enumerate() {
        if index + 1 >= from {
            let mut start = 0;
            while let Some(found) = line[start..].find(token) {
                seen += 1;
                let at = start + found;
                if seen == nth {
                    return Ok((index + 1, at + 1, offset + at));
                }
                start = at + token.len();
            }
        }
        offset += line.len();
    }
    Err(format!(
        "token {token:?} occurrence {nth} is not in the input"
    ))
}

/// One input line, without its terminator.
pub fn line_text(text: &str, line: usize) -> &str {
    text.split('\n')
        .nth(line - 1)
        .unwrap_or("")
        .trim_end_matches('\r')
}

fn blank(line: &str) -> bool {
    line.bytes().all(|b| b.is_ascii_whitespace())
}

/// The lines `first..=last`, a comment `strip` prefix removed, outer blank
/// lines trimmed, each kept line ending in LF; an empty span is "".
fn span(text: &str, first: usize, last: usize, strip: &str) -> String {
    let lines: Vec<String> = (first..=last)
        .filter(|_| first <= last)
        .map(|n| {
            let line = line_text(text, n);
            line.strip_prefix(strip)
                .map(String::from)
                .unwrap_or_else(|| {
                    if !strip.is_empty() && line.trim_end() == strip.trim_end() {
                        String::new()
                    } else {
                        line.to_string()
                    }
                })
        })
        .collect();
    let start = lines.iter().position(|l| !blank(l));
    let end = lines.iter().rposition(|l| !blank(l));
    match (start, end) {
        (Some(s), Some(e)) => lines[s..=e].iter().map(|l| format!("{l}\n")).collect(),
        _ => String::new(),
    }
}

/// §FS-show.3.2.1: `[§…](…)` collapses to its citation outside a Markdown fence.
pub fn flatten(body: &str, marker: &str, markdown: bool) -> String {
    let mut fenced = false;
    let mut out = String::new();
    for line in body.split_inclusive('\n') {
        if markdown && line.trim_start().starts_with("```") {
            fenced = !fenced;
            out.push_str(line);
            continue;
        }
        if fenced {
            out.push_str(line);
            continue;
        }
        let mut rest = line;
        while let Some(at) = rest.find(&format!("[{marker}")) {
            let after = &rest[at + 1..];
            let close = after.find("](");
            let wrapped = close.and_then(|c| {
                let token = &after[..c];
                let tail = &after[c + 2..];
                let mut depth = 1;
                tail.char_indices().find_map(|(i, ch)| {
                    depth += match ch {
                        '(' => 1,
                        ')' => -1,
                        _ => 0,
                    };
                    (depth == 0).then_some((token, c + 2 + i + 1))
                })
            });
            match wrapped {
                Some((token, used)) if !token.contains(char::is_whitespace) => {
                    out.push_str(&rest[..at]);
                    out.push_str(token);
                    rest = &after[used..];
                }
                _ => {
                    out.push_str(&rest[..at + 1]);
                    rest = &rest[at + 1..];
                }
            }
        }
        out.push_str(rest);
    }
    out
}

/// §FS-list.3.4.2: non-blank lines, ASCII-whitespace words and UTF-8 bytes.
pub fn measure(body: &str) -> [usize; 3] {
    let lines = body.split('\n').filter(|l| !blank(l)).count();
    let words = body
        .split(|c: char| c.is_ascii_whitespace())
        .filter(|w| !w.is_empty())
        .count();
    [lines, words, body.len()]
}

impl Model {
    pub fn new(facts: &Value, files: BTreeMap<String, String>) -> Result<Self, String> {
        let marker = facts["marker"].as_str().unwrap_or("§").to_string();
        let mut model = Model {
            facts: facts.clone(),
            files,
            decls: Vec::new(),
            sections: Vec::new(),
            cites: Vec::new(),
        };
        for d in facts["declarations"].as_array().into_iter().flatten() {
            let file = field(d, "file")?.to_string();
            let text = model.text(&file)?;
            let heading = field(d, "heading")?;
            let nth = d["nth"].as_u64().unwrap_or(1) as usize;
            let (line, column, _) = locate(text, heading, nth, 1)?;
            // §FS-declarations.stubs.pairing.1: the stubs that point at it.
            let mut via = Vec::new();
            for stub in d["via"].as_array().into_iter().flatten() {
                let file = field(stub, "file")?;
                let heading = field(stub, "heading")?;
                let (line, column, _) = locate(model.text(file)?, heading, 1, 1)?;
                via.push((file.to_string(), line, column, heading.to_string()));
            }
            model.decls.push(Decl {
                id: field(d, "id")?.into(),
                kind: field(d, "kind")?.into(),
                project: d.get("project").cloned().unwrap_or(Value::Null),
                file,
                line,
                column,
                text: heading.into(),
                facts: d.clone(),
                lead: None,
                full: None,
                toc: None,
                via,
            });
        }
        for s in facts["sections"].as_array().into_iter().flatten() {
            let of = field(s, "of")?;
            // §REQ-one-answer.proof.evidence: duplicate homes own separate
            // section maps; an ID alone cannot choose their physical owner.
            let owners: Vec<_> = model
                .decls
                .iter()
                .enumerate()
                .filter(|(_, d)| d.id == of && s["file"].as_str().is_none_or(|file| d.file == file))
                .map(|(index, _)| index)
                .collect();
            let [owner] = owners.as_slice() else {
                return Err(format!(
                    "section of {of} needs exactly one physical owner: {s}"
                ));
            };
            let owner = *owner;
            let file = model.decls[owner].file.clone();
            let text = model.text(&file)?;
            let heading = field(s, "heading")?;
            let nth = s["nth"].as_u64().unwrap_or(1) as usize;
            let front_matter = s["frontMatter"] == true;
            if front_matter && !s["slice"].is_string() {
                return Err(format!("a front-matter section needs its slice: {s}"));
            }
            // The block precedes the heading, so it is found from the file's start.
            let from = if front_matter {
                1
            } else {
                model.decls[owner].line
            };
            let (line, column, _) = locate(text, heading, nth, from)?;
            let strip = model.decls[owner].facts["strip"].as_str().unwrap_or("");
            let raw = line_text(text, line);
            let raw = raw.strip_prefix(strip).unwrap_or(raw).to_string();
            let parent = s["parent"].as_str().map(|p| {
                model
                    .sections
                    .iter()
                    .rposition(|o| o.owner == owner && o.path == p)
                    .ok_or_else(|| format!("section parent {p} is not declared before"))
            });
            model.sections.push(Section {
                owner,
                path: field(s, "path")?.into(),
                file,
                line,
                column,
                text: heading.into(),
                raw,
                parent: parent.transpose()?,
                end: s["end"].as_u64().ok_or("section lacks end")? as usize,
                front_matter,
                facts: s.clone(),
                lead: String::new(),
                full: String::new(),
                toc: String::new(),
            });
        }
        model.bodies(&marker)?;
        for c in facts["citations"].as_array().into_iter().flatten() {
            let file = field(c, "file")?.to_string();
            let token = field(c, "token")?;
            let nth = c["nth"].as_u64().unwrap_or(1) as usize;
            let (line, column, _) = locate(model.text(&file)?, token, nth, 1)?;
            let query = c["query"]
                .as_str()
                .unwrap_or_else(|| token.trim_start_matches(marker.as_str()));
            let bare = query.rsplit('/').next().unwrap();
            let (id, section) = match c.get("section") {
                Some(section) => (field(c, "id")?.to_string(), section.clone()),
                None => match bare.split_once('.') {
                    Some((id, section)) => (id.to_string(), json!(section)),
                    None => (bare.to_string(), Value::Null),
                },
            };
            model.cites.push(Cite {
                file,
                line,
                column,
                text: token.into(),
                id,
                section,
                facts: c.clone(),
            });
        }
        // §FS-refs.2, §FS-cover.2: refs, cover and the editor read citations in
        // (path, line, column) order, whatever order a manifest authored them in.
        model
            .cites
            .sort_by(|a, b| (&a.file, a.line, a.column).cmp(&(&b.file, b.line, b.column)));
        Ok(model)
    }

    pub fn text(&self, file: &str) -> Result<&str, String> {
        self.files
            .get(file)
            .map(String::as_str)
            .ok_or_else(|| format!("no input file {file}"))
    }

    fn markdown(file: &str) -> bool {
        file.ends_with(".md")
    }

    fn bodies(&mut self, marker: &str) -> Result<(), String> {
        for index in 0..self.sections.len() {
            let s = &self.sections[index];
            let decl = &self.decls[s.owner];
            let strip = decl.facts["strip"].as_str().unwrap_or("");
            let text = self.text(&s.file)?;
            let children: Vec<_> = (0..self.sections.len())
                .filter(|&o| self.descends(o, index))
                .collect();
            let first = self
                .sections
                .iter()
                .enumerate()
                .filter(|(o, _)| self.sections[*o].parent == Some(index))
                .map(|(_, c)| c.line)
                .min()
                .unwrap_or(s.end + 1);
            let md = Self::markdown(&s.file);
            let (lead, full, toc) = match s.facts["slice"].as_str() {
                // §FS-output-shapes.4.3: a JSON element reads as its exact slice.
                Some(slice) => (slice.to_string(), slice.to_string(), slice.to_string()),
                None => {
                    let lead = flatten(&span(text, s.line, first - 1, strip), marker, md);
                    let full = flatten(&span(text, s.line, s.end, strip), marker, md);
                    let toc = self.toc(&lead, &children);
                    (lead, full, toc)
                }
            };
            let s = &mut self.sections[index];
            (s.lead, s.full, s.toc) = (lead, full, toc);
        }
        for index in 0..self.decls.len() {
            let d = &self.decls[index];
            // §FS-output-shapes.4.3: a JSON member reads as its exact slice in every mode.
            if let Some(slice) = d.facts["slice"].as_str().map(String::from) {
                let d = &mut self.decls[index];
                (d.lead, d.full, d.toc) = (Some(slice.clone()), Some(slice.clone()), Some(slice));
                continue;
            }
            let Some(body) = d.facts["body"].as_array() else {
                continue;
            };
            let (first, last) = (
                body[0].as_u64().unwrap() as usize,
                body[1].as_u64().unwrap() as usize,
            );
            let strip = d.facts["strip"].as_str().unwrap_or("");
            let text = self.text(&d.file)?;
            // §FS-fields.front-matter.surfaces: the block neither cuts the lead
            // nor joins the whole declaration's --toc.
            let children: Vec<_> = (0..self.sections.len())
                .filter(|&o| self.sections[o].owner == index && !self.sections[o].front_matter)
                .collect();
            let cut = children
                .iter()
                .map(|&o| self.sections[o].line)
                .min()
                .unwrap_or(last + 1);
            let md = Self::markdown(&d.file);
            let lead = flatten(&span(text, first, cut - 1, strip), marker, md);
            let full = flatten(&span(text, first, last, strip), marker, md);
            let toc = self.toc(&lead, &children);
            let d = &mut self.decls[index];
            (d.lead, d.full, d.toc) = (Some(lead), Some(full), Some(toc));
        }
        Ok(())
    }

    /// Whether section `o` is nested, at any depth, under section `under`.
    pub fn descends(&self, o: usize, under: usize) -> bool {
        let mut at = self.sections[o].parent;
        while let Some(p) = at {
            if p == under {
                return true;
            }
            at = self.sections[p].parent;
        }
        false
    }

    /// §FS-show.2.1.2: the lead, a blank line, then each nested heading.
    fn toc(&self, lead: &str, children: &[usize]) -> String {
        let headings: String = children
            .iter()
            .map(|&o| {
                let s = &self.sections[o];
                format!("{}\n", s.facts["toc"].as_str().unwrap_or(&s.raw))
            })
            .collect();
        match (lead.is_empty(), headings.is_empty()) {
            (_, true) => lead.into(),
            (true, false) => headings,
            (false, false) => format!("{lead}\n{headings}"),
        }
    }

    /// Nesting depth of section `o` below `under` (a declaration when `None`).
    pub fn depth(&self, o: usize, under: Option<usize>) -> usize {
        let mut depth = 1;
        let mut at = self.sections[o].parent;
        while at != under {
            depth += 1;
            at = at.and_then(|p| self.sections[p].parent);
        }
        depth
    }
}
