//! Scanner-recorded query ambiguities shared by show and refs (§FS-refs.4,
//! §FS-show.2.2.1, §FS-show.2.2.2). No body extraction is needed to decide
//! whether a recorded target has multiple claimants. A stub's homes are found in
//! its target, scanned or not (§FS-refs.4.1): a target the walk did not reach is
//! recorded by the scanner's own pass over it (§FS-show.2.3.7).

use std::path::Path;

use super::show_query::ShowQueryError;
use super::stub_home::stub_home;
use crate::config::{Config, display_path};
use crate::grammar::render_id;
use crate::model::{
    Declaration, FindingSite, Findings, Id, TextOverlays, is_stub_for_inline_decl,
    resolve_stub_target,
};

/// Refuse only independent homes or the requested section's recorded collision
/// (§FS-refs.4). Absent declarations and sections remain valid citation queries.
/// A stub's homes are those of its target, scanned or not, so a bare ID and a
/// section coordinate are refused from the one record `stub_home` chooses; a
/// stub it cannot pair refuses nothing (§FS-refs.4.1).
pub(crate) fn declaration_ambiguity_refusal(
    config: &Config,
    path_config: &Config,
    findings: &Findings,
    id: &Id,
    section: Option<&str>,
) -> Option<ShowQueryError> {
    let decls = findings.declarations.get(id)?;
    if let Some(refusal) = ambiguous_id_refusal(config, path_config, decls, id) {
        return Some(refusal);
    }
    let decl = decls.iter().find(|decl| decl.is_stub).unwrap_or(&decls[0]);
    let file = if let Some(target) = &decl.defined_in {
        resolve_stub_target(&config.root, &decl.file, target)
    } else {
        decl.file.clone()
    };
    // §FS-refs.4.1: refused from the record show would read, before a bare ID returns.
    let body_decl = match stub_home(
        config,
        path_config,
        decls,
        decl,
        &file,
        id,
        &TextOverlays::new(),
    ) {
        Ok(body_decl) => body_decl,
        Err(refusal) => return Some(refusal),
    };
    let section = section?;
    ambiguous_section_refusal(config, path_config, &body_decl, &file, id, section)
}

/// A valid stub/inline pair is one home; multiple independent homes refuse with
/// all sites in show's deterministic order (§FS-show.2.2.1, §FS-refs.4).
pub(super) fn ambiguous_id_refusal(
    config: &Config,
    path_config: &Config,
    decls: &[Declaration],
    id: &Id,
) -> Option<ShowQueryError> {
    let homes: Vec<&Declaration> = decls
        .iter()
        .filter(|decl| !is_stub_for_inline_decl(&config.root, decl, decls))
        .collect();
    if homes.len() <= 1 {
        return None;
    }
    // §FS-errors.3.1: every site is spelled from the effective report root.
    let mut sites: Vec<(String, String, usize)> = homes
        .iter()
        .map(|d| {
            let path = display_path(path_config, &d.file);
            (format!("{path}:{}", d.line), path, d.line)
        })
        .collect();
    sites.sort_by(|a, b| a.0.cmp(&b.0));
    let message = format!(
        "ambiguous ID: {} (declared at {})",
        render_id(&config.grammar, id),
        sites
            .iter()
            .map(|(rendered, ..)| rendered.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    // §FS-errors.5.2.1: JSON carries the same sites, without parsing prose.
    Some(ShowQueryError {
        code: "ambiguous",
        sites: sites
            .into_iter()
            .map(|(_, path, line)| FindingSite { path, line })
            .collect(),
        message,
    })
}

/// Refuse exactly the requested coordinate's scanner-recorded claims
/// (§FS-show.2.2.2, §FS-refs.4), before reading a body or filtering citations.
/// `body_decl` is the record `stub_home` chose, so a stub's sections are its
/// inline home's; its own prose declares none
/// (§FS-declarations.checks.duplicate-section.2, §FS-show.2.3.7). `path_config`
/// owns report paths.
pub(super) fn ambiguous_section_refusal(
    config: &Config,
    path_config: &Config,
    body_decl: &Declaration,
    file: &Path,
    id: &Id,
    section: &str,
) -> Option<ShowQueryError> {
    let mut lines: Vec<usize> = body_decl
        .duplicate_sections
        .iter()
        .filter(|(path, _)| path == section)
        .map(|(_, info)| info.line)
        .collect();
    if lines.is_empty() {
        return None;
    }
    // §AR-scanner.2.2.3: the map holds the first claimant, the list the rest.
    lines.extend(body_decl.sections.get(section).map(|first| first.line));
    lines.sort_unstable();
    let rendered = display_path(path_config, file);
    let sites_text = lines
        .iter()
        .map(|line| format!("{rendered}:{line}"))
        .collect::<Vec<_>>()
        .join(", ");
    let message = format!(
        "ambiguous section: {}{}{} (declared at {sites_text})",
        render_id(&config.grammar, id),
        config.section_separator,
        section
    );
    // §FS-errors.5.2.1: the message and JSON carry every claimant in file order.
    let sites = lines
        .iter()
        .map(|line| FindingSite {
            path: rendered.clone(),
            line: *line,
        })
        .collect();
    Some(ShowQueryError {
        code: "ambiguous-section",
        message,
        sites,
    })
}
