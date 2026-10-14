//! The shared atomic writer (§AR-config.8.3): complete bytes by same-directory
//! rename, the missing parent chain created and rolled back on failure, and the
//! original's permissions kept. `fetch` installs a snapshot through it
//! (§FS-fetch.4, §FS-fetch.5, §REQ-no-data-loss.2) and `config migrate` replaces
//! the config through it, asking it first to make sure the file did not change
//! under the run (§FS-config-migrate.6.2, §FS-config-migrate.6.3). What it reads
//! of the file it replaces it asks sources for (§AR-system.2.13); what it writes
//! is its own. Each caller maps a failure into its own refusal.

use std::fs;
use std::path::{Path, PathBuf};

use crate::sources::{path_metadata, read_bytes};

/// One install: the bytes, the temporary file's name, and what the target must
/// still hold just before the rename.
pub(crate) struct Install<'a> {
    pub(crate) bytes: &'a [u8],
    /// The temporary file is `.grund-<tag>-<pid>-<n>.tmp` beside the target.
    pub(crate) tag: &'a str,
    /// The target's bytes when the run read it; `None` installs over whatever
    /// is there (§FS-config-migrate.6.3).
    pub(crate) expected: Option<&'a [u8]>,
}

/// Why an install left the target as it was.
#[derive(Debug)]
pub(crate) enum InstallFailure {
    /// An I/O error, with the sentence that says what was being done.
    Io {
        error: std::io::Error,
        context: String,
    },
    /// A refusal no I/O error carries.
    Operational(String),
    /// The target's bytes are no longer `expected` (§FS-config-migrate.6.3).
    Changed,
}

/// §FS-fetch.4, §FS-config-migrate.6.2: install complete bytes by same-directory
/// rename, leaving an unchanged target untouched and, on any failure, the target
/// byte for byte and no temporary file behind.
pub(crate) fn atomic_install(path: &Path, install: Install<'_>) -> Result<(), InstallFailure> {
    let current = read_bytes(path).ok();
    if install
        .expected
        .is_some_and(|expected| current.as_deref() != Some(expected))
    {
        return Err(InstallFailure::Changed);
    }
    if current.as_deref() == Some(install.bytes) {
        return Ok(());
    }
    let io = |error, context: String| InstallFailure::Io { error, context };
    let existing_permissions = match path_metadata(path) {
        Ok(metadata) => Some(metadata.permissions()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => None,
        Err(err) => {
            return Err(io(
                err,
                format!("cannot read metadata for {}", path.display()),
            ));
        }
    };
    let parent = path
        .parent()
        .ok_or_else(|| InstallFailure::Operational(format!("cannot write {}", path.display())))?;
    let created_directories = create_parent_directories(parent, path)?;
    let fail = |failure| {
        rollback_created_directories(&created_directories);
        Err(failure)
    };
    let mut temporary = None;
    for attempt in 0..100u32 {
        let name = format!(
            ".grund-{}-{}-{attempt}.tmp",
            install.tag,
            std::process::id()
        );
        let candidate = parent.join(name);
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(file) => {
                temporary = Some((candidate, file));
                break;
            }
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(err) => return fail(io(err, format!("cannot write {}", path.display()))),
        }
    }
    let Some((temporary_path, mut file)) = temporary else {
        return fail(InstallFailure::Operational(format!(
            "cannot create temporary file for {}",
            path.display()
        )));
    };
    use std::io::Write;
    let installed = fault(Stage::Write)
        .and_then(|_| file.write_all(install.bytes))
        .and_then(|_| match existing_permissions {
            Some(permissions) => file.set_permissions(permissions),
            None => Ok(()),
        })
        .and_then(|_| fault(Stage::Sync))
        .and_then(|_| file.sync_all());
    drop(file);
    if let Err(err) = installed {
        let _ = fs::remove_file(&temporary_path);
        return fail(io(err, format!("cannot write {}", path.display())));
    }
    // §FS-config-migrate.6.3: read the target again just before the rename.
    if let Some(expected) = install.expected {
        concurrent_edit(path);
        if read_bytes(path).ok().as_deref() != Some(expected) {
            let _ = fs::remove_file(&temporary_path);
            return fail(InstallFailure::Changed);
        }
    }
    if let Err(err) = fault(Stage::Rename).and_then(|_| fs::rename(&temporary_path, path)) {
        let _ = fs::remove_file(&temporary_path);
        return fail(io(
            err,
            format!("cannot atomically replace {}", path.display()),
        ));
    }
    Ok(())
}

/// Create only the missing parent chain and remember exactly what this install
/// added, so every later failure can restore the prior tree (§FS-fetch.4,
/// §FS-fetch.5, §REQ-no-data-loss.2).
fn create_parent_directories(parent: &Path, target: &Path) -> Result<Vec<PathBuf>, InstallFailure> {
    let cannot_write = || format!("cannot write {}", target.display());
    let mut missing = Vec::new();
    let mut cursor = parent;
    loop {
        match path_metadata(cursor) {
            Ok(metadata) if metadata.is_dir() => break,
            Ok(_) => {
                return Err(InstallFailure::Operational(format!(
                    "cannot write {}: {} is not a directory",
                    target.display(),
                    cursor.display()
                )));
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                missing.push(cursor.to_path_buf());
                let Some(up) = cursor.parent() else {
                    let context = cannot_write();
                    return Err(InstallFailure::Io {
                        error: err,
                        context,
                    });
                };
                cursor = up;
            }
            Err(error) => {
                let context = cannot_write();
                return Err(InstallFailure::Io { error, context });
            }
        }
    }

    let mut created = Vec::new();
    for directory in missing.into_iter().rev() {
        if let Err(error) = fs::create_dir(&directory) {
            rollback_created_directories(&created);
            let context = cannot_write();
            return Err(InstallFailure::Io { error, context });
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

/// The steps a test may fail an install at (§FS-config-migrate.6.2).
#[cfg_attr(not(test), allow(dead_code))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Stage {
    Write,
    Sync,
    Rename,
}

#[cfg(test)]
thread_local! {
    /// The stage the next install on this thread fails at, and the edit a test
    /// makes to the target just before the rename check.
    pub(crate) static FAULT: std::cell::RefCell<(Option<Stage>, Option<Vec<u8>>)> =
        const { std::cell::RefCell::new((None, None)) };
}

#[cfg(test)]
fn fault(stage: Stage) -> std::io::Result<()> {
    FAULT.with(|fault| match fault.borrow().0 {
        Some(at) if at == stage => Err(std::io::Error::other(format!("injected at {stage:?}"))),
        _ => Ok(()),
    })
}

#[cfg(not(test))]
fn fault(_: Stage) -> std::io::Result<()> {
    Ok(())
}

/// A test's concurrent edit, made just before the rename check (§FS-config-migrate.6.3).
#[cfg(test)]
fn concurrent_edit(path: &Path) {
    if let Some(bytes) = FAULT.with(|fault| fault.borrow_mut().1.take()) {
        fs::write(path, bytes).expect("concurrent edit");
    }
}

#[cfg(not(test))]
fn concurrent_edit(_: &Path) {}
