/// `grund config migrate --to 2 [PATH] [--write] [--drop-comments]`
/// (§FS-config-migrate.1): the usage is answered before anything is loaded
/// (§FS-config-migrate.1.2), the engine returns one record, and this renders it —
/// the candidate on stdout (§FS-config-migrate.2.1), the report on stderr in the
/// order of §FS-config-migrate.2.2, and the exit of §FS-config-migrate.2.3.
fn command_config_migrate(args: &[String]) -> ExitCode {
    let mut to: Option<String> = None;
    let mut write = false;
    let mut drop_comments = false;
    let mut path: Option<PathBuf> = None;
    let mut words = args.iter();
    while let Some(arg) = words.next() {
        match arg.as_str() {
            "--write" => write = true,
            "--drop-comments" => drop_comments = true,
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
            if let Some(candidate) = report.stdout() {
                print!("{candidate}");
            }
            render_migrate_report(&report);
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

/// §FS-config-migrate.2.2: the report's lines, in order, never wrapped.
fn render_migrate_report(report: &MigrateReport) {
    let config = &report.config;
    if report.write {
        eprintln!("{config}: version 1 → 2, --write");
    } else {
        eprintln!("{config}: version 1 → 2, preview, nothing written");
    }
    for refusal in &report.refusals {
        match refusal.anchor {
            RefusalAnchor::Line(line) => eprintln!("refused: {config}:{line}: {}", refusal.what),
            RefusalAnchor::Unset => eprintln!("refused: {config}: {}", refusal.what),
            RefusalAnchor::File => eprintln!("refused: {config} {}", refusal.what),
        }
        for detail in &refusal.details {
            eprintln!("  {detail}");
        }
        eprintln!("  repair: {}", refusal.repair);
    }
    if !report.comments.is_empty() {
        eprintln!(
            "comments with no single key to follow (--write refuses; --drop-comments drops exactly these):"
        );
        for comment in &report.comments {
            eprintln!("  {}  {}", comment_lines(config, comment), comment.place);
        }
    }
    for notice in &report.notices {
        let verb = if notice.dropped { "dropped" } else { "changed" };
        eprintln!("{verb}: {config}:{}: {}: {}", notice.line, notice.key, notice.why);
    }
    if report.refusals.is_empty() {
        for mapped in &report.mapped {
            eprintln!(
                "mapped: {config}:{}: {} → {}",
                mapped.line, mapped.key, mapped.spelling
            );
        }
        for written in &report.written {
            eprintln!("written: {written}: version 1's default, written out");
        }
        for member in &report.members {
            eprintln!("member: {member}");
        }
        match &report.gates {
            Some(GateResult::Equal) => eprintln!(
                "compared: the records are equal, and grund check --suggestions on this tree gives the same findings and the same exit before and after"
            ),
            Some(GateResult::Differs { gate, lines }) => {
                match gate {
                    Gate::Records => eprintln!("differs: the records"),
                    Gate::Report => eprintln!("differs: grund check --suggestions on this tree"),
                }
                for line in lines {
                    eprintln!("{line}");
                }
            }
            None => {}
        }
        if report.code_row_note {
            eprintln!(
                "note: the next grund init adds a `code` row to the AGENTS.md Project map, because the complement's places are now listed; grund check does not compare that table"
            );
        }
    }
    if report.wrote {
        eprintln!(
            "wrote {config}: version 1 ({} lines) replaced by version 2",
            report.v1_lines
        );
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
            eprintln!("dropped the comments at {joined} (version 1 lines)");
        }
        eprintln!("to get version 1 back: it is in version control, e.g. git restore {config}");
        eprintln!(
            "to see this candidate again: restore version 1 and run grund config migrate --to 2"
        );
    } else if report.write {
        eprintln!("not written: {config} is unchanged");
    }
}

/// `<config>:<first>-<last>`, or `<config>:<line>` for one line.
fn comment_lines(config: &str, comment: &ListedComment) -> String {
    if comment.first == comment.last {
        format!("{config}:{}", comment.first)
    } else {
        format!("{config}:{}-{}", comment.first, comment.last)
    }
}
