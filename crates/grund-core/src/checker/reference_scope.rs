//! Configured-scope narrowing and the out-of-scope reference tier
//! (§AR-core-module-layout.1.5, §FS-check.1.3, §FS-check.3.14).

use anyhow::Result;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::references::{ReferenceTier, check_citation_resolution};
use super::sections::retain_heading_findings_in_scope;
use crate::config::{Config, unwalked_home_roots};
use crate::model::{CheckReport, DeclarationSource, Diagnostic, Findings, sort_path_key};
use crate::resolver::{WorkspaceCheckTarget, WorkspaceProject};
use crate::scanner::scan_roots_for;

/// The roots a run *without* `--full` walks: the explicit path argument, or
/// `[scan] include` resolved against the config root (§FS-config.3.5.7). Under
/// `--full` the walk is wider than this, and the difference is what separates
/// the ordinary report from the out-of-scope tier (§FS-check.1.3).
///
/// Each root is held in both its written and its canonical form: the walk yields
/// paths built from `config.root`, while `E2E` case declarations carry
/// canonicalized directories (§AR-scanner.6.2), and a scope test has to answer the
/// same for both.
pub(crate) struct ScanScope {
    roots: Vec<PathBuf>,
    /// The `scan = false` homes under those roots (§FS-config.3.4.7.5): listed by
    /// the config, and read by this run only because `--full` widened the walk.
    unwalked: Vec<PathBuf>,
}

impl ScanScope {
    pub(super) fn contains(&self, path: &Path) -> bool {
        // §FS-config.3.4.7.5: a file in a home the config lists without walking is
        // outside the configured scope even when a root above it is inside — the
        // scope is a set of roots, less the homes a run without `--full` never reads.
        if self.unwalked.iter().any(|home| path.starts_with(home)) {
            return false;
        }
        self.roots
            .iter()
            .any(|root| path == root || path.starts_with(root))
    }
}

/// §FS-check.1.3: the configured scope of this run, or `None` when the walk was
/// already the configured one — without `--full` there is no second tier, and no
/// narrowing to do.
pub(crate) fn configured_scope(
    config: &Config,
    path: &Path,
    path_provided: bool,
    full: bool,
) -> Result<Option<ScanScope>> {
    if !full {
        return Ok(None);
    }
    let mut roots = scan_roots_for(config, Some(path), path_provided, false)?;
    let canonical = roots
        .iter()
        .filter_map(|root| fs::canonicalize(root).ok())
        .collect::<Vec<_>>();
    roots.extend(canonical);
    roots.sort_by_key(|root| sort_path_key(root));
    roots.dedup();
    // Both spellings again, for the same reason the roots carry both: a finding
    // is recorded under the path the walk reached it by (§FS-config.3.5.2.1).
    let mut unwalked = unwalked_home_roots(config);
    let canonical = unwalked
        .iter()
        .filter_map(|home| fs::canonicalize(home).ok())
        .collect::<Vec<_>>();
    unwalked.extend(canonical);
    unwalked.sort_by_key(|home| sort_path_key(home));
    unwalked.dedup();
    Ok(Some(ScanScope { roots, unwalked }))
}

/// §FS-check.1.3.4: drop everything the wider `--full` walk read from outside the
/// configured scope, so every rule but the out-of-scope tier sees exactly the
/// tree a run without the flag sees and reports exactly what it reports. A no-op
/// without `--full`. Nothing is cloned — the walk's findings are narrowed in
/// place, after the tier has been read off the whole of them.
///
/// Why there is no undo pass for the shorthand resolutions §AR-scanner.2.6.6 made
/// against the whole walk: the narrowed declarations yield the candidate set
/// `report_shorthand_citation` judges a site against — one predicate covering the
/// unqualified and the cross-member qualified form alike, where an undo pass here
/// could only reach the unqualified one.
pub(crate) fn retain_findings_in_scope(findings: &mut Findings, scope: Option<&ScanScope>) {
    let Some(scope) = scope else { return };
    findings.declarations.retain(|_, decls| {
        decls.retain(|decl| {
            matches!(decl.source, DeclarationSource::Json { .. }) || scope.contains(&decl.file)
        });
        !decls.is_empty()
    });
    findings.citations.retain(|cite| scope.contains(&cite.file));
    findings
        .local_section_citation_candidates
        .retain(|candidate| scope.contains(&candidate.file));
    retain_heading_findings_in_scope(findings, scope);
    findings
        .value_bindings
        .retain(|binding| scope.contains(&binding.file));
    findings
        .invalid_value_bindings
        .retain(|site| scope.contains(&site.file));
    // Home JSON is catalog input under every scope; Markdown value errors obey
    // the ordinary configured scope (§FS-values.2.2, §FS-check.1.3.3).
    findings.invalid_value_declarations.retain(|site| {
        matches!(site.source, DeclarationSource::Json { .. }) || scope.contains(&site.file)
    });
    findings
        .escaped_citations
        .retain(|cite| scope.contains(&cite.file));
    // §FS-declarations.checks.declaration-near-miss asks a question about the configured scope, so
    // a `--full` walk's extra files are dropped with the rest: `--full` widens the *reference* tier
    // (§FS-check.3.14.2) and nothing else.
    findings
        .near_miss_headings
        .retain(|heading| scope.contains(&heading.file));
    findings.scanned_files.retain(|file| scope.contains(file));
    // The shorthand resolutions §AR-scanner.2.6.6 performed against the whole walk
    // are deliberately left standing: a site whose declaration the retains above
    // just dropped is re-judged in `report_shorthand_citation` (§FS-check.3.13).
}

/// §FS-check.3.14.3: the out-of-scope tier — the reference-resolution family run
/// over the citation sites the wider `--full` walk found outside the configured
/// scope, resolved against the *whole* walk so a citation whose declaration is
/// also out there still resolves. Empty without `--full`.
pub(crate) fn out_of_scope_references(
    findings: &Findings,
    config: &Config,
    workspace: &BTreeMap<String, WorkspaceCheckTarget<'_>>,
    scope: Option<&ScanScope>,
) -> Vec<Diagnostic> {
    let Some(scope) = scope else {
        return Vec::new();
    };
    let mut tier = CheckReport::default();
    check_citation_resolution(
        findings,
        config,
        config,
        workspace,
        ReferenceTier::OutOfScope,
        Some(scope),
        &mut tier,
    );
    tier.errors.into_iter().map(tag_out_of_scope).collect()
}

/// §FS-check.1.3.8: the out-of-scope tier for a workspace run — one pass per
/// project, tiered against that project's own `[scan] include`, and resolved
/// against every project's *whole* walk, which is why it runs before the
/// findings are narrowed. `include` is a per-project statement, so a member
/// widens past its own and no other.
pub(crate) fn workspace_out_of_scope_references(
    projects: &[WorkspaceProject],
    scopes: &[Option<ScanScope>],
) -> Vec<Diagnostic> {
    if scopes.iter().all(Option::is_none) {
        return Vec::new();
    }
    let workspace = projects
        .iter()
        .map(|project| {
            (
                project.alias.clone(),
                WorkspaceCheckTarget {
                    findings: &project.findings,
                    config: &project.config,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut diagnostics = Vec::new();
    for (project, scope) in projects.iter().zip(scopes) {
        diagnostics.extend(out_of_scope_references(
            &project.findings,
            &project.config,
            &workspace,
            scope.as_ref(),
        ));
    }
    diagnostics
}

/// §FS-check.3.14: the tier's own code and message shape.
///
/// The code is the in-scope one under an `out-of-scope-` prefix, so a
/// `--format=json` consumer filters the tier by prefix and the rule by exact
/// match on the `code` field the report shape already carries (§FS-errors.5.1) —
/// one code for all four would leave the rule readable only in the prose.
///
/// The tier leads the message rather than trailing it: out here the fix is
/// usually to widen `[scan] include`, so a rule's own fix-it hint ("did you
/// mean …?") is the fact most likely to be wrong and least deserving of being
/// read first.
pub(crate) fn tag_out_of_scope(mut diagnostic: Diagnostic) -> Diagnostic {
    diagnostic.code = match diagnostic.code {
        "dangling" => "out-of-scope-dangling",
        "missing-section" => "out-of-scope-missing-section",
        "local-section-citation" => "out-of-scope-local-section-citation",
        "unknown-project" => "out-of-scope-unknown-project",
        "shorthand-citation" => "out-of-scope-shorthand-citation",
        // `check_citation_resolution` emits exactly the five families above; anything
        // else would be a new rule joining the family without a tier code.
        other => other,
    };
    diagnostic.message = format!("outside [scan] include: {}", diagnostic.message);
    diagnostic
}
