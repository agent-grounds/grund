//! What [§FS-config-explain](../../../docs/functional-spec/FS-config-explain.md#fs-config-explain-grund-config-explain-answers-from-the-installed-binary-and-in-the-version-the-projects-config-is-read-as-what-any-key-accepts-and-means-and-what-any-check-code-means-and-what-turns-it-on)'s
//! coverage test reads from the code catalog, and the tables a line spells.

#![allow(dead_code)]

use std::fs;
use std::path::Path;

/// The roots a setting is spelled under, in either version; a bracketed word
/// under any other root is not a setting.
pub const TABLE_ROOTS: &[&str] = &[
    "schema",
    "rules",
    "presentation",
    "workspace",
    "reference",
    "id",
    "kinds",
    "scan",
    "output",
    "fmt",
    "citations",
];

pub fn repo_file(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {relative}: {error}"))
}

/// One row of §FS-errors.5.5, its cells as the catalog writes them.
pub struct Row {
    pub code: String,
    pub meaning: String,
    pub severity: String,
    pub ramp: String,
    pub enabled: String,
    pub fix: String,
    pub note: String,
}

/// The catalog's rows, each cell found by its column's header.
pub fn catalog() -> Vec<Row> {
    let text = repo_file("docs/functional-spec/FS-errors.md");
    let cells = |line: &str| -> Vec<String> {
        let inner = line.trim().trim_start_matches('|').trim_end_matches('|');
        inner
            .split('|')
            .map(|cell| cell.trim().to_string())
            .collect()
    };
    let mut lines = text
        .lines()
        .skip_while(|line| !line.starts_with("| Code | Meaning |"));
    let header = cells(lines.next().expect("the catalog's header row"));
    let column = |name: &str| header.iter().position(|cell| cell == name).unwrap();
    let (meaning, severity, ramp, enabled, fix, note) = (
        column("Meaning"),
        column("Severity"),
        column("Ramp"),
        column("Enabled by"),
        column("Fix"),
        column("Note"),
    );
    lines
        .skip(1)
        .take_while(|line| line.starts_with('|'))
        .map(|line| {
            let row = cells(line);
            Row {
                code: row[0].trim_matches('`').to_string(),
                meaning: row[meaning].clone(),
                severity: row[severity].clone(),
                ramp: row[ramp].clone(),
                enabled: row[enabled].clone(),
                fix: row[fix].clone(),
                note: row[note].clone(),
            }
        })
        .collect()
}

/// §FS-errors.5.5.1: what a cell says in `version`, split as `v2: … · v1: …`
/// or the whole cell when it holds for both.
pub fn side(cell: &str, version: u32) -> String {
    match cell
        .strip_prefix("v2: ")
        .and_then(|rest| rest.split_once(" · v1: "))
    {
        Some((v2, v1)) => if version == 2 { v2 } else { v1 }.to_string(),
        None => cell.to_string(),
    }
}

/// §FS-errors.5.5.1: a cell as it is printed, its backticks dropped.
pub fn printed(text: &str) -> String {
    text.replace('`', "")
}

/// What a code's answer in `version` must carry, from its row: its meaning, its
/// severity's words and each clause of what turns it on, with its remark.
pub fn code_expectations(row: &Row, version: u32) -> Vec<String> {
    let mut expected = vec![printed(&row.meaning)];
    let severity = printed(&side(&row.severity, version));
    if !severity.starts_with("not in version") {
        let words = severity.strip_prefix("none — ").unwrap_or(&severity);
        expected.push(
            words
                .strip_prefix("by strength: ")
                .unwrap_or(words)
                .to_string(),
        );
    }
    let enabled = side(&row.enabled, version);
    if enabled == "—" {
        expected.push("always".to_string());
    } else if enabled.starts_with("not in version 1") {
        expected.push("grund config migrate --to 2".to_string());
    } else if enabled.starts_with("not in version") {
        expected.push(format!("version {version}"));
    } else {
        let (clauses, remark) = enabled.split_once("; ").unwrap_or((&enabled, ""));
        expected.extend(clauses.split(" and ").map(printed));
        if !remark.is_empty() {
            expected.push(printed(remark));
        }
    }
    if row.ramp != "—" {
        expected.push(printed(&row.ramp));
    }
    expected
}

/// A spelled table's segments match an admitted one's, a placeholder matching
/// any placeholder.
pub fn same_table(spelled: &str, admitted: &str) -> bool {
    let segments = |table: &str| -> Vec<String> {
        table
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split('.')
            .map(|segment| {
                if segment.starts_with('<') {
                    "<>".to_string()
                } else {
                    segment.to_string()
                }
            })
            .collect()
    };
    spelled.starts_with("[[") == admitted.starts_with("[[")
        && segments(spelled) == segments(admitted)
}

/// Every table a line spells under a setting's root: `[schema.leads.words]`,
/// `[[kinds]]`, `[rules.citations.<kind>.grounding]`.
pub fn spelled_tables(line: &str) -> Vec<String> {
    let mut tables = Vec::new();
    let mut rest = line;
    while let Some(start) = rest.find('[') {
        let candidate = &rest[start..];
        let open = if candidate.starts_with("[[") { 2 } else { 1 };
        let body = &candidate[open..];
        let len = body
            .find(|c: char| !(c.is_ascii_alphanumeric() || "_.<>-".contains(c)))
            .unwrap_or(body.len());
        let name = &body[..len];
        let closes = body[len..].starts_with(&"]".repeat(open));
        let root = name.split('.').next().unwrap_or("");
        if closes && TABLE_ROOTS.contains(&root) {
            let close = "]".repeat(open);
            tables.push(format!("{}{name}{close}", "[".repeat(open)));
        }
        rest = &candidate[open..];
    }
    tables
}

/// Whether a line names a version, which labels the spellings in it.
pub fn labelled(line: &str) -> bool {
    ["v1", "v2", "version 1", "version 2"]
        .iter()
        .any(|label| line.contains(label))
}

/// The settings an answer in `version` names that the version does not admit,
/// outside a line that labels them with a version.
pub fn foreign_settings(stdout: &str, tables: &[(String, Vec<String>)]) -> Vec<String> {
    stdout
        .lines()
        .filter(|line| !labelled(line))
        .flat_map(spelled_tables)
        .filter(|spelled| !tables.iter().any(|(table, _)| same_table(spelled, table)))
        .collect()
}

/// §FS-errors.5.5.1, §FS-config-explain.4: the `Fix:` and `Note:` lines a code's
/// answer in `version` prints, from its row: a template as written, backticks
/// kept and references expanded; one `Fix, <label>:` line per case; `none — <why>`
/// as written; and `none — not in version N` for an absent side. `None` for a
/// row whose *Fix* cell is blank, which no answer can be held to.
pub fn remedy_lines(rows: &[Row], row: &Row, version: u32) -> Option<Vec<String>> {
    let fix = whole(rows, side(&row.fix, version), version)?;
    if fix.is_empty() {
        return None;
    }
    let mut lines = Vec::new();
    if fix.starts_with("none — ") {
        lines.push(format!("Fix: {fix}"));
    } else if fix.starts_with("not in version") {
        lines.push(format!("Fix: none — {fix}"));
    } else if fix.contains(" ‖ ") {
        for case in fix.split(" ‖ ") {
            let (label, template) = case.split_once(": ")?;
            lines.push(format!(
                "Fix, {label}: {}",
                expand(rows, template, version)?
            ));
        }
    } else {
        lines.push(format!("Fix: {}", expand(rows, &fix, version)?));
    }
    let note = side(&row.note, version);
    if !note.is_empty() && note != "—" && !note.starts_with("not in version") {
        lines.push(format!("Note: {}", printed(&note)));
    }
    Some(lines)
}

/// A *Fix* side that is nothing but `` {fix of `<code>`} `` read as that row's side,
/// cases and all; any other side as written.
fn whole(rows: &[Row], fix: String, version: u32) -> Option<String> {
    match fix
        .strip_prefix("{fix of `")
        .and_then(|rest| rest.strip_suffix("`}"))
        .filter(|code| !code.contains('`'))
    {
        Some(code) => whole(rows, referred(rows, code, version)?, version),
        None => Some(fix),
    }
}

/// The side of `code`'s *Fix* a reference names: the same version's, or the other
/// where that one is absent.
fn referred(rows: &[Row], code: &str, version: u32) -> Option<String> {
    let target = rows.iter().find(|row| row.code == code)?;
    let words = side(&target.fix, version);
    Some(if words.starts_with("not in version") {
        side(&target.fix, 3 - version)
    } else {
        words
    })
}

/// A template with each `` {fix of `<code>`} `` replaced by that row's template on
/// the same side, or its other side where that one is absent, and each
/// `` {fix of `<code>`, case `<label>`} `` by that side's case so labelled.
fn expand(rows: &[Row], template: &str, version: u32) -> Option<String> {
    let mut out = String::new();
    let mut rest = template;
    while let Some(start) = rest.find("{fix of `") {
        out.push_str(&rest[..start]);
        let after = &rest[start + "{fix of `".len()..];
        let end = after.find('}')?;
        let (code, label) = match after[..end].split_once("`, case `") {
            Some((code, label)) => (code, Some(label.strip_suffix('`')?)),
            None => (after[..end].strip_suffix('`')?, None),
        };
        let words = whole(rows, referred(rows, code, version)?, version)?;
        let words = match label {
            Some(label) => words
                .split(" ‖ ")
                .find_map(|case| case.strip_prefix(&format!("{label}: ")))?
                .to_string(),
            None => words,
        };
        out.push_str(&expand(rows, &words, version)?);
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    Some(out)
}
