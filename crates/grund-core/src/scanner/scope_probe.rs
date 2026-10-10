use std::path::Path;

use super::walk::walk_reads_any_file;
use crate::config::{Frame, Schema, root_scope_roots};

/// Whether the effective configured scope contains a file the scanner would read
/// (§FS-config.3.5, §FS-init.2.2.2). Root selection stays beside the walk so init
/// cannot grow a second, path-existence approximation of scanner policy. Both
/// levels use lazy `any`, stopping at the first readable file in the first root
/// that contains one (§GOAL-fast-feedback).
pub(crate) fn effective_scope_reads_any_file(schema: &Schema, frame: Frame<'_>) -> bool {
    effective_scope_reads_any_file_with(schema, frame, |root| {
        walk_reads_any_file(schema, frame, root)
    })
}

/// Apply the effective root order lazily, separated only so the short-circuit
/// itself can be pinned without replacing scanner behavior (§FS-config.3.5).
pub(crate) fn effective_scope_reads_any_file_with(
    schema: &Schema,
    frame: Frame<'_>,
    mut root_reads_any_file: impl FnMut(&Path) -> bool,
) -> bool {
    root_scope_roots(schema, frame.root(), frame.run.scope.full)
        .iter()
        .any(|root| root_reads_any_file(root))
}
