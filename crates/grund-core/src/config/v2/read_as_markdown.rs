//! The version-2 load refusals about a name the configuration does not read as Markdown where a
//! Markdown file is meant (§FS-config-v2.checks), which grund 0.23.0 warned about and grund 1.0.0
//! refuses (§DF-markdown-homes-refused-in-1-0.2.1). Each is asked of the lowered source model, so
//! the reader gives them only once every older question has its answer
//! (§FS-config-v2.reader.4.order). Whether a name is read as Markdown is the one reading, never
//! the name (§FS-fmt.2.3.1).

use std::path::Path;

use super::schema::RowDraft;
use super::walk::Reader;
use crate::config::home_remedies::{BECAME_1_0, INDEX_NOT_MARKDOWN, MD_NOT_MARKDOWN};
use crate::config::kind::KindIndex;
use crate::config::{Readings, written_language};

/// §FS-config-v2.checks.config-md-not-markdown: a `[schema.sources.extensions]` entry that maps
/// `md` to a language that does not read Markdown is refused at each line that writes it.
pub(super) fn md_not_markdown(r: &Reader, lines: &[usize]) -> Vec<(usize, String)> {
    let sources = &r.project.schema.sources;
    let Some(language) = written_language(sources, Path::new("name.md")) else {
        return Vec::new();
    };
    if language.reads_markdown() {
        return Vec::new();
    }
    let name = &language.name;
    (lines.iter())
        .map(|line| {
            let message = format!(
                "[schema.sources.extensions] maps `md` to `{name}`, which does not read Markdown; \
                 `.md` is always Markdown; {MD_NOT_MARKDOWN}{BECAME_1_0}"
            );
            (*line, message)
        })
        .collect()
}

/// §FS-config-v2.checks.config-index-not-markdown: an `index` that names a file the
/// configuration does not read as Markdown (§FS-fmt.2.3.1) is refused at its line. A row that
/// names none keeps the default `README.md`, which this never asks
/// (§DF-markdown-homes-refused-in-1-0.2.5).
pub(super) fn index_not_markdown(r: &Reader, row: &RowDraft) -> Option<(usize, String)> {
    let Some((KindIndex::Named(index), line)) = &row.index else {
        return None;
    };
    if Readings::of_written(&r.project.schema.sources, Path::new(index)).markdown {
        return None;
    }
    let kind = &row.name;
    let message = format!(
        "`[schema.kinds.{kind}] index` must name a Markdown file; `{index}` is not read as \
         Markdown; {INDEX_NOT_MARKDOWN}{BECAME_1_0}"
    );
    Some((*line, message))
}
