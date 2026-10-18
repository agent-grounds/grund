/// `grund fetch <ID>`: the sole explicit external integration entry point
/// (§FS-fetch.1, §FS-fetch.7).
fn command_fetch(args: &[String]) -> ExitCode {
    // §FS-cli.3: fetch takes no flags, so a run flag here is an unknown flag, not a miscount.
    if let Some(flag) = args.iter().find(|a| a.starts_with('-')) {
        eprintln!("error: unknown flag `{flag}`");
        return ExitCode::from(2);
    }
    if args.len() != 1 {
        eprintln!("error: fetch requires exactly one <ID>");
        return ExitCode::from(2);
    }
    let (run_warnings, result) = fetch_snapshot_with_run_warnings(&args[0], Path::new("."));
    render_run_warnings(&run_warnings);
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            let (line, code) = fetch_failure_report(err.kind, &err.message);
            eprintln!("{line}");
            ExitCode::from(code)
        }
    }
}

/// How a failed fetch is reported (§FS-fetch.7): a query failure is core's
/// message alone, exit 1, and an operational one `error: <message>`, exit 2.
/// Core words every failure, so a kind this release does not name is reported
/// as an operational one (§AR-bindings.2.1).
fn fetch_failure_report(kind: FetchFailureKind, message: &str) -> (String, u8) {
    match kind {
        FetchFailureKind::Query => (message.to_owned(), 1),
        FetchFailureKind::Operational => (format!("error: {message}"), 2),
        _ => (format!("error: {message}"), 2),
    }
}
