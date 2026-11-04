//! Snapshot-home discovery for the explicit materializer (§FS-fetch.4, §FS-fetch.5), deciding from
//! what `fetch_home.rs` reads; the shared writer in `atomic_install.rs` installs what they decide
//! (§REQ-no-data-loss.2).

use std::path::Path;

use super::atomic_install::{Install, InstallFailure, atomic_install};
use super::fetch::{FetchFailure, fetch_io, fetch_operational};
use super::fetch_home::{Requested, declarations_at_depth, refuse_nested_declaration};
use crate::config::Config;
use crate::grammar::Grammar;
use crate::model::{OperationDiagnostic, format_path};
use crate::scanner::walk_scannable_files_with_sources;
use crate::sources::{path_exists, path_is_dir, read_bytes};

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

/// §FS-fetch.4: replace only the named H2 or insert it in ID order.
pub(super) fn write_file_home(
    path: &Path,
    grammar: &Grammar,
    requested: &Requested<'_>,
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
        .filter(|declaration| &declaration.id == requested.id)
        .collect::<Vec<_>>();
    if matching.len() > 1 {
        return Err(fetch_operational(
            "snapshot home has multiple declarations for requested ID",
        ));
    }
    let (start, end) = if let Some(existing) = matching.first() {
        refuse_nested_declaration(
            &original,
            existing,
            path,
            grammar,
            requested,
            "to its own H2",
        )?;
        (existing.start, existing.end)
    } else {
        let requested = grammar.render(requested.id, 3);
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
    requested: &Requested<'_>,
    local: &str,
    snapshot: &[u8],
    diagnostic: &mut Option<anyhow::Error>,
) -> std::result::Result<(), FetchFailure> {
    let mut matches = Vec::new();
    let mut nested = None;
    if path_exists(folder) {
        if !path_is_dir(folder) {
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
            let bytes = read_bytes(&path).map_err(|err| {
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
                .any(|declaration| declaration.id == *requested.id);
            if contains_requested {
                let declaration = declarations
                    .iter()
                    .find(|declaration| declaration.id == *requested.id)
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
                if declaration.id == *requested.id {
                    nested = refuse_nested_declaration(
                        &bytes,
                        &declaration,
                        &path,
                        &config.grammar,
                        requested,
                        "to its own file",
                    )
                    .err();
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
    if let Some(refusal) = nested {
        return Err(refusal);
    }
    let target = matches
        .first()
        .and_then(|declaration| declaration.path.clone())
        .unwrap_or_else(|| folder.join(format!("{local}.md")));
    if matches.is_empty() && path_exists(&target) {
        return Err(fetch_operational(format!(
            "snapshot file {} already exists for a different ID",
            target.display()
        )));
    }
    install(&target, snapshot, diagnostic)
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
