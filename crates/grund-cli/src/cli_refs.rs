/// `grund refs <ID> [--descendants] [--summary] [--total] [--format text|json]`:
/// every citation of one ID, rendered as `path:line`, with resolver rejection
/// a failed query (§FS-refs.3, §FS-refs.4).
/// `--descendants` widens the section filter to the subtree (§FS-refs.1) and
/// adds no classification of its own, so it is resolved here before rendering
/// exactly as `--section` and `--summary` are (§FS-refs.4). `--total` is the
/// third rung of the same fold and is resolved the same way (§FS-refs.3.4).
/// `--glob` and `--from` put the line in a mode of its own, whose every positional
/// is the path (§FS-refs.1.1), and ID mode reads every other line as before.
fn command_refs(args: &[String]) -> ExitCode {
    if args.is_empty() {
        eprintln!("error: refs requires an ID");
        return ExitCode::from(2);
    }
    let site_mode = refs_site_mode_requested(args);
    let mut site_line = RefsSiteLine::default();
    let mut id_arg = None;
    let mut path = PathBuf::from(".");
    let mut path_provided = false;
    let mut section_override: Option<String> = None;
    let mut descendants = false;
    let mut format_override: Option<String> = None;
    let mut summary = false;
    let mut total = false;
    let mut idx = 0;
    while idx < args.len() {
        match args[idx].as_str() {
            "--summary" => summary = true,
            // §FS-refs.3.4: `--total` is a rung of the fold, not a mode, so it
            // is read here beside `--summary` and wins over it.
            "--total" => total = true,
            "--descendants" => descendants = true,
            "--section" => {
                idx += 1;
                if idx >= args.len() {
                    eprintln!("error: --section requires a value");
                    return ExitCode::from(2);
                }
                section_override = Some(args[idx].clone());
            }
            // §FS-coordinate-globs.errors.flags: repeatable, its value next or after `=`.
            "--glob" => {
                idx += 1;
                let Some(pattern) = args.get(idx) else {
                    eprintln!("error: --glob requires a value");
                    return ExitCode::from(2);
                };
                site_line.globs.push(pattern.clone());
            }
            other if other.starts_with("--glob=") => {
                site_line.globs.push(other.trim_start_matches("--glob=").to_string());
            }
            // §FS-refs.1.2: a missing or empty value is refused where it is read.
            "--from" => {
                idx += 1;
                match args.get(idx) {
                    Some(source) if !source.is_empty() => site_line.sources.push(source.clone()),
                    _ => {
                        eprintln!("error: --from requires a value");
                        return ExitCode::from(2);
                    }
                }
            }
            other if other.starts_with("--from=") => {
                let source = other.trim_start_matches("--from=");
                if source.is_empty() {
                    eprintln!("error: --from requires a value");
                    return ExitCode::from(2);
                }
                site_line.sources.push(source.to_string());
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
            other if site_mode => site_line.paths.push(other.to_string()),
            other if id_arg.is_none() => id_arg = Some(other.to_string()),
            other => {
                if path_provided {
                    eprintln!("error: refs takes an ID and at most one path argument");
                    return ExitCode::from(2);
                }
                path = PathBuf::from(other);
                path_provided = true;
            }
        }
        idx += 1;
    }
    if site_mode {
        site_line.section = section_override;
        site_line.descendants = descendants;
        site_line.summary = summary;
        site_line.total = total;
        site_line.format = format_override;
        return command_refs_sites(site_line);
    }
    let Some(id_arg) = id_arg else {
        eprintln!("error: refs requires an ID");
        return ExitCode::from(2);
    };
    // §FS-cli.3.5: a bad `--format` is answered before discovery, so a broken
    // `grund.toml` cannot shadow it.
    if let Err(code) = run_format("refs", format_override.as_deref()) {
        return code;
    }
    let mut opts = RefsOpts::default();
    opts.path = path;
    opts.path_provided = path_provided;
    opts.id = id_arg;
    opts.section = section_override;
    opts.descendants = descendants;
    // §FS-show.3.5.1: prose travels beside the unchanged public result records.
    let annotated = grund_core::refs_with_text_annotations(opts);
    let (metadata, hint, index_entries, set_aside) = match annotated {
        Ok(output) => output,
        Err(err) => {
            // §FS-refs.4: recorded ambiguities refuse before every successful
            // renderer.
            if let Some(refusal) = err.downcast_ref::<ShowQueryError>()
                && let Some(output) = err.downcast_ref::<RefsOutput>()
            {
                render_run_warnings(&output.warnings);
                let format =
                    match command_output_format("refs", &output.output_format, format_override) {
                        Ok(format) => format,
                        Err(code) => return code,
                    };
                if !output.scan_errors.is_empty() {
                    return exit_after_scan_errors(&output.scan_errors);
                }
                // §FS-errors.5.2.1: use the shared typed sites, not a prose parse.
                if format == "json" {
                    print_bare_query_json(refusal.code, &refusal.message, &refusal.sites);
                } else {
                    eprintln!("{}", refusal.message);
                }
                return ExitCode::from(1);
            }
            eprintln!("error: {err:#}");
            return ExitCode::from(2);
        }
    };
    let outcome = &metadata.outcome;
    let output = &outcome.output;
    render_run_warnings(&output.warnings);
    let format = match command_output_format("refs", &output.output_format, format_override) {
        Ok(format) => format,
        Err(code) => return code,
    };
    // §FS-refs.4.1.1: each broken stub set aside beside a real home is named, in
    // text only; JSON keeps stderr empty and leaves the finding to `check`.
    if format != "json" {
        for warning in &set_aside {
            eprintln!("warning: {warning}");
        }
    }
    if let Some(failure) = &outcome.query_failure {
        if !output.scan_errors.is_empty() {
            return exit_after_scan_errors(&output.scan_errors);
        }
        return render_refs_query_failure(failure, &format, hint.as_deref());
    }
    if let Some(note) = &output.note {
        eprintln!("note: {note}");
    }
    if total {
        render_refs_total(&output.hits, &format, usage_clause);
    } else if summary {
        render_refs_summary(&output.hits, output.workspace, &format, &index_entries);
    } else if format == "json" {
        for hit in &output.hits {
            println!(
                "{}",
                render_ref_hit_json(hit, output.workspace, metadata.kind_title.as_deref())
            );
        }
    } else {
        render_refs_text(&output.hits, &index_entries);
    }

    exit_after_scan_errors(&output.scan_errors)
}

/// Render a typed resolver rejection as the failed query `show` prints for the
/// same operand, exit `1` (§FS-refs.4, §FS-errors.5.2). The format and
/// summary/section/total flags are already resolved, so no renderer can
/// reclassify the operand.
fn render_refs_query_failure(failure: &RefsQueryFailure, format: &str, hint: Option<&str>) -> ExitCode {
    if format == "json" {
        print_bare_query_json(failure.kind.code(), &failure.message, &[]);
    } else {
        eprintln!("{}", failure.message);
        // §FS-show.3.5.1: the core's selected-project guidance owns this sentence.
        if let Some(hint) = hint {
            eprintln!("{hint}");
        }
    }
    ExitCode::from(1)
}

/// The citation set's size instead of its members (§FS-refs.3.4): the hover's
/// own clause in text, `not cited` at zero included — or, under `--from`, the
/// source's own (§FS-refs.from.records) — and exactly one object under
/// `--format json` at every count, with no `project` member even in a
/// workspace, whose paths are already root-relative and distinct
/// (§FS-workspace.8.2).
fn render_refs_total(hits: &[RefHit], format: &str, clause: fn(grund_core::LspUsage) -> String) {
    // §FS-lsp.1.2.5: the engine's fold and the engine's wording, never a second
    // tally or a second phrasing kept beside them.
    let usage = usage_over_paths(hits.iter().map(|hit| hit.path.as_str()));
    if format == "json" {
        println!("{{\"sites\":{},\"files\":{}}}", usage.sites, usage.files);
    } else {
        println!("{}", clause(usage));
    }
}

/// §FS-refs.3.3.1: append excluded sites to the complete existing summary row.
fn render_refs_summary(
    hits: &[RefHit],
    workspace: bool,
    format: &str,
    index_entries: &BTreeSet<(String, usize, usize)>,
) {
    let annotations = if format == "json" {
        BTreeMap::new()
    } else {
        refs_summary_annotations(hits, index_entries)
    };
    let mut by_file: BTreeMap<String, (Option<String>, usize, BTreeSet<usize>)> = BTreeMap::new();
    for hit in hits {
        let entry = by_file
            .entry(hit.path.clone())
            .or_insert_with(|| (hit.project.clone(), 0, BTreeSet::new()));
        entry.1 += 1;
        entry.2.insert(hit.line);
    }
    for (path, (project, count, lines)) in by_file {
        if format == "json" {
            let lines_json = lines
                .iter()
                .map(|line| line.to_string())
                .collect::<Vec<_>>()
                .join(",");
            if workspace {
                println!(
                    "{{\"project\":\"{}\",\"path\":\"{}\",\"count\":{},\"lines\":[{}]}}",
                    json_escape(project.as_deref().unwrap_or("")),
                    json_escape(&path),
                    count,
                    lines_json
                );
            } else {
                println!(
                    "{{\"path\":\"{}\",\"count\":{},\"lines\":[{}]}}",
                    json_escape(&path),
                    count,
                    lines_json
                );
            }
        } else {
            let label = if lines.len() == 1 { "line" } else { "lines" };
            let lines_text = lines
                .iter()
                .map(|line| line.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            let annotation = annotations.get(path.as_str()).map_or("", String::as_str);
            println!("{path}: {count} ({label} {lines_text}){annotation}");
        }
    }
}

fn render_ref_hit_json(hit: &RefHit, workspace: bool, kind_title: Option<&str>) -> String {
    let project_field = if workspace {
        format!(
            "\"project\":\"{}\",",
            json_escape(hit.project.as_deref().unwrap_or(""))
        )
    } else {
        String::new()
    };
    // §FS-refs.3.2: append target metadata only to detailed records.
    let metadata = kind_title
        .map(|title| format!(",\"kind_title\":\"{}\"", json_escape(title)))
        .unwrap_or_default();
    format!(
        "{{{}\"path\":\"{}\",\"line\":{},\"column\":{},\"id\":\"{}\",\"section\":{},\"marker\":{},\"text\":\"{}\",\"enclosing_declaration\":{},\"enclosing_section\":{}{}}}",
        project_field,
        json_escape(&hit.path),
        hit.line,
        hit.column,
        json_escape(&hit.id),
        json_string_or_null(hit.section.as_deref()),
        hit.marker,
        json_escape(&hit.text),
        // §FS-refs.3.2: the citing site's unit between `text` and `kind_title`,
        // which stays the record's final field (§FS-refs.3.2.1).
        json_string_or_null(hit.enclosing_declaration.as_deref()),
        json_string_or_null(hit.enclosing_section.as_deref()),
        metadata
    )
}

fn add_kind_filters(filters: &mut BTreeSet<String>, raw: &str) {
    for value in raw.split(',') {
        filters.insert(value.to_string());
    }
}

fn add_project_filters(filters: &mut BTreeSet<String>, raw: &str) {
    for value in raw.split(',') {
        if !value.is_empty() {
            filters.insert(value.to_string());
        }
    }
}
