//! §AR-config.4: meaning validation, run once on the lowered `Project` whichever
//! reader produced it. Spelling — an unknown key, a wrong type, a value outside
//! a closed set, a `[[kinds]]` entry no row can hold — stays with the reader
//! that read it; what is judged here is what the records say together: the
//! marker `strict` requires, the inline-note budget, the grounding pair, one
//! complement, unique and prefix-free names, the member lists, and the
//! `[citations]` rules against the final kind set.
//!
//! Every error keeps the text and the `path:line` anchor it had when the reader
//! raised it (§FS-config.4.3), from the `ConfigLocation` each validated value
//! was lowered with.

use anyhow::{Result, anyhow};
use std::path::Path;

use super::citations::validate_citation_rules;
use super::kind::KindConfig;
use super::project::{KindGrounding, Project};
use super::v1::bail_config;
use super::workspace_block::validate_workspace_lists;
use crate::model::format_path;

/// §AR-config.4: the `[reference]` meanings, which read no `[[kinds]]` row:
/// the marker `strict` requires and the inline-note budget. `path` is the report
/// path of the file the project was read from.
pub(super) fn reference(path: &Path, project: &Project) -> Result<()> {
    let citation = &project.schema.citation;
    if citation.strict && citation.marker.is_empty() {
        return Err(anyhow!(
            "{}: reference.strict requires a non-empty marker",
            format_path(path)
        ));
    }
    let notes = &project.schema.notes;
    if notes.suggested_lines > notes.max_lines {
        let line = notes
            .suggested_lines_source
            .as_ref()
            .or(notes.max_lines_source.as_ref())
            .map_or(1, |source| source.line);
        bail_config(
            path,
            line,
            "reference.inline_note_suggested_lines must be <= inline_note_max_lines".to_string(),
        )?;
    }
    Ok(())
}

/// §AR-config.4: the meanings of the final kind table — the grounding pair per
/// row and globally, one complement, unique and prefix-free names.
pub(super) fn kinds(path: &Path, project: &Project) -> Result<()> {
    let kinds = project.kind_configs();
    validate_kind_grounding(path, project, &kinds)?;
    validate_kind_table(path, &kinds)?;
    // §FS-config.3.4.8: both grounding keys resolve per row against these
    // defaults, so the cross-section rule is asked once the kind table is final
    // — the built-in table included.
    validate_global_grounding(path, project, &kinds)
}

/// §AR-config.4: the member lists and the `[citations]` rules against the final
/// kind set, judged after the grammar compiles, as v1 always judged them.
pub(super) fn lists(path: &Path, project: &Project) -> Result<()> {
    // §AR-workspace.5.2: post-parse invariants run on every load, not gated on which
    // section appeared. Free-form `project_name` is slug-checked later
    // (§AR-workspace.5.3); both member lists are shape-checked in `workspace_block.rs`.
    validate_workspace_lists(&project.workspace)?;
    // §FS-config.3.9.5: validate `[citations]` against the final kind set.
    if project.rules.citations.declared {
        // §FS-config-v2.rules.citations: each version names the rules its own way.
        let table = if project.version == 2 {
            "rules.citations"
        } else {
            "citations"
        };
        validate_citation_rules(
            path,
            table,
            &project.kind_configs(),
            &project.rules.citations,
        )?;
    }
    Ok(())
}

/// Every rule the two row keys have to satisfy (§FS-config.3.4.8.5), each closing a
/// state they cannot describe. Run over the lowered rows once the reader has
/// refused every malformed entry, so a row that is already malformed is
/// reported as that rather than as a grounding error.
///
/// The `[reference]` boolean is the default the rows resolve against: whether a
/// row's level is dead is a question about the row's *effective* boolean, not
/// about what it wrote.
fn validate_kind_grounding(path: &Path, project: &Project, kinds: &[KindConfig]) -> Result<()> {
    let global_require = project.rules.grounding.require;
    for kind in kinds {
        let Some(grounding) = project.rules.grounding.kinds.get(&kind.kind) else {
            continue;
        };
        // §FS-config.3.4.7.4: nothing in an unwalked home is read, so the rule
        // could never fire — the reasoning that already refuses a
        // `[citations.<kind>]` rule on an unwalked citing kind.
        if kind.require_grounding == Some(true)
            && !kind.scan
            && let Some(source) = &grounding.require_source
        {
            bail_config(
                path,
                source.line,
                format!(
                    "kind `{}` sets `require_grounding = true` and `scan = false` (no file in an unwalked home is read, so the rule could never fire)",
                    kind.kind
                ),
            )?;
        }
        // §FS-config.3.4.8.5: a citable single-file kind is one declaration document,
        // which §FS-check.3.6.1.1 leaves alone — as `index` means nothing on a file kind.
        // A non-citable `file` home is governed like any other, so there they mean.
        if kind.file.is_some()
            && kind.citable
            && let Some((key, line)) = grounding_key_site(grounding)
        {
            bail_config(
                path,
                line,
                format!(
                    "kind `{}` sets `{key}` with `file` on a citable kind (a citable single-file kind is one declaration document, which the grounding rule leaves alone — a non-citable one takes both keys)",
                    kind.kind
                ),
            )?;
        }
        // §FS-config.3.4.8.5: a level on a row whose *effective* `require_grounding` is
        // off is a unit for a rule that never runs there — the row spelling of the
        // `[reference]` rejection below, and what keeps §AR-scanner.2.7 off such a tree.
        if let Some(source) = &grounding.level_source
            && !kind.require_grounding.unwrap_or(global_require)
        {
            let cause = if kind.require_grounding == Some(false) {
                "`require_grounding = false` (the level could never fire)".to_string()
            } else {
                "nothing turns grounding on for it (set `require_grounding = true` here or in [reference])".to_string()
            };
            bail_config(
                path,
                source.line,
                format!("kind `{}` sets `grounding_level` and {cause}", kind.kind),
            )?;
        }
    }
    Ok(())
}

/// Which of the two keys a row wrote first, for the citable-`file` rejection
/// above — by line, so the message names the key the reader can go and delete.
fn grounding_key_site(grounding: &KindGrounding) -> Option<(&'static str, usize)> {
    let require = grounding
        .require_source
        .as_ref()
        .map(|source| ("require_grounding", source.line));
    let level = grounding
        .level_source
        .as_ref()
        .map(|source| ("grounding_level", source.line));
    match (require, level) {
        (Some(a), Some(b)) if b.1 < a.1 => Some(b),
        (Some(a), _) => Some(a),
        (None, other) => other,
    }
}

/// The rules over the whole kind table (§FS-config.3.4): one complement, unique
/// names, and no citable name a prefix of another's.
fn validate_kind_table(path: &Path, kinds: &[KindConfig]) -> Result<()> {
    // §FS-config.3.9.2.1: the homeless kind is the complement of every configured home,
    // and a complement is one place. Two rows claiming it would leave the fallback
    // with no single answer, so the second is refused rather than resolved by order.
    let homeless: Vec<&KindConfig> = kinds
        .iter()
        .filter(|kind| !kind.citable && kind.folder.is_none() && kind.file.is_none())
        .collect();
    if let [first, second, ..] = homeless.as_slice() {
        return Err(anyhow!(
            "{}: kinds `{}` and `{}` both declare the homeless kind (no `folder`, no `file`) — there is one complement of every home",
            format_path(path),
            first.kind,
            second.kind
        ));
    }
    // §FS-config.3.4: names are unique across the whole table — `[citations.*]`
    // and `grund list --kind` key on one, so two rows wearing one name is a
    // config with no answer to "which".
    for (i, a) in kinds.iter().enumerate() {
        if kinds[..i].iter().any(|b| b.kind == a.kind) {
            return Err(anyhow!(
                "{}: kind `{}` is declared twice",
                format_path(path),
                a.kind
            ));
        }
    }
    // Reject kinds whose name is a prefix of another kind's name (§FS-config.3.4 —
    // tokenization would be ambiguous). Scoped to *citable* kinds: `DAT-foo` parses as
    // `DA` or `DAT`, and a name that never appears in an ID never tokenizes.
    for (i, a) in kinds.iter().enumerate() {
        for (j, b) in kinds.iter().enumerate() {
            if i != j
                && a.citable
                && b.citable
                && a.kind.len() <= b.kind.len()
                && b.kind.starts_with(a.kind.as_str())
            {
                return Err(anyhow!(
                    "{}: kinds `{}` and `{}` collide (one is a prefix of the other)",
                    format_path(path),
                    a.kind,
                    b.kind
                ));
            }
        }
    }
    Ok(())
}

/// §FS-config.3.4.8.5: `[reference] grounding_level` with the global boolean off
/// and no row turning grounding on is a unit for a rule nothing switched on —
/// the row-scoped rejection above, one scope up. Asked of the final table,
/// because "no row turns it on" is a question about the resolved rows.
fn validate_global_grounding(path: &Path, project: &Project, kinds: &[KindConfig]) -> Result<()> {
    let grounding = &project.rules.grounding;
    let Some(source) = &grounding.level_source else {
        return Ok(());
    };
    if grounding.require
        || kinds
            .iter()
            .any(|kind| kind.require_grounding == Some(true))
    {
        return Ok(());
    }
    bail_config(
        path,
        source.line,
        "[reference] `grounding_level` is set and nothing turns grounding on (set `require_grounding` here or on a [[kinds]] row)".to_string(),
    )
}
