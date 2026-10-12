//! The run's buffers and the one view every reader of a file's text asks
//! (§AR-system.2.13): a file's text is what a save would write, the open buffer
//! where there is one and the disk where there is none (§FS-scan.text.buffer).
//! A buffer is the file at its physical location, under whatever spelling the
//! editor opened it and whatever spelling a reader reaches it by
//! (§FS-scan.identity.buffers), and a buffer with no file behind it is a file
//! that exists (§FS-scan.text.buffer-only).

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use super::disk::{input_read_dir, input_read_to_string, path_exists, path_is_file};
use super::identity::canonicalize_existing_prefix;
use super::observe::observe_input;
use crate::model::normalize_path_lexically;

/// The editor's open documents for one run, keyed by physical path
/// (§AR-config.1.5, §FS-scan.identity.buffers). A command-line run has none.
#[derive(Clone, Default)]
pub(crate) struct Buffers {
    open: Arc<BTreeMap<PathBuf, String>>,
    /// What this run has read, kept when the run is the editor's snapshot.
    /// Shared by every member's run.
    record: Option<Arc<Mutex<Record>>>,
}

/// What a recording run has read (§FS-lsp.responsiveness.3).
#[derive(Default)]
struct Record {
    /// The disk text it read, so a later answer over it reads nothing again
    /// (§AR-lsp.5.2). Never a buffer's text: a buffer is in front of it.
    disk: BTreeMap<PathBuf, String>,
    /// Every path it read, whether a buffer or the disk answered: what its
    /// snapshot goes stale on (§AR-lsp.2.1).
    read: BTreeSet<PathBuf>,
}

impl Buffers {
    /// The documents an editor handed its run, each keyed by the location its
    /// path resolves to, so two spellings of one file are one buffer.
    pub(crate) fn new(open: impl IntoIterator<Item = (PathBuf, String)>) -> Self {
        Self {
            open: Arc::new(
                open.into_iter()
                    .map(|(path, text)| (canonicalize_existing_prefix(&path), text))
                    .collect(),
            ),
            record: None,
        }
    }

    /// These buffers, keeping every path the run reads and every file text it
    /// reads from disk, under the path it was read by: the snapshot a hover
    /// answers from without reading the file again (§FS-lsp.responsiveness.3),
    /// and what that snapshot goes stale on (§AR-lsp.2.1).
    pub(crate) fn recording(self) -> Self {
        Self {
            record: Some(Arc::default()),
            ..self
        }
    }

    /// The buffer that is the file at `path`, under any spelling of it.
    fn get(&self, path: &Path) -> Option<&str> {
        if self.open.is_empty() {
            return None;
        }
        self.open
            .get(&canonicalize_existing_prefix(path))
            .map(String::as_str)
    }

    /// The disk text a recording run already read at `path`, as it read it.
    fn recorded(&self, path: &Path) -> Option<String> {
        let record = self.record.as_ref()?;
        record.lock().ok()?.disk.get(path).cloned()
    }

    /// Every path a recording run has read so far, from a buffer or from disk,
    /// under the path it was read by: what its snapshot goes stale on
    /// (§AR-lsp.2.1).
    pub(crate) fn recorded_paths(&self) -> Vec<PathBuf> {
        self.record
            .as_ref()
            .and_then(|record| {
                record
                    .lock()
                    .ok()
                    .map(|record| record.read.iter().cloned().collect())
            })
            .unwrap_or_default()
    }

    /// Note that the run read `path`, if it records its reads, whichever of a
    /// buffer and the disk answers it (§AR-lsp.2.1).
    fn note_read(&self, path: &Path) {
        if let Some(record) = &self.record
            && let Ok(mut record) = record.lock()
            && !record.read.contains(path)
        {
            record.read.insert(path.to_path_buf());
        }
    }

    /// Keep `text`, read from disk at `path`, if this run records its reads.
    fn record(&self, path: &Path, text: &str) {
        if let Some(record) = &self.record
            && let Ok(mut record) = record.lock()
        {
            record.disk.insert(path.to_path_buf(), text.to_string());
        }
    }
}

/// What a reader asks for a file's text and existence (§AR-system.2.13): the
/// run's buffers in front of the disk, or the `saved` view, the disk alone.
#[derive(Clone, Copy)]
pub(crate) struct Sources<'a> {
    buffers: Option<&'a Buffers>,
}

impl<'a> Sources<'a> {
    /// The view over `buffers`: what a save would write (§FS-scan.text.buffer).
    pub(crate) fn of(buffers: &'a Buffers) -> Self {
        Self {
            buffers: Some(buffers),
        }
    }

    /// The disk alone, for a reader no run is handed to: a configuration read
    /// before any run is built, or a writer reading what it is about to replace.
    pub(crate) fn on_disk() -> Sources<'static> {
        Sources { buffers: None }
    }

    /// The disk past the buffers, for the readers §REQ-one-answer.differences.text
    /// still lists until each row closes (§DF-four-editor-checks-read-saved-text).
    /// It is the one name those readers use, so finding `Sources::saved()` finds
    /// every one of them.
    pub(crate) fn saved() -> Sources<'static> {
        Sources { buffers: None }
    }

    /// The open buffer that is the file at `path`, if the run holds one.
    pub(crate) fn buffer(self, path: &Path) -> Option<&'a str> {
        self.buffers.and_then(|buffers| buffers.get(path))
    }

    /// The file's text as a save would write it (§FS-scan.text.buffer). A
    /// recording run notes every path it reads, a buffer's included, and reads
    /// each file from disk once, answering again from what it kept
    /// (§FS-lsp.responsiveness.3, §AR-lsp.2.1).
    pub(crate) fn text(self, path: &Path) -> std::io::Result<Cow<'a, str>> {
        let Some(buffers) = self.buffers else {
            return input_read_to_string(path).map(Cow::Owned);
        };
        buffers.note_read(path);
        if let Some(text) = buffers.get(path) {
            return Ok(Cow::Borrowed(text));
        }
        if let Some(text) = buffers.recorded(path) {
            return Ok(Cow::Owned(text));
        }
        let text = input_read_to_string(path)?;
        buffers.record(path, &text);
        Ok(Cow::Owned(text))
    }

    /// The directory's entries on disk, observed first (§FS-check.6.1.3); the
    /// buffers with no file behind them are `buffer_only_files`.
    pub(crate) fn read_dir(self, path: impl AsRef<Path>) -> std::io::Result<std::fs::ReadDir> {
        input_read_dir(path)
    }

    /// Whether the file is there: on disk, or open in a buffer with nothing
    /// behind it yet (§FS-scan.text.buffer-only).
    pub(crate) fn exists(self, path: &Path) -> bool {
        self.buffer(path).is_some() || path_exists(path)
    }

    /// Whether `path` is a file, a buffer being one (§FS-scan.text.buffer-only).
    pub(crate) fn is_file(self, path: &Path) -> bool {
        self.buffer(path).is_some() || path_is_file(path)
    }

    /// The buffers with no file behind them, each under the path the editor
    /// named it by: the files only the run has, which the walk never meets on
    /// disk and asks the scan predicate about instead (§AR-scanner.1.13).
    pub(crate) fn buffer_only_files(self) -> Vec<PathBuf> {
        self.buffers
            .map(|buffers| {
                buffers
                    .open
                    .keys()
                    .filter(|path| !path_exists(path))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The file a stub's link names (§FS-check.6.1.1): relative to the stub's
    /// own directory where that file is there, and to the config root where it
    /// is not, a buffer counting as there (§FS-scan.text.buffer-only). Both
    /// candidates are observed before one is chosen, missing directories and
    /// followed links included (§FS-check.6.1.3).
    pub(crate) fn stub_target(self, root: &Path, stub_file: &Path, target: &Path) -> PathBuf {
        if target.is_absolute() {
            observe_input(target, false);
            return target.to_path_buf();
        }
        let stub_file = if stub_file.is_absolute() {
            stub_file.to_path_buf()
        } else {
            root.join(stub_file)
        };
        let markdown_relative =
            normalize_path_lexically(&stub_file.parent().unwrap_or(root).join(target));
        let root_relative = normalize_path_lexically(&root.join(target));
        observe_input(&markdown_relative, false);
        observe_input(&root_relative, false);
        if self.exists(&markdown_relative) {
            markdown_relative
        } else {
            root_relative
        }
    }
}
