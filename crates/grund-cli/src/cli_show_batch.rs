#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct BatchInputQuery {
    id: String,
    #[serde(default)]
    section: Option<String>,
}

/// Validate the complete NDJSON stream before entering the workspace loader;
/// empty lines are not records and physical line numbers stay reportable
/// (§FS-show.1.8, §FS-show.2.6.3).
fn read_batch_queries() -> Result<Vec<BatchShowQuery>, String> {
    use std::io::Read;

    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .map_err(|error| format!("read batch input: {error}"))?;
    input
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| {
            serde_json::from_str::<BatchInputQuery>(line)
                .map(|query| BatchShowQuery {
                    id: query.id,
                    section: query.section,
                })
                .map_err(|error| {
                    format!(
                        "batch input line {}: {}",
                        index + 1,
                        concise_batch_input_error(&error.to_string())
                    )
                })
        })
        .collect()
}

fn concise_batch_input_error(message: &str) -> String {
    if let Some(rest) = message.strip_prefix("unknown field `")
        && let Some((field, _)) = rest.split_once('`')
    {
        return format!("unknown field `{field}`");
    }
    message.to_string()
}

/// Emit the batch's records in query order, every per-query outcome on stdout in
/// every format, and map their aggregate verdict (§FS-show.3.3, §FS-cli.5).
fn command_show_batch(
    path: PathBuf,
    path_provided: bool,
    mode: ShowMode,
    format: ShowFormat,
    all: bool,
    globs: &[String],
) -> ExitCode {
    // §FS-show.1.9: a glob generates the batch `--all` would, so it reads no stdin.
    let queries = if all || !globs.is_empty() {
        None
    } else {
        match read_batch_queries() {
            Ok(queries) => Some(queries),
            Err(message) => {
                eprintln!("error: {message}");
                return ExitCode::from(2);
            }
        }
    };
    let mut opts = ShowOpts::default();
    opts.path = path;
    opts.mode = mode;
    opts.format = format;
    let (run_warnings, batch) = show_batch_with_globs(queries, globs, opts, path_provided);
    render_run_warnings(&run_warnings);
    let records = match batch {
        Ok(run) => {
            render_glob_notes(run.note.as_deref());
            run.records
        }
        Err(error) => {
            // §FS-errors.2.2.5: a scan failure prints `check`'s line, the record's own.
            eprintln!("error: {}", ApiScanError::printed(&error));
            return ExitCode::from(2);
        }
    };
    let failed = records.iter().any(|record| record.result.is_err());
    match format {
        ShowFormat::Json => records.into_iter().for_each(print_batch_envelope),
        ShowFormat::Text | ShowFormat::Markdown => print_framed_records(records),
        _ => return unnamed_variant::<ShowFormat>(),
    }
    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// One record as its NDJSON envelope, a failure kept inside it with no hint
/// (§FS-output-shapes.4.1, §FS-output-shapes.6.1.2).
fn print_batch_envelope(record: BatchShowRecord) {
    let section = record
        .query
        .section
        .as_deref()
        .map(|section| format!("\"{}\"", json_escape(section)))
        .unwrap_or_else(|| "null".to_string());
    let query = format!(
        "{{\"id\":\"{}\",\"section\":{section}}}",
        json_escape(&record.query.id)
    );
    match record.result {
        Ok(output) => {
            let result = output.json.unwrap_or_default();
            println!("{{\"query\":{query},\"ok\":true,\"result\":{result},\"error\":null}}")
        }
        // §FS-output-shapes.4.1: `error` is the unchanged finding object
        // of §FS-output-shapes.1, so `authority` closes it here too —
        // present and `null`, no query failure being a rule's.
        Err(error) => println!(
            "{{\"query\":{query},\"ok\":false,\"result\":null,\"error\":{{\"severity\":\"error\",\"path\":null,\"line\":null,\"code\":\"{}\",\"message\":\"{}\",\"sites\":{},\"authority\":null}}}}",
            error.code,
            json_escape(&error.message),
            render_finding_sites_json(&error.sites)
        ),
    }
}

/// The `text` and `md` stream: each record under its `==> <coordinate> <==` header,
/// one empty line before every header but the first (§FS-show.3.3.1). Under a
/// header is the single read's body, or a failure's message and the single read's
/// `hint:` lines, with no `error:` prefix (§FS-show.3.3.2).
fn print_framed_records(records: Vec<BatchShowRecord>) {
    use std::io::Write;

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let mut printed = false;
    let mut ends_in_newline = true;
    for record in records {
        let mut bytes = String::new();
        if printed {
            if !ends_in_newline {
                bytes.push('\n');
            }
            bytes.push('\n');
        }
        bytes.push_str(&format!("==> {} <==\n", record.coordinate));
        match record.result {
            Ok(output) => bytes.push_str(&output.body),
            Err(failure) => {
                bytes.push_str(&failure.message);
                bytes.push('\n');
                for hint in &failure.hints {
                    bytes.push_str(hint);
                    bytes.push('\n');
                }
            }
        }
        ends_in_newline = bytes.ends_with('\n');
        printed = true;
        // A closed pipe ends the stream; the aggregate exit still stands (§FS-cli.5).
        if out.write_all(bytes.as_bytes()).is_err() {
            return;
        }
    }
    let _ = out.flush();
}
