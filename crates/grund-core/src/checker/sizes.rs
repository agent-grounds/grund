use crate::config::{Frame, LeadSizeWarning, Schema, measure_point_text};
use crate::grammar::render_id;
use crate::model::{
    Catalog, CheckReport, Declaration, Diagnostic, Id, SectionInfo, TextOverlays, id_homes,
};
use crate::resolver::{PointBodyCache, point_body_pair};

/// Opt-in point-lead budget checking (§FS-declarations.checks.oversized-lead).
///
/// The scanner owns the site set and `resolver/point_body.rs` owns the slicing. This
/// pass only applies the configured strict threshold and constructs the fixed
/// warning, keeping CLI and LSP on the same checker path.
pub(super) fn check_oversized_leads(
    findings: &Catalog,
    schema: &Schema,
    frame: Frame<'_>,
    overlays: &TextOverlays,
    report: &mut CheckReport,
) {
    let Some(warning) = schema.leads else {
        return;
    };
    let mut cache = PointBodyCache::new(overlays);
    for (id, declarations) in &findings.declarations {
        // §FS-declarations.checks.oversized-lead.4: the scanned sites, so a stub that
        // stands for a home outside the walk is passed over by the slicer.
        let homes = id_homes(declarations);
        for declaration in homes.stand_ins() {
            check_oversized_lead_site(
                &mut cache,
                schema,
                frame,
                id,
                declaration,
                None,
                warning,
                report,
            );

            for (section, info) in &declaration.sections {
                check_oversized_lead_site(
                    &mut cache,
                    schema,
                    frame,
                    id,
                    declaration,
                    Some((section.as_str(), info)),
                    warning,
                    report,
                );
            }
            for (section, info) in &declaration.duplicate_sections {
                check_oversized_lead_site(
                    &mut cache,
                    schema,
                    frame,
                    id,
                    declaration,
                    Some((section.as_str(), info)),
                    warning,
                    report,
                );
            }
        }
    }
}

/// Judge and, when needed, report one declaration or section site using the
/// exact fixed warning contract (§FS-declarations.checks.oversized-lead.1).
#[allow(clippy::too_many_arguments)]
fn check_oversized_lead_site(
    cache: &mut PointBodyCache<'_>,
    schema: &Schema,
    frame: Frame<'_>,
    id: &Id,
    declaration: &Declaration,
    section: Option<(&str, &SectionInfo)>,
    warning: LeadSizeWarning,
    report: &mut CheckReport,
) {
    let Ok(Some((lead, _))) = point_body_pair(cache, schema, frame, id, declaration, section)
    else {
        // A stub is broken or has its home outside the scan, unjudged either way
        // (§FS-declarations.checks.oversized-lead.4). A read failure is the scan's.
        return;
    };
    let actual = measure_point_text(&lead, warning.unit);
    if actual <= warning.max {
        return;
    }
    let mut coordinate = render_id(frame.grammar(), id);
    if let Some((section, _)) = section {
        coordinate.push_str(&schema.ids.section_separator);
        coordinate.push_str(section);
    }
    if let Some(alias) = frame.alias {
        coordinate = format!("{alias}/{coordinate}");
    }
    report.warnings.push(Diagnostic {
        code: "oversized-lead",
        path: Some(declaration.file.clone()),
        line: Some(
            section
                .map(|(_, info)| info.line)
                .unwrap_or(declaration.line),
        ),
        column: None,
        message: format!(
            "{coordinate} lead is {actual} {}, over the configured maximum of {}; move detail into citable child sections, or promote a child section to its own ID after running grund refs {coordinate} --summary",
            warning.unit.as_str(),
            warning.max,
        ),
        sites: Vec::new(),
    authority: Vec::new(),});
}
