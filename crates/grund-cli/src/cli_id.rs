/// `grund id <KIND> "<title>" [--explain] [--format text|json]`: propose the
/// next conflict-free ID for a new declaration (§FS-id).
fn command_id(args: &[String]) -> ExitCode {
    let mut positional = Vec::new();
    let mut width = 3usize;
    let mut format = "text".to_string();
    let mut explain = false;
    let mut idx = 0;
    while idx < args.len() {
        match args[idx].as_str() {
            "--explain" => explain = true,
            "--width" => {
                idx += 1;
                if idx >= args.len() {
                    eprintln!("error: --width requires a value");
                    return ExitCode::from(2);
                }
                width = match args[idx].parse::<usize>() {
                    Ok(value) => value,
                    Err(_) => {
                        eprintln!("error: --width requires a positive integer");
                        return ExitCode::from(2);
                    }
                };
            }
            other if other.starts_with("--format=") => {
                format = other.trim_start_matches("--format=").to_string();
            }
            "--format" => {
                idx += 1;
                if idx >= args.len() {
                    eprintln!("error: --format requires a value");
                    return ExitCode::from(2);
                }
                format = args[idx].clone();
            }
            other if other.starts_with('-') => {
                eprintln!("error: unknown flag `{other}`");
                return ExitCode::from(2);
            }
            other => positional.push(other.to_string()),
        }
        idx += 1;
    }
    if positional.len() < 2 {
        eprintln!("error: id requires <KIND> and <title>");
        return ExitCode::from(2);
    }
    if positional.len() > 3 {
        eprintln!("error: id takes <KIND>, <title>, and at most one path argument");
        return ExitCode::from(2);
    }
    if !matches!(format.as_str(), "text" | "json") {
        eprintln!("error: unsupported id format `{format}`");
        return ExitCode::from(2);
    }

    let kind = &positional[0];
    let title = &positional[1];
    let path_provided = positional.get(2).is_some();
    let path = positional
        .get(2)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let mut opts = IdOpts::default();
    opts.path = path;
    opts.path_provided = path_provided;
    opts.width = width;
    let (run_warnings, proposed) = propose_id_with_run_warnings(kind, title, opts);
    render_run_warnings(&run_warnings);
    let outcome = match proposed {
        Ok(outcome) => outcome,
        Err(err) => {
            eprintln!("error: {err:#}");
            return ExitCode::from(2);
        }
    };
    match outcome {
        IdProposalOutcome::UnknownKind { headline, known } => {
            eprintln!("error: {headline}");
            // §FS-list.1.1.1: core renders the line, `none` included.
            eprintln!("{}", IdProposalOutcome::known_kinds_line(&known));
            ExitCode::from(2)
        }
        IdProposalOutcome::Rejected { message } => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
        IdProposalOutcome::Proposed(proposal) => {
            print_id_proposal(&proposal, &format, explain);
            ExitCode::SUCCESS
        }
        _ => unnamed_variant::<IdProposalOutcome>(),
    }
}

fn print_id_proposal(proposal: &IdProposal, format: &str, explain: bool) {
    if format == "json" {
        // §FS-id.2.2: a v2 object adds `folders` right after `folder`.
        let folders = proposal.folders.as_ref().map(|folders| {
            let folders: Vec<String> = folders
                .iter()
                .map(|folder| format!("\"{}\"", json_escape(folder)))
                .collect();
            format!(",\"folders\":[{}]", folders.join(","))
        });
        println!(
            "{{\"id\":\"{}\",\"kind\":\"{}\",\"number\":{},\"slug\":\"{}\",\"folder\":\"{}\"{},\"file\":\"{}\"}}",
            json_escape(&proposal.id),
            json_escape(&proposal.kind),
            proposal
                .number
                .map(|number| number.to_string())
                .unwrap_or_else(|| "null".to_string()),
            json_escape(&proposal.slug),
            json_escape(proposal.folder.as_deref().unwrap_or("")),
            folders.unwrap_or_default(),
            json_escape(proposal.file.as_deref().unwrap_or(""))
        );
        return;
    }

    println!("{}", proposal.id);
    if !explain {
        return;
    }
    match proposal.folder.as_deref() {
        Some(folder) if proposal.kind == "E2E" => {
            let case_dir = proposal
                .e2e_case_dir
                .as_deref()
                .unwrap_or(proposal.id.as_str());
            eprintln!(
                "next: create the case directory at {folder}/{case_dir}/ with expected.exit and fixtures, then cite it as §{}",
                proposal.id
            );
        }
        Some(folder) => eprintln!(
            "next: write the declaration at {folder}/{}.md  (H1: `# {}: <one-line statement>`), then cite it as §{}",
            proposal.id, proposal.id, proposal.id
        ),
        None if proposal.file.is_some() => {
            let file = proposal.file.as_deref().unwrap();
            // §FS-id.2.3: an H2 by convention, or the H1 where the file
            // holds the kind's single declaration — which is a fact about
            // that file, not about the kind being named `GRUND`.
            let (heading_name, heading_marker) = if proposal.file_holds_single_declaration {
                ("H1", "#")
            } else {
                ("H2", "##")
            };
            eprintln!(
                "next: add the declaration to {file}  ({heading_name}: `{heading_marker} {}: <one-line statement>`), then cite it as §{}",
                proposal.id, proposal.id
            );
        }
        None => eprintln!(
            "next: write the declaration with H1 `# {}: <one-line statement>`, then cite it as §{}",
            proposal.id, proposal.id
        ),
    }
}
