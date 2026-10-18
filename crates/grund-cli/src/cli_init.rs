/// §FS-init.1.7, §REQ-backwards-compatibility.2: the one line a run that spells
/// `--docs` prints, naming the replacement and the release that removes the old
/// spelling (§DF-init-templates.2.2). The release is written out in full so the
/// release guard reads it (§FS-distribution.4.2.2).
const DOCS_FLAG_DEPRECATION: &str = "warning: --docs is deprecated and is removed in grund 0.19.0; use --scaffold, which lays the same starter files";

fn command_init(args: &[String]) -> ExitCode {
    let mut path: Option<PathBuf> = None;
    let mut name: Option<String> = None;
    let mut description: Option<String> = None;
    let mut template: Option<String> = None;
    let mut docs = false;
    let mut docs_spelled = false;
    let mut force = false;
    let mut dry_run = false;
    let mut check = false;
    let mut no_vcs = false;
    let mut agent_selection = InitAgentEntrypointSelection::default();
    let mut idx = 0;
    while idx < args.len() {
        match args[idx].as_str() {
            // §FS-init.1: lay the template's starter files as well.
            "--scaffold" => docs = true,
            // §FS-init.1.7: the deprecated spelling of `--scaffold`, warned below.
            "--docs" => (docs, docs_spelled) = (true, true),
            // §FS-init.1.6: the template to lay; core names and refuses it.
            "--template" => {
                idx += 1;
                if idx >= args.len() {
                    eprintln!("error: --template requires a value");
                    return ExitCode::from(2);
                }
                template = Some(args[idx].clone());
            }
            other if other.starts_with("--template=") => {
                template = Some(other.trim_start_matches("--template=").to_string());
            }
            "--force" => force = true,
            "--dry-run" => dry_run = true,
            // §FS-init.1.4: the same preview, taken as a verdict.
            "--check" => check = true,
            "--no-vcs" => no_vcs = true,
            "--agents-md" => agent_selection.canonical = true,
            "--claude" => agent_selection.claude = true,
            "--gemini" => agent_selection.gemini = true,
            "--pi" => agent_selection.pi = true,
            "--copilot" => agent_selection.copilot = true,
            "--cursor" => agent_selection.cursor = true,
            "--windsurf" => agent_selection.windsurf = true,
            "--zed" => agent_selection.zed = true,
            "--name" => {
                idx += 1;
                if idx >= args.len() {
                    eprintln!("error: --name requires a value");
                    return ExitCode::from(2);
                }
                name = Some(args[idx].clone());
            }
            other if other.starts_with("--name=") => {
                name = Some(other.trim_start_matches("--name=").to_string());
            }
            "--description" => {
                idx += 1;
                if idx >= args.len() {
                    eprintln!("error: --description requires a value");
                    return ExitCode::from(2);
                }
                description = Some(args[idx].clone());
            }
            other if other.starts_with("--description=") => {
                description = Some(other.trim_start_matches("--description=").to_string());
            }
            other if other.starts_with('-') => {
                eprintln!("error: unknown flag `{other}`");
                return ExitCode::from(2);
            }
            other => {
                if path.is_some() {
                    eprintln!("error: init takes at most one path argument");
                    return ExitCode::from(2);
                }
                path = Some(PathBuf::from(other));
            }
        }
        idx += 1;
    }
    if docs_spelled {
        eprintln!("{DOCS_FLAG_DEPRECATION}");
    }
    let mut opts = InitOpts::default();
    opts.target = path.unwrap_or_else(|| PathBuf::from("."));
    opts.name = name;
    opts.description = description;
    opts.template = template;
    opts.docs = docs;
    opts.force = force;
    opts.dry_run = dry_run;
    opts.check = check;
    opts.no_vcs = no_vcs;
    opts.agent_selection = agent_selection;
    let output = match init(opts) {
        Ok(output) => output,
        // §FS-init.4.2: `2` wins over `1` — a run that could not be performed
        // produced no report to gate on.
        Err(err) => {
            render_init_output(&err.output);
            eprintln!("error: {err}");
            return ExitCode::from(2);
        }
    };
    render_init_output(&output);
    if output.has_errors() {
        return ExitCode::from(1);
    }
    // §FS-init.4.1: `--check` draws its verdict from the report it just printed —
    // `1` when any reported path was a `would-…`, nothing else. `--dry-run`
    // alone keeps `0` (§REQ-backwards-compatibility.1).
    if check && output.has_pending_changes() {
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}

fn render_init_output(output: &InitOutput) {
    // §FS-check.4.10.7, §FS-workspace.6.1.7: `init` expands the
    // outermost workspace above its target, so it carries the run's warnings
    // ahead of its own report (§FS-distribution.3.1).
    render_run_warnings(&output.warnings);
    for finding in &output.errors {
        match (finding.path.as_deref(), finding.line) {
            (Some(path), Some(line)) => println!(
                "{path}:{line}: {}: {}",
                finding.severity, finding.message
            ),
            _ => println!("{}: {}", finding.severity, finding.message),
        }
    }
    for event in &output.events {
        eprintln!("{} {}", event.verb, event.path);
    }
    // §FS-init.2.3.4.17.4: reported, never fatal — a note names something the run
    // could not do that the caller would otherwise have to notice for itself.
    for note in &output.notes {
        eprintln!("note: {note}");
    }
    if let Some(next) = &output.next {
        render_init_next(next);
    }
}

fn render_init_next(next: &InitNext) {
    // §FS-init.2.2.2: the single process frontend prints the core-rendered
    // decision, so the shipped CLI stays bound to the engine's data.
    eprint!("{}", next.render());
}
