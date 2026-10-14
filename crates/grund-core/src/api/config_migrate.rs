//! `grund config migrate --to 2` as the embedding surface (§FS-config-migrate,
//! §AR-config.8): the translation is config's, and this adds what sits above it —
//! the report gate, which runs `check` twice (§FS-config-migrate.5.2), the
//! members still on version 1 (§FS-config-migrate.8), and the write, through the
//! shared atomic writer (§FS-config-migrate.6). It returns the record and
//! renders nothing.

use std::path::PathBuf;

use super::check::{CheckOpts, CheckOutput, check_with_run_warnings};
use crate::config::migrate::{
    Gate, GateResult, Located, MigrateFailure, MigrateOutcome, MigrateReport, Prepared, locate,
    translate,
};
use crate::config::{
    Invocation, PathBase, Project, Selection, config_at, config_file_in, with_report_path_base,
};
use crate::model::{Finding, format_path, relative_from_base};
use crate::writers::{Install, InstallFailure, atomic_install};

/// The options of one `grund config migrate --to 2` run (§FS-config-migrate.1).
#[derive(Clone, Debug)]
pub struct MigrateOpts {
    pub path: PathBuf,
    pub write: bool,
    pub drop_comments: bool,
}

/// §FS-config-migrate: migrate the config `opts.path` discovers, writing it only
/// with `write`, past both gates, and only by an atomic replace.
pub fn config_migrate(opts: &MigrateOpts) -> Result<MigrateOutcome, MigrateFailure> {
    let Some(located) = locate(&opts.path) else {
        return Err(MigrateFailure::NoConfig {
            path: format_path(&opts.path),
        });
    };
    let prepared =
        translate(&located, opts.write, opts.drop_comments).map_err(MigrateFailure::Load)?;
    let (mut report, v1_text, v1, line_map) = match prepared {
        Prepared::AlreadyV2 => {
            return Ok(MigrateOutcome::AlreadyV2 {
                config: located.config,
            });
        }
        Prepared::Translated {
            report,
            v1_text,
            v1,
            line_map,
        } => (*report, v1_text, *v1, line_map),
    };
    if !report.refusals.is_empty() {
        return Ok(MigrateOutcome::Migrated(report));
    }
    report.members = members(&located);
    if let (Some(GateResult::Equal), Some(candidate)) = (&report.gates, &report.candidate) {
        let lines = report_gate(&located, &v1, candidate, &line_map, &report.config);
        if !lines.is_empty() {
            report.gates = Some(GateResult::Differs {
                gate: Gate::Report,
                lines,
            });
        }
    }
    if opts.write && !report.refused() {
        write(&located, &v1_text, &report)?;
        report.wrote = true;
    }
    Ok(MigrateOutcome::Migrated(report))
}

/// §FS-config-migrate.6: the candidate over the config, by the shared writer,
/// which re-reads the original just before the rename (§FS-config-migrate.6.3).
fn write(located: &Located, v1_text: &str, report: &MigrateReport) -> Result<(), MigrateFailure> {
    let candidate = report.candidate.as_deref().unwrap_or_default();
    let install = Install {
        bytes: candidate.as_bytes(),
        tag: "migrate",
        expected: Some(v1_text.as_bytes()),
    };
    atomic_install(&located.file, install).map_err(|failure| match failure {
        InstallFailure::Changed => MigrateFailure::Changed {
            config: report.config.clone(),
        },
        InstallFailure::Io { error, context } => MigrateFailure::Write {
            config: report.config.clone(),
            reason: format!("{context}: {error}"),
        },
        InstallFailure::Operational(reason) => MigrateFailure::Write {
            config: report.config.clone(),
            reason,
        },
    })
}

/// §FS-config-migrate.8: each member of the root's `[workspace]` still on
/// version 1, by its config's path from the root.
fn members(located: &Located) -> Vec<String> {
    let invocation = Invocation::new(&located.root, false, Selection::Project);
    let Ok(config) = config_at(&located.root, &located.root, None, &invocation) else {
        return Vec::new();
    };
    if !config.project().workspace.declared {
        return Vec::new();
    }
    let Ok(roots) = crate::workspace::expand_workspace_members(&config) else {
        return Vec::new();
    };
    roots
        .iter()
        .filter_map(|root| {
            let file = config_file_in(root)?;
            let text = std::fs::read_to_string(&file).ok()?;
            if crate::config::selects_v2(&text) {
                return None;
            }
            let member = format_path(&relative_from_base(&located.root, root));
            let file = format_path(&relative_from_base(&located.root, &file));
            Some(format!(
                "{file} is version 1; migrate it with grund config migrate --to 2 {member}"
            ))
        })
        .collect()
}

/// §FS-config-migrate.5.2: `check --suggestions` on the root with the file as
/// read and with the candidate in its place; the lines that differ, `- ` for
/// version 1 and `+ ` for the candidate.
pub(super) fn report_gate(
    located: &Located,
    v1: &Project,
    candidate: &str,
    line_map: &[(usize, usize)],
    config: &str,
) -> Vec<String> {
    let run = || {
        check_with_run_warnings(CheckOpts {
            path: located.root.clone(),
            path_provided: true,
            include_suggestions: true,
            ..CheckOpts::default()
        })
    };
    let before = lines(run(), None, config);
    // `[output] relative_paths = false` is `--path-base=invocation` once dropped.
    let base = crate::config::report_path_base()
        .or((!v1.presentation.output.relative_paths).then_some(PathBase::Invocation));
    let after = crate::config::with_config_text(&located.file, candidate, || {
        with_report_path_base(base, run)
    });
    let after = lines(after, Some(line_map), config);
    let mut out: Vec<String> = before
        .iter()
        .filter(|line| !after.contains(line))
        .map(|line| format!("- {line}"))
        .collect();
    out.extend(
        after
            .iter()
            .filter(|line| !before.contains(line))
            .map(|line| format!("+ {line}")),
    );
    out
}

/// One check run as comparable lines: each finding by channel, the run
/// warnings, and the exit. A finding in the config is moved to its v1 line, and
/// the v2 table names in a message are spelled as v1.
fn lines(
    (warnings, output): (Vec<Finding>, anyhow::Result<CheckOutput>),
    line_map: Option<&[(usize, usize)]>,
    config: &str,
) -> Vec<String> {
    let v1_line = |map: &[(usize, usize)], at: usize| {
        map.iter()
            .find(|(v2, _)| *v2 == at)
            .map_or(usize::MAX, |(_, v1)| *v1)
    };
    let finding = |channel: &str, f: &Finding| {
        let mut line = f.line;
        if let Some(map) = line_map
            && f.path.as_deref() == Some(config)
        {
            line = f.line.map(|at| v1_line(map, at));
        }
        let message = match line_map {
            Some(map) => v1_spelling(&config_lines(&f.message, config, |at| v1_line(map, at))),
            None => f.message.clone(),
        };
        format!(
            "{channel} {} {} {}:{}:{} {message}",
            f.severity,
            f.code,
            f.path.as_deref().unwrap_or("-"),
            line.map_or("-".into(), |l| l.to_string()),
            f.column.map_or("-".into(), |c| c.to_string()),
        )
    };
    let mut out: Vec<String> = warnings.iter().map(|w| finding("run-warning", w)).collect();
    match output {
        Ok(output) => {
            let report = &output.report;
            out.extend(report.errors.iter().map(|f| finding("error", f)));
            out.extend(report.warnings.iter().map(|f| finding("warning", f)));
            out.extend(report.suggestions.iter().map(|f| finding("suggestion", f)));
            out.extend(output.warnings.iter().map(|f| finding("run-warning", f)));
            let failed = !report.errors.is_empty() || output.had_scan_errors;
            out.push(format!("exit {}", u8::from(failed)));
        }
        Err(error) => out.push(format!("error {error:#}")),
    }
    out
}

/// Each `<config>:<line>` a message names, its line moved by `v1_line`: a
/// message that points into the config points at the key's v1 line.
pub(super) fn config_lines(
    message: &str,
    config: &str,
    v1_line: impl Fn(usize) -> usize,
) -> String {
    let prefix = format!("{config}:");
    let mut out = String::new();
    let mut rest = message;
    while let Some(at) = rest.find(&prefix) {
        let (head, tail) = rest.split_at(at + prefix.len());
        out.push_str(head);
        let digits = tail.len() - tail.trim_start_matches(|c: char| c.is_ascii_digit()).len();
        // `sub/grund.toml:3` names another config, not this one.
        let other = rest[..at].ends_with(|c: char| c == '/' || c == '\\' || c.is_alphanumeric());
        match tail[..digits].parse::<usize>() {
            Ok(line) if !other => out.push_str(&v1_line(line).to_string()),
            _ => out.push_str(&tail[..digits]),
        }
        rest = &tail[digits..];
    }
    out.push_str(rest);
    out
}

/// §FS-config-migrate.5.2: the config-table names in a message, spelled as v1.
pub(super) fn v1_spelling(message: &str) -> String {
    const NAMES: &[(&str, &str)] = &[
        ("[schema.sources] exclude", "[scan] exclude"),
        ("[schema.sources] extensions", "[scan] extensions"),
        ("[schema.kinds.code] folders", "[scan] include"),
        ("[schema.kinds.code] files", "[scan] include"),
        ("[rules.citations", "[citations"),
        ("[schema] id_format", "[id] format"),
    ];
    let mut out = message.to_string();
    for (v2, v1) in NAMES {
        out = out.replace(v2, v1);
    }
    out
}
