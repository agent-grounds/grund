use super::ambiguity::{ambiguous_id_refusal, ambiguous_section_refusal};
use crate::config::{Config, display_path};
use crate::grammar::render_id;
use crate::model::{
    Catalog, Declaration, DeclarationSource, Id, SectionInfo, ShowOutput, ShowRenderMode,
    StubResolution, TextOverlays, format_path, id_homes, json_escape, paths_same_location,
};
use crate::resolver::{
    extract_declaration_body, heading_anchor, section_site_anchor, show_e2e_case,
    takes_heading_anchor,
};
use anyhow::{Result, anyhow};

#[cfg(test)]
pub(crate) fn show_declaration(
    config: &Config,
    path_config: &Config,
    findings: &Catalog,
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
    .map(|(output, _)| output)
}

/// `path_config` renders every path this function reports (§FS-config.3.6) — the
/// sites a refusal names (§FS-errors.3.1) as well as the E2E manifest's, whose
/// JSON is baked here rather than in `render_show_output_json`. It must be the
/// same config the caller hands that renderer, so a member's declaration reports
/// against the root the rest of the run spells its paths from
/// (§FS-workspace.8.1.2). `config` stays the *project's*: it owns the ID grammar
/// `render_id` reads and the tree the body is read out of.
///
/// Beside the output it returns the record the body was read out of, a stub's
/// target declaration rather than the stub (§FS-show.2.3.7), which is the record
/// `render_show_output_json` derives the heading anchors from.
///
/// Which record that is, and whether the ID is refused instead, is the one
/// derivation of its homes from the verdict the scan recorded on each stub
/// (§FS-declarations.checks.broken-stub.4): nothing here reads a stub's target.
pub(crate) fn show_declaration_with_overlays<'a>(
    config: &Config,
    path_config: &Config,
    findings: &'a Catalog,
    id: &Id,
    section: Option<&str>,
    mode: ShowRenderMode,
    include_heading: bool,
    overlays: &TextOverlays,
) -> Result<(ShowOutput, &'a Declaration)> {
    let not_found = || {
        anyhow!(crate::model::OperationDiagnostic::new(
            "query",
            "not-found",
            format!("ID not found: {}", render_id(&config.grammar, id))
        ))
    };
    let decls = findings.declarations.get(id).ok_or_else(not_found)?;
    let homes = id_homes(decls);
    // §FS-show.2.2.1: share the independent-home refusal with refs (§FS-refs.4).
    if let Some(refusal) = ambiguous_id_refusal(config, path_config, &homes, id) {
        return Err(refusal.into());
    }
    let home = homes.sole().ok_or_else(not_found)?;
    let record = home.record;
    if matches!(record.source, DeclarationSource::Json { .. }) {
        let output = show_json_value(config, id, record, section)?;
        return Ok((output, record));
    }
    if let Some(case) = &record.e2e_case {
        // §FS-show.2.4: the kind's title is presentation's, read here.
        let title = config
            .kinds
            .iter()
            .find(|kind| kind.kind == id.kind)
            .and_then(|kind| kind.title.as_deref());
        let frame = config.frame().displayed_by(path_config.display());
        let output = show_e2e_case(config.schema(), frame, title, id, case, section, mode)?;
        return Ok((output, record));
    }
    if let (Some(verdict), Some(target)) = (&record.stub_resolution, &record.defined_in)
        && verdict.is_broken()
    {
        // §FS-show.2.3.4: the first line for a missing target, the second for the rest.
        let rendered = render_id(&config.grammar, id);
        let reason = match verdict {
            StubResolution::Missing => "which does not exist".to_string(),
            _ => format!("which contains no inline declaration of {rendered}"),
        };
        return Err(anyhow!(crate::model::OperationDiagnostic::new(
            "query",
            "broken-stub",
            format!(
                "broken stub: {rendered} (stub at {}:{} points at {}, {reason})",
                display_path(path_config, &record.file),
                record.line,
                format_path(target),
            )
        )));
    }
    if let Some(section) = section
        && let Some(refusal) = ambiguous_section_refusal(config, path_config, record, id, section)
    {
        return Err(refusal.into());
    }
    if let Some(section) = section
        && !record.sections.contains_key(section)
    {
        return Err(anyhow!(crate::model::OperationDiagnostic::new(
            "query",
            "missing-section",
            format!(
                "section not found: {}{}{}",
                render_id(&config.grammar, id),
                config.section_separator,
                section
            )
        )));
    }
    // §FS-show.2.3.7: a stub reads its target's declaration, found by the ID.
    let output = extract_declaration_body(
        &record.file,
        id,
        record,
        section,
        mode,
        include_heading,
        config.schema(),
        config.frame(),
        overlays,
    )?;
    Ok((output, record))
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
                anyhow!(crate::model::OperationDiagnostic::new(
                    "query",
                    "missing-section",
                    format!(
                        "section not found: {}{}{}",
                        render_id(&config.grammar, id),
                        config.section_separator,
                        section
                    )
                ))
            })?;
            let value = info.value.as_ref().ok_or_else(|| {
                anyhow!(crate::model::OperationDiagnostic::new(
                    "query",
                    "missing-section",
                    format!(
                        "section not found: {}{}{}",
                        render_id(&config.grammar, id),
                        config.section_separator,
                        section
                    )
                ))
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

/// `config` is the *target project's* config — it owns the ID grammar and marker
/// that `render_id` needs. `path_config` is the workspace render config, i.e. the
/// same base `grund list` renders against, so an `<alias>/<ID>` resolved from a
/// workspace root reports a path relative to that root rather than to the member
/// (§FS-config.3.6: paths are relative to *the config root*, and §FS-integrations.3.1.5
/// joins this path against the root `grund-open` discovered).
///
/// `home` is the record `show_declaration_with_overlays` read the body out of.
pub(crate) fn render_show_output_json(
    config: &Config,
    path_config: &Config,
    home: &Declaration,
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
    let anchors = ShowAnchors::of(config, home, section, output);
    let mut extra = String::new();
    if matches!(mode, ShowRenderMode::Toc) {
        extra.push_str(",\"sections\":[");
        extra.push_str(
            &output
                .sections
                .iter()
                .zip(&anchors.sections)
                .map(|(section, anchor)| {
                    format!(
                        "{{\"path\":\"{}\",\"title\":\"{}\",\"depth\":{},\"anchor\":{}}}",
                        json_escape(&section.path),
                        json_escape(&section.title),
                        section.depth,
                        json_anchor(anchor.as_deref())
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
    // §FS-show.3.1.3.1: the heading anchor always precedes the closing location pair.
    extra.push_str(&format!(
        ",\"anchor\":{}",
        json_anchor(anchors.selected.as_deref())
    ));
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

fn json_anchor(anchor: Option<&str>) -> String {
    match anchor {
        Some(anchor) => format!("\"{}\"", json_escape(anchor)),
        None => "null".to_string(),
    }
}

/// The heading anchors a JSON show object carries (§FS-show.3.1.3.1): the
/// selected coordinate's, and one per `--toc` entry in the entries' order. Each
/// is the fragment `fmt --cross-refs` derives under the target project's profile
/// (`config`), or `None` where the site has no heading anchor — a source or JSON
/// home, or the `none` profile. Derived from the record the body was read out
/// of, so a stub reports its target's (null for source), scanned or not
/// (§FS-show.2.3.7), and never its own heading.
struct ShowAnchors {
    selected: Option<String>,
    sections: Vec<Option<String>>,
}

impl ShowAnchors {
    fn of(config: &Config, decl: &Declaration, section: Option<&str>, output: &ShowOutput) -> Self {
        let none = || Self {
            selected: None,
            sections: vec![None; output.sections.len()],
        };
        let presentation = &config.project().presentation;
        if !takes_heading_anchor(&output.path, presentation)
            || decl.is_stub
            || !paths_same_location(&decl.file, &output.path)
        {
            return none();
        }
        // §FS-show.3.1.3.1: each `--toc` entry anchors on its own heading site.
        // The entries are the recorded sites after the selected heading, in
        // document order; a path the cursor cannot place falls back to the map.
        let mut sites: Vec<(&str, &SectionInfo)> = decl
            .sections
            .iter()
            .map(|(path, info)| (path.as_str(), info))
            .chain(
                decl.duplicate_sections
                    .iter()
                    .map(|(path, info)| (path.as_str(), info)),
            )
            .filter(|(_, info)| info.line > output.line)
            .collect();
        sites.sort_by_key(|(_, info)| info.line);
        let mut cursor = 0;
        let sections = output
            .sections
            .iter()
            .map(|entry| {
                let site = match sites[cursor..]
                    .iter()
                    .position(|(path, _)| *path == entry.path)
                {
                    Some(offset) => {
                        cursor += offset + 1;
                        Some(sites[cursor - 1].1)
                    }
                    None => decl.sections.get(&entry.path),
                };
                site.map(|site| section_site_anchor(site, presentation))
            })
            .collect();
        Self {
            selected: heading_anchor(decl, section, presentation, config.frame()),
            sections,
        }
    }
}
