// §FS-examples.5.4: external argv shares copying, stdin and invocation with Grund.
/// The command a case runs, as written: its `command.args`, or the default every
/// case without one runs. One reader, so "is this a mutating case?", "does it use
/// the copy?", and the argument expansion below cannot disagree about what the
/// case does.
fn case_command(case: &Path) -> String {
    if let Some(words) = external_words(case) {
        return serde_json::to_string(&words).expect("validated argv");
    }
    let command_file = case.join("command.args");
    if command_file.exists() {
        read_to_string(command_file)
    } else {
        DEFAULT_COMMAND.to_string()
    }
}

const DEFAULT_COMMAND: &str = "check {repo}";

fn is_mutating_case(case: &Path) -> bool {
    let command = case_command(case);
    let cwd = case_command_cwd(case);
    command.contains("--write") || command.contains("{repo_copy}") || cwd.contains("{repo_copy}")
}

/// Shared direct invocation and stdin for both manifests (§FS-examples.5.4).
fn run_grund(case: &Path, cwd: &Path, args: &[String], name: &str) -> Output {
    let input_path = case.join("command.stdin");
    let external = external_words(case).is_some();
    let (executable, argv) = if external {
        (args[0].as_str(), &args[1..])
    } else {
        (env!("CARGO_BIN_EXE_grund"), args)
    };
    let mut command = Command::new(executable);
    command.args(argv).current_dir(cwd);
    if !input_path.exists() {
        return command
            .output()
            .unwrap_or_else(|err| panic!("{name}: run executable {executable:?}: {err}"));
    }

    // §FS-show.1.8: an e2e manifest can exercise an stdin query stream without
    // wrapping the public command in a shell.
    let input = fs::read(&input_path)
        .unwrap_or_else(|err| panic!("{name}: read {}: {err}", input_path.display()));
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|err| panic!("{name}: run executable {executable:?}: {err}"));
    let write_result = child.stdin.take().expect("piped stdin").write_all(&input);
    if let Err(err) = write_result
        && err.kind() != std::io::ErrorKind::BrokenPipe
    {
        panic!("{name}: write stdin: {err}");
    }
    child
        .wait_with_output()
        .unwrap_or_else(|err| panic!("{name}: read grund output: {err}"))
}

/// Optional working-directory manifest for a command whose public form relies
/// on config discovery from `.`. This is shared by examples and e2e cases per
/// §FS-examples.5, and accepts only `{repo}` or `{repo_copy}`.
fn case_command_cwd(case: &Path) -> String {
    let path = case.join("command.cwd");
    if path.exists() {
        read_to_string(path).trim().to_string()
    } else {
        String::new()
    }
}

fn command_cwd(manifest_dir: &Path, case: &Path, name: &str) -> PathBuf {
    match case_command_cwd(case).as_str() {
        "" => manifest_dir.to_path_buf(),
        "{repo}" => case.join("repo"),
        "{repo_copy}" => manifest_dir.join("target/e2e-work").join(name).join("repo"),
        other => panic!("{name}: command.cwd must be {{repo}} or {{repo_copy}}, got {other:?}"),
    }
}

/// Validate before copying, then expand shared argv placeholders (§FS-examples.5.4).
fn command_args(manifest_dir: &Path, case: &Path, name: &str) -> Vec<String> {
    let repo = case.join("repo");
    let repo_arg = repo
        .strip_prefix(manifest_dir)
        .unwrap_or(&repo)
        .to_string_lossy()
        .into_owned();
    let repo_copy = manifest_dir.join("target/e2e-work").join(name).join("repo");
    let repo_copy_arg = repo_copy
        .strip_prefix(manifest_dir)
        .unwrap_or(&repo_copy)
        .to_string_lossy()
        .into_owned();
    let command = case_command(case);
    // Split before the copy: a refused line spends no fixture (§FS-examples.5.3).
    let external = external_words(case);
    let is_external = external.is_some();
    let words = external.unwrap_or_else(|| split_command(name, &command));
    if command.contains("{repo_copy}") || case_command_cwd(case) == "{repo_copy}" {
        if let Some(parent) = repo_copy.parent() {
            let _ = fs::remove_dir_all(parent);
            fs::create_dir_all(parent)
                .unwrap_or_else(|err| panic!("{name}: create {}: {err}", parent.display()));
        }
        copy_dir(&repo, &repo_copy);
        create_case_symlinks(case, &repo_copy, name);
    }

    words
        .into_iter()
        .map(|arg| {
            let arg = arg.as_str();
            if is_external && arg == "{grund}" {
                env!("CARGO_BIN_EXE_grund").to_string()
            } else if let Some(suffix) = arg.strip_prefix("{repo}/") {
                PathBuf::from(&repo_arg)
                    .join(suffix)
                    .to_string_lossy()
                    .into_owned()
            } else if let Some(suffix) = arg.strip_prefix("{repo_copy}/") {
                PathBuf::from(&repo_copy_arg)
                    .join(suffix)
                    .to_string_lossy()
                    .into_owned()
            } else if arg == "{repo}" {
                repo_arg.clone()
            } else if arg == "{repo_copy}" {
                repo_copy_arg.clone()
            } else {
                arg.to_string()
            }
        })
        .collect()
}

/// The words a POSIX shell would split a case's `command.args` into, honouring
/// its quotes and nothing else (§FS-examples.5.3): whitespace outside quotes
/// separates arguments, `'…'` and `"…"` keep what they enclose in one argument
/// with the quotes removed, and parts with no whitespace between them join. A
/// `--rule` sentence (§FS-rules.8) is several words by grammar, and
/// `id FS 'Password reset'` a title of two, so a manifest split on whitespace
/// alone could express neither. What a shell would read differently — a
/// backslash outside single quotes, a quote never closed — is refused, naming
/// the case and `command.args` the way `command.cwd` does, rather than run as
/// an argv its author did not write.
fn split_command(name: &str, command: &str) -> Vec<String> {
    let refuse = |what: &str| -> ! { panic!("{name}: command.args: {what}: {:?}", command.trim()) };
    let mut args = Vec::new();
    // `None` until a word starts, so a bare `''` is still one (empty) argument.
    let mut word: Option<String> = None;
    let mut quote: Option<char> = None;
    for character in command.chars() {
        match (quote, character) {
            (Some(open), c) if c == open => quote = None,
            (None | Some('"'), '\\') => refuse(
                "a backslash outside single quotes is not an escape here; \
                 quote the argument instead, in single quotes to keep the backslash",
            ),
            (Some(_), c) => word.get_or_insert_default().push(c),
            (None, '\'' | '"') => {
                quote = Some(character);
                word.get_or_insert_default();
            }
            (None, c) if c.is_whitespace() => args.extend(word.take()),
            (None, c) => word.get_or_insert_default().push(c),
        }
    }
    match quote {
        Some('\'') => refuse("unclosed single quote"),
        Some(_) => refuse("unclosed double quote"),
        None => args.extend(word),
    }
    args
}

/// Validate external manifests before copying or spawning (§FS-examples.5.4).
fn external_words(case: &Path) -> Option<Vec<String>> {
    let path = case.join("command.external");
    if !path.exists() {
        return None;
    }
    let name = case_name(case);
    assert!(
        !case.join("command.args").exists(),
        "{name}: command.external and command.args are mutually exclusive"
    );
    let words: Vec<String> = serde_json::from_str(&read_to_string(path))
        .unwrap_or_else(|err| panic!("{name}: command.external: expected JSON string argv: {err}"));
    assert!(
        words.first().is_some_and(|word| !word.is_empty()),
        "{name}: command.external needs a nonempty executable and argv"
    );
    Some(words)
}
