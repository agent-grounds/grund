// Per-file lookups over one project's snapshot (§FS-lsp.responsiveness.1).

/// Where a snapshot's records lie, grouped by the file they are in and, inside
/// it, by line, so answering about one document costs the records in that
/// document rather than every record in the workspace
/// (§FS-lsp.responsiveness.1).
///
/// Positions into the snapshot's own vectors rather than borrows of them: the
/// index is held beside the snapshot it describes and rebuilt with it, so a
/// borrow would only make the pair self-referential. Every list keeps the
/// snapshot's order, so a lookup answers with the record the linear scan it
/// replaces would have found first — the published range is unchanged
/// (§FS-lsp.4).
#[derive(Default)]
struct SnapshotIndex {
    citations: FileIndex,
    finding_ranges: FileIndex,
    declarations: FileIndex,
    sections: FileIndex,
    stubs: FileIndex,
}

impl SnapshotIndex {
    fn new(snapshot: &LspSnapshot) -> Self {
        Self {
            citations: FileIndex::build(&snapshot.citations, |record| {
                (record.path.as_path(), record.line)
            }),
            finding_ranges: FileIndex::build(&snapshot.finding_ranges, |record| {
                (record.path.as_path(), record.line)
            }),
            declarations: FileIndex::build(&snapshot.declarations, |record| {
                (record.path.as_path(), record.line)
            }),
            sections: FileIndex::build(&snapshot.sections, |record| {
                (record.path.as_path(), record.line)
            }),
            stubs: FileIndex::build(&snapshot.stubs, |record| {
                (record.path.as_path(), record.line)
            }),
        }
    }
}

/// One collection's record positions, keyed by the resolved path the records
/// already carry. A probe is matched by equality against a path the caller
/// resolved once, never by resolving either side again
/// (§FS-lsp.responsiveness.2).
#[derive(Default)]
struct FileIndex {
    files: BTreeMap<PathBuf, FileRecords>,
}

#[derive(Default)]
struct FileRecords {
    all: Vec<usize>,
    by_line: BTreeMap<usize, Vec<usize>>,
}

impl FileIndex {
    fn build<T>(records: &[T], key: impl Fn(&T) -> (&Path, usize)) -> Self {
        let mut files: BTreeMap<PathBuf, FileRecords> = BTreeMap::new();
        for (position, record) in records.iter().enumerate() {
            let (path, line) = key(record);
            let file = files.entry(path.to_path_buf()).or_default();
            file.all.push(position);
            file.by_line.entry(line).or_default().push(position);
        }
        Self { files }
    }

    /// Every record in `path`, in snapshot order. `path` is already resolved.
    fn in_file(&self, path: &Path) -> &[usize] {
        self.files.get(path).map_or(&[], |file| &file.all)
    }

    /// Every record on one line of `path`, in snapshot order. `path` is already
    /// resolved.
    fn on_line(&self, path: &Path, line: usize) -> &[usize] {
        self.files
            .get(path)
            .and_then(|file| file.by_line.get(&line))
            .map_or(&[], Vec::as_slice)
    }
}

/// The citations one file carries, in snapshot order. `path` is already
/// resolved, so this walks the file's own records rather than the workspace's
/// (§FS-lsp.responsiveness.1).
fn citations_in<'a>(
    snapshot: &'a LspSnapshot,
    index: &'a SnapshotIndex,
    path: &Path,
) -> impl Iterator<Item = &'a LspCitation> + use<'a> {
    index
        .citations
        .in_file(path)
        .iter()
        .map(|&position| &snapshot.citations[position])
}
