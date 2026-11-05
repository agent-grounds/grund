//! §FS-config-migrate.3.1.4.1, §FS-config-migrate.3.1.5.1: the files a refusal's
//! detail names are the ones the version 1 scan of this tree reads, by the scan's
//! own walk (§AR-scanner.1). Config may not read the scanner (§AR-system.4), so
//! the surface above it hands that walk down, as it hands `scope_on_disk`
//! (§AR-config.8.1). There is one read set a run, asked of that walk at most once,
//! and only by a refusal that has a detail to fill.

use anyhow::Result;
use std::cell::OnceCell;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::config::discovery::project_config;
use crate::config::project::Project;
use crate::config::run::{Invocation, Selection};
use crate::model::normalize_path_lexically;

/// The walk the surface above config runs over a version 1 façade: the files
/// that scan reads, each spelled from the façade's root (§AR-config.8.1).
pub(crate) type V1Walk<'a> = dyn Fn(Config) -> Result<Vec<PathBuf>> + 'a;

/// What the version 1 scan of the tree at `root` reads, walked on first ask.
pub(super) struct Reads<'a> {
    v1: &'a Project,
    root: &'a Path,
    walk: &'a V1Walk<'a>,
    files: OnceCell<Vec<PathBuf>>,
}

impl<'a> Reads<'a> {
    pub(super) fn new(v1: &'a Project, root: &'a Path, walk: &'a V1Walk<'a>) -> Self {
        Self {
            v1,
            root,
            walk,
            files: OnceCell::new(),
        }
    }

    /// The files read at `path` or below it, from the root and with `/`, sorted
    /// as the printed names sort. An exclude entry is no scan root, so one that
    /// names a single file names it only where the scan reads it
    /// (§FS-config-migrate.3.1.4.1). A name that is not UTF-8 is printed with its
    /// bad bytes replaced, though the scan asked the path itself
    /// (§FS-config-migrate.3.1.9).
    pub(super) fn under(&self, path: &str) -> Result<Vec<String>> {
        let at = normalize_path_lexically(&self.root.join(path));
        let mut out: Vec<String> = self
            .files()?
            .iter()
            .filter(|file| file.starts_with(&at))
            .filter_map(|file| file.strip_prefix(self.root).ok())
            .map(|relative| {
                let names: Vec<_> = relative
                    .components()
                    .map(|name| name.as_os_str().to_string_lossy())
                    .collect();
                names.join("/")
            })
            .collect();
        out.sort();
        Ok(out)
    }

    /// The version 1 project as its own scan reads it, its path-shaped `[scan]
    /// exclude` entries left in: the scan compares each entry with one directory's
    /// own name, which holds no `/`, so they match nothing, as removing them keeps
    /// it (§FS-config.3.5.16, §FS-config-migrate.3.1.4). A walk that fails as a
    /// whole fails the run, since nothing can then say what version 1 reads.
    fn files(&self) -> Result<&[PathBuf]> {
        if let Some(files) = self.files.get() {
            return Ok(files);
        }
        let invocation = Invocation::new(self.root, false, Selection::Project);
        let facade = project_config(self.v1, self.root, &invocation)?;
        let files = (self.walk)(facade)?;
        Ok(self.files.get_or_init(|| files))
    }
}
