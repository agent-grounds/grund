//! §FS-config-migrate.4: the v1 file's comments in the candidate. A comment
//! attached to a key that maps one-to-one moves above the v2 key or header it
//! became, a trailing one stays on that line, the header block stays at the top,
//! and every other comment is listed with its lines. The lines come from the v1
//! reader's own line pass (§FS-config-migrate.4.3).

use super::comment_guidance::Guidance;
use super::keys::Keys;
use super::record::{ListedComment, MigrateNotice, MigrateReport};
use crate::config::v1::{LineKind, LineRecord};

/// Where a carried comment lands in the candidate.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Target {
    Header(String),
    Key(String, String),
}

impl Target {
    /// §FS-config-migrate.2.2: name the actual omitted or shared carrier.
    fn description(&self) -> String {
        match self {
            Self::Header(table) => format!("[{table}]"),
            Self::Key(table, key) if table.is_empty() => key.clone(),
            Self::Key(table, key) => format!("[{table}] {key}"),
        }
    }
}

/// §FS-config-migrate.4.1: the v2 header or key a v1 key maps one-to-one to,
/// or `None` for a key that is dropped, split or merged.
fn key_target(section: &str, key: &str, row: Option<&str>) -> Option<Target> {
    let k = |table: &str, key: &str| Some(Target::Key(table.into(), key.into()));
    match (section, key) {
        ("", "grund_config_version") | ("", "project_name") => k("", key),
        ("", "project_description") => k("presentation", "description"),
        ("reference", "conversation") | ("reference", "trigger") => k("presentation", key),
        ("reference", "marker") | ("reference", "shorthand") => k("schema", key),
        ("reference", "inline_note_max_lines") => k("schema.notes.lines", "must"),
        ("reference", "inline_note_max_columns") => k("schema.notes.columns", "must"),
        ("reference", "inline_style") => k("schema.notes.text", "must"),
        ("reference", "lead_size_warning") => k("schema.leads.words", "warn"),
        ("id", "format") => k("schema", "id_format"),
        ("id", "number_pattern" | "section_separator" | "slug_pattern") => k("schema", key),
        ("id", "section_heading_levels") => k("schema", "heading_depth"),
        ("kinds", _) => {
            let row = row?;
            let table = format!("schema.kinds.{row}");
            match key {
                "kind" => Some(Target::Header(table)),
                "file" => k(&table, "files"),
                "folder" => k(&table, "folders"),
                "format" => k(&table, "id_format"),
                "rules" | "values" => k(&table, "form"),
                "citable" | "scan" | "index" | "fetch" => k(&table, key),
                "title" => k(&format!("presentation.kinds.{row}"), "title"),
                "resolve" => k("rules.resolution", row),
                _ => None,
            }
        }
        ("scan", "exclude" | "respect_gitignore") => k("schema.sources", key),
        ("fmt.cross_refs", "anchor_format") => k("presentation.fmt", "anchors"),
        ("fmt.cross_refs", "enabled") => k("presentation.fmt", "links"),
        ("fmt", "exclude") => k("presentation.fmt", "exclude"),
        ("workspace", _) => k("workspace", key),
        ("citations", "default") => k("rules.citations", "default"),
        (section, key) if section.starts_with("citations.") => k(&format!("rules.{section}"), key),
        _ => None,
    }
}

/// §FS-config-migrate.4.1: the v2 header a v1 header maps one-to-one to.
fn header_target(name: &str, row: Option<&str>) -> Option<Target> {
    match name {
        "kinds" => row.map(|row| Target::Header(format!("schema.kinds.{row}"))),
        "workspace" => Some(Target::Header("workspace".into())),
        name if name == "citations" || name.starts_with("citations.") => {
            Some(Target::Header(format!("rules.{name}")))
        }
        _ => None,
    }
}

/// The v1 key a line is, as the report names it.
pub(super) fn v1_key(record: &LineRecord) -> String {
    match &record.kind {
        LineKind::Header { name, array, .. } if *array => format!("[[{name}]]"),
        LineKind::Header { name, .. } => format!("[{name}]"),
        LineKind::Key { section, key, .. } if section.is_empty() => key.clone(),
        LineKind::Key { section, key, .. } if section == "kinds" => format!("[[kinds]] {key}"),
        LineKind::Key { section, key, .. } => format!("[{section}] {key}"),
        _ => String::new(),
    }
}

/// §FS-config-migrate.4.1.1: classify the mapping's vocabulary, even when this
/// particular value keeps its spelling. Only call for an actually carried comment.
fn translated_comment(record: &LineRecord, keys: &Keys) -> Option<MigrateNotice> {
    let LineKind::Key { section, key, .. } = &record.kind else {
        return None;
    };
    if !matches!(
        (section.as_str(), key.as_str()),
        ("reference", "inline_style" | "lead_size_warning")
            | ("id", "section_heading_levels")
            | ("kinds", "resolve" | "rules" | "values")
            | ("fmt.cross_refs", "enabled")
            | ("scan", "exclude")
    ) {
        return None;
    }
    let row = keys.row_at(record.line).map(|block| block.name.as_str());
    let Target::Key(table, key) = key_target(section, key, row)? else {
        return None;
    };
    Some(MigrateNotice {
        line: record.line,
        dropped: false,
        key: v1_key(record),
        why: format!("value vocabulary translated to [{table}] {key}; check the carried comment"),
    })
}

/// §FS-config-migrate.4: place `lines`' comments into `body`, the rendered candidate.
/// Report only successful carries onto translated values (§FS-config-migrate.4.1.1).
/// Listed entries retain local recovery guidance (§FS-config-migrate.2.2).
pub(super) fn carry(
    body: &str,
    lines: &[LineRecord],
    keys: &Keys,
    guidance: &Guidance<'_>,
    report: &mut MigrateReport,
) -> String {
    let index = TableIndex::of(body);
    let mut above: Vec<Vec<String>> = vec![Vec::new(); index.lines.len()];
    let mut trailing: Vec<Option<String>> = vec![None; index.lines.len()];
    let mut listed = Vec::new();
    let mut header = Vec::new();
    let mut commented = vec![false; lines.len()];
    let mut i = 0;
    while i < lines.len() {
        let record = &lines[i];
        let target_of = |record: &LineRecord| {
            let row = keys.row_at(record.line).map(|block| block.name.as_str());
            match &record.kind {
                LineKind::Key { section, key, .. } => key_target(section, key, row),
                LineKind::Header { name, .. } => header_target(name, row),
                _ => None,
            }
        };
        if record.kind != LineKind::Comment {
            let comment = match &record.kind {
                LineKind::Key { trailing, .. } | LineKind::Header { trailing, .. } => {
                    trailing.clone()
                }
                _ => None,
            };
            if let Some(comment) = comment {
                let target = target_of(record);
                let at = target.as_ref().and_then(|t| index.find(t));
                // Two v1 lines can land on one v2 line, which keeps one trailing
                // comment: a later one is listed, never overwritten (§FS-config-migrate.4.4).
                match at {
                    Some(at) if trailing[at].is_none() => {
                        trailing[at] = Some(comment);
                        commented[i] = true;
                    }
                    _ => listed.push(ListedComment {
                        first: record.line,
                        last: record.line,
                        place: format!(
                            "inside {}: {}",
                            v1_key(record),
                            guidance.attached(
                                record,
                                target.map(|t| t.description()),
                                at.is_some()
                            )
                        ),
                    }),
                }
            }
            i += 1;
            continue;
        }
        let start = i;
        while i < lines.len() && lines[i].kind == LineKind::Comment {
            i += 1;
        }
        let run: Vec<String> = lines[start..i]
            .iter()
            .map(|r| r.raw.trim().to_string())
            .collect();
        let (first, last) = (lines[start].line, lines[i - 1].line);
        if first == 1 {
            header = run;
            continue;
        }
        let next = lines.get(i).filter(|r| !matches!(r.kind, LineKind::Blank));
        let Some(next) = next else {
            listed.push(ListedComment {
                first,
                last,
                place: format!("standing alone: {}", guidance.standalone()),
            });
            continue;
        };
        let target = target_of(next);
        match target.as_ref().and_then(|t| index.find(t)) {
            Some(at) => {
                above[at].extend(run);
                commented[i] = true;
            }
            None => listed.push(ListedComment {
                first,
                last,
                place: format!(
                    "above {}: {}",
                    v1_key(next),
                    guidance.attached(next, target.map(|t| t.description()), false)
                ),
            }),
        }
    }
    let mut text = String::new();
    if !header.is_empty() {
        for line in &header {
            text.push_str(line);
            text.push('\n');
        }
        text.push('\n');
    }
    for (at, line) in index.lines.iter().enumerate() {
        for comment in &above[at] {
            text.push_str(comment);
            text.push('\n');
        }
        text.push_str(line);
        if let Some(comment) = &trailing[at] {
            text.push(' ');
            text.push_str(comment);
        }
        text.push('\n');
    }
    report.comments = listed;
    report.notices.extend(
        lines
            .iter()
            .zip(commented)
            .filter(|(_, carried)| *carried)
            .filter_map(|(record, _)| translated_comment(record, keys)),
    );
    report.notices.sort_by_key(|notice| notice.line);
    text
}

/// The candidate's lines, each with the table it sits in.
struct TableIndex<'a> {
    lines: Vec<&'a str>,
    tables: Vec<String>,
}

impl<'a> TableIndex<'a> {
    fn of(body: &'a str) -> Self {
        let mut table = String::new();
        let mut tables = Vec::new();
        let lines: Vec<&str> = body.lines().collect();
        for line in &lines {
            if line.starts_with('[') {
                table = line.trim_matches(['[', ']']).to_string();
            }
            tables.push(table.clone());
        }
        Self { lines, tables }
    }

    fn find(&self, target: &Target) -> Option<usize> {
        (0..self.lines.len()).find(|&at| {
            let line = self.lines[at];
            match target {
                Target::Header(name) => line.starts_with('[') && self.tables[at] == *name,
                Target::Key(table, key) => {
                    !line.starts_with('[')
                        && self.tables[at] == *table
                        && line.split_once('=').is_some_and(|(k, _)| k.trim() == key)
                }
            }
        })
    }
}

/// §FS-config-migrate.5.2: each candidate line that carries a key, with the v1
/// line of the key it came from, through which a finding located in the config
/// is compared.
pub(super) fn line_map(text: &str, lines: &[LineRecord], keys: &Keys) -> Vec<(usize, usize)> {
    let index = TableIndex::of(text);
    let mut out = Vec::new();
    for record in lines {
        let row = keys.row_at(record.line).map(|block| block.name.as_str());
        let target = match &record.kind {
            LineKind::Key { section, key, .. } => key_target(section, key, row),
            LineKind::Header { name, .. } => header_target(name, row),
            _ => None,
        };
        if let Some(at) = target.and_then(|target| index.find(&target)) {
            out.push((at + 1, record.line));
        }
    }
    out
}
