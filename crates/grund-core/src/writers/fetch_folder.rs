//! A `folder` home's target (§FS-fetch.5): which file declares the ID, read from the one scan
//! that ran before the integration (§FS-fetch.6.1), or the `<ID>.md` the output creates or fills
//! (§FS-fetch.5.5), and the refusals that say why the folder cannot take it. Every path is
//! spelled from the report base (§FS-errors.terms).

use std::path::{Path, PathBuf};

use super::fetch::{FetchFailure, fetch_io, fetch_operational};
use super::fetch_splice::Owned;
use super::fetch_target::{
    Before, Request, Site, Target, link_target, text_of, unscanned, with_io_source,
};
use crate::model::{scanned_path_key, sort_path_key};
use crate::scanner::{ScanError, folder_value_declarations};
use crate::sources::{path_exists, path_is_dir, path_is_file, read_bytes};

/// §FS-fetch.5: replace the one file that declares the ID, or create or fill `<ID>.md`, after
/// `declared_elsewhere` refused a declaration outside the home (§FS-fetch.5.3), refusing a value source that declares it
/// (§FS-fetch.5.1), a file it would not replace that declares it (§FS-fetch.5.2), and a file
/// that holds it beside other content (§FS-fetch.5.5).
pub(super) fn place_in_folder(
    request: &Request<'_>,
    before: &Before,
    folder: &Path,
    diagnostic: &mut Option<anyhow::Error>,
) -> Result<Target, FetchFailure> {
    let local = request.local;
    let snapshot = folder.join(format!("{local}.md"));
    if path_exists(folder) {
        if !path_is_dir(folder) {
            return Err(fetch_operational(format!(
                "snapshot folder {} is not a directory",
                request.shown(folder)
            )));
        }
        refuse_value_source(request, folder)?;
    }
    // §FS-fetch.5: what the walk could not enter in the home, but a link dangling at the target,
    // which §FS-fetch.links names once the target is known.
    if let Some(error) = before.errors.iter().find(|error| {
        request.home.holds(&error.path)
            && !path_is_file(&error.path)
            && scanned_path_key(&error.path) != scanned_path_key(&snapshot)
    }) {
        let failure = unread_folder(request, folder, error);
        return Err(with_io_source(request, &error.path, failure, diagnostic));
    }
    let mut files: Vec<&Path> = before
        .catalog
        .files
        .keys()
        .map(PathBuf::as_path)
        .chain(before.errors.iter().map(|error| error.path.as_path()))
        .filter(|path| request.home.holds(path) && path_is_file(path))
        .collect();
    files.sort_by_key(|path| sort_path_key(path));
    files.dedup();
    refuse_declaring_file(request, before, &files, diagnostic)?;
    let mut matches = Vec::new();
    for path in files.iter().filter(|path| is_replaceable_markdown(path)) {
        if let Some(error) = before.error(path) {
            return Err(match read_bytes(path) {
                Err(err) => {
                    let context = format!("cannot read {}", request.shown(path));
                    fetch_io(diagnostic, path, err, context)
                }
                Ok(_) => unscanned(request, error, diagnostic),
            });
        }
        // §FS-fetch.5.5: a broken stub in the home is the home's own declaration; a healthy one
        // was refused above, at its target.
        let declaring = before.declaring_or_stub(request.id, path);
        let (Some(first), Some(record)) = (declaring.first(), before.record(path)) else {
            continue;
        };
        // §FS-fetch.5.5: the file is the declaration and nothing else.
        let owned = Owned::of(record, first.line, first.body_end);
        let text = record.text();
        if !text[..owned.start].trim().is_empty() || !text[owned.end..].trim().is_empty() {
            return Err(fetch_operational(format!(
                "snapshot file {} contains the requested ID beside other content",
                request.shown(path)
            )));
        }
        matches.push((path.to_path_buf(), record.text().to_string()));
    }
    if matches.len() > 1 {
        return Err(fetch_operational(
            "snapshot folder has multiple declarations for requested ID",
        ));
    }
    let (path, text) = match matches.pop() {
        Some((path, text)) => (path, Some(text)),
        None => (
            snapshot.clone(),
            unclaimed_snapshot(request, before, &snapshot)?,
        ),
    };
    let install_at = link_target(request, before, &path)?;
    Ok(Target {
        path,
        install_at,
        before: text,
        site: Site::Whole,
    })
}

/// §FS-fetch.5.5: what `<ID>.md` may hold when no file declares the ID: nothing, or only
/// whitespace, which the output fills.
fn unclaimed_snapshot(
    request: &Request<'_>,
    before: &Before,
    snapshot: &Path,
) -> Result<Option<String>, FetchFailure> {
    let Some(text) = text_of(snapshot) else {
        return Ok(None);
    };
    let declares = before
        .catalog
        .declarations
        .values()
        .flatten()
        .any(|decl| scanned_path_key(&decl.file) == scanned_path_key(snapshot));
    if declares {
        return Err(fetch_operational(format!(
            "snapshot file {} already exists for a different ID",
            request.shown(snapshot)
        )));
    }
    if !text.trim().is_empty() {
        return Err(fetch_operational(format!(
            "snapshot file {} already exists and the scan reads no ID in it; move or rename it, \
             and fetch again",
            request.shown(snapshot)
        )));
    }
    Ok(Some(text))
}

/// §FS-fetch.5: fetch's own refusal of a folder the scan could not read at `error`'s path. The
/// reason it quotes is the scan failure's, every path in it spelled as the refusal spells its
/// own (§FS-errors.2.2.5, §AR-scanner.1.failure), so nothing the record carries beside its
/// reason is dropped.
fn unread_folder(request: &Request<'_>, folder: &Path, error: &ScanError) -> FetchFailure {
    fetch_operational(format!(
        "cannot read snapshot folder {} at {}: {}",
        request.shown(folder),
        request.shown(&error.path),
        error.spelled_reason(|path| request.shown(path))
    ))
}

/// §FS-fetch.5.1: value inputs bypass the general walk's scan filters, so their own reader is
/// asked before any Markdown match can win.
fn refuse_value_source(request: &Request<'_>, folder: &Path) -> Result<(), FetchFailure> {
    let config = request.config;
    let values =
        folder_value_declarations(config.schema(), config.frame(), folder, &request.id.kind)
            .map_err(|error| unread_folder(request, folder, &error))?;
    match values
        .declarations
        .get(request.id)
        .and_then(|sites| sites.first())
    {
        Some(declaration) => Err(fetch_operational(format!(
            "cannot fetch {}: value source {} already declares it; keep the fact in the value \
             source or move it outside the value home before fetching",
            request.local,
            request.shown(&declaration.file)
        ))),
        None => Ok(()),
    }
}

/// §FS-fetch.5.2: in the walk's sorted order, the first file the Markdown branch would not
/// replace that declares the ID, or that the scan could not read, decides. A file the scan could
/// not read gets the line `grund check` prints for it (§FS-check.2.4), unless an undecodable
/// `.md` file comes earlier: §FS-fetch.5.4 names the first, so a declaring file still comes
/// ahead of every judgement of a Markdown file.
fn refuse_declaring_file(
    request: &Request<'_>,
    before: &Before,
    files: &[&Path],
    diagnostic: &mut Option<anyhow::Error>,
) -> Result<(), FetchFailure> {
    let declares = |path: &Path| {
        before
            .catalog
            .declarations
            .get(request.id)
            .into_iter()
            .flatten()
            .any(|decl| scanned_path_key(&decl.file) == scanned_path_key(path))
    };
    for (position, path) in files.iter().enumerate() {
        if is_replaceable_markdown(path) {
            continue;
        }
        if declares(path) {
            return Err(already_declared(request.local, &request.shown(path)));
        }
        if let Some(error) = before.error(path) {
            let earlier = files[..position].iter().find_map(|earlier| {
                (is_replaceable_markdown(earlier) && read_bytes(earlier).is_ok())
                    .then(|| before.error(earlier))
                    .flatten()
            });
            return Err(unscanned(request, earlier.unwrap_or(error), diagnostic));
        }
    }
    Ok(())
}

/// §FS-fetch.5.2, §FS-fetch.5.3: the one refusal of an ID another file already declares.
pub(super) fn already_declared(local: &str, shown: &str) -> FetchFailure {
    fetch_operational(format!(
        "cannot fetch {local}: {shown} already declares it; keep the fact in that file, or take \
         the declaration out of it and fetch again"
    ))
}

/// §FS-fetch.5: what the Markdown branch replaces is a file named `.md`.
fn is_replaceable_markdown(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()) == Some("md")
}
