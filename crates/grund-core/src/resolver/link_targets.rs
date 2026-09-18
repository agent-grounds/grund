//! Where a cross-reference link points (§AR-system.2.10, §FS-fmt.6.2): the
//! repo-relative path to the declaration's home file — following an inline-spec
//! stub to its real source file — and the heading anchor a Markdown home takes.
//!
//! A function of the loaded findings rather than of a rule or a write: it
//! resolves the ID against the whole project's declarations, follows a stub to
//! the file that really declares it, and re-reads a home file when the cited
//! section is not already in the section map (§AR-resolver.placement). Two components
//! ask it for the same answer — `grund fmt --cross-refs` for the link it writes
//! (§FS-fmt.6) and the checker's index-entry rule for the link it compares
//! against (§FS-check.3.18) — so it sat in `writers/fmt_link_targets.rs` while
//! the checker read it upward out of a component above it (§AR-system.4). The
//! derivation of an anchor *from* heading text is the lexical half and is
//! `grammar/anchors.rs`.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::config::Config;
use crate::grammar::{
    PythonDocstringScanState, anchor_slug, declaration_id_on_line, reduce_heading_text, render_id,
    section_anchor_text, section_path, source_scan_line,
};
use crate::model::{Declaration, Findings, Id, is_stub_for_inline_decl, resolve_stub_target};

/// Compute the link URL for a citation: a repo-relative path to the declaration's
/// home file — following an inline-spec stub to its real source file — plus a
/// heading anchor whenever the home is Markdown: the cited section's heading for a
/// `.<section>` citation, the declaration's own heading for a bare-ID citation
/// (§FS-fmt.6.2, §DF-md-link-anchor-strategy, §DF-declaration-anchor). A source-file
/// home (a stub's target) and the `none` profile both get a bare file link.
/// `None` if the ID does not resolve (§FS-fmt.6.3).
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
/// still drives anchor profile (§FS-fmt.6.7) and stub resolution, but the
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
    let is_md = home.extension().and_then(|e| e.to_str()) == Some("md");
    if !is_md || config.cross_ref_anchor_format == "none" {
        return Some(rel);
    }
    let heading = match section {
        Some(sec) => home_decl
            .sections
            .get(sec)
            .map(|section| section.title.clone())
            .or_else(|| section_heading_text(&home, id, sec, config).ok().flatten())?,
        // §DF-declaration-anchor: a bare-ID citation to a Markdown home links to
        // that declaration's own heading anchor, not just the file.
        None => declaration_heading_text(home_decl, config),
    };
    let anchor = anchor_slug(&heading, &config.cross_ref_anchor_format);
    Some(format!("{}#{}", rel, anchor))
}

/// The text content of a Markdown declaration's `# <ID>: <title>` heading — the
/// `<ID>` rendered per `[id] format`, then `: <title>` if the heading carries one — i.e.
/// what a renderer slugifies for the declaration's own anchor. The title is reduced
/// to its rendered form (`reduce_heading_text`), matching `section_anchor_text`
/// (§DF-declaration-anchor, §DF-github-anchor-fidelity).
fn declaration_heading_text(decl: &Declaration, config: &Config) -> String {
    let id = render_id(config, &decl.id);
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

/// Re-read a home file to find the heading text of a cited section — the fallback
/// when the section isn't already in the declaration's section map, so a link
/// anchor is always re-derived from the current heading (§FS-fmt.6.3,
/// §DF-md-link-anchor-strategy).
fn section_heading_text(
    path: &Path,
    id: &Id,
    section: &str,
    config: &Config,
) -> Result<Option<String>> {
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let is_md = path.extension().and_then(|e| e.to_str()) == Some("md");
    let is_py = path.extension().and_then(|e| e.to_str()) == Some("py");
    let mut in_decl = false;
    let mut py_docstring = PythonDocstringScanState::default();
    for line in text.lines() {
        let scan = source_scan_line(line, is_py, config.docstring_python, &mut py_docstring);
        let scan_line = scan.text;
        if let Some((found, _)) =
            declaration_id_on_line(&config.grammar, scan_line, scan.in_py_docstring, is_md)
        {
            if in_decl && &found != id {
                break;
            }
            if &found == id {
                in_decl = true;
                continue;
            }
        }
        if !in_decl {
            continue;
        }
        if let Some(caps) = config.grammar.section_re.captures(scan_line)
            && section_path(&caps).is_some_and(|found| found == section)
        {
            return Ok(Some(section_anchor_text(scan_line, section)));
        }
    }
    Ok(None)
}
