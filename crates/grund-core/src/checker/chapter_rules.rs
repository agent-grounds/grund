//! Checker orchestration for chapter rules (§FS-rules.4, §FS-rules.11,
//! §AR-checker.1). Grammar, facts, evaluation, and deduplication stay owned by
//! the rules component; this module only sequences and merges their results.

use crate::config::{Config, NamespaceMatch};
use crate::grammar::render_id;
use crate::model::{CheckReport, Diagnostic, Findings};
use crate::resolver::WorkspaceCheckTarget;
use crate::rules::RuleAnchor;
use crate::rules::engine::{evaluate, subject_resolves};
use crate::rules::markdown::{adapt_markdown, adapt_workspace};
use crate::rules::sentence::{
    Cardinality, ParsedRule, RuleLevel, RulePolarity, RuleRelation, RuleSubject, RuleTargets,
    RuleVocabulary, TargetMode, parse_rule,
};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn vocabulary(config: &Config) -> RuleVocabulary {
    let kinds = config
        .kinds
        .iter()
        .filter(|kind| kind.citable)
        .map(|kind| kind.kind.clone())
        .collect::<BTreeSet<_>>();
    RuleVocabulary {
        kinds: kinds.clone(),
        target_kinds: kinds,
        named_sections: config.named_sections,
    }
}

pub(crate) fn parse_ad_hoc(config: &Config, sentence: &str) -> anyhow::Result<ParsedRule> {
    parse_rule(
        sentence,
        "--rule".into(),
        RuleAnchor {
            path: String::new(),
            line: 0,
            column: None,
        },
        &vocabulary(config),
    )
    .map_err(|error| anyhow::anyhow!(error.message))
}

/// Validate configured declarations for `init` and return their exact authored
/// sentences in qualified-rule-ID order (§FS-rules.4, §FS-rules.9).
pub(crate) fn configured_rule_sentences(
    findings: &Findings,
    config: &Config,
) -> anyhow::Result<Vec<(String, String)>> {
    let vocab = vocabulary(config);
    let facts = adapt_markdown(findings, config, true);
    let rule_kinds = config
        .kinds
        .iter()
        .filter(|kind| kind.rules)
        .map(|kind| kind.kind.as_str())
        .collect::<BTreeSet<_>>();
    let mut rows = Vec::new();
    for (id, declarations) in &findings.declarations {
        if !rule_kinds.contains(id.kind.as_str()) {
            continue;
        }
        for declaration in declarations {
            let origin = render_id(&config.grammar, id);
            let title = declaration.title.as_deref().ok_or_else(|| {
                anyhow::anyhow!("{origin} is not a valid rule: rule declaration has no title")
            })?;
            if declaration.body_end <= declaration.line {
                return Err(anyhow::anyhow!(
                    "{origin} is not a valid rule: rule rationale is empty"
                ));
            }
            let parsed = parse_rule(
                title,
                origin.clone(),
                RuleAnchor {
                    path: declaration.file.to_string_lossy().into_owned(),
                    line: declaration.line,
                    column: None,
                },
                &vocab,
            )
            .map_err(|error| anyhow::anyhow!("{origin} is not a valid rule: {}", error.message))?;
            if !subject_resolves(&parsed, &facts) {
                let literal = match &parsed.subject {
                    RuleSubject::ExactDeclaration(value) => value.clone(),
                    RuleSubject::ExactChapter { declaration, path } => {
                        format!("{declaration}.{path}")
                    }
                    _ => unreachable!(),
                };
                return Err(anyhow::anyhow!(
                    "{origin} is not a valid rule: literal subject {literal} does not resolve"
                ));
            }
            rows.push((origin, title.to_string()));
        }
    }
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(rows)
}

/// Append configured and optional ad-hoc rule results to the shared report.
/// An incomplete scan passes an explicitly incomplete snapshot to the engine.
pub(crate) fn check_chapter_rules(
    findings: &Findings,
    config: &Config,
    complete: bool,
    ad_hoc: Option<ParsedRule>,
    workspace: Option<(&str, &BTreeMap<String, WorkspaceCheckTarget<'_>>)>,
    report: &mut CheckReport,
) {
    if !config.kinds.iter().any(|kind| kind.rules) && ad_hoc.is_none() {
        return;
    }
    let mut vocab = vocabulary(config);
    if let Some((_, projects)) = workspace {
        vocab.target_kinds.extend(
            projects
                .values()
                .flat_map(|project| project.config.kinds.iter())
                .filter(|kind| kind.citable)
                .map(|kind| kind.kind.clone()),
        );
    }
    let mut rules = Vec::new();
    let rule_kinds = config
        .kinds
        .iter()
        .filter(|kind| kind.rules)
        .map(|kind| kind.kind.as_str())
        .collect::<BTreeSet<_>>();
    for (id, declarations) in &findings.declarations {
        if !rule_kinds.contains(id.kind.as_str()) {
            continue;
        }
        for declaration in declarations {
            let origin = render_id(&config.grammar, id);
            let anchor = RuleAnchor {
                path: declaration.file.to_string_lossy().into_owned(),
                line: declaration.line,
                column: None,
            };
            let parsed = declaration
                .title
                .as_deref()
                .ok_or_else(|| "rule declaration has no title".to_string())
                .and_then(|title| {
                    parse_rule(title, origin.clone(), anchor, &vocab).map_err(|error| error.message)
                });
            match parsed {
                Ok(rule) if declaration.body_end > declaration.line => rules.push(rule),
                Ok(_) => report.errors.push(invalid_rule(
                    &origin,
                    declaration.file.to_string_lossy().as_ref(),
                    declaration.line,
                    "rule rationale is empty",
                )),
                Err(message) => report.errors.push(invalid_rule(
                    &origin,
                    declaration.file.to_string_lossy().as_ref(),
                    declaration.line,
                    &message,
                )),
            }
        }
    }
    if let Some(rule) = ad_hoc {
        rules.push(rule);
    }
    let facts = match workspace {
        Some((selected, projects)) => adapt_workspace(selected, projects, complete),
        None => adapt_markdown(findings, config, complete),
    };
    rules.retain(|rule| {
        if subject_resolves(rule, &facts) {
            return true;
        }
        let literal = match &rule.subject {
            RuleSubject::ExactDeclaration(value) => value.clone(),
            RuleSubject::ExactChapter { declaration, path } => format!("{declaration}.{path}"),
            _ => return true,
        };
        let (path, line) = if rule.origin == "--rule" {
            (None, None)
        } else {
            (
                Some(rule.anchor.path.clone().into()),
                Some(rule.anchor.line),
            )
        };
        report.errors.push(Diagnostic {
            code: "invalid-rule",
            path,
            line,
            column: rule.anchor.column,
            message: format!(
                "{} is not a valid rule: literal subject {literal} does not resolve",
                rule.origin
            ),
            sites: Vec::new(),
        });
        false
    });
    rules.retain(|rule| !duplicates_config(rule, config));
    for result in evaluate(&rules, &facts) {
        if result.recommended {
            report.suggestions.push(result.diagnostic);
        } else {
            report.errors.push(result.diagnostic);
        }
    }
}

fn invalid_rule(origin: &str, path: &str, line: usize, message: &str) -> Diagnostic {
    Diagnostic {
        code: "invalid-rule",
        path: Some(path.into()),
        line: Some(line),
        column: None,
        message: format!("{origin} is not a valid rule: {message}"),
        sites: Vec::new(),
    }
}

/// Preserve existing citation-direction bytes when its bare-kind constraint is
/// semantically identical to a rule (§FS-rules.6).
fn duplicates_config(rule: &ParsedRule, config: &Config) -> bool {
    let (
        RuleSubject::Kind(subject),
        RuleTargets::Kinds {
            values,
            mode: TargetMode::Aggregate,
        },
    ) = (&rule.subject, &rule.targets)
    else {
        return false;
    };
    if rule.relation != RuleRelation::Cite {
        return false;
    }
    let expected_cardinality = match rule.polarity {
        RulePolarity::Positive => Cardinality::AT_LEAST_ONE,
        RulePolarity::Prohibiting => Cardinality::NONE,
    };
    if rule.cardinality != expected_cardinality {
        return false;
    }
    let Some(directions) = config.citations.per_kind.get(subject) else {
        return false;
    };
    let entries = match (rule.level, rule.polarity) {
        (RuleLevel::Required, RulePolarity::Positive) => &directions.must,
        (RuleLevel::Recommended, RulePolarity::Positive) => &directions.should,
        (RuleLevel::Required, RulePolarity::Prohibiting) => &directions.must_not,
        (RuleLevel::Recommended, RulePolarity::Prohibiting) => &directions.should_not,
    };
    entries.iter().any(|entry| {
        let mut configured = entry
            .targets
            .iter()
            .map(|target| match &target.namespace {
                NamespaceMatch::Local => target.kind.clone(),
                NamespaceMatch::Alias(alias) => format!("{alias}/{}", target.kind),
                NamespaceMatch::Any => format!("*/{}", target.kind),
            })
            .collect::<Vec<_>>();
        configured.sort();
        configured.dedup();
        &configured == values
    })
}
