//! The two modes `refs` selects its sites by instead of one ID (§FS-refs.1.1): the
//! sites whose cited coordinate a coordinate glob matches (§FS-refs.globs), and the
//! sites one coordinate's own body holds (§FS-refs.from). Both read the set ID mode
//! lists from (§FS-refs.2), list each `(path, line, column)` once in that order, and
//! spell a record's `id` as `cover` does, with no target's kind title (§FS-refs.3.2).

use anyhow::Result;
use std::collections::BTreeSet;
use std::path::PathBuf;

use super::cover::{cited_id, target_config};
use super::refs::{RefHit, RefsOutput};
use super::refs_query::{hit_path, refs_scan_errors, site_key};
use super::report::context_run_warnings;
use super::show::ShowSource;
use crate::checker::ScanScope;
use crate::grammar::{path_at_or_under, render_catalog_id};
use crate::model::{Citation, Declaration, Finding, Id, ShowRenderMode};
use crate::queries::{GlobSelection, read_globs, reported_scope, reports_site};
use crate::resolver::{
    WorkspaceContext, load_classifying_workspace_context, load_reported_workspace_context,
};
use crate::sources::paths_same_location;

/// What a `refs` run selects its sites by when it names no one ID (§FS-refs.1.1).
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RefsSites {
    /// The sites whose cited coordinate any of the patterns matches, unioned
    /// (§FS-refs.globs). No pattern selects no site.
    Globs(Vec<String>),
    /// The sites in the body of the coordinate, spelled as for `grund <ID>`
    /// (§FS-refs.from).
    From(String),
}

/// The options of a `refs` run in [`RefsSites`] mode: the scan path, as ID mode's
/// (§FS-refs.1), and the selection.
#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct RefsSitesOpts {
    pub path: PathBuf,
    pub path_provided: bool,
    pub sites: RefsSites,
}

/// Programmatic `refs --glob` and `refs --from`: the selected citation sites in
/// [`RefsOutput`], whose `note` is the patterns' (§FS-coordinate-globs.errors.note),
/// never ID mode's (§FS-refs.globs.unresolved). A refused pattern, and a source `grund
/// <ID>` would refuse, are `Err` as that query raises them (§FS-refs.from). The run's
/// cautions come back beside the answer, so a refusal keeps them
/// (§FS-distribution.3.1).
#[doc(hidden)]
pub fn refs_sites_with_run_warnings(opts: RefsSitesOpts) -> (Vec<Finding>, Result<RefsOutput>) {
    let mut cautions = Vec::new();
    let result = refs_sites_run(opts, &mut cautions, None);
    (cautions, result)
}

/// §FS-refs.3.1.1 / §FS-refs.3.3.1: the CLI's text-only classification, beside
/// the unchanged site-mode records and run warnings, without a second scan.
#[doc(hidden)]
pub fn refs_sites_with_text_annotations(
    opts: RefsSitesOpts,
) -> (
    Vec<Finding>,
    Result<(RefsOutput, BTreeSet<(String, usize, usize)>)>,
) {
    let mut cautions = Vec::new();
    let mut entries = BTreeSet::new();
    let result =
        refs_sites_run(opts, &mut cautions, Some(&mut entries)).map(|output| (output, entries));
    (cautions, result)
}

/// One selected site, with the index of the project whose catalog holds it.
struct Site<'c> {
    project: usize,
    citation: &'c Citation,
}

fn refs_sites_run(
    opts: RefsSitesOpts,
    cautions: &mut Vec<Finding>,
    index_entries: Option<&mut BTreeSet<(String, usize, usize)>>,
) -> Result<RefsOutput> {
    // §FS-coordinate-globs.errors.shape: what the grammar alone refuses, before discovery.
    let globs = match &opts.sites {
        RefsSites::Globs(patterns) => read_globs(patterns)?,
        RefsSites::From(_) => Vec::new(),
    };
    // §AR-scanner.2.4.2: the classifying load, which gives each site its enclosing unit.
    let context = load_classifying_workspace_context(&opts.path, opts.path_provided)?;
    *cautions = context_run_warnings(&context);
    let scope = reported_scope(&context, &opts.path, opts.path_provided)?;
    let (mut sites, note) = match &opts.sites {
        RefsSites::Globs(_) => {
            let selection = GlobSelection::new(&context, &globs)?;
            (glob_sites(&context, &selection), selection.note())
        }
        RefsSites::From(source) => {
            // §AR-resolver.3.3: reuse `show`'s resolution scope, keeping the scanned edges.
            let source_context = scope
                .as_ref()
                .map(|_| load_reported_workspace_context(&opts.path, opts.path_provided))
                .transpose()?;
            (
                from_sites(
                    &context,
                    source_context.as_ref().unwrap_or(&context),
                    source,
                    scope.as_ref(),
                )?,
                None,
            )
        }
    };
    // §AR-resolver.3.3: each mode lists the report scope's sites (§FS-refs.1.1).
    sites.retain(|site| {
        let declarations = &context.projects[site.project].findings.declarations;
        reports_site(scope.as_ref(), declarations, site.citation)
    });
    // §FS-imports.graph.diamond: a site of a tree two alias paths reach, under its canonical one.
    for site in &mut sites {
        (site.project, site.citation) = context.canonical_site(site.project, site.citation);
    }
    // §FS-refs.globs: in `(path, line, column)` order, each site once however many
    // patterns select it or catalogs hold it.
    let key = |site: &Site| site_key(&context, &context.projects[site.project], site.citation);
    sites.sort_by_cached_key(key);
    sites.dedup_by_key(|site| key(site));
    let hits = sites
        .iter()
        .map(|site| {
            let project = &context.projects[site.project];
            let citation = site.citation;
            RefHit {
                project: context.workspace_loaded.then(|| project.alias.clone()),
                path: hit_path(&context, project, &citation.file),
                line: citation.line,
                column: citation.column,
                // §FS-refs.3.2: as `cover` spells it, the alias it was written with kept.
                id: cited_id(
                    target_config(&context, site.project, citation),
                    &context.projects[context
                        .citation_target(site.project, citation)
                        .unwrap_or(site.project)]
                    .findings,
                    citation,
                    &project.config.marker,
                    project.mount.as_ref(),
                ),
                section: citation.section.clone(),
                marker: citation.has_marker,
                text: citation.text.clone(),
                enclosing_declaration: citation.enclosing_declaration.as_ref().map(|id| {
                    render_catalog_id(&project.config.grammar, &project.findings.declarations, id)
                }),
                enclosing_section: citation.enclosing_section.clone(),
            }
        })
        .collect();
    if let Some(entries) = index_entries {
        *entries = super::refs_annotations::index_entry_sites(
            &context,
            sites
                .iter()
                .map(|site| (&context.projects[site.project], site.citation)),
        );
    }
    Ok(RefsOutput {
        output_format: context.render_config().output_format.clone(),
        workspace: context.workspace_loaded,
        hits,
        note,
        scan_errors: refs_scan_errors(&context),
        warnings: context_run_warnings(&context),
    })
}

/// Every edge of every loaded project whose cited coordinate a pattern selects,
/// dangling or not (§FS-refs.globs.unresolved, §FS-coordinate-globs.matching.citations).
fn glob_sites<'c>(context: &'c WorkspaceContext, selection: &GlobSelection) -> Vec<Site<'c>> {
    context
        .projects
        .iter()
        .enumerate()
        .flat_map(|(index, project)| {
            // §FS-refs.2: the set holds what a stub's target lends, resolved.
            project
                .findings
                .edges()
                .filter(move |citation| selection.selects_citation(context, index, citation))
                .map(move |citation| Site {
                    project: index,
                    citation,
                })
        })
        .collect()
}

/// The sites the source's own body holds: resolved and refused as `grund <ID>`
/// resolves it in `source_context` (§FS-refs.from), then the edges the selected
/// scan gives to what it owns (§FS-refs.from.owned), lent ones included
/// (§FS-refs.from.stubs). The site's file must lie within the path's scope,
/// even when a stub outside that path lends it (§FS-refs.from).
fn from_sites<'c>(
    context: &'c WorkspaceContext,
    source_context: &WorkspaceContext,
    source: &str,
    scope: Option<&ScanScope>,
) -> Result<Vec<Site<'c>>> {
    let source = ShowSource::resolve(source_context, source, None)?;
    let (_, home) = source.read(source_context, ShowRenderMode::Brief, false)?;
    let project = context
        .projects
        .iter()
        .position(|project| paths_same_location(&project.config.root, &source.project.config.root))
        .expect("source resolution and the site scan discover the same projects");
    let section = source.section.as_deref();
    Ok(context.projects[project]
        .findings
        .edges()
        .filter(|citation| scope.is_none_or(|scope| scope.contains(&citation.file)))
        .filter(|citation| owned(citation, &source.id, section, home))
        .map(|citation| Site { project, citation })
        .collect())
}

/// Whether the scan gives a site to the source's declaration in its home, under no
/// section, a proper ancestor of the source's section, or that section, by whole
/// `.` components (§FS-refs.from.owned, §FS-config.3.3.4).
fn owned(citation: &Citation, id: &Id, section: Option<&str>, home: &Declaration) -> bool {
    citation.enclosing_declaration.as_ref() == Some(id)
        && paths_same_location(&citation.file, &home.file)
        && match citation.enclosing_section.as_deref() {
            None => true,
            Some(enclosing) => {
                section.is_some_and(|section| path_at_or_under(section, enclosing, "."))
            }
        }
}
