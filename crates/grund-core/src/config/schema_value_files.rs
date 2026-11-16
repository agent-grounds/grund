//! §FS-schema.4.1.4: where a value kind's values are written, lowered from its row
//! for the schema view (§AR-config.6.3): the one home `grund fetch` writes, or each
//! home with the forms the scan reads there, asked by the rule the scan reads a value
//! home by (§FS-values.2.2), so the view names no form the scan does not read.

use std::path::Path;

use super::file_reading::written_value_format;
use super::project::Project;
use super::rows::{Extent, Row};
use super::schema_slot::{ValueFiles, ValueForm, ValueHome};

/// §FS-schema.4.1.4: where a value kind's values are written — the one home
/// `grund fetch` writes, or each home with the forms the scan reads there, a
/// folder's asked of a Markdown, a JSON and a YAML name inside it by the rule the
/// scan reads a value home by.
pub(super) fn value_files(project: &Project, row: &Row) -> ValueFiles {
    let sources = &project.schema.sources;
    let form = |name: &Path| ValueForm::from(written_value_format(sources, name));
    if row.fetches() {
        // §FS-fetch.2: the one home a fetched row writes, a file before a folder.
        let home = row.fetch_home().map(|home| {
            let folder = row.file().is_none();
            let place = if folder {
                folder_place(home)
            } else {
                home.to_string()
            };
            let forms = Vec::new();
            ValueHome {
                place,
                folder,
                forms,
            }
        });
        let homes = home.into_iter().collect();
        return ValueFiles {
            fetched: true,
            homes,
        };
    }
    let homes = row.places.iter().filter_map(|place| match &place.extent {
        Extent::File(path) => Some(ValueHome {
            place: path.clone(),
            folder: false,
            forms: vec![form(Path::new(path))],
        }),
        Extent::Folder(path) => {
            let folder = Path::new(path);
            let probes = ["md", "json", "yaml"].map(|ext| form(&folder.join(format!("x.{ext}"))));
            let mut forms = probes.to_vec();
            forms.sort();
            forms.dedup();
            Some(ValueHome {
                place: folder_place(path),
                folder: true,
                forms,
            })
        }
        Extent::Complement => None,
    });
    ValueFiles {
        fetched: false,
        homes: homes.collect(),
    }
}

/// §FS-schema.4: a folder as the view spells a place, with a trailing `/`.
fn folder_place(path: &str) -> String {
    format!("{}/", path.trim_end_matches('/'))
}
