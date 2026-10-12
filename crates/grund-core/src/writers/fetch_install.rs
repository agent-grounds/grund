//! Atomic installation for the explicit materializer: complete bytes by
//! same-directory rename, the missing parent chain created and rolled back on
//! failure (§FS-fetch.4, §FS-fetch.5, §REQ-no-data-loss.2). What it reads of the
//! file it replaces it asks sources for (§AR-system.2.13); what it writes is its own.

use std::fs;
use std::path::{Path, PathBuf};

use super::fetch::{FetchFailure, fetch_io, fetch_operational};
use crate::sources::{path_metadata, read_bytes};

/// §FS-fetch.4 / §FS-fetch.5: install complete bytes by same-directory rename,
/// leaving an unchanged target untouched.
pub(super) fn atomic_install(
    path: &Path,
    bytes: &[u8],
    diagnostic: &mut Option<anyhow::Error>,
) -> std::result::Result<(), FetchFailure> {
    if read_bytes(path).ok().as_deref() == Some(bytes) {
        return Ok(());
    }
    let existing_permissions = match path_metadata(path) {
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
        match path_metadata(cursor) {
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
