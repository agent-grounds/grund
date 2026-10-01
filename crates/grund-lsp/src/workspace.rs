/// Resolve an editor folder to the scan root it names. A discovered config owns
/// the scan, including sibling `[scan] include` roots; only a zero-config folder
/// remains its own boundary (§FS-lsp.2.2.1).
fn project_root(folder: &Path) -> Result<PathBuf> {
    let config = effective_config(folder)?;
    if config.config_file.is_some() {
        Ok(canonical_snapshot_path(&config.root))
    } else {
        Ok(canonical_snapshot_path(folder))
    }
}

struct ProjectSnapshot {
    root: PathBuf,
    snapshot: LspSnapshot,
    kind_titles: BTreeMap<String, String>,
    /// The directories `snapshot.scanned_files` lie in, so a file created since
    /// the last scan is still recognized as this project's — a new file under a
    /// symlinked or parent-relative `[scan] include` has no root prefix and is
    /// in no scan yet, and would otherwise look like nobody's (§FS-lsp.2.2).
    scanned_dirs: BTreeSet<PathBuf>,
    /// Where `snapshot`'s records lie, so a lookup about one file does not scan
    /// every record in the workspace (§FS-lsp.responsiveness.1). Derived from
    /// the snapshot here, beside `scanned_dirs`, so it is rebuilt exactly when
    /// the snapshot is and the two cannot disagree.
    index: SnapshotIndex,
}

impl ProjectSnapshot {
    fn new(root: PathBuf, metadata: LspSnapshotWithMetadata) -> Self {
        let snapshot = metadata.snapshot;
        let scanned_dirs = snapshot
            .scanned_files
            .iter()
            .filter_map(|file| file.parent().map(Path::to_path_buf))
            .collect();
        let index = SnapshotIndex::new(&snapshot);
        Self {
            root,
            snapshot,
            kind_titles: metadata.kind_titles,
            scanned_dirs,
            index,
        }
    }

    /// Whether this project's scan may need rebuilding for `path` (already
    /// canonicalized). The directory fallback deliberately over-approximates
    /// newly created files; ownership below never uses that approximation
    /// (§FS-lsp.2.2.2).
    fn might_cover(&self, path: &Path) -> bool {
        path.starts_with(&self.root)
            || self.snapshot.scanned_files.contains(path)
            || path
                .parent()
                .is_some_and(|parent| self.scanned_dirs.contains(parent))
    }

    fn containing_root_depth(&self, path: &Path) -> Option<usize> {
        path.starts_with(&self.root)
            .then(|| self.root.components().count())
    }
}
