//! Where a cross-reference link points (§AR-system.2.10, §FS-fmt.6.2): the
//! repo-relative path to the declaration's home file — following an inline-spec
//! stub to its real source file — and the heading anchor a Markdown home takes.
//!
//! A function of the loaded findings rather than of a rule or a write: it
//! resolves the ID against the whole project's declarations, follows a stub to
//! the file that really declares it, and takes the declaration's own heading and
//! a cited section's from the scanner's record of the home, a stub's target's
//! where the walk did not reach it (§FS-fmt.6.2.1.1, §AR-resolver.placement,
//! §AR-resolver.5). Two components ask it for the same answer —
//! `grund fmt --cross-refs` for the link it writes (§FS-fmt.6) and the checker's
//! index-entry rule for the link it compares against (§FS-check.3.18.5) — so it
//! sat in `writers/fmt_link_targets.rs` while the checker read it upward out of a
//! component above it (§AR-system.4). The derivation of an anchor *from* heading
//! text is the lexical half and is `grammar/anchors.rs`.

use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use super::anchor_numbering::numbered_anchor;
use super::settled_headings::SettledHeadings;
use crate::config::{Frame, Owners, Presentation, Schema};
use crate::grammar::{
    Grammar, anchor_slug, reduce_heading_text, render_declaration_id, without_closing_sequence,
};
use crate::model::{
    Catalog, Declaration, FileRecord, FileRecords, Id, SectionInfo, configured_home_path_key,
    paired_declarations, scanned_decl_relative_path, scanned_path_key,
};
use crate::sources::{canonical_path, physical_path_key};

/// Compute the link URL for a citation: a repo-relative path to the declaration's
/// home file — following an inline-spec stub to its real source file — plus a
/// heading anchor whenever the home is Markdown: the cited section's heading for a
/// `.<section>` citation, the declaration's own heading for a bare-ID citation
/// (§FS-fmt.6.2, §DF-md-link-anchor-strategy, §DF-declaration-anchor). A source-file
/// home (a stub's target) and the `none` profile both get a bare file link.
/// `None` if the ID does not resolve (§FS-fmt.6.4), `None` for a broken stub and
/// for an ID with more than one declaration once its stubs are paired
/// (§FS-fmt.6.4.2, §AR-scanner.4.6), and `None` for a `.<section>`
/// citation of a section the declaration does not have, whether or not the link
/// takes a heading anchor (§FS-fmt.6.4.1).
///
/// The anchor profile is presentation's, so the record is handed in rather than
/// read off the project (§AR-resolver.6).
pub(crate) fn markdown_link_target(
    from_file: &Path,
    id: &Id,
    section: Option<&str>,
    presentation: &Presentation,
    frame: Frame<'_>,
    findings: &Catalog,
) -> Option<String> {
    let settled = SettledHeadings::default();
    markdown_link_target_with_root(
        from_file,
        id,
        section,
        presentation,
        frame,
        findings,
        None,
        &settled,
    )
}

/// The canonical bare-ID link from each index file to the declarations it can
/// name, keyed by (index file, ID): what `Expected` hands the kind-index pass so
/// it compares a page against `fmt`'s destination, and names the entry a
/// not-listed declaration owes, without the checker reading the anchor profile
/// (§AR-checker.1.3, §AR-checker.2.16, §FS-check.3.18.3, §FS-check.3.18.5.2).
/// An index is a citable folder row's Markdown `index`, and an ID is looked up
/// under the first row of its own kind that keeps that file, as the kind-index
/// pass looks it up. Two sets of IDs, and no more: every ID the index cites,
/// which external enrollment compares, and every declaration of the kind under
/// a folder that index lists, the only ones the finding can call not listed
/// there. Linking every declaration of the kind from every index instead costs
/// folders × declarations under a per-folder layout (§GOAL-fast-feedback). An
/// ID `fmt` would not link (§FS-fmt.6.4.2) has no entry. `settled` is the
/// heading lines a `fmt` run leaves (§FS-fmt.6.3.3), empty for every other caller.
pub(crate) fn index_link_targets<'c>(
    presentation: &Presentation,
    schema: &Schema,
    frame: Frame<'_>,
    findings: &'c Catalog,
    settled: &SettledHeadings,
) -> BTreeMap<(PathBuf, Id), String> {
    index_link_targets_for(presentation, schema, frame, findings, settled, None)
}

/// `index_link_targets`, linking only the IDs in `only` when it is given: the
/// entries a classifier restricted to those IDs compares (§FS-refs.3.1.1).
pub(crate) fn index_link_targets_for<'c>(
    presentation: &Presentation,
    schema: &Schema,
    frame: Frame<'_>,
    findings: &'c Catalog,
    settled: &SettledHeadings,
    only: Option<&BTreeSet<Id>>,
) -> BTreeMap<(PathBuf, Id), String> {
    // §FS-config-v2.schema.places.ownership: one index in each folder of the row.
    let places: Vec<(&str, &str, PathBuf, PathBuf)> = schema
        .rows
        .iter()
        .flat_map(|row| {
            row.index_paths()
                .into_iter()
                .map(move |(folder, index)| (row, folder, index))
        })
        .filter(|(_, _, index)| index.extension().and_then(|ext| ext.to_str()) == Some("md"))
        .map(|(row, folder, index)| {
            let file = frame.root().join(&index);
            (row.name.as_str(), folder, scanned_path_key(&index), file)
        })
        .collect();
    if places.is_empty() {
        return BTreeMap::new();
    }
    // Each index file by its place in path order, so a link is keyed by a number
    // until the end rather than by a path compared component by component.
    let mut files: Vec<&PathBuf> = places.iter().map(|(.., file)| file).collect();
    files.sort();
    files.dedup();
    let slot = |file: &PathBuf| {
        files
            .binary_search(&file)
            .expect("every index file has a slot")
    };
    let mut indexes: BTreeMap<&Path, Vec<(&str, usize)>> = BTreeMap::new();
    // Each folder with an index, keyed as a declaration's relative path is read.
    let mut folders: HashMap<PathBuf, Vec<(&str, &str, usize)>> = HashMap::new();
    for (kind, folder, key, file) in &places {
        indexes.entry(key).or_default().push((kind, slot(file)));
        folders
            .entry(configured_home_path_key(folder))
            .or_default()
            .push((folder, kind, slot(file)));
    }
    let mut found: BTreeMap<(usize, &Id), String> = BTreeMap::new();
    let mut link = |file: usize, id: &'c Id| {
        if only.is_some_and(|ids| !ids.contains(id)) {
            return;
        }
        let Entry::Vacant(entry) = found.entry((file, id)) else {
            return;
        };
        if let Some(target) = markdown_link_target_with_root(
            files[file],
            id,
            None,
            presentation,
            frame,
            findings,
            None,
            settled,
        ) {
            entry.insert(target);
        }
    };
    // The IDs each index cites with the marker: what enrollment compares.
    let configured_root = scanned_path_key(frame.root());
    let physical_root = physical_path_key(frame.root());
    for citation in &findings.citations {
        if citation.namespace.is_some()
            || citation.section.is_some()
            || !citation.has_marker
            || citation.shorthand
            || only.is_some_and(|ids| !ids.contains(&citation.id))
            || !findings.declarations.contains_key(&citation.id)
        {
            continue;
        }
        let Some(relative) =
            scanned_decl_relative_path(&citation.file, &configured_root, &physical_root)
        else {
            continue;
        };
        if let Some((_, file)) = indexes
            .get(relative.as_ref())
            .and_then(|rows| rows.iter().find(|(kind, _)| *kind == citation.id.kind))
        {
            link(*file, &citation.id);
        }
    }
    // §FS-check.3.18.5.2: the declarations under the index's folders, by the
    // membership test the kind-index pass owes entries by (§FS-check.3.18.2),
    // asked only of the folders above each declaration.
    let owners = Owners::new(schema, frame);
    for (id, decls) in &findings.declarations {
        if only.is_some_and(|ids| !ids.contains(id)) {
            continue;
        }
        for decl in decls {
            let Some(relative) = owners.relative(&decl.file) else {
                continue;
            };
            for (folder, kind, file) in relative
                .ancestors()
                .filter_map(|ancestor| folders.get(ancestor))
                .flatten()
            {
                if *kind == id.kind && schema.folder_owns(folder, &relative) {
                    link(*file, id);
                }
            }
        }
    }
    found
        .into_iter()
        .map(|((file, id), target)| ((files[file].clone(), id.clone()), target))
        .collect()
}

/// §FS-workspace.8.5: same as `markdown_link_target`, but with an explicit
/// `path_root` override for relative-path computation. The target's presentation
/// and frame still drive the anchor profile (§FS-fmt.6.7.1), and its catalog the
/// stub verdicts, but the link path is anchored at `path_root` (the workspace
/// root) when the citing file and the target's home live in different projects.
/// The anchor names the heading as `settled` says the run leaves it
/// (§FS-fmt.6.3.3), and as the scan read it where `settled` is silent.
#[allow(clippy::too_many_arguments)]
pub(crate) fn markdown_link_target_with_root(
    from_file: &Path,
    id: &Id,
    section: Option<&str>,
    presentation: &Presentation,
    frame: Frame<'_>,
    findings: &Catalog,
    path_root: Option<&Path>,
    settled: &SettledHeadings,
) -> Option<String> {
    // §FS-fmt.6.4.2: only an ID with one declaration, not a broken stub, is linked, read
    // off the verdict the scan recorded on each stub (§AR-scanner.4.6).
    let sole = paired_declarations(findings.declarations.get(id)?).sole()?;
    if sole.is_broken_stub() {
        return None;
    }
    let record = sole.record;
    let rel = match path_root {
        Some(root) => relative_url_under(from_file, &record.file, root),
        None => relative_url(from_file, &record.file, frame.root()),
    };
    // §FS-fmt.6.4.1: a cited section is one the scan records, in a stub's target
    // outside the walk too (§FS-check.3.2.1); one it lacks gets no link, anchor or not.
    if section.is_some_and(|sec| !record.sections.contains_key(sec)) {
        return None;
    }
    // The scan read the home's file as its language reads it, and recorded that.
    let markdown = findings
        .files
        .get(&record.file)
        .is_some_and(|file| file.markdown);
    // §FS-fields.front-matter.surfaces: a front-matter coordinate links to its file.
    let front_matter = section
        .and_then(|sec| record.sections.get(sec))
        .is_some_and(|info| info.front_matter.is_some());
    if !takes_heading_anchor(markdown, presentation) || front_matter {
        return Some(rel);
    }
    // §FS-fmt.6.2.1.1: a bare ID's heading is the scan's record too.
    let anchor = anchor_as_left(
        record,
        section,
        presentation,
        frame,
        &findings.files,
        settled,
    )?;
    Some(format!("{}#{}", rel, anchor))
}

/// Whether a link into a home carries a heading anchor at all: only a home that
/// reads as Markdown does (`markdown`, its language's answer,
/// §FS-config-v2.schema.sources.extensions), and not under the `none` profile
/// (§FS-fmt.6.2, §FS-fmt.6.7.1). `show --format=json` asks the same question for
/// its `anchor` field, so a `null` there is exactly a bare file link here
/// (§FS-show.3.1.3.1).
pub(crate) fn takes_heading_anchor(markdown: bool, presentation: &Presentation) -> bool {
    markdown && presentation.fmt.anchor_format != "none"
}

/// The heading anchor, without its `#`, of `decl` in its Markdown home: the
/// cited section's heading for a `.<section>` coordinate, the declaration's own
/// heading for a bare ID (§FS-fmt.6.2, §DF-md-link-anchor-strategy,
/// §DF-declaration-anchor). The one derivation `fmt --cross-refs` writes and
/// `show --format=json` reports, so the two cannot drift (§FS-show.3.1.3.1).
/// It reads the heading out of the current scan's record on every call, so each
/// pass re-derives the anchor from the heading as it now stands (§FS-fmt.6.3).
///
/// `None` when `decl`'s section map holds no section at that path: which headings
/// are sections, and where the body ends, are the scan's answer, the one `check`
/// reports against, so a heading the scan leaves out of the body gives no anchor
/// (§FS-fmt.6.4.1, §AR-scanner.2.4.1). The caller has already checked
/// `takes_heading_anchor` and passes the record the scan made of the home. A
/// section heading `settled` says the run rewrites is anchored as the run leaves
/// it (§FS-fmt.6.3.3); a declaration heading is one `fmt` never rewrites.
pub(crate) fn heading_anchor(
    decl: &Declaration,
    section: Option<&str>,
    presentation: &Presentation,
    frame: Frame<'_>,
    settled: &SettledHeadings,
) -> Option<String> {
    let heading = match section {
        Some(sec) => {
            // §FS-fields.front-matter.surfaces: a front-matter coordinate has no heading.
            let info = decl
                .sections
                .get(sec)
                .filter(|info| info.front_matter.is_none())?;
            settled
                .section_title(&decl.file, sec, info)
                .unwrap_or_else(|| info.title.clone())
        }
        // §DF-declaration-anchor: a bare-ID citation to a Markdown home links to
        // that declaration's own heading anchor, not just the file.
        None => declaration_heading_text(decl, frame.grammar()),
    };
    Some(rendered_anchor(&heading, presentation))
}

/// [`heading_anchor`], numbered as the renderer numbers a repeated slug among
/// the headings of `decl`'s home (§DF-repeated-heading-anchor.decision.count):
/// the anchor `show` reports of the tree as it stands.
pub(crate) fn numbered_heading_anchor(
    decl: &Declaration,
    section: Option<&str>,
    presentation: &Presentation,
    frame: Frame<'_>,
    files: &FileRecords,
) -> Option<String> {
    let settled = SettledHeadings::default();
    anchor_as_left(decl, section, presentation, frame, files, &settled)
}

/// [`numbered_heading_anchor`] of the tree as a `fmt` run leaves it: the anchor
/// `fmt --cross-refs` writes, with every heading of the home read as `settled`
/// says the run writes it (§FS-fmt.6.3.3).
fn anchor_as_left(
    decl: &Declaration,
    section: Option<&str>,
    presentation: &Presentation,
    frame: Frame<'_>,
    files: &FileRecords,
    settled: &SettledHeadings,
) -> Option<String> {
    let base = heading_anchor(decl, section, presentation, frame, settled)?;
    let line = match section {
        Some(sec) => decl.sections.get(sec)?.line,
        None => decl.line,
    };
    // No heading precedes a file's first line, so its home need not be looked up.
    if line == 1 {
        return Some(base);
    }
    Some(numbered_anchor(
        files.get(&decl.file).map(AsRef::as_ref),
        settled.of_file(&decl.file),
        line,
        base,
        &presentation.fmt.anchor_format,
    ))
}

/// The anchor of one recorded section heading site, from the heading text the
/// scanner stored for it (§FS-show.3.1.3.1): each `--toc` entry reads its own
/// site, so a duplicated path still carries the anchor of the heading listed,
/// numbered among its home's headings (§DF-repeated-heading-anchor.decision.count).
pub(crate) fn section_site_anchor(
    site: &SectionInfo,
    presentation: &Presentation,
    home: Option<&FileRecord>,
) -> String {
    let base = rendered_anchor(&site.title, presentation);
    numbered_anchor(home, None, site.line, base, &presentation.fmt.anchor_format)
}

/// The anchor of a heading's stored text, slugged from its rendered text: a
/// closing `#` run is not part of it (§FS-declarations.heading.rendered-text,
/// §DF-closing-hashes-are-not-the-heading), while the `--toc` title keeps it.
fn rendered_anchor(heading: &str, presentation: &Presentation) -> String {
    anchor_slug(
        without_closing_sequence(heading),
        &presentation.fmt.anchor_format,
    )
}

/// The text content of a Markdown declaration's `# <ID>: <title>` heading — the
/// `<ID>` rendered per `[id] format`, then `: <title>` if the heading carries one — i.e.
/// what a renderer slugifies for the declaration's own anchor. The title is reduced
/// to its rendered form (`reduce_heading_text`), matching `section_anchor_text`
/// (§DF-declaration-anchor, §DF-github-anchor-fidelity).
/// The record is the target declaration's, including through a stub; its numeric
/// source spelling supplies the ID, never the querying citation's digits
/// (§FS-fmt.6.2.2, §FS-config.3.2.8).
fn declaration_heading_text(decl: &Declaration, grammar: &Grammar) -> String {
    let id = render_declaration_id(grammar, decl);
    match &decl.title {
        Some(title) => format!("{id}: {}", reduce_heading_text(title)),
        None => id,
    }
}

/// `../`-style relative path from one repo file to another — the link form
/// `grund fmt --cross-refs` writes (§FS-fmt.6.2).
fn relative_url(from_file: &Path, to_file: &Path, root: &Path) -> String {
    relative_url_under(from_file, to_file, root)
}

/// Same as `relative_url`, but uses an explicit project root for stripping —
/// the workspace-root variant (§FS-workspace.8.5) so a citing file in one
/// project can link to a target file in another project under the same
/// workspace root with a single common-prefix walk.
fn relative_url_under(from_file: &Path, to_file: &Path, root: &Path) -> String {
    let (from_rel, to_rel) = match (from_file.strip_prefix(root), to_file.strip_prefix(root)) {
        (Ok(from_rel), Ok(to_rel)) => (
            std::borrow::Cow::Borrowed(from_rel),
            std::borrow::Cow::Borrowed(to_rel),
        ),
        _ => {
            let root = canonical_path(root).unwrap_or_else(|_| root.to_path_buf());
            let from_file = canonical_path(from_file).unwrap_or_else(|_| from_file.to_path_buf());
            let to_file = canonical_path(to_file).unwrap_or_else(|_| to_file.to_path_buf());
            let from_rel = from_file
                .strip_prefix(&root)
                .map(Path::to_path_buf)
                .unwrap_or(from_file);
            let to_rel = to_file
                .strip_prefix(&root)
                .map(Path::to_path_buf)
                .unwrap_or(to_file);
            (
                std::borrow::Cow::Owned(from_rel),
                std::borrow::Cow::Owned(to_rel),
            )
        }
    };
    let from_dir = from_rel.parent().unwrap_or(Path::new(""));
    let from_components = path_components(from_dir);
    let to_components = path_components(&to_rel);
    let mut common = 0;
    while common < from_components.len()
        && common < to_components.len()
        && from_components[common] == to_components[common]
    {
        common += 1;
    }
    let mut parts = Vec::new();
    for _ in common..from_components.len() {
        parts.push("..".to_string());
    }
    parts.extend(to_components[common..].iter().cloned());
    if parts.is_empty() {
        ".".to_string()
    } else {
        parts.join("/")
    }
}

fn path_components(path: &Path) -> Vec<String> {
    path.components()
        .filter_map(|component| match component {
            std::path::Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect()
}
