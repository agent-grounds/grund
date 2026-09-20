//! Checker orchestration for chapter rules (§FS-rules.4, §FS-rules.11,
//! §AR-checker.1). Grammar, facts, evaluation, and deduplication stay owned by
//! the rules component; this module only sequences and merges their results.

use crate::config::Config;
use crate::grammar::render_id;
use crate::model::{CheckReport, Declaration, Diagnostic, Findings};
use crate::resolver::WorkspaceCheckTarget;
use crate::rules::RuleAnchor;
use crate::rules::engine::{
    citation_precedence, evaluate, evaluate_suggestions, unresolved_subject_diagnostic,
};
use crate::rules::markdown::{adapt_markdown, adapt_workspace};
use crate::rules::sentence::{ParsedRule, RuleVocabulary, parse_rule};
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
        target_namespaces: BTreeMap::new(),
        named_sections: config.named_sections,
        id_grammars: vec![config.grammar.clone()],
        section_separators: vec![config.section_separator.clone()],
    }
}

pub(crate) fn parse_ad_hoc(config: &Config, sentence: &str) -> anyhow::Result<ParsedRule> {
    parse_ad_hoc_with_vocabulary(sentence, vocabulary(config))
}

pub(crate) fn parse_ad_hoc_with_workspace(
    config: &Config,
    sentence: &str,
    projects: &BTreeMap<String, WorkspaceCheckTarget<'_>>,
) -> anyhow::Result<ParsedRule> {
    let mut vocab = vocabulary(config);
    add_workspace_targets(&mut vocab, projects);
    parse_ad_hoc_with_vocabulary(sentence, vocab)
}

fn parse_ad_hoc_with_vocabulary(
    sentence: &str,
    vocabulary: RuleVocabulary,
) -> anyhow::Result<ParsedRule> {
    parse_rule(
        sentence,
        "--rule".into(),
        RuleAnchor {
            path: String::new(),
            line: 0,
            column: None,
        },
        &vocabulary,
    )
    .map_err(|error| anyhow::anyhow!(error.message))
}

fn add_workspace_targets(
    vocabulary: &mut RuleVocabulary,
    projects: &BTreeMap<String, WorkspaceCheckTarget<'_>>,
) {
    vocabulary
        .target_namespaces
        .extend(projects.iter().map(|(alias, project)| {
            let kinds = project
                .config
                .kinds
                .iter()
                .filter(|kind| kind.citable)
                .map(|kind| kind.kind.clone())
                .collect();
            (alias.clone(), kinds)
        }));
}

/// Validate configured declarations for `init` and return their exact authored
/// sentences in qualified-rule-ID order (§FS-rules.4, §FS-rules.9).
pub(crate) fn configured_rule_sentences(
    findings: &Findings,
    config: &Config,
) -> Result<Vec<(String, String)>, Diagnostic> {
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
            let path = declaration.file.to_string_lossy();
            let title = declaration.title.as_deref().ok_or_else(|| {
                invalid_rule(
                    &origin,
                    path.as_ref(),
                    declaration.line,
                    "rule declaration has no title",
                )
            })?;
            if !rule_has_rationale(declaration) {
                return Err(invalid_rule(
                    &origin,
                    path.as_ref(),
                    declaration.line,
                    "rule rationale is empty",
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
            .map_err(|error| {
                invalid_rule(&origin, path.as_ref(), declaration.line, &error.message)
            })?;
            if let Some(diagnostic) = unresolved_subject_diagnostic(&parsed, &facts) {
                return Err(diagnostic);
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
        add_workspace_targets(&mut vocab, projects);
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
                Ok(rule) if rule_has_rationale(declaration) => rules.push(rule),
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
    let precedence = citation_precedence(config);
    report.errors.extend(evaluate(&rules, &precedence, &facts));
    report
        .suggestions
        .extend(evaluate_suggestions(&rules, &precedence, &facts));
}

/// A rule rationale contains authored non-whitespace text after its declaration
/// heading. Both `check` and `init` use this one predicate so a blank body can
/// never render or execute on one surface only (§FS-rules.1, §FS-rules.4).
pub(crate) fn rule_has_rationale(declaration: &Declaration) -> bool {
    declaration.body_has_content
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
