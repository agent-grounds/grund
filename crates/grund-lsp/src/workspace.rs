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
    /// Cached core authoring context (§FS-lsp.1.6.4). Optional only for the
    /// existing transport fixtures that construct navigation snapshots.
    completion: Option<grund_core::LspCompletionContext>,
    /// The context the snapshot was loaded from, which a citation hover answers
    /// from (§FS-lsp.responsiveness.3, §AR-lsp.5.2). Optional for the same
    /// fixtures as `completion`.
    show: Option<grund_core::LspShowContext>,
    /// The directories `snapshot.scanned_files` lie in, so a file created since
    /// the last scan is still recognized as this project's — a new file under a
    /// symlinked or parent-relative `[scan] include` has no root prefix and is
    /// in no scan yet, and would otherwise look like nobody's (§FS-lsp.2.2).
    scanned_dirs: BTreeSet<PathBuf>,
    /// The files the snapshot's run read outside its scan, from an open buffer
    /// or from disk — a stub's target in another editor folder, say —
    /// canonicalized: an edit to one of them, or its close back to disk, makes
    /// this snapshot stale as surely as an edit to a scanned file (§AR-lsp.2.1).
    read_outside_scan: BTreeSet<PathBuf>,
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
            completion: None,
            show: None,
            scanned_dirs,
            read_outside_scan: BTreeSet::new(),
            index,
        }
    }

    /// Install authoring and navigation from the same scan (§FS-lsp.1.6.4).
    fn with_completion(root: PathBuf, loaded: grund_core::LspSnapshotWithCompletion) -> Self {
        let mut project = Self::new(root, loaded.metadata);
        let scanned = &project.snapshot.scanned_files;
        project.read_outside_scan = loaded
            .show
            .read_files()
            .into_iter()
            .filter(|path| !scanned.contains(path))
            .map(|path| canonical_snapshot_path(&path))
            .filter(|path| !scanned.contains(path))
            .collect();
        project.completion = Some(loaded.completion);
        project.show = Some(loaded.show);
        project
    }

    /// Whether this project's scan may need rebuilding for `path` (already
    /// canonicalized): a file it scanned, or one it read outside its scan
    /// (§AR-lsp.2.1). The directory fallback deliberately over-approximates
    /// newly created files; ownership below never uses that approximation
    /// (§FS-lsp.2.2.2).
    fn might_cover(&self, path: &Path) -> bool {
        path.starts_with(&self.root)
            || self.snapshot.scanned_files.contains(path)
            || self.read_outside_scan.contains(path)
            || path
                .parent()
                .is_some_and(|parent| self.scanned_dirs.contains(parent))
    }

    fn containing_root_depth(&self, path: &Path) -> Option<usize> {
        path.starts_with(&self.root)
            .then(|| self.root.components().count())
    }
}

/// A config that will not load, as the editor is told about it (§FS-lsp.2.2.4.1):
/// the file and line the load error names, and the error `grund check` prints.
/// Read off the core's own error, never out of its message text, so the editor
/// and the CLI locate it the same way (§FS-lsp.1.1).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ConfigFailure {
    file: PathBuf,
    line: Option<usize>,
    code: &'static str,
    message: String,
    /// The config's path as `message` spells it.
    report_path: Option<String>,
    /// The project left on its last good scan, if it has one (§FS-lsp.2.2.4).
    retained_root: Option<PathBuf>,
}

impl ConfigFailure {
    /// `None` for a failure that names no config file: it stays on stderr alone. One
    /// inside an imported tree goes on the import line that reached it, in the
    /// consumer's config (§FS-imports.editor).
    fn of(error: &anyhow::Error, retained_root: Option<PathBuf>) -> Option<Self> {
        let diagnostic = error.downcast_ref::<grund_core::OperationDiagnostic>()?;
        let (file, line, report_path) = match &diagnostic.import_site {
            Some((file, line)) => (file.clone(), Some(*line), None),
            None => (
                diagnostic.file.clone()?,
                diagnostic.line,
                diagnostic.path.clone(),
            ),
        };
        Some(Self {
            file,
            line,
            code: diagnostic.code,
            message: format!("{error:#}"),
            report_path,
            retained_root,
        })
    }

    /// Whether `message` spells the config from its own project, `grund.toml`
    /// rather than the `m/grund.toml` a workspace root's scan reads it as: the
    /// one `grund check` prints when run in that project.
    fn spelled_from_own_project(&self) -> bool {
        let Some(dir) = self.file.parent() else {
            return false;
        };
        let own = if dir.file_name().is_some_and(|name| name == ".agents") {
            dir.parent().unwrap_or(dir)
        } else {
            dir
        };
        self.report_path
            .as_deref()
            .is_some_and(|path| own.join(path) == self.file)
    }
}
