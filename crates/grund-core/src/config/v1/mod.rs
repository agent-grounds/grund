//! The version-1 reader (§AR-config.2): what a `grund.toml` that omits
//! `grund_config_version`, or sets it to `1`, means as a `Project`. Nothing
//! outside this directory knows a v1 spelling: the section walk is `parse.rs`,
//! the three sections with a grammar of their own are `kind_table.rs`,
//! `citations.rs` and `grounding.rs`, and the defaults every key lowers over
//! are `defaults.rs` and `kind_defaults.rs`, applied here and nowhere else
//! (§AR-config.3.2). `mapping.rs` is the lowering table of §AR-config.3 as data.

mod citations;
mod defaults;
mod grounding;
mod kind_defaults;
mod kind_rows;
mod kind_table;
mod kind_values;
// Read by the unit tests that hold the reader to it (§AR-config.3.3).
#[cfg(test)]
pub(super) mod mapping;
mod parse;
mod scan_block;

use anyhow::Result;
use std::path::Path;

use super::project::Project;

pub(super) use defaults::default_project;
pub(super) use parse::{bail_config, parse_string, parse_usize};
pub(crate) use parse::{parse_string_list, strip_comment};

/// §AR-config.2: lower the v1 file at `read_path` into a `Project`, starting
/// from what an already-authored v1 file means before it says anything
/// (§AR-config.3.2). `report_path` is the path every error names, and `root` the
/// config root a value home must exist under (§FS-config.3.4.9). Only spelling
/// is refused here; the lowered project is judged by `config/validate.rs`.
pub(super) fn read(read_path: &Path, report_path: &Path, root: &Path) -> Result<Project> {
    let mut project = default_project(true);
    parse::parse_config_file(read_path, report_path, root, &mut project)?;
    Ok(project)
}
