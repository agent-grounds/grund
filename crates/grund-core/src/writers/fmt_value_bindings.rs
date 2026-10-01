//! Formatter protection for exact value bindings. Authority comes from the
//! scanner's local/workspace records so formatting cannot turn a binding into a
//! link and silently disable comparison (§FS-values.8, §AR-scanner.2.3.5).

use crate::checker::{
    binding_aims_at_embedded_value_authority, binding_target_has_any_value_authority,
};
use crate::config::Config;
use crate::grammar::MarkdownLineCitation;
use crate::model::{
    Findings, component_text_is_valid, value_binding_section_ends_in_coordinate,
    value_binding_section_shape_is_valid,
};
use crate::resolver::WorkspaceContext;

pub(super) fn markdown_citation_is_value_binding(
    line: &str,
    citation: &MarkdownLineCitation,
    config: &Config,
    findings: &Findings,
    workspace: Option<&WorkspaceContext>,
) -> bool {
    let (target_config, target_findings) = match citation.namespace.as_deref() {
        Some(alias) => {
            let Some(project) = workspace.and_then(|workspace| workspace.project_by_alias(alias))
            else {
                return false;
            };
            (&project.config, &project.findings)
        }
        None => (config, findings),
    };
    let section = citation.section.as_deref();
    // §FS-values.8: a component binding under either authority is protected by
    // the test that made it one, and a bare whole-value ID by the authority it
    // binds whole, whether the space rule lets it agree or refuses it
    // (§FS-values.3.1.2, §FS-values.9.2). A path aimed at a marked or chapter
    // root, a declared chapter, or below a component is protected by the one
    // predicate check's refusals share.
    let is_binding_shape = section.is_none_or(|section| {
        value_binding_section_shape_is_valid(section)
            && value_binding_section_ends_in_coordinate(section)
    }) && binding_target_has_any_value_authority(
        target_findings,
        target_config,
        &citation.id,
        section,
    );
    if !is_binding_shape
        && !section.is_some_and(|section| {
            binding_aims_at_embedded_value_authority(
                target_findings,
                target_config,
                &citation.id,
                section,
            )
        })
    {
        return false;
    }
    let Some(before_marker) = line[..citation.marker_start].strip_suffix(" (") else {
        return false;
    };
    let Some(close_tick) = before_marker.len().checked_sub(1) else {
        return false;
    };
    if before_marker.as_bytes().get(close_tick) != Some(&b'`')
        || !line[citation.token_end..].starts_with(')')
    {
        return false;
    }
    let Some(open_tick) = before_marker[..close_tick].rfind('`') else {
        return false;
    };
    component_text_is_valid(&before_marker[open_tick + 1..close_tick])
}
