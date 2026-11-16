//! A failed workspace fetch retains its child's explanation (§FS-fetch.2.2), exit class,
//! qualified context and unchanged tree (§FS-fetch.2). The integration checks its local argv.
#![cfg(unix)]

#[path = "../../crates/grund-cli/tests/support/external_facts.rs"]
mod support;

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use support::{assert_code, executable, stderr, stdout, tree_with_directories, write};

const SNAPSHOT: &str = "# GR-1: Existing snapshot\n\nPreserve these bytes.\n";

struct Workspace(PathBuf);

impl Workspace {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "grund-1358-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).expect("claim workspace root");
        write(
            &root,
            "grund.toml",
            "grund_config_version = 2\nproject_name = \"root\"\n\n\
             [workspace]\nmembers = [\"graal\"]\n",
        );
        write(
            &root,
            "graal/grund.toml",
            "grund_config_version = 2\nproject_name = \"graal\"\n\n\
             [schema.kinds.GR]\nfolders = [\"docs/gr\"]\nid_format = \"{kind}-{number}\"\n\
             index = false\nfetch = \"scripts/fetch\"\n",
        );
        write(&root, "graal/docs/gr/GR-1.md", SNAPSHOT);
        executable(
            &root,
            "graal/scripts/fetch",
            include_str!("support/fetch_stderr.sh"),
        );
        Self(root)
    }

    fn fetch(&self, id: &str) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(["fetch", &format!("graal/{id}")])
            .current_dir(&self.0)
            .env("NO_COLOR", "1")
            .output()
            .expect("run grund fetch")
    }

    fn failed(&self, id: &str, status: &str) -> Output {
        let before = tree_with_directories(&self.0);
        let output = self.fetch(id);
        assert_code(&output, 2, id);
        assert_eq!(stdout(&output), "", "snapshot data must stay off stdout");
        let prefix =
            format!("error: fetch integration `scripts/fetch` for graal/{id} exited {status}");
        assert!(stderr(&output).starts_with(&prefix), "{}", stderr(&output));
        assert_eq!(
            tree_with_directories(&self.0),
            before,
            "failed fetch mutated tree"
        );
        output
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn assert_reason(id: &str, status: &str, reason: &str) {
    let workspace = Workspace::new();
    let output = workspace.failed(id, status);
    assert!(
        stderr(&output).contains(reason),
        "missing child diagnostic {reason:?}; status={:?}, stdout={:?}, stderr={:?}",
        output.status.code(),
        stdout(&output),
        stderr(&output)
    );
}

/// §FS-fetch.2.2: a missing dependency's explanation survives a failed refetch.
#[test]
fn fetch_stderr_missing_cli_reason_reaches_user() {
    assert_reason("GR-1", "1", "fetch-ticket: gdev-cli is not on PATH");
}

/// §FS-fetch.2.2: an unknown ticket has its own explanation, without creating a snapshot.
#[test]
fn fetch_stderr_unknown_ticket_reason_reaches_user() {
    assert_reason("GR-404", "1", "fetch-ticket: GR-404: Issue Does Not Exist");
}

/// §FS-fetch.2.2: line breaks survive, and diagnostic bytes are displayed lossily.
#[test]
fn fetch_stderr_multiline_non_utf8_reason_reaches_user() {
    assert_reason(
        "GR-4",
        "1",
        "fetch-ticket: credential expired\nrenew token: bad byte \u{fffd}",
    );
}

/// §FS-fetch.2.2: the shared unsuccessful-status path also keeps signal diagnostics.
#[test]
fn fetch_stderr_signal_reason_reaches_user() {
    assert_reason(
        "GR-5",
        "signal",
        "fetch-ticket: interrupted while contacting provider",
    );
}

/// §FS-fetch.2.2: a successful integration's diagnostics stay quiet; stdout installs verbatim.
#[test]
fn fetch_stderr_success_is_quiet_and_installs_snapshot() {
    let workspace = Workspace::new();
    let output = workspace.fetch("GR-2");
    assert_code(&output, 0, "noisy successful fetch");
    assert_eq!(stdout(&output), "");
    assert_eq!(stderr(&output), "");
    assert_eq!(
        fs::read(workspace.0.join("graal/docs/gr/GR-2.md")).expect("installed snapshot"),
        b"# GR-2: A ticket\n\nFetched by the fixture.\n"
    );
    assert_eq!(
        fs::read_to_string(workspace.0.join("graal/docs/gr/GR-1.md")).unwrap(),
        SNAPSHOT
    );
}

/// §FS-fetch.2.2: no diagnostic bytes means no change to the existing failure message.
#[test]
fn fetch_stderr_empty_failure_keeps_message() {
    let workspace = Workspace::new();
    let output = workspace.failed("GR-3", "1");
    assert_eq!(
        stderr(&output),
        "error: fetch integration `scripts/fetch` for graal/GR-3 exited 1\n"
    );
}
