//! A view of normalized rows and their configured citation tables
//! (§AR-config.1.3, §FS-config-v2.rules.citations).

use super::project::Schema;
use super::rows::Extent;
use super::validate::citations_table;

/// A scanned non-citable row's explicit places (§AR-config.1.3) and active-version
/// citation table (§FS-config-v2.rules.citations, §FS-config.3.9), independent of
/// whether a citation obligation has already been written.
#[derive(Clone, Debug)]
pub(crate) struct CitationPlace {
    pub(crate) name: String,
    pub(crate) files: Vec<String>,
    pub(crate) folders: Vec<String>,
    pub(crate) table: String,
}

/// Project each normalized row's scanned file/folder places (§AR-config.1.3,
/// §FS-config-v2.schema.places.boundaries.6), reading the active-version table
/// spelling from the config validator (§FS-config-v2.rules.citations, §FS-config.3.9).
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
