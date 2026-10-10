use crate::config::{Frame, LeadSizeWarning, PointSizeUnit, Schema, Strength, measure_point_text};
use crate::grammar::render_id;
use crate::model::{
    Catalog, CheckReport, Declaration, Diagnostic, Id, SectionInfo, TextOverlays,
    paired_declarations,
};
use crate::resolver::{PointBodyCache, point_body_pair};

/// Opt-in point-lead budget checking (§FS-declarations.checks.oversized-lead).
///
/// The scanner owns the site set and `resolver/point_body.rs` owns the slicing. This
/// pass only applies the configured strict threshold and constructs the fixed
/// warning, keeping CLI and LSP on the same checker path. The sites are the
/// stand-ins of each ID's declarations once its stubs are paired by their verdicts
/// (§FS-declarations.stubs.verdict).
pub(super) fn check_oversized_leads(
    findings: &Catalog,
    schema: &Schema,
    frame: Frame<'_>,
    overlays: &TextOverlays,
    report: &mut CheckReport,
) {
    // §FS-declarations.checks.oversized-lead: v1's one budget is a warning; v2's
    // `[schema.leads.words]` puts one on each strength's channel, strongest first.
    let mut budgets: Vec<(Strength, LeadSizeWarning)> = schema
        .leads
        .map(|warning| (Strength::Warn, warning))
        .into_iter()
        .collect();
    budgets.extend(
        schema
            .lead_thresholds
            .iter()
            .filter(|threshold| threshold.strength != Strength::May)
            .map(|threshold| {
                let budget = LeadSizeWarning {
                    max: threshold.value,
                    unit: PointSizeUnit::Words,
                };
                (threshold.strength, budget)
            }),
    );
    budgets.sort_by_key(|(strength, _)| *strength as u8);
    if budgets.is_empty() {
        return;
    }
    let budgets = budgets.as_slice();
    let mut cache = PointBodyCache::new(overlays);
    for (id, declarations) in &findings.declarations {
        // §FS-declarations.checks.oversized-lead.4: the scanned sites, so a stub that
        // stands for a declaration outside the walk is passed over by the slicer.
        let paired = paired_declarations(declarations);
        for declaration in paired.stand_ins() {
            check_oversized_lead_site(
                &mut cache,
                schema,
                frame,
                id,
                declaration,
                None,
                budgets,
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
                    budgets,
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
                    budgets,
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
    budgets: &[(Strength, LeadSizeWarning)],
    report: &mut CheckReport,
) {
    let Ok(Some((lead, _))) = point_body_pair(cache, schema, frame, id, declaration, section)
    else {
        // A stub is broken or has its home outside the scan, unjudged either way
        // (§FS-declarations.checks.oversized-lead.4). A read failure is the scan's.
        return;
    };
    // The strongest budget the lead is over: one finding, on its channel.
    let Some((strength, warning, actual)) = budgets.iter().find_map(|(strength, warning)| {
        let actual = measure_point_text(&lead, warning.unit);
        (actual > warning.max).then_some((*strength, *warning, actual))
    }) else {
        return;
    };
    let mut coordinate = render_id(frame.grammar(), id);
    if let Some((section, _)) = section {
        coordinate.push_str(&schema.ids.section_separator);
        coordinate.push_str(section);
    }
    if let Some(alias) = frame.alias {
        coordinate = format!("{alias}/{coordinate}");
    }
    let channel = match strength {
        Strength::Must => &mut report.errors,
        Strength::Warn => &mut report.warnings,
        Strength::Should | Strength::May => &mut report.suggestions,
    };
    channel.push(Diagnostic {
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
