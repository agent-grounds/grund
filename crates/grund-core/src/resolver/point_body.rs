//! The point-body pair (§AR-system.2.10): one catalog site's lead and full body,
//! sliced through the exact `show` slicer beside it, for the two rules that
//! measure a lead — the size catalog of §FS-list.3.4 and the lead-budget warning
//! of §FS-check.4.13.
//!
//! It is one file rather than a function in `body.rs` because it is the *answer*
//! a measurement reads, not the slicer: it chooses the body a site has at all —
//! a JSON member's recorded slice, an e2e case's manifest, a retained stub's
//! nothing — and only then falls through to the slicer (§FS-list.2).
//!
//! Both of its readers slice the lead through that one slicer, which is why
//! neither can be a pure function in a lower component: the answer is a function
//! of the findings a run loaded (§AR-system.4).

use anyhow::Result;

use super::body::{PointBodyCache, PointBodySite, extract_declaration_body_cached};
use super::e2e_body::show_e2e_case;
use crate::config::Config;
use crate::model::{Declaration, DeclarationSource, Id, SectionInfo, ShowRenderMode};
// §AR-system.4: one read above this component — the cross-reference flattening
// of §DF-show-cross-ref-flattening from the writers' `fmt_links.rs`, the inverse
// of the formatter's own wrapper, which a measured body has to agree with.
use crate::writers::flatten_cross_ref_links;

/// Return the lead/full text pair for one catalog site. JSON values and E2E
/// cases already carry their canonical show bodies in scanner records; text
/// declarations use the cached show slicer. A retained stub is broken (healthy
/// stub rows collapse onto their inline home) and therefore unmeasurable
/// (§FS-list.2, §FS-list.3.4).
pub(crate) fn point_body_pair(
    cache: &mut PointBodyCache<'_>,
    config: &Config,
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
    if let Some(case) = &declaration.e2e_case {
        let lead = show_e2e_case(config, config, id, case, None, ShowRenderMode::Default)?.body;
        let full = show_e2e_case(config, config, id, case, None, ShowRenderMode::Full)?.body;
        return Ok(Some((lead, full)));
    }

    let section_path = section.map(|(path, _)| path);
    let site = PointBodySite {
        declaration_line: declaration.line,
        declaration_body_end: (section.is_some() || declaration.body_end > declaration.line)
            .then_some(declaration.body_end),
        section_line: section.map(|(_, info)| info.line),
    };
    let mut lead = extract_declaration_body_cached(
        cache,
        &declaration.file,
        id,
        section_path,
        ShowRenderMode::Default,
        false,
        config,
        Some(site),
    )?
    .body;
    let mut full = extract_declaration_body_cached(
        cache,
        &declaration.file,
        id,
        section_path,
        ShowRenderMode::Full,
        false,
        config,
        Some(site),
    )?
    .body;
    // Text and JSON `show` flatten generated cross-reference wrappers before
    // exposing their bodies; point measurements promise those same bytes
    // (§FS-show.3.2, §FS-list.3.4).
    lead = flatten_cross_ref_links(&lead, config);
    full = flatten_cross_ref_links(&full, config);
    Ok(Some((lead, full)))
}
