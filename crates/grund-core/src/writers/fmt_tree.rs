//! Formatter tree and scope orchestration (§AR-core-module-layout.1.5),
//! including automatic cross-reference enablement (§FS-fmt.6.6). Per-file and
//! per-line rewriting remains in `fmt_rewrite.rs`.

use anyhow::Result;
use std::path::Path;

use super::fmt_complete_findings::{CompleteFindings, CompleteScan};
use super::fmt_plan::{PlannedFile, TreePlan, WalkRead, partial_io_error};
use super::fmt_rewrite::{FmtLineOpts, rewrite_file};
use super::fmt_row::FmtRow;
use super::fmt_section_lock::{lock_targets, section_lock_changes};
use super::fmt_settle::settle_headings;
use crate::checker::{KindIndexEntries, KindIndexFiles};
use crate::config::{Config, display_path, fmt_excluded};
use crate::grammar::FMT_DIRECTIVE;
use crate::model::Catalog;
use crate::resolver::{SettledHeadings, ShorthandTargets, WorkspaceContext, index_link_targets};
use crate::scanner::{
    ApiScanError, api_scan_error, may_give_locked_heading, walk_scannable_files,
    walk_scannable_files_reporting,
};

/// §FS-fmt.6.6: whether this invocation turns the cross-reference pass on by
/// itself — `[fmt.cross_refs] enabled` and at least one Markdown file in its
/// scope, identically for dry-run and write mode.
pub(crate) fn auto_cross_refs_for_scope(
    config: &Config,
    scope: Option<&Path>,
    explicit_scope: bool,
) -> Result<bool> {
    if !config.fmt_cross_refs_enabled {
        return Ok(false);
    }
    scope_contains_markdown(config, scope, explicit_scope)
}

fn scope_contains_markdown(
    config: &Config,
    scope: Option<&Path>,
    explicit_scope: bool,
) -> Result<bool> {
    Ok(
        walk_scannable_files(config.schema(), config.frame(), scope, explicit_scope)?
            .iter()
            .any(|path| config.reading(path).markdown),
    )
}

/// What one `fmt` walk produced (§FS-fmt.3). Every path here is already rendered
/// against the run's config, because that is where the config naming it is at
/// hand; the command surface prints them and maps the exit code.
pub(crate) struct FmtTreeOutcome {
    /// The lines it rewrote — or, in a dry run, would have — each with what it
    /// did there (§FS-fmt.3.5.1, §FS-fmt.3.7).
    pub(crate) changes: Vec<FmtRow>,
    /// The paths it could not read at all (§FS-check.2.4).
    pub(crate) scan_errors: Vec<ApiScanError>,
    /// The files it read and would not rewrite, because a link reaches them from
    /// outside the config root (§FS-fmt.2.3.2). Named in both modes: `--write`
    /// did not write them, and the dry run is saying `--write` will not.
    pub(crate) refused_writes: Vec<String>,
}

/// What `fmt_tree` rewrites and against what context — grouped so the walk
/// inputs (config + scope) and the rewrite knobs travel separately.
pub(crate) struct FmtRunOpts<'a> {
    pub(crate) add_marker: bool,
    pub(crate) cross_refs: bool,
    pub(crate) write: bool,
    /// The config every path in the *report* is rendered against — the workspace
    /// root's, where `check` renders too (§FS-fmt.3). Each project is walked and
    /// rewritten under its own config, and rendering against that one instead
    /// spelled a member's file from the member root: `docs/FS-003.md` for
    /// `packages/sub/docs/FS-003.md`, which is a different real file in the same
    /// run's output.
    pub(crate) render: &'a Config,
    pub(crate) workspace: Option<&'a WorkspaceContext>,
    /// Whole-project findings the caller has already produced, carrying the
    /// proof that the scan making them met no error (§FS-fmt.7.4) — a
    /// workspace-root `fmt` reuses each project's `WorkspaceContext` scan this
    /// way. There is no way to put an unproven set here, which is the point:
    /// see `complete_findings`. `None` falls back to a complete scan inside
    /// `fmt_tree`, whose errors become one structured strict abort rather than
    /// a partial result.
    pub(crate) precomputed_findings: Option<CompleteFindings<'a>>,
    /// §FS-fmt.6.1.1 / §DF-index-always-linkified: run the cross-reference pass on
    /// a kind's index file even where `[fmt.cross_refs] enabled = false` turned
    /// `cross_refs` off. It decides *which files* the pass touches when the pass
    /// runs at all; dry-run and write mode both enable this carve-out so the
    /// former previews the exact index-entry wraps the latter applies.
    pub(crate) index_cross_refs: bool,
    /// The heading lines a workspace run leaves in every project, settled before
    /// its first project is walked, since a link crosses into a project its own
    /// walk has not reached yet (§FS-fmt.6.3.3). `None` settles this walk's own.
    pub(crate) settled: Option<&'a SettledHeadings>,
}

/// Walk the tree and rewrite each scannable file line by line — never touching a
/// declaration heading or anything inside a fenced code block (§FS-fmt.2.3) — and
/// either write the changes back (`--write`) or just collect `(path, line, label)`
/// for `--check`/dry-run (§FS-fmt.3). `--cross-refs` needs the full `Catalog` first
/// so a link is only emitted when its target resolves (§FS-fmt.6.4).
///
/// Why the link pass takes the whole project's declarations and the shorthand
/// pass does not: a wrap's URL is computed from the declaration's home file,
/// which may live anywhere in the project tree, so workspace mode reuses the
/// caller's whole-project findings — the `WorkspaceContext` scan, no second disk
/// pass per project — and falls back to a strict project scan otherwise. That
/// fallback preserves the link target (§AR-system.2.10) for `grund fmt --cross-refs path/to/file.md`,
/// where the caller's findings are scope-narrow and a cross-file home would
/// otherwise be invisible. A shorthand needs the same declaration set, but only
/// where the tree actually contains one, and paying for a scan on every run of
/// every numbered repo to serve a rewrite most of them never need is the wrong
/// trade (§GOAL-fast-feedback): the walk starts without findings and scans on the
/// first candidate it meets.
///
/// Why the unreadable paths are collected here: they are rendered while the
/// config that names them is at hand, and `fmt` walks the tree `check` walks —
/// the alternative is the one command that edits files in place also being the
/// one that will not say which files it never saw.
///
/// Why a file reached from outside the config root is refused: editing it would
/// put this project's bytes into a file the project does not own. The refusal is
/// named once, on stderr, with the exit code untouched — it is intended behavior
/// and not a failure of the run. The dry run refuses it too and reports no
/// rewrite for it: a dry run predicts what `--write` does, and a pending rewrite
/// `--write` will never perform is one no edit can clear, so `fmt --check` would
/// exit `1` on this tree forever and a gate built on it could never pass.
///
/// Why nothing is written until the walk and the lock pass have both returned
/// (§FS-fmt.7.4.3): the scan a cross-reference, a shorthand or the lock pass
/// needs is taken where the run first needs it, so a refusal can come after the
/// walk has worked out earlier files. Planning every write first puts each such
/// refusal, and any a later pass adds, before the first write by construction,
/// where a guard at each call site is the shape §FS-fmt.7.4.1 records failing.
/// The lock pass then judges the tree the plan leaves, not the one the walk
/// read (§FS-section-lock.write.same-run).
pub(crate) fn fmt_tree(
    config: &Config,
    scope: Option<&Path>,
    explicit_scope: bool,
    opts: &FmtRunOpts<'_>,
) -> Result<FmtTreeOutcome> {
    let plan = plan_tree(config, scope, explicit_scope, opts)?;
    // §FS-section-lock.write.rules: a run over the whole project brings the locks along.
    let locks = match plan.judged(config, opts.render)? {
        // One project's walk leaves every other project as it was loaded.
        Some(judged) => section_lock_changes(&judged, &lock_targets(opts.workspace, &[])),
        None => Vec::new(),
    };
    plan.write(config, opts.render, locks, opts.write)
}

/// The walk of [`fmt_tree`], up to the first write (§FS-fmt.7.4.3): every file's
/// rewrite worked out, in either mode, and nothing written. A workspace run
/// plans every project this way before it judges any project's locks
/// (§FS-section-lock.write.same-run).
pub(super) fn plan_tree<'a>(
    config: &Config,
    scope: Option<&Path>,
    explicit_scope: bool,
    opts: &FmtRunOpts<'a>,
) -> Result<TreePlan<'a>> {
    let mut changes = Vec::new();
    let mut refused_writes = Vec::new();
    let mut planned: Vec<PlannedFile> = Vec::new();
    let mut relative_seen = false;
    let add_marker = opts.add_marker;
    let cross_refs = opts.cross_refs;
    let workspace = opts.workspace;
    let precomputed_findings = opts.precomputed_findings.map(CompleteFindings::findings);
    // §FS-fmt.6.3: the link pass needs the whole project's declarations, because
    // a wrap's URL comes from a home file that may sit outside the rewrite scope.
    // §FS-fmt.2.4.5's scan is deferred instead, to the first shorthand candidate.
    let walked =
        walk_scannable_files_reporting(config.schema(), config.frame(), scope, explicit_scope)?;
    // §FS-fmt.2.5.1: the files this config takes out of every rewrite. The walk
    // above is untouched — only what happens to each file's bytes changes.
    let excluded = fmt_excluded(config)?;
    // §FS-fmt.6.1.1: where the always-linkify carve-out may fire. Built for every
    // run that could need it, since §FS-fmt.2.5.3 makes it outrank a suppressed
    // scope too — a handful of configured paths, so nothing is deferred here.
    let index_files = if opts.index_cross_refs {
        KindIndexFiles::new(config.schema(), config.frame())
    } else {
        KindIndexFiles::empty()
    };
    let index_in_scope =
        !index_files.is_empty() && walked.files.iter().any(|path| index_files.contains(path));
    let link_pass = cross_refs || index_in_scope;
    let owned_findings = if link_pass && precomputed_findings.is_none() {
        Some(CompleteScan::of_tree_or_abort(config, opts.render)?)
    } else {
        None
    };
    let mut findings: Option<&Catalog> = if link_pass {
        precomputed_findings.or(owned_findings.as_ref().map(CompleteScan::findings))
    } else {
        precomputed_findings
    };
    // §FS-fmt.6.1.2: which IDs each index owes an entry for. A pass over the
    // declarations, so it waits for the first file that asks (§GOAL-fast-feedback)
    // — an index in the walk forced the link pass, so they are there by then.
    let mut index_entries: Option<KindIndexEntries> = None;
    // Holds the scan the shorthand pass triggers, so the borrow in `findings`
    // outlives the file that asked for it, and the plan keeps it for the locks.
    let mut shorthand_findings: Option<CompleteScan> = None;
    // §FS-fmt.2.4.5: built once for the whole walk, not once per line — see
    // `ShorthandTargets`. Rebuilt at most once, when the deferred scan lands.
    let mut shorthand_targets = ShorthandTargets::new(config.frame(), findings, workspace);
    // §FS-fmt.6.3.3: every anchor the link pass writes names its heading as this
    // run leaves it, so the heading lines are settled before the first is written.
    let own_settled = match (opts.settled, findings) {
        (None, Some(findings)) if link_pass => settle_headings(
            config,
            walked
                .files
                .iter()
                .filter(|path| !walked.outside_root.contains(*path)),
            &excluded,
            add_marker,
            findings,
            workspace,
            &shorthand_targets,
        ),
        _ => SettledHeadings::default(),
    };
    let settled = opts.settled.unwrap_or(&own_settled);
    // §FS-fmt.3.1: the paths this walk could not read, rendered here while the
    // config that names them is at hand — the same account `check` owes of the
    // tree it walks (§FS-check.2.4).
    let scan_errors: Vec<ApiScanError> = walked
        .errors
        .iter()
        .map(|(file, message)| api_scan_error(opts.render.frame(), file, message))
        .collect();
    for path in walked.files {
        // §FS-fmt.2.3.2: this file was reached through a link that leaves the
        // config root, so the rewrite stops here (§REQ-no-data-loss.2). The dry
        // run refuses it too, and reports no rewrite for it (§FS-fmt.3).
        if walked.outside_root.contains(&path) {
            refused_writes.push(display_path(opts.render, &path));
            continue;
        }
        // §FS-fmt.7.4.3: nothing is written yet, so a file the walk cannot read
        // ends the run with no change to report, in either mode.
        // §AR-system.2.13: the read goes through the run's sources; only the writes are ours.
        let original = config.frame().sources().text(&path).map_err(|error| {
            partial_io_error(
                opts.render,
                &path,
                "read",
                error,
                &[],
                &scan_errors,
                &refused_writes,
            )
        })?;
        // §FS-config-v2.schema.sources: each file is read as its language is.
        let reading = config.reading(&path);
        let is_md = reading.markdown;
        // §FS-fmt.2.5.1: an excluded file keeps its bytes, so the ordinary pass
        // is off here — every rewrite with it (§FS-fmt.2.5).
        let file_excluded = excluded.contains(&path);
        // §FS-fmt.6.1.1, §FS-fmt.2.5.3: an index's entries are linkified whatever
        // turned the ordinary pass off here — `enabled = false`, the exclude
        // list, or a region the text carries. The entries, not the page.
        let carve_out = index_files.contains(&path)
            && (!cross_refs || file_excluded || original.contains(FMT_DIRECTIVE));
        if carve_out
            && index_entries.is_none()
            && let Some(findings) = findings
        {
            // §AR-checker.2.16: the enrollment destination, as `Expected` carries it.
            let index_targets = index_link_targets(
                &config.project().presentation,
                config.schema(),
                config.frame(),
                findings,
                settled,
            );
            index_entries = Some(KindIndexEntries::new(
                findings,
                config.schema(),
                config.frame(),
                &index_targets,
            ));
        }
        let index_entry_ids = carve_out
            .then(|| index_entries.as_ref().and_then(|it| it.entries_in(&path)))
            .flatten();
        let cross_refs = cross_refs && !file_excluded;
        let file_changes_start = changes.len();
        let mut rewritten = rewrite_file(
            &original,
            &path,
            config,
            reading,
            &FmtLineOpts {
                add_marker,
                cross_refs,
                excluded: file_excluded,
                index_entry_ids,
                findings,
                workspace,
                shorthand_targets: &shorthand_targets,
                settled,
            },
            &mut changes,
        );
        // §FS-fmt.2.4.5: a shorthand to expand and no declarations yet. Scan once,
        // then redo *this* file — every file already walked is final, because
        // having no candidate is exactly why the scan had not happened by then.
        if rewritten.saw_shorthand_candidate && findings.is_none() {
            shorthand_findings = Some(CompleteScan::of_tree_or_abort(config, opts.render)?);
            findings = shorthand_findings.as_ref().map(CompleteScan::findings);
            shorthand_targets = ShorthandTargets::new(config.frame(), findings, workspace);
            changes.truncate(file_changes_start);
            rewritten = rewrite_file(
                &original,
                &path,
                config,
                reading,
                &FmtLineOpts {
                    add_marker,
                    cross_refs,
                    excluded: file_excluded,
                    index_entry_ids,
                    findings,
                    workspace,
                    shorthand_targets: &shorthand_targets,
                    settled,
                },
                &mut changes,
            );
        }
        // §FS-fmt.7.3: kept in either mode, so a dry run judges the tree `--write` leaves.
        let text = rewritten.changed.then(|| {
            let mut text = rewritten.lines.join("\n");
            if original.ends_with('\n') {
                text.push('\n');
            }
            text
        });
        // §FS-section-lock.write.same-run: asked of the text the run leaves.
        let markdown = |target: &Path| config.reading(target).markdown;
        let left = text.as_deref().unwrap_or(&original);
        relative_seen = relative_seen
            || is_md && may_give_locked_heading(config.frame(), &path, left, &markdown);
        if let Some(text) = text {
            // §FS-fmt.7.4.3: planned here, written once every refusal is behind the run.
            planned.push(PlannedFile {
                path,
                text,
                changes: file_changes_start..changes.len(),
            });
        }
    }
    let read = match (precomputed_findings, owned_findings.or(shorthand_findings)) {
        (Some(given), _) => WalkRead::Given(given),
        (None, Some(scan)) => WalkRead::Scanned(scan),
        (None, None) => WalkRead::Unread,
    };
    Ok(TreePlan {
        outcome: FmtTreeOutcome {
            changes,
            scan_errors,
            refused_writes,
        },
        planned,
        relative_seen,
        root_scope: walked.root_scope,
        read,
    })
}
