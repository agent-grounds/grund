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
use crate::config::exclude_globs::{ExcludeGlobs, matches_only_its_path, spelled_path};
use crate::config::project::Project;
use crate::config::run::{Invocation, Selection};
use crate::model::normalize_path_lexically;

/// The walk the surface above config runs over a version 1 façade: the files
/// that scan reads, each spelled from the façade's root (§AR-config.8.1).
pub(crate) type V1Walk<'a> = dyn Fn(Config) -> Result<Vec<PathBuf>> + 'a;

/// What the version 1 scan reads, walked on first ask and cached as lexical paths
/// for both refusal details (§FS-config-migrate.3.1.4.1, §FS-config-migrate.3.1.5.1).
pub(super) struct Reads<'a> {
    v1: &'a Project,
    walk_root: &'a Path,
    root: PathBuf,
    scan_roots: Vec<PathBuf>,
    walk: &'a V1Walk<'a>,
    files: OnceCell<Vec<PathBuf>>,
}

impl<'a> Reads<'a> {
    /// §FS-config-migrate.3.1.4.1: compare and name paths in lexical coordinates,
    /// including candidate scan roots; keep the original root for the v1 walk.
    pub(super) fn new(
        v1: &'a Project,
        root: &'a Path,
        scan_roots: &[PathBuf],
        walk: &'a V1Walk<'a>,
    ) -> Self {
        Self {
            v1,
            walk_root: root,
            root: normalize_path_lexically(root),
            scan_roots: scan_roots
                .iter()
                .map(|path| normalize_path_lexically(path))
                .collect(),
            walk,
            files: OnceCell::new(),
        }
    }

    /// The files read that a `[scan] exclude` entry would stop, read alone as the
    /// version 2 glob it resembles, each met as the walk meets it, down from the
    /// file's scan root (§FS-config-migrate.3.1.4.1, §AR-config.8.1). Named as
    /// [`Self::named`] names them.
    pub(super) fn stopped_by(&self, entry: &str) -> Result<Vec<String>> {
        let globs = ExcludeGlobs::alone(entry);
        // §FS-config-migrate.3.1.4.1: an entry that stops nothing, or can match only the path it
        // spells, asks the scan only for a path on disk; any other entry always asks.
        if (globs.is_none() || matches_only_its_path(entry)) && !self.on_disk(spelled_path(entry)) {
            return Ok(Vec::new());
        }
        let files = self.files()?;
        let Some(globs) = globs else {
            return Ok(Vec::new());
        };
        let stopped = files.iter().filter(|file| {
            globs
                .excluding_below(&self.root, self.scan_root(file), file)
                .is_some()
        });
        Ok(self.named(stopped))
    }

    /// The files read at the literal `path` or below it: a home is a literal path,
    /// not a glob (§FS-config-migrate.3.1.5.1). Both sides use lexical coordinates
    /// (§FS-config-migrate.3.1.4.1). Named as [`Self::named`] names them.
    pub(super) fn under(&self, path: &str) -> Result<Vec<String>> {
        // §FS-config-migrate.3.1.5.1: an inner home not on disk names nothing, and a walk
        // that cannot run does not fail the run for it.
        if !self.on_disk(path) {
            return Ok(Vec::new());
        }
        let at = normalize_path_lexically(&self.root.join(path));
        Ok(self.named(self.files()?.iter().filter(|file| file.starts_with(&at))))
    }

    /// Nobody reads a path not on disk (§FS-check.4.10.2).
    fn on_disk(&self, path: &str) -> bool {
        normalize_path_lexically(&self.root.join(path)).exists()
    }

    /// §FS-config-migrate.3.1.4.1: the deepest place holding `file`, else the config root.
    fn scan_root(&self, file: &Path) -> &Path {
        self.scan_roots
            .iter()
            .filter(|place| file.starts_with(place))
            .max_by_key(|place| place.components().count())
            .map_or(self.root.as_path(), PathBuf::as_path)
    }

    /// Lexical `files` from the lexical root (§FS-config-migrate.3.1.4.1), with `/`
    /// and sorted as the printed names sort. A name that is not UTF-8 is printed
    /// with its bad bytes replaced, though the scan asked the path itself
    /// (§FS-config-migrate.3.1.9).
    fn named<'f>(&self, files: impl Iterator<Item = &'f PathBuf>) -> Vec<String> {
        let mut out: Vec<String> = files
            .filter_map(|file| file.strip_prefix(&self.root).ok())
            .map(|relative| {
                let names: Vec<_> = relative
                    .components()
                    .map(|name| name.as_os_str().to_string_lossy())
                    .collect();
                names.join("/")
            })
            .collect();
        out.sort();
        out
    }

    /// The version 1 project as its own scan reads it, its path-shaped `[scan]
    /// exclude` entries left in: the scan compares each entry with one directory's
    /// own name, which holds no `/`, so they match nothing, as removing them keeps
    /// it (§FS-config.3.5.16, §FS-config-migrate.3.1.4). A walk that fails as a
    /// whole fails the run, since nothing can then say what version 1 reads.
    /// Cache its paths in lexical coordinates for both refusal details, without
    /// changing read membership or resolving symlinks (§FS-config-migrate.3.1.4.1).
    fn files(&self) -> Result<&[PathBuf]> {
        if let Some(files) = self.files.get() {
            return Ok(files);
        }
        let invocation = Invocation::new(self.walk_root, false, Selection::Project);
        let facade = project_config(self.v1, self.walk_root, &invocation)?;
        let files = (self.walk)(facade)?
            .iter()
            .map(|path| normalize_path_lexically(path))
            .collect();
        Ok(self.files.get_or_init(|| files))
    }
}
