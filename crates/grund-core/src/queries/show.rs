use anyhow::{Result, anyhow};
use std::path::Path;

use super::show_query::ShowQueryError;
use crate::checker::file_declares_inline_home;
use crate::config::{Config, display_path};
use crate::grammar::render_id;
use crate::model::{
    Declaration, DeclarationSource, FindingSite, Findings, Id, ShowOutput, ShowRenderMode,
    TextOverlays, format_path, is_stub_for_inline_decl, json_escape, paths_same_location,
    resolve_stub_target,
};
use crate::resolver::{extract_declaration_body, show_e2e_case};

#[cfg(test)]
pub(crate) fn show_declaration(
    config: &Config,
    path_config: &Config,
    findings: &Findings,
    id: &Id,
    section: Option<&str>,
    mode: ShowRenderMode,
    include_heading: bool,
) -> Result<ShowOutput> {
    show_declaration_with_overlays(
        config,
        path_config,
        findings,
        id,
        section,
        mode,
        include_heading,
        &TextOverlays::new(),
    )
}

/// `path_config` renders every path this function reports (§FS-config.3.6) — the
/// sites a refusal names (§FS-errors.3.1) as well as the E2E manifest's, whose
/// JSON is baked here rather than in `render_show_output_json`. It must be the
/// same config the caller hands that renderer, so a member's declaration reports
/// against the root the rest of the run spells its paths from
/// (§FS-workspace.8.1.2). `config` stays the *project's*: it owns the ID grammar
/// `render_id` reads and the tree the body is read out of.
pub(crate) fn show_declaration_with_overlays(
    config: &Config,
    path_config: &Config,
    findings: &Findings,
    id: &Id,
    section: Option<&str>,
    mode: ShowRenderMode,
    include_heading: bool,
    overlays: &TextOverlays,
) -> Result<ShowOutput> {
    let root = &config.root;
    let decls = findings
        .declarations
        .get(id)
        .ok_or_else(|| anyhow!("ID not found: {}", render_id(&config.grammar, id)))?;
    let homes: Vec<&Declaration> = decls
        .iter()
        .filter(|decl| !is_stub_for_inline_decl(root, decl, decls))
        .collect();
    if homes.len() > 1 {
        // §FS-errors.3.1: every path this refusal names is spelled from the report
        // root, like the `path` of any diagnostic printed beside it.
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
        // §FS-errors.5.2: the same `[{ path, line }]` pairs, in the same order, ride
        // along in a typed carrier so the JSON printer never re-parses this prose.
        return Err(ShowQueryError {
            code: "ambiguous",
            sites: sites
                .into_iter()
                .map(|(_, path, line)| FindingSite { path, line })
                .collect(),
            message,
        }
        .into());
    }
    let decl = decls.iter().find(|decl| decl.is_stub).unwrap_or(&decls[0]);
    if matches!(decl.source, DeclarationSource::Json { .. }) {
        return show_json_value(config, id, decl, section);
    }
    if let Some(case) = &decl.e2e_case {
        return show_e2e_case(config, path_config, id, case, section, mode);
    }
    let file = if let Some(target) = &decl.defined_in {
        resolve_stub_target(root, &decl.file, target)
    } else {
        decl.file.clone()
    };
    if decl.is_stub {
        if !file.exists() {
            return Err(anyhow!(
                "broken stub: {} (stub at {}:{} points at {}, which does not exist)",
                render_id(&config.grammar, id),
                display_path(path_config, &decl.file),
                decl.line,
                format_path(decl.defined_in.as_ref().unwrap())
            ));
        }
        if !file_declares_inline_home(&file, id, config).unwrap_or(false) {
            return Err(anyhow!(
                "broken stub: {} (stub at {}:{} points at {}, which contains no inline declaration of {})",
                render_id(&config.grammar, id),
                display_path(path_config, &decl.file),
                decl.line,
                format_path(decl.defined_in.as_ref().unwrap()),
                render_id(&config.grammar, id)
            ));
        }
    }
    let body_decl = if decl.is_stub {
        decls
            .iter()
            .find(|other| paths_same_location(&other.file, &file))
            .unwrap_or(decl)
    } else {
        decl
    };
    if let Some(section) = section
        && let Some(refusal) =
            ambiguous_section_refusal(config, path_config, decls, decl, &file, id, section)
    {
        return Err(refusal);
    }
    if let Some(section) = section
        && !body_decl.sections.contains_key(section)
    {
        return Err(anyhow!(
            "section not found: {}{}{}",
            render_id(&config.grammar, id),
            config.section_separator,
            section
        ));
    }
    extract_declaration_body(
        &file,
        id,
        body_decl,
        section,
        mode,
        include_heading,
        config,
        overlays,
    )
}

/// JSON values have source slices rather than Markdown bodies. Every show mode
/// therefore returns the exact member or element bytes and invents no heading
/// or outline (§FS-values.6.2, §FS-show.2).
fn show_json_value(
    config: &Config,
    id: &Id,
    decl: &Declaration,
    section: Option<&str>,
) -> Result<ShowOutput> {
    let (body, line) = match section {
        Some(section) => {
            let info = decl.sections.get(section).ok_or_else(|| {
                anyhow!(
                    "section not found: {}{}{}",
                    render_id(&config.grammar, id),
                    config.section_separator,
                    section
                )
            })?;
            let value = info.value.as_ref().ok_or_else(|| {
                anyhow!(
                    "section not found: {}{}{}",
                    render_id(&config.grammar, id),
                    config.section_separator,
                    section
                )
            })?;
            (value.source_slice.clone(), info.line)
        }
        None => match &decl.source {
            DeclarationSource::Json { member_slice, .. } => (member_slice.clone(), decl.line),
            DeclarationSource::Text => unreachable!("JSON value branch requires JSON source"),
        },
    };
    Ok(ShowOutput {
        body,
        path: decl.file.clone(),
        line,
        json: None,
        sections: Vec::new(),
    })
}

/// §FS-show.2.2.2: refuse a section coordinate two headings claim, before the
/// body is read.
///
/// The claimants are the ones the *scan* recorded (§AR-scanner.2.2.4) — the same
/// record §FS-declarations.checks.duplicate-section.4 reports from — so `show` refuses exactly the
/// coordinates `check` names and no others. Re-detecting them while extracting
/// the body would be a second, weaker reader: it would have to redo the fence
/// tracking, the heading-level gate, and the body bounds, and any of the three
/// getting a different answer is a coordinate one command calls clean and the
/// other will not resolve.
///
/// For a stub the sections belong to the **inline home**, which is the file the
/// body comes out of; a stub's own prose declares none (§FS-declarations.checks.duplicate-section.2). Paths
/// in the message use `path_config`, the report-path config named on
/// `show_declaration_with_overlays`.
fn ambiguous_section_refusal(
    config: &Config,
    path_config: &Config,
    decls: &[Declaration],
    decl: &Declaration,
    file: &Path,
    id: &Id,
    section: &str,
) -> Option<anyhow::Error> {
    let body_decl = if decl.is_stub {
        decls
            .iter()
            .find(|other| paths_same_location(&other.file, file))
            .unwrap_or(decl)
    } else {
        decl
    };
    let mut lines: Vec<usize> = body_decl
        .duplicate_sections
        .iter()
        .filter(|(path, _)| path == section)
        .map(|(_, info)| info.line)
        .collect();
    if lines.is_empty() {
        return None;
    }
    // The map holds the first claimant (§AR-scanner.2.2.3); the list holds the
    // rest. Sorted so the sites read in file order, as §FS-show.2.2.1 requires.
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
    // §FS-errors.5.2: the same sites the message just named, same order, for the
    // JSON printer to read instead of re-parsing this prose.
    let sites = lines
        .iter()
        .map(|line| FindingSite {
            path: rendered.clone(),
            line: *line,
        })
        .collect();
    Some(
        ShowQueryError {
            code: "ambiguous-section",
            message,
            sites,
        }
        .into(),
    )
}

/// `config` is the *target project's* config — it owns the ID grammar and marker
/// that `render_id` needs. `path_config` is the workspace render config, i.e. the
/// same base `grund list` renders against, so an `<alias>/<ID>` resolved from a
/// workspace root reports a path relative to that root rather than to the member
/// (§FS-config.3.6: paths are relative to *the config root*, and §FS-integrations.3.1.5
/// joins this path against the root `grund-open` discovered).
pub(crate) fn render_show_output_json(
    config: &Config,
    path_config: &Config,
    id: &Id,
    section: Option<&str>,
    mode: ShowRenderMode,
    output: &ShowOutput,
) -> String {
    // Pre-baked JSON (the §FS-show.2.4 E2E manifest) was rendered by
    // show_e2e_case against this same `path_config`, so returning it verbatim
    // keeps the §FS-config.3.6 path promise.
    if let Some(json) = &output.json {
        return json.clone();
    }
    let mut extra = String::new();
    if matches!(mode, ShowRenderMode::Toc) {
        extra.push_str(",\"sections\":[");
        extra.push_str(
            &output
                .sections
                .iter()
                .map(|section| {
                    format!(
                        "{{\"path\":\"{}\",\"title\":\"{}\",\"depth\":{}}}",
                        json_escape(&section.path),
                        json_escape(&section.title),
                        section.depth
                    )
                })
                .collect::<Vec<_>>()
                .join(","),
        );
        extra.push(']');
    }
    // §FS-config.3.4.3, §FS-show.3.1: selected-target metadata follows the
    // section map but precedes the installed resolver's terminal location pair.
    if let Some(title) = config
        .kinds
        .iter()
        .find(|kind| kind.kind == id.kind)
        .and_then(|kind| kind.title.as_deref())
    {
        extra.push_str(&format!(",\"kind_title\":\"{}\"", json_escape(title)));
    }
    format!(
        "{{\"id\":\"{}\",\"section\":{},\"body\":\"{}\"{},\"path\":\"{}\",\"line\":{}}}",
        json_escape(&render_id(&config.grammar, id)),
        match section {
            Some(section) => format!("\"{}\"", json_escape(section)),
            None => "null".to_string(),
        },
        json_escape(&output.body),
        extra,
        json_escape(&display_path(path_config, &output.path)),
        output.line,
    )
}
