/// `grund cover [PATH] [--format text|json]`: the citation graph grouped by
/// scanned file, for finding what is under- or un-cited (§FS-cover). With
/// `--lines`, which declaration and section own those lines of one file instead
/// (§FS-cover.6).
///
/// Why `--format` is validated before the scan: a bad value is a usage error the
/// caller can fix without touching the repository, while the scan itself can fail
/// first, on a workspace whose members will not expand.
fn command_cover(args: &[String]) -> ExitCode {
    let mut path = PathBuf::from(".");
    let mut path_provided = false;
    let mut format_override: Option<String> = None;
    let mut lines: Vec<String> = Vec::new();
    let mut idx = 0;
    while idx < args.len() {
        match args[idx].as_str() {
            // §FS-cover.6.1: repeatable, each value kept as written so a refusal
            // can echo it (§FS-cover.6.4).
            other if other.starts_with("--lines=") => {
                lines.push(other.trim_start_matches("--lines=").to_string());
            }
            "--lines" => {
                idx += 1;
                if idx >= args.len() {
                    eprintln!("error: --lines requires a value");
                    return ExitCode::from(2);
                }
                lines.push(args[idx].clone());
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
                    eprintln!("error: cover takes at most one path argument");
                    return ExitCode::from(2);
                }
                path = PathBuf::from(other);
                path_provided = true;
            }
        }
        idx += 1;
    }
    // §FS-cover.1.1: a bad `--format` value is a usage error, answered before the scan
    // so that which of two errors a caller sees does not depend on the tree they
    // happened to point at — the scan can now fail first (§FS-cover.4, §FS-cli.3.5).
    if let Err(code) = run_format("cover", format_override.as_deref()) {
        return code;
    }
    if !lines.is_empty() {
        return command_cover_lines(
            CoverLinesOpts {
                path,
                path_provided,
                lines,
            },
            format_override,
        );
    }
    let opts = CoverOpts {
        path,
        path_provided,
    };
    // One load, whichever view is rendered: `cover` and `cover_text` build the
    // same index (§FS-workspace.8.6.2), and calling both walked every project in
    // the workspace twice.
    let output = match cover(opts) {
        Ok(output) => output,
        Err(err) => {
            eprintln!("error: {err:#}");
            return ExitCode::from(2);
        }
    };
    render_run_warnings(&output.warnings);
    let format = match command_output_format("cover", &output.output_format, format_override) {
        Ok(format) => format,
        Err(code) => return code,
    };
    if format == "json" {
        render_cover_json(&output.entries);
    } else {
        render_cover_text(&output.entries);
    }
    exit_after_scan_errors(&output.scan_errors)
}

fn render_cover_json(entries: &[grund_core::CoverEntry]) {
    for entry in entries {
        let citation_json = entry
            .citations
            .iter()
            .map(render_cover_citation_json)
            .collect::<Vec<_>>()
            .join(",");
        println!(
            "{{{}\"path\":\"{}\",\"citations\":[{}]}}",
            cover_project_field(entry.project.as_deref()),
            json_escape(&entry.path),
            citation_json
        );
    }
}

/// §FS-cover.3.2: the leading `"project":"<alias>"` a workspace run adds, and
/// nothing at all outside one — a single-project repo's JSON keeps the bytes it
/// had (§DF-cover-workspace-scope.2.3).
fn cover_project_field(alias: Option<&str>) -> String {
    alias
        .map(|alias| format!("\"project\":\"{}\",", json_escape(alias)))
        .unwrap_or_default()
}

/// §FS-cover.3.1: the human view prints the file, then each citation's
/// `line:column` and verbatim token — the alias is already in the token and the
/// path already renders from the workspace root, so no field of the JSON view is
/// missing here.
fn render_cover_text(entries: &[grund_core::CoverEntry]) {
    for entry in entries {
        println!("{}:", entry.path);
        if entry.citations.is_empty() {
            println!("  (no citations)");
        } else {
            for citation in &entry.citations {
                println!("  {}:{} {}", citation.line, citation.column, citation.text);
            }
        }
    }
}

fn render_cover_citation_json(citation: &CoverCitation) -> String {
    format!(
        "{{{}\"path\":\"{}\",\"line\":{},\"column\":{},\"id\":\"{}\",\"section\":{},\"marker\":{},\"text\":\"{}\",\"enclosing_declaration\":{},\"enclosing_section\":{}}}",
        cover_project_field(citation.project.as_deref()),
        json_escape(&citation.path),
        citation.line,
        citation.column,
        json_escape(&citation.id),
        json_string_or_null(citation.section.as_deref()),
        citation.marker,
        json_escape(&citation.text),
        // §FS-cover.3.2: the citing site's unit, after `text`; the field-order
        // parity with `refs` is of the fields the two share.
        json_string_or_null(citation.enclosing_declaration.as_deref()),
        json_string_or_null(citation.enclosing_section.as_deref())
    )
}

/// `grund cover <file> --lines <range>…`: the owners of each range, one record
/// per range in the order given; the per-file record of §FS-cover.3 is never
/// mixed in (§FS-cover.6.1). Every refusal of the request is the engine's, so the
/// binding refuses it with the same words (§FS-cover.6.4).
fn command_cover_lines(opts: CoverLinesOpts, format_override: Option<String>) -> ExitCode {
    let output = match cover_lines(opts) {
        Ok(output) => output,
        Err(err) => {
            eprintln!("error: {err:#}");
            return ExitCode::from(2);
        }
    };
    render_run_warnings(&output.warnings);
    let format = match command_output_format("cover", &output.output_format, format_override) {
        Ok(format) => format,
        Err(code) => return code,
    };
    for record in &output.records {
        if format == "json" {
            println!("{}", render_cover_lines_json(record));
        } else {
            render_cover_lines_text(record);
        }
    }
    exit_after_scan_errors(&output.scan_errors)
}

/// §FS-output-shapes.5.3: `path`, `start`, `end`, `owners`, each owner
/// `declaration`, `start`, `end`, `sections`, each section `section`, `start`,
/// `end` — in that order, with the leading `project` a workspace record carries.
fn render_cover_lines_json(record: &grund_core::CoverLines) -> String {
    let owners = record
        .owners
        .iter()
        .map(|owner| {
            let sections = owner
                .sections
                .iter()
                .map(|section| {
                    format!(
                        "{{\"section\":{},\"start\":{},\"end\":{}}}",
                        json_string_or_null(section.section.as_deref()),
                        section.start,
                        section.end
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "{{\"declaration\":\"{}\",\"start\":{},\"end\":{},\"sections\":[{}]}}",
                json_escape(&owner.declaration),
                owner.start,
                owner.end,
                sections
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{{}\"path\":\"{}\",\"start\":{},\"end\":{},\"owners\":[{}]}}",
        cover_project_field(record.project.as_deref()),
        json_escape(&record.path),
        record.start,
        record.end,
        owners
    )
}

/// §FS-cover.6.3: a `<path>:<range>` heading, then one row per run in line order
/// — the declaration, the declaration and its section, or `(no owner)` for the
/// lines between owner runs.
fn render_cover_lines_text(record: &grund_core::CoverLines) {
    let span = |start: usize, end: usize| {
        if start == end {
            start.to_string()
        } else {
            format!("{start}-{end}")
        }
    };
    println!("{}:{}", record.path, span(record.start, record.end));
    let mut next = record.start;
    for owner in &record.owners {
        if owner.start > next {
            println!("  {}  (no owner)", span(next, owner.start - 1));
        }
        for section in &owner.sections {
            let unit = match &section.section {
                Some(path) => format!("{}.{path}", owner.declaration),
                None => owner.declaration.clone(),
            };
            println!("  {}  {unit}", span(section.start, section.end));
        }
        next = owner.end + 1;
    }
    if next <= record.end {
        println!("  {}  (no owner)", span(next, record.end));
    }
}
