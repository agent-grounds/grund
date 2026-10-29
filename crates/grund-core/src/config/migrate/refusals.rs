//! §FS-config-migrate.3.1: the v1 settings version 2 cannot spell with the same
//! meaning, each with the v1 repair that gets to one. Never approximated.

use std::path::Path;

use super::comments::v1_key;
use super::keys::Keys;
use super::places::scanned_files;
use super::record::{MigrateRefusal, RefusalAnchor};
use crate::config::grounding::configured_row_grounding;
use crate::config::point_sizes::PointSizeUnit;
use crate::config::project::Project;
use crate::config::v1::LineKind;
use crate::config::v2::{read_string, read_string_list};

fn at(line: Option<usize>) -> RefusalAnchor {
    line.map_or(RefusalAnchor::Unset, RefusalAnchor::Line)
}

fn refusal(anchor: RefusalAnchor, what: String, repair: &str) -> MigrateRefusal {
    MigrateRefusal {
        anchor,
        what,
        details: Vec::new(),
        repair: repair.to_string(),
    }
}

/// Every refusal of §FS-config-migrate.3.1 but the places', which `places.rs`
/// finds, and the sources', which `languages.rs` finds.
pub(super) fn settings(v1: &Project, keys: &Keys, root: &Path) -> Vec<MigrateRefusal> {
    let mut out = Vec::new();
    let schema = &v1.schema;
    // §FS-config-migrate.3.1.2
    if !schema.citation.strict {
        out.push(refusal(
            at(keys.line("reference", "strict")),
            "[reference] strict = false recognizes bare citations, and version 2 recognizes only marked ones".into(),
            "run grund fmt --marker --write, then set [reference] strict = true under version 1",
        ));
    }
    // §FS-config-migrate.3.1.1
    if !schema.ids.named_sections {
        out.push(match keys.line("id", "named_sections") {
            Some(line) => refusal(
                RefusalAnchor::Line(line),
                "[id] named_sections = false turns named sections off, and version 2 always reads them".into(),
                "set [id] named_sections = true under version 1; grund check shows its effect",
            ),
            None => refusal(
                RefusalAnchor::Unset,
                "[id] named_sections is not set, so version 1 reads no named sections, and version 2 always reads them".into(),
                "add [id] named_sections = true under version 1; grund check shows its effect",
            ),
        });
    }
    // §FS-config-migrate.3.1.7: a lead counted in lines or bytes.
    if let Some(leads) = &schema.leads
        && leads.unit != PointSizeUnit::Words
    {
        out.push(refusal(
            at(keys.line("reference", "lead_size_warning")),
            format!(
                "[reference] lead_size_warning counts {}, and version 2 counts a lead in words",
                leads.unit.as_str()
            ),
            "restate it with unit = \"words\" under version 1",
        ));
    }
    let fmt = &v1.presentation.fmt;
    if !fmt.cross_refs_enabled {
        out.push(refusal(
            at(keys.line("fmt.cross_refs", "enabled")),
            "[fmt.cross_refs] enabled = false has no version 2 spelling in this grund".into(),
            "none yet; keep version 1, or set enabled = true under version 1 if fmt may write links",
        ));
    }
    if fmt.anchor_format != "github" {
        out.push(refusal(
            at(keys.line("fmt.cross_refs", "anchor_format")),
            format!(
                "[fmt.cross_refs] anchor_format = \"{}\" has no version 2 spelling in this grund",
                fmt.anchor_format
            ),
            "none yet; keep version 1, or set anchor_format = \"github\" under version 1",
        ));
    }
    for row in &schema.rows {
        // §FS-config-migrate.3.1.6
        if row.is_complement() && row.name != "code" {
            out.push(refusal(
                at(keys.row_line(&row.name, "kind")),
                format!(
                    "[[kinds]] `{}` is a non-citable row with no home, which renames the complement; version 2 calls the complement `code`",
                    row.name
                ),
                &format!(
                    "rename the row, and its [citations.{}] table, to `code` under version 1",
                    row.name
                ),
            ));
        }
        // §FS-config-migrate.3.1.7: a value chapter would become a field.
        if row
            .kind
            .as_ref()
            .is_some_and(|kind| kind.value_chapter().is_some())
        {
            out.push(refusal(
                at(keys.row_line(&row.name, "value_chapter")),
                format!(
                    "[[kinds]] `{}` value_chapter would become a field, which adds a \"Declaration shapes\" section grund check compares",
                    row.name
                ),
                "none keeps the generated guidance; remove value_chapter under version 1 only if that section may change",
            ));
        }
    }
    out.extend(grounding(v1, keys));
    out.extend(path_shaped_excludes(v1, keys, root));
    out
}

/// §FS-config-migrate.3.1.8: each key whose value holds a string that closes early, at
/// its line, naming the first such string as version 2's warning names it
/// (§FS-config-v2.reader.6.window), with that warning's repair made under version 1:
/// the first half for a scalar, both halves for a list element. `path` names the file
/// a value the string readers refuse would be refused in, which no refusal here shows.
pub(super) fn strings_closing_early(keys: &Keys, path: &Path) -> Vec<MigrateRefusal> {
    let mut out = Vec::new();
    for record in keys.lines {
        let LineKind::Key { value, .. } = &record.kind else {
            continue;
        };
        let (line, element) = (record.line, value.starts_with('['));
        let early = if element {
            let read = read_string_list(path, line, value).ok();
            read.and_then(|(_, early)| early.first().copied())
        } else {
            read_string(path, line, value)
                .ok()
                .and_then(|(_, early)| early)
        };
        let Some(early) = early else {
            continue;
        };
        let split = if element {
            ", or a `,` between two strings"
        } else {
            ""
        };
        let what = format!(
            "{} reads the one string `{}`, and version 2 ends a string at its first \
             unescaped quote",
            v1_key(record),
            early.text()
        );
        let repair = format!(
            "end the string at its closing quote under version 1: write `\\\"` for a quote \
             inside it{split}; grund check shows its effect"
        );
        out.push(refusal(RefusalAnchor::Line(line), what, &repair));
    }
    out
}

/// §FS-config-migrate.mapping `[reference] grounding_level`: a level on a place
/// grounding is off for, which only `--require-grounding` would switch on, and
/// which no version-2 rung waits for. A row whose own `require_grounding =
/// false` the flag cannot override reads no level, so it is not refused.
fn grounding(v1: &Project, keys: &Keys) -> Vec<MigrateRefusal> {
    let rules = &v1.rules;
    let repair = "set require_grounding = true, or remove grounding_level, under version 1";
    let mut out = Vec::new();
    if !rules.grounding.require && rules.grounding.level != 1 {
        out.push(refusal(
            at(keys.line("reference", "grounding_level")),
            format!(
                "[reference] grounding_level = {} asks for a unit only --require-grounding switches on, and no version 2 rung waits for that flag",
                rules.grounding.level
            ),
            repair,
        ));
        return out;
    }
    for row in &v1.schema.rows {
        let (require, level) = configured_row_grounding(rules, &row.name);
        // An explicit row `false` wins over the flag (§FS-config.3.4.8.3).
        let explicit = rules
            .grounding
            .kinds
            .get(&row.name)
            .and_then(|row| row.require);
        if !require && level != 1 && explicit.is_none() {
            out.push(refusal(
                at(keys.row_line(&row.name, "grounding_level")),
                format!(
                    "[[kinds]] `{}` grounding_level = {level} asks for a unit only --require-grounding switches on, and no version 2 rung waits for that flag",
                    row.name
                ),
                repair,
            ));
        }
    }
    out
}

/// §FS-config-migrate.3.1.4: each path-shaped exclude entry, with the files the
/// glob it resembles would stop reading on this tree.
fn path_shaped_excludes(v1: &Project, keys: &Keys, root: &Path) -> Vec<MigrateRefusal> {
    let line = keys.line("scan", "exclude");
    let mut out = Vec::new();
    for entry in v1.schema.sources.exclude.iter().filter(|e| e.contains('/')) {
        let path = entry.trim_matches('/');
        let files = if root.join(path).exists() {
            scanned_files(v1, root, path)
        } else {
            Vec::new()
        };
        let count = match files.len() {
            1 => "1 file".to_string(),
            n => format!("{n} files"),
        };
        let details = if files.is_empty() {
            Vec::new()
        } else {
            vec![format!("would stop reading: {}", files.join(", "))]
        };
        out.push(MigrateRefusal {
            anchor: at(line),
            what: format!(
                "[scan] exclude entry `{entry}` contains `/`, so version 1 never matched it; as a version 2 glob it would stop reading {count}"
            ),
            details,
            repair: format!(
                "remove `{entry}` from exclude under version 1, which keeps what is scanned exactly as it is"
            ),
        });
    }
    out
}
