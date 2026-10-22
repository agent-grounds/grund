//! `grund config migrate --to 2` as the embedding surface (§FS-config-migrate,
//! §AR-config.8): the translation is config's, and this adds what sits above it —
//! the report gate, which runs `check` twice (§FS-config-migrate.5.2), the
//! members still on version 1 (§FS-config-migrate.8), and the write, through the
//! shared atomic writer (§FS-config-migrate.6). It returns the record and
//! renders nothing.

use std::path::{Path, PathBuf};

use super::check::{CheckOpts, CheckOutput, check_with_projects};
use super::config_findings::{CONFIG_VERSION_1, version_1_caution};
use super::config_migrate_guidance::guidance_refusals;
use crate::checker::stale_section_noun;
use crate::config::migrate::{
    Gate, GateResult, Located, MigrateFailure, MigrateOutcome, MigrateReport, Prepared, locate,
    member_on_v1, translate,
};
use crate::config::{Invocation, PathBase, Project, Selection, config_at, with_report_path_base};
use crate::model::{Code, Finding, GENERATED_SECTIONS, format_path, relative_from_base};
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
    let (mut report, v1_text, v1, line_map, value_lines) = match prepared {
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
            value_lines,
        } => (*report, v1_text, *v1, line_map, value_lines),
    };
    if !report.refusals.is_empty() {
        return Ok(MigrateOutcome::Migrated(Box::new(report)));
    }
    report.members = members(&located);
    // §FS-config-migrate.5.3: guidance `check` would compare refuses before the report gate.
    if let (Some(GateResult::Equal), Some(candidate)) = (&report.gates, &report.candidate)
        && let Some(refusals) = guidance_refusals(&located, candidate, &value_lines)
        && !refusals.is_empty()
    {
        report.refusals = refusals;
        return Ok(MigrateOutcome::Migrated(Box::new(report)));
    }
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
    Ok(MigrateOutcome::Migrated(Box::new(report)))
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
            let file = member_on_v1(root)?;
            let member = format_path(&relative_from_base(&located.root, root));
            let file = format_path(&relative_from_base(&located.root, &file));
            // §FS-check.checks.config-version-1: the caution's renderer, so both say it alike.
            Some(version_1_caution(&file, Some(&member)))
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
        check_with_projects(CheckOpts {
            path: located.root.clone(),
            path_provided: true,
            include_suggestions: true,
            ..CheckOpts::default()
        })
    };
    let before = lines(run(), None, config, &located.root);
    // `[output] relative_paths = false` is `--path-base=invocation` once dropped.
    let base = crate::config::report_path_base()
        .or((!v1.presentation.output.relative_paths).then_some(PathBase::Invocation));
    let after = crate::config::with_config_text(&located.file, candidate, || {
        with_report_path_base(base, run)
    });
    let after = lines(after, Some(line_map), config, &located.root);
    // Compared as multisets: a line twice in one run and once in the other differs.
    let mut out = surplus(&before, &after, "- ");
    out.extend(surplus(&after, &before, "+ "));
    out
}

/// Each line of `ours` that `theirs` holds fewer times, once per extra copy,
/// prefixed with `mark`.
pub(super) fn surplus(ours: &[String], theirs: &[String], mark: &str) -> Vec<String> {
    let mut left: Vec<&String> = theirs.iter().collect();
    ours.iter()
        .filter(|line| match left.iter().position(|other| other == line) {
            Some(at) => {
                left.swap_remove(at);
                false
            }
            None => true,
        })
        .map(|line| format!("{mark}{line}"))
        .collect()
}

/// One check run as comparable lines: each project checked, each finding by
/// channel, the run warnings, and the exit. A finding in the config is moved to
/// its v1 line, and the v2 table names in a message are spelled as v1.
fn lines(
    (warnings, output): (Vec<Finding>, anyhow::Result<(CheckOutput, Vec<PathBuf>)>),
    line_map: Option<&[(usize, usize)]>,
    config: &str,
    root_dir: &Path,
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
    // §FS-config-migrate.5.2: the v1 run's own caution is what migrating ends, not a
    // difference the candidate makes, so neither run carries it; nor a stale block
    // outside the generated sections, which the guidance gate passes (.5.3).
    let compared = |f: &&Finding| f.code != CONFIG_VERSION_1.as_str() && !describes_config(f);
    let mut out: Vec<String> = warnings
        .iter()
        .filter(compared)
        .map(|w| finding("run-warning", w))
        .collect();
    match output {
        Ok((output, projects)) => {
            out.extend(projects.iter().map(|root| {
                let shown = format_path(&relative_from_base(root_dir, root));
                format!("project {}", if shown.is_empty() { "." } else { &shown })
            }));
            let report = &output.report;
            out.extend(
                report
                    .errors
                    .iter()
                    .filter(compared)
                    .map(|f| finding("error", f)),
            );
            out.extend(
                report
                    .warnings
                    .iter()
                    .filter(compared)
                    .map(|f| finding("warning", f)),
            );
            out.extend(
                report
                    .suggestions
                    .iter()
                    .filter(compared)
                    .map(|f| finding("suggestion", f)),
            );
            out.extend(
                output
                    .warnings
                    .iter()
                    .filter(compared)
                    .map(|f| finding("run-warning", f)),
            );
            let failed = report.errors.iter().any(|f| compared(&f)) || output.had_scan_errors;
            out.push(format!("exit {}", u8::from(failed)));
        }
        Err(error) => out.push(format!("error {error:#}")),
    }
    out
}

/// §FS-config-migrate.5.2: an `agents-init` finding about a block's text outside
/// its version and its five generated sections, which describes the config and
/// is expected to change under a migration (§FS-config-migrate.5.3).
pub(super) fn describes_config(finding: &Finding) -> bool {
    finding.code == Code::AGENTS_INIT.as_str()
        && stale_section_noun(&finding.message)
            .is_some_and(|noun| !GENERATED_SECTIONS.contains(&noun))
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
