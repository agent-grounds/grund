//! The version-2 reader (§FS-config-v2, §AR-config.7): what a `grund.toml` that
//! writes `grund_config_version = 2` means as a `Project`. It lowers into the
//! same records v1 does (§DA-config-concern-records.2.4), so the checker reads
//! one project whichever reader produced it; nothing here shares v1's spelling
//! validator, and v1 keeps every meaning it had (§FS-config.5.2).
//!
//! `walk.rs` is the line walk and the bookkeeping every table shares — explicit
//! headers, duplicate keys, the strength-key collision; `tables.rs` names the
//! table a header opens, the keys each admits, and where a v1 table's settings
//! went; `schema.rs`, `measures.rs`, `rules.rs` and
//! `presentation.rs` read the keys of each concern, `fields.rs` a kind's
//! fields (§FS-fields), `front_matter.rs` the front-matter field's own tables,
//! and `sources.rs` the source model, which `sources_model.rs` lowers and
//! renders; `strings.rs` reads every string a key or a value holds
//! (§FS-config-v2.reader.6); `finish.rs` judges what only the whole file can say and lowers the
//! rows; `defaults.rs` is the v2 epoch; and `render.rs` writes a project back
//! out in v2 spelling.

mod defaults;
mod fields;
mod finish;
mod front_matter;
mod measures;
mod places;
mod presentation;
mod render;
mod rules;
mod schema;
mod sources;
mod sources_model;
mod strings;
mod tables;
#[cfg(test)]
mod tests_keys;
#[cfg(test)]
mod tests_walk;
mod walk;

pub(crate) use sources::{holds_glob, literal_suffix};
pub(super) use strings::{read_string, read_string_list};
#[cfg(test)]
pub(super) use tables::V1_TABLES;
pub(super) use tables::{admitted, v1_homes};
pub(crate) use walk::{OpenLine, joined_value};

use anyhow::Result;
use std::path::Path;

use super::project::Project;
use super::v1::strip_comment;

/// §FS-config.5: whether `text` selects the v2 reader — an explicit
/// `grund_config_version = 2` among the top-level keys, before any header. An
/// omitted version, an explicit `1`, and every other value stay with v1, which
/// refuses a version it does not know.
pub(crate) fn selects(text: &str) -> bool {
    version(text) == Some("2")
}

/// §FS-config.5: the `grund_config_version` `text` writes, as it is written —
/// the first top-level key of that name, before any header — or `None` where
/// the top level names no version. Every reader is chosen by this one reading.
pub(crate) fn version(text: &str) -> Option<&str> {
    for raw in text.lines() {
        let line = strip_comment(raw).trim();
        if line.starts_with('[') {
            return None;
        }
        if let Some((key, value)) = line.split_once('=')
            && key.trim() == "grund_config_version"
        {
            return Some(value.trim());
        }
    }
    None
}

/// §FS-config-v2.reader: lower a v2 file's `text` into a `Project`, refusing
/// every spelling at the line it was written at (§FS-config-v2.reader.4).
/// `report_path` is the path every error names, and `root` the config root a
/// value home must exist under (§FS-fields.forms). The meanings both readers
/// share are judged afterwards, once, in `config/validate.rs` (§AR-config.4).
pub(super) fn read(text: &str, report_path: &Path, root: &Path) -> Result<Project> {
    let mut reader = walk::Reader::new(report_path);
    walk::walk(text, &mut reader)?;
    finish::finish(reader, root)
}
