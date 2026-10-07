//! The v1 spelling of `[[kinds]]` (§FS-config.3.4), beside `grounding.rs` and
//! `citations.rs` (§AR-core-module-layout.1): the per-key reader that fills one
//! `[[kinds]]` entry. The section walk stays in `parse.rs`; what becomes of the
//! entries once the file is read — the refusals of an entry no row can hold, and
//! the rows the rest lower into — is `kind_rows.rs` (§AR-config.3.1); and the
//! rules over the whole lowered table — unique names, one complement,
//! prefix-free citable names, the grounding pair — are `config/validate.rs`'s
//! (§AR-config.4).

use anyhow::Result;
use std::path::{Component, Path};

use super::grounding::ParsedGrounding;
use super::parse::{bail_config, parse_bool, parse_string};
use crate::config::kind::{KindConfig, KindIndex, KindResolution};
use crate::grammar::id_grammar_literal_slash_error;
use crate::model::named_section_component;

/// One `[[kinds]]` entry as the parser has it so far: the entry itself, the line
/// its `[[kinds]]` header sat on (what an entry-level error anchors at), and
/// whether the entry has already named its kind — the one thing "sets `kind`
/// twice" needs to know (§FS-config.3.4).
pub(super) struct ParsedKind {
    pub(super) config: KindConfig,
    pub(super) header_line: usize,
    named: bool,
    /// The row's `require_grounding` / `grounding_level`, each with the line it
    /// was written on (§FS-config.3.4.8) — read by `grounding.rs`, which
    /// owns both keys and every rule about them.
    pub(super) grounding: ParsedGrounding,
    pub(super) values_line: Option<usize>,
    rules_line: Option<usize>,
    /// The line `value_chapter` was written on, which its own refusals anchor
    /// at (§FS-config.3.4.13).
    pub(super) value_chapter_line: Option<usize>,
}

impl ParsedKind {
    pub(super) fn new(header_line: usize) -> Self {
        Self {
            config: KindConfig {
                kind: String::new(),
                folder: None,
                file: None,
                title: None,
                index: KindIndex::Default,
                citable: true,
                scan: true,
                require_grounding: None,
                grounding_level: None,
                values: false,
                value_chapter: None,
                rules: false,
                format: None,
                resolve: None,
                fetch: None,
            },
            header_line,
            named: false,
            grounding: ParsedGrounding::default(),
            values_line: None,
            rules_line: None,
            value_chapter_line: None,
        }
    }
}

/// Read one `key = value` line inside a `[[kinds]]` block into `slot`
/// (§FS-config.3.4). Returns `false` for a key this section does not define, so
/// the caller reports it as an unknown config key (§FS-config.4.3).
pub(super) fn parse_kinds_key(
    path: &Path,
    line_no: usize,
    key: &str,
    value: &str,
    current_kind: &mut Option<ParsedKind>,
) -> Result<bool> {
    match key {
        // §FS-config.3.4.6: `prefix` stopped loading in 0.13.0. Refused at its own
        // line ahead of any name bookkeeping, so the anchor follows the key rather
        // than the order a row that also sets `kind` happens to spell the two in.
        "prefix" => {
            return bail_config(
                path,
                line_no,
                "[[kinds]] `prefix` was removed in grund 0.13.0 — rename it to `kind`".to_string(),
            );
        }
        // §FS-config.3.4: `kind` is the name every entry declares.
        "kind" => {
            let name = parse_string(path, line_no, value)?;
            // §FS-config.3.2.3: a citable kind's name is the leading component of
            // every ID in it, so a `/` here lands in the ID as surely as one in
            // `slug_pattern` does.
            if let Some(message) =
                id_grammar_literal_slash_error(&format!("[[kinds]] {key} `{name}`"), &name)
            {
                bail_config(path, line_no, message)?;
            }
            let Some(slot) = current_kind.as_mut() else {
                bail_config(path, line_no, format!("`{key}` outside of [[kinds]] block"))?;
                unreachable!();
            };
            if slot.named {
                bail_config(path, line_no, format!("[[kinds]] sets `{key}` twice"))?;
            }
            slot.named = true;
            slot.config.kind = name;
        }
        // §FS-config.3.4: `citable = false` makes the entry a place rather than
        // an ID namespace — scanned and directed, never declared in.
        "citable" => {
            let citable = parse_bool(path, line_no, value)?;
            if let Some(slot) = current_kind.as_mut() {
                slot.config.citable = citable;
            } else {
                bail_config(
                    path,
                    line_no,
                    "`citable` outside of [[kinds]] block".to_string(),
                )?;
            }
        }
        // §FS-config.3.4.7: `scan = false` lists the place without walking it —
        // for content that ships verbatim and is nothing of this repository's
        // to check. What it may combine with is validated in `apply_parsed_kinds`.
        "scan" => {
            let scan = parse_bool(path, line_no, value)?;
            if let Some(slot) = current_kind.as_mut() {
                slot.config.scan = scan;
            } else {
                bail_config(
                    path,
                    line_no,
                    "`scan` outside of [[kinds]] block".to_string(),
                )?;
            }
        }
        // §FS-config.3.4.9 / §FS-values.1: the sole value-related key is an
        // absent-by-default boolean on the owning kind row.
        "values" => {
            let values = parse_bool(path, line_no, value)?;
            let Some(slot) = current_kind.as_mut() else {
                bail_config(
                    path,
                    line_no,
                    "`values` outside of [[kinds]] block".to_string(),
                )?;
                unreachable!();
            };
            if slot.values_line.replace(line_no).is_some() {
                bail_config(path, line_no, "[[kinds]] sets `values` twice".to_string())?;
            }
            slot.config.values = values;
        }
        // §FS-config.3.4.13 / §FS-values.2.5: name the chapter whose named
        // children are this kind's value roots. The handle grammar is checked
        // here; every relationship it needs is checked in `apply_parsed_kinds`.
        "value_chapter" => {
            let chapter = parse_string(path, line_no, value)?;
            let Some(slot) = current_kind.as_mut() else {
                bail_config(
                    path,
                    line_no,
                    "`value_chapter` outside of [[kinds]] block".to_string(),
                )?;
                unreachable!();
            };
            if slot.value_chapter_line.replace(line_no).is_some() {
                bail_config(
                    path,
                    line_no,
                    "[[kinds]] sets `value_chapter` twice".to_string(),
                )?;
            }
            // §FS-config.3.2.7: the value is a section handle, so an empty
            // string is the key's own absence rather than a nameless chapter.
            if chapter.is_empty() {
                slot.config.value_chapter = None;
            } else {
                if !named_section_component(&chapter) {
                    bail_config(
                        path,
                        line_no,
                        format!(
                            "[[kinds]] `value_chapter` must be a section handle matching `[a-z][a-z0-9-]*` (`{chapter}` is not)"
                        ),
                    )?;
                }
                slot.config.value_chapter = Some(chapter);
            }
        }
        // §FS-config.3.4.12 / §FS-rules.1: opt a citable Markdown kind into
        // declaration-title rules. Parsing the titles remains a post-scan job.
        "rules" => {
            let rules = parse_bool(path, line_no, value)?;
            let Some(slot) = current_kind.as_mut() else {
                bail_config(
                    path,
                    line_no,
                    "`rules` outside of [[kinds]] block".to_string(),
                )?;
                unreachable!();
            };
            if slot.rules_line.replace(line_no).is_some() {
                bail_config(path, line_no, "[[kinds]] sets `rules` twice".to_string())?;
            }
            slot.config.rules = rules;
        }
        "folder" => {
            let folder = parse_string(path, line_no, value)?;
            if let Some(slot) = current_kind.as_mut() {
                slot.config.folder = Some(folder);
            } else {
                bail_config(
                    path,
                    line_no,
                    "`folder` outside of [[kinds]] block".to_string(),
                )?;
            }
        }
        "file" => {
            let file = parse_string(path, line_no, value)?;
            if let Some(slot) = current_kind.as_mut() {
                slot.config.file = Some(file);
            } else {
                bail_config(
                    path,
                    line_no,
                    "`file` outside of [[kinds]] block".to_string(),
                )?;
            }
        }
        // §FS-config.3.4: `index` names the file under `folder` that must list the
        // folder's declarations (§FS-check.3.18). A file name or `false`; `true` names
        // no file, so it is rejected — the key's own absence already spells "default".
        "index" => {
            let index = if value == "false" {
                KindIndex::Disabled
            } else if value == "true" {
                bail_config(
                    path,
                    line_no,
                    "[[kinds]] `index` takes a file name or `false` (omit the key for the default `README.md`)"
                        .to_string(),
                )?;
                unreachable!();
            } else {
                let name = parse_string(path, line_no, value)?;
                if let Some(message) = kind_index_name_error(&name) {
                    bail_config(path, line_no, message)?;
                }
                KindIndex::Named(name)
            };
            if let Some(slot) = current_kind.as_mut() {
                slot.config.index = index;
            } else {
                bail_config(
                    path,
                    line_no,
                    "`index` outside of [[kinds]] block".to_string(),
                )?;
            }
        }
        "title" => {
            let title = parse_string(path, line_no, value)?;
            if let Some(slot) = current_kind.as_mut() {
                slot.config.title = Some(title);
            } else {
                bail_config(
                    path,
                    line_no,
                    "`title` outside of [[kinds]] block".to_string(),
                )?;
            }
        }
        // §FS-config.3.4.10: snapshot kinds may override their ID shape and
        // configure one explicit materializer plus its fixed obligation.
        "format" => {
            let format = parse_string(path, line_no, value)?;
            let Some(slot) = current_kind.as_mut() else {
                bail_config(
                    path,
                    line_no,
                    "`format` outside of [[kinds]] block".to_string(),
                )?;
                unreachable!();
            };
            slot.config.format = Some(format);
        }
        "resolve" => {
            let value = parse_string(path, line_no, value)?;
            let resolution = match value.as_str() {
                "must" => KindResolution::Must,
                "should" => KindResolution::Should,
                _ => bail_config(
                    path,
                    line_no,
                    "[[kinds]] `resolve` must be must or should".to_string(),
                )?,
            };
            let Some(slot) = current_kind.as_mut() else {
                bail_config(
                    path,
                    line_no,
                    "`resolve` outside of [[kinds]] block".to_string(),
                )?;
                unreachable!();
            };
            slot.config.resolve = Some(resolution);
        }
        "fetch" => {
            let fetch = parse_string(path, line_no, value)?;
            let Some(slot) = current_kind.as_mut() else {
                bail_config(
                    path,
                    line_no,
                    "`fetch` outside of [[kinds]] block".to_string(),
                )?;
                unreachable!();
            };
            slot.config.fetch = Some(fetch);
        }
        _ => return Ok(false),
    }
    Ok(true)
}

/// Why `index = "<name>"` is not a usable index file name, or `None`
/// (§FS-config.3.4). Two rules, each closing a state the rules built on this key
/// cannot describe:
///
/// * **It names a file inside `folder`.** The value is joined onto `folder`, and
///   an absolute path or one that climbs out with `..` silently replaces the
///   folder instead of naming a file in it — `grund check` would then read, and
///   `grund fmt --write` would then rewrite, a file outside the tree the config
///   describes (§FS-non-goals.11). `.` is rejected with them: it names the same
///   file by a path no message should have to print.
/// * **It names a Markdown file.** `--cross-refs` runs on `.md` files only
///   (§FS-fmt.6.1), so an index with any other extension is one the formatter can
///   never linkify — and §FS-check.3.17.3, whose whole licence is that
///   `grund fmt --write` fixes it, would be an error no command could clear
///   (§DF-index-entry-form.2.3).
fn kind_index_name_error(name: &str) -> Option<String> {
    if name.is_empty() {
        return Some("[[kinds]] `index` must name a file (use `false` to opt out)".to_string());
    }
    let inside_folder = !name.contains('\\')
        && Path::new(name)
            .components()
            .all(|component| matches!(component, Component::Normal(_)));
    if !inside_folder {
        return Some(format!(
            "[[kinds]] `index` must be a relative path inside `folder` (`{name}` is not)"
        ));
    }
    if Path::new(name).extension().and_then(|ext| ext.to_str()) != Some("md") {
        return Some(format!(
            "[[kinds]] `index` must name a Markdown file (`{name}` is not a `.md` file)"
        ));
    }
    None
}
