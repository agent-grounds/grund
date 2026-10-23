//! Checker orchestration for chapter rules (§FS-rules.4, §FS-rules.11,
//! §AR-checker.1). Grammar, facts, evaluation, and deduplication stay owned by
//! the rules component; this module only sequences and merges their results.

use crate::config::{Form, Frame, KindScope, KindTable, Rules, Schema};
use crate::grammar::render_catalog_id;
use crate::model::{
    Catalog, CheckReport, Code, Declaration, Diagnostic, Switch, paired_declarations,
};
use crate::resolver::WorkspaceCheckTarget;
use crate::rules::RuleAnchor;
use crate::rules::engine::{
    evaluate, evaluate_suggestions, one_rules_authority, unresolved_subject_diagnostic,
};
use crate::rules::markdown::{MarkdownProject, adapt_markdown, adapt_workspace};
use crate::rules::sentence::{ParsedRule, RuleSubject, RuleVocabulary, parse_rule};
use std::collections::{BTreeMap, BTreeSet};

use super::direction_findings::directions;
use super::fields::pair_fields_with_rules;
use super::rule_obligations::{arm_rule_directions, empty_rule_obligations};

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

/// The names of `schema`'s rule kinds, whose declarations are rules (§FS-rules.1).
fn rule_kinds(schema: &Schema) -> BTreeSet<&str> {
    schema
        .kinds()
        .filter(|(_, kind)| kind.form == Form::Rule)
        .map(|(name, _)| name)
        .collect()
}

/// One rule as the loader reads it (§FS-rules.7.10): the record its ID pairs with
/// (§FS-declarations.stubs.pairing.1), whose title and body are the sentence and
/// rationale `grund <ID> --brief` prints, and the declaration it is reported at —
/// the record itself, or the stub standing for a target the scan does not read.
struct RuleDeclaration<'a> {
    origin: String,
    record: &'a Declaration,
    site: &'a Declaration,
}

impl RuleDeclaration<'_> {
    fn anchor(&self) -> RuleAnchor {
        RuleAnchor {
            path: self.site.file.to_string_lossy().into_owned(),
            line: self.site.line,
            column: None,
        }
    }

    fn invalid(&self, message: &str) -> Diagnostic {
        invalid_rule(
            &self.origin,
            self.site.file.to_string_lossy().as_ref(),
            self.site.line,
            message,
        )
    }
}

/// Every rule `findings` declares, each paired declaration once, so a stub and the
/// scanned target it points at are one rule and a stub to an unscanned target is
/// its target's rule (§FS-rules.7.10, §AR-scanner.4.6). `check` and `init` both
/// read rules from here.
fn rule_declarations<'a>(
    findings: &'a Catalog,
    schema: &Schema,
    frame: Frame<'_>,
) -> Vec<RuleDeclaration<'a>> {
    let rule_kinds = rule_kinds(schema);
    findings
        .declarations
        .iter()
        .filter(|(id, _)| rule_kinds.contains(id.kind.as_str()))
        .flat_map(|(id, declarations)| {
            let origin = render_catalog_id(frame.grammar(), &findings.declarations, id);
            paired_declarations(declarations)
                .iter()
                .map(|paired| RuleDeclaration {
                    origin: origin.clone(),
                    record: paired.record,
                    site: paired.stand_in,
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

/// The vocabulary a rule of `schema`'s project is read in: its subject and a
/// bare object in that project's scope (§AR-config.1.5), named
/// sections as that project sets them.
pub(crate) fn vocabulary(schema: &Schema, rules: &Rules, frame: Frame<'_>) -> RuleVocabulary {
    let table = KindTable::standalone(schema);
    let scope = KindScope::Project("");
    let kinds: BTreeSet<String> = table
        .citable(scope)
        .into_iter()
        .map(str::to_string)
        .collect();
    RuleVocabulary {
        kinds,
        named_sections: table.named_sections(scope),
        named_kinds: BTreeSet::new(),
        // §AR-rules.1, §AR-config.1.5: wire both rule surfaces to the same
        // project-local vocabulary and configured place routes.
        non_citable: crate::rules::sentence::SubjectKinds::for_rules(
            table.non_citable_entries(scope),
            schema,
            rules,
            frame.run,
            frame.version,
        ),
        objects: table,
        id_grammars: vec![frame.grammar().clone()],
        section_separators: vec![schema.ids.section_separator.clone()],
    }
}

pub(crate) fn parse_ad_hoc(
    schema: &Schema,
    rules: &Rules,
    frame: Frame<'_>,
    sentence: &str,
) -> anyhow::Result<ParsedRule> {
    parse_ad_hoc_with_vocabulary(schema, sentence, vocabulary(schema, rules, frame))
}

pub(crate) fn parse_ad_hoc_with_workspace(
    schema: &Schema,
    rules: &Rules,
    frame: Frame<'_>,
    sentence: &str,
    projects: &BTreeMap<String, WorkspaceCheckTarget<'_>>,
) -> anyhow::Result<ParsedRule> {
    let mut vocab = vocabulary(schema, rules, frame);
    add_workspace_targets(&mut vocab, projects);
    parse_ad_hoc_with_vocabulary(schema, sentence, vocab)
}

/// `schema` is the one whose kinds the subject is read against, so it is the
/// one whose kinds a refusal that offers no form lists (§AR-rules.1).
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
        // §AR-rules.1: no form, and a kind is missing, so the kinds
        // follow.
        if error.unrecovered {
            anyhow::anyhow!(
                "{}\n{}",
                error.message,
                KindTable::standalone(schema).known_kinds_line(KindScope::Project(""))
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
    // §FS-rules.4.1.1: one namespace per project the run loaded; holding any at
    // all is what separates a scope that judged an alias and said no from one
    // that could not judge it.
    vocabulary.objects.add_projects(
        projects
            .iter()
            .map(|(alias, project)| (alias.as_str(), project.schema)),
    );
}

/// The vocabulary a run that already loaded its workspace resolves rule objects
/// against (§FS-rules.4.1). An empty project map is no workspace at all, so it
/// yields the plain single-project vocabulary and every namespaced object kind
/// in it is unverifiable here.
pub(crate) fn workspace_vocabulary(
    schema: &Schema,
    rules: &Rules,
    frame: Frame<'_>,
    projects: &BTreeMap<String, WorkspaceCheckTarget<'_>>,
) -> RuleVocabulary {
    let mut vocab = vocabulary(schema, rules, frame);
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
    rules: &Rules,
    frame: Frame<'_>,
    declared: &[(String, Schema)],
) -> RuleVocabulary {
    let mut vocab = vocabulary(schema, rules, frame);
    vocab.objects.add_projects(
        declared
            .iter()
            .map(|(alias, schema)| (alias.as_str(), schema)),
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
    /// Each rendered rule's subject kind, by qualified rule ID: the kind whose
    /// destination its bullet lands in (§FS-init.2.3.5.11). A rule about another
    /// project's kind, or a declaration no local kind names, has none and lands
    /// at the root.
    pub(crate) subjects: BTreeMap<String, String>,
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
    let mut rows = Vec::new();
    let mut unverifiable = Vec::new();
    let mut subjects = BTreeMap::new();
    for rule in rule_declarations(findings, schema, frame) {
        let title = rule
            .record
            .title
            .as_deref()
            .ok_or_else(|| rule.invalid("rule declaration has no title"))?;
        if !rule_has_rationale(rule.record) {
            return Err(rule.invalid("rule rationale is empty"));
        }
        let (parsed, unverifiable_here) =
            parse_rule(title, rule.origin.clone(), rule.anchor(), vocab)
                .map_err(|error| rule.invalid(&error.message))?;
        // §FS-rules.4.1: the object's namespace is the whole of what is
        // unverifiable, so every other question is put either way.
        if let Some(diagnostic) = unresolved_subject_diagnostic(&parsed, &facts) {
            return Err(diagnostic);
        }
        // §FS-rules.4.1.2: reported, rendered, and not a reason to withhold
        // the write.
        if let Some(message) = unverifiable_here {
            unverifiable.push(rule.invalid(&message));
        }
        if let Some(kind) = subject_kind(&parsed.subject, schema) {
            subjects.insert(rule.origin.clone(), kind);
        }
        rows.push((rule.origin, title.to_string()));
    }
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(ConfiguredRules {
        rows,
        subjects,
        unverifiable,
    })
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
    // §FS-check.1.4.1: a rule kind or a `--rule` turns the rule checks on.
    report.armed.arm(Switch::Rules);
    let mut vocab = vocabulary(schema, rules, frame);
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
    let errors_before = report.errors.len();
    for rule in rule_declarations(findings, schema, frame) {
        let parsed = rule
            .record
            .title
            .as_deref()
            .ok_or_else(|| "rule declaration has no title".to_string())
            .and_then(|title| {
                parse_rule(title, rule.origin.clone(), rule.anchor(), &vocab)
                    .map_err(|error| error.message)
            });
        match parsed {
            Ok(_) if !rule_has_rationale(rule.record) => {
                report.errors.push(rule.invalid("rule rationale is empty"));
            }
            Ok((parsed, None)) => parsed_rules.push(parsed),
            // §FS-rules.4.1: reported rather than evaluated — but the
            // subject is asked here, so one directory reaches one verdict.
            Ok((parsed, Some(message))) => {
                report.errors.push(
                    unresolved_subject_diagnostic(&parsed, &facts)
                        .unwrap_or_else(|| rule.invalid(&message)),
                );
            }
            Err(message) => report.errors.push(rule.invalid(&message)),
        }
    }
    if let Some(rule) = ad_hoc {
        parsed_rules.push(rule);
    }
    let unread = report.errors.len() > errors_before;
    arm_rule_directions(&parsed_rules, schema, unread, &mut report.armed);
    report.warnings.extend(empty_rule_obligations(
        findings,
        rules,
        schema,
        frame,
        &parsed_rules,
    ));
    let precedence = directions(findings, rules, schema, frame, &parsed_rules);
    // §FS-rules.7: one channel per level, so the absence arrives among the
    // errors its level already fills and every caller's own sort orders it.
    // §FS-rules.7.10: a rule declared through a stub reports on the ordinary channels.
    report
        .errors
        .extend(evaluate(&parsed_rules, &precedence, &facts));
    report
        .suggestions
        .extend(evaluate_suggestions(&parsed_rules, &precedence, &facts));
    // §FS-rules.6: a field that expresses a rule reports a shared absence once.
    pair_fields_with_rules(findings, schema, frame, report);
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
        code: Code::INVALID_RULE,
        path: Some(path.into()),
        line: Some(line),
        column: None,
        message: format!("{origin} is not a valid rule: {message}"),
        sites: Vec::new(),
        authority: one_rules_authority(origin),
    }
}

/// The local kind a rule is about (§FS-init.2.3.5.11): a kind subject's own, or
/// the kind whose prefix names the declaration an exact subject selects.
fn subject_kind(subject: &RuleSubject, schema: &Schema) -> Option<String> {
    let named = match subject {
        RuleSubject::Kind(kind) | RuleSubject::ChapterOfKind { kind, .. } => {
            return (!kind.contains('/')).then(|| kind.clone());
        }
        RuleSubject::ExactDeclaration(declaration)
        | RuleSubject::ExactChapter { declaration, .. } => declaration.to_string(),
    };
    if named.contains('/') {
        return None;
    }
    schema
        .rows
        .iter()
        .filter(|row| row.kind.is_some())
        .map(|row| row.name.as_str())
        .filter(|kind| {
            named
                .strip_prefix(kind)
                .is_some_and(|rest| rest.starts_with('-'))
        })
        .max_by_key(|kind| kind.len())
        .map(str::to_string)
}
