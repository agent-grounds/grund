use crate::sources::{
    canonical_path, is_directory, lookup_refused, path_lookup, resolved_as_far_as_answered,
};
use anyhow::{Result, anyhow};
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

use super::e2e::{e2e_cases_folder, is_direct_e2e_case_dir};
use super::global_excludes::GlobalExcludes;
use super::mounted_ignore::{mounted_ignore_mode, rules_above};
use super::read_predicate::{Unread, directory_refusal, globally_ignored};
use super::walk_boundaries::owned_by_another_project;
pub(super) use super::walk_reporting::walk_scannable_files_reporting;
pub(super) use super::walk_roots::scan_roots;
pub(crate) use super::walk_roots::scan_roots_for;
use crate::config::ExcludeGlobs;
use crate::config::{Frame, Schema, unwalked_homes};
use crate::grammar::Grammar;
use crate::model::{configured_home_path_key, scanned_decl_relative_path};

/// The walker one scan root is traversed with (§AR-scanner.1): the hidden-name and
/// ignore-file rules the builder carries, the global excludes anchored at each
/// entry's repository top (§FS-scan.scope.ignore-files.global), and the workspace
/// boundary, `[scan] exclude`, canonical project-root boundary, unwalked-home and
/// E2E-case prunes [`WalkDirFilter`] carries.
///
/// Built apart from the reporting pass so [`walk_reads_any_file`] uses the
/// same one. That is not a tidiness point: §FS-check.4.10 reports a tree *because*
/// no scan reads it, so a probe that pruned differently from the scan would caution
/// a repository about content the scan would have skipped anyway — the one outcome
/// that finding cannot afford (§DF-unread-opted-out-block).
///
/// `link_roots` and `looping_links` are the filter's two outputs and belong to the
/// caller, because the reporting walk reads both after the traversal; `met` is what
/// else it records of the directories it prunes, for the pass that reads them.
#[allow(clippy::too_many_arguments)]
pub(super) fn scannable_walker(
    schema: &Schema,
    frame: Frame<'_>,
    scan_root: &Path,
    canonical_scan_root: &Path,
    physical_root: &Path,
    link_roots: &std::sync::Arc<std::sync::Mutex<Vec<PathBuf>>>,
    looping_links: &std::sync::Arc<std::sync::Mutex<Vec<(PathBuf, PathBuf)>>>,
    met: Met,
) -> Result<ignore::Walk> {
    Ok(scannable_walk_builder(
        schema,
        frame,
        scan_root,
        canonical_scan_root,
        physical_root,
        link_roots,
        looping_links,
        met,
    )?
    .build())
}

/// The E2E case directories one walk met and left to the manifest pass, as the walk spelled
/// them (§FS-declarations.e2e-case.excluded): the pass reads its cases from this listing rather
/// than listing the cases root again (§FS-check.2.4.2, §AR-scanner.6.4).
pub(super) type MetCases = std::sync::Arc<std::sync::Mutex<Vec<PathBuf>>>;

/// The directory links one walk refused because their canonical target lies outside the project
/// root, as the walk spelled them, for the empty-scan warning to name (§FS-check.2.2.2.6.1).
pub(super) type MetLinks = std::sync::Arc<std::sync::Mutex<Vec<PathBuf>>>;

/// What one walk records of the directories its filter prunes, for a pass that reads them once
/// the traversal ends rather than listing the tree again. A recorder left `None` records nothing:
/// the reporting walk asks for the case directories alone, and only the empty-scan probe's walk,
/// run after a scan that read nothing, asks for the outward links (§GOAL-fast-feedback).
#[derive(Clone, Default)]
pub(super) struct Met {
    cases: Option<MetCases>,
    outward_links: Option<MetLinks>,
}

impl Met {
    /// A walk that records into `cases` the case directories it meets and the configuration does
    /// not keep out (§FS-declarations.e2e-case.excluded).
    pub(super) fn cases(cases: &MetCases) -> Self {
        Self {
            cases: Some(std::sync::Arc::clone(cases)),
            outward_links: None,
        }
    }

    /// A walk that records into `links` every directory link its gate refuses as out of the
    /// project root (§FS-check.2.2.2.6.1).
    pub(super) fn outward_links(links: &MetLinks) -> Self {
        Self {
            cases: None,
            outward_links: Some(std::sync::Arc::clone(links)),
        }
    }
}

/// [`scannable_walker`] before it is built, so a probe that only needs the walk's
/// first levels bounds its depth without a second copy of its rules
/// (§FS-check.2.2.2.5.2). `met` is what the walk records of the directories it prunes.
#[allow(clippy::too_many_arguments)]
pub(super) fn scannable_walk_builder(
    schema: &Schema,
    frame: Frame<'_>,
    scan_root: &Path,
    canonical_scan_root: &Path,
    physical_root: &Path,
    link_roots: &std::sync::Arc<std::sync::Mutex<Vec<PathBuf>>>,
    looping_links: &std::sync::Arc<std::sync::Mutex<Vec<(PathBuf, PathBuf)>>>,
    met: Met,
) -> Result<WalkBuilder> {
    // §FS-check.6.1.3: include ignore discovery outside the source roots.
    if schema.sources.respect_gitignore && !crate::config::observe_ignore_inputs(scan_root) {
        return Err(anyhow!("watch ignore coverage failed"));
    }
    let mut builder = WalkBuilder::new(scan_root);
    builder.hidden(false);
    // §FS-config.3.5.1: a symlink is part of the tree at the path it occupies, so
    // the walk reads through it — a linked file as the file, a linked directory by
    // descending; the entry keeps its in-tree path either way.
    builder.follow_links(true);
    if !schema.sources.respect_gitignore {
        builder
            .ignore(false)
            .git_ignore(false)
            .git_global(false)
            .git_exclude(false)
            .parents(false);
    }
    // §FS-remote-projects.ignore: a mounted root reads the ignore files at and below
    // its own root, git checkout or not, and none above it or the machine's; those
    // above a scan root below it are the filter's to ask, as the global file is.
    let mounted_root = frame.run.workspace.mounted_root.as_deref();
    // §FS-scan.scope.ignore-files.global: no one root the builder could be handed anchors
    // every entry, so its global matcher is off and the global file is asked here, first,
    // where the walker would have asked it: before the filter sees the entry.
    let global = match mounted_root.filter(|_| schema.sources.respect_gitignore) {
        Some(mounted_root) => {
            mounted_ignore_mode(&mut builder);
            rules_above(mounted_root, scan_root)
        }
        None if schema.sources.respect_gitignore => GlobalExcludes::take_over(&mut builder)?,
        None => None,
    };
    let filter = WalkDirFilter::new(
        schema,
        frame,
        scan_root,
        canonical_scan_root,
        physical_root,
        link_roots,
        looping_links,
    );
    let filter = WalkDirFilter { met, ..filter };
    builder.filter_entry(move |entry| {
        !global
            .as_ref()
            .is_some_and(|global| globally_ignored(global, entry))
            && filter.keep(entry)
    });
    Ok(builder)
}

/// The directory filter the walk runs on every entry it meets: the workspace
/// boundary (§AR-workspace.6), hidden names and `[scan] exclude`
/// (§FS-config.3.5.3), and the E2E case directories the manifest pass owns
/// (§AR-scanner.6.4).
///
/// The **name** tests read the in-tree path, so a followed link is pruned under
/// the name it wears in the tree — `docs/node_modules -> ../../node_modules` is
/// excluded exactly as a real directory of that name would be. The canonical
/// project-root fence and the two **ownership** tests read the canonical path as
/// well, because the physical root, a member root, and a case directory are
/// properties of the directory rather than of the name it is reached under
/// (§AR-scanner.1.3): reached through a link they match neither the in-tree
/// prefix, the precomputed suffix, nor the parent compare, and the walk would
/// descend into a namespace that is not its to read.
///
/// `link_roots` is how a link-reached directory is recognized without a syscall
/// per entry. The sequential walker filters a directory before its children, so
/// every directory link the walk meets is recorded here before anything below it
/// is asked about, and a prefix test then answers for the whole subtree; a link
/// already covered by one is not recorded again, so the list holds the disjoint
/// link subtrees and nothing more.
pub(super) struct WalkDirFilter {
    scan_root: PathBuf,
    canonical_scan_root: PathBuf,
    physical_root: PathBuf,
    boundary_suffixes: Vec<PathBuf>,
    boundary_roots: Vec<PathBuf>,
    excluded: Vec<String>,
    /// A v2 project's `[schema.sources] exclude` (§FS-config-v2.schema.sources.exclude);
    /// `None` for v1 and for an empty list.
    globs: Option<ExcludeGlobs>,
    /// The `scan = false` homes this walk prunes, as home path keys
    /// (§FS-config.3.4.7.2). Empty under `--full`, and empty in the tree that
    /// configures no such kind — which is every tree that never pays for the
    /// test below (§GOAL-fast-feedback).
    unwalked_homes: Vec<PathBuf>,
    e2e_cases_root: Option<PathBuf>,
    /// The config root, the run's other project roots, and the ID shape an E2E
    /// case directory is read with: the frame's, owned, because the filter
    /// outlives the walk's borrow of it (§AR-scanner.1).
    root: PathBuf,
    project_roots: Vec<PathBuf>,
    /// The roots of the trees this run imports, which tell a link out of the project
    /// that lands in one from one that lands nowhere the run reads
    /// (§FS-scan.scope.links.owed.fence).
    mounted_roots: Vec<PathBuf>,
    id_format: String,
    grammar: Grammar,
    link_roots: std::sync::Arc<std::sync::Mutex<Vec<PathBuf>>>,
    looping_links: std::sync::Arc<std::sync::Mutex<Vec<(PathBuf, PathBuf)>>>,
    /// Where the walk records each E2E case directory it meets and the configuration does not
    /// keep out, for the manifest pass (§FS-declarations.e2e-case.excluded), and each outward
    /// directory link it refuses, for the empty-scan probe (§FS-check.2.2.2.6.1); records
    /// nothing for the read predicate's filter, which walks nothing.
    met: Met,
}

impl WalkDirFilter {
    /// The filter one scan root is walked with, and the one the read predicate
    /// and walk-error reporting ask of a single path (§AR-scanner.1.13): the same
    /// fields from the same frame, so they cannot prune differently.
    pub(super) fn new(
        schema: &Schema,
        frame: Frame<'_>,
        scan_root: &Path,
        canonical_scan_root: &Path,
        physical_root: &Path,
        link_roots: &std::sync::Arc<std::sync::Mutex<Vec<PathBuf>>>,
        looping_links: &std::sync::Arc<std::sync::Mutex<Vec<(PathBuf, PathBuf)>>>,
    ) -> Self {
        // §AR-workspace.6: precompute the boundary path components once, expressed
        // relative to the canonical scan root, so the walker filter is a single
        // component-suffix compare — no per-entry `canonicalize`, no allocation.
        let boundary_suffixes: Vec<PathBuf> = frame
            .run
            .workspace
            .boundary_roots
            .iter()
            .filter_map(|root| root.strip_prefix(canonical_scan_root).ok())
            .map(Path::to_path_buf)
            .collect();
        Self {
            scan_root: scan_root.to_path_buf(),
            canonical_scan_root: canonical_scan_root.to_path_buf(),
            boundary_suffixes,
            boundary_roots: frame.run.workspace.boundary_roots.clone(),
            excluded: schema.sources.exclude.clone(),
            globs: ExcludeGlobs::of(schema),
            e2e_cases_root: e2e_cases_folder(schema).map(|folder| frame.root().join(folder)),
            physical_root: physical_root.to_path_buf(),
            unwalked_homes: walk_pruned_home_keys(schema, frame, scan_root, physical_root),
            root: frame.root().to_path_buf(),
            project_roots: frame.run.workspace.project_roots.clone(),
            mounted_roots: frame.run.workspace.mounted_roots.clone(),
            id_format: schema.ids.format.clone(),
            grammar: frame.grammar().clone(),
            link_roots: std::sync::Arc::clone(link_roots),
            looping_links: std::sync::Arc::clone(looping_links),
            met: Met::default(),
        }
    }

    /// Whether the walk keeps this entry — and, for a directory, descends into
    /// it. Files are otherwise not filtered here; the extension test is the
    /// caller's.
    ///
    /// Why the unwalked-home prune runs first, and on files too: keeping such a home
    /// out of the roots is only half the rule — the walk meets the same home again as
    /// a descendant of any root above it, the config root or an `include` entry it
    /// sits under, and read there it would be walked after all. It is the one rule
    /// asked of files as well as directories, which is why it is asked before the
    /// directory gate: a single-file home (`file = "docs/template.md"`) is never a
    /// directory to prune, and pruning its parent is not on offer — the parent is
    /// `docs`. Testing the in-tree path rather than the name prunes *this* home and
    /// nothing that merely shares its last component. The list is empty under `--full`
    /// and in every tree that configures no such kind, which is where the cost of
    /// asking per entry would otherwise fall.
    ///
    /// Why the walk stops at a link whose target is at or above the walk root: the
    /// walker compares a target against the directories it is *inside*, and this one
    /// is not one of them, so `docs/up -> ..` sends it down a complete second copy of
    /// the tree before it notices at `docs/up/docs/up`. That descent reports findings
    /// from a tree the run has just called unreadable and reads past `[scan] include`,
    /// so the link is pruned here and the report raised afterwards.
    ///
    /// Every case directory the walk meets is pruned, and `met_cases` records each one the
    /// configuration does not keep out, for the manifest pass to read
    /// (§FS-declarations.e2e-case.excluded). The ignore files have been asked by then, since
    /// the walker asks them before this filter, and `exclude` is asked by `kept_out_by_exclude`.
    ///
    /// Where the empty-scan probe asks for them, `met` also records each directory link the gate
    /// refuses as out of the project root, as the gate's own verdict, so the warning names the
    /// link this walk pruned and judges none again (§FS-check.2.2.2.6.1). A case that links out of
    /// the project is not one of them: the manifest pass reads it, so nothing went unread there.
    fn keep(&self, entry: &ignore::DirEntry) -> bool {
        let is_dir = entry.file_type().is_some_and(is_directory);
        let refusal = self.refusal(entry.path(), entry.depth(), is_dir, entry.path_is_symlink());
        let record = |met: &MetCases| {
            met.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(entry.path().to_path_buf());
        };
        let case = || is_dir && refusal.is_some_and(|reason| self.met_case(entry.path(), reason));
        if let Some(met) = &self.met.cases
            && case()
        {
            record(met);
        }
        // §FS-check.2.2.2.6.1: the gate refuses a directory link before anything below it.
        if let Some(met) = &self.met.outward_links
            && refusal == Some(Unread::OutsideProject)
            && !case()
        {
            record(met);
        }
        refusal.is_none()
    }

    /// Whether the directory at `path`, which `refusal` refused for `reason`, is a case the
    /// manifest pass reads. Only `exclude` and the ignore files keep a case out
    /// (§FS-declarations.e2e-case.excluded). The walk's other prunes stop a descent, and the walk
    /// never descends into a case anyway. So a case that is a link out of the project, or that
    /// stands in a home the walk does not walk, is still read, as it was before the pass read
    /// the walk's listing.
    fn met_case(&self, path: &Path, reason: Unread) -> bool {
        match reason {
            Unread::E2eCase => true,
            // `refusal` answers `UnwalkedHome` before it asks `exclude`, so all three ask it here.
            Unread::OutsideProject | Unread::AnotherProject | Unread::UnwalkedHome => {
                self.is_e2e_case_dir(path) && !self.kept_out_by_exclude(path, true)
            }
            _ => false,
        }
    }

    /// Whether `exclude` keeps the entry at `path` out: a v2 pattern, asked as a file or a
    /// directory by `is_dir`, or, for a directory, a hidden name or a name a version 1
    /// `[scan] exclude` lists. These are the same tests `refusal` asks, and the ones a
    /// looping or dangling link's report asks, which the walker never filters
    /// (§FS-config-v2.schema.sources.exclude.roots).
    pub(super) fn kept_out_by_exclude(&self, path: &Path, is_dir: bool) -> bool {
        self.excluded_by_pattern(path, is_dir)
            || (is_dir && directory_refusal(path, &self.excluded).is_some())
    }

    /// Whether a v2 `exclude` pattern matches `path`, by the spelling the walk reached it under
    /// (§FS-config-v2.schema.sources.exclude.roots).
    fn excluded_by_pattern(&self, path: &Path, is_dir: bool) -> bool {
        self.globs
            .as_ref()
            .is_some_and(|globs| globs.excluding(&self.root, path, is_dir).is_some())
    }

    /// Whether `exclude` keeps out everything this walk would read under the ignore file at
    /// `path`, so that the file governs nothing it reads: what the file's directory holds, or what
    /// this walk's root holds where the root lies below the file. Excluding the file itself, or a
    /// directory above this walk, leaves the file's rules in force on what the walk reads
    /// (§FS-scan.scope.ignore-files.unparsed).
    pub(super) fn governs_nothing_read(&self, path: &Path) -> bool {
        let governed = path
            .parent()
            .filter(|dir| dir.starts_with(&self.scan_root))
            .unwrap_or(&self.scan_root);
        self.globs
            .as_ref()
            .is_some_and(|globs| globs.excludes_all_of(&self.root, &self.scan_root, governed))
    }

    /// Which rule prunes the entry at `path`, `depth` below the scan root, or
    /// `None` where the walk keeps it: `keep` for the walk, and the read
    /// predicate for one path the walk never meets on disk (§AR-scanner.1.13).
    pub(super) fn refusal(
        &self,
        path: &Path,
        depth: usize,
        is_dir: bool,
        is_link: bool,
    ) -> Option<Unread> {
        // §FS-check.6.1.1: linked files/dirs expose targets before descent/read.
        if is_link && !crate::config::observe_input(path, is_dir) {
            return Some(Unread::Unobserved);
        }
        if depth == 0 {
            return None;
        }
        // §FS-config.3.4.7.2: a home the config lists without walking is pruned here as
        // well as left out of `kind_home_roots`, on the in-tree path the way
        // §AR-scanner.2.4.2 decides which home a file is in (§GOAL-fast-feedback).
        if !self.unwalked_homes.is_empty()
            && let Some(relative) =
                scanned_decl_relative_path(path, &self.root, &self.physical_root)
            && self
                .unwalked_homes
                .iter()
                .any(|home| relative.starts_with(home))
        {
            return Some(Unread::UnwalkedHome);
        }
        // §FS-config-v2.schema.sources.exclude.roots: below a root, a glob prunes a
        // file or a directory by the spelling the walk reached it under.
        if self.excluded_by_pattern(path, is_dir) {
            return Some(Unread::ExcludedPath);
        }
        if !is_dir {
            return None;
        }
        let resolved = self.resolved_link_dir(path, is_link);
        let resolved = resolved.as_deref();
        // §FS-scan.scope.links.unresolvable.unopenable: the link's own name first, so a
        // link kept out by it answers as kept out wherever it points.
        if let Some(reason) = directory_refusal(path, &self.excluded) {
            return Some(reason);
        }
        if let Some(reason) = self.scan_boundary_crossed(path, resolved) {
            return Some(reason);
        }
        if self.is_e2e_case_dir(path) || resolved.is_some_and(|path| self.is_e2e_case_dir(path)) {
            return Some(Unread::E2eCase);
        }
        // §FS-config.3.5.5: a directory link whose target is at or above the walk root
        // is a loop, and the one kind the walker cannot see, so it is pruned here and
        // the report raised afterwards from `looping_links` (§AR-scanner.1.9).
        if is_link
            && let Some(resolved) = resolved
            && self.canonical_scan_root.starts_with(resolved)
        {
            self.looping_links
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push((path.to_path_buf(), resolved.to_path_buf()));
            return Some(Unread::LoopingLink);
        }
        None
    }

    /// The canonical path of a directory the walk reached **through** a symlink
    /// — the link itself, or anything below one — and `None` for a directory
    /// reached under its own name, which is the ordinary case and pays no
    /// syscall (§AR-scanner.1.3, §GOAL-fast-feedback).
    fn resolved_link_dir(&self, path: &Path, is_link: bool) -> Option<PathBuf> {
        let mut link_roots = self
            .link_roots
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let covered = link_roots.iter().any(|root| path.starts_with(root));
        if is_link && !covered {
            link_roots.push(path.to_path_buf());
        }
        drop(link_roots);
        // §FS-scan.scope.links.owed.refused: a target the lookup is refused on the way to is
        // placed as far as the disk answers, so the fence still judges where it goes.
        (is_link || covered)
            .then(|| {
                canonical_path(path).ok().or_else(|| {
                    path_lookup(path)
                        .is_err_and(|error| lookup_refused(&error))
                        .then(|| resolved_as_far_as_answered(path))
                })
            })
            .flatten()
    }

    /// A link-reached directory outside the canonical project root is out of
    /// bounds (§FS-config.3.5.1, §AR-scanner.1.2). Loaded-workspace ownership is
    /// stronger: another project remains out of bounds even when it lies inside
    /// this project's physical root (§FS-workspace.6.2, §AR-workspace.6.2). Both are
    /// judged on the canonical target, which `realpath` works out without opening it,
    /// so walk-error reporting asks the same of a link whose target cannot be opened,
    /// and reads from the answer whether anything in the run reads that tree
    /// (§FS-scan.scope.links.owed.fence).
    fn scan_boundary_crossed(&self, path: &Path, resolved: Option<&Path>) -> Option<Unread> {
        if let Ok(relative) = path.strip_prefix(&self.scan_root)
            && self
                .boundary_suffixes
                .iter()
                .any(|suffix| relative == suffix.as_path())
        {
            return Some(Unread::AnotherProject);
        }
        let resolved = resolved?;
        if self
            .boundary_roots
            .iter()
            .any(|root| resolved.starts_with(root))
            || owned_by_another_project(&self.project_roots, &self.physical_root, resolved)
        {
            return Some(Unread::AnotherProject);
        }
        if resolved.starts_with(&self.physical_root) {
            return None;
        }
        let imported = self
            .mounted_roots
            .iter()
            .any(|root| resolved.starts_with(root));
        Some(if imported {
            Unread::AnotherProject
        } else {
            Unread::OutsideProject
        })
    }

    fn is_e2e_case_dir(&self, path: &Path) -> bool {
        is_direct_e2e_case_dir(
            path,
            self.e2e_cases_root.as_deref(),
            &self.id_format,
            &self.grammar,
        )
    }
}

/// The same homes as home *path keys*, for the walk's per-entry test: it
/// compares an in-tree path stripped to the config root, the way the scanner
/// decides which home a file is in (§AR-scanner.2.4.2), so a root reached through
/// a symlink still recognizes them.
///
/// Empty for the two walks that are meant to read such a home. `--full` reaches
/// it like any directory nobody configured (§FS-check.1.3). And a walk whose own
/// root is at or inside one is a path a user typed — `grund check docs/templates`
/// — which reads the directory it names the way an explicit argument already
/// reads past `[scan] include` (§FS-config.3.4.7.3); the key describes the default
/// scope, and `grund check .` resolves to that scope rather than to this branch.
fn walk_pruned_home_keys(
    schema: &Schema,
    frame: Frame<'_>,
    scan_root: &Path,
    physical_root: &Path,
) -> Vec<PathBuf> {
    if frame.run.scope.full {
        return Vec::new();
    }
    let homes = unwalked_homes(schema)
        .map(configured_home_path_key)
        .collect::<Vec<_>>();
    let asked_for_one = scanned_decl_relative_path(scan_root, frame.root(), physical_root)
        .is_some_and(|relative| homes.iter().any(|home| relative.starts_with(home)));
    if asked_for_one { Vec::new() } else { homes }
}
