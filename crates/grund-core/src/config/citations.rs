//! The `[citations]` half of the config (§FS-config.3.9): the parsed direction
//! rules — levels, namespace matchers and cited targets — and the reader and
//! whole-section validator that produce them.
//!
//! Beside `kind_table.rs` and `grounding.rs` for the same reason they are their
//! own files: `[citations]` is one section of `grund.toml` with a grammar of its
//! own (a disjunction of `[alias/]KIND` targets per RFC-2119 level) and
//! cross-key rules only the finalized kind set can settle
//! (§AR-core-module-layout.1). The records lived in `model/records.rs` while
//! config was a file-name category; they are config's own now (§AR-system.2.3).

use anyhow::{Result, anyhow};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use super::parse::{bail_config, parse_string, parse_string_list};
use super::record::{Config, citing_kind_names};
use super::workspace_block::{INVALID_ALIAS_PATH_EXPECTED, invalid_alias_path_segment};
use crate::model::format_path;

/// One RFC-2119 level a `[citations]` rule entry can carry (§FS-config.3.9.1,
/// §DF-citation-directions.2.1).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CitationLevel {
    Must,
    Should,
    May,
    ShouldNot,
    MustNot,
}

/// How a rule entry's namespace qualifier matches a citation's namespace
/// (§FS-config.3.9.3): bare `KIND` is local-only, `alias/KIND` pins one member,
/// `*/KIND` matches any namespace — rule grammar only, never a citation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NamespaceMatch {
    Local,
    Alias(String),
    Any,
}

/// One cited target in a rule entry: a namespace qualifier plus a kind prefix.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CitationTarget {
    pub namespace: NamespaceMatch,
    pub kind: String,
}

/// One `[citations]` array entry — a disjunction of targets joined by `|`
/// (§FS-config.3.9.1.1). Satisfied by a citation matching any one target.
#[derive(Clone, Debug)]
pub struct CitationDisjunction {
    pub targets: Vec<CitationTarget>,
}

/// The direction rules for one citing kind (§FS-config.3.9). `must` / `should`
/// are obligations checked per declaration; `should_not` / `must_not` are
/// prohibitions checked per citation site; `may` is an explicit permission that
/// punches a hole in a stricter `default`.
#[derive(Clone, Debug, Default)]
pub struct KindCitationRules {
    pub default: Option<CitationLevel>,
    pub must: Vec<CitationDisjunction>,
    pub should: Vec<CitationDisjunction>,
    pub may: Vec<CitationDisjunction>,
    pub should_not: Vec<CitationDisjunction>,
    pub must_not: Vec<CitationDisjunction>,
}

/// The parsed `[citations]` section (§FS-config.3.9): the global default level
/// and the per-citing-kind rule tables. `declared` records whether the section
/// was present at all — absent means no direction checks run.
#[derive(Clone, Debug, Default)]
pub struct CitationRules {
    pub declared: bool,
    pub global_default: Option<CitationLevel>,
    pub per_kind: BTreeMap<String, KindCitationRules>,
}
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
    // §FS-config.3.9: the kind is the last segment, so a nested member is pinned
    // by its whole alias path (`group/api/AR`) exactly as it is cited
    // (§FS-workspace.6.1).
    let (namespace, kind) = match token.rsplit_once('/') {
        Some((qualifier, kind)) => {
            let namespace = if qualifier == "*" {
                NamespaceMatch::Any
            } else {
                // §FS-config.3.9.3.1: config diagnostics name the citation target's
                // qualifier and kind, while the CLI keeps its own `<alias>/<ID>`
                // vocabulary. Both surfaces use the same segment validation.
                if let Some(message) = invalid_citation_target_message(token, qualifier, kind) {
                    bail_config(path, line_no, message)?;
                }
                NamespaceMatch::Alias(qualifier.to_string())
            };
            (namespace, kind)
        }
        None => (NamespaceMatch::Local, token),
    };
    if kind.is_empty() {
        bail_config(
            path,
            line_no,
            format!("citation target `{token}` names no kind"),
        )?;
    }
    Ok(CitationTarget {
        namespace,
        kind: kind.to_string(),
    })
}

/// Render the `[citations]` form of an invalid namespace qualifier
/// (§FS-config.3.9.3.1). This is intentionally separate from the CLI alias-path
/// message: the final segment here is a citation kind, not an ID.
fn invalid_citation_target_message(token: &str, qualifier: &str, kind: &str) -> Option<String> {
    let bad = invalid_alias_path_segment(qualifier)?;
    let detail = if qualifier.is_empty() {
        format!("namespace qualifier before kind `{kind}` is empty")
    } else if bad.is_empty() {
        format!("invalid namespace qualifier segment (empty) in `{qualifier}` before kind `{kind}`")
    } else {
        format!("invalid namespace qualifier segment `{bad}` in `{qualifier}` before kind `{kind}`")
    };
    let wildcard = if bad == "*" {
        "; `*` may only be the whole qualifier"
    } else {
        ""
    };
    Some(format!(
        "citation target `{token}`: {detail} ({INVALID_ALIAS_PATH_EXPECTED}){wildcard}"
    ))
}

/// Validate the parsed `[citations]` rules against the finalized kind set
/// (§FS-config.3.9.5): every citing kind is a configured kind or `code`, every
/// target names a *citable* configured kind, and no two targets of the same
/// cited kind whose namespace matchers overlap sit at different levels.
pub(super) fn validate_citation_rules(path: &Path, config: &Config) -> Result<()> {
    // The citing side is any name in the table plus `code` — a non-citable kind
    // cites like any other place (§FS-config.3.9). The cited side is narrower:
    // only a citable kind has IDs to be the target of a citation.
    let citing_known: BTreeSet<&str> = citing_kind_names(&config.kinds).into_iter().collect();
    let known: BTreeSet<&str> = config
        .kinds
        .iter()
        .filter(|k| k.citable)
        .map(|k| k.kind.as_str())
        .collect();
    for (citing, rules) in &config.citations.per_kind {
        if !citing_known.contains(citing.as_str()) {
            return Err(anyhow!(
                "{}: [citations.{citing}] names an unknown kind `{citing}`",
                format_path(path)
            ));
        }
        // §FS-config.3.4.7.6: a rule on a kind whose home is not walked could
        // never fire — the vacuous pass §DF-non-citable-kinds.2.5 refused, one
        // level up — so the config is refused where it makes the promise.
        if config.kinds.iter().any(|k| k.kind == *citing && !k.scan) {
            return Err(anyhow!(
                "{}: [citations.{citing}] names an unwalked kind `{citing}` (its home is `scan = false`, so no file in it is checked and the rule could never fire)",
                format_path(path)
            ));
        }
        // Flatten every target with the level it was declared at, rejecting any
        // that names an unconfigured kind on the way.
        let mut targets: Vec<(&'static str, &CitationTarget)> = Vec::new();
        let levels: [(&'static str, &[CitationDisjunction]); 5] = [
            ("must", &rules.must),
            ("should", &rules.should),
            ("may", &rules.may),
            ("should-not", &rules.should_not),
            ("must-not", &rules.must_not),
        ];
        for (level_name, disjunctions) in levels {
            for disjunction in disjunctions {
                for target in &disjunction.targets {
                    if !known.contains(target.kind.as_str()) {
                        // A non-citable kind is a name the table knows and a
                        // citation can never carry, so say which of the two it
                        // is rather than calling a configured kind unknown.
                        let why = if citing_known.contains(target.kind.as_str()) {
                            "a non-citable target kind"
                        } else {
                            "an unknown target kind"
                        };
                        return Err(anyhow!(
                            "{}: [citations.{citing}] {level_name} names {why} `{}`",
                            format_path(path),
                            target.kind
                        ));
                    }
                    targets.push((level_name, target));
                }
            }
        }
        // Two targets of one kind whose matchers can match the same citation (e.g.
        // bare `AR` and `*/AR`) must not sit at different levels — such a citation
        // would have no single level (§FS-config.3.9.5.1). Identical entries are fine.
        for (index, (level_a, a)) in targets.iter().enumerate() {
            for (level_b, b) in targets.iter().skip(index + 1) {
                if level_a != level_b
                    && a.kind == b.kind
                    && namespaces_overlap(&a.namespace, &b.namespace)
                {
                    return Err(anyhow!(
                        "{}: [citations.{citing}] `{}` ({level_a}) and `{}` ({level_b}) overlap (a citation matching both has no single level)",
                        format_path(path),
                        render_citation_target(a),
                        render_citation_target(b)
                    ));
                }
            }
        }
    }
    Ok(())
}

/// Whether two rule-target namespace matchers can match the same citation
/// (§FS-config.3.9.3): `*/` (any namespace) overlaps every qualifier; otherwise
/// two matchers overlap only when identical — both local, or the same pinned
/// alias. A local matcher and a pinned-alias matcher are disjoint, so permitting
/// a local kind while forbidding one member's same kind is allowed.
fn namespaces_overlap(a: &NamespaceMatch, b: &NamespaceMatch) -> bool {
    match (a, b) {
        (NamespaceMatch::Any, _) | (_, NamespaceMatch::Any) => true,
        (NamespaceMatch::Local, NamespaceMatch::Local) => true,
        (NamespaceMatch::Alias(left), NamespaceMatch::Alias(right)) => left == right,
        _ => false,
    }
}

/// The namespace qualifier as written in config — for round-tripping in
/// `grund config show` and for the duplicate-target message (§FS-config.3.9).
fn citation_namespace_label(namespace: &NamespaceMatch) -> String {
    match namespace {
        NamespaceMatch::Local => String::new(),
        NamespaceMatch::Any => "*/".to_string(),
        NamespaceMatch::Alias(alias) => format!("{alias}/"),
    }
}

pub(crate) fn render_citation_target(target: &CitationTarget) -> String {
    format!(
        "{}{}",
        citation_namespace_label(&target.namespace),
        target.kind
    )
}
