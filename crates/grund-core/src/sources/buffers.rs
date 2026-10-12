//! The run's buffers and the one view every reader of a file's text asks
//! (§AR-system.2.13): a file's text is what a save would write, the open buffer
//! where there is one and the disk where there is none (§FS-scan.text.buffer).
//! A buffer is the file at its physical location, under whatever spelling the
//! editor opened it and whatever spelling a reader reaches it by
//! (§FS-scan.identity.buffers), and a buffer with no file behind it is a file
//! that exists (§FS-scan.text.buffer-only).

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::disk::input_read_to_string;
use super::identity::canonicalize_existing_prefix;
use super::observe::observe_input;
use crate::model::normalize_path_lexically;

/// The editor's open documents for one run, keyed by physical path
/// (§AR-config.1.5, §FS-scan.identity.buffers). A command-line run has none.
#[derive(Clone, Default)]
pub(crate) struct Buffers(Arc<BTreeMap<PathBuf, String>>);

impl Buffers {
    /// The documents an editor handed its run, each keyed by the location its
    /// path resolves to, so two spellings of one file are one buffer.
    pub(crate) fn new(open: impl IntoIterator<Item = (PathBuf, String)>) -> Self {
        Self(Arc::new(
            open.into_iter()
                .map(|(path, text)| (canonicalize_existing_prefix(&path), text))
                .collect(),
        ))
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The buffer that is the file at `path`, under any spelling of it.
    fn get(&self, path: &Path) -> Option<&str> {
        if self.0.is_empty() {
            return None;
        }
        self.0
            .get(&canonicalize_existing_prefix(path))
            .map(String::as_str)
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

    /// The open buffer that is the file at `path`, if the run holds one.
    pub(crate) fn buffer(self, path: &Path) -> Option<&'a str> {
        self.buffers.and_then(|buffers| buffers.get(path))
    }

    /// The file's text as a save would write it (§FS-scan.text.buffer).
    pub(crate) fn text(self, path: &Path) -> std::io::Result<Cow<'a, str>> {
        match self.buffer(path) {
            Some(text) => Ok(Cow::Borrowed(text)),
            None => input_read_to_string(path).map(Cow::Owned),
        }
    }

    /// Whether the file is there: on disk, or open in a buffer with nothing
    /// behind it yet (§FS-scan.text.buffer-only).
    pub(crate) fn exists(self, path: &Path) -> bool {
        self.buffer(path).is_some() || path.exists()
    }

    /// Whether `path` is a file, a buffer being one (§FS-scan.text.buffer-only).
    pub(crate) fn is_file(self, path: &Path) -> bool {
        self.buffer(path).is_some() || path.is_file()
    }

    /// The buffers with no file behind them, each under the path the editor
    /// named it by: the files only the run has, which the walk never meets on
    /// disk and asks the scan predicate about instead (§AR-scanner.1.13).
    pub(crate) fn buffer_only_files(self) -> Vec<PathBuf> {
        self.buffers
            .map(|buffers| {
                buffers
                    .0
                    .keys()
                    .filter(|path| !path.exists())
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
