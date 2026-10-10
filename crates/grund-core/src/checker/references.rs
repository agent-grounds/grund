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
use crate::config::{Frame, KindResolution, Origin, Schema};
use crate::grammar::render_qualified_id;
use crate::model::{Catalog, CheckReport, Diagnostic};
use crate::resolver::{
    WorkspaceCheckTarget, join_alternatives, section_resolves, target_for_citation,
};
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

/// The last release cut before the declaration-local section form became an
/// error — the "from" half of the pair §FS-check.3.24.1 requires every shape of
/// the finding to name, in the past tense §FS-distribution.4.2 closes the
/// vocabulary on. `local_section_rule_releases_are_ordered_and_behind_us` holds
/// both halves to releases that actually happened, the way
/// `INDEX_RULE_PRIOR_RELEASE` is held.
pub(super) const LOCAL_SECTION_RULE_PRIOR_RELEASE: &str = "0.13.1";

/// The release the declaration-local section verdict moved in — the "to" half of
/// that pair (§FS-check.3.24.1).
pub(super) const LOCAL_SECTION_RULE_RELEASE: &str = "0.14.0";

/// §FS-check.3.24.1: the clause every shape of the finding ends with. Appended,
/// never woven in, so the text the rule shipped with survives as a verbatim
/// contiguous prefix wherever that text still stands (§FS-check.3.24.2).
fn local_section_release_attribution() -> String {
    format!(
        " — unchecked in grund {LOCAL_SECTION_RULE_PRIOR_RELEASE}, an error in {LOCAL_SECTION_RULE_RELEASE}"
    )
}

/// §FS-check.3.24, §FS-check.3.24.3: the remedy of every shape that has no full
/// citation to propose — an ownerless site, an unsupported token, and an owned site
/// whose owner lacks the section. `<tail>` is the token with its marker taken off,
/// so the escape is the token the author would type.
fn full_citation_or_escape(schema: &Schema, written: &str) -> String {
    let marker = &schema.citation.marker;
    let tail = written.strip_prefix(marker).unwrap_or(written);
    format!("write a full citation or <{marker}>{tail} to show the shape without citing it")
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
///
/// `resolve` answers a target kind's `resolve` policy by the citation's namespace
/// (§FS-check.4.12): a rule of the target's, which its `WorkspaceCheckTarget`
/// does not carry (§AR-config.5).
#[allow(clippy::too_many_arguments)]
pub(super) fn check_citation_resolution(
    findings: &Catalog,
    schema: &Schema,
    frame: Frame<'_>,
    workspace: &BTreeMap<String, WorkspaceCheckTarget<'_>>,
    resolve: &dyn Fn(Option<&str>, &str) -> Option<KindResolution>,
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
        let written = cite.text.trim();
        let section = cite.section.as_deref().unwrap_or_default();
        let owner = render_qualified_id(frame.grammar(), None, &cite.id);
        // §FS-check.3.24.3: where `missing section` fires beside this site, its full
        // citation would only trade one error for the other; name the absence and the
        // escape instead.
        let resolves = section_resolves(findings, schema, frame, &cite.id, section);
        let remedy = if resolves {
            format!(
                "write {}{owner}{}{section}",
                schema.citation.marker, schema.ids.section_separator
            )
        } else {
            format!(
                "{owner} has no section {section}, so {}",
                full_citation_or_escape(schema, written)
            )
        };
        // §FS-check.3.24.1: offered exactly where the next formatter pass would
        // write this site — the test §FS-check.3.14.4 makes for the sibling rule,
        // so no finding names a command that would answer `rewrote 0 lines`.
        let command = if cite.shorthand_rewritable
            && tier == ReferenceTier::Configured
            // §FS-check.3.24.1: an absent target section is a site fmt declines.
            && resolves
        {
            "; run `grund fmt --write`"
        } else {
            ""
        };
        report.errors.push(Diagnostic {
            code: "local-section-citation",
            path: Some(cite.file.clone()),
            line: Some(cite.line),
            column: Some(cite.column),
            message: format!(
                "local section citation {written}; {remedy}{}{command}",
                local_section_release_attribution(),
            ),
            sites: Vec::new(),
            authority: Vec::new(),
        });
    }
    for candidate in &findings.local_section_citation_candidates {
        if outside.is_some_and(|scope| scope.contains(&candidate.file)) {
            continue;
        }
        let written = candidate.text.trim();
        let guidance = full_citation_or_escape(schema, written);
        let mut message = if candidate.section.is_none() {
            format!("unsupported local section citation {written}; {guidance}")
        } else {
            format!("local section citation {written} has no enclosing declaration; {guidance}")
        };
        // §FS-check.3.24.1: both candidate shapes name the releases and neither
        // ever names the command — §FS-fmt.2.4 leaves them byte-identical
        // because there is no owner to expand them against.
        message.push_str(&local_section_release_attribution());
        report.errors.push(Diagnostic {
            code: "local-section-citation",
            path: Some(candidate.file.clone()),
            line: Some(candidate.line),
            column: Some(candidate.column),
            message,
            sites: Vec::new(),
            authority: Vec::new(),
        });
    }
    let mut shorthand_indexes = ShorthandIndexes::default();
    for cite in &findings.citations {
        if let Some(scope) = outside
            && scope.contains(&cite.file)
        {
            continue;
        }
        let Some(target) = target_for_citation(cite, findings, schema, frame, workspace) else {
            // `target_for_citation` only returns `None` when the
            // namespace is present and unknown — so the namespace is always
            // Some here (§AR-resolver.1).
            let namespace = cite
                .namespace
                .as_deref()
                .expect("resolver only returns None for qualified citations");
            // §FS-workspace.4.3: unverified, not unknown — see this function's docs.
            if namespace_is_unverified(frame.run, namespace) {
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
                    &frame.run.workspace.scope_path,
                ),
                sites: Vec::new(),
                authority: Vec::new(),
            });
            continue;
        };
        // §FS-check.3.13.3 / §AR-checker.2.12: the shorthand pass, and the one rule that
        // can end this citation early — an unresolved shorthand skips the dangling check
        // below rather than adding `unknown reference FS-042` for a token that is not an ID.
        if cite.shorthand
            && report_shorthand_citation(
                cite,
                schema,
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
        if !target.catalog.declarations.contains_key(&cite.id) {
            // §FS-check.3.1.3: a fetch-enabled target kind keeps snapshots.
            let snapshot_row = target.schema.rows.iter().find(|row| {
                row.name == cite.id.kind
                    && row
                        .kind
                        .as_ref()
                        .is_some_and(|kind| matches!(kind.origin, Origin::External { .. }))
            });
            let in_inline_code = citation_in_markdown_inline_code(cite);
            let (code, message, warning) = if let Some(row) = snapshot_row {
                // §FS-check.3.14.1: out-of-scope citations stay fixed dangling errors;
                // a target kind's in-scope `should` must not demote this opt-in tier.
                let should_warn = tier == ReferenceTier::Configured
                    && resolve(cite.namespace.as_deref(), &row.name)
                        == Some(KindResolution::Should);
                let home = row
                    .file()
                    .or(row.folder())
                    .expect("fetch-enabled kind has exactly one home after config validation");
                let home = frame.display_path(&target.run.root.join(home));
                let message = missing_snapshot_message(
                    target.schema,
                    target.frame(),
                    cite.namespace.as_deref(),
                    target.catalog,
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
                        target.schema,
                        target.frame(),
                        cite.namespace.as_deref(),
                        target.catalog,
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
                authority: Vec::new(),
            };
            if warning {
                // §FS-check.4.12: `should` is a distinct fixed warning and
                // leaves a warning-only check at exit 0.
                report.warnings.push(diagnostic);
            } else {
                report.errors.push(diagnostic);
            }
            continue;
        }
        // §FS-check.3.2: the ID resolves but no declaration has a heading at the
        // cited section path — the lookup §FS-fmt.2.4.6 declines a local rewrite on,
        // which reads a stub's sections from its target, scanned or not (§FS-check.3.2.1).
        if let Some(sec) = &cite.section {
            if !section_resolves(target.catalog, target.schema, target.frame(), &cite.id, sec) {
                let coordinate = format!(
                    "{}{}{}",
                    render_qualified_id(
                        &target.compiled.grammar,
                        cite.namespace.as_deref(),
                        &cite.id
                    ),
                    target.schema.ids.section_separator,
                    sec
                );
                let message = if target.schema.ids.named_sections
                    && target.compiled.grammar.is_named_section(Some(sec))
                    && cite.has_marker
                {
                    format!(
                        "section not found: {coordinate}; write <{}> before it to show the shape without citing it",
                        target.schema.citation.marker
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
                    authority: Vec::new(),
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
        // §FS-check.3.8.4: the final wording §FS-errors.3.3's migration reached in 0.16.0.
        return format!(
            "unknown project alias {namespace}; the {scope_path} project and its descendants are in scope here — check from the workspace root for a path outside that subtree"
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
