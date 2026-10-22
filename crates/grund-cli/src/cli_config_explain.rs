/// `grund config explain [TARGET] [PATH] [--path PATH] [--codes | --all]`
/// (§FS-config-explain.1): the usage is answered before anything is loaded
/// (§FS-cli.requirements.before-load), the engine returns one typed answer, and
/// this prints it — on stdout with exit 0, or as the bare query failure of
/// §FS-errors.2.3 with exit 1 where no version that answers knows the target
/// (§FS-config-explain.1.3).
fn command_config_explain(args: &[String]) -> ExitCode {
    let (ask, path) = match explain_usage(args) {
        Ok(usage) => usage,
        Err(message) => {
            eprintln!("error: {message}");
            return ExitCode::from(2);
        }
    };
    let explained = match config_explain(path.as_deref().unwrap_or(Path::new(".")), &ask) {
        Ok(explained) => explained,
        Err(err) => {
            eprintln!("error: {err:#}");
            return ExitCode::from(2);
        }
    };
    // §FS-errors.4: the path is repeated as core spells it, with `/` on every OS.
    let given = path.is_some().then_some(explained.path.as_str());
    let next = |target: &str| explain_call(target, given);
    if explained.found_nothing() {
        for (version, answer) in &explained.answers {
            if let ExplainAnswer::Unknown(unknown) = answer {
                eprintln!("{}", explain_unknown(*version, unknown));
            }
        }
        return ExitCode::FAILURE;
    }
    match &explained.file {
        Some(file) => {
            let (version, answer) = &explained.answers[0];
            println!("{file} is read as version {version}.");
            println!();
            print_explain_answer(*version, answer, &next);
        }
        // §FS-config-explain.2: both versions, version 2 first.
        None => {
            println!(
                "No grund.toml here. grund init writes version 2; until a file exists, check reads version 1's defaults."
            );
            for (version, answer) in &explained.answers {
                println!();
                println!("Version {version}:");
                println!();
                print_explain_answer(*version, answer, &next);
            }
        }
    }
    ExitCode::SUCCESS
}

/// §FS-config-explain.1: the first positional is the target and the second the
/// path, unless `--path` names the path; `--codes` and `--all` take no target.
fn explain_usage(args: &[String]) -> Result<(ExplainAsk, Option<PathBuf>), String> {
    let (mut positionals, mut flag_path, mut flag) = (Vec::new(), None, None);
    let mut words = args.iter();
    while let Some(arg) = words.next() {
        match arg.as_str() {
            "--codes" | "--all" => {
                if flag.is_some_and(|seen| seen != arg) {
                    return Err("config explain takes --codes or --all, not both".to_string());
                }
                flag = Some(arg.as_str());
            }
            "--path" => match words.next() {
                Some(path) => flag_path = Some(PathBuf::from(path)),
                None => return Err("--path requires a value".to_string()),
            },
            other if other.starts_with("--path=") => {
                flag_path = Some(PathBuf::from(&other["--path=".len()..]));
            }
            other if other.starts_with('-') => return Err(format!("unknown flag `{other}`")),
            other => positionals.push(other),
        }
    }
    // A positional past the target and the path, or past the target where
    // `--path` names the path, is a second target.
    let room = if flag_path.is_some() { 1 } else { 2 };
    if let Some(extra) = positionals.get(room) {
        return Err(format!(
            "config explain takes one target; `{extra}` is a second (the path is the positional after the target, or --path)"
        ));
    }
    if let (Some(flag), Some(target)) = (flag, positionals.first()) {
        return Err(format!(
            "config explain {flag} takes no target, and `{target}` is one (give a path with --path)"
        ));
    }
    let path = flag_path.or_else(|| positionals.get(1).map(PathBuf::from));
    let ask = match (flag, positionals.first()) {
        (Some("--codes"), _) => ExplainAsk::Codes,
        (Some(_), _) => ExplainAsk::All,
        (None, Some(target)) => ExplainAsk::Target(target.to_string()),
        (None, None) => ExplainAsk::Index,
    };
    Ok((ask, path))
}

/// The call that asks for `target`, quoted for a shell where it holds a space or
/// a bracket, with the path this call was given.
fn explain_call(target: &str, path: Option<&str>) -> String {
    let quoted = |word: &str| {
        if word.contains([' ', '[', ']', '<', '>', '*']) {
            format!("'{word}'")
        } else {
            word.to_string()
        }
    };
    match path {
        Some(path) => format!("grund config explain {} {}", quoted(target), quoted(path)),
        None => format!("grund config explain {}", quoted(target)),
    }
}

/// One version's answer, by rung (§FS-config-explain.1.2), each but `--all`'s
/// ending in the `Next:` line that asks for the rung below it.
fn print_explain_answer(version: u32, answer: &ExplainAnswer, next: &dyn Fn(&str) -> String) {
    let follow = match answer {
        ExplainAnswer::Index { tables, codes } => {
            for table in tables {
                print_explain_index_line(table);
            }
            println!("{codes} check codes: {}", next_call("--codes"));
            tables.iter().find(|table| !table.header.is_empty()).map(|table| table.header.clone())
        }
        ExplainAnswer::Table(table) => {
            print_explain_table(table);
            let first = table.keys.first();
            first.map(|key| explain_key_target(&key.table, &key.key)).or_else(|| table.tables.first().cloned())
        }
        ExplainAnswer::Key(key) => {
            print_explain_key(key);
            key.members.first().map(|member| explain_key_target(&key.table, member))
        }
        ExplainAnswer::Code(code) => {
            print_explain_code(version, code);
            code.tables.first().cloned()
        }
        ExplainAnswer::Codes(codes) => {
            for code in codes {
                println!("{}: {}", code.code, code.meaning);
            }
            codes.first().map(|code| code.code.clone())
        }
        ExplainAnswer::All { tables, codes } => {
            for table in tables {
                print_explain_table(table);
                println!();
            }
            for code in codes {
                print_explain_code(version, code);
                println!();
            }
            None
        }
        // §FS-config-explain.2: with no file, the version that does not know a
        // target the other does says so, and the call still answers.
        ExplainAnswer::Unknown(unknown) => {
            println!("{}", explain_unknown(version, unknown));
            None
        }
    };
    if let Some(target) = follow {
        println!();
        println!("Next: {}", next(&target));
    }
}

/// `--codes` is a flag rather than a target, so it follows the call unquoted.
fn next_call(flag: &str) -> String {
    format!("grund config explain {flag}")
}

/// An index line: the top level's keys, or a table and what it is for.
fn print_explain_index_line(table: &ExplainTable) {
    if table.header.is_empty() {
        let keys: Vec<&str> = table.keys.iter().map(|key| key.key.as_str()).collect();
        println!("Top-level keys: {}", keys.join(", "));
        return;
    }
    match &table.meaning {
        Some(meaning) => println!("{}  {meaning}", table.header),
        None => println!("{}", table.header),
    }
}

/// §FS-config-explain.1.2: a table, what it is for and each key it admits, or
/// the tables under it where it holds only tables.
fn print_explain_table(table: &ExplainTable) {
    println!("{}", if table.header.is_empty() { "Top level" } else { &table.header });
    if let Some(meaning) = &table.meaning {
        println!("{meaning}");
    }
    if !table.keys.is_empty() {
        println!();
        for key in &table.keys {
            println!("  {} ({}): {}", key.key, key.values, key.meaning);
        }
    }
    if !table.tables.is_empty() {
        println!();
        println!("Tables under it: {}", table.tables.join(", "));
    }
}

/// §FS-config-explain.3: a key, its values in words and what it does.
fn print_explain_key(key: &ExplainKey) {
    println!("{}", explain_key_target(&key.table, &key.key));
    println!("Values: {}", key.values);
    println!("{}", key.meaning);
}

/// The target that asks for `key`: after its table's header, bare at the top level.
fn explain_key_target(table: &str, key: &str) -> String {
    if table.is_empty() {
        key.to_string()
    } else {
        format!("{table} {key}")
    }
}

/// §FS-config-explain.4: what a code means, its severity, what turns it on,
/// and its ramp, read from its row for `version`.
fn print_explain_code(version: u32, code: &ExplainCode) {
    println!("{}", code.code);
    println!("{}", code.meaning);
    println!();
    println!("Severity: {}", explain_severity(&code.severity));
    match &code.enabled {
        CodeEnabled::Always => println!("On: always."),
        CodeEnabled::Absent { .. } => {
            println!("On: never; a file read as version {version} does not turn it on.");
            if version < 2 {
                println!("    grund config migrate --to 2 gets to a version that can.");
            }
        }
        CodeEnabled::Clauses { clauses, remark } => {
            let mut lead = "On: when";
            for clause in clauses {
                println!("{lead} {} ({})", clause.text, explain_clause_by(clause.by));
                lead = "    and";
            }
            if let Some(remark) = remark {
                println!("    {remark}");
            }
        }
    }
    if let Some(ramp) = &code.ramp {
        println!("Ramp: {ramp}");
    }
}

/// §FS-config-explain.4: which of the four a *Severity* cell is, in words.
fn explain_severity(severity: &CodeSeverity) -> String {
    match severity {
        CodeSeverity::Fixed(severity) => format!("{severity}, fixed."),
        CodeSeverity::Suggestion(_) => "none — a suggestion, shown with --suggestions.".to_string(),
        CodeSeverity::Spent(what) => format!("none — {what}."),
        // A cell that already says what each strength gives is not told it twice.
        CodeSeverity::Strength(by) if by.contains(" an error") => format!("by strength: {by}."),
        CodeSeverity::Strength(by) => format!(
            "by strength: {by}. A strength gives: must an error, warn a warning, should a suggestion, may not checked."
        ),
        CodeSeverity::Setting(text) => format!("chosen by a setting: {text}."),
        CodeSeverity::Absent(text) => format!("none — {text}."),
    }
}

/// §FS-errors.5.5.1: what meets a clause of an *Enabled by* cell.
fn explain_clause_by(by: ClauseBy) -> &'static str {
    match by {
        ClauseBy::Setting => "a setting",
        ClauseBy::Flag => "a flag",
        ClauseBy::Either => "a setting or a flag",
        ClauseBy::Condition => "a condition of the tree",
    }
}

/// §FS-config-explain.5: what a version that does not know a target says,
/// naming the target, the version and the next step; a list of homes is the
/// `hint:` line that may follow a bare query failure (§FS-errors.2.3.1).
fn explain_unknown(version: u32, unknown: &ExplainUnknown) -> String {
    match unknown {
        ExplainUnknown::VersionOneTable { table, homes } => format!(
            "version {version} has no table `{table}`: it is a version 1 table\nhint: version {version} writes its settings in {}",
            homes.join(", ")
        ),
        ExplainUnknown::VersionTwoTable { table } => format!(
            "version {version} has no table `{table}`: it is a version 2 table; grund config migrate --to 2 moves this config to version 2"
        ),
        ExplainUnknown::Key { table, key, admitted } if admitted.is_empty() => format!(
            "version {version} `{table}` has no key `{key}`: it holds only tables; grund config explain '{table}' names them"
        ),
        ExplainUnknown::Key { table, key, admitted } => format!(
            "version {version} `{table}` has no key `{key}` (expected one of: {})",
            admitted.join(", ")
        ),
        ExplainUnknown::Bracketed { target, spelling, key } => format!(
            "version {version} has no top-level key or check code `{target}`; for the {}, ask grund config explain '{spelling}'",
            if *key { "key" } else { "table" }
        ),
        ExplainUnknown::Other { target } => format!(
            "version {version} knows no table, key or check code `{target}`; grund config explain lists the tables, grund config explain --codes the codes"
        ),
    }
}
