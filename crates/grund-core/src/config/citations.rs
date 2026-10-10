//! The `[citations]` half of the config (§FS-config.3.9): the parsed direction
//! rules — levels, namespace matchers and cited targets — the target grammar
//! the rules engine shares, and the whole-section validator `validate.rs` runs
//! on the lowered project (§AR-config.4). The v1 spelling of the two tables is
//! read in `v1/citations.rs`.
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

use super::kind::KindConfig;
use super::record::citing_kind_names;
use super::workspace_block::{INVALID_ALIAS_PATH_EXPECTED, invalid_alias_path_segment};
use crate::model::format_path;

/// One RFC-2119 level a `[citations]` rule entry can carry (§FS-config.3.9.1,
/// §DF-citation-directions.2.1). `Warn` and `WarnNot` are v2's: the `must`
/// and `must-not` findings on the warning channel (§FS-config-v2.rules.strengths),
/// which the v1 reader never produces.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CitationLevel {
    Must,
    Warn,
    Should,
    May,
    ShouldNot,
    WarnNot,
    MustNot,
}

impl CitationLevel {
    /// The key the level is written as, in either version (§FS-config.3.9.1,
    /// §FS-config-v2.rules.citations).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Must => "must",
            Self::Warn => "warn",
            Self::Should => "should",
            Self::May => "may",
            Self::ShouldNot => "should-not",
            Self::WarnNot => "warn-not",
            Self::MustNot => "must-not",
        }
    }
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
#[derive(Clone, Debug, PartialEq)]
pub struct CitationDisjunction {
    pub targets: Vec<CitationTarget>,
}

/// The direction rules for one citing kind (§FS-config.3.9). `must` / `should`
/// are obligations checked per declaration; `should_not` / `must_not` are
/// prohibitions checked per citation site; `may` is an explicit permission that
/// punches a hole in a stricter `default`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct KindCitationRules {
    pub default: Option<CitationLevel>,
    pub must: Vec<CitationDisjunction>,
    /// v2's `warn` list (§FS-config-v2.rules.citations); empty in v1.
    pub warn: Vec<CitationDisjunction>,
    pub should: Vec<CitationDisjunction>,
    pub may: Vec<CitationDisjunction>,
    pub should_not: Vec<CitationDisjunction>,
    /// v2's `warn-not` list (§FS-config-v2.rules.citations); empty in v1.
    pub warn_not: Vec<CitationDisjunction>,
    pub must_not: Vec<CitationDisjunction>,
}

impl KindCitationRules {
    /// Every list with its level, strongest obligation first — the order a
    /// citation site is matched in (§FS-config.3.9.4).
    pub fn lists(&self) -> [(CitationLevel, &[CitationDisjunction]); 7] {
        [
            (CitationLevel::Must, &self.must),
            (CitationLevel::Warn, &self.warn),
            (CitationLevel::Should, &self.should),
            (CitationLevel::May, &self.may),
            (CitationLevel::ShouldNot, &self.should_not),
            (CitationLevel::WarnNot, &self.warn_not),
            (CitationLevel::MustNot, &self.must_not),
        ]
    }
}

/// The parsed `[citations]` section (§FS-config.3.9): the global default level
/// and the per-citing-kind rule tables. `declared` records whether the section
/// was present at all — absent means no direction checks run.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CitationRules {
    pub declared: bool,
    pub global_default: Option<CitationLevel>,
    pub per_kind: BTreeMap<String, KindCitationRules>,
}
/// Parse the released namespace-aware target-entry grammar without attaching a
/// config-file location. Chapter-rule object targets reuse this exact lexical
/// boundary (§FS-rules.2, §FS-config.3.9.3.1).
pub(crate) fn parse_citation_target_entry(token: &str) -> Result<CitationTarget, String> {
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
                    return Err(message);
                }
                NamespaceMatch::Alias(qualifier.to_string())
            };
            (namespace, kind)
        }
        None => (NamespaceMatch::Local, token),
    };
    if kind.is_empty() {
        return Err(format!("citation target `{token}` names no kind"));
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
/// `table` is the spelling the messages name the rules by: `citations` in v1,
/// `rules.citations` in v2 (§FS-config-v2.rules.citations).
pub(super) fn validate_citation_rules(
    path: &Path,
    table: &str,
    kinds: &[KindConfig],
    citations: &CitationRules,
) -> Result<()> {
    // The citing side is any name in the table plus `code` — a non-citable kind
    // cites like any other place (§FS-config.3.9). The cited side is narrower:
    // only a citable kind has IDs to be the target of a citation.
    let citing_known: BTreeSet<&str> = citing_kind_names(kinds).into_iter().collect();
    let known: BTreeSet<&str> = kinds
        .iter()
        .filter(|k| k.citable)
        .map(|k| k.kind.as_str())
        .collect();
    for (citing, rules) in &citations.per_kind {
        if !citing_known.contains(citing.as_str()) {
            return Err(anyhow!(
                "{}: [{table}.{citing}] names an unknown kind `{citing}`",
                format_path(path)
            ));
        }
        // §FS-config.3.4.7.6: a rule on a kind whose home is not walked could
        // never fire — the vacuous pass §DF-non-citable-kinds.2.5 refused, one
        // level up — so the config is refused where it makes the promise.
        if kinds.iter().any(|k| k.kind == *citing && !k.scan) {
            return Err(anyhow!(
                "{}: [{table}.{citing}] names an unwalked kind `{citing}` (its home is `scan = false`, so no file in it is checked and the rule could never fire)",
                format_path(path)
            ));
        }
        // Flatten every target with the level it was declared at, rejecting any
        // that names an unconfigured kind on the way.
        let mut targets: Vec<(&'static str, &CitationTarget)> = Vec::new();
        for (level, disjunctions) in rules.lists() {
            let level_name = level.as_str();
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
                            "{}: [{table}.{citing}] {level_name} names {why} `{}`",
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
                        "{}: [{table}.{citing}] `{}` ({level_a}) and `{}` ({level_b}) overlap (a citation matching both has no single level)",
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
pub(super) fn namespaces_overlap(a: &NamespaceMatch, b: &NamespaceMatch) -> bool {
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
