//! The unsited warning and its migration identity
//! (§FS-check.checks.closed-citation-grounding, §FS-config-migrate.5.2).

use crate::config::{
    ClosedCitationGrounding, Config, Display, GroundingSource, closed_citation_grounding,
    config_file_under, grounding_setting, grounding_unit,
};
use crate::model::{Code, Diagnostic, remedies};

/// A diagnostic's semantic identity and original wording travel together through
/// the report gate (§FS-config-migrate.5.2); provenance is absent from the key.
pub(super) struct ClosedGroundingWarning {
    pub(super) key: String,
    pub(super) diagnostic: Diagnostic,
}

/// Render the shared conflict facts once for each selected loaded project config
/// (§FS-check.checks.closed-citation-grounding).
pub(super) fn warnings(config: &Config, display: Display<'_>) -> Vec<ClosedGroundingWarning> {
    let Some(read) = config.config_file.as_ref() else {
        return Vec::new();
    };
    let file = display.path(&config_file_under(&config.root, read));
    closed_citation_grounding(config.project(), config.run())
        .into_iter()
        .map(|facts| {
            let message = message(&file, config.project().version, &facts);
            ClosedGroundingWarning {
                key: format!(
                    "{:?}",
                    (&file, &facts.kind, facts.strength.as_str(), facts.level)
                ),
                diagnostic: Diagnostic {
                    code: Code::CLOSED_CITATION_GROUNDING,
                    path: None,
                    line: None,
                    column: None,
                    message,
                    sites: Vec::new(),
                    authority: Vec::new(),
                },
            }
        })
        .collect()
}

/// Version-specific setting fragments and the catalog's remedies, with exactly
/// one config breadcrumb (§FS-check.checks.closed-citation-grounding).
fn message(file: &str, version: u32, facts: &ClosedCitationGrounding) -> String {
    let kind = &facts.kind;
    let v2 = version >= 2;
    let own = matches!(facts.source, GroundingSource::Row);
    let setting = match facts.source {
        GroundingSource::Flag => "--require-grounding".to_string(),
        _ if v2 => {
            let table = if own {
                format!("rules.citations.{kind}.grounding")
            } else {
                "rules.citations.grounding".to_string()
            };
            format!(
                "[{table}] {} = \"{}\"",
                facts.strength.as_str(),
                grounding_unit(facts.level)
            )
        }
        GroundingSource::Row => grounding_setting(kind, version, str::to_string),
        GroundingSource::Project => "[reference] require_grounding = true".to_string(),
    };
    let citations = if v2 { "rules.citations" } else { "citations" };
    let fallback = if facts.row_fallback {
        format!("{citations}.{kind}")
    } else {
        citations.to_string()
    };
    let remedy = if own {
        remedies::closed_citation_grounding_own(version, kind)
    } else {
        remedies::closed_citation_grounding_inherited(version, kind)
    };
    format!(
        "{file}: row `{kind}`: {setting} asks for citations, but [{fallback}] default = \"must-not\" permits no target; {remedy}"
    )
}
