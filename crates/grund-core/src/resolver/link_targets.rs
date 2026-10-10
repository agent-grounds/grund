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

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::config::{Frame, Presentation, Schema};
use crate::grammar::{Grammar, anchor_slug, reduce_heading_text, render_id};
use crate::model::{
    Catalog, Declaration, Id, SectionInfo, id_homes, physical_path_key, scanned_decl_relative_path,
    scanned_path_key,
};

/// Compute the link URL for a citation: a repo-relative path to the declaration's
/// home file — following an inline-spec stub to its real source file — plus a
/// heading anchor whenever the home is Markdown: the cited section's heading for a
/// `.<section>` citation, the declaration's own heading for a bare-ID citation
/// (§FS-fmt.6.2, §DF-md-link-anchor-strategy, §DF-declaration-anchor). A source-file
/// home (a stub's target) and the `none` profile both get a bare file link.
/// `None` if the ID does not resolve (§FS-fmt.6.4), `None` for a broken stub and
/// for an ID with more than one home (§FS-fmt.6.4.2), and `None` for a `.<section>`
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
    markdown_link_target_with_root(from_file, id, section, presentation, frame, findings, None)
}

/// The canonical bare-ID link of every citation an index file records, keyed by
/// (index file, ID): what `Expected` hands the kind-index pass so external
/// enrollment compares a page against `fmt`'s destination without the checker
/// reading the anchor profile (§AR-checker.1.3, §AR-checker.2.16,
/// §FS-check.3.18.3). An index is a citable folder row's Markdown `index`, and
/// a citation is looked up under the first row of its own kind that keeps that
/// file, as the kind-index pass looks it up.
pub(crate) fn index_link_targets(
    presentation: &Presentation,
    schema: &Schema,
    frame: Frame<'_>,
    findings: &Catalog,
) -> BTreeMap<(PathBuf, Id), String> {
    let mut indexes: BTreeMap<PathBuf, Vec<(&str, PathBuf)>> = BTreeMap::new();
    for row in &schema.rows {
        let Some(index) = row.index_path() else {
            continue;
        };
        if index.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        indexes
            .entry(scanned_path_key(&index))
            .or_default()
            .push((row.name.as_str(), frame.root().join(&index)));
    }
    let mut targets = BTreeMap::new();
    if indexes.is_empty() {
        return targets;
    }
    let configured_root = scanned_path_key(frame.root());
    let physical_root = physical_path_key(frame.root());
    for citation in &findings.citations {
        if citation.namespace.is_some()
            || citation.section.is_some()
            || !citation.has_marker
            || citation.shorthand
            || !findings.declarations.contains_key(&citation.id)
        {
            continue;
        }
        let Some(relative) =
            scanned_decl_relative_path(&citation.file, &configured_root, &physical_root)
        else {
            continue;
        };
        let Some((_, index_file)) = indexes
            .get(relative.as_ref())
            .and_then(|rows| rows.iter().find(|(kind, _)| *kind == citation.id.kind))
        else {
            continue;
        };
        let key = (index_file.clone(), citation.id.clone());
        if targets.contains_key(&key) {
            continue;
        }
        if let Some(target) = markdown_link_target(
            index_file,
            &citation.id,
            None,
            presentation,
            frame,
            findings,
        ) {
            targets.insert(key, target);
        }
    }
    targets
}

/// §FS-workspace.8.5: same as `markdown_link_target`, but with an explicit
/// `path_root` override for relative-path computation. The target's presentation
/// and frame still drive the anchor profile (§FS-fmt.6.7.1), and its catalog the
/// stub verdicts, but the link path is anchored at `path_root` (the workspace
/// root) when the citing file and the target's home live in different projects.
pub(crate) fn markdown_link_target_with_root(
    from_file: &Path,
    id: &Id,
    section: Option<&str>,
    presentation: &Presentation,
    frame: Frame<'_>,
    findings: &Catalog,
    path_root: Option<&Path>,
) -> Option<String> {
    // §FS-fmt.6.4.2: only an ID with one home that is not a broken stub is linked, read
    // off the verdict the scan recorded on each stub (§FS-declarations.checks.broken-stub.4).
    let home = id_homes(findings.declarations.get(id)?).sole()?;
    if home.is_broken_stub() {
        return None;
    }
    let record = home.record;
    let rel = match path_root {
        Some(root) => relative_url_under(from_file, &record.file, root),
        None => relative_url(from_file, &record.file, frame.root()),
    };
    // §FS-fmt.6.4.1: a cited section is one the scan records, in a stub's target
    // outside the walk too (§FS-check.3.2.1); one it lacks gets no link, anchor or not.
    if section.is_some_and(|sec| !record.sections.contains_key(sec)) {
        return None;
    }
    if !takes_heading_anchor(&record.file, presentation) {
        return Some(rel);
    }
    // §FS-fmt.6.2.1.1: a bare ID's heading is the scan's record too.
    let anchor = heading_anchor(record, section, presentation, frame)?;
    Some(format!("{}#{}", rel, anchor))
}

/// Whether a link into `home` carries a heading anchor at all: only a Markdown
/// home does, and not under the `none` profile (§FS-fmt.6.2, §FS-fmt.6.7.1).
/// `show --format=json` asks the same question for its `anchor` field, so a
/// `null` there is exactly a bare file link here (§FS-show.3.1.3.1).
pub(crate) fn takes_heading_anchor(home: &Path, presentation: &Presentation) -> bool {
    home.extension().and_then(|e| e.to_str()) == Some("md")
        && presentation.fmt.anchor_format != "none"
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
/// `takes_heading_anchor` and passes the record the scan made of the home.
pub(crate) fn heading_anchor(
    decl: &Declaration,
    section: Option<&str>,
    presentation: &Presentation,
    frame: Frame<'_>,
) -> Option<String> {
    let heading = match section {
        Some(sec) => decl.sections.get(sec)?.title.clone(),
        // §DF-declaration-anchor: a bare-ID citation to a Markdown home links to
        // that declaration's own heading anchor, not just the file.
        None => declaration_heading_text(decl, frame.grammar()),
    };
    Some(anchor_slug(&heading, &presentation.fmt.anchor_format))
}

/// The anchor of one recorded section heading site, from the heading text the
/// scanner stored for it (§FS-show.3.1.3.1): each `--toc` entry reads its own
/// site, so a duplicated path still carries the anchor of the heading listed.
pub(crate) fn section_site_anchor(site: &SectionInfo, presentation: &Presentation) -> String {
    anchor_slug(&site.title, &presentation.fmt.anchor_format)
}

/// The text content of a Markdown declaration's `# <ID>: <title>` heading — the
/// `<ID>` rendered per `[id] format`, then `: <title>` if the heading carries one — i.e.
/// what a renderer slugifies for the declaration's own anchor. The title is reduced
/// to its rendered form (`reduce_heading_text`), matching `section_anchor_text`
/// (§DF-declaration-anchor, §DF-github-anchor-fidelity).
fn declaration_heading_text(decl: &Declaration, grammar: &Grammar) -> String {
    let id = render_id(grammar, &decl.id);
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
            let root = std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
            let from_file =
                std::fs::canonicalize(from_file).unwrap_or_else(|_| from_file.to_path_buf());
            let to_file = std::fs::canonicalize(to_file).unwrap_or_else(|_| to_file.to_path_buf());
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
