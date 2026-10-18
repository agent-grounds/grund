//! §FS-distribution.3.1.3 and §FS-integrations.1.3: an embedder that asks
//! `integrations` for `client = Kitty, write = true` installs exactly what
//! `grund integrations kitty --write` installs — the same files with the same
//! bytes — and its events name the targets the CLI reports, with the CLI's verbs.
#![cfg(unix)]

#[path = "../../../tests/integration/binaries.rs"]
mod binaries;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A fresh `HOME` and `XDG_CONFIG_HOME` with one agent in use, so the run
/// writes a client block, the resolver, the preference file and one agent's
/// instruction block (§FS-integrations.4.3.9).
fn sandbox(name: &str) -> PathBuf {
    let root = binaries::repo_root()
        .join("target/integration-work/rust-api")
        .join(name);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("home/.claude")).expect("create sandbox HOME");
    fs::create_dir_all(root.join("xdg")).expect("create sandbox XDG_CONFIG_HOME");
    root
}

fn run(program: &Path, args: &[&str], root: &Path) -> (String, String) {
    let output = Command::new(program)
        .args(args)
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("HOME", root.join("home"))
        .env("XDG_CONFIG_HOME", root.join("xdg"))
        .output()
        .unwrap_or_else(|err| panic!("run {}: {err}", program.display()));
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(
        output.status.success(),
        "{} failed:\n{stderr}",
        program.display()
    );
    let root = root.display().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).replace(&root, "<sandbox>");
    (stdout, stderr.replace(&root, "<sandbox>"))
}

/// Every file under the sandbox, by relative path, with its bytes.
fn files(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(root: &Path, dir: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for entry in fs::read_dir(dir).expect("read sandbox dir") {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                walk(root, &path, out);
            } else {
                let relative = path.strip_prefix(root).expect("under sandbox");
                out.insert(
                    relative.display().to_string(),
                    fs::read(&path).expect("read"),
                );
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(root, root, &mut out);
    out
}

#[test]
fn an_embedder_installs_kitty_with_the_bytes_the_cli_writes() {
    let embedder = sandbox("embedder");
    let cli = sandbox("cli");
    let example = PathBuf::from(env!("CARGO_BIN_EXE_grund-rust-api-example"));
    let (events, _) = run(&example, &["kitty", "--write"], &embedder);
    let (_, reported) = run(
        &binaries::grund(),
        &["integrations", "kitty", "--write"],
        &cli,
    );

    let written = files(&embedder);
    assert!(!written.is_empty(), "the embedder wrote nothing");
    assert_eq!(
        written,
        files(&cli),
        "the embedder and the CLI wrote different files or bytes"
    );

    // The CLI's report line is the verb and the path, then its own note.
    let cli_events: Vec<String> = reported
        .lines()
        .filter(|line| !line.starts_with("warning:") && !line.starts_with("note:"))
        .map(|line| line.split(" (").next().unwrap_or(line).to_string())
        .collect();
    let embedder_events: Vec<&str> = events.lines().collect();
    assert_eq!(
        embedder_events, cli_events,
        "the events are not the targets the CLI reports"
    );
}
