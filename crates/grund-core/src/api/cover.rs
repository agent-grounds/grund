//! The published `cover` contract and the index behind it (§AR-system.2.9):
//! every citation in the run grouped by the file it sits in, across every
//! project the loader returned, with no output format chosen and no exit code
//! mapped (§FS-cover, §FS-workspace.8.6, §AR-bindings.2).

use anyhow::Result;
use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::config::{Config, display_path};
use crate::grammar::render_id;
use crate::model::{Citation, sort_path_key};
use crate::resolver::{WorkspaceContext, load_narrowable_workspace_context};
use crate::scanner::{ApiScanError, api_scan_error};

#[derive(Clone)]
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
pub struct CoverCitation {
    /// The alias of the project whose tree contains this citation site — the
    /// *citing* project, as in `refs` (§FS-workspace.8.2). `None` outside
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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoverEntry {
    /// The alias of the project this scanned file belongs to. A file belongs to
    /// exactly one project by the boundary rule (§FS-workspace.6), so this is
    /// unambiguous. `None` outside workspace mode (§FS-workspace.8.6).
    pub project: Option<String>,
    pub path: String,
    pub citations: Vec<CoverCitation>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CoverOutput {
    pub output_format: String,
    pub entries: Vec<CoverEntry>,
    pub scan_errors: Vec<ApiScanError>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoverTextCitation {
    pub line: usize,
    pub column: usize,
    pub text: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoverTextEntry {
    pub path: String,
    pub citations: Vec<CoverTextCitation>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CoverTextOutput {
    pub output_format: String,
    pub entries: Vec<CoverTextEntry>,
    pub scan_errors: Vec<ApiScanError>,
}

/// One scanned file's row in the cover index, with the project that owns it
/// (§FS-workspace.8.6). Borrowed from the [`WorkspaceContext`] the caller holds
/// so both `cover` and `cover_text` build the index once, the same way.
struct CoverRow<'a> {
    /// `None` outside workspace mode — see [`CoverEntry::project`].
    alias: Option<&'a str>,
    /// Rendered against the workspace root in workspace mode, so a member's
    /// file is spelled the way `[workspace] members` spells it
    /// (§FS-workspace.8.6).
    path: String,
    citations: Vec<CoverCitationRow<'a>>,
}

struct CoverCitationRow<'a> {
    citation: &'a Citation,
    /// The config the ID renders under: the **target** project's, matching
    /// `refs` (§FS-workspace.8.2). Falls back to the citing project's config
    /// when the alias names no loaded project — `cover` reports the graph and
    /// leaves the unknown-alias verdict to `check` (§FS-workspace.8.1).
    target_config: &'a Config,
}

impl CoverCitationRow<'_> {
    /// The canonical spelling of the token the file wrote: alias-qualified only
    /// when the citation itself was, never qualified on the file's behalf
    /// (§FS-workspace.8.6).
    fn rendered_id(&self) -> String {
        let id = render_id(self.target_config, &self.citation.id);
        match &self.citation.namespace {
            Some(alias) => format!("{alias}/{id}"),
            None => id,
        }
    }
}

/// Group every scanned file in the run — every project the loader returned —
/// with every citation in it, qualified or not (§FS-workspace.8.6,
/// §DF-cover-workspace-scope). No consumer-end filter: `cover` is keyed by
/// file, so a dropped row and a file with nothing to say print the same, which
/// is the silent skip §REQ-no-missed-citation.1 forbids.
fn cover_rows(context: &WorkspaceContext) -> Vec<CoverRow<'_>> {
    // A file lives in exactly one project (§FS-workspace.6), so the index needs
    // no merge across projects; keying by absolute path is enough to keep the
    // per-file grouping and the owning alias in step.
    let mut by_file: BTreeMap<&PathBuf, (usize, Vec<CoverCitationRow>)> = BTreeMap::new();
    for (index, project) in context.projects.iter().enumerate() {
        for file in &project.findings.scanned_files {
            by_file.entry(file).or_insert_with(|| (index, Vec::new()));
        }
        for citation in &project.findings.citations {
            let target_config = citation
                .namespace
                .as_deref()
                .and_then(|alias| context.project_by_alias(alias))
                .map(|target| &target.config)
                .unwrap_or(&project.config);
            by_file
                .entry(&citation.file)
                .or_insert_with(|| (index, Vec::new()))
                .1
                .push(CoverCitationRow {
                    citation,
                    target_config,
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
/// the way `grund check <dir>` does.
///
/// §AR-scanner.2.4: `cover` groups citations by file and never reads
/// citing-side classification — skip the scan post-pass (§AR-benchmarks).
fn cover_context(opts: &CoverOpts) -> Result<WorkspaceContext> {
    load_narrowable_workspace_context(&opts.path, opts.path_provided)
}

/// Every loaded project's scan errors, in project order (§FS-workspace.8.7):
/// a member's unreadable file fails the run at the workspace root, because the
/// index the run just printed is incomplete for the tree it claimed.
///
/// Sorting by path lets a reader comparing `cover` and `check` on one tree read
/// one list twice rather than two interleavings. `sort_path_key` on the
/// unrendered path is how every other path ordering in the crate keys one; a
/// second definition of "path order" here is a thing that drifts.
fn cover_scan_errors(context: &WorkspaceContext) -> Vec<ApiScanError> {
    // §FS-errors.4: the same base the rows render against — `render_config`, the
    // workspace root in workspace mode and the only project otherwise. A path
    // spelled against the member names no file from where the run was launched.
    let config = context.render_config();
    let mut errors = context
        .projects
        .iter()
        .flat_map(|project| project.scan_errors.iter())
        .collect::<Vec<_>>();
    // §FS-errors.4: by path, not by the order the projects were loaded — the
    // same order `check` prints the same errors in, keyed with `sort_path_key`
    // on the path itself, before rendering.
    errors.sort_by_key(|(path, message)| (sort_path_key(path), message.clone()));
    errors
        .into_iter()
        .map(|(path, message)| api_scan_error(config, path, message))
        .collect()
}

/// Programmatic `cover`: group every citation by scanned file, across every
/// project the run loaded, without choosing a CLI output format or process exit
/// code (§AR-bindings.2, §FS-workspace.8.6).
pub fn cover(opts: CoverOpts) -> Result<CoverOutput> {
    let context = cover_context(&opts)?;
    let entries = cover_rows(&context)
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
                })
                .collect(),
            path: row.path,
        })
        .collect();
    Ok(CoverOutput {
        output_format: context.render_config().output_format.clone(),
        entries,
        scan_errors: cover_scan_errors(&context),
    })
}

/// Programmatic text-oriented `cover`: return only the citation fields needed
/// for the default human-readable cover view while still leaving rendering to
/// frontends (§AR-bindings.2). Same index as [`cover`] (§FS-workspace.8.6);
/// the text view carries no alias because the path already renders from the
/// workspace root and the token is printed verbatim (§FS-cover.3.1).
pub fn cover_text(opts: CoverOpts) -> Result<CoverTextOutput> {
    let context = cover_context(&opts)?;
    let entries = cover_rows(&context)
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
    Ok(CoverTextOutput {
        output_format: context.render_config().output_format.clone(),
        entries,
        scan_errors: cover_scan_errors(&context),
    })
}
