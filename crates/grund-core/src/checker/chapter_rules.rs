//! Checker orchestration for chapter rules (§FS-rules.4, §FS-rules.11,
//! §AR-checker.1). Grammar, facts, evaluation, and deduplication stay owned by
//! the rules component; this module only sequences and merges their results.

use crate::config::{Form, Frame, Rules, Schema, known_kinds_line};
use crate::grammar::render_id;
use crate::model::{Catalog, CheckReport, Declaration, Diagnostic};
use crate::resolver::WorkspaceCheckTarget;
use crate::rules::RuleAnchor;
use crate::rules::engine::{
    citation_precedence, evaluate, evaluate_suggestions, one_rules_authority,
    unresolved_subject_diagnostic,
};
use crate::rules::markdown::{MarkdownProject, adapt_markdown, adapt_workspace};
use crate::rules::sentence::{ParsedRule, RuleVocabulary, parse_rule};
use std::collections::{BTreeMap, BTreeSet};

use super::support::sort_diagnostics;

/// What the rules adapter reads of one project's records: the façade stays out of
/// both components (§AR-config.5, §AR-rules.3). `name` is the scope a standalone
/// adaptation selects; a workspace adaptation selects by alias instead.
pub(crate) fn markdown_project<'a>(
    name: Option<&'a str>,
    target: &'a WorkspaceCheckTarget<'a>,
) -> MarkdownProject<'a> {
    MarkdownProject {
        name,
        grammar: &target.compiled.grammar,
        section_separator: &target.schema.ids.section_separator,
    }
}

/// The names of the kinds a rule can name in `schema`: its citable rows.
fn citable_kinds(schema: &Schema) -> BTreeSet<String> {
    schema.kinds().map(|(name, _)| name.to_string()).collect()
}

/// The names of `schema`'s rule kinds, whose declarations are rules (§FS-rules.1).
fn rule_kinds(schema: &Schema) -> BTreeSet<&str> {
    schema
        .kinds()
        .filter(|(_, kind)| matches!(kind.form, Form::Rule { .. }))
        .map(|(name, _)| name)
        .collect()
}

pub(crate) fn vocabulary(schema: &Schema, frame: Frame<'_>) -> RuleVocabulary {
    let kinds = citable_kinds(schema);
    RuleVocabulary {
        kinds: kinds.clone(),
        target_kinds: kinds,
        target_namespaces: BTreeMap::new(),
        named_sections: schema.ids.named_sections,
        id_grammars: vec![frame.grammar().clone()],
        section_separators: vec![schema.ids.section_separator.clone()],
    }
}

pub(crate) fn parse_ad_hoc(
    schema: &Schema,
    frame: Frame<'_>,
    sentence: &str,
) -> anyhow::Result<ParsedRule> {
    parse_ad_hoc_with_vocabulary(schema, sentence, vocabulary(schema, frame))
}

pub(crate) fn parse_ad_hoc_with_workspace(
    schema: &Schema,
    frame: Frame<'_>,
    sentence: &str,
    projects: &BTreeMap<String, WorkspaceCheckTarget<'_>>,
) -> anyhow::Result<ParsedRule> {
    let mut vocab = vocabulary(schema, frame);
    add_workspace_targets(&mut vocab, projects);
    parse_ad_hoc_with_vocabulary(schema, sentence, vocab)
}

/// `schema` is the one whose kinds the subject is read against, so it is the
/// one whose kinds a refusal that offers no form lists (§FS-rules.3.5.2,
/// §FS-rules.3.5.4.4).
fn parse_ad_hoc_with_vocabulary(
    schema: &Schema,
    sentence: &str,
    vocabulary: RuleVocabulary,
) -> anyhow::Result<ParsedRule> {
    let parsed = parse_rule(
        sentence,
        "--rule".into(),
        RuleAnchor {
            path: String::new(),
            line: 0,
            column: None,
        },
        &vocabulary,
    )
    .map_err(|error| {
        // §FS-rules.3.5.4.4: no form, and a kind is missing, so the kinds follow.
        if error.unrecovered {
            anyhow::anyhow!(
                "{}\n{}",
                error.message,
                known_kinds_line(schema.kinds().map(|(name, _)| name))
            )
        } else {
            anyhow::anyhow!(error.message)
        }
    })?;
    // §FS-errors.3.7: `--rule` has no rule heading to report at, so a sentence
    // this scope cannot verify is refused before the scan like any other the
    // vocabulary check turns down.
    match parsed {
        (_, Some(message)) => Err(anyhow::anyhow!(message)),
        (rule, None) => Ok(rule),
    }
}

fn add_workspace_targets(
    vocabulary: &mut RuleVocabulary,
    projects: &BTreeMap<String, WorkspaceCheckTarget<'_>>,
) {
    add_namespaces(
        vocabulary,
        projects
            .iter()
            .map(|(alias, project)| (alias.clone(), project.schema)),
    );
}

/// One namespace per project the run loaded. Holding any at all is what
/// separates a scope that judged an alias and said no from one that could not
/// judge it (§FS-rules.4.1.1).
fn add_namespaces<'a>(
    vocabulary: &mut RuleVocabulary,
    namespaces: impl IntoIterator<Item = (String, &'a Schema)>,
) {
    vocabulary.target_namespaces.extend(
        namespaces
            .into_iter()
            .map(|(alias, schema)| (alias, citable_kinds(schema))),
    );
}

/// The vocabulary a run that already loaded its workspace resolves rule objects
/// against (§FS-rules.4.1). An empty project map is no workspace at all, so it
/// yields the plain single-project vocabulary and every namespaced object kind
/// in it is unverifiable here.
pub(crate) fn workspace_vocabulary(
    schema: &Schema,
    frame: Frame<'_>,
    projects: &BTreeMap<String, WorkspaceCheckTarget<'_>>,
) -> RuleVocabulary {
    let mut vocab = vocabulary(schema, frame);
    if !projects.is_empty() {
        add_workspace_targets(&mut vocab, projects);
    }
    vocab
}

/// The vocabulary for a command that resolves rules without loading a workspace
/// of its own — `init` (§FS-rules.4.1).
///
/// `declared` is the workspace this project *declares*, by alias, never one
/// climbed to from above: `init` does climb to render `### Workspace members`,
/// but that is teaching and this is judging, and judging off a climbed tree
/// would make the same bytes valid or invalid depending on what happens to sit
/// on disk beside the checkout (§FS-workspace.5.1, §DF-unverifiable-rule-scope).
/// The workspace component reads it (`declared_member_schemas`), stage 1 only and
/// best-effort, so the alias set is `check`'s by construction, and a project
/// that declares `[workspace]` holds at least its own namespace whatever its
/// member list expands to (§FS-rules.4.1.1).
pub(crate) fn declared_workspace_vocabulary(
    schema: &Schema,
    frame: Frame<'_>,
    declared: &[(String, Schema)],
) -> RuleVocabulary {
    let mut vocab = vocabulary(schema, frame);
    add_namespaces(
        &mut vocab,
        declared
            .iter()
            .map(|(alias, schema)| (alias.clone(), schema)),
    );
    vocab
}

/// What one validation of a project's configured rules yields: the bullets to
/// render, and the rules this scope could not verify (§FS-rules.4.1.2).
pub(crate) struct ConfiguredRules {
    /// `(qualified rule ID, authored sentence)` in qualified-rule-ID order, one
    /// per rendered rule — every valid rule and every rule unverifiable here
    /// (§FS-rules.9).
    pub(crate) rows: Vec<(String, String)>,
    /// One located `invalid-rule` per rule this scope cannot verify. `init`
    /// reports these and writes anyway; `check` leaves them to
    /// `check_chapter_rules`, which reports them at the rule's own site.
    pub(crate) unverifiable: Vec<Diagnostic>,
}

/// Validate configured declarations against `vocab` and return their exact
/// authored sentences in qualified-rule-ID order (§FS-rules.4, §FS-rules.9).
///
/// `Err` is the genuinely invalid rule and nothing else: the caller withholds
/// the managed-block write for it and for no other failure (§FS-rules.4.1.2).
/// A rule that is only unverifiable here still earns its row, because the
/// bullet is the authored sentence and one tree must render one block
/// (§FS-rules.9.1).
pub(crate) fn configured_rule_sentences(
    findings: &Catalog,
    schema: &Schema,
    frame: Frame<'_>,
    vocab: &RuleVocabulary,
) -> Result<ConfiguredRules, Diagnostic> {
    let local = WorkspaceCheckTarget::of(findings, schema, frame);
    let facts = adapt_markdown(findings, markdown_project(frame.name, &local), true);
    let rule_kinds = rule_kinds(schema);
    let mut rows = Vec::new();
    let mut unverifiable = Vec::new();
    for (id, declarations) in &findings.declarations {
        if !rule_kinds.contains(id.kind.as_str()) {
            continue;
        }
        for declaration in declarations {
            let origin = render_id(frame.grammar(), id);
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
            let (parsed, unverifiable_here) = parse_rule(
                title,
                origin.clone(),
                RuleAnchor {
                    path: declaration.file.to_string_lossy().into_owned(),
                    line: declaration.line,
                    column: None,
                },
                vocab,
            )
            .map_err(|error| {
                invalid_rule(&origin, path.as_ref(), declaration.line, &error.message)
            })?;
            // §FS-rules.4.1: the object's namespace is the whole of what is
            // unverifiable, so every other question is put either way.
            if let Some(diagnostic) = unresolved_subject_diagnostic(&parsed, &facts) {
                return Err(diagnostic);
            }
            // §FS-rules.4.1.2: reported, rendered, and not a reason to withhold
            // the write.
            if let Some(message) = unverifiable_here {
                unverifiable.push(invalid_rule(
                    &origin,
                    path.as_ref(),
                    declaration.line,
                    &message,
                ));
            }
            rows.push((origin, title.to_string()));
        }
    }
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(ConfiguredRules { rows, unverifiable })
}

/// Append configured and optional ad-hoc rule results to the shared report.
/// An incomplete scan passes an explicitly incomplete snapshot to the engine.
#[allow(clippy::too_many_arguments)]
pub(crate) fn check_chapter_rules(
    findings: &Catalog,
    rules: &Rules,
    schema: &Schema,
    frame: Frame<'_>,
    complete: bool,
    ad_hoc: Option<ParsedRule>,
    workspace: Option<(&str, &BTreeMap<String, WorkspaceCheckTarget<'_>>)>,
    report: &mut CheckReport,
) {
    let rule_kinds = rule_kinds(schema);
    if rule_kinds.is_empty() && ad_hoc.is_none() {
        return;
    }
    let mut vocab = vocabulary(schema, frame);
    if let Some((_, projects)) = workspace {
        add_workspace_targets(&mut vocab, projects);
    }
    let local = WorkspaceCheckTarget::of(findings, schema, frame);
    let facts = match workspace {
        Some((selected, projects)) => {
            let projects = projects
                .iter()
                .map(|(alias, target)| {
                    let project = markdown_project(None, target);
                    (alias.as_str(), target.catalog, project)
                })
                .collect::<Vec<_>>();
            adapt_workspace(selected, &projects, complete)
        }
        None => adapt_markdown(findings, markdown_project(frame.name, &local), complete),
    };
    let mut parsed_rules = Vec::new();
    for (id, declarations) in &findings.declarations {
        if !rule_kinds.contains(id.kind.as_str()) {
            continue;
        }
        for declaration in declarations {
            let origin = render_id(frame.grammar(), id);
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
            let path = declaration.file.to_string_lossy();
            match parsed {
                Ok(_) if !rule_has_rationale(declaration) => report.errors.push(invalid_rule(
                    &origin,
                    path.as_ref(),
                    declaration.line,
                    "rule rationale is empty",
                )),
                Ok((rule, None)) => parsed_rules.push(rule),
                // §FS-rules.4.1: reported rather than evaluated — but the
                // subject is asked here, so one directory reaches one verdict.
                Ok((rule, Some(message))) => {
                    report.errors.push(
                        unresolved_subject_diagnostic(&rule, &facts).unwrap_or_else(|| {
                            invalid_rule(&origin, path.as_ref(), declaration.line, &message)
                        }),
                    );
                }
                Err(message) => report.errors.push(invalid_rule(
                    &origin,
                    path.as_ref(),
                    declaration.line,
                    &message,
                )),
            }
        }
    }
    if let Some(rule) = ad_hoc {
        parsed_rules.push(rule);
    }
    let precedence = citation_precedence(&rules.citations);
    // §FS-rules.7: one channel per level, so the absence arrives among the
    // errors its level already fills and every caller's own sort orders it.
    let (errors, ramp_warnings) = evaluate(&parsed_rules, &precedence, &facts);
    report.errors.extend(errors);
    // §FS-rules.7.8 / §FS-errors.4.1: the single-project `grund-core` arm is the one
    // caller that does not sort warnings downstream, and the sort is idempotent over
    // the already-ordered channel every caller hands in, so it needs no guard.
    report.warnings.extend(ramp_warnings);
    sort_diagnostics(&mut report.warnings);
    report
        .suggestions
        .extend(evaluate_suggestions(&parsed_rules, &precedence, &facts));
}

/// A rule rationale contains authored non-whitespace text after its declaration
/// heading. Both `check` and `init` use this one predicate so a blank body can
/// never render or execute on one surface only (§FS-rules.1, §FS-rules.4).
pub(crate) fn rule_has_rationale(declaration: &Declaration) -> bool {
    declaration.body_has_content
}

/// §FS-rules.7.6: a diagnostic *about* a rule rather than an evaluation *by*
/// one, so it never reaches the engine's group join — and it names its one rule
/// as its authority anyway, which is what keeps an unusable sentence inside a
/// scoped report instead of letting it read as `success`.
fn invalid_rule(origin: &str, path: &str, line: usize, message: &str) -> Diagnostic {
    Diagnostic {
        code: "invalid-rule",
        path: Some(path.into()),
        line: Some(line),
        column: None,
        message: format!("{origin} is not a valid rule: {message}"),
        sites: Vec::new(),
        authority: one_rules_authority(origin),
    }
}
