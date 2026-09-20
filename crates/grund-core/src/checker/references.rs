//! The reference-resolution rule family — dangling citations (§FS-check.3.1),
//! missing sections (§FS-check.3.2), unknown project aliases (§FS-check.3.8),
//! and unresolved number-only shorthands (§FS-check.3.13). The scope layer that
//! decides where the family is reported is its `reference_scope.rs` sibling
//! (§FS-check.1.3, §FS-check.3.14, §AR-checker.2.13,
//! §AR-core-module-layout.1.5).
//!
//! It sits beside `report.rs` rather than inside it because `grund check --full`
//! runs this one family a second time, over the part of the tree `[scan] include`
//! leaves out, while every other rule stays inside the configured scope
//! (§AR-core-module-layout.1).

use super::reference_scope::ScanScope;
use super::shorthand::{ShorthandIndexes, report_shorthand_citation};
use super::support::{
    citation_in_markdown_inline_code, close_enough_for_hint, dangling_message, edit_distance,
    missing_snapshot_message,
};
use crate::config::{Config, KindResolution, display_path};
use crate::grammar::render_qualified_id;
use crate::model::{CheckReport, Diagnostic, Findings};
use crate::resolver::{WorkspaceCheckTarget, join_alternatives, target_for_citation};
use crate::workspace::namespace_is_unverified;
use std::collections::BTreeMap;

/// Which of a `--full` run's two scopes a citation site is being judged on
/// (§FS-check.1.3.3). It changes exactly one thing: outside the configured scope,
/// `grund fmt --write` will not rewrite the site either, so the mechanical
/// shorthand form is withheld there (§FS-check.3.14.4).
#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum ReferenceTier {
    Configured,
    OutOfScope,
}

/// §AR-checker.2.3, §AR-checker.2.4, §AR-checker.2.12: resolve every citation and
/// report the ones that resolve to nothing. `outside`, when set, restricts the
/// pass to sites *outside* that scope — the out-of-scope tier's one difference in
/// what it looks at (§FS-check.3.14); the ordinary run passes `None` and judges
/// every site the walk found.
///
/// §FS-workspace.4.3: an alias path into an absent optional member is neither
/// resolved nor unknown — it is *unverified*, and the run says so once at the entry
/// that made the skip legal rather than at every site (§FS-check.4.9.1). Every other
/// unknown alias still errors here.
pub(super) fn check_citation_resolution(
    findings: &Findings,
    config: &Config,
    path_config: &Config,
    workspace: &BTreeMap<String, WorkspaceCheckTarget<'_>>,
    tier: ReferenceTier,
    outside: Option<&ScanScope>,
    report: &mut CheckReport,
) {
    // §FS-check.3.24: every persisted local spelling is independently loud.
    // Resolved forms are ordinary citations; unresolved/unsupported forms stay
    // in the diagnostic-only candidate list and never acquire a target.
    for cite in findings.citations.iter().filter(|cite| cite.local_section) {
        if outside.is_some_and(|scope| scope.contains(&cite.file)) {
            continue;
        }
        let section = cite.section.as_deref().unwrap_or_default();
        report.errors.push(Diagnostic {
            code: "local-section-citation",
            path: Some(cite.file.clone()),
            line: Some(cite.line),
            column: Some(cite.column),
            message: format!(
                "local section citation {}; write {}{}{}{}",
                cite.text.trim(),
                config.marker,
                render_qualified_id(&config.grammar, None, &cite.id),
                config.section_separator,
                section,
            ),
            sites: Vec::new(),
        });
    }
    for candidate in &findings.local_section_citation_candidates {
        if outside.is_some_and(|scope| scope.contains(&candidate.file)) {
            continue;
        }
        let written = candidate.text.trim();
        let tail = written.strip_prefix(&config.marker).unwrap_or(written);
        let message = if candidate.section.is_none() {
            format!(
                "unsupported local section citation {written}; write a full citation or <{}>{tail} to show the shape without citing it",
                config.marker,
            )
        } else {
            format!(
                "local section citation {written} has no enclosing declaration; write a full citation or <{}>{tail} to show the shape without citing it",
                config.marker,
            )
        };
        report.errors.push(Diagnostic {
            code: "local-section-citation",
            path: Some(candidate.file.clone()),
            line: Some(candidate.line),
            column: Some(candidate.column),
            message,
            sites: Vec::new(),
        });
    }
    let mut shorthand_indexes = ShorthandIndexes::default();
    for cite in &findings.citations {
        if let Some(scope) = outside
            && scope.contains(&cite.file)
        {
            continue;
        }
        let Some(target) = target_for_citation(cite, findings, config, workspace) else {
            // `target_for_citation` only returns `None` when the
            // namespace is present and unknown — so the namespace is always
            // Some here (§AR-resolver.1).
            let namespace = cite
                .namespace
                .as_deref()
                .expect("resolver only returns None for qualified citations");
            // §FS-workspace.4.3: unverified, not unknown — see this function's docs.
            if namespace_is_unverified(config, namespace) {
                continue;
            }
            report.errors.push(Diagnostic {
                code: "unknown-project",
                path: Some(cite.file.clone()),
                line: Some(cite.line),
                column: Some(cite.column),
                message: unknown_project_message(
                    namespace,
                    workspace.keys().map(String::as_str),
                    &config.workspace_scope_path,
                ),
                sites: Vec::new(),
            });
            continue;
        };
        // §FS-check.3.13.3 / §AR-checker.2.12: the shorthand pass, and the one rule that
        // can end this citation early — an unresolved shorthand skips the dangling check
        // below rather than adding `unknown reference FS-042` for a token that is not an ID.
        if cite.shorthand
            && report_shorthand_citation(
                cite,
                config,
                &target,
                tier,
                &mut shorthand_indexes,
                report,
            )
        {
            continue;
        }
        // §FS-check.3.1 / §FS-workspace.4.1: a citation whose ID is declared
        // nowhere in its target namespace is dangling.
        let Some(decls) = target.findings.declarations.get(&cite.id) else {
            let snapshot_kind = target
                .config
                .kinds
                .iter()
                .find(|kind| kind.kind == cite.id.kind && kind.fetch.is_some());
            let in_inline_code = citation_in_markdown_inline_code(cite);
            let (code, message, warning) = if let Some(kind) = snapshot_kind {
                // §FS-check.3.14.1: out-of-scope citations stay fixed dangling errors;
                // a target kind's in-scope `should` must not demote this opt-in tier.
                let should_warn = tier == ReferenceTier::Configured
                    && kind.resolve == Some(KindResolution::Should);
                let home = kind
                    .file
                    .as_deref()
                    .or(kind.folder.as_deref())
                    .expect("fetch-enabled kind has exactly one home after config validation");
                let home = display_path(path_config, &target.config.root.join(home));
                let message = missing_snapshot_message(
                    target.config,
                    cite.namespace.as_deref(),
                    target.findings,
                    &cite.id,
                    in_inline_code,
                    &home,
                    !should_warn,
                );
                (
                    if should_warn {
                        "missing-snapshot"
                    } else {
                        "dangling"
                    },
                    message,
                    should_warn,
                )
            } else {
                (
                    "dangling",
                    dangling_message(
                        target.config,
                        cite.namespace.as_deref(),
                        target.findings,
                        &cite.id,
                        in_inline_code,
                    ),
                    false,
                )
            };
            let diagnostic = Diagnostic {
                code,
                path: Some(cite.file.clone()),
                line: Some(cite.line),
                column: Some(cite.column),
                message,
                sites: Vec::new(),
            };
            if warning {
                // §FS-check.4.12: `should` is a distinct fixed warning and
                // leaves a warning-only check at exit 0.
                report.warnings.push(diagnostic);
            } else {
                report.errors.push(diagnostic);
            }
            continue;
        };
        // §FS-check.3.2: the ID resolves but no declaration has a heading at the
        // cited section path.
        if let Some(sec) = &cite.section {
            let any_match = decls.iter().any(|d| d.sections.contains_key(sec));
            if !any_match {
                let coordinate = format!(
                    "{}{}{}",
                    render_qualified_id(
                        &target.config.grammar,
                        cite.namespace.as_deref(),
                        &cite.id
                    ),
                    target.config.section_separator,
                    sec
                );
                let message = if target.config.named_sections
                    && target.config.grammar.is_named_section(Some(sec))
                    && cite.has_marker
                {
                    format!(
                        "section not found: {coordinate}; write <{}> before it to show the shape without citing it",
                        target.config.marker
                    )
                } else {
                    format!("missing section {coordinate}")
                };
                report.errors.push(Diagnostic {
                    code: "missing-section",
                    path: Some(cite.file.clone()),
                    line: Some(cite.line),
                    column: Some(cite.column),
                    message,
                    sites: Vec::new(),
                });
            }
        }
    }
}

/// §FS-check.3.8: the unknown-alias message. A qualified citation names its
/// target by the whole alias path (§FS-workspace.6.1), and the mistake that
/// invites is writing a project's short name where its path is required. So
/// before giving up, look for a project the citation could have meant: one
/// whose path continues the written segments (`group/alpha` for `group`), one
/// whose path *ends* with what was written (`sprayer` for `group/sprayer`), one
/// whose last segment matches under a different parent (a wrong prefix), or one
/// a typo away. §GOAL-friendliness-first — the reader who has the tree in front
/// of them is the one who does not need this; the agent editing one file is.
///
/// A **narrowed** run (non-empty `scope_path`) normally withholds every tier:
/// it cannot tell a path that dropped a prefix from one that correctly names a
/// project above or beside the subtree. A written path that strictly extends
/// the scope segment by segment is the exception, because every candidate the
/// run loaded for that path is inside the subtree it can judge
/// (§FS-check.3.8.1). Ineligible paths keep the scope-only message
/// (§FS-check.3.8.3, §FS-workspace.6.1.5).
pub(crate) fn unknown_project_message<'a>(
    namespace: &str,
    known: impl Iterator<Item = &'a str>,
    scope_path: &str,
) -> String {
    if !scope_path.is_empty() && !alias_strictly_extends_scope(namespace, scope_path) {
        // §FS-check.3.8.1: 0.13.2 keeps the legacy diagnostic as a prefix while
        // appending §FS-errors.3.3's fixed compatibility explanation and horizon.
        return format!(
            "unknown project alias {namespace}; only the {scope_path} subtree is in scope here — check from the workspace root for a path outside it — here, the {scope_path} subtree means the {scope_path} project and its descendants; this wording changes in grund 0.15.0"
        );
    }
    let candidates = nearest_project_aliases(namespace, known);
    if candidates.is_empty() {
        return format!("unknown project alias {namespace}");
    }
    format!(
        "unknown project alias {namespace}; did you mean {}?",
        join_alternatives(&candidates)
    )
}

/// §FS-check.3.8.1: eligibility is a strict segment-prefix relation, not a
/// lexical prefix (`grouped/alpha` is outside scope `group`).
fn alias_strictly_extends_scope(namespace: &str, scope_path: &str) -> bool {
    let mut namespace_segments = namespace.split('/');
    scope_path
        .split('/')
        .all(|segment| namespace_segments.next() == Some(segment))
        && namespace_segments.next().is_some()
}

/// The projects a written alias path plausibly meant, best tier first. Tiers do
/// not mix: a proper-prefix continuation or suffix match is near-certain, and
/// diluting either with edit-distance noise would make the good hint harder to
/// act on. The outermost root always asks; a narrowed run asks only for a
/// strict extension of its scope (§FS-check.3.8.1), so no tier here is
/// conditional on scope.
pub(crate) fn nearest_project_aliases<'a>(
    namespace: &str,
    known: impl Iterator<Item = &'a str>,
) -> Vec<String> {
    let written: Vec<&str> = namespace.split('/').collect();
    let (mut prefix, mut suffix, mut same_leaf, mut near) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for candidate in known {
        let segments: Vec<&str> = candidate.split('/').collect();
        if segments.len() > written.len() && segments.starts_with(&written) {
            prefix.push(candidate.to_string());
        } else if segments.len() > written.len() && segments.ends_with(&written) {
            suffix.push(candidate.to_string());
        } else if segments.last() == written.last() {
            same_leaf.push(candidate.to_string());
        } else if close_enough_for_hint(
            edit_distance(namespace, candidate),
            namespace.chars().count(),
            candidate.chars().count(),
        ) {
            near.push(candidate.to_string());
        }
    }
    let mut best = if !prefix.is_empty() {
        prefix
    } else if !suffix.is_empty() {
        suffix
    } else if !same_leaf.is_empty() {
        same_leaf
    } else {
        near
    };
    best.sort();
    best.dedup();
    // Three is enough to disambiguate the common `api` collision without
    // turning one finding into a catalogue; `grund list` is the catalogue.
    best.truncate(3);
    best
}
