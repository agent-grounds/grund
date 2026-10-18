//! §FS-integrations.1.3, §AR-bindings.3 — the bytes `grund integrations …
//! --write` prints and writes, pinned before the CLI renders the engine's
//! `integrations` answer
//! (§FS-integrations.1.3.1) and held after it: each client on a first run and a
//! re-run, every guidance target with and without an agent override, a missing
//! agent home, the user configuration's cautions, and a filesystem failure
//! partway through a run. A case's golden is every invocation's exit code,
//! stdout and stderr, then every file and directory the sandbox ends with, with
//! the sandbox's own path spelled `<sandbox>`.
//!
//! `GRUND_BLESS_INTEGRATIONS=1` rewrites the goldens from the binary under test.

use super::binaries;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

const CLIENTS: [&str; 6] = ["codium", "iterm2", "kitty", "tmux", "vscode", "wezterm"];
const TARGETS: [&str; 6] = ["file", "path", "web", "vscode", "vscodium", "cursor"];
const AGENT_HOMES: [&str; 6] = [".codex", ".claude", ".gemini", ".copilot", "xdg/zed", ".pi"];

/// One case's sandbox: `home/` is `HOME`, `xdg/` is `XDG_CONFIG_HOME`.
struct Sandbox {
    root: PathBuf,
}

impl Sandbox {
    fn new(case: &str) -> Self {
        let root = binaries::repo_root()
            .join("target/integration-work/write-goldens")
            .join(case);
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("home")).expect("create sandbox HOME");
        fs::create_dir_all(root.join("xdg")).expect("create sandbox XDG_CONFIG_HOME");
        Sandbox { root }
    }

    fn home(&self) -> PathBuf {
        self.root.join("home")
    }

    /// A path under the sandbox: `xdg/…` lands in `XDG_CONFIG_HOME`, anything
    /// else under `HOME`.
    fn at(&self, relative: &str) -> PathBuf {
        match relative.strip_prefix("xdg/") {
            Some(rest) => self.root.join("xdg").join(rest),
            None => self.home().join(relative),
        }
    }

    fn with_agent_homes(self) -> Self {
        for home in AGENT_HOMES {
            fs::create_dir_all(self.at(home)).expect("create agent home");
        }
        self
    }

    fn with_file(self, relative: &str, text: &str) -> Self {
        let path = self.at(relative);
        fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
        fs::write(path, text).expect("write fixture file");
        self
    }

    fn with_dir(self, relative: &str) -> Self {
        fs::create_dir_all(self.at(relative)).expect("create fixture directory");
        self
    }

    /// Run `grund <args>` with nothing from the caller's environment but `PATH`,
    /// and append what it said to the transcript.
    fn run(&self, transcript: &mut String, args: &[&str]) {
        let output = Command::new(binaries::grund())
            .args(args)
            .env_clear()
            .env("PATH", std::env::var("PATH").unwrap_or_default())
            .env("HOME", self.home())
            .env("XDG_CONFIG_HOME", self.root.join("xdg"))
            .current_dir(&self.root)
            .output()
            .expect("run grund integrations");
        transcript.push_str(&format!(
            "$ grund {}\nexit: {}\n--- stdout\n{}--- stderr\n{}",
            args.join(" "),
            output
                .status
                .code()
                .map_or("signal".into(), |code| code.to_string()),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        ));
    }

    /// Every directory and file under the sandbox, sorted, with file bytes and
    /// the executable bit, so a write the transcript does not mention still shows.
    fn tree(&self, transcript: &mut String) {
        transcript.push_str("=== tree\n");
        let mut entries = Vec::new();
        walk(&self.root, &mut entries);
        entries.sort();
        for path in entries {
            let relative = path.strip_prefix(&self.root).expect("under sandbox");
            let meta = fs::metadata(&path).expect("stat sandbox entry");
            if meta.is_dir() {
                transcript.push_str(&format!("--- dir {}/\n", relative.display()));
            } else {
                let mode = if meta.permissions().mode() & 0o111 != 0 {
                    " (executable)"
                } else {
                    ""
                };
                transcript.push_str(&format!("--- file {}{mode}\n", relative.display()));
                transcript.push_str(&String::from_utf8_lossy(&fs::read(&path).expect("read")));
                transcript.push_str("\n--- end\n");
            }
        }
    }

    fn finish(self, case: &str, mut transcript: String) -> (String, String) {
        self.tree(&mut transcript);
        let normalized = transcript.replace(&self.root.display().to_string(), "<sandbox>");
        let golden = binaries::repo_root()
            .join("tests/integration/integrations_goldens")
            .join(format!("{case}.golden"));
        if std::env::var_os("GRUND_BLESS_INTEGRATIONS").is_some() {
            fs::create_dir_all(golden.parent().expect("golden dir")).expect("golden dir");
            fs::write(&golden, &normalized).expect("bless golden");
        }
        let expected = fs::read_to_string(&golden)
            .unwrap_or_else(|err| panic!("read {}: {err}", golden.display()));
        (expected, normalized)
    }
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("read sandbox dir") {
        let path = entry.expect("dir entry").path();
        out.push(path.clone());
        if fs::symlink_metadata(&path).expect("lstat").is_dir() {
            walk(&path, out);
        }
    }
}

/// Run one case to its golden and report every mismatch at once.
fn golden(case: &str, sandbox: Sandbox, runs: &[&[&str]]) -> Option<String> {
    let mut transcript = String::new();
    for args in runs {
        sandbox.run(&mut transcript, args);
    }
    let (expected, actual) = sandbox.finish(case, transcript);
    (expected != actual).then(|| {
        let line = expected
            .lines()
            .zip(actual.lines())
            .position(|(left, right)| left != right)
            .map_or("its length".to_string(), |at| format!("line {}", at + 1));
        format!("{case}: differs from its golden at {line}\n--- actual\n{actual}")
    })
}

fn assert_goldens(failures: Vec<Option<String>>) {
    let failures: Vec<String> = failures.into_iter().flatten().collect();
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// §FS-integrations.4 and §FS-integrations.6.1: every client's first `--write`
/// and its re-run, on a machine with every agent home, and a WezTerm config that
/// already exists without its apply call (§FS-integrations.4.1.3).
#[test]
fn each_client_writes_the_same_bytes_on_a_first_run_and_a_re_run() {
    let mut failures = Vec::new();
    for client in CLIENTS {
        let case = format!("client-{client}");
        let write: &[&str] = &["integrations", client, "--write"];
        failures.push(golden(
            &case,
            Sandbox::new(&case).with_agent_homes(),
            &[write, write],
        ));
    }
    let case = "client-wezterm-unwired";
    let sandbox = Sandbox::new(case).with_file(
        "xdg/wezterm/wezterm.lua",
        "local wezterm = require 'wezterm'\nlocal config = wezterm.config_builder()\nreturn config\n",
    );
    failures.push(golden(
        case,
        sandbox,
        &[&["integrations", "wezterm", "--write"]],
    ));
    assert_goldens(failures);
}

/// §FS-integrations.4.3.2, §FS-integrations.4.4: every guidance target, written
/// machine-wide and then scoped to one agent, gated per agent
/// (§FS-integrations.4.3.3).
#[test]
fn every_guidance_target_writes_the_same_bytes_with_and_without_an_agent_override() {
    let mut failures = Vec::new();
    for target in TARGETS {
        let case = format!("target-{target}");
        let base: &[&str] = &[
            "integrations",
            "--write",
            "--conversation",
            "link",
            "--conversation-target",
            target,
        ];
        failures.push(golden(
            &case,
            Sandbox::new(&case).with_agent_homes(),
            &[base],
        ));

        let case = format!("target-{target}-agent-codex");
        let scoped: &[&str] = &[
            "integrations",
            "kitty",
            "--write",
            "--conversation",
            "link",
            "--conversation-target",
            target,
            "--agent",
            "codex",
        ];
        failures.push(golden(
            &case,
            Sandbox::new(&case).with_agent_homes(),
            &[scoped, scoped],
        ));
    }
    assert_goldens(failures);
}

/// §FS-integrations.4.3.9: no agent home at all, so every instruction target is
/// reported skipped and nothing is created for it.
#[test]
fn a_machine_with_no_agent_home_skips_every_instruction_target() {
    let case = "agents-missing";
    assert_goldens(vec![golden(
        case,
        Sandbox::new(case),
        &[&["integrations", "kitty", "--write"]],
    )]);
}

/// §FS-integrations.4.3.5: an unused key, an unknown agent, an uninterpretable
/// value and a repeated key are each one warning, and none is an error.
#[test]
fn the_user_configurations_cautions_are_warnings_with_the_same_bytes() {
    let case = "user-config-cautions";
    let sandbox = Sandbox::new(case).with_agent_homes().with_file(
        "xdg/grund/config.toml",
        concat!(
            "[reference]\n",
            "conversation = \"link\"\n",
            "conversation = \"plain\"\n",
            "conversation_target = \"telnet\"\n",
            "marker = \"\u{a7}\"\n",
            "[reference.agents.nobody]\n",
            "conversation_target = \"web\"\n",
            "[reference.agents.codex]\n",
            "conversation_target = \"vscode\"\n",
        ),
    );
    assert_goldens(vec![golden(
        case,
        sandbox,
        &[&["integrations", "tmux", "--write"]],
    )]);
}

/// §FS-integrations.4.3.14: a failure partway leaves what was already written,
/// reports it, and exits 2 — once where the resolver cannot be written after the
/// client block was, once where an agent's instruction file cannot be read.
#[test]
fn a_filesystem_failure_partway_through_a_run_keeps_the_same_bytes() {
    let case = "failure-resolver";
    let sandbox = Sandbox::new(case)
        .with_agent_homes()
        .with_file(".local/bin", "not a directory\n");
    let resolver = golden(case, sandbox, &[&["integrations", "kitty", "--write"]]);

    let case = "failure-agent-file";
    let sandbox = Sandbox::new(case)
        .with_agent_homes()
        .with_dir(".claude/CLAUDE.md");
    let agent = golden(case, sandbox, &[&["integrations", "kitty", "--write"]]);
    assert_goldens(vec![resolver, agent]);
}
