/// `grund config adopt-builtins [PATH] [--write] [--drop-comments]` (§FS-config-adopt-builtins.1):
/// the usage is answered before anything is loaded (§FS-config-adopt-builtins.1.2), the engine
/// returns one record, and this renders it (§AR-config.10). The candidate goes to stdout
/// (§FS-config-adopt-builtins.5.1), the report to stderr in the order of
/// §FS-config-adopt-builtins.5.2, and the exit is §FS-config-adopt-builtins.10's.
fn command_config_adopt(args: &[String]) -> ExitCode {
    let mut write = false;
    let mut drop_comments = false;
    let mut path: Option<PathBuf> = None;
    for arg in args {
        match arg.as_str() {
            "--write" => write = true,
            "--drop-comments" => drop_comments = true,
            // §FS-config-adopt-builtins.1.2: `--to` and `--format` are unknown here.
            other if other.starts_with('-') => {
                eprintln!("error: unknown flag `{other}`");
                return ExitCode::from(2);
            }
            other => {
                if path.is_some() {
                    eprintln!("error: config adopt-builtins takes at most one path argument");
                    return ExitCode::from(2);
                }
                path = Some(PathBuf::from(other));
            }
        }
    }
    let opts = AdoptOpts {
        path: path.unwrap_or_else(|| ".".into()),
        write,
        drop_comments,
    };
    match config_adopt(&opts) {
        Ok(AdoptOutcome::NothingToAdopt { config, kept }) => {
            // §FS-config-adopt-builtins.9: no heading, no comparison, empty stdout.
            for kept in &kept {
                eprintln!("{}", adopt_kept_line(kept));
            }
            eprintln!("{config}: nothing to adopt; the file is unchanged");
            ExitCode::SUCCESS
        }
        Ok(AdoptOutcome::Adopted(report)) => {
            if let Some(candidate) = report.stdout() {
                print!("{candidate}");
            }
            let mut err = std::io::stderr().lock();
            render_adopt_report(&report, &mut err)
                .expect("failed printing the adoption report to stderr");
            ExitCode::from(report.exit())
        }
        Err(failure) => {
            eprintln!("error: {failure}");
            ExitCode::from(failure.exit())
        }
    }
}

/// §FS-config-adopt-builtins.5.2: the report, every line on `out` in the spec's order.
fn render_adopt_report(
    report: &AdoptReport,
    out: &mut impl std::io::Write,
) -> std::io::Result<()> {
    let config = &report.config;
    let mode = if report.write {
        "--write"
    } else {
        "preview, nothing written"
    };
    writeln!(out, "{config}: adopting version 2's built-in languages, {mode}")?;
    if report.symlink {
        // §FS-config-adopt-builtins.8.1: refused before the plan.
        writeln!(
            out,
            "refused: {config} is a symbolic link, and replacing it would replace the link with a regular file"
        )?;
        writeln!(
            out,
            "  repair: replace the link with the file it points to, or adopt the built-in languages in that file where it is the config"
        )?;
        return adopt_closing(out, report, "the config is a symbolic link");
    }
    for adopted in &report.adopted {
        let opt_in = if adopted.opt_in {
            ", opt-in, listed in languages"
        } else {
            ""
        };
        writeln!(
            out,
            "adopts: {}: definition {} \u{2192} built-in {}{opt_in}",
            adopted.suffix, adopted.definition, adopted.row
        )?;
        for difference in &adopted.differences {
            writeln!(out, "  reads differently: {difference}")?;
        }
    }
    for added in &report.added {
        writeln!(out, "added: {}: admitted with {}", added.suffixes.join(", "), added.row)?;
    }
    for kept in &report.kept {
        writeln!(out, "{}", adopt_kept_line(kept))?;
    }
    if !report.comments.is_empty() {
        // §FS-config-adopt-builtins.7.
        writeln!(
            out,
            "comments on lines adoption removes (--write refuses; --drop-comments drops exactly these):"
        )?;
        for comment in &report.comments {
            writeln!(out, "  {}  {}", adopt_range(config, comment), comment.place)?;
        }
    }
    if let Some(error) = &report.load_error {
        writeln!(out, "refused: the candidate does not load: {error}")?;
    }
    for line in report.readings.iter().chain(&report.records) {
        writeln!(out, "{line}")?;
    }
    if let Some(compared) = &report.compared {
        adopt_compared(out, compared)?;
    }
    if report.wrote {
        return adopt_wrote(out, report);
    }
    if report.refused() {
        return adopt_closing(out, report, &adopt_causes(report));
    }
    Ok(())
}

/// §FS-config-adopt-builtins.5.2 item 4: one kept suffix and why.
fn adopt_kept_line(kept: &KeptSuffix) -> String {
    let suffix = &kept.suffix;
    match &kept.why {
        KeptWhy::NoRow => format!(
            "kept: {suffix}: definition {0} and {suffix} = \"{0}\"; version 2 has no built-in language for `.{suffix}`",
            kept.definition
        ),
        KeptWhy::Class { row, class } => format!("kept: {suffix}: built-in {row} reads it as {class}"),
    }
}

/// `<config>:<line>` for one line, `<config>:<first>-<last>` for more.
fn adopt_range(config: &str, comment: &AdoptComment) -> String {
    if comment.first == comment.last {
        format!("{config}:{}", comment.first)
    } else {
        format!("{config}:{}-{}", comment.first, comment.last)
    }
}

/// §FS-config-adopt-builtins.6.4: the two `check --suggestions` runs, and each finding only one
/// reports, as the line `grund check` prints for it, in `check`'s order.
fn adopt_compared(out: &mut impl std::io::Write, compared: &Compared) -> std::io::Result<()> {
    let changes = compared.removed.len() + compared.added.len();
    let changed = match changes {
        0 => "no finding changes".to_string(),
        1 => "1 finding changes".to_string(),
        n => format!("{n} findings change"),
    };
    writeln!(
        out,
        "compared: the candidate reads as planned; grund check --suggestions: exit {} \u{2192} {}, {changed}",
        compared.before, compared.after
    )?;
    for (mark, findings) in [("-", &compared.removed), ("+", &compared.added)] {
        for (severity, finding) in check_order(findings) {
            writeln!(out, "  {mark} {}", check_text_line(severity, finding))?;
        }
    }
    Ok(())
}

/// Findings in the order `grund check` prints them: what it prints on stderr first, then each
/// channel in turn, sorted within it (§FS-errors.4.1).
fn check_order<'a>(findings: &'a [(&'static str, Finding)]) -> Vec<(&'static str, &'a Finding)> {
    let rank = |severity: &str| match severity {
        "error" => 0,
        "warning" => 1,
        _ => 2,
    };
    let mut ordered: Vec<(&'static str, &Finding)> =
        findings.iter().map(|(severity, f)| (*severity, f)).collect();
    ordered.sort_by(|(sa, a), (sb, b)| {
        (!a.is_cli_level(), rank(sa))
            .cmp(&(!b.is_cli_level(), rank(sb)))
            .then_with(|| finding_cmp(a, b))
    });
    ordered
}

/// §FS-config-adopt-builtins.8.2: what a write says after the report.
fn adopt_wrote(out: &mut impl std::io::Write, report: &AdoptReport) -> std::io::Result<()> {
    let config = &report.config;
    let at = |name: &str| {
        report
            .languages
            .iter()
            .position(|listed| listed == name)
            .unwrap_or(usize::MAX)
    };
    let mut rows: Vec<&str> = Vec::new();
    for adopted in &report.adopted {
        if !rows.contains(&adopted.row.as_str()) {
            rows.push(&adopted.row);
        }
    }
    rows.sort_by_key(|row| at(row));
    let mut kept: Vec<(&str, Vec<&str>)> = Vec::new();
    for suffix in &report.kept {
        match kept.iter_mut().find(|(name, _)| *name == suffix.definition) {
            Some((_, suffixes)) => suffixes.push(&suffix.suffix),
            None => kept.push((&suffix.definition, vec![&suffix.suffix])),
        }
    }
    kept.sort_by_key(|(name, _)| at(name));
    let mut line = format!("wrote {config}: adopted built-in {}", and_joined(&rows));
    for (definition, suffixes) in &kept {
        line.push_str(&format!("; kept {definition} for {}", and_joined(suffixes)));
    }
    writeln!(out, "{line}")?;
    if report.drop_comments && !report.comments.is_empty() {
        let ranges: Vec<String> = report
            .comments
            .iter()
            .map(|comment| adopt_range(config, comment))
            .collect();
        let ranges: Vec<&str> = ranges.iter().map(String::as_str).collect();
        writeln!(
            out,
            "dropped the comments at {} (lines before adoption)",
            and_joined(&ranges)
        )?;
    }
    writeln!(
        out,
        "to get the previous file back: it is in version control, e.g. git restore {config}"
    )?;
    writeln!(
        out,
        "to see this candidate again: restore it and run grund config adopt-builtins"
    )
}

/// §FS-config-adopt-builtins.5.2 item 7: every blocking cause, in the spec's order.
fn adopt_causes(report: &AdoptReport) -> String {
    let mut causes: Vec<String> = Vec::new();
    if report.load_error.is_some() {
        causes.push("the candidate does not load (see the error above)".into());
    }
    if !report.readings.is_empty() {
        causes.push("the readings differ from the plan (inspect the comparison above)".into());
    }
    if !report.records.is_empty() {
        causes.push("the records differ (inspect the comparison above)".into());
    }
    if !report.comments.is_empty() && !report.drop_comments {
        let lost = match report.comments.len() {
            1 => "1 comment would be lost".to_string(),
            n => format!("{n} comments would be lost"),
        };
        causes.push(format!(
            "{lost} (pass --drop-comments to drop exactly the comments listed above)"
        ));
    }
    causes.join("; ")
}

/// The refused run's last line: `not written:` after `--write`, `would not write:` for a preview.
fn adopt_closing(
    out: &mut impl std::io::Write,
    report: &AdoptReport,
    causes: &str,
) -> std::io::Result<()> {
    if report.write {
        writeln!(out, "not written: {causes}; {} is unchanged", report.config)
    } else {
        writeln!(out, "would not write: {causes}")
    }
}

/// `a`, `a and b`, `a, b and c`.
fn and_joined(items: &[&str]) -> String {
    match items {
        [] => String::new(),
        [one] => one.to_string(),
        [init @ .., last] => format!("{} and {last}", init.join(", ")),
    }
}
