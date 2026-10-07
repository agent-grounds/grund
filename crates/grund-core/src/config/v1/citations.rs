//! The v1 spelling of `[citations]` and `[citations.<KIND>]` (§FS-config.3.9):
//! one key at a time, into the `CitationRules` the rules concern holds
//! (§AR-config.1.2). What the rules mean against the lowered kind set is
//! judged in `config/validate.rs` (§AR-config.4); the target grammar this reads
//! each entry with is shared with the rules engine and stays in
//! `config/citations.rs`.

use anyhow::{Result, anyhow};
use std::path::Path;

use super::parse::{bail_config, parse_string, parse_string_list};
use crate::config::citations::{
    CitationDisjunction, CitationLevel, CitationRules, CitationTarget, parse_citation_target_entry,
};
use crate::model::format_path;

/// Parse one `[citations]` / `[citations.<KIND>]` key (§FS-config.3.9). The
/// top-level table takes only `default`; a per-kind table takes `default` plus
/// the five level lists.
pub(super) fn parse_citation_entry(
    path: &Path,
    line_no: usize,
    section: &str,
    key: &str,
    value: &str,
    citations: &mut CitationRules,
) -> Result<()> {
    if section == "citations" {
        return match key {
            "default" => {
                citations.global_default = Some(parse_citation_level(path, line_no, value)?);
                Ok(())
            }
            other => bail_config(
                path,
                line_no,
                format!(
                    "unknown key `{other}` in [citations] (expected `default`, or a [citations.<KIND>] table)"
                ),
            ),
        };
    }
    let kind = section
        .strip_prefix("citations.")
        .expect("caller guarantees a citations. section");
    let rules = citations.per_kind.entry(kind.to_string()).or_default();
    match key {
        "default" => rules.default = Some(parse_citation_level(path, line_no, value)?),
        "must" => rules.must = parse_citation_disjunctions(path, line_no, value)?,
        "should" => rules.should = parse_citation_disjunctions(path, line_no, value)?,
        "may" => rules.may = parse_citation_disjunctions(path, line_no, value)?,
        "should-not" => rules.should_not = parse_citation_disjunctions(path, line_no, value)?,
        "must-not" => rules.must_not = parse_citation_disjunctions(path, line_no, value)?,
        other => bail_config(
            path,
            line_no,
            format!(
                "unknown key `{other}` in [citations.{kind}] (expected must, should, may, should-not, must-not, or default)"
            ),
        )?,
    }
    Ok(())
}

fn parse_citation_level(path: &Path, line_no: usize, value: &str) -> Result<CitationLevel> {
    let level = parse_string(path, line_no, value)?;
    match level.as_str() {
        "must" => Ok(CitationLevel::Must),
        "should" => Ok(CitationLevel::Should),
        "may" => Ok(CitationLevel::May),
        "should-not" => Ok(CitationLevel::ShouldNot),
        "must-not" => Ok(CitationLevel::MustNot),
        other => bail_config(
            path,
            line_no,
            format!(
                "unknown citation level `{other}` (expected must, should, may, should-not, or must-not)"
            ),
        ),
    }
}

fn parse_citation_disjunctions(
    path: &Path,
    line_no: usize,
    value: &str,
) -> Result<Vec<CitationDisjunction>> {
    parse_string_list(path, line_no, value)?
        .iter()
        .map(|entry| parse_citation_disjunction(path, line_no, entry))
        .collect()
}

fn parse_citation_disjunction(
    path: &Path,
    line_no: usize,
    entry: &str,
) -> Result<CitationDisjunction> {
    let mut targets = Vec::new();
    for token in entry.split('|') {
        let token = token.trim();
        if token.is_empty() {
            bail_config(path, line_no, "empty citation target".to_string())?;
        }
        targets.push(parse_citation_target(path, line_no, token)?);
    }
    Ok(CitationDisjunction { targets })
}

fn parse_citation_target(path: &Path, line_no: usize, token: &str) -> Result<CitationTarget> {
    parse_citation_target_entry(token)
        .map_err(|message| anyhow!("{}:{line_no}: {message}", format_path(path)))
}
