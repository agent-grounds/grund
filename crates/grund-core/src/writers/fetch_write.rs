//! Snapshot-home discovery and declaration ownership for the explicit
//! materializer (§FS-fetch.4, §FS-fetch.5); the shared writer in
//! `atomic_install.rs` installs what they decide (§REQ-no-data-loss.2).

use std::path::{Path, PathBuf};

use super::atomic_install::{Install, InstallFailure, atomic_install};
use super::fetch::{FetchFailure, fetch_io, fetch_operational};
use crate::config::Config;
use crate::grammar::{
    Grammar, MarkdownBlocks, markdown_line_records, near_miss_heading, parse_id_arg,
    parse_longest_id_prefix,
};
use crate::model::{Id, OperationDiagnostic, format_path};
use crate::scanner::{markdown_heading_level, walk_scannable_files_with_sources};
use crate::sources::{path_exists, path_is_dir, path_is_file, path_is_symlink, read_bytes};

/// `ignore::Error` owns the I/O error but does not expose it through `source()`.
/// Retain both its original context and that original I/O cause for embedders
/// (§FS-distribution.3.3.2), without changing the scanner's rendered reason.
#[derive(Debug)]
struct SnapshotWalkSource(ignore::Error);

impl std::fmt::Display for SnapshotWalkSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl std::error::Error for SnapshotWalkSource {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.io_error().map(|io| io as &dyn std::error::Error)
    }
}

/// Locate the fallible walk or an admitted traversal source while preserving the
/// existing fetch refusal text (§FS-distribution.3.3.2, §FS-fetch.7).
fn fetch_walk_failure(
    diagnostic: &mut Option<anyhow::Error>,
    path: &Path,
    error: anyhow::Error,
    message: String,
) -> FetchFailure {
    let source = match error
        .chain()
        .find_map(|source| source.downcast_ref::<std::io::Error>())
    {
        Some(io) => OperationDiagnostic::filesystem(path, io, message.clone()),
        None => {
            let mut source = OperationDiagnostic::new("operation", "fetch", message.clone());
            source.path = Some(format_path(path));
            source
        }
    };
    *diagnostic = Some(error.context(source));
    fetch_operational(message)
}

#[derive(Clone)]
struct SnapshotDeclaration {
    id: Id,
    start: usize,
    end: usize,
    path: Option<PathBuf>,
}

/// The refusal of a heading a fetch cannot take as a declaration, quoting its text after the
/// hashes. `heading` is the line with its indentation trimmed.
fn malformed_heading(heading: &str) -> String {
    format!(
        "snapshot home has malformed declaration heading `{}`",
        heading.trim_start_matches('#').trim()
    )
}

fn declarations_at_depth(
    bytes: &[u8],
    grammar: &Grammar,
    depth: usize,
) -> std::result::Result<Vec<SnapshotDeclaration>, FetchFailure> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| fetch_operational("snapshot home is not UTF-8 Markdown"))?;
    let mut headings = Vec::<(usize, Option<Id>)>::new();
    let mut offset = 0usize;
    // §FS-check.1.1.5.2: the whole snapshot home, so its front matter holds no heading.
    let mut blocks = MarkdownBlocks::file(text);
    for line in text.split_inclusive('\n') {
        let without_newline = line.strip_suffix('\n').unwrap_or(line);
        let content = without_newline
            .strip_suffix('\r')
            .unwrap_or(without_newline);
        let trimmed = content.trim_start();
        // §FS-check.1.1.5.1, §FS-check.1.1.5.2: neither a fence, a raw-text HTML block
        // nor front matter holds a heading.
        if !blocks.line(content).may_be_heading() {
            offset += line.len();
            continue;
        }
        let hashes = trimmed.bytes().take_while(|byte| *byte == b'#').count();
        if markdown_heading_level(trimmed) == Some(depth) {
            let start = offset + content.len() - trimmed.len();
            let tail = trimmed[hashes..].trim_start();
            let id = match tail.split_once(':') {
                Some((raw, title)) => match parse_id_arg(raw.trim(), grammar) {
                    Ok((id, None)) if !title.trim().is_empty() => Some(id),
                    Ok((_, None)) => {
                        return Err(fetch_operational(malformed_heading(trimmed)));
                    }
                    _ if near_miss_heading(grammar, trimmed, false, true).is_some() => {
                        return Err(fetch_operational(malformed_heading(trimmed)));
                    }
                    _ => None,
                },
                None if parse_longest_id_prefix(tail, grammar).is_some() => {
                    return Err(fetch_operational(malformed_heading(trimmed)));
                }
                None => None,
            };
            headings.push((start, id));
        }
        offset += line.len();
    }
    Ok(headings
        .iter()
        .enumerate()
        .filter_map(|(index, (start, id))| {
            let id = id.as_ref()?;
            // The blank separator before the next sibling is outside the
            // declaration-local replacement and therefore remains byte-for-byte.
            let end = headings.get(index + 1).map_or(bytes.len(), |next| {
                let end = next.0;
                if bytes[..end].ends_with(b"\r\n\r\n") {
                    end - 2
                } else if bytes[..end].ends_with(b"\n\n") {
                    end - 1
                } else {
                    end
                }
            });
            Some(SnapshotDeclaration {
                id: id.clone(),
                start: *start,
                end,
                path: None,
            })
        })
        .collect())
}

/// §FS-fetch.4: replace only the named H2 or insert it in ID order.
pub(super) fn write_file_home(
    path: &Path,
    grammar: &Grammar,
    requested: &Id,
    snapshot: &[u8],
    diagnostic: &mut Option<anyhow::Error>,
) -> std::result::Result<(), FetchFailure> {
    let original = match read_bytes(path) {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(err) => {
            return Err(fetch_io(
                diagnostic,
                path,
                err,
                format!("cannot read {}", path.display()),
            ));
        }
    };
    let declarations = declarations_at_depth(&original, grammar, 2)?;
    let matching = declarations
        .iter()
        .filter(|declaration| &declaration.id == requested)
        .collect::<Vec<_>>();
    if matching.len() > 1 {
        return Err(fetch_operational(
            "snapshot home has multiple declarations for requested ID",
        ));
    }
    let (start, end) = if let Some(existing) = matching.first() {
        (existing.start, existing.end)
    } else {
        let requested = grammar.render(requested, 3);
        let start = declarations
            .iter()
            .find(|declaration| grammar.render(&declaration.id, 3) > requested)
            .map_or(original.len(), |declaration| declaration.start);
        (start, start)
    };
    let mut replacement = Vec::with_capacity(original.len() + snapshot.len());
    let line_ending: &[u8] = if original.windows(2).any(|pair| pair == b"\r\n") {
        b"\r\n"
    } else {
        b"\n"
    };
    replacement.extend_from_slice(&original[..start]);
    if !replacement.is_empty()
        && !replacement.ends_with(b"\n\n")
        && !replacement.ends_with(b"\r\n\r\n")
    {
        replacement.extend_from_slice(line_ending);
    }
    replacement.extend_from_slice(snapshot);
    if end < original.len()
        && !original[end..].starts_with(b"\n")
        && !original[end..].starts_with(b"\r\n")
        && !replacement.ends_with(b"\n\n")
        && !replacement.ends_with(b"\r\n\r\n")
    {
        replacement.extend_from_slice(line_ending);
    }
    replacement.extend_from_slice(&original[end..]);
    install(path, &replacement, diagnostic)
}

/// §FS-fetch.5: replace the unique declaring file or create `<ID>.md`.
pub(super) fn write_folder_home(
    folder: &Path,
    config: &Config,
    requested: &Id,
    local: &str,
    snapshot: &[u8],
    diagnostic: &mut Option<anyhow::Error>,
) -> std::result::Result<(), FetchFailure> {
    let mut matches = Vec::new();
    if path_exists(folder) {
        if !path_is_dir(folder) {
            return Err(fetch_operational(format!(
                "snapshot folder {} is not a directory",
                folder.display()
            )));
        }
        // §FS-fetch.5.1: discovery uses the scanner's recursive folder traversal,
        // including its ignore, exclusion, hidden-file, and symlink semantics.
        let mut sources = std::collections::BTreeMap::new();
        let walked = walk_scannable_files_with_sources(
            config.schema(),
            config.frame(),
            Some(folder),
            true,
            &mut |report, error| {
                sources.entry(report.clone()).or_insert(error);
            },
        )
        .map_err(|err| {
            let message = format!("cannot read snapshot folder {}: {err:#}", folder.display());
            fetch_walk_failure(diagnostic, folder, err, message)
        })?;
        if let Some(report @ (path, message)) = walked.errors.first() {
            let message = format!(
                "cannot read snapshot folder {} at {}: {message}",
                folder.display(),
                path.display()
            );
            return Err(match sources.remove(report) {
                Some(source) => fetch_walk_failure(
                    diagnostic,
                    path,
                    anyhow::Error::new(SnapshotWalkSource(source)),
                    message,
                ),
                None => fetch_operational(message),
            });
        }
        for path in walked.files {
            if path.extension().and_then(|extension| extension.to_str()) != Some("md") {
                continue;
            }
            if let SnapshotFile::Declares(declaration) =
                read_snapshot_file(&path, config, requested, diagnostic)?
            {
                matches.push(declaration);
            }
        }
    }
    if matches.len() > 1 {
        return Err(fetch_operational(
            "snapshot folder has multiple declarations for requested ID",
        ));
    }
    let target = matches
        .first()
        .and_then(|declaration| declaration.path.clone())
        .unwrap_or_else(|| folder.join(format!("{local}.md")));
    // §FS-fetch.5.2: with no declaring file found, what stands at `<home>/<ID>.md` is
    // judged by what it holds, whether or not the walk admitted it.
    if matches.is_empty() && path_exists(&target) {
        if path_is_symlink(&target) || !path_is_file(&target) {
            return Err(fetch_operational(format!(
                "snapshot file {} is not a regular file",
                target.display()
            )));
        }
        let bytes = read_snapshot_bytes(&target, diagnostic)?;
        let held = snapshot_file_holds(&bytes, &target, config, requested)?;
        if !matches!(held, SnapshotFile::Declares(_)) {
            let another_h1 = matches!(held, SnapshotFile::Other);
            return Err(fetch_operational(undeclared_target_refusal(
                &target, &bytes, config, requested, local, another_h1,
            )));
        }
    }
    install(&target, snapshot, diagnostic)
}

/// §FS-fetch.5.2: what `<home>/<ID>.md` is told when `fetch` takes no H1 of it as declaring the
/// requested ID, by what a scan records there at any depth, the whole file read before a nested
/// record is believed. This reads for the words alone: the file is refused whatever it finds,
/// and the walk's search for a declaring file does not ask it.
fn undeclared_target_refusal(
    path: &Path,
    bytes: &[u8],
    config: &Config,
    requested: &Id,
    local: &str,
    another_h1: bool,
) -> String {
    let path_shown = path.display();
    let text = String::from_utf8_lossy(bytes);
    let mut blocks = MarkdownBlocks::file(&text);
    let mut another = another_h1;
    let mut nested = false;
    for line in text.lines() {
        if !blocks.line(line).may_be_heading() {
            continue;
        }
        let Some(token) = markdown_line_records(&config.grammar, line) else {
            continue;
        };
        if !matches!(parse_id_arg(token, &config.grammar), Ok((id, None)) if id == *requested) {
            another = true;
            continue;
        }
        let trimmed = line.trim_start();
        if markdown_heading_level(trimmed) == Some(1) {
            return malformed_heading(trimmed);
        }
        nested = true;
    }
    if nested {
        format!("snapshot file {path_shown} declares {local} only in a nested heading")
    } else if another {
        format!("snapshot file {path_shown} already exists for a different ID")
    } else {
        format!("snapshot file {path_shown} already exists and does not declare {local}")
    }
}

/// What one Markdown file in a folder home holds at the home's native depth, an H1, for the
/// requested ID.
enum SnapshotFile {
    /// An H1 declares the requested ID alone, so it is the declaring file.
    Declares(SnapshotDeclaration),
    /// H1s declare other IDs and not the requested one.
    Other,
    /// No H1 declares anything.
    Nothing,
}

/// §FS-fetch.5 / §FS-fetch.5.2: read one folder-home file the way the walk's files are
/// read, refusing one that holds the requested ID beside other content. The walk and the
/// read of `<home>/<ID>.md` both ask this, so they cannot disagree on what a file holds.
fn read_snapshot_file(
    path: &Path,
    config: &Config,
    requested: &Id,
    diagnostic: &mut Option<anyhow::Error>,
) -> std::result::Result<SnapshotFile, FetchFailure> {
    let bytes = read_snapshot_bytes(path, diagnostic)?;
    snapshot_file_holds(&bytes, path, config, requested)
}

fn read_snapshot_bytes(
    path: &Path,
    diagnostic: &mut Option<anyhow::Error>,
) -> std::result::Result<Vec<u8>, FetchFailure> {
    read_bytes(path).map_err(|err| {
        fetch_io(
            diagnostic,
            path,
            err,
            format!("cannot read {}", path.display()),
        )
    })
}

fn snapshot_file_holds(
    bytes: &[u8],
    path: &Path,
    config: &Config,
    requested: &Id,
) -> std::result::Result<SnapshotFile, FetchFailure> {
    let declarations = declarations_at_depth(bytes, &config.grammar, 1)?;
    let Some(declaration) = declarations
        .iter()
        .find(|declaration| declaration.id == *requested)
    else {
        return Ok(if declarations.is_empty() {
            SnapshotFile::Nothing
        } else {
            SnapshotFile::Other
        });
    };
    let has_content_before = bytes[..declaration.start]
        .iter()
        .any(|byte| !byte.is_ascii_whitespace());
    let has_content_after = bytes[declaration.end..]
        .iter()
        .any(|byte| !byte.is_ascii_whitespace());
    if declarations.len() != 1 || has_content_before || has_content_after {
        return Err(fetch_operational(format!(
            "snapshot file {} contains the requested ID beside other content",
            path.display()
        )));
    }
    let mut declaration = declarations.into_iter().next().expect("one declaration");
    declaration.path = Some(path.to_path_buf());
    Ok(SnapshotFile::Declares(declaration))
}

/// §FS-fetch.4 / §FS-fetch.5: install complete bytes through the shared writer,
/// each of its failures as fetch's refusal.
fn install(
    path: &Path,
    bytes: &[u8],
    diagnostic: &mut Option<anyhow::Error>,
) -> std::result::Result<(), FetchFailure> {
    let install = Install {
        bytes,
        tag: "fetch",
        expected: None,
    };
    atomic_install(path, install).map_err(|failure| match failure {
        InstallFailure::Io { error, context } => fetch_io(diagnostic, path, error, context),
        InstallFailure::Operational(message) => fetch_operational(message),
        InstallFailure::Changed => fetch_operational(format!("{} changed", path.display())),
    })
}
