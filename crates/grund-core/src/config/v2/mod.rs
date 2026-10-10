//! The version-2 reader (§FS-config-v2, §AR-config.7): what a `grund.toml` that
//! writes `grund_config_version = 2` means as a `Project`. It lowers into the
//! same records v1 does (§DA-config-concern-records.2.4), so the checker reads
//! one project whichever reader produced it; nothing here shares v1's spelling
//! validator, and v1 keeps every meaning it had (§FS-config.5.2).
//!
//! `walk.rs` is the line walk and the bookkeeping every table shares — explicit
//! headers, duplicate keys, the strength-key collision; `tables.rs` names the
//! table a header opens; `schema.rs`, `measures.rs`, `rules.rs` and
//! `presentation.rs` read the keys of each concern; `finish.rs` judges what
//! only the whole file can say and lowers the rows; `defaults.rs` is the v2
//! epoch; and `render.rs` writes a project back out in v2 spelling.

mod defaults;
mod finish;
mod measures;
mod presentation;
mod render;
mod rules;
mod schema;
mod tables;
mod walk;

use anyhow::Result;
use std::path::Path;

use super::project::Project;
use super::v1::strip_comment;

/// §FS-config.5: whether `text` selects the v2 reader — an explicit
/// `grund_config_version = 2` among the top-level keys, before any header. An
/// omitted version, an explicit `1`, and every other value stay with v1, which
/// refuses a version it does not know.
pub(super) fn selects(text: &str) -> bool {
    for raw in text.lines() {
        let line = strip_comment(raw).trim();
        if line.starts_with('[') {
            return false;
        }
        if let Some((key, value)) = line.split_once('=')
            && key.trim() == "grund_config_version"
        {
            return value.trim() == "2";
        }
    }
    false
}

/// §FS-config-v2.reader: lower a v2 file's `text` into a `Project`, refusing
/// every spelling at the line it was written at (§FS-config-v2.reader.4).
/// `report_path` is the path every error names. The meanings both readers share
/// are judged afterwards, once, in `config/validate.rs` (§AR-config.4).
pub(super) fn read(text: &str, report_path: &Path) -> Result<Project> {
    let mut reader = walk::Reader::new(report_path);
    walk::walk(text, &mut reader)?;
    finish::finish(reader)
}
