//! The invalid value declarations the scan recorded, as `check` reports them (§FS-values.5.1):
//! one derivation `check`'s value rules and `fetch`'s structural clause both read
//! (§FS-fetch.admission.3, §AR-system.principle).

use crate::config::Frame;
use crate::grammar::render_catalog_id;
use crate::model::{Catalog, Code, Diagnostic, paired_declarations, remedies};

/// Each invalid value declaration the scan recorded, as `check` reports it (§FS-values.5.1),
/// once its ID's stubs are paired and a duplicate is left to the duplicate finding. `check` and
/// `fetch`'s structural clause both read it (§FS-fetch.admission.3).
pub(crate) fn invalid_value_declarations<'a>(
    findings: &'a Catalog,
    frame: Frame<'a>,
) -> impl Iterator<Item = Diagnostic> + 'a {
    findings
        .invalid_value_declarations
        .iter()
        .filter_map(move |site| {
            if site.id.as_ref().is_some_and(|id| {
                findings
                    .declarations
                    .get(id)
                    .is_some_and(|decls| paired_declarations(decls).len() > 1)
            }) {
                return None;
            }
            Some(Diagnostic {
                code: Code::INVALID_VALUE_DECLARATION,
                path: Some(site.file.clone()),
                line: Some(site.line),
                column: site.column,
                message: match &site.id {
                    Some(id) => format!(
                        "invalid value declaration for {}: {}; {}",
                        render_catalog_id(frame.grammar(), &findings.declarations, id),
                        site.message,
                        remedies::invalid_value_declaration()
                    ),
                    None => format!(
                        "invalid value declaration: {}; {}",
                        site.message,
                        remedies::invalid_value_declaration()
                    ),
                },
                sites: Vec::new(),
                authority: Vec::new(),
            })
        })
}
