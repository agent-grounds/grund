//! The point-body pair (§AR-system.2.10): one catalog site's lead and full body,
//! sliced through the exact `show` slicer beside it, for the two rules that
//! measure a lead — the size catalog of §FS-list.3.4 and the lead-budget warning
//! of §FS-declarations.checks.oversized-lead.
//!
//! It is one file rather than a function in `body.rs` because it is the *answer*
//! a measurement reads, not the slicer: it chooses the body a site has at all —
//! a JSON member's recorded slice, a front-matter coordinate's slice, an e2e
//! case's manifest, a stub's nothing — and only then falls through to the
//! slicer (§FS-list.2).
//!
//! Both of its readers slice the lead through that one slicer, which is why
//! neither can be a pure function in a lower component: the answer is a function
//! of the findings a run loaded (§AR-system.4). The one thing it reads that is
//! not — flattening the formatter's cross-reference wrappers, so a measured body
//! is the body `show` prints (§FS-show.3.2) — is recognition, and lexical, so it
//! is `grammar/fmt_cross_refs.rs` and reads downward.

use anyhow::Result;

use super::body::slice_site;
use super::e2e_body::show_e2e_case;
use crate::config::{Frame, Schema};
use crate::grammar::{flatten_cross_ref_links, render_declaration_id};
use crate::model::{Declaration, DeclarationSource, FileRecords, Id, SectionInfo, ShowRenderMode};
use crate::scanner::SliceSink;

/// Return the lead/full text pair for one catalog site. JSON values and E2E
/// cases already carry their canonical show bodies in scanner records; text
/// declarations are sliced out of the file record the scan kept, lead and full in
/// one walk (§AR-scanner.2.8). A stub that reaches it has no body to
/// measure: either it is broken (§FS-list.2.5, §FS-list.3.4), or its home lies
/// outside the walk and the caller is `check`'s lead budget, which judges scanned
/// sites only (§FS-declarations.checks.oversized-lead.4). The size catalog passes
/// such a stub's home instead (§FS-list.3.4.6).
pub(crate) fn point_body_pair(
    files: &FileRecords,
    schema: &Schema,
    frame: Frame<'_>,
    id: &Id,
    declaration: &Declaration,
    section: Option<(&str, &SectionInfo)>,
) -> Result<Option<(String, String)>> {
    if declaration.is_stub {
        return Ok(None);
    }
    if matches!(declaration.source, DeclarationSource::Json { .. }) {
        let body = match section {
            Some((_, info)) => info
                .value
                .as_ref()
                .map(|value| value.source_slice.clone())
                .unwrap_or_default(),
            None => match &declaration.source {
                DeclarationSource::Json { member_slice, .. } => member_slice.clone(),
                DeclarationSource::Text => unreachable!("guarded JSON source"),
            },
        };
        return Ok(Some((body.clone(), body)));
    }
    // §FS-fields.front-matter.surfaces: lead and full both measure the slice.
    if let Some(slice) = section.and_then(|(_, info)| info.front_matter.as_ref()) {
        return Ok(Some((slice.text.clone(), slice.text.clone())));
    }
    if let Some(case) = &declaration.e2e_case {
        let spelling = render_declaration_id(frame.grammar(), declaration);
        let lead = show_e2e_case(
            schema,
            frame,
            None,
            &spelling,
            case,
            None,
            ShowRenderMode::Default,
        )?
        .body;
        let full = show_e2e_case(
            schema,
            frame,
            None,
            &spelling,
            case,
            None,
            ShowRenderMode::Full,
        )?
        .body;
        return Ok(Some((lead, full)));
    }

    let sinks = vec![
        SliceSink::new(ShowRenderMode::Default, false),
        SliceSink::new(ShowRenderMode::Full, false),
    ];
    let section_path = section.map(|(path, _)| path);
    let claimant = section.map(|(_, info)| info.line);
    let (_, views) = slice_site(
        files,
        id,
        declaration,
        section_path,
        claimant,
        sinks,
        schema,
        frame,
    )?;
    let mut views = views.into_iter().map(SliceSink::finish);
    let (mut lead, _) = views.next().unwrap_or_default();
    let (mut full, _) = views.next().unwrap_or_default();
    // §FS-config-v2.schema.sources: Markdown-ness is the file's language's.
    let reading = frame.compiled.reading(schema, &declaration.file);
    // Text and JSON `show` flatten generated wrappers; point measurements promise
    // those same bytes (§FS-show.3.2.2, §FS-list.3.4.1). Markdown measurements retain
    // authored destination bytes inside fences.
    lead = flatten_cross_ref_links(&lead, reading.lexical, reading.markdown);
    full = flatten_cross_ref_links(&full, reading.lexical, reading.markdown);
    Ok(Some((lead, full)))
}
