//! The fs-home-moves warning (§FS-check.checks.fs-home-moves): a project loaded
//! with no config file keeps its `FS` declarations in the built-in home, and
//! grund 0.19.0 moves that home, so it hears one release early. Removed in
//! 0.19.0 with the move (§FS-check.checks.fs-home-moves.2).

use super::homes::{KindHomeIndex, paths_same_location_key};
use crate::config::{Frame, LEGACY_FS_FOLDER, Schema};
use crate::grammar::render_id;
use crate::model::{Catalog, CheckReport, Declaration, Diagnostic};

/// §FS-check.checks.fs-home-moves.2: the release that moves the built-in `FS`
/// home, and removes this check with it.
pub(super) const FS_HOME_FLIP: &str = "0.19.0";

/// §FS-check.checks.fs-home-moves: one warning per project, at the lowest-line
/// `FS` declaration in the built-in file home, and only where no config file
/// was read — every config names its own `FS` home or keeps the older folder
/// (§FS-check.checks.fs-home-moves.1).
pub(super) fn check_fs_home_moves(
    catalog: &Catalog,
    schema: &Schema,
    frame: Frame<'_>,
    report: &mut CheckReport,
) {
    if frame.run.config_file.is_some() {
        return;
    }
    let homes = KindHomeIndex::new(schema, frame);
    let Some(home) = homes.single_file_for_kind("FS") else {
        return;
    };
    let Some(first) = first_declaration_in(catalog, "FS", &home.physical_path) else {
        return;
    };
    report.warnings.push(Diagnostic {
        code: "fs-home-moves",
        path: Some(first.file.clone()),
        line: Some(first.line),
        column: None,
        message: format!(
            "{id} is in {file}, the FS home only by default, and grund {FS_HOME_FLIP} moves \
             that default to {LEGACY_FS_FOLDER}/ — run `grund init` to write a grund.toml \
             that keeps FS in {file}, or move the FS declarations into {LEGACY_FS_FOLDER}/",
            id = render_id(frame.grammar(), &first.id),
            file = home.path,
        ),
        sites: Vec::new(),
        authority: Vec::new(),
    });
}

/// The `kind` declaration with the lowest line in the file at `physical_path`;
/// on a tie, the lowest ID, so the pick never depends on map order.
pub(super) fn first_declaration_in<'a>(
    catalog: &'a Catalog,
    kind: &str,
    physical_path: &std::path::Path,
) -> Option<&'a Declaration> {
    catalog
        .declarations
        .iter()
        .filter(|(id, _)| id.kind == kind)
        .flat_map(|(_, declarations)| declarations)
        .filter(|declaration| paths_same_location_key(&declaration.file, physical_path))
        .min_by_key(|declaration| declaration.line)
}
