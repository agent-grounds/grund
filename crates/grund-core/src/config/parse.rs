//! The `grund.toml` reader (§FS-config.3): one line-oriented pass over the file
//! that fills a `Config`, the scalar value parsers every section shares, and the
//! `[workspace]` member and alias validation.
//!
//! The reader rather than the record or the discovery: `discovery.rs` says which
//! file governs a directory and `record.rs` says what a filled `Config` means,
//! while the three sections with a grammar of their own — `[[kinds]]`,
//! `[citations]` and the grounding pair — read their own keys in
//! `kind_table.rs`, `citations.rs` and `grounding.rs` (§AR-core-module-layout.1).

use anyhow::{Context, Result, anyhow};
use std::fs;
use std::path::Path;

use super::citations::{parse_citation_entry, validate_citation_rules};
use super::fmt_block::validate_fmt_exclude;
use super::grounding::{
    check_grounding_level, parse_kind_grounding_key, validate_global_grounding,
};
use super::kind_table::{ParsedKind, apply_parsed_kinds, parse_kinds_key};
use super::point_sizes::parse_lead_size_warning;
use super::record::{Config, ConfigLocation, ShorthandPolicy};
use super::workspace_block::validate_workspace_lists;
use crate::grammar::{id_grammar_key_slash_error, is_escaped};
use crate::model::format_path;

/// Parse one `grund.toml` over `config` — the schema of §FS-config.3 and its
/// subsections (`[reference]` 3.1, `[id]` 3.2/3.3, `[[kinds]]` 3.4, `[scan]` 3.5,
/// `[output]` 3.6, `[fmt.cross_refs]` 3.7, `[fmt]` 3.10). Any unknown section/key or malformed
/// value is a hard error reported as `path:line:` (§FS-config.4.3, §FS-errors.2.1).
pub(super) fn parse_config_file(
    read_path: &Path,
    report_path: &Path,
    config: &mut Config,
) -> Result<()> {
    let text = fs::read_to_string(read_path)
        .with_context(|| format!("read {}", format_path(report_path)))?;
    // Everything below reports problems against the stable relative path.
    let path = report_path;
    let mut section = String::new();
    let mut grammar_dirty = false;
    let mut parsed_kinds: Vec<ParsedKind> = Vec::new();
    let mut current_kind: Option<ParsedKind> = None;
    let mut kinds_block_seen = false;
    let mut inline_note_suggested_lines_source = None;
    // §FS-config.3.4.8: where `[reference] grounding_level` was written, so the
    // "nothing turns grounding on" rejection anchors at the key rather than at
    // line 1. Asked once the `[[kinds]]` table is final.
    let mut grounding_level_source = None;
    let mut inline_note_max_lines_source = None;
    for (idx, raw_line) in text.lines().enumerate() {
        let line_no = idx + 1;
        let line = strip_comment(raw_line).trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            let is_array_table = line.starts_with("[[") && line.ends_with("]]");
            section = line.trim_matches(['[', ']']).to_string();
            match section.as_str() {
                "reference" | "scan" | "output" | "id" | "fmt" | "fmt.cross_refs" | "workspace" => {
                    if section == "workspace" && is_array_table {
                        bail_config(path, line_no, "expected `[workspace]` (table)".to_string())?;
                    }
                    if section == "workspace" {
                        config.workspace_declared = true;
                        // §FS-workspace.6.1: the block's own anchor, for the
                        // errors that are about the block and not about a key.
                        config.workspace_section_source = Some(ConfigLocation {
                            path: path.to_path_buf(),
                            line: line_no,
                        });
                    }
                }
                "kinds" => {
                    if !is_array_table {
                        bail_config(
                            path,
                            line_no,
                            "expected `[[kinds]]` (array of tables)".to_string(),
                        )?;
                    }
                    // Flush any open kind entry, then start a new one.
                    if let Some(kind) = current_kind.take() {
                        parsed_kinds.push(kind);
                    }
                    current_kind = Some(ParsedKind::new(line_no));
                    kinds_block_seen = true;
                }
                // §FS-config.3.9: `[citations]` and per-kind `[citations.<KIND>]`
                // tables. Both are plain tables, never arrays of tables.
                other if other == "citations" || other.starts_with("citations.") => {
                    if is_array_table {
                        bail_config(
                            path,
                            line_no,
                            "expected `[citations]` / `[citations.<KIND>]` (table)".to_string(),
                        )?;
                    }
                    config.citations.declared = true;
                    if let Some(kind) = other.strip_prefix("citations.") {
                        config
                            .citations
                            .per_kind
                            .entry(kind.to_string())
                            .or_default();
                    }
                }
                other => bail_config(path, line_no, format!("unknown config section `{other}`"))?,
            }
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            bail_config(path, line_no, "expected `key = value`".to_string())?;
            unreachable!();
        };
        let key = key.trim();
        let value = value.trim();
        match (section.as_str(), key) {
            ("", "grund_config_version") => {
                if value != "1" {
                    bail_config(
                        path,
                        line_no,
                        format!(
                            "unsupported config version `{value}` \
                             (this grund understands grund_config_version = 1; \
                             upgrade grund if the config is newer)"
                        ),
                    )?;
                }
            }
            ("", "project_name") => {
                config.project_name = Some(parse_string(path, line_no, value)?);
                config.project_name_source = Some(ConfigLocation {
                    path: path.to_path_buf(),
                    line: line_no,
                });
            }
            ("", "project_description") => {
                let description = parse_string(path, line_no, value)?;
                // §FS-config.3: the key feeds single-line workspace member
                // bullets, so an embedded line break is a config error.
                if description.contains('\n') || description.contains('\r') {
                    bail_config(
                        path,
                        line_no,
                        "project_description must be a single line".to_string(),
                    )?;
                }
                config.project_description = Some(description);
            }
            ("reference", "marker") => config.marker = parse_string(path, line_no, value)?,
            ("reference", "trigger") => config.trigger = parse_string(path, line_no, value)?,
            ("reference", "strict") => config.strict = parse_bool(path, line_no, value)?,
            // §FS-config.3.1: closed persisted-form policy. Parsing the string
            // first keeps non-string failures on the ordinary located path.
            ("reference", "shorthand") => {
                let policy = parse_string(path, line_no, value)?;
                config.shorthand = match policy.as_str() {
                    "canonical" => ShorthandPolicy::Canonical,
                    "accepted" => ShorthandPolicy::Accepted,
                    _ => bail_config(
                        path,
                        line_no,
                        format!(
                            "unknown [reference] shorthand `{policy}` (expected canonical or accepted)"
                        ),
                    )?,
                };
            }
            ("reference", "require_grounding") => {
                config.require_grounding = parse_bool(path, line_no, value)?
            }
            // §FS-config.3.4.8: the default unit inside every governed file; the
            // key's own rules live in `grounding.rs` with its row twin.
            ("reference", "grounding_level") => {
                config.grounding_level = parse_usize(path, line_no, value)?;
                check_grounding_level(path, line_no, config.grounding_level)?;
                grounding_level_source = Some(line_no);
            }
            ("reference", "conversation") => {
                // §FS-config.3.1, §DF-repo-conversation-opinion.2.2: closed enum with the
                // single member "link" — `plain` encodes machine state and stays user-scoped.
                let opinion = parse_string(path, line_no, value)?;
                if opinion != "link" {
                    bail_config(
                        path,
                        line_no,
                        format!("unknown [reference] conversation `{opinion}` (expected link)"),
                    )?;
                }
                config.conversation = Some(opinion);
            }
            // §FS-config.3.1: one closed inline table opts this project into the
            // fixed-warning lead budget. Absence is the complete off switch.
            ("reference", "lead_size_warning") => {
                config.lead_size_warning = Some(parse_lead_size_warning(path, line_no, value)?);
            }
            ("reference", "inline_style") => {
                let style = parse_string(path, line_no, value)?;
                if !matches!(style.as_str(), "citation-with-note" | "citation-only") {
                    bail_config(
                        path,
                        line_no,
                        "unknown [reference] inline_style".to_string(),
                    )?;
                }
                config.inline_style = style;
            }
            ("reference", "inline_note_suggested_lines") => {
                config.inline_note_suggested_lines = parse_usize(path, line_no, value)?;
                inline_note_suggested_lines_source = Some(line_no);
            }
            ("reference", "inline_note_max_lines") => {
                config.inline_note_max_lines = parse_usize(path, line_no, value)?;
                inline_note_max_lines_source = Some(line_no);
            }
            ("reference", "inline_note_max_columns") => {
                config.inline_note_max_columns = parse_usize(path, line_no, value)?
            }
            // §FS-inline-citation-style.2.2: two closed enums, rejected on load like
            // `inline_style` above — a typo must not read as "no house style".
            ("reference", "inline_note_layout") => {
                let layout = parse_string(path, line_no, value)?;
                if !matches!(layout.as_str(), "any" | "citation-first-colon") {
                    bail_config(
                        path,
                        line_no,
                        format!(
                            "unknown [reference] inline_note_layout `{layout}` (expected any or citation-first-colon)"
                        ),
                    )?;
                }
                config.inline_note_layout = layout;
            }
            ("reference", "inline_note_layout_check") => {
                let level = parse_string(path, line_no, value)?;
                if !matches!(level.as_str(), "off" | "warn" | "error") {
                    bail_config(
                        path,
                        line_no,
                        format!(
                            "unknown [reference] inline_note_layout_check `{level}` (expected off, warn, or error)"
                        ),
                    )?;
                }
                config.inline_note_layout_check = level;
            }
            ("reference", "warn_on_suggested") => {
                config.warn_on_suggested = parse_bool(path, line_no, value)?
            }
            // §FS-config.3.2: the keys an ID is built from share one rule — no `/` —
            // so they share one arm and `id_grammar_rules.rs` answers per key; checked
            // here in the config-error style (§FS-errors.2.1), `Grammar::build` backstops.
            ("id", key @ ("format" | "section_separator" | "number_pattern" | "slug_pattern")) => {
                let parsed = parse_string(path, line_no, value)?;
                if let Some(message) = id_grammar_key_slash_error(key, &parsed) {
                    bail_config(path, line_no, message)?;
                }
                match key {
                    "format" => config.id_format = parsed,
                    "section_separator" => config.section_separator = parsed,
                    "number_pattern" => config.number_pattern = parsed,
                    _ => config.slug_pattern = parsed,
                }
                grammar_dirty = true;
            }
            // §FS-config.3.2: named coordinates are an explicit, absent-by-default
            // grammar change, so parsing the key recompiles every shared pattern.
            ("id", "named_sections") => {
                config.named_sections = parse_bool(path, line_no, value)?;
                grammar_dirty = true;
            }
            ("id", "section_heading_levels") => {
                let mode = parse_string(path, line_no, value)?;
                if !matches!(mode.as_str(), "strict" | "warn" | "loose") {
                    bail_config(
                        path,
                        line_no,
                        format!(
                            "unknown [id] section_heading_levels `{mode}` (expected strict, warn, or loose)"
                        ),
                    )?;
                }
                config.section_heading_levels = mode;
            }
            // §FS-config.3.4.8: the two grounding keys are `grounding.rs`'s
            // on both sides — the row and the `[reference]` default — so the
            // section walk hands them there rather than through the row reader.
            ("kinds", key @ ("require_grounding" | "grounding_level")) => {
                parse_kind_grounding_key(path, line_no, key, value, &mut current_kind)?;
            }
            // §FS-config.3.4: the `[[kinds]]` keys, in `kind_table.rs`
            // (§AR-core-module-layout.1).
            ("kinds", key) => {
                if !parse_kinds_key(path, line_no, key, value, &mut current_kind)? {
                    bail_config(path, line_no, format!("unknown config key `{key}`"))?;
                }
            }
            ("scan", "include") => config.include = Some(parse_string_list(path, line_no, value)?),
            ("scan", "exclude") => config.exclude = parse_string_list(path, line_no, value)?,
            ("scan", "extensions") => config.extensions = parse_string_list(path, line_no, value)?,
            ("scan", "comment_prefixes") => {
                config.comment_prefixes = parse_string_list(path, line_no, value)?;
                grammar_dirty = true;
            }
            ("scan", "docstring_python") => {
                config.docstring_python = parse_bool(path, line_no, value)?;
            }
            ("scan", "respect_gitignore") => {
                config.respect_gitignore = parse_bool(path, line_no, value)?;
            }
            ("output", "format") => {
                let format = parse_string(path, line_no, value)?;
                if !matches!(format.as_str(), "text" | "json") {
                    bail_config(path, line_no, "unsupported output format".to_string())?;
                }
                config.output_format = format;
            }
            ("output", "color") => {
                // Reserved — colored output is not yet implemented (§FS-config.6,
                // §FS-errors.3): the value is inert today but still validated against
                // the documented set, so a typo fails on load instead of being ignored.
                let color = parse_string(path, line_no, value)?;
                if !matches!(color.as_str(), "auto" | "always" | "never") {
                    bail_config(
                        path,
                        line_no,
                        format!(
                            "unknown [output] color `{color}` (expected auto, always, or never)"
                        ),
                    )?;
                }
            }
            ("output", "relative_paths") => {
                config.relative_paths = parse_bool(path, line_no, value)?;
            }
            // §FS-config.3.10: validated as it is parsed, so a malformed glob is a
            // config error at its own line rather than a surprise at the first
            // `grund fmt` (§FS-config.4.3).
            ("fmt", "exclude") => {
                let patterns = parse_string_list(path, line_no, value)?;
                if let Err(message) = validate_fmt_exclude(&patterns) {
                    bail_config(path, line_no, message)?;
                }
                config.fmt_exclude = patterns;
            }
            ("fmt.cross_refs", "enabled") => {
                config.fmt_cross_refs_enabled = parse_bool(path, line_no, value)?;
            }
            ("fmt.cross_refs", "anchor_format") => {
                let format = parse_string(path, line_no, value)?;
                if !matches!(
                    format.as_str(),
                    "github" | "gitlab" | "mkdocs" | "pandoc" | "none"
                ) {
                    bail_config(path, line_no, "unknown md link anchor format".to_string())?;
                }
                config.cross_ref_anchor_format = format;
            }
            ("workspace", "members") => {
                config.workspace_members = parse_string_list(path, line_no, value)?;
                config.workspace_members_source = Some(ConfigLocation {
                    path: path.to_path_buf(),
                    line: line_no,
                });
            }
            // §FS-config.3.8, §FS-workspace.2.2: the sibling list, read by the same
            // parser as `members` — which is what keeps `grund_config_version` at 1
            // and a binary older than the key refusing it rather than ignoring it.
            ("workspace", "optional_members") => {
                config.workspace_optional_members = parse_string_list(path, line_no, value)?;
                config.workspace_optional_members_source = Some(ConfigLocation {
                    path: path.to_path_buf(),
                    line: line_no,
                });
            }
            ("workspace", "include_root") => {
                config.workspace_include_root = parse_bool(path, line_no, value)?;
                // §FS-check.4.10: the key that decides is the line to open, so it
                // is located like `members` above rather than left to the block's
                // own `[workspace]` header.
                config.workspace_include_root_source = Some(ConfigLocation {
                    path: path.to_path_buf(),
                    line: line_no,
                });
            }
            // §FS-config.3.9: `[citations]` `default`, and the level keys of each
            // `[citations.<KIND>]` table.
            (s, k) if s == "citations" || s.starts_with("citations.") => {
                parse_citation_entry(path, line_no, s, k, value, &mut config.citations)?;
            }
            _ => bail_config(path, line_no, format!("unknown config key `{key}`"))?,
        }
    }
    if let Some(kind) = current_kind.take() {
        parsed_kinds.push(kind);
    }
    if config.strict && config.marker.is_empty() {
        return Err(anyhow!(
            "{}: reference.strict requires a non-empty marker",
            format_path(path)
        ));
    }
    if config.inline_note_suggested_lines > config.inline_note_max_lines {
        let line = inline_note_suggested_lines_source
            .or(inline_note_max_lines_source)
            .unwrap_or(1);
        bail_config(
            path,
            line,
            "reference.inline_note_suggested_lines must be <= inline_note_max_lines".to_string(),
        )?;
    }
    if kinds_block_seen {
        apply_parsed_kinds(path, parsed_kinds, config)?;
    }
    // §FS-config.3.4.8: both keys resolve per row against these defaults, so the
    // cross-section rule and the derived scanner flag are asked once the kind
    // table is final — the built-in table included.
    validate_global_grounding(path, config, grounding_level_source)?;
    config.recompute_grounding_units();
    if grammar_dirty || kinds_block_seen {
        config
            .rebuild_grammar()
            .with_context(|| format!("{}: invalid [id] grammar", format_path(path)))?;
    }
    // §AR-workspace.5.2: post-parse invariants run on every load, not gated on which
    // section appeared. Free-form `project_name` is slug-checked later
    // (§AR-workspace.5.3); both member lists are shape-checked in `workspace_block.rs`.
    validate_workspace_lists(config)?;
    // §FS-config.3.9.5: validate `[citations]` after the kind set is final.
    if config.citations.declared {
        validate_citation_rules(path, config)?;
    }
    Ok(())
}

/// Drop a trailing `#`-comment from a `grund.toml` line (§FS-config.3).
pub(crate) fn strip_comment(line: &str) -> &str {
    // A `#` inside a quoted string is not a comment marker. Walk the line and stop at the
    // first unquoted `#`; otherwise return the line unchanged.
    let bytes = line.as_bytes();
    let mut in_string = false;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' if !is_escaped(bytes, i) => in_string = !in_string,
            b'#' if !in_string => return &line[..i],
            _ => {}
        }
        i += 1;
    }
    line
}

/// Fail config parsing with a `path:line: message` error — the located-finding
/// shape applied to a malformed `grund.toml` (§FS-config.4.3, §FS-errors.2.1).
pub(super) fn bail_config<T>(path: &Path, line: usize, message: String) -> Result<T> {
    Err(anyhow!("{}:{}: {}", format_path(path), line, message))
}

pub(super) fn parse_string(path: &Path, line: usize, value: &str) -> Result<String> {
    if !(value.starts_with('"') && value.ends_with('"') && value.len() >= 2) {
        return bail_config(path, line, "expected string".to_string());
    }
    let inner = &value[1..value.len() - 1];
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('\\') => out.push('\\'),
            Some('"') => out.push('"'),
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some(other) => {
                return bail_config(
                    path,
                    line,
                    format!("invalid escape sequence `\\{other}` in string"),
                );
            }
            None => {
                return bail_config(path, line, "trailing backslash in string".to_string());
            }
        }
    }
    Ok(out)
}

pub(super) fn parse_bool(path: &Path, line: usize, value: &str) -> Result<bool> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => bail_config(path, line, "expected boolean".to_string()),
    }
}

pub(super) fn parse_usize(path: &Path, line: usize, value: &str) -> Result<usize> {
    value.parse::<usize>().map_err(|_| {
        anyhow!(
            "{}:{}: expected non-negative integer",
            format_path(path),
            line
        )
    })
}

pub(crate) fn parse_string_list(path: &Path, line: usize, value: &str) -> Result<Vec<String>> {
    if !value.starts_with('[') || !value.ends_with(']') {
        return bail_config(path, line, "expected string list".to_string());
    }
    let inner = value[1..value.len() - 1].trim();
    if inner.is_empty() {
        return Ok(Vec::new());
    }
    inner
        .split(',')
        .map(|part| parse_string(path, line, part.trim()))
        .collect()
}
