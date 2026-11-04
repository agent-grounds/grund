//! The ordinary per-file pass over an integration's in-memory snapshot, before
//! the writer installs it (§FS-fetch.3.sections, §AR-scanner.2).

use std::path::Path;
use std::sync::Arc;

use super::file_pass::scan_file_text;
use crate::config::{Frame, Schema};
use crate::model::{Catalog, FileText};

/// Record provider text as the selected project reads its destination, without
/// reading that destination or walking the project (§FS-fetch.3.sections).
/// All paths, collisions, body spans and line numbers come from the existing
/// scanner pass; citations are recorded without being resolved.
pub(crate) fn scan_snapshot_text(
    path: &Path,
    text: &str,
    schema: &Schema,
    frame: Frame<'_>,
) -> anyhow::Result<Catalog> {
    let mut records = Catalog::default();
    scan_file_text(
        path,
        FileText::Buffer(Arc::from(text)),
        schema,
        frame,
        &mut records,
        &[],
    )?;
    Ok(records)
}
