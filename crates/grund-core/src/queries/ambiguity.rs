//! Scanner-recorded query ambiguities shared by show and refs (§FS-refs.4,
//! §FS-show.2.2.1, §FS-show.2.2.2). No body extraction is needed to decide
//! whether a recorded target has multiple claimants. A stub's homes are found in
//! its target, scanned or not (§FS-refs.4.1), read off the verdict the scan
//! recorded on the stub (§AR-scanner.4.6).

use super::show::broken_stub_message;
use super::show_query::ShowQueryError;
use crate::config::{Config, display_path};
use crate::grammar::render_declaration_id;
use crate::model::{
    Catalog, Declaration, FindingSite, Id, MissingStubRepair, Paired, PairedDeclarations,
    paired_declarations,
};

/// Refuse only independent homes or the requested section's recorded collision
/// (§FS-refs.4). Absent declarations and sections remain valid citation queries.
/// A stub's homes are those of its target, scanned or not, so a bare ID and a
/// section coordinate are refused from the one record `show` reads; a broken
/// stub's own record has no section to refuse (§FS-refs.4.1).
///
/// A broken stub is no home here: beside a real home it is set aside rather than
/// counted, and what is returned is the text of one warning per stub set aside,
/// in `path:line` order (§FS-refs.4.1.1). A broken stub that is the only home
/// sets nothing aside. `show` decides on every declaration, and so does the
/// refusal this returns: it names every claimant, as `show`'s does.
pub(crate) fn declaration_ambiguity_refusal(
    config: &Config,
    path_config: &Config,
    findings: &Catalog,
    id: &Id,
    section: Option<&str>,
) -> Result<Vec<String>, ShowQueryError> {
    let Some(decls) = findings.declarations.get(id) else {
        return Ok(Vec::new());
    };
    let paired = paired_declarations(decls);
    // §FS-refs.4.1.1: refuse among the homes that remain once broken stubs are set aside.
    let homes: Vec<Paired<'_>> = paired
        .iter()
        .filter(|home| !home.is_broken_stub())
        .collect();
    if homes.len() > 1
        && let Some(refusal) = ambiguous_id_refusal(config, path_config, &paired, id)
    {
        return Err(refusal);
    }
    // §FS-refs.4.1: refused from the record show would read, before a bare ID returns.
    let record = match homes.as_slice() {
        [home] => Some(home.record),
        _ => paired.sole().map(|sole| sole.record),
    };
    if let (Some(record), Some(section)) = (record, section)
        && let Some(refusal) = ambiguous_section_refusal(config, path_config, record, id, section)
    {
        return Err(refusal);
    }
    if homes.is_empty() {
        return Ok(Vec::new());
    }
    Ok(set_aside_warnings(config, path_config, &paired))
}

/// §FS-refs.4.1.1: the warning each broken stub set aside beside a real home earns,
/// in `path:line` order — the `broken stub:` clause the ambiguity refusal carries
/// for it (§FS-declarations.stubs.repair), or, for a verdict that clause does not
/// cover, the line `show` refuses the stub alone with (§FS-show.2.3.4).
fn set_aside_warnings(
    config: &Config,
    path_config: &Config,
    paired: &PairedDeclarations<'_>,
) -> Vec<String> {
    let Some(first) = paired.records().next() else {
        return Vec::new();
    };
    // Spelled as the refusal spells the ID, so the clause is the refusal's own.
    let rendered_id = render_declaration_id(&config.grammar, first);
    let repair = MissingStubRepair::new(paired);
    paired
        .iter()
        .filter(Paired::is_broken_stub)
        .filter_map(|stub| {
            repair
                .stub_clause(stub.record, &rendered_id, |path| {
                    display_path(path_config, path)
                })
                .or_else(|| broken_stub_message(config, path_config, stub.record))
        })
        .collect()
}

/// A valid stub/inline pair is one home; multiple independent homes refuse with
/// all sites in show's deterministic order (§FS-show.2.2.1, §FS-refs.4).
/// Missing-target guidance shares the checker's derivation and located suffix
/// (§AR-system.2.2).
pub(super) fn ambiguous_id_refusal(
    config: &Config,
    path_config: &Config,
    paired: &PairedDeclarations<'_>,
    _id: &Id,
) -> Option<ShowQueryError> {
    if paired.len() <= 1 {
        return None;
    }
    // §FS-declarations.stubs.pairing.3: a declaration a stub stands for is at its
    // target, at each line there that declares the ID (§AR-scanner.4.6).
    // §FS-errors.3.1: every site is spelled from the effective report root.
    let mut sites: Vec<(String, String, usize)> = paired
        .iter()
        .map(|declaration| {
            let (path, line) = (
                display_path(path_config, &declaration.record.file),
                declaration.record.line,
            );
            (format!("{path}:{line}"), path, line)
        })
        .collect();
    sites.sort_by(|a, b| a.0.cmp(&b.0));
    let rendered_id = render_declaration_id(
        &config.grammar,
        paired.records().next().expect("multiple records"),
    );
    let mut message = format!(
        "ambiguous ID: {rendered_id} (declared at {})",
        sites
            .iter()
            .map(|(rendered, ..)| rendered.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    message.push_str(
        &MissingStubRepair::new(paired)
            .located_suffix(&rendered_id, |path| display_path(path_config, path)),
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
/// `body_decl` is the record of the ID's one declaration, so a stub's sections are
/// its inline home's; its own prose declares none
/// (§FS-declarations.checks.duplicate-section.2, §FS-show.2.3.7). `path_config`
/// owns report paths.
pub(super) fn ambiguous_section_refusal(
    config: &Config,
    path_config: &Config,
    body_decl: &Declaration,
    _id: &Id,
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
    let rendered = display_path(path_config, &body_decl.file);
    let sites_text = lines
        .iter()
        .map(|line| format!("{rendered}:{line}"))
        .collect::<Vec<_>>()
        .join(", ");
    let message = format!(
        "ambiguous section: {}{}{} (declared at {sites_text})",
        render_declaration_id(&config.grammar, body_decl),
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
