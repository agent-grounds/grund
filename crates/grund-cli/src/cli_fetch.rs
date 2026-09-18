/// `grund fetch <ID>`: the sole explicit external integration entry point
/// (§FS-fetch.1, §FS-fetch.7).
fn command_fetch(args: &[String]) -> ExitCode {
    if args.len() != 1 {
        eprintln!("error: fetch requires exactly one <ID>");
        return ExitCode::from(2);
    }
    let (run_warnings, result) = fetch_snapshot_with_run_warnings(&args[0], Path::new("."));
    render_run_warnings(&run_warnings);
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => match err.kind {
            FetchFailureKind::Query => {
                eprintln!("{}", err.message);
                ExitCode::FAILURE
            }
            FetchFailureKind::Operational => {
                eprintln!("error: {}", err.message);
                ExitCode::from(2)
            }
        },
    }
}
