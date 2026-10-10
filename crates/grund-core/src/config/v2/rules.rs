//! The rules concern's keys (§FS-config-v2.rules): `[rules.citations]` and each
//! kind's lists, the grounding ladders, and `[rules.resolution]`. Each lowers
//! into the record v1's spelling fills, with `warn` and `warn-not` as the
//! warning channel beside them (§FS-config-v2.rules.strengths).

use anyhow::Result;

use super::walk::Reader;
use crate::config::citations::{
    CitationDisjunction, CitationLevel, namespaces_overlap, parse_citation_target_entry,
    render_citation_target,
};
use crate::config::kind::KindResolution;
use crate::config::project::{KindGrounding, Rung, Strength};
use crate::config::v1::{parse_string, parse_string_list};

/// `[rules.citations]`: the project's `default` alone.
pub(super) fn citations_key(r: &mut Reader, key: &str, value: &str, line: usize) -> Result<bool> {
    if key != "default" {
        return Ok(false);
    }
    let default = parse_default(r, value, line)?;
    r.project.rules.citations.global_default = Some(default);
    Ok(true)
}

/// `[rules.citations.<KIND>]`: the seven lists and the kind's `default`. Each
/// list is kept with its line, for the overlap the table's close refuses.
pub(super) fn citation_kind_key(
    r: &mut Reader,
    kind: &str,
    lists: &mut Vec<(CitationLevel, usize)>,
    key: &str,
    value: &str,
    line: usize,
) -> Result<bool> {
    let level = match key {
        "default" => {
            let default = parse_default(r, value, line)?;
            r.project
                .rules
                .citations
                .per_kind
                .entry(kind.to_string())
                .or_default()
                .default = Some(default);
            return Ok(true);
        }
        "must" => CitationLevel::Must,
        "warn" => CitationLevel::Warn,
        "should" => CitationLevel::Should,
        "may" => CitationLevel::May,
        "should-not" => CitationLevel::ShouldNot,
        "warn-not" => CitationLevel::WarnNot,
        "must-not" => CitationLevel::MustNot,
        _ => return Ok(false),
    };
    let parsed = parse_disjunctions(r, value, line)?;
    let rules = r
        .project
        .rules
        .citations
        .per_kind
        .entry(kind.to_string())
        .or_default();
    *match level {
        CitationLevel::Must => &mut rules.must,
        CitationLevel::Warn => &mut rules.warn,
        CitationLevel::Should => &mut rules.should,
        CitationLevel::May => &mut rules.may,
        CitationLevel::ShouldNot => &mut rules.should_not,
        CitationLevel::WarnNot => &mut rules.warn_not,
        CitationLevel::MustNot => &mut rules.must_not,
    } = parsed;
    lists.push((level, line));
    Ok(true)
}

/// §FS-config-v2.rules.citations.default: a default may forbid, never oblige.
fn parse_default(r: &Reader, value: &str, line: usize) -> Result<CitationLevel> {
    let level = parse_string(r.path, line, value)?;
    match level.as_str() {
        "may" => Ok(CitationLevel::May),
        "should-not" => Ok(CitationLevel::ShouldNot),
        "warn-not" => Ok(CitationLevel::WarnNot),
        "must-not" => Ok(CitationLevel::MustNot),
        "must" | "warn" | "should" => r.fail(
            line,
            format!(
                "`default = \"{level}\"` is not a v2 citation default (expected may, should-not, warn-not, or must-not)"
            ),
        ),
        _ => r.fail(
            line,
            format!(
                "unknown citation default `{level}` (expected may, should-not, warn-not, or must-not)"
            ),
        ),
    }
}

/// A list of targets, each a target or an any-of `A|B` in v1's target grammar
/// (§FS-config.3.9.3).
fn parse_disjunctions(r: &Reader, value: &str, line: usize) -> Result<Vec<CitationDisjunction>> {
    let mut disjunctions = Vec::new();
    for entry in parse_string_list(r.path, line, value)? {
        let mut targets = Vec::new();
        for token in entry.split('|').map(str::trim) {
            if token.is_empty() {
                return r.fail(line, "empty citation target".to_string());
            }
            match parse_citation_target_entry(token) {
                Ok(target) => targets.push(target),
                Err(message) => return r.fail(line, message),
            }
        }
        disjunctions.push(CitationDisjunction { targets });
    }
    Ok(disjunctions)
}

/// §FS-config-v2.rules.citations: two lists whose targets can match one citation
/// are refused at the later list's line (§FS-config.3.9.5.1).
pub(super) fn close_citation_kind(
    r: &mut Reader,
    kind: &str,
    lists: &[(CitationLevel, usize)],
) -> Result<()> {
    let Some(rules) = r.project.rules.citations.per_kind.get(kind) else {
        return Ok(());
    };
    let list = |level: CitationLevel| {
        rules
            .lists()
            .into_iter()
            .find(|(at, _)| *at == level)
            .map_or(&[][..], |(_, list)| list)
    };
    let targets = |level| list(level).iter().flat_map(|d| d.targets.iter());
    for (index, &(later, line)) in lists.iter().enumerate() {
        for &(earlier, _) in &lists[..index] {
            for a in targets(earlier) {
                if let Some(b) = targets(later)
                    .find(|b| a.kind == b.kind && namespaces_overlap(&a.namespace, &b.namespace))
                {
                    let message = format!(
                        "[rules.citations.{kind}] `{}` ({}) and `{}` ({}) overlap (a citation matching both has no single level)",
                        render_citation_target(a),
                        earlier.as_str(),
                        render_citation_target(b),
                        later.as_str()
                    );
                    return r.fail(line, message);
                }
            }
        }
    }
    Ok(())
}

/// §FS-config-v2.rules.grounding: one rung, a positive strength and a unit.
pub(super) fn ladder_key(
    r: &mut Reader,
    rungs: &mut Vec<Rung>,
    key: &str,
    value: &str,
    line: usize,
) -> Result<bool> {
    let Some(strength) = Strength::parse(key) else {
        return Ok(false);
    };
    let unit = parse_string(r.path, line, value)?;
    let level = match unit.as_str() {
        "file" => 1,
        "h2" => 2,
        "h3" => 3,
        "h4" => 4,
        "h5" => 5,
        "h6" => 6,
        _ => {
            return r.fail(
                line,
                format!("unknown grounding unit `{unit}` (expected file, or h2 to h6)"),
            );
        }
    };
    rungs.push(Rung {
        strength,
        level,
        source: r.at(line),
    });
    Ok(true)
}

/// The unit a rung's level spells.
pub(super) fn unit(level: usize) -> String {
    if level <= 1 {
        "file".to_string()
    } else {
        format!("h{level}")
    }
}

/// §FS-config-v2.rules.grounding: order a ladder, then lower it whole. Its
/// `must` rung is v1's effective pair, so everything that reads the pair keeps
/// reading it; a row's ladder replaces the project's, inheriting no rung.
pub(super) fn close_ladder(
    r: &mut Reader,
    place: Option<String>,
    mut rungs: Vec<Rung>,
) -> Result<()> {
    rungs.sort_by_key(|rung| rung.strength as u8);
    for (index, stronger) in rungs.iter().enumerate() {
        if let Some(weaker) = rungs[index + 1..].iter().find(|w| stronger.level > w.level) {
            let line = stronger.source.as_ref().map_or(1, |source| source.line);
            let message = format!(
                "grounding ladder: `{}` ({}) is finer than `{}` ({}); a stronger rung never uses a finer unit",
                stronger.strength.as_str(),
                unit(stronger.level),
                weaker.strength.as_str(),
                unit(weaker.level)
            );
            return r.fail(line, message);
        }
    }
    let must = rungs.iter().find(|rung| rung.strength == Strength::Must);
    let (require, level) = (must.is_some(), must.map_or(1, |rung| rung.level));
    let grounding = &mut r.project.rules.grounding;
    match place {
        None => {
            grounding.require = require;
            grounding.level = level;
            grounding.ladder = Some(rungs);
        }
        Some(place) => {
            let row = KindGrounding {
                require: Some(require),
                require_source: None,
                level: Some(level),
                level_source: None,
                ladder: Some(rungs),
            };
            grounding.kinds.insert(place, row);
        }
    }
    Ok(())
}

/// §FS-config-v2.rules.resolution: a kind's missing-snapshot strength, `must`
/// or v1's `should` warning spelled `warn`. Whether the kind fetches is a
/// question about its row, asked once every row is read.
pub(super) fn resolution_key(r: &mut Reader, key: &str, value: &str, line: usize) -> Result<bool> {
    let strength = parse_string(r.path, line, value)?;
    let resolution = match strength.as_str() {
        "must" => KindResolution::Must,
        "warn" => KindResolution::Should,
        "should" | "may" => {
            return r.fail(
                line,
                format!(
                    "`[rules.resolution] {key}` takes must or warn (a missing snapshot is never a suggestion)"
                ),
            );
        }
        _ => {
            return r.fail(
                line,
                format!("unknown [rules.resolution] strength `{strength}` (expected must or warn)"),
            );
        }
    };
    r.resolution.push((key.to_string(), resolution, line));
    Ok(true)
}
