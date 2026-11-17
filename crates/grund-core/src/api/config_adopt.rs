//! `grund config adopt-builtins` as the embedding surface (§FS-config-adopt-builtins,
//! §AR-config.10): the plan, the edit and the gate are config's, and this adds what sits above
//! them. That is the two `check --suggestions` runs, kept as a difference to print rather than a
//! verdict (§FS-config-adopt-builtins.6.4, §AR-config.10.3), and the write, through the shared
//! atomic writer (§FS-config-adopt-builtins.8). It returns the record and renders nothing.

use std::path::{Path, PathBuf};

use super::check::{CheckOpts, CheckOutput, check_with_projects};
use crate::config::adopt::{
    AdoptFailure, AdoptOutcome, AdoptReport, Compared, Prepared, prepare, still_holds,
};
use crate::config::migrate::{Located, locate};
use crate::model::{Finding, format_path};
use crate::writers::{Holds, Install, InstallFailure, atomic_install};

/// The options of one `grund config adopt-builtins` run (§FS-config-adopt-builtins.1).
#[derive(Clone, Debug)]
pub struct AdoptOpts {
    pub path: PathBuf,
    pub write: bool,
    pub drop_comments: bool,
}

/// §FS-config-adopt-builtins: adopt the built-in languages in the config `opts.path` discovers,
/// writing it only with `write`, past the whole gate, and only by an atomic replace.
pub fn config_adopt(opts: &AdoptOpts) -> Result<AdoptOutcome, AdoptFailure> {
    let Some(located) = locate(&opts.path) else {
        return Err(AdoptFailure::NoConfig {
            path: format_path(&opts.path),
        });
    };
    let (mut report, text) = match prepare(&located, opts.write, opts.drop_comments)? {
        Prepared::Nothing(outcome) => return Ok(outcome),
        Prepared::Symlink(report) => return Ok(AdoptOutcome::Adopted(report)),
        Prepared::Planned { report, text } => (*report, text),
    };
    // §FS-config-adopt-builtins.6.4: only a candidate that reads as planned is compared.
    if report.load_error.is_none() && report.readings.is_empty() {
        report.compared = Some(compare(&located, &report.candidate)?);
        if !still_holds(&located, &text) {
            return Err(AdoptFailure::Changed {
                config: located.config,
            });
        }
    }
    if opts.write && !report.refused() {
        write(&located, &text, &report)?;
        report.wrote = true;
    }
    Ok(AdoptOutcome::Adopted(Box::new(report)))
}

/// §FS-config-adopt-builtins.6.4: `check --suggestions` on the root with the file as read and with
/// the candidate in its place, as migration's report gate runs it (§FS-config-migrate.5.2): the
/// two exits, and each finding only one of the runs reports.
fn compare(located: &Located, candidate: &str) -> Result<Compared, AdoptFailure> {
    let run = || {
        check_with_projects(CheckOpts {
            path: located.root.clone(),
            path_provided: true,
            include_suggestions: true,
            ..CheckOpts::default()
        })
    };
    let (before, file) = findings(run())?;
    between_checks(&located.file);
    let after = crate::config::with_config_text(&located.file, candidate, run);
    let (after, adopted) = findings(after)?;
    // Compared as multisets: a finding twice in one run and once in the other changed.
    Ok(Compared {
        before,
        after,
        removed: surplus(&file, &adopted),
        added: surplus(&adopted, &file),
    })
}

/// One run's exit, as `grund check` maps it (§FS-cli.5), and its findings by channel: the run
/// warnings first, then the report's errors, warnings and suggestions.
fn findings(
    (warnings, output): (Vec<Finding>, anyhow::Result<(CheckOutput, Vec<PathBuf>)>),
) -> Result<(u8, Vec<(&'static str, Finding)>), AdoptFailure> {
    let (output, _) = output.map_err(AdoptFailure::Compare)?;
    let report = output.report;
    let exit = if output.had_scan_errors {
        2
    } else {
        u8::from(!report.errors.is_empty())
    };
    let mut all: Vec<(&'static str, Finding)> =
        warnings.into_iter().map(|w| ("warning", w)).collect();
    all.extend(report.errors.into_iter().map(|f| ("error", f)));
    all.extend(report.warnings.into_iter().map(|f| ("warning", f)));
    all.extend(report.suggestions.into_iter().map(|f| ("suggestion", f)));
    Ok((exit, all))
}

/// Each finding of `ours` that `theirs` holds fewer times, once per extra copy.
fn surplus(
    ours: &[(&'static str, Finding)],
    theirs: &[(&'static str, Finding)],
) -> Vec<(&'static str, Finding)> {
    let mut left: Vec<&(&'static str, Finding)> = theirs.iter().collect();
    ours.iter()
        .filter(
            |finding| match left.iter().position(|other| other == finding) {
                Some(at) => {
                    left.swap_remove(at);
                    false
                }
                None => true,
            },
        )
        .cloned()
        .collect()
}

/// §FS-config-adopt-builtins.8.1: the candidate over the config, by the shared writer, which reads
/// the original again just before the rename (§FS-config-migrate.6.3); adoption gives only the
/// messages.
fn write(located: &Located, text: &str, report: &AdoptReport) -> Result<(), AdoptFailure> {
    let install = Install {
        bytes: report.candidate.as_bytes(),
        tag: "adopt",
        expected: Holds::Bytes(text.as_bytes()),
    };
    let config = || report.config.clone();
    atomic_install(&located.file, install).map_err(|failure| match failure {
        InstallFailure::Changed => AdoptFailure::Changed { config: config() },
        InstallFailure::Io { error, context } => AdoptFailure::Write {
            config: config(),
            reason: format!("{context}: {error}"),
        },
        InstallFailure::Operational(reason) => AdoptFailure::Write {
            config: config(),
            reason,
        },
    })
}

#[cfg(test)]
thread_local! {
    /// The bytes a test writes over the config between the two `check` runs
    /// (§FS-config-adopt-builtins.6.4).
    pub(crate) static BETWEEN_CHECKS: std::cell::RefCell<Option<String>> =
        const { std::cell::RefCell::new(None) };
}

/// A test's edit of the config between the two `check` runs, written by the shared fixture
/// writer, since only the components that own the disk touch it (§AR-system.5).
#[cfg(test)]
fn between_checks(path: &Path) {
    if let Some(text) = BETWEEN_CHECKS.with(|slot| slot.borrow_mut().take()) {
        crate::testing::write(path, &text);
    }
}

#[cfg(not(test))]
fn between_checks(_: &Path) {}
