//! Configured-scope narrowing and the out-of-scope reference tier
//! (§AR-core-module-layout.1.5, §FS-check.1.3, §FS-check.3.14).

use anyhow::Result;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::references::{ReferenceTier, check_citation_resolution};
use super::sections::retain_heading_findings_in_scope;
use crate::config::{Frame, Schema, unwalked_home_roots};
use crate::model::{Catalog, CheckReport, DeclarationSource, Diagnostic, sort_path_key};
use crate::resolver::{WorkspaceCheckTarget, WorkspaceProject};
use crate::scanner::{CANONICAL_AGENT_ENTRYPOINT, COMPANION_AGENT_ENTRYPOINTS, scan_roots_for};
use crate::workspace::scope_is_config_root;

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
    pub(crate) fn contains(&self, path: &Path) -> bool {
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
    schema: &Schema,
    frame: Frame<'_>,
    path: &Path,
    path_provided: bool,
    full: bool,
) -> Result<Option<ScanScope>> {
    if !full {
        return Ok(None);
    }
    let mut roots = scan_roots_for(schema, frame, Some(path), path_provided, false, false)?;
    let canonical = roots
        .iter()
        .filter_map(|root| fs::canonicalize(root).ok())
        .collect::<Vec<_>>();
    roots.extend(canonical);
    roots.sort_by_key(|root| sort_path_key(root));
    roots.dedup();
    // Both spellings again, for the same reason the roots carry both: a finding
    // is recorded under the path the walk reached it by (§FS-config.3.5.2.1).
    let mut unwalked = unwalked_home_roots(schema, frame.root());
    let canonical = unwalked
        .iter()
        .filter_map(|home| fs::canonicalize(home).ok())
        .collect::<Vec<_>>();
    unwalked.extend(canonical);
    unwalked.sort_by_key(|home| sort_path_key(home));
    unwalked.dedup();
    Ok(Some(ScanScope { roots, unwalked }))
}

/// §FS-check.1.3.6.1: the **report** scope of this run — the set of files a finding
/// may be about. It is exactly the explicit path, and `None` for a run over the
/// config root, which narrows nothing and whose report is the whole walk.
///
/// The twin of [`configured_scope`] and deliberately not the same thing
/// (§AR-resolver.3.3). That one is read *before* any rule runs, so `--full` stays
/// additive; this one is applied *after* every rule has run, which is what lets a
/// path-scoped run resolve against the whole project and still answer about the
/// path alone. Both spellings of each root are kept for the reason they are there,
/// and `unwalked` is empty on purpose: the report scope is the path the caller
/// typed, even where that path is a kind home the ordinary walk prunes
/// (§FS-config.3.4.7.3).
pub(crate) fn path_report_scope(
    schema: &Schema,
    frame: Frame<'_>,
    path: &Path,
    path_provided: bool,
) -> Result<Option<ScanScope>> {
    if scope_is_config_root(frame.root(), path, path_provided) {
        return Ok(None);
    }
    let mut roots = scan_roots_for(schema, frame, Some(path), path_provided, false, false)?;
    let canonical = roots
        .iter()
        .filter_map(|root| fs::canonicalize(root).ok())
        .collect::<Vec<_>>();
    roots.extend(canonical);
    roots.sort_by_key(|root| sort_path_key(root));
    roots.dedup();
    Ok(Some(ScanScope {
        roots,
        unwalked: Vec::new(),
    }))
}

/// §FS-check.1.3.6.1: narrow a channel of the report to the report scope, after
/// every rule has run over the wider resolution scope. A no-op for a run over the
/// config root.
///
/// Three things survive the narrowing. A run-level finding carries no path — the
/// config findings, the scope cautions, the workspace run warnings — and the
/// filter never sees one. A finding that spans several sites is in scope at any of
/// them (§FS-check.1.3.6.2), which is what reports a duplicate declaration from
/// either twin and a value mismatch from the declaring side; the diagnostic is
/// kept whole rather than re-anchored, because re-anchoring would print a line
/// `grund check .` never prints. And the agent-entrypoint probe of §FS-check.3.5
/// asks about the project root rather than about a scanned file, and already
/// reports when no source file is scanned at all; `AGENTS.md` lies outside every
/// path but the root, so a blanket filter would delete the one diagnostic a narrow
/// run still owes about the root.
///
/// The probe is exempt as *that finding* and not as a file: the exemption tests the
/// `code`, so an ordinary dangling, style or declaration finding about `AGENTS.md`
/// is dropped like any other outside the path. Testing the filename instead leaked
/// the whole rule set over the entrypoint, and an entrypoint inside `[scan] include`
/// is the ordinary configuration — this repository's own `grund.toml` writes it.
pub(crate) fn retain_diagnostics_in_report_scope(
    diagnostics: &mut Vec<Diagnostic>,
    frame: Frame<'_>,
    scope: Option<&ScanScope>,
) {
    let Some(scope) = scope else { return };
    let entrypoints = agent_entrypoint_paths(frame);
    diagnostics.retain(|diagnostic| match &diagnostic.path {
        None => true,
        Some(path) => {
            scope.contains(path)
                // §FS-check.1.3.6.2: the anchor may be the site the caller did not type.
                || diagnostic.sites.iter().any(|site| scope.contains(&site.path))
                // §FS-rules.9.1.1: a stale block, chapter rules included, reaches a narrowed run.
                || (diagnostic.code == AGENTS_INIT_CODE
                    && entrypoints.iter().any(|entrypoint| entrypoint == path))
        }
    });
}

/// The §FS-check.3.5 probe's one code (`checker/agents.rs`), which is what the
/// report filter exempts rather than the files the probe looks at.
const AGENTS_INIT_CODE: &str = "agents-init";

/// §FS-check.1.3.6.1: whether the walk read any file the report scope owns — what
/// the §FS-check.2.2 empty-scan caution asks, since the cautions are computed
/// against the report scope and not the resolution scope. A path holding no
/// scannable file still earns its caution however much the wider walk read.
pub(crate) fn scope_read_any_file(findings: &Catalog, scope: Option<&ScanScope>) -> bool {
    match scope {
        None => !findings.scanned_files.is_empty(),
        Some(scope) => findings
            .scanned_files
            .iter()
            .any(|file| scope.contains(file)),
    }
}

/// The files the §FS-check.3.5 probe reports about, derived from the one table that
/// spells the supported agent set (§FS-init.2.1.1) rather than restated beside it.
/// Names only, so this costs no walk: the question is which paths that check *may*
/// have anchored a diagnostic at, and a file it never looked at anchors nothing.
fn agent_entrypoint_paths(frame: Frame<'_>) -> Vec<PathBuf> {
    std::iter::once(CANONICAL_AGENT_ENTRYPOINT)
        .chain(
            COMPANION_AGENT_ENTRYPOINTS
                .iter()
                .map(|entrypoint| entrypoint.rel),
        )
        .map(|rel| frame.root().join(rel))
        .collect()
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
pub(crate) fn retain_findings_in_scope(findings: &mut Catalog, scope: Option<&ScanScope>) {
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
    // §FS-check.checks.glob-citation: narrowed to the configured scope like the rest.
    findings
        .glob_citations
        .retain(|pattern| scope.contains(&pattern.file));
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
    findings: &Catalog,
    schema: &Schema,
    frame: Frame<'_>,
    workspace: &BTreeMap<String, WorkspaceCheckTarget<'_>>,
    scope: Option<&ScanScope>,
) -> Vec<Diagnostic> {
    let Some(scope) = scope else {
        return Vec::new();
    };
    let mut tier = CheckReport::default();
    check_citation_resolution(
        findings,
        schema,
        frame,
        workspace,
        // §FS-check.3.14.1: out of scope, a `should` never demotes a dangling error.
        &|_, _| None,
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
                WorkspaceCheckTarget::of(
                    &project.findings,
                    project.config.schema(),
                    project.config.frame(),
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut diagnostics = Vec::new();
    for (project, scope) in projects.iter().zip(scopes) {
        diagnostics.extend(out_of_scope_references(
            &project.findings,
            project.config.schema(),
            project.config.frame(),
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
/// one code for all five would leave the rule readable only in the prose.
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
