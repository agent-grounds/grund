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

use std::path::Path;

use super::stub_home::home_as_scanned;
use crate::config::Config;
use crate::grammar::{anchor_slug, reduce_heading_text, render_id};
use crate::model::{
    Declaration, Findings, Id, SectionInfo, is_stub_for_inline_decl, resolve_stub_target,
};

/// Compute the link URL for a citation: a repo-relative path to the declaration's
/// home file — following an inline-spec stub to its real source file — plus a
/// heading anchor whenever the home is Markdown: the cited section's heading for a
/// `.<section>` citation, the declaration's own heading for a bare-ID citation
/// (§FS-fmt.6.2, §DF-md-link-anchor-strategy, §DF-declaration-anchor). A source-file
/// home (a stub's target) and the `none` profile both get a bare file link.
/// `None` if the ID does not resolve (§FS-fmt.6.4), and `None` for a `.<section>`
/// citation of a section the declaration does not have, whether or not the link
/// takes a heading anchor (§FS-fmt.6.4.1).
pub(crate) fn markdown_link_target(
    from_file: &Path,
    id: &Id,
    section: Option<&str>,
    config: &Config,
    findings: &Findings,
) -> Option<String> {
    markdown_link_target_with_root(from_file, id, section, config, findings, None)
}

/// §FS-workspace.8.5: same as `markdown_link_target`, but with an explicit
/// `path_root` override for relative-path computation. The target's `config`
/// still drives anchor profile (§FS-fmt.6.7.1) and stub resolution, but the
/// link path is anchored at `path_root` (the workspace root) when the
/// citing file and the target's home live in different projects.
pub(crate) fn markdown_link_target_with_root(
    from_file: &Path,
    id: &Id,
    section: Option<&str>,
    config: &Config,
    findings: &Findings,
    path_root: Option<&Path>,
) -> Option<String> {
    let decls = findings.declarations.get(id)?;
    let stub = decls.iter().find(|decl| decl.is_stub);
    let home_decl = decls
        .iter()
        .find(|decl| !is_stub_for_inline_decl(&config.root, decl, decls))
        .or_else(|| decls.first())?;
    let home = if let Some(stub) = stub {
        let target = stub.defined_in.as_ref()?;
        resolve_stub_target(&config.root, &stub.file, target)
    } else {
        home_decl.file.clone()
    };
    let rel = match path_root {
        Some(root) => relative_url_under(from_file, &home, root),
        None => relative_url(from_file, &home, config),
    };
    // §FS-fmt.6.4.1: a cited section is one the scan records, in a stub's target
    // outside the walk too (§FS-check.3.2.1); one it lacks gets no link, anchor or not.
    let scanned = match section {
        Some(sec) => {
            let scanned = home_as_scanned(findings, config, id, home_decl);
            if !scanned.sections.contains_key(sec) {
                return None;
            }
            Some(scanned)
        }
        None => None,
    };
    if !takes_heading_anchor(&home, config) {
        return Some(rel);
    }
    // §FS-fmt.6.2.1.1: a bare ID's heading is the scan's record too.
    let anchor_decl = scanned.unwrap_or_else(|| home_as_scanned(findings, config, id, home_decl));
    let anchor = heading_anchor(anchor_decl, section, config)?;
    Some(format!("{}#{}", rel, anchor))
}

/// Whether a link into `home` carries a heading anchor at all: only a Markdown
/// home does, and not under the `none` profile (§FS-fmt.6.2, §FS-fmt.6.7.1).
/// `show --format=json` asks the same question for its `anchor` field, so a
/// `null` there is exactly a bare file link here (§FS-show.3.1.3.1).
pub(crate) fn takes_heading_anchor(home: &Path, config: &Config) -> bool {
    home.extension().and_then(|e| e.to_str()) == Some("md")
        && config.cross_ref_anchor_format != "none"
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
    config: &Config,
) -> Option<String> {
    let heading = match section {
        Some(sec) => decl.sections.get(sec)?.title.clone(),
        // §DF-declaration-anchor: a bare-ID citation to a Markdown home links to
        // that declaration's own heading anchor, not just the file.
        None => declaration_heading_text(decl, config),
    };
    Some(anchor_slug(&heading, &config.cross_ref_anchor_format))
}

/// The anchor of one recorded section heading site, from the heading text the
/// scanner stored for it (§FS-show.3.1.3.1): each `--toc` entry reads its own
/// site, so a duplicated path still carries the anchor of the heading listed.
pub(crate) fn section_site_anchor(site: &SectionInfo, config: &Config) -> String {
    anchor_slug(&site.title, &config.cross_ref_anchor_format)
}

/// The text content of a Markdown declaration's `# <ID>: <title>` heading — the
/// `<ID>` rendered per `[id] format`, then `: <title>` if the heading carries one — i.e.
/// what a renderer slugifies for the declaration's own anchor. The title is reduced
/// to its rendered form (`reduce_heading_text`), matching `section_anchor_text`
/// (§DF-declaration-anchor, §DF-github-anchor-fidelity).
fn declaration_heading_text(decl: &Declaration, config: &Config) -> String {
    let id = render_id(&config.grammar, &decl.id);
    match &decl.title {
        Some(title) => format!("{id}: {}", reduce_heading_text(title)),
        None => id,
    }
}

/// `../`-style relative path from one repo file to another — the link form
/// `grund fmt --cross-refs` writes (§FS-fmt.6.2).
fn relative_url(from_file: &Path, to_file: &Path, config: &Config) -> String {
    relative_url_under(from_file, to_file, &config.root)
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
