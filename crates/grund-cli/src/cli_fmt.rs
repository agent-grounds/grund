/// `grund fmt [--check|--write] [--marker]`: normalize citations in bulk —
/// rewrite `$$` triggers to `§` and keep cross-reference links current
/// (§FS-fmt).
fn command_fmt(args: &[String]) -> ExitCode {
    let mut path = PathBuf::from(".");
    let mut path_provided = false;
    let mut write = false;
    let mut check_flag = false;
    let mut marker = false;
    let mut cross_refs = false;
    for arg in args {
        match arg.as_str() {
            "--check" => check_flag = true,
            "--write" => write = true,
            "--marker" => marker = true,
            "--cross-refs" => cross_refs = true,
            other if other.starts_with('-') => {
                eprintln!("error: unknown flag `{other}`");
                return ExitCode::from(2);
            }
            other => {
                if path_provided {
                    eprintln!("error: fmt takes at most one path argument");
                    return ExitCode::from(2);
                }
                path = PathBuf::from(other);
                path_provided = true;
            }
        }
    }
    if write && check_flag {
        eprintln!("error: --check and --write cannot be used together");
        return ExitCode::from(2);
    }
    let mut opts = FmtOpts::default();
    opts.path = path;
    opts.path_provided = path_provided;
    opts.write = write;
    opts.add_marker = marker;
    opts.cross_refs = cross_refs;
    let output = match format_references(opts) {
        Ok(output) => output,
        Err(err) => {
            if let Some(abort) = err.downcast_ref::<FmtScanAbort>() {
                // §FS-fmt.3.4: the engine keeps strict scan failures structured;
                // this frontend owns the prefix on every complete refusal line.
                for error in &abort.scan_errors {
                    eprintln!(
                        "error: nothing was rewritten: {}: {}",
                        error.path, error.message
                    );
                }
            } else if let Some(abort) = err.downcast_ref::<FmtConflictAbort>() {
                // §FS-fmt.3.4: a block's line carries its first line and its finding's message.
                for conflict in &abort.conflicts {
                    let path = conflict.path.as_deref().unwrap_or_default();
                    let line = conflict.line.unwrap_or_default();
                    eprintln!(
                        "error: nothing was rewritten: {path}:{line}: {}",
                        conflict.message
                    );
                }
            } else {
                eprintln!("error: {err:#}");
            }
            return ExitCode::from(2);
        }
    };
    render_run_warnings(&output.warnings);
    // §FS-fmt.2.3.2: a file the walk read and the rewrite would not write through,
    // named once, exit code untouched — the refusal is the intended behavior and
    // not a failure of the run.
    for path in &output.refused_writes {
        eprintln!("warning: {path}: not rewritten: the symlink target is outside the config root");
    }
    if write {
        let mut files = output
            .changes
            .iter()
            .map(|change| change.path.clone())
            .collect::<Vec<_>>();
        files.sort();
        files.dedup();
        // §FS-fmt.3.7: the totals sum what each change says it did.
        let mut counts = grund_core::FmtCounts::default();
        for change in &output.changes {
            counts += change.counts;
        }
        let totals = fmt_write_totals(&counts);
        println!(
            "rewrote {} line{}{}{}",
            output.changes.len(),
            if output.changes.len() == 1 { "" } else { "s" },
            if files.is_empty() { "" } else { ":" },
            if totals.is_empty() { String::new() } else { format!(" {totals}") }
        );
        for path in &files {
            let in_file = || output.changes.iter().filter(|change| &change.path == path);
            // §FS-section-lock.write.report: a lock file created or deleted says so.
            let lock_file = in_file().find_map(|change| change.lock_file);
            let suffix = lock_file.map_or(String::new(), |it| format!(" {}", it.as_str()));
            println!("  {path} ({}){suffix}", in_file().count());
        }
    } else {
        for change in &output.changes {
            println!("{}:{}: {}", change.path, change.line, change.label);
        }
    }
    if !output.scan_errors.is_empty() {
        // §FS-fmt.3.1: `fmt` walks the tree `check` walks and owes the same account of the paths in
        // it that could not be read (§FS-check.2.4). What it did rewrite is real and, under
        // `--write`, already on disk — the `2` says the tree it ran over was not the whole tree.
        for error in &output.scan_errors {
            eprintln!("error: {}: {}", error.path, error.message);
        }
        return ExitCode::from(2);
    }
    if write || output.changes.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// The `--write` summary's totals (§FS-fmt.3.7): one term per kind, in
/// §FS-fmt.3.5.1's label order with the lock last, a zero left out and a one
/// singular, joined by `, `. A class added later takes a unit here too
/// (§FS-fmt.requirements.next-class).
fn fmt_write_totals(counts: &grund_core::FmtCounts) -> String {
    let terms = [
        (counts.triggers_marked, "trigger marked", "triggers marked"),
        (counts.citations_marked, "citation marked", "citations marked"),
        (counts.local_sections_expanded, "local section expanded", "local sections expanded"),
        (counts.shorthands_expanded, "shorthand expanded", "shorthands expanded"),
        (counts.links_written, "link written", "links written"),
        (counts.section_lock_entries, "section-lock entry", "section-lock entries"),
    ];
    let terms = terms.iter().filter(|(count, _, _)| *count > 0);
    let terms = terms.map(|(count, one, many)| format!("{count} {}", if *count == 1 { one } else { many }));
    terms.collect::<Vec<_>>().join(", ")
}
