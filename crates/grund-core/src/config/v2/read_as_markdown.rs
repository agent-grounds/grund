//! The version-2 load warnings about a name the configuration does not read as Markdown where a
//! Markdown file is meant (§FS-config-v2.checks): each loads as written, and says what grund
//! 1.0.0 refuses (§REQ-backwards-compatibility.2). Whether a name is read as Markdown is the one
//! reading, never the name (§FS-fmt.2.3.1).

use std::path::Path;

use super::schema::RowDraft;
use super::walk::Reader;
use crate::config::home_requirement::UNTIL_1_0;
use crate::config::kind::KindIndex;
use crate::config::{Readings, written_language};
use crate::model::{Code, remedies};

/// §FS-config-v2.checks.config-md-not-markdown: a `[schema.sources.extensions]` entry that maps
/// `md` to a language that does not read Markdown loads, and is read as written until 1.0.0.
pub(super) fn md_not_markdown(r: &Reader, lines: &[usize]) -> Vec<(usize, Code, String)> {
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
                 `.md` is always Markdown, and until then a `.md` file is read as `{name}`; \
                 {}{UNTIL_1_0}",
                remedies::config_md_not_markdown()
            );
            (*line, Code::CONFIG_MD_NOT_MARKDOWN, message)
        })
        .collect()
}

/// §FS-config-v2.checks.config-index-not-markdown: an `index` that names a file the
/// configuration does not read as Markdown (§FS-fmt.2.3.1) loads, and its folders are judged as
/// under `index = false` until 1.0.0.
pub(super) fn index_not_markdown(r: &Reader, row: &RowDraft) -> Option<(usize, Code, String)> {
    let Some((KindIndex::Named(index), line)) = &row.index else {
        return None;
    };
    if Readings::of_written(&r.project.schema.sources, Path::new(index)).markdown {
        return None;
    }
    let kind = &row.name;
    let message = format!(
        "`[schema.kinds.{kind}] index` must name a Markdown file; `{index}` is not read as \
         Markdown, so until then its folders are judged as under `index = false`; {}{UNTIL_1_0}",
        remedies::config_index_not_markdown()
    );
    Some((*line, Code::CONFIG_INDEX_NOT_MARKDOWN, message))
}
