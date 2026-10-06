/// Parse and validate the complete `grund check` input, including repeatable
/// exact-code selectors (§FS-check.1.4), then select only after the full scan and
/// before rendering and the exit decision (§FS-check.2.1.2).
fn command_check(args: &[String]) -> ExitCode {
    let mut path = PathBuf::from(".");
    let mut path_provided = false;
    let mut format_override = None;
    let mut require_grounding = false;
    let mut include_suggestions = false;
    let mut full = false;
    let mut rule = None;
    let mut watch = false;
    let mut selection = CheckFindingSelection::default();
    let mut idx = 0;
    while idx < args.len() {
        match args[idx].as_str() {
            // §FS-check.6.4: parsing/validation is shared with one-shot check.
            "--watch" => watch = true,
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
            "--require-grounding" => require_grounding = true,
            "--suggestions" => include_suggestions = true,
            "--rule" => {
                if rule.is_some() {
                    eprintln!("error: --rule may only appear once");
                    return ExitCode::from(2);
                }
                idx += 1;
                if idx >= args.len() {
                    eprintln!("error: --rule requires a sentence");
                    return ExitCode::from(2);
                }
                rule = Some(args[idx].clone());
            }
            other if other.starts_with("--rule=") => {
                if rule.is_some() {
                    eprintln!("error: --rule may only appear once");
                    return ExitCode::from(2);
                }
                rule = Some(other.trim_start_matches("--rule=").to_string());
            }
            other if other.starts_with("--only=") => {
                let value = other
                    .strip_prefix("--only=")
                    .expect("guarded by starts_with");
                if let Err(err) = selection.add_only(value) {
                    eprintln!("error: {err}");
                    return ExitCode::from(2);
                }
            }
            "--only" => {
                idx += 1;
                if idx >= args.len() {
                    eprintln!("error: --only requires a finding code");
                    return ExitCode::from(2);
                }
                if let Err(err) = selection.add_only(&args[idx]) {
                    eprintln!("error: {err}");
                    return ExitCode::from(2);
                }
            }
            other if other.starts_with("--ignore=") => {
                let value = other
                    .strip_prefix("--ignore=")
                    .expect("guarded by starts_with");
                if let Err(err) = selection.add_ignore(value) {
                    eprintln!("error: {err}");
                    return ExitCode::from(2);
                }
            }
            "--ignore" => {
                idx += 1;
                if idx >= args.len() {
                    eprintln!("error: --ignore requires a finding code");
                    return ExitCode::from(2);
                }
                if let Err(err) = selection.add_ignore(&args[idx]) {
                    eprintln!("error: {err}");
                    return ExitCode::from(2);
                }
            }
            // §FS-rules.8: a boolean, so there is no value to validate and
            // repeating it asks the same thing twice.
            "--only-rule" => selection.scope_to_trial_rule(),
            // §FS-check.1.3: widen the walk past `[scan] include` for this run.
            "--full" => full = true,
            other if other.starts_with('-') => {
                eprintln!("error: unknown flag `{other}`");
                return ExitCode::from(2);
            }
            other => {
                if path_provided {
                    eprintln!("error: check takes at most one path argument");
                    return ExitCode::from(2);
                }
                path = PathBuf::from(other);
                path_provided = true;
            }
        }
        idx += 1;
    }
    if let Some(format) = &format_override
        && !matches!(format.as_str(), "text" | "json")
    {
        eprintln!("error: unsupported check format `{format}`");
        return ExitCode::from(2);
    }
    // §FS-rules.8, §FS-check.1.4: scoping to no sentence would print `success`
    // and exit 0, reading as a verdict rather than as the mistake it is — so it
    // refuses here, with the other selector values, before any scan.
    if selection.scopes_to_trial_rule() && rule.is_none() {
        eprintln!("error: --only-rule requires --rule");
        return ExitCode::from(2);
    }
    let opts = CheckOpts {
        path,
        path_provided,
        require_grounding,
        include_suggestions,
        full,
        rule,
    };
    if watch {
        return command_check_watch(opts, selection, format_override);
    }
    let output = prepare_check_run(opts, &selection, format_override.as_deref());
    if let Err(err) = publish_check_run(&output) {
        eprintln!("error: publishing check report: {err}");
        return ExitCode::from(2);
    }
    ExitCode::from(output.status)
}

/// A complete private report, prepared by the one-shot path and published by
/// either frontend mode (§FS-check.6.2, §FS-check.6.3.3).
struct PreparedCheckRun {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    status: u8,
    format: String,
}

fn prepare_check_run(
    opts: CheckOpts,
    selection: &CheckFindingSelection,
    format_override: Option<&str>,
) -> PreparedCheckRun {
    let (run_warnings, output) = check_with_run_warnings(opts);
    let mut prepared = PreparedCheckRun {
        stdout: Vec::new(),
        stderr: Vec::new(),
        status: 2,
        format: format_override.unwrap_or("text").to_string(),
    };
    // §FS-check.4.10.8: render root warnings once, before the report
    // or later refusal. They remain data until this frontend chooses their stream
    // and shape (§FS-distribution.3.1).
    for warning in &run_warnings {
        prepared
            .stderr
            .extend_from_slice(format!("warning: {}\n", warning.message).as_bytes());
    }
    let mut output = match output {
        Ok(output) => output,
        Err(err) => {
            prepared
                .stderr
                .extend_from_slice(format!("error: {err:#}\n").as_bytes());
            return prepared;
        }
    };
    let format = format_override.unwrap_or(&output.output_format);
    prepared.format = format.to_string();
    if !matches!(format, "text" | "json") {
        prepared
            .stderr
            .extend_from_slice(format!("error: unsupported check format `{format}`\n").as_bytes());
        return prepared;
    }
    // §FS-check.2.1.2: the complete API report exists before the CLI applies its
    // presentation query; retained diagnostics then use ordinary rendering.
    output
        .report
        .errors
        .retain(|finding| selection.retains(finding.code, &finding.authority));
    output
        .report
        .warnings
        .retain(|finding| selection.retains(finding.code, &finding.authority));
    output
        .report
        .suggestions
        .retain(|finding| selection.retains(finding.code, &finding.authority));
    if format == "json" {
        render_check_json(&output.report, &mut prepared);
    } else {
        render_check_text(&output.report, run_warnings.len(), &mut prepared);
    }
    prepared.status = if output.had_scan_errors {
        2
    } else if output.report.errors.is_empty() {
        0
    } else {
        1
    };
    prepared
}

/// Both flushes delimit completion (§FS-check.6.3.3). Once this starts, SIGINT
/// cannot discard half a report; the caller checks interruption beforehand.
fn publish_check_run(run: &PreparedCheckRun) -> std::io::Result<()> {
    use std::io::Write;
    let mut stdout = std::io::stdout().lock();
    let mut stderr = std::io::stderr().lock();
    stderr.write_all(&run.stderr)?;
    #[cfg(feature = "test-watch")]
    watch_capture_write(1, &run.stderr);
    stdout.write_all(&run.stdout)?;
    #[cfg(feature = "test-watch")]
    watch_capture_write(0, &run.stdout);
    stdout.flush()?;
    watch_observe!(WatchObservation::StdoutFlushed);
    stderr.flush()?;
    watch_observe!(WatchObservation::StderrFlushed);
    Ok(())
}

/// Compare one format's retained findings by the fixed bytewise key
/// (§FS-errors.4.1); text applies it per channel and JSON applies it globally.
fn finding_cmp(a: &Finding, b: &Finding) -> std::cmp::Ordering {
    (a.path.as_deref(), a.line.unwrap_or(0), a.message.as_str()).cmp(&(
        b.path.as_deref(),
        b.line.unwrap_or(0),
        b.message.as_str(),
    ))
}

fn sorted_text_findings(report: &Report) -> Vec<(&'static str, &Finding)> {
    // §FS-check.2.3.2: suggestions exist only when requested, so chaining is a no-op.
    // §FS-errors.4.1: text has fixed channel groups, sorted within each group.
    let mut errors = report
        .errors
        .iter()
        .map(|finding| ("error", finding))
        .collect::<Vec<_>>();
    let mut warnings = report
        .warnings
        .iter()
        .map(|finding| ("warning", finding))
        .collect::<Vec<_>>();
    let mut suggestions = report
        .suggestions
        .iter()
        .map(|finding| ("suggestion", finding))
        .collect::<Vec<_>>();
    errors.sort_by(|(_, a), (_, b)| finding_cmp(a, b));
    warnings.sort_by(|(_, a), (_, b)| finding_cmp(a, b));
    suggestions.sort_by(|(_, a), (_, b)| finding_cmp(a, b));
    errors.extend(warnings);
    errors.extend(suggestions);
    errors
}

/// Keep JSON in its pre-existing global bytewise location/message order
/// (§FS-check.2.1.4, §FS-errors.4.1), independent of text's severity groups.
fn sorted_json_findings(report: &Report) -> Vec<(&'static str, &Finding)> {
    let mut findings = report
        .warnings
        .iter()
        .map(|finding| ("warning", finding))
        .chain(report.errors.iter().map(|finding| ("error", finding)))
        .chain(
            report
                .suggestions
                .iter()
                .map(|finding| ("suggestion", finding)),
        )
        .collect::<Vec<_>>();
    findings.sort_by(|(_, a), (_, b)| finding_cmp(a, b));
    findings
}

/// `run_warnings` is how many `[workspace]` cautions this run already printed on
/// stderr, before this report existed (§FS-check.4.10.11, §FS-workspace.6.1.7). They are not report findings, so nothing in `report`
/// records them — and a run that says part of its tree is unchecked must not also
/// say `success` (§FS-check.2.1.3).
fn render_check_text(report: &Report, run_warnings: usize, prepared: &mut PreparedCheckRun) {
    // §FS-check.2.3.2: suggestions never suppress `success`, but when present
    // (caller passed --suggestions) they are printed, so the marker only stands
    // in for a run with nothing at all to show.
    if run_warnings == 0
        && report.errors.is_empty()
        && report.warnings.is_empty()
        && report.suggestions.is_empty()
    {
        prepared.stdout.extend_from_slice(b"success\n");
        return;
    }
    for (severity, finding) in sorted_text_findings(report) {
        let line = match (finding.path.as_deref(), finding.line) {
            // §FS-errors.2.1.1: retain the jump-friendly location prefix and
            // place `check`'s structural channel before unchanged message bytes.
            (Some(path), Some(line)) => {
                format!("{path}:{line}: {severity}: {}", finding.message)
            }
            (Some(path), None) => format!("{severity}: {path}: {}", finding.message),
            _ => format!("{severity}: {}", finding.message),
        };
        if finding.line.is_some() {
            prepared
                .stdout
                .extend_from_slice(format!("{line}\n").as_bytes());
        } else {
            prepared
                .stderr
                .extend_from_slice(format!("{line}\n").as_bytes());
        }
    }
}

fn render_check_json(report: &Report, prepared: &mut PreparedCheckRun) {
    for (severity, finding) in sorted_json_findings(report) {
        let object = render_finding_json(severity, finding);
        if finding.line.is_some() {
            prepared
                .stdout
                .extend_from_slice(format!("{object}\n").as_bytes());
        } else {
            prepared
                .stderr
                .extend_from_slice(format!("{object}\n").as_bytes());
        }
    }
}

/// §FS-errors.5.1: the record's last value — `null` where no rule authored the
/// finding, else its origins in the bytewise order the engine already sorted
/// them into, which is the order the message tail joins (§FS-rules.6).
fn authority_json(authority: &[String]) -> String {
    if authority.is_empty() {
        return "null".to_string();
    }
    let origins = authority
        .iter()
        .map(|origin| format!("\"{}\"", json_escape(origin)))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{origins}]")
}

fn render_finding_json(severity: &str, finding: &Finding) -> String {
    let path = finding
        .path
        .as_deref()
        .map(|path| format!("\"{}\"", json_escape(path)))
        .unwrap_or_else(|| "null".to_string());
    let line = finding
        .line
        .map(|line| line.to_string())
        .unwrap_or_else(|| "null".to_string());
    let sites = if finding.sites.is_empty() {
        "null".to_string()
    } else {
        let values = finding
            .sites
            .iter()
            .map(|site| {
                format!(
                    "{{\"path\":\"{}\",\"line\":{}}}",
                    json_escape(&site.path),
                    site.line
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!("[{}]", values)
    };
    // §FS-errors.5.1: a suggestion carries `"channel":"suggestion"` rather than a
    // `"severity"`, so the frozen `{error, warning}` set stays intact.
    let tag = if severity == "suggestion" {
        "\"channel\":\"suggestion\"".to_string()
    } else {
        format!("\"severity\":\"{severity}\"")
    };
    format!(
        "{{{},\"path\":{},\"line\":{},\"code\":\"{}\",\"message\":\"{}\",\"sites\":{},\"authority\":{}}}",
        tag,
        path,
        line,
        finding.code,
        json_escape(&finding.message),
        sites,
        authority_json(&finding.authority)
    )
}
