//! Checker orchestration for chapter rules (§FS-rules.4, §FS-rules.11,
//! §AR-checker.1). Grammar, facts, evaluation, and deduplication stay owned by
//! the rules component; this module only sequences and merges their results.

use crate::config::{Config, known_kinds_line};
use crate::grammar::render_id;
use crate::model::{Catalog, CheckReport, Declaration, Diagnostic, Id};
use crate::resolver::{SectionHome, WorkspaceCheckTarget, section_home};
use crate::rules::RuleAnchor;
use crate::rules::engine::{
    citation_precedence, evaluate, evaluate_suggestions, one_rules_authority,
    unresolved_subject_diagnostic,
};
use crate::rules::markdown::{MarkdownProject, SectionHomes, adapt_markdown, adapt_workspace};
use crate::rules::sentence::{ParsedRule, RuleVocabulary, parse_rule};
use crate::workspace::expand_workspace_tree;
use std::collections::{BTreeMap, BTreeSet};

use super::support::sort_diagnostics;

/// What the rules adapter reads of `config`: the façade stays in the checker
/// (§AR-config.5, §AR-rules.3).
pub(crate) fn markdown_project(config: &Config) -> MarkdownProject<'_> {
    MarkdownProject {
        name: config.project_name.as_deref(),
        root: &config.root,
        grammar: &config.grammar,
        section_separator: &config.section_separator,
        homes: config,
    }
}

/// A project's sections resolve as `check` resolves them, a stub's in its target
/// read under this project's scan settings (§FS-check.3.2.1, §AR-resolver.5).
impl SectionHomes for Config {
    fn section_home<'c>(
        &self,
        findings: &'c Catalog,
        id: &Id,
        section: &str,
    ) -> Option<SectionHome<'c>> {
        section_home(findings, self, id, section)
    }
}

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
    parse_ad_hoc_with_vocabulary(config, sentence, vocabulary(config))
}

pub(crate) fn parse_ad_hoc_with_workspace(
    config: &Config,
    sentence: &str,
    projects: &BTreeMap<String, WorkspaceCheckTarget<'_>>,
) -> anyhow::Result<ParsedRule> {
    let mut vocab = vocabulary(config);
    add_workspace_targets(&mut vocab, projects);
    parse_ad_hoc_with_vocabulary(config, sentence, vocab)
}

/// `config` is the one whose kinds the subject is read against, so it is the
/// one whose kinds a refusal that offers no form lists (§FS-rules.3.5.2).
fn parse_ad_hoc_with_vocabulary(
    config: &Config,
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
        // §FS-rules.3.5.2: nothing to paste back, so the kinds follow the reason.
        if error.unrecovered {
            anyhow::anyhow!("{}\n{}", error.message, known_kinds_line(&config.kinds))
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
            .map(|(alias, project)| (alias.clone(), project.config)),
    );
}

/// One namespace per project the run loaded. Holding any at all is what
/// separates a scope that judged an alias and said no from one that could not
/// judge it (§FS-rules.4.1.1).
fn add_namespaces<'a>(
    vocabulary: &mut RuleVocabulary,
    namespaces: impl IntoIterator<Item = (String, &'a Config)>,
) {
    vocabulary
        .target_namespaces
        .extend(namespaces.into_iter().map(|(alias, config)| {
            let kinds = config
                .kinds
                .iter()
                .filter(|kind| kind.citable)
                .map(|kind| kind.kind.clone())
                .collect();
            (alias, kinds)
        }));
}

/// The vocabulary a run that already loaded its workspace resolves rule objects
/// against (§FS-rules.4.1). An empty project map is no workspace at all, so it
/// yields the plain single-project vocabulary and every namespaced object kind
/// in it is unverifiable here.
pub(crate) fn workspace_vocabulary(
    config: &Config,
    projects: &BTreeMap<String, WorkspaceCheckTarget<'_>>,
) -> RuleVocabulary {
    let mut vocab = vocabulary(config);
    if !projects.is_empty() {
        add_workspace_targets(&mut vocab, projects);
    }
    vocab
}

/// The vocabulary for a command that resolves rules without loading a workspace
/// of its own — `init` (§FS-rules.4.1).
///
/// Read from the workspace this config *declares*, never one climbed to from
/// above: `init` does climb to render `### Workspace members`, but that is
/// teaching and this is judging, and judging off a climbed tree would make the
/// same bytes valid or invalid depending on what happens to sit on disk beside
/// the checkout. A member cloned alone would then get a different verdict from
/// the same rule (§FS-workspace.5.1, §DF-unverifiable-rule-scope).
///
/// Only stage 1 of the workspace load, because the aliases are all that is
/// wanted and stages 2 and 3 scan every member's whole tree. Expanding is what
/// makes the alias set `check`'s by construction. Best-effort like every other
/// workspace read `init` does: an expansion that fails raises nothing and costs
/// the run only the members it could not reach.
///
/// It does not cost the run the workspace itself. A config that declares
/// `[workspace]` holds at least its own project's namespace, whatever its member
/// list expands to — the entry an empty `members` already yields, recovered here
/// by expanding the same tree with the member list emptied. So a run standing at
/// a workspace root is never told that no workspace is in scope and sent to the
/// workspace root, one directory reaches one verdict whether a member is absent
/// or unlisted, and the rule an unreachable member's alias names is an invalid
/// one rather than a bullet written out of a broken tree (§FS-rules.4.1.1).
pub(crate) fn declared_workspace_vocabulary(config: &Config) -> RuleVocabulary {
    let mut vocab = vocabulary(config);
    if !config.workspace_declared {
        return vocab;
    }
    let mut root_config = config.clone();
    let entries = expand_workspace_tree(&mut root_config).unwrap_or_else(|_| {
        let mut alone = config.without_members();
        expand_workspace_tree(&mut alone).unwrap_or_default()
    });
    add_namespaces(
        &mut vocab,
        entries
            .iter()
            .map(|entry| (entry.alias.clone(), &entry.config)),
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
    config: &Config,
    vocab: &RuleVocabulary,
) -> Result<ConfiguredRules, Diagnostic> {
    let facts = adapt_markdown(findings, markdown_project(config), true);
    let rule_kinds = config
        .kinds
        .iter()
        .filter(|kind| kind.rules)
        .map(|kind| kind.kind.as_str())
        .collect::<BTreeSet<_>>();
    let mut rows = Vec::new();
    let mut unverifiable = Vec::new();
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
pub(crate) fn check_chapter_rules(
    findings: &Catalog,
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
    let facts = match workspace {
        Some((selected, projects)) => {
            let projects = projects
                .iter()
                .map(|(alias, target)| {
                    let project = markdown_project(target.config);
                    (alias.as_str(), target.findings, project)
                })
                .collect::<Vec<_>>();
            adapt_workspace(selected, &projects, complete)
        }
        None => adapt_markdown(findings, markdown_project(config), complete),
    };
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
            let path = declaration.file.to_string_lossy();
            match parsed {
                Ok(_) if !rule_has_rationale(declaration) => report.errors.push(invalid_rule(
                    &origin,
                    path.as_ref(),
                    declaration.line,
                    "rule rationale is empty",
                )),
                Ok((rule, None)) => rules.push(rule),
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
        rules.push(rule);
    }
    let precedence = citation_precedence(&config.citations);
    // §FS-rules.7: one channel per level, so the absence arrives among the
    // errors its level already fills and every caller's own sort orders it.
    let (errors, ramp_warnings) = evaluate(&rules, &precedence, &facts);
    report.errors.extend(errors);
    // §FS-rules.7.8 / §FS-errors.4.1: the single-project `grund-core` arm is the one
    // caller that does not sort warnings downstream, and the sort is idempotent over
    // the already-ordered channel every caller hands in, so it needs no guard.
    report.warnings.extend(ramp_warnings);
    sort_diagnostics(&mut report.warnings);
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
