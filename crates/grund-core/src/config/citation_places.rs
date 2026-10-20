//! Configured citation routes for refused place subjects
//! (§FS-rule-refusals.sentences.rewrite.no-form.place).

use super::project::Schema;
use super::rows::Extent;
use super::validate::citations_table;

/// A scanned non-citable row's explicit places and active-version citation
/// table (§FS-rule-refusals.sentences.rewrite.no-form.place). This is config
/// data, independent of whether a citation obligation has already been written.
#[derive(Clone, Debug)]
pub(crate) struct CitationPlace {
    pub(crate) name: String,
    pub(crate) files: Vec<String>,
    pub(crate) folders: Vec<String>,
    pub(crate) table: String,
}

/// Derive the eligible places once from normalized rows, retaining each
/// scanned file/folder separately (§FS-rule-refusals.sentences.rewrite.no-form.place).
pub(crate) fn citation_places(schema: &Schema, version: u32) -> Vec<CitationPlace> {
    let table = citations_table(version);
    schema
        .rows
        .iter()
        .filter(|row| row.kind.is_none())
        .filter_map(|row| {
            let mut files = Vec::new();
            let mut folders = Vec::new();
            for place in row.places.iter().filter(|place| place.scanned) {
                match &place.extent {
                    Extent::File(path) => files.push(path.clone()),
                    Extent::Folder(path) => folders.push(path.trim_end_matches('/').to_string()),
                    Extent::Complement => {}
                }
            }
            (!files.is_empty() || !folders.is_empty()).then(|| CitationPlace {
                name: row.name.clone(),
                files,
                folders,
                table: format!("{table}.{}", row.name),
            })
        })
        .collect()
}
