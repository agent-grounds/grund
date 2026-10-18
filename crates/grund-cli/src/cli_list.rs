/// `grund list [PATH] [--selector S] [--glob PATTERN]... [--kind K,...] [--unused]
/// [--summary | --total] [--size[=lines,words,bytes]] [--top N] [--format text|json]`:
/// the ID catalog — every declared ID with its `path:line` and title
/// (§FS-list). The complement of `grund refs`.
fn command_list(args: &[String]) -> ExitCode {
    let mut path = PathBuf::from(".");
    let mut path_provided = false;
    let mut kind_filter: BTreeSet<String> = BTreeSet::new();
    let mut project_filter: BTreeSet<String> = BTreeSet::new();
    let mut unused_only = false;
    let mut summary = false;
    let mut size_units: Option<Vec<PointSizeUnit>> = None;
    let mut top: Option<usize> = None;
    let mut top_seen = false;
    let mut format_override: Option<String> = None;
    let mut selector = None;
    let mut globs = Vec::new();
    let mut total = false;
    let mut idx = 0;
    while idx < args.len() {
        match args[idx].as_str() {
            "--unused" => unused_only = true,
            "--summary" => summary = true,
            // §FS-list.total: repeating it is harmless.
            "--total" => total = true,
            // §FS-coordinate-globs.errors.flags: repeatable, its value next or after `=`.
            "--glob" => {
                idx += 1;
                let Some(pattern) = args.get(idx) else {
                    eprintln!("error: --glob requires a value");
                    return ExitCode::from(2);
                };
                globs.push(pattern.clone());
            }
            other if other.starts_with("--glob=") => {
                globs.push(other.trim_start_matches("--glob=").to_string());
            }
            "--selector" => {
                if selector.is_some() {
                    eprintln!("error: --selector may only appear once");
                    return ExitCode::from(2);
                }
                idx += 1;
                if idx >= args.len() {
                    eprintln!("error: --selector requires a value");
                    return ExitCode::from(2);
                }
                selector = Some(args[idx].clone());
            }
            other if other.starts_with("--selector=") => {
                if selector.is_some() {
                    eprintln!("error: --selector may only appear once");
                    return ExitCode::from(2);
                }
                selector = Some(other.trim_start_matches("--selector=").to_string());
            }
            "--size" => {
                if size_units.is_some() {
                    eprintln!("error: --size may only appear once");
                    return ExitCode::from(2);
                }
                size_units = Some(vec![
                    PointSizeUnit::Lines,
                    PointSizeUnit::Words,
                    PointSizeUnit::Bytes,
                ]);
            }
            other if other.starts_with("--size=") => {
                if size_units.is_some() {
                    eprintln!("error: --size may only appear once");
                    return ExitCode::from(2);
                }
                let raw = other.strip_prefix("--size=").expect("guarded size flag");
                let mut units = Vec::new();
                for name in raw.split(',') {
                    if name.is_empty() {
                        eprintln!("error: --size requires one or more units");
                        return ExitCode::from(2);
                    }
                    let Some(unit) = PointSizeUnit::parse(name) else {
                        eprintln!(
                            "error: unknown size unit `{name}` (expected lines, words, or bytes)"
                        );
                        return ExitCode::from(2);
                    };
                    if !units.contains(&unit) {
                        units.push(unit);
                    }
                }
                size_units = Some(units);
            }
            other if other.starts_with("--top=") => {
                if top_seen {
                    eprintln!("error: --top may only appear once");
                    return ExitCode::from(2);
                }
                top_seen = true;
                let raw = other.strip_prefix("--top=").expect("guarded top flag");
                top = match raw.parse::<usize>() {
                    Ok(value) if value > 0 => Some(value),
                    _ => {
                        eprintln!("error: --top requires a positive integer");
                        return ExitCode::from(2);
                    }
                };
            }
            "--top" => {
                if top_seen {
                    eprintln!("error: --top may only appear once");
                    return ExitCode::from(2);
                }
                top_seen = true;
                idx += 1;
                top = match args.get(idx).and_then(|raw| raw.parse::<usize>().ok()) {
                    Some(value) if value > 0 => Some(value),
                    _ => {
                        eprintln!("error: --top requires a positive integer");
                        return ExitCode::from(2);
                    }
                };
            }
            "--kind" => {
                idx += 1;
                if idx >= args.len() {
                    eprintln!("error: --kind requires a value");
                    return ExitCode::from(2);
                }
                add_kind_filters(&mut kind_filter, &args[idx]);
            }
            other if other.starts_with("--kind=") => {
                add_kind_filters(&mut kind_filter, other.trim_start_matches("--kind="));
            }
            "--project" => {
                idx += 1;
                if idx >= args.len() {
                    eprintln!("error: --project requires a value");
                    return ExitCode::from(2);
                }
                add_project_filters(&mut project_filter, &args[idx]);
            }
            other if other.starts_with("--project=") => {
                add_project_filters(&mut project_filter, other.trim_start_matches("--project="));
            }
            other if other.starts_with("--format=") => {
                format_override = Some(other.trim_start_matches("--format=").to_string());
            }
            "--format" => {
                idx += 1;
                if idx >= args.len() {
                    eprintln!("error: --format requires a value");
                    return ExitCode::from(2);
                }
                format_override = Some(args[idx].clone());
            }
            other if other.starts_with('-') => {
                eprintln!("error: unknown flag `{other}`");
                return ExitCode::from(2);
            }
            other => {
                if path_provided {
                    eprintln!("error: list takes at most one path argument");
                    return ExitCode::from(2);
                }
                path = PathBuf::from(other);
                path_provided = true;
            }
        }
        idx += 1;
    }
    // §FS-cli.3.5: a bad `--format` is answered before discovery, so a broken
    // `grund.toml` cannot shadow it.
    if let Err(code) = run_format("list", format_override.as_deref()) {
        return code;
    }
    // §FS-list.total: a count beside a fold or a measurement is refused first, in this order.
    for (combined, flag) in [
        (summary, "--summary"),
        (size_units.is_some(), "--size"),
        (top_seen, "--top"),
    ] {
        if total && combined {
            eprintln!("error: --total cannot be combined with {flag}");
            return ExitCode::from(2);
        }
    }
    // §FS-list.1.7: size-selector combinations are launch errors decided before
    // config discovery or scanning.
    if size_units.is_some() && summary {
        eprintln!("error: --size cannot be combined with --summary");
        return ExitCode::from(2);
    }
    if top_seen && summary {
        eprintln!("error: --top cannot be combined with --summary");
        return ExitCode::from(2);
    }
    if top_seen && size_units.is_none() {
        eprintln!("error: --top requires --size");
        return ExitCode::from(2);
    }

    if let Some(units) = size_units {
        let mut opts = ListSizeOpts::default();
        opts.path = path.clone();
        opts.path_provided = path_provided;
        opts.kind_filter = kind_filter;
        opts.project_filter = project_filter;
        opts.unused_only = unused_only;
        opts.selector = selector;
        opts.globs = globs;
        opts.units = units;
        opts.top = top;
        let output = match list_sizes(opts) {
            Ok(output) => output,
            Err(err) => return list_refused(&err, &path, path_provided),
        };
        render_run_warnings(&output.warnings);
        render_glob_notes(output.note.as_deref());
        let format = match command_output_format("list", &output.output_format, format_override) {
            Ok(format) => format,
            Err(code) => return code,
        };
        if format == "json" {
            for entry in &output.entries {
                println!("{}", render_list_size_entry_json(entry));
            }
        } else {
            render_list_size_text(&output.entries);
        }
        return exit_after_scan_errors(&output.scan_errors);
    }

    // §FS-check.4.10.7, §FS-workspace.6.1.7: the run's warnings come
    // back beside the catalog, so a refused `--project` or `--kind` still prints
    // the cautions the workspace pass settled before it (§FS-distribution.3.1).
    let mut opts = ListOpts::default();
    opts.path = path.clone();
    opts.path_provided = path_provided;
    opts.kind_filter = kind_filter;
    opts.project_filter = project_filter;
    opts.unused_only = unused_only;
    opts.selector = selector;
    opts.globs = globs;
    let (run_warnings, listed) = list_with_run_warnings(opts);
    render_run_warnings(&run_warnings);
    let output = match listed {
        Ok(output) => output,
        Err(err) => return list_refused(&err, &path, path_provided),
    };
    render_glob_notes(output.note.as_deref());
    let format = match command_output_format("list", &output.output_format, format_override) {
        Ok(format) => format,
        Err(code) => return code,
    };

    if total {
        // §FS-list.total: the count of the rows the same filters list, zero included.
        let count = output.entries.len();
        if format == "json" {
            println!("{}", serde_json::json!({ "count": count }));
        } else {
            println!("{count}");
        }
    } else if summary {
        render_list_summary(&output.summaries, output.workspace, &format);
    } else if format == "json" {
        for entry in &output.entries {
            println!("{}", render_list_entry_json(entry));
        }
    } else {
        render_list_text(&output.entries);
    }

    exit_after_scan_errors(&output.scan_errors)
}

/// A refused `list`: its error line, and the hint for a scan path that was more
/// likely a coordinate glob (§FS-coordinate-globs.errors.hint).
fn list_refused(err: &impl std::fmt::Display, path: &Path, path_provided: bool) -> ExitCode {
    let message = format!("{err:#}");
    eprintln!("error: {message}");
    let typed = path.to_string_lossy();
    if path_provided
        && message.starts_with("path does not exist:")
        && reads_as_coordinate_glob(&typed)
    {
        eprintln!(
            "hint: `list` reads `{typed}` as a scan path; for a coordinate pattern, use `grund list --glob '{typed}'`"
        );
    }
    ExitCode::from(2)
}

/// One `note:` line per coordinate glob no kind in scope can start
/// (§FS-coordinate-globs.errors.note).
fn render_glob_notes(note: Option<&str>) {
    for line in note.into_iter().flat_map(str::lines) {
        eprintln!("note: {line}");
    }
}
