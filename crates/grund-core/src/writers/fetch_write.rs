//! Snapshot-home discovery, declaration ownership, and atomic installation for
//! the explicit materializer (§FS-fetch.4, §FS-fetch.5, §REQ-no-data-loss.2).

use std::fs;
use std::path::{Path, PathBuf};

use super::fetch::{FetchFailure, fetch_io, fetch_operational};
use crate::config::Config;
use crate::grammar::{
    Grammar, MarkdownBlocks, near_miss_heading, parse_id_arg, parse_longest_id_prefix,
};
use crate::model::{Id, OperationDiagnostic, format_path};
use crate::scanner::{markdown_heading_level, walk_scannable_files_with_sources};

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

fn declarations_at_depth(
    bytes: &[u8],
    grammar: &Grammar,
    depth: usize,
) -> std::result::Result<Vec<SnapshotDeclaration>, FetchFailure> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| fetch_operational("snapshot home is not UTF-8 Markdown"))?;
    let mut headings = Vec::<(usize, Option<Id>)>::new();
    let mut offset = 0usize;
    let mut blocks = MarkdownBlocks::default();
    for line in text.split_inclusive('\n') {
        let without_newline = line.strip_suffix('\n').unwrap_or(line);
        let content = without_newline
            .strip_suffix('\r')
            .unwrap_or(without_newline);
        let trimmed = content.trim_start();
        // §FS-check.1.1.5.1: neither a fence nor a raw-text HTML block holds a heading.
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
                        return Err(fetch_operational(format!(
                            "snapshot home has malformed declaration heading `{}`",
                            tail.trim()
                        )));
                    }
                    _ if near_miss_heading(grammar, trimmed, false, true).is_some() => {
                        return Err(fetch_operational(format!(
                            "snapshot home has malformed declaration heading `{}`",
                            tail.trim()
                        )));
                    }
                    _ => None,
                },
                None if parse_longest_id_prefix(tail, grammar).is_some() => {
                    return Err(fetch_operational(format!(
                        "snapshot home has malformed declaration heading `{}`",
                        tail.trim()
                    )));
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
    let original = match fs::read(path) {
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
    atomic_install(path, &replacement, diagnostic)
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
    if folder.exists() {
        if !folder.is_dir() {
            return Err(fetch_operational(format!(
                "snapshot folder {} is not a directory",
                folder.display()
            )));
        }
        // §FS-fetch.5: discovery uses the scanner's recursive folder traversal,
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
            let bytes = fs::read(&path).map_err(|err| {
                fetch_io(
                    diagnostic,
                    &path,
                    err,
                    format!("cannot read {}", path.display()),
                )
            })?;
            let declarations = declarations_at_depth(&bytes, &config.grammar, 1)?;
            let contains_requested = declarations
                .iter()
                .any(|declaration| declaration.id == *requested);
            if contains_requested {
                let declaration = declarations
                    .iter()
                    .find(|declaration| declaration.id == *requested)
                    .expect("contains requested declaration");
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
            }
            for mut declaration in declarations {
                if declaration.id == *requested {
                    declaration.path = Some(path.clone());
                    matches.push(declaration);
                }
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
    if matches.is_empty() && target.exists() {
        return Err(fetch_operational(format!(
            "snapshot file {} already exists for a different ID",
            target.display()
        )));
    }
    atomic_install(&target, snapshot, diagnostic)
}

/// §FS-fetch.4 / §FS-fetch.5: install complete bytes by same-directory rename,
/// leaving an unchanged target untouched.
fn atomic_install(
    path: &Path,
    bytes: &[u8],
    diagnostic: &mut Option<anyhow::Error>,
) -> std::result::Result<(), FetchFailure> {
    if fs::read(path).ok().as_deref() == Some(bytes) {
        return Ok(());
    }
    let existing_permissions = match fs::metadata(path) {
        Ok(metadata) => Some(metadata.permissions()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => None,
        Err(err) => {
            return Err(fetch_io(
                diagnostic,
                path,
                err,
                format!("cannot read metadata for {}", path.display()),
            ));
        }
    };
    let parent = path
        .parent()
        .ok_or_else(|| fetch_operational(format!("cannot write {}", path.display())))?;
    let created_directories = create_parent_directories(parent, path, diagnostic)?;
    let mut temporary = None;
    for attempt in 0..100u32 {
        let candidate = parent.join(format!(".grund-fetch-{}-{attempt}.tmp", std::process::id()));
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(file) => {
                temporary = Some((candidate, file));
                break;
            }
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(err) => {
                rollback_created_directories(&created_directories);
                return Err(fetch_io(
                    diagnostic,
                    path,
                    err,
                    format!("cannot write {}", path.display()),
                ));
            }
        }
    }
    let Some((temporary_path, mut file)) = temporary else {
        rollback_created_directories(&created_directories);
        return Err(fetch_operational(format!(
            "cannot create temporary file for {}",
            path.display()
        )));
    };
    use std::io::Write;
    let installed = file.write_all(bytes).and_then(|_| {
        if let Some(permissions) = existing_permissions {
            file.set_permissions(permissions)?;
        }
        file.sync_all()
    });
    if let Err(err) = installed {
        drop(file);
        let _ = fs::remove_file(&temporary_path);
        rollback_created_directories(&created_directories);
        return Err(fetch_io(
            diagnostic,
            path,
            err,
            format!("cannot write {}", path.display()),
        ));
    }
    drop(file);
    if let Err(err) = fs::rename(&temporary_path, path) {
        let _ = fs::remove_file(&temporary_path);
        rollback_created_directories(&created_directories);
        return Err(fetch_io(
            diagnostic,
            path,
            err,
            format!("cannot atomically replace {}", path.display()),
        ));
    }
    Ok(())
}

/// Create only the missing parent chain and remember exactly what this fetch
/// added, so every later failure can restore the prior tree (§FS-fetch.4,
/// §FS-fetch.5, §REQ-no-data-loss.2).
fn create_parent_directories(
    parent: &Path,
    target: &Path,
    diagnostic: &mut Option<anyhow::Error>,
) -> std::result::Result<Vec<PathBuf>, FetchFailure> {
    let mut missing = Vec::new();
    let mut cursor = parent;
    loop {
        match fs::metadata(cursor) {
            Ok(metadata) if metadata.is_dir() => break,
            Ok(_) => {
                return Err(fetch_operational(format!(
                    "cannot write {}: {} is not a directory",
                    target.display(),
                    cursor.display()
                )));
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                missing.push(cursor.to_path_buf());
                cursor = cursor.parent().ok_or_else(|| {
                    fetch_io(
                        diagnostic,
                        target,
                        err,
                        format!("cannot write {}", target.display()),
                    )
                })?;
            }
            Err(err) => {
                return Err(fetch_io(
                    diagnostic,
                    target,
                    err,
                    format!("cannot write {}", target.display()),
                ));
            }
        }
    }

    let mut created = Vec::new();
    for directory in missing.into_iter().rev() {
        if let Err(err) = fs::create_dir(&directory) {
            rollback_created_directories(&created);
            return Err(fetch_io(
                diagnostic,
                target,
                err,
                format!("cannot write {}", target.display()),
            ));
        }
        created.push(directory);
    }
    Ok(created)
}

fn rollback_created_directories(created: &[PathBuf]) {
    for directory in created.iter().rev() {
        let _ = fs::remove_dir(directory);
    }
}
