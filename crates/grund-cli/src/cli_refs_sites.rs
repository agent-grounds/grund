/// The `refs` flags that take a value as the next word, which a mode scan skips
/// (§FS-cli.4.1).
const REFS_VALUE_FLAGS: &[&str] = &["--section", "--format", "--glob", "--from"];

/// Whether the line names `--glob` or `--from`, either spelling, anywhere a flag
/// can stand: the positionals are collected before they are read, so the mode is
/// known first (§FS-refs.1.1).
fn refs_site_mode_requested(args: &[String]) -> bool {
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--glob" | "--from" => return true,
            flag if flag.starts_with("--glob=") || flag.starts_with("--from=") => return true,
            flag if REFS_VALUE_FLAGS.contains(&flag) => index += 1,
            _ => {}
        }
        index += 1;
    }
    false
}

/// A `refs` line under `--glob` or `--from`, read whole before anything is refused
/// (§FS-refs.1.2).
#[derive(Default)]
struct RefsSiteLine {
    globs: Vec<String>,
    sources: Vec<String>,
    paths: Vec<String>,
    section: Option<String>,
    descendants: bool,
    summary: bool,
    total: bool,
    format: Option<String>,
}

/// `grund refs --glob <pattern>…` and `grund refs --from <coordinate>`: the sites a
/// pattern's cited coordinates or a point's own body select, in ID mode's shapes
/// (§FS-refs.globs, §FS-refs.from). Every refusal is answered before discovery, a bad
/// `--format` first, then §FS-refs.1.2's table, then a pattern's own.
fn command_refs_sites(line: RefsSiteLine) -> ExitCode {
    if let Err(code) = run_format("refs", line.format.as_deref()) {
        return code;
    }
    if let Some(refusal) = refs_site_refusal(&line) {
        eprintln!("error: {refusal}");
        return ExitCode::from(2);
    }
    if let Err(error) = check_coordinate_globs(&line.globs) {
        eprintln!("error: {error:#}");
        return ExitCode::from(2);
    }
    let path_provided = !line.paths.is_empty();
    let path = PathBuf::from(line.paths.first().map_or(".", String::as_str));
    let source = line.sources.first().cloned();
    let sites = match &source {
        Some(source) => RefsSites::From(source.clone()),
        None => RefsSites::Globs(line.globs.clone()),
    };
    let (warnings, result) = grund_core::refs_sites_with_text_annotations(RefsSitesOpts {
        path: path.clone(),
        path_provided,
        sites,
    });
    render_run_warnings(&warnings);
    let (output, index_entries) = match result {
        Ok(output) => output,
        Err(err) => {
            // §FS-errors.5.2: the typed sites of an ambiguous source, as `show` reads them.
            let sites = err
                .downcast_ref::<ShowQueryError>()
                .map(|carrier| carrier.sites.as_slice())
                .unwrap_or(&[]);
            let refusal = RefsSiteRefusal {
                message: format!("{err:#}"),
                sites,
                path: &path,
                path_provided,
            };
            return refs_sites_refused(refusal, source, line.format);
        }
    };
    let format = match command_output_format("refs", &output.output_format, line.format) {
        Ok(format) => format,
        Err(code) => return code,
    };
    render_glob_notes(output.note.as_deref());
    if line.total {
        // §FS-refs.from.records: what a point cites reads as the source's sentence.
        let clause = if source.is_some() { cites_clause } else { usage_clause };
        render_refs_total(&output.hits, &format, clause);
    } else if line.summary {
        render_refs_summary(&output.hits, output.workspace, &format, &index_entries);
    } else if format == "json" {
        // §FS-refs.3.2: a record names no one queried target, so it carries no kind title.
        for hit in &output.hits {
            println!("{}", render_ref_hit_json(hit, output.workspace, None));
        }
    } else {
        render_refs_text(&output.hits, &index_entries);
    }
    exit_after_scan_errors(&output.scan_errors)
}

/// The first row of §FS-refs.1.2's table a site-mode line falls under, after its
/// `--format`: the flags that cannot stand together, then a second path. Beside one
/// `--glob`, `--section` and `--descendants`, alone or together, offer the pattern that
/// asks for them.
fn refs_site_refusal(line: &RefsSiteLine) -> Option<String> {
    if line.sources.len() > 1 {
        return Some("--from may only appear once".to_string());
    }
    let from = !line.sources.is_empty();
    if from && !line.globs.is_empty() {
        return Some("--glob cannot be combined with --from".to_string());
    }
    let id_flag = match (&line.section, line.descendants) {
        (Some(_), _) => Some("--section"),
        (None, true) => Some("--descendants"),
        (None, false) => None,
    };
    if let Some(flag) = id_flag {
        if from {
            return Some(format!("{flag} cannot be combined with --from"));
        }
        let offer = match line.globs.as_slice() {
            [pattern] => glob_beside_section(pattern, line.section.as_deref(), line.descendants),
            _ => None,
        };
        return Some(match offer {
            Some(offer) => format!("{flag} cannot be combined with --glob; accepted glob: \"{offer}\""),
            None => format!("{flag} cannot be combined with --glob"),
        });
    }
    (line.paths.len() > 1).then(|| "refs takes at most one path argument".to_string())
}

/// A site-mode run that failed: a missing path, with the hint for an ID typed where
/// the path goes (§FS-refs.1.3); a `--from` source refused with the bytes `grund <ID>`
/// prints for it (§FS-refs.from); and anything else a usage error, exit `2`.
fn refs_sites_refused(
    refusal: RefsSiteRefusal,
    source: Option<String>,
    format: Option<String>,
) -> ExitCode {
    let RefsSiteRefusal {
        message,
        sites,
        path,
        path_provided,
    } = refusal;
    if path_provided && message.starts_with("path does not exist:") {
        eprintln!("error: {message}");
        print_refs_path_hint(&path.to_string_lossy(), source.is_some());
        return ExitCode::from(2);
    }
    let Some(source) = source else {
        eprintln!("error: {message}");
        return ExitCode::from(2);
    };
    // §FS-refs.3: refs' own format, `--format` or else the configured one.
    let format = format
        .or_else(|| effective_config(path).ok().map(|config| config.output_format))
        .unwrap_or_else(|| "text".to_string());
    render_show_error(&source, path, false, &format, &message, sites)
}

/// What a failed site-mode run left to render: the error's text, an ambiguous
/// source's sites, and the path the run was given.
struct RefsSiteRefusal<'a> {
    message: String,
    sites: &'a [FindingSite],
    path: &'a Path,
    path_provided: bool,
}

/// §FS-refs.1.3: the missing path reads as an ID, so the line says how to ask for it.
fn print_refs_path_hint(typed: &str, from: bool) {
    if !reads_as_coordinate(typed) {
        return;
    }
    if from {
        eprintln!(
            "hint: with `--from`, `refs` reads `{typed}` as a scan path; to list its citations, use `grund refs {typed}`"
        );
    } else {
        eprintln!(
            "hint: with `--glob`, `refs` reads `{typed}` as a scan path; to add its citations, use `--glob '{typed}{{,.**}}'`"
        );
    }
}
