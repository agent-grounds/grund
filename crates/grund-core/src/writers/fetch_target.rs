//! Which file fetch writes and what it replaces there (§FS-fetch.4, §FS-fetch.5), read from the
//! one scan of the selected project that ran before the integration (§FS-fetch.6.1): which file
//! declares the ID, which the scan could not read, and whether a link at the target stays inside
//! the config root (§FS-fetch.links). No file is read here a second time to find a heading; the
//! scan's records are the answer. Every path a refusal names is spelled from the report base
//! (§FS-errors.terms).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::fetch::{FetchFailure, fetch_operational};
use super::fetch_folder::{already_declared, place_in_folder};
use super::fetch_splice::Owned;
use crate::config::Config;
use crate::model::{
    Catalog, Declaration, FileRecord, Id, OperationDiagnostic, format_path, paired_declarations,
    relative_from_base, scanned_path_key,
};
use crate::scanner::{ScanError, scan_error_message};
use crate::sources::{link_metadata, path_is_dir, path_is_file, read_bytes};
use crate::workspace::WorkspaceCitationTarget;

/// The kind's snapshot home: one file, its declarations at H2, or one folder, at H1.
pub(super) enum Home {
    File(PathBuf),
    Folder(PathBuf),
}

impl Home {
    /// The level a fetched declaration takes in this home (§FS-fetch.3).
    pub(super) fn level(&self) -> usize {
        match self {
            Home::File(_) => 2,
            Home::Folder(_) => 1,
        }
    }

    /// Whether the scan's `path` is in the home.
    pub(super) fn holds(&self, path: &Path) -> bool {
        let path = scanned_path_key(path);
        match self {
            Home::File(file) => path == scanned_path_key(file),
            Home::Folder(folder) => path.starts_with(scanned_path_key(folder)),
        }
    }
}

/// What one fetch asks: the ID, in the selected project, into its home.
pub(super) struct Request<'a> {
    pub(super) config: &'a Config,
    pub(super) id: &'a Id,
    pub(super) local: &'a str,
    pub(super) report_base: &'a Path,
    pub(super) home: Home,
}

impl Request<'_> {
    /// `path` as a refusal names it (§FS-errors.terms).
    pub(super) fn shown(&self, path: &Path) -> String {
        format_path(&relative_from_base(self.report_base, path))
    }
}

/// The tree as the scan read it before the integration ran (§FS-fetch.6.1).
pub(super) struct Before {
    pub(super) catalog: Catalog,
    pub(super) errors: Vec<ScanError>,
    /// The workspace's projects that scan parsed `§<alias>/<ID>` against, which every reading
    /// of the home after it parses against too (§FS-workspace.1.2).
    pub(super) targets: Vec<WorkspaceCitationTarget>,
}

impl Before {
    /// The scan's record of the file at `path`.
    pub(super) fn record(&self, path: &Path) -> Option<&FileRecord> {
        let key = scanned_path_key(path);
        self.catalog
            .files
            .iter()
            .find(|(file, _)| scanned_path_key(file) == key)
            .map(|(_, record)| record.as_ref())
    }

    /// Each file the scan read that its walk did not, a stub's target, with the text it read.
    pub(super) fn unwalked(&self) -> impl Iterator<Item = (PathBuf, String)> + '_ {
        let walked: BTreeSet<PathBuf> = self
            .catalog
            .scanned_files
            .iter()
            .map(|path| scanned_path_key(path))
            .collect();
        self.catalog
            .files
            .iter()
            .filter(move |(path, _)| !walked.contains(&scanned_path_key(path)))
            .map(|(path, record)| (path.clone(), record.text().to_string()))
    }

    /// The walk's own spelling of the file at `path`, where the walk read it.
    pub(super) fn walked(&self, path: &Path) -> Option<&Path> {
        let key = scanned_path_key(path);
        self.catalog
            .scanned_files
            .iter()
            .find(|file| scanned_path_key(file) == key)
            .map(PathBuf::as_path)
    }

    pub(super) fn error(&self, path: &Path) -> Option<&ScanError> {
        let key = scanned_path_key(path);
        self.errors
            .iter()
            .find(|error| scanned_path_key(&error.path) == key)
    }

    /// The declarations of `id` the scan read in `path`, stubs left out, in line order.
    pub(super) fn declaring<'b>(&'b self, id: &Id, path: &Path) -> Vec<&'b Declaration> {
        let mut found = self.declaring_or_stub(id, path);
        found.retain(|decl| !decl.is_stub);
        found
    }

    /// The declarations of `id` the scan read in `path`, stubs included, in line order.
    pub(super) fn declaring_or_stub<'b>(&'b self, id: &Id, path: &Path) -> Vec<&'b Declaration> {
        let key = scanned_path_key(path);
        let mut found: Vec<_> = self
            .catalog
            .declarations
            .get(id)
            .into_iter()
            .flatten()
            .filter(|decl| scanned_path_key(&decl.file) == key)
            .collect();
        found.sort_by_key(|decl| decl.line);
        found
    }
}

/// Where the output goes: the file and the range it replaces, `None` text for no file yet.
pub(super) struct Target {
    /// The path the scan reads the file by.
    pub(super) path: PathBuf,
    /// Where the bytes are installed: through a link that stays inside the root, its target.
    pub(super) install_at: PathBuf,
    /// The file's text before the integration ran, `None` where there was no file.
    pub(super) before: Option<String>,
    /// What the output takes the place of.
    pub(super) site: Site,
}

pub(super) enum Site {
    /// The declaration's own range (§FS-fetch.4).
    Replace(Owned),
    /// Nothing, at this byte, which starts this line (§FS-fetch.4).
    Insert(usize, usize),
    /// The whole file (§FS-fetch.5).
    Whole,
}

/// §FS-fetch.5.3, §FS-fetch.4: an ID the scan reads as declared in a file outside the home, a
/// `file` home or a `folder` one, is refused before the integration runs. Pairing supplies
/// targets even when unscanned, in path:line order, and a declaration reached through a stub is
/// named at its target. This refusal precedes every judgement of the home's contents.
pub(super) fn declared_elsewhere(
    request: &Request<'_>,
    before: &Before,
) -> Result<(), FetchFailure> {
    let elsewhere = before
        .catalog
        .declarations
        .get(request.id)
        .and_then(|decls| {
            paired_declarations(decls)
                .records()
                .find(|record| !request.home.holds(&record.file))
        });
    match elsewhere {
        Some(declaration) => Err(already_declared(
            request.local,
            &request.shown(&declaration.file),
        )),
        None => Ok(()),
    }
}

/// §FS-fetch.4, §FS-fetch.5: the target and its site, or the refusal that names why the home
/// cannot take the output.
pub(super) fn place(
    request: &Request<'_>,
    before: &Before,
    diagnostic: &mut Option<anyhow::Error>,
) -> Result<Target, FetchFailure> {
    match &request.home {
        Home::File(file) => place_in_file(request, before, file, diagnostic),
        Home::Folder(folder) => place_in_folder(request, before, folder, diagnostic),
    }
}

fn place_in_file(
    request: &Request<'_>,
    before: &Before,
    file: &Path,
    diagnostic: &mut Option<anyhow::Error>,
) -> Result<Target, FetchFailure> {
    // §FS-check.2.4.3: a file home is the row's own root; check's line names it.
    if let Some(error) = before.error(file).filter(|_| path_is_file(file)) {
        return Err(unscanned(request, error, diagnostic));
    }
    let install_at = link_target(request, before, file)?;
    let Some(record) = before.record(file) else {
        let text = text_of(file);
        let at = text.as_ref().map_or(0, String::len);
        let line = text.as_ref().map_or(1, |text| text.lines().count() + 1);
        return Ok(Target {
            path: file.to_path_buf(),
            install_at,
            before: text,
            site: Site::Insert(at, line),
        });
    };
    let declaring = before.declaring(request.id, file);
    let site = match declaring.as_slice() {
        [decl] => Site::Replace(Owned::of(record, decl.line, decl.body_end)),
        [] => {
            // §FS-fetch.4: ID order among the home's own declarations at the home's level.
            let grammar = &request.config.grammar;
            let requested = grammar.render(request.id, 3);
            let mut siblings: Vec<&Declaration> = before
                .catalog
                .declarations
                .values()
                .flatten()
                .filter(|decl| {
                    !decl.is_stub
                        && decl.heading_level == request.home.level()
                        && scanned_path_key(&decl.file) == scanned_path_key(file)
                })
                .collect();
            siblings.sort_by_key(|decl| decl.line);
            match siblings
                .iter()
                .find(|decl| grammar.render(&decl.id, 3) > requested)
            {
                Some(next) => Site::Insert(
                    record.line_start(next.line).unwrap_or(record.text().len()),
                    next.line,
                ),
                None => Site::Insert(record.text().len(), record.total_lines + 1),
            }
        }
        _ => {
            return Err(fetch_operational(
                "snapshot home has multiple declarations for requested ID",
            ));
        }
    };
    Ok(Target {
        path: file.to_path_buf(),
        install_at,
        before: Some(record.text().to_string()),
        site,
    })
}

/// §FS-fetch.links: where the bytes go when the target is a link, decided by the walk's own record
/// of it and nothing else. A link the walk reported as broken is refused; one it read as a file is
/// refused where it recorded the file as outside the config root (§FS-fmt.2.3.2), and otherwise
/// written through at the physical path the walk published for it (§AR-scanner.1.8); one it did
/// not read as a file, a link to a directory, has no record to decide by and is refused. A folder
/// home's `<ID>.md` link the walk ignores was refused before this (§FS-fetch.4.2.1). No link is
/// resolved here.
pub(super) fn link_target(
    request: &Request<'_>,
    before: &Before,
    target: &Path,
) -> Result<PathBuf, FetchFailure> {
    let is_link = link_metadata(target).is_ok_and(|meta| meta.file_type().is_symlink());
    if !is_link {
        return Ok(target.to_path_buf());
    }
    let refuse = |why: &str| {
        fetch_operational(format!(
            "cannot fetch {} into {}: {why}",
            request.local,
            request.shown(target)
        ))
    };
    if before.error(target).is_some_and(ScanError::is_broken_link) {
        return Err(refuse(
            "the symlink target does not exist; create the target, or remove the link, and \
             fetch again",
        ));
    }
    let spellings = &request.config.frame().run.spellings;
    let Some((walked, physical)) = before
        .walked(target)
        .and_then(|walked| Some((walked, spellings.physical(walked)?)))
    else {
        return Err(refuse(
            "the scan does not read the symlink as a file; remove the link, or let the scan \
             read it, and fetch again",
        ));
    };
    if before.catalog.outside_root.contains(walked) {
        return Err(refuse(
            "the symlink target is outside the config root; replace the link with the file it \
             points to, and fetch again",
        ));
    }
    Ok(physical)
}

/// §FS-fetch.5.4: a home file the scan could not read, as the line `grund check` prints for it.
pub(super) fn unscanned(
    request: &Request<'_>,
    error: &ScanError,
    diagnostic: &mut Option<anyhow::Error>,
) -> FetchFailure {
    let failure = fetch_operational(format!(
        "{}: {}",
        request.shown(&error.path),
        scan_error_message(error)
    ));
    with_io_source(request, &error.path, failure, diagnostic)
}

/// §FS-distribution.3.3.2: a refusal over a path the scan could not read keeps that read's
/// filesystem class and I/O source for an embedder. The scan keeps only the reason, so the path
/// is read again here for its source; a read that now succeeds leaves the refusal as it is.
pub(super) fn with_io_source(
    request: &Request<'_>,
    path: &Path,
    failure: FetchFailure,
    diagnostic: &mut Option<anyhow::Error>,
) -> FetchFailure {
    let read = if path_is_dir(path) {
        request.config.frame().sources().read_dir(path).map(drop)
    } else {
        read_bytes(path).map(drop)
    };
    if let Err(io) = read {
        let source = OperationDiagnostic::filesystem(path, &io, failure.message.clone());
        *diagnostic = Some(anyhow::Error::new(io).context(source));
    }
    failure
}

/// The text of a file the scan did not record, `None` where there is none to read.
pub(super) fn text_of(path: &Path) -> Option<String> {
    read_bytes(path)
        .ok()
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
}
