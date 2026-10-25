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

/// The same owner supplies argv, cwd and every later reader (§FS-examples.5.6.1).
struct CaseExecution {
    copy: Option<CaseCopy>,
    args: Vec<String>,
    cwd: PathBuf,
    is_external: bool,
}

impl CaseExecution {
    fn copied_repo(&self, name: &str) -> &Path {
        self.copy.as_ref().map(|copy| copy.repo.as_path()).unwrap_or_else(|| {
            panic!("{name}: expected.repo requires command.args or command.cwd to use {{repo_copy}}")
        })
    }

    /// Normalize before the comparison/refresh split (§FS-examples.5.6.2).
    fn normalize(&self, text: &str) -> String {
        self.copy
            .as_ref()
            .map_or_else(|| text.to_string(), |copy| copy.normalize(text))
    }
}

/// Validate all command manifests before allocating or copying, and retain the
/// owner even if copying or link creation unwinds (§FS-examples.5.6.1).
fn prepare_case(manifest_dir: &Path, case: &Path, name: &str) -> CaseExecution {
    let command = case_command(case);
    let external = external_words(case);
    let is_external = external.is_some();
    let words = external.unwrap_or_else(|| split_command(name, &command));
    let cwd = case_command_cwd(case);
    assert!(
        matches!(cwd.as_str(), "" | "{repo}" | "{repo_copy}"),
        "{name}: command.cwd must be {{repo}} or {{repo_copy}}, got {cwd:?}"
    );
    // Reading this manifest validates it before the fixture is copied, too.
    let _ = case_symlinks(case);
    let copy = (command.contains("{repo_copy}") || cwd == "{repo_copy}")
        .then(|| CaseCopy::new(manifest_dir, name));
    if let Some(copy) = &copy {
        copy_dir(&case.join("repo"), &copy.repo);
        create_case_symlinks(case, &copy.repo, name);
    }
    let args = command_args(manifest_dir, case, words, is_external, copy.as_ref());
    let cwd = match cwd.as_str() {
        "{repo}" => case.join("repo"),
        "{repo_copy}" => copy.as_ref().expect("copy selected by cwd").repo.clone(),
        _ => manifest_dir.to_path_buf(),
    };
    CaseExecution {
        copy,
        args,
        cwd,
        is_external,
    }
}

/// Expand whole and suffix arguments from the invocation's copy (§FS-examples.5.6.1).
fn command_args(
    manifest_dir: &Path,
    case: &Path,
    words: Vec<String>,
    is_external: bool,
    copy: Option<&CaseCopy>,
) -> Vec<String> {
    let repo = case.join("repo");
    let repo_arg = repo
        .strip_prefix(manifest_dir)
        .unwrap_or(&repo)
        .to_string_lossy()
        .into_owned();
    let repo_copy_arg = copy.map(|copy| {
        copy.repo
            .strip_prefix(manifest_dir)
            .unwrap_or(&copy.repo)
            .to_string_lossy()
            .into_owned()
    });

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
                PathBuf::from(repo_copy_arg.as_ref().expect("copy selected by argv"))
                    .join(suffix)
                    .to_string_lossy()
                    .into_owned()
            } else if arg == "{repo}" {
                repo_arg.clone()
            } else if arg == "{repo_copy}" {
                repo_copy_arg.clone().expect("copy selected by argv")
            } else {
                arg.to_string()
            }
        })
        .collect()
}

// One manifest grammar for corpus and agreement (§FS-examples.5.3).
include!("case_words.rs");

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

/// A case's own command run in its copy and kept, for a test that goes on to
/// run more commands over the tree it left (§FS-examples.5.6.1). The copy is
/// removed when this is dropped. Only `init_then_check.rs` reads it, so the
/// other binaries that include the runner allow it unused.
#[allow(dead_code)]
pub struct CopiedRun {
    execution: CaseExecution,
    pub output: Output,
}

#[allow(dead_code)]
impl CopiedRun {
    pub fn repo(&self) -> &Path {
        &self.execution.copy.as_ref().expect("a copied case").repo
    }
}

/// [`CopiedRun`] for `case`, or `None` where its command runs in place.
#[allow(dead_code)]
pub fn run_in_copy(manifest_dir: &Path, case: &Path) -> Option<CopiedRun> {
    let name = case.file_name()?.to_string_lossy().into_owned();
    let execution = prepare_case(manifest_dir, case, &name);
    execution.copy.as_ref()?;
    let output = run_grund(case, &execution.cwd, &execution.args, &name);
    Some(CopiedRun { execution, output })
}
