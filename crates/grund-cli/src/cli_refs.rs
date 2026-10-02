/// `grund refs <ID> [--descendants] [--summary] [--total] [--format text|json]`:
/// every citation of one ID, rendered as `path:line`, with resolver rejection
/// following the staged query-failure contract (§FS-refs.3, §FS-refs.4).
/// `--descendants` widens the section filter to the subtree (§FS-refs.1) and
/// adds no classification of its own, so it is resolved here before rendering
/// exactly as `--section` and `--summary` are (§FS-refs.4). `--total` is the
/// third rung of the same fold and is resolved the same way (§FS-refs.3.4).
fn command_refs(args: &[String]) -> ExitCode {
    if args.is_empty() {
        eprintln!("error: refs requires an ID");
        return ExitCode::from(2);
    }
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
    let Some(id_arg) = id_arg else {
        eprintln!("error: refs requires an ID");
        return ExitCode::from(2);
    };
    let metadata = match refs_with_metadata(RefsOpts {
        path,
        path_provided,
        id: id_arg,
        section: section_override,
        descendants,
    }) {
        Ok(output) => output,
        Err(err) => {
            // §FS-refs.4: recorded ambiguities refuse now, independently of
            // the grammar-resolver ramp, before every successful renderer.
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
    if let Some(failure) = &outcome.query_failure {
        if !output.scan_errors.is_empty() {
            return exit_after_scan_errors(&output.scan_errors);
        }
        return render_refs_query_failure(failure, &format);
    }
    if let Some(note) = &output.note {
        eprintln!("note: {note}");
    }
    if total {
        render_refs_total(&output.hits, &format);
    } else if summary {
        render_refs_summary(&output.hits, output.workspace, &format);
    } else if format == "json" {
        for hit in &output.hits {
            println!(
                "{}",
                render_ref_hit_json(hit, output.workspace, metadata.kind_title.as_deref())
            );
        }
    } else {
        for hit in &output.hits {
            println!("{}:{}: {}", hit.path, hit.line, hit.text);
        }
    }

    exit_after_scan_errors(&output.scan_errors)
}

/// Render the release-selected wire form of a typed resolver rejection
/// (§FS-refs.4, §FS-errors.5.2). The format and summary/section flags are already
/// resolved, so no renderer can reclassify the operand.
fn render_refs_query_failure(failure: &RefsQueryFailure, format: &str) -> ExitCode {
    if refs_query_failure_is_exit_one() {
        if format == "json" {
            print_bare_query_json(failure.kind.code(), &failure.message, &[]);
        } else {
            eprintln!("{}", failure.message);
            print_refs_query_failure_hint(failure);
        }
        ExitCode::from(1)
    } else {
        eprintln!("error: {}", failure.message);
        print_refs_query_failure_hint(failure);
        eprintln!("{REFS_QUERY_FAILURE_WARNING}");
        ExitCode::from(2)
    }
}

fn print_refs_query_failure_hint(failure: &RefsQueryFailure) {
    if let Some(format) = &failure.format_hint {
        eprintln!(
            "hint: this repo's [id] format is `{format}` (run `grund config show`); `grund list` shows the IDs that exist"
        );
    }
}

/// The citation set's size instead of its members (§FS-refs.3.4): the hover's
/// own clause in text, `not cited` at zero included, and exactly one object
/// under `--format json` at every count — with no `project` member even in a
/// workspace, whose paths are already root-relative and distinct
/// (§FS-workspace.8.2).
fn render_refs_total(hits: &[RefHit], format: &str) {
    // §FS-lsp.1.2.5: the engine's fold and the engine's wording, never a second
    // tally or a second phrasing kept beside them.
    let usage = usage_over_paths(hits.iter().map(|hit| hit.path.as_str()));
    if format == "json" {
        println!("{{\"sites\":{},\"files\":{}}}", usage.sites, usage.files);
    } else {
        println!("{}", usage_clause(usage));
    }
}

fn render_refs_summary(hits: &[RefHit], workspace: bool, format: &str) {
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
            println!("{path}: {count} ({label} {lines_text})");
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
