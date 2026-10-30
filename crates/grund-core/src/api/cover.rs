//! The published `cover` contract and the index behind it (§AR-system.2.9):
//! every citation in the run grouped by the file it sits in, across every
//! project the loader returned, with no output format chosen and no exit code
//! mapped (§FS-cover, §FS-workspace.8.6, §AR-bindings.2). Under coordinate globs
//! each file keeps its row and only the citations they select (§FS-cover.globs).

use anyhow::Result;
use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::config::{Config, display_path};
use crate::grammar::{render_catalog_id, render_citation_id};
use crate::model::{Catalog, Citation, Finding, sort_path_key};
use crate::queries::{GlobSelection, read_globs};
use crate::resolver::{Mounted, WorkspaceContext, load_narrowable_workspace_context};
use crate::scanner::{ApiScanError, api_scan_error};

use super::report::context_run_warnings;

#[derive(Clone)]
#[non_exhaustive]
pub struct CoverOpts {
    pub path: PathBuf,
    pub path_provided: bool,
}

impl Default for CoverOpts {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            path_provided: false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct CoverCitation {
    /// The alias of the project whose tree contains this citation site — the
    /// *citing* project, as in `refs` (§FS-workspace.8.2.4). `None` outside
    /// workspace mode, where the JSON field is omitted entirely so a
    /// single-project repo's bytes are unchanged (§DF-cover-workspace-scope.2.3).
    pub project: Option<String>,
    pub path: String,
    pub line: usize,
    pub column: usize,
    pub id: String,
    pub section: Option<String>,
    pub marker: bool,
    pub text: String,
    /// The nearest declaration whose body contains this **citing** site, or
    /// `None` where the site sits in no declaration body (§FS-cover.3.2). Bare —
    /// rendered under the *citing* project's `[id]` config and never
    /// alias-qualified, because the declaration it names sits in the citing file
    /// and `project` already says which project that is (§FS-workspace.8.6).
    pub enclosing_declaration: Option<String>,
    /// The nearest accepted section path containing this **citing** site,
    /// numbered and named alike, or `None` where no accepted section contains it
    /// (§FS-cover.3.2, §AR-scanner.2.4.4).
    pub enclosing_section: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct CoverEntry {
    /// The alias of the project this scanned file belongs to. A file belongs to
    /// exactly one project by the boundary rule (§FS-workspace.6), so this is
    /// unambiguous. `None` outside workspace mode (§FS-workspace.8.6).
    pub project: Option<String>,
    pub path: String,
    pub citations: Vec<CoverCitation>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct CoverOutput {
    pub output_format: String,
    pub entries: Vec<CoverEntry>,
    pub scan_errors: Vec<ApiScanError>,
    /// The run's warning channel (§FS-distribution.3.1): the three `[workspace]`
    /// cautions of §FS-check.3.29.15, §FS-check.4.10.11 and
    /// §FS-workspace.6.1.7. A frontend renders each as one CLI-level `warning:`
    /// on stderr (§FS-check.2.1.1).
    ///
    /// Three keep the anchor the engine gave them — the `grund.toml` line their
    /// own message already names — and an editor publishes those on that line
    /// (§FS-lsp.1.1.3). §FS-check.3.29.15's is the exception: it carries no
    /// location field at all, states its location inside its own text
    /// (§FS-check.3.29.7), and reaches an editor off `check`'s report instead,
    /// located and as an error (§FS-check.3.29.13).
    pub warnings: Vec<Finding>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct CoverTextCitation {
    pub line: usize,
    pub column: usize,
    pub text: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct CoverTextEntry {
    pub path: String,
    pub citations: Vec<CoverTextCitation>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct CoverTextOutput {
    pub output_format: String,
    pub entries: Vec<CoverTextEntry>,
    pub scan_errors: Vec<ApiScanError>,
    /// The run's warning channel (§FS-distribution.3.1): the three `[workspace]`
    /// cautions of §FS-check.3.29.15, §FS-check.4.10.11 and
    /// §FS-workspace.6.1.7. A frontend renders each as one CLI-level `warning:`
    /// on stderr (§FS-check.2.1.1).
    ///
    /// Three keep the anchor the engine gave them — the `grund.toml` line their
    /// own message already names — and an editor publishes those on that line
    /// (§FS-lsp.1.1.3). §FS-check.3.29.15's is the exception: it carries no
    /// location field at all, states its location inside its own text
    /// (§FS-check.3.29.7), and reaches an editor off `check`'s report instead,
    /// located and as an error (§FS-check.3.29.13).
    pub warnings: Vec<Finding>,
}

/// One scanned file's row in the cover index, with the project that owns it
/// (§FS-workspace.8.6). Borrowed from the [`WorkspaceContext`] the caller holds
/// so both `cover` and `cover_text` build the index once, the same way.
struct CoverRow<'a> {
    /// `None` outside workspace mode — see [`CoverEntry::project`].
    alias: Option<&'a str>,
    /// Rendered against the workspace root in workspace mode, so a member's
    /// file is spelled the way `[workspace] members` spells it
    /// (§FS-workspace.8.6.5).
    path: String,
    citations: Vec<CoverCitationRow<'a>>,
}

struct CoverCitationRow<'a> {
    citation: &'a Citation,
    target_catalog: &'a Catalog,
    citing_catalog: &'a Catalog,
    /// The config the ID renders under: the **target** project's, matching
    /// `refs` (§FS-workspace.8.2.4). Falls back to the citing project's config
    /// when the alias names no loaded project — `cover` reports the graph and
    /// leaves the unknown-alias verdict to `check` (§FS-workspace.8.1).
    target_config: &'a Config,
    /// The config `enclosing_declaration` renders under: the **citing**
    /// project's, because the declaration it names sits in the citing file
    /// (§FS-workspace.8.6). The two differ only where a qualified citation
    /// crosses into a project whose `[id]` grammar spells IDs another way.
    citing_config: &'a Config,
    /// The mount the citing project was grafted through, which holds the namespace
    /// the file wrote (§FS-remote-projects.mount.rule).
    citing_mount: Option<&'a Mounted>,
}

impl CoverCitationRow<'_> {
    fn rendered_id(&self) -> String {
        cited_id(
            self.target_config,
            self.target_catalog,
            self.citation,
            &self.citing_config.marker,
            self.citing_mount,
        )
    }

    /// The citing site's enclosing declaration, bare and rendered under the
    /// citing project's grammar (§FS-cover.3.2, §FS-workspace.8.6).
    fn rendered_enclosing_declaration(&self) -> Option<String> {
        self.citation.enclosing_declaration.as_ref().map(|id| {
            render_catalog_id(
                &self.citing_config.grammar,
                &self.citing_catalog.declarations,
                id,
            )
        })
    }
}

/// The config a citation in the project at `citing` renders its ID under: the
/// **target** project's, matching `refs` (§FS-workspace.8.2.4), or the citing
/// project's where the alias names no loaded project (§FS-workspace.8.1).
pub(super) fn target_config<'a>(
    context: &'a WorkspaceContext,
    citing: usize,
    citation: &Citation,
) -> &'a Config {
    let target = context.citation_target(citing, citation).unwrap_or(citing);
    &context.projects[target].config
}

/// The canonical spelling of the token the file wrote: alias-qualified only
/// when the citation itself was, never qualified on the file's behalf
/// (§FS-workspace.8.6.4). `refs --glob` and `refs --from` spell a record's `id`
/// so too (§FS-refs.3.2). A citation a mount grafted keeps the namespace its file
/// wrote, which `mount` maps back (§FS-remote-projects.mount.rule).
pub(super) fn cited_id(
    target_config: &Config,
    catalog: &Catalog,
    citation: &Citation,
    marker: &str,
    mount: Option<&Mounted>,
) -> String {
    let id = render_citation_id(
        &target_config.grammar,
        &catalog.declarations,
        citation,
        marker,
    );
    match (&citation.namespace, mount) {
        (Some(alias), Some(mount)) => format!("{}/{id}", mount.written_namespace(alias)),
        (Some(alias), None) => format!("{alias}/{id}"),
        (None, _) => id,
    }
}

/// Group every scanned file in the run — every project the loader returned —
/// with every citation in it, qualified or not (§FS-workspace.8.6,
/// §DF-cover-workspace-scope). No consumer-end filter: `cover` is keyed by
/// file, so a dropped row and a file with nothing to say print the same, which
/// is the silent skip §REQ-no-missed-citation.1 forbids. Coordinate globs drop
/// citations, never a row: a file whose citations they all pass over keeps its
/// row, empty (§FS-cover.globs, §AR-resolver.3.2).
fn cover_rows<'a>(
    context: &'a WorkspaceContext,
    globs: Option<&GlobSelection>,
) -> Vec<CoverRow<'a>> {
    // A file lives in exactly one project (§FS-workspace.6), so the index needs
    // no merge across projects; keying by absolute path is enough to keep the
    // per-file grouping and the owning alias in step.
    let mut by_file: BTreeMap<&PathBuf, (usize, Vec<CoverCitationRow>)> = BTreeMap::new();
    for (index, project) in context.projects.iter().enumerate() {
        // §FS-imports.graph.diamond: each file once, under its canonical alias path.
        if !project.reports_sites() {
            continue;
        }
        for file in &project.findings.scanned_files {
            by_file.entry(file).or_insert_with(|| (index, Vec::new()));
        }
        for citation in &project.findings.citations {
            let row = by_file
                .entry(&citation.file)
                .or_insert_with(|| (index, Vec::new()));
            if globs.is_some_and(|globs| !globs.selects_citation(context, index, citation)) {
                continue;
            }
            row.1.push(CoverCitationRow {
                citation,
                target_catalog: &context.projects
                    [context.citation_target(index, citation).unwrap_or(index)]
                .findings,
                citing_catalog: &project.findings,
                target_config: target_config(context, index, citation),
                citing_config: &project.config,
                citing_mount: project.mount.as_ref(),
            });
        }
    }

    let mut rows = by_file
        .into_iter()
        .map(|(file, (index, mut citations))| {
            citations.sort_by_key(|row| (row.citation.line, row.citation.column));
            let project = &context.projects[index];
            let path = if context.workspace_loaded {
                display_path(context.render_config(), file)
            } else {
                display_path(&project.config, file)
            };
            CoverRow {
                alias: context.workspace_loaded.then_some(project.alias.as_str()),
                path,
                citations,
            }
        })
        .collect::<Vec<_>>();
    // §FS-cover.2: sorted by the *rendered* path, so the order a caller reads
    // is the order it can diff against (§FS-errors.4).
    rows.sort_by(|left, right| left.path.cmp(&right.path));
    rows
}

/// Load the projects `cover` indexes and their scan errors, rendered against
/// the same base as the rows (§FS-workspace.8.6).
///
/// The narrowable loader, not the plain one: `cover`'s `<path>` bounds the walk
/// (§FS-cover.1), so a scope inside the workspace root stays one narrowed scan
/// (§AR-resolver.3.3), which `grund check <dir>` no longer is (§FS-check.1.3.6.1).
///
/// §AR-scanner.2.4.2: `cover` publishes each site's enclosing declaration and
/// section (§FS-cover.3.2), so it performs the citing-side post-pass — one of the
/// four runs that do, and the reason the narrowable loader asks for it.
fn cover_context(opts: &CoverOpts) -> Result<WorkspaceContext> {
    load_narrowable_workspace_context(&opts.path, opts.path_provided, &[])
}

/// Every loaded project's scan errors, in project order (§FS-workspace.8.7):
/// a member's unreadable file fails the run at the workspace root, because the
/// index the run just printed is incomplete for the tree it claimed.
///
/// Sorting by path lets a reader comparing `cover` and `check` on one tree read
/// one list twice rather than two interleavings. `sort_path_key` on the
/// unrendered path is how every other path ordering in the crate keys one; a
/// second definition of "path order" here is a thing that drifts.
pub(super) fn cover_scan_errors(context: &WorkspaceContext) -> Vec<ApiScanError> {
    // §FS-errors.4: the same base the rows render against — `render_config`, the
    // workspace root in workspace mode and the only project otherwise. A path
    // spelled against the member names no file from where the run was launched.
    let config = context.render_config();
    let mut errors = context
        .projects
        .iter()
        .flat_map(|project| project.scan_errors.iter())
        .collect::<Vec<_>>();
    // §FS-errors.4.1: by path, not by the order the projects were loaded — the
    // same order `check` prints the same errors in, keyed with `sort_path_key`
    // on the path itself, before rendering.
    errors.sort_by_key(|(path, message)| (sort_path_key(path), message.clone()));
    errors
        .into_iter()
        .map(|(path, message)| api_scan_error(config.frame(), path, message))
        .collect()
}

/// Programmatic `cover`: group every citation by scanned file, across every
/// project the run loaded, without choosing a CLI output format or process exit
/// code (§AR-bindings.2, §FS-workspace.8.6).
pub fn cover(opts: CoverOpts) -> Result<CoverOutput> {
    cover_with_run_warnings(opts).1
}

/// Additive cautions survive a later refusal (§FS-distribution.3.1).
pub fn cover_with_run_warnings(opts: CoverOpts) -> (Vec<Finding>, Result<CoverOutput>) {
    let (cautions, result) = cover_with_globs(opts, &[]);
    (cautions, result.map(|(output, _)| output))
}

/// [`cover_with_run_warnings`] keeping, in each file's row, only the citations the
/// coordinate globs select, with the patterns' note beside the output
/// (§FS-cover.globs, §FS-coordinate-globs.errors.note). No globs select every
/// citation, as `cover` does.
#[doc(hidden)]
pub fn cover_with_globs(
    opts: CoverOpts,
    globs: &[String],
) -> (Vec<Finding>, Result<(CoverOutput, Option<String>)>) {
    let mut cautions = Vec::new();
    let result = cover_run(opts, globs, &mut cautions);
    (cautions, result)
}

/// The run's glob selection, read before discovery and checked against the loaded
/// projects after it (§FS-coordinate-globs.errors.shape); `None` without globs.
fn cover_globs<'g>(
    context: &WorkspaceContext,
    globs: &'g [crate::grammar::CoordinateGlob],
) -> Result<Option<GlobSelection<'g>>> {
    match globs.is_empty() {
        true => Ok(None),
        false => GlobSelection::new(context, globs).map(Some),
    }
}

fn cover_run(
    opts: CoverOpts,
    globs: &[String],
    cautions: &mut Vec<Finding>,
) -> Result<(CoverOutput, Option<String>)> {
    let globs = read_globs(globs)?;
    let context = cover_context(&opts)?;
    *cautions = context_run_warnings(&context);
    let selection = cover_globs(&context, &globs)?;
    let entries = cover_rows(&context, selection.as_ref())
        .into_iter()
        .map(|row| CoverEntry {
            project: row.alias.map(str::to_string),
            citations: row
                .citations
                .iter()
                .map(|citation_row| CoverCitation {
                    project: row.alias.map(str::to_string),
                    // The nested object repeats the row's path so `cover` and
                    // `refs` JSON stay comparable field for field (§FS-cover.3.2).
                    path: row.path.clone(),
                    line: citation_row.citation.line,
                    column: citation_row.citation.column,
                    id: citation_row.rendered_id(),
                    section: citation_row.citation.section.clone(),
                    marker: citation_row.citation.has_marker,
                    text: citation_row.citation.text.clone(),
                    // §FS-cover.3.2: the citing-side counterpart of `id` and
                    // `section`, after `text`.
                    enclosing_declaration: citation_row.rendered_enclosing_declaration(),
                    enclosing_section: citation_row.citation.enclosing_section.clone(),
                })
                .collect(),
            path: row.path,
        })
        .collect();
    let output = CoverOutput {
        output_format: context.render_config().output_format.clone(),
        entries,
        scan_errors: cover_scan_errors(&context),
        warnings: context_run_warnings(&context),
    };
    Ok((output, selection.and_then(|selection| selection.note())))
}

/// Programmatic text-oriented `cover`: return only the citation fields needed
/// for the default human-readable cover view while still leaving rendering to
/// frontends (§AR-bindings.2). Same index as [`cover`] (§FS-workspace.8.6);
/// the text view carries no alias because the path already renders from the
/// workspace root and the token is printed verbatim (§FS-cover.3.1).
pub fn cover_text(opts: CoverOpts) -> Result<CoverTextOutput> {
    cover_text_with_run_warnings(opts).1
}

/// Additive cautions survive a later refusal (§FS-distribution.3.1).
pub fn cover_text_with_run_warnings(opts: CoverOpts) -> (Vec<Finding>, Result<CoverTextOutput>) {
    let (cautions, result) = cover_text_with_globs(opts, &[]);
    (cautions, result.map(|(output, _)| output))
}

/// [`cover_text_with_run_warnings`] under coordinate globs, as [`cover_with_globs`]
/// selects them (§FS-cover.globs).
pub(super) fn cover_text_with_globs(
    opts: CoverOpts,
    globs: &[String],
) -> (Vec<Finding>, Result<(CoverTextOutput, Option<String>)>) {
    let mut cautions = Vec::new();
    let result = cover_text_run(opts, globs, &mut cautions);
    (cautions, result)
}

fn cover_text_run(
    opts: CoverOpts,
    globs: &[String],
    cautions: &mut Vec<Finding>,
) -> Result<(CoverTextOutput, Option<String>)> {
    let globs = read_globs(globs)?;
    let context = cover_context(&opts)?;
    *cautions = context_run_warnings(&context);
    let selection = cover_globs(&context, &globs)?;
    let entries = cover_rows(&context, selection.as_ref())
        .into_iter()
        .map(|row| CoverTextEntry {
            path: row.path,
            citations: row
                .citations
                .iter()
                .map(|citation_row| CoverTextCitation {
                    line: citation_row.citation.line,
                    column: citation_row.citation.column,
                    text: citation_row.citation.text.clone(),
                })
                .collect(),
        })
        .collect();
    let output = CoverTextOutput {
        output_format: context.render_config().output_format.clone(),
        entries,
        scan_errors: cover_scan_errors(&context),
        warnings: context_run_warnings(&context),
    };
    Ok((output, selection.and_then(|selection| selection.note())))
}
