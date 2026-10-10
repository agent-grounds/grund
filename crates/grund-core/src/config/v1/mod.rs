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
use kind_table::ParsedKind;

pub(super) use defaults::default_project;
pub(super) use parse::{bail_config, parse_bool, parse_string, parse_usize};
pub(crate) use parse::{parse_string_list, strip_comment};

/// A v1 file read, its `[[kinds]]` table not yet lowered (§AR-config.2).
/// The two steps are apart so the `[reference]` meanings are judged before
/// the table's refusals, the order v1 always reported a file's first error in
/// (§AR-config.4).
pub(super) struct Read {
    pub(super) project: Project,
    kinds: Option<Vec<ParsedKind>>,
}

/// §AR-config.2: lower the v1 file's `text` into a `Project`, starting
/// from what an already-authored v1 file means before it says anything
/// (§AR-config.3.2), all but its `[[kinds]]` table, which [`Read::lower_kinds`]
/// lowers. `report_path` is the path every error names. Only spelling is refused
/// here; the lowered project is judged by `config/validate.rs`.
pub(super) fn read_sections(text: &str, report_path: &Path) -> Result<Read> {
    let mut project = default_project(true);
    let kinds = parse::parse_config_file(text, report_path, &mut project)?;
    Ok(Read { project, kinds })
}

impl Read {
    /// Lower the `[[kinds]]` table, refusing an entry no row can hold
    /// (§AR-config.3.1). `root` is the config root a value home must exist under
    /// (§FS-config.3.4.9).
    pub(super) fn lower_kinds(self, report_path: &Path, root: &Path) -> Result<Project> {
        let Read { mut project, kinds } = self;
        if let Some(kinds) = kinds {
            kind_rows::lower_parsed_kinds(report_path, root, kinds, &mut project)?;
        }
        Ok(project)
    }
}

/// The whole v1 file as a `Project`, unjudged: both steps of [`Read`] in turn.
#[cfg(test)]
pub(super) fn read(read_path: &Path, report_path: &Path, root: &Path) -> Result<Project> {
    let text = std::fs::read_to_string(read_path)?;
    read_sections(&text, report_path)?.lower_kinds(report_path, root)
}
