use std::path::PathBuf;
use std::process::ExitCode;

use crate::config::{display_path, kind_prefixes, non_citable_kind_error};
use crate::model::Id;
use crate::model::json_escape;
use crate::scanner::{e2e_case_dir_name, scan_tree_strict};
use crate::workspace::resolve_workspace_config;
use crate::writers::{format_id, slugify_title};

pub(super) fn command_id(args: &[String]) -> ExitCode {
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
    let config = match resolve_workspace_config(&path) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("error: {err:#}");
            return ExitCode::from(2);
        }
    };
    // §FS-id.1: `id` mints an ID, so the kind it is handed has to be one that
    // has IDs. A configured non-citable kind is refused with the reason — it is
    // a place, and there is nothing to allocate in it.
    let kind_config = match config
        .kinds
        .iter()
        .find(|candidate| &candidate.kind == kind)
    {
        Some(kind_config) if kind_config.citable => kind_config,
        other => {
            match other {
                Some(kind_config) => eprintln!("error: {}", non_citable_kind_error(kind_config)),
                None => eprintln!("error: unknown kind `{kind}`"),
            }
            eprintln!("known kinds: {}", kind_prefixes(&config.kinds).join(", "));
            return ExitCode::from(2);
        }
    };
    let slug = slugify_title(title, &config.slug_pattern);
    if slug.is_empty() {
        eprintln!("title produces empty slug after normalization: \"{title}\"");
        return ExitCode::FAILURE;
    }
    let findings = match scan_tree_strict(&config, Some(&path), path_provided) {
        Ok(findings) => findings,
        Err(err) => {
            eprintln!("error: {err:#}");
            return ExitCode::from(2);
        }
    };
    let kind_format = kind_config.effective_format(&config);
    let uses_number = kind_format.contains("{number}");
    let number = if uses_number {
        let max = findings
            .declarations
            .keys()
            .filter(|id| &id.kind == kind)
            .filter_map(|id| id.num)
            .max()
            .unwrap_or(0);
        Some(max + 1)
    } else {
        None
    };
    let id = Id {
        kind: kind.clone(),
        num: number,
        slug: if kind_format.contains("{slug}") {
            Some(slug.clone())
        } else {
            None
        },
    };
    if let Some(decls) = findings.declarations.get(&id)
        && let Some(decl) = decls.first()
    {
        eprintln!(
            "proposed ID `{}` already declared at {}:{}",
            format_id(&id, &config, width),
            display_path(&config, &decl.file),
            decl.line
        );
        return ExitCode::FAILURE;
    }
    let rendered = format_id(&id, &config, width);
    if format == "json" {
        let folder = kind_config.folder.as_deref().unwrap_or("");
        let file = kind_config.file.as_deref().unwrap_or("");
        println!(
            "{{\"id\":\"{}\",\"kind\":\"{}\",\"number\":{},\"slug\":\"{}\",\"folder\":\"{}\",\"file\":\"{}\"}}",
            json_escape(&rendered),
            json_escape(kind),
            number
                .map(|number| number.to_string())
                .unwrap_or_else(|| "null".to_string()),
            json_escape(&slug),
            json_escape(folder),
            json_escape(file)
        );
    } else {
        println!("{rendered}");
        if explain {
            match kind_config.folder.as_deref() {
                Some(folder) if kind == "E2E" => {
                    let case_dir = e2e_case_dir_name(&config, &rendered);
                    eprintln!(
                        "next: create the case directory at {folder}/{case_dir}/ with expected.exit and fixtures, then cite it as §{rendered}"
                    );
                }
                Some(folder) => eprintln!(
                    "next: write the declaration at {folder}/{rendered}.md  (H1: `# {rendered}: <one-line statement>`), then cite it as §{rendered}"
                ),
                None if kind_config.file.is_some() => {
                    let file = kind_config.file.as_deref().unwrap();
                    let (heading_name, heading_marker) = if kind == "GRUND" {
                        ("H1", "#")
                    } else {
                        ("H2", "##")
                    };
                    eprintln!(
                        "next: add the declaration to {file}  ({heading_name}: `{heading_marker} {rendered}: <one-line statement>`), then cite it as §{rendered}"
                    );
                }
                None => eprintln!(
                    "next: write the declaration with H1 `# {rendered}: <one-line statement>`, then cite it as §{rendered}"
                ),
            }
        }
    }
    ExitCode::SUCCESS
}
