/// `grund config migrate --to 2 [PATH] [--write] [--drop-comments] [--full]`
/// (§FS-config-migrate.1): the usage is answered before anything is loaded
/// (§FS-config-migrate.1.2), the engine returns one record, and this renders it —
/// the candidate on stdout (§FS-config-migrate.2.1), the report on stderr in the
/// order of §FS-config-migrate.2.2 or §FS-config-migrate.6.5, selected by
/// §FS-config-migrate.1.3, and the exit of §FS-config-migrate.2.3.
fn command_config_migrate(args: &[String]) -> ExitCode {
    let mut to: Option<String> = None;
    let mut write = false;
    let mut drop_comments = false;
    let mut full = false;
    let mut path: Option<PathBuf> = None;
    let mut words = args.iter();
    while let Some(arg) = words.next() {
        match arg.as_str() {
            "--write" => write = true,
            "--drop-comments" => drop_comments = true,
            "--full" => full = true,
            "--to" => to = words.next().cloned(),
            other if other.starts_with("--to=") => to = Some(other["--to=".len()..].to_string()),
            other if other.starts_with('-') => {
                eprintln!("error: unknown flag `{other}`");
                return ExitCode::from(2);
            }
            other => {
                if path.is_some() {
                    eprintln!("error: config migrate takes at most one path argument");
                    return ExitCode::from(2);
                }
                path = Some(PathBuf::from(other));
            }
        }
    }
    match to.as_deref() {
        Some("2") => {}
        None => {
            eprintln!("error: config migrate needs --to 2 (the version to migrate to)");
            return ExitCode::from(2);
        }
        Some(other) => {
            eprintln!("error: config migrate converts to version 2 only, not `{other}`");
            return ExitCode::from(2);
        }
    }
    let opts = MigrateOpts {
        path: path.unwrap_or_else(|| ".".into()),
        write,
        drop_comments,
    };
    match config_migrate(&opts) {
        Ok(MigrateOutcome::AlreadyV2 { config }) => {
            eprintln!("{config} is already version 2; nothing to migrate");
            ExitCode::SUCCESS
        }
        Ok(MigrateOutcome::Migrated(report)) => {
            let presentation = MigratePresentation::for_report(&report, full);
            if presentation == MigratePresentation::Complete {
                if let Some(candidate) = report.stdout() {
                    print!("{candidate}");
                }
            }
            render_migrate_report(&report, presentation, &mut std::io::stderr().lock())
                .expect("failed printing migration report to stderr");
            if report.refused() {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(failure) => {
            match failure {
                MigrateFailure::NoConfig { path } => {
                    eprintln!("error: no grund.toml at {path} or above it; grund init writes one");
                }
                MigrateFailure::Load(error) => eprintln!("error: {error:#}"),
                MigrateFailure::Write { config, reason } => eprintln!(
                    "error: {config}: could not write the migrated config ({reason}); the original is unchanged"
                ),
                MigrateFailure::Changed { config } => eprintln!(
                    "error: {config} changed while it was being migrated; the original is unchanged, run the migration again"
                ),
            }
            ExitCode::from(2)
        }
    }
}

/// §FS-config-migrate.6.5, §FS-config-migrate.2.1: one presentation decision
/// controls both streams; only an actual write can omit preview detail.
#[derive(Clone, Copy, Eq, PartialEq)]
enum MigratePresentation {
    CompactWrite,
    Complete,
}

impl MigratePresentation {
    /// §FS-config-migrate.1.3: `--full` restores complete successful-write output.
    fn for_report(report: &MigrateReport, full: bool) -> Self {
        if report.wrote && !full {
            Self::CompactWrite
        } else {
            Self::Complete
        }
    }
}

/// §FS-config-migrate.6.5: retained fields move ahead of advice on compact
/// success; §FS-config-migrate.2.2 keeps complete output in its legacy order.
/// The local sink lets callers inspect lines without redirecting global stderr.
fn render_migrate_report(
    report: &MigrateReport,
    presentation: MigratePresentation,
    out: &mut impl std::io::Write,
) -> std::io::Result<()> {
    if presentation == MigratePresentation::CompactWrite {
        render_migrate_outcome(report, out)?;
        render_migrate_notices(report, out)?;
        render_migrate_members(report, out)?;
        render_migrate_project_map(report, out)?;
    } else {
        render_migrate_complete(report, out)?;
        if report.wrote {
            render_migrate_outcome(report, out)?;
        }
    }
    if report.wrote {
        render_migrate_advice(report, out)?;
    } else if report.write {
        render_migrate_refused(report, out)?;
    }
    Ok(())
}

/// §FS-config-migrate.2.2: close a refused write with each blocking cause and
/// its own remedy, counting refusal blocks and listed comment ranges.
fn render_migrate_refused(
    report: &MigrateReport,
    out: &mut impl std::io::Write,
) -> std::io::Result<()> {
    let mut causes = Vec::new();
    match report.refusals.len() {
        0 => {}
        1 => causes.push("1 refusal (follow the repairs above)".to_string()),
        count => causes.push(format!("{count} refusals (follow the repairs above)")),
    }
    if !report.drop_comments {
        match report.comments.len() {
            0 => {}
            1 => causes.push(
                "1 comment has no single key to follow (move it, or re-run with --drop-comments)"
                    .to_string(),
            ),
            count => causes.push(format!(
                "{count} comments have no single key to follow (move them, or re-run with --drop-comments)"
            )),
        }
    }
    if let Some(GateResult::Differs { gate, .. }) = &report.gates {
        causes.push(match gate {
            Gate::Records => "the records differ (inspect the comparison above)".to_string(),
            Gate::Report => {
                "grund check --suggestions differs (inspect the comparison above)".to_string()
            }
        });
    }
    writeln!(
        out,
        "not written: {}; {} is unchanged",
        causes.join("; "),
        report.config
    )
}

/// §FS-config-migrate.2.2: the complete report's lines, in order, never wrapped.
fn render_migrate_complete(
    report: &MigrateReport,
    out: &mut impl std::io::Write,
) -> std::io::Result<()> {
    let config = &report.config;
    let refused_write = report.write && report.refused();
    if report.write {
        writeln!(out, "{config}: version 1 → 2, --write")?;
    } else {
        writeln!(out, "{config}: version 1 → 2, preview, nothing written")?;
    }
    for refusal in &report.refusals {
        match refusal.anchor {
            RefusalAnchor::Line(line) => {
                writeln!(out, "refused: {config}:{line}: {}", refusal.what)?
            }
            RefusalAnchor::Unset => writeln!(out, "refused: {config}: {}", refusal.what)?,
            RefusalAnchor::File => writeln!(out, "refused: {config} {}", refusal.what)?,
        }
        for detail in &refusal.details {
            writeln!(out, "  {detail}")?;
        }
        writeln!(out, "  repair: {}", refusal.repair)?;
    }
    if !report.comments.is_empty() {
        writeln!(
            out,
            "comments with no single key to follow (--write refuses; --drop-comments drops exactly these):"
        )?;
        for comment in &report.comments {
            writeln!(
                out,
                "  {}  {}",
                comment_lines(config, comment),
                comment.place
            )?;
        }
    }
    render_migrate_notices(report, out)?;
    if report.refusals.is_empty() {
        for mapped in &report.mapped {
            writeln!(
                out,
                "mapped: {config}:{}: {} → {}",
                mapped.line, mapped.key, mapped.spelling
            )?;
        }
        if !refused_write {
            for written in &report.written {
                writeln!(out, "written: {written}: version 1's default, written out")?;
            }
        }
        render_migrate_members(report, out)?;
        match &report.gates {
            Some(GateResult::Equal) => writeln!(
                out,
                "compared: the records are equal, and grund check --suggestions on this tree gives the same findings and the same exit before and after"
            )?,
            Some(GateResult::Differs { gate, lines }) => {
                match gate {
                    Gate::Records => writeln!(out, "differs: the records")?,
                    Gate::Report => {
                        writeln!(out, "differs: grund check --suggestions on this tree")?
                    }
                }
                for line in lines {
                    writeln!(out, "{line}")?;
                }
            }
            None => {}
        }
        if !refused_write {
            render_migrate_project_map(report, out)?;
        }
    }
    Ok(())
}

/// §FS-config-migrate.3.2, §FS-config-migrate.4.1.1: retain notices and carried
/// comment warnings in both presentations, using the report's file order.
fn render_migrate_notices(
    report: &MigrateReport,
    out: &mut impl std::io::Write,
) -> std::io::Result<()> {
    for notice in &report.notices {
        let verb = if notice.dropped { "dropped" } else { "changed" };
        writeln!(
            out,
            "{verb}: {}:{}: {}: {}",
            report.config, notice.line, notice.key, notice.why
        )?;
    }
    Ok(())
}

/// §FS-config-migrate.8: each remaining version-1 member's migration command.
fn render_migrate_members(
    report: &MigrateReport,
    out: &mut impl std::io::Write,
) -> std::io::Result<()> {
    for member in &report.members {
        writeln!(out, "member: {member}")?;
    }
    Ok(())
}

/// §FS-config-migrate.5.3: the distinct Project map note survives both modes.
fn render_migrate_project_map(
    report: &MigrateReport,
    out: &mut impl std::io::Write,
) -> std::io::Result<()> {
    if report.code_row_note {
        writeln!(
            out,
            "note: the next grund init adds a `code` row to the AGENTS.md Project map, because the complement's places are now listed; grund check reports the block stale until then"
        )?;
    }
    Ok(())
}

/// §FS-config-migrate.6.5: outcome and original dropped ranges are emitted once.
fn render_migrate_outcome(
    report: &MigrateReport,
    out: &mut impl std::io::Write,
) -> std::io::Result<()> {
    let config = &report.config;
    writeln!(
        out,
        "wrote {config}: version 1 ({} lines) replaced by version 2",
        report.v1_lines
    )?;
    if !report.comments.is_empty() {
        let ranges: Vec<String> = report
            .comments
            .iter()
            .map(|comment| comment_lines(config, comment))
            .collect();
        let joined = match ranges.split_last() {
            Some((last, [])) => last.clone(),
            Some((last, init)) => format!("{} and {last}", init.join(", ")),
            None => String::new(),
        };
        writeln!(out, "dropped the comments at {joined} (version 1 lines)")?;
    }
    Ok(())
}

/// §FS-config-migrate.6.5: undo/re-preview advice ends with the qualified
/// refresh reminder, without inspecting or writing managed instructions.
fn render_migrate_advice(
    report: &MigrateReport,
    out: &mut impl std::io::Write,
) -> std::io::Result<()> {
    writeln!(
        out,
        "to get version 1 back: it is in version control, e.g. git restore {}",
        report.config
    )?;
    writeln!(
        out,
        "to see this candidate again: restore version 1 and run grund config migrate --to 2"
    )?;
    writeln!(
        out,
        "note: existing AGENTS.md managed instructions may still describe version 1; run grund init in the migrated project to refresh them"
    )
}

/// §FS-config-migrate.4.3, §FS-config-migrate.6.5: `<config>:<first>-<last>`,
/// or `<config>:<line>` for one original line, in either presentation.
fn comment_lines(config: &str, comment: &ListedComment) -> String {
    if comment.first == comment.last {
        format!("{config}:{}", comment.first)
    } else {
        format!("{config}:{}-{}", comment.first, comment.last)
    }
}

#[cfg(test)]
#[path = "tests_config_migrate.rs"]
mod tests_config_migrate;
