//! Snapshot-retention and external-scan edge cases for multi-root sessions
//! (§FS-lsp.2.2, §AR-lsp.2).

mod support;

use serde_json::{Value, json};
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin};
use std::sync::mpsc;
use support::*;

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
    fs::write(path, text).expect("write fixture");
}

fn write_project(root: &Path, source_name: &str) -> (PathBuf, PathBuf) {
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n[scan]\ninclude = [\"docs\", \"src\"]\n\
         extensions = [\"md\", \"java\"]\n",
    );
    let spec = root.join("docs/FS-001-example.md");
    let source = root.join(format!("src/{source_name}.java"));
    write(&spec, "# FS-001-example: Example\n\nLead.\n");
    write(&source, "/// Uses §FS-001-example.\nfinal class Use {}\n");
    (spec, source)
}

fn references(
    child: &mut Child,
    stdin: &mut ChildStdin,
    receiver: &mpsc::Receiver<Value>,
    id: i64,
    spec: &Path,
) -> Vec<Value> {
    send_message(
        stdin,
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "textDocument/references",
            "params": {
                "textDocument": { "uri": file_uri(spec) },
                "position": { "line": 0, "character": 8 },
                "context": { "includeDeclaration": false }
            }
        }),
    );
    recv_response_or_panic(receiver, child, id)["result"]
        .as_array()
        .cloned()
        .unwrap_or_default()
}

fn cites(locations: &[Value], path: &Path) -> bool {
    let uri = file_uri(path);
    locations
        .iter()
        .any(|location| location["uri"].as_str() == Some(uri.as_str()))
}

fn stop_server(
    child: &mut Child,
    stdin: &mut ChildStdin,
    receiver: &mpsc::Receiver<Value>,
    id: i64,
) {
    send_message(
        stdin,
        json!({ "jsonrpc": "2.0", "id": id, "method": "shutdown", "params": null }),
    );
    recv_response_or_panic(receiver, child, id);
    send_message(stdin, json!({ "jsonrpc": "2.0", "method": "exit" }));
}

#[test]
fn a_temporarily_invalid_config_keeps_its_snapshot_while_other_projects_refresh() {
    let base = std::env::temp_dir().join(format!("grund-lsp-retain-{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    let first = base.join("first");
    let second = base.join("second");
    let (first_spec, first_source) = write_project(&first, "FirstUse");
    let (second_spec, second_source) = write_project(&second, "SecondUse");

    let (mut child, mut stdin, receiver) =
        start_server_with_workspace_folders(&base, &[&first, &second]);
    assert!(cites(
        &references(&mut child, &mut stdin, &receiver, 2, &first_spec),
        &first_source
    ));

    write(
        &first.join("grund.toml"),
        "grund_config_version = 1\n[scan\n",
    );
    write(&second_source, "final class SecondUse {}\n");
    send_message(
        &mut stdin,
        json!({
            "jsonrpc": "2.0",
            "method": "workspace/didChangeWatchedFiles",
            "params": { "changes": [] }
        }),
    );

    assert!(
        cites(
            &references(&mut child, &mut stdin, &receiver, 3, &first_spec),
            &first_source
        ),
        "the invalid project's last good snapshot stays available"
    );
    assert!(
        !cites(
            &references(&mut child, &mut stdin, &receiver, 4, &second_spec),
            &second_source
        ),
        "a neighboring healthy project still refreshes"
    );

    stop_server(&mut child, &mut stdin, &receiver, 5);
    drop(stdin);
    wait_for_exit(&mut child);
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn a_scanned_sibling_does_not_give_another_project_ownership() {
    let base = std::env::temp_dir().join(format!("grund-lsp-exact-owner-{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    let first = base.join("first");
    let second = base.join("nested/second");
    let shared = base.join("shared");
    write(
        &first.join("grund.toml"),
        "grund_config_version = 1\n[scan]\ninclude = [\"docs\", \"../shared\"]\n\
         extensions = [\"md\"]\n",
    );
    write(
        &first.join("docs/FS-001-example.md"),
        "# FS-001-example: Example\n\nLead.\n",
    );
    let user = shared.join("user.md");
    write(&user, "See §FS-001-example.\n");
    write(
        &second.join("grund.toml"),
        "grund_config_version = 1\n[scan]\ninclude = [\"../../shared\"]\n\
         extensions = [\"java\"]\n",
    );
    write(&shared.join("Noise.java"), "final class Noise {}\n");

    let (mut child, mut stdin, receiver) =
        start_server_with_workspace_folders(&base, &[&first, &second]);
    let hover = hover_result(&mut stdin, &receiver, &mut child, 2, &file_uri(&user), 0, 6);
    assert!(
        hover["contents"]["value"]
            .as_str()
            .is_some_and(|body| body.contains("FS-001-example")),
        "only the project that actually scanned the document may own it: {hover}"
    );

    stop_server(&mut child, &mut stdin, &receiver, 3);
    drop(stdin);
    wait_for_exit(&mut child);
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn two_external_scan_claims_are_rejected_as_ambiguous() {
    let base =
        std::env::temp_dir().join(format!("grund-lsp-ambiguous-owner-{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    let first = base.join("first");
    let second = base.join("nested/second");
    let shared = base.join("shared");
    write(
        &first.join("grund.toml"),
        "grund_config_version = 1\n[scan]\ninclude = [\"docs\", \"../shared\"]\n\
         extensions = [\"md\"]\n",
    );
    write(
        &first.join("docs/FS-001-example.md"),
        "# FS-001-example: First\n\nFirst project body.\n",
    );
    write(
        &second.join("grund.toml"),
        "grund_config_version = 1\n[scan]\ninclude = [\"docs\", \"../../shared\"]\n\
         extensions = [\"md\"]\n",
    );
    write(
        &second.join("docs/FS-001-example.md"),
        "# FS-001-example: Second\n\nSecond project body.\n",
    );
    let user = shared.join("FS-002-user.md");
    write(&user, "# FS-002-user: User\n\nSee §FS-001-example.\n");

    let (mut child, mut stdin, receiver) =
        start_server_with_workspace_folders(&base, &[&first, &second]);
    let hover = hover_result(&mut stdin, &receiver, &mut child, 2, &file_uri(&user), 2, 6);
    assert!(
        hover.is_null(),
        "an external file scanned by two independent namespaces has no arbitrary owner: {hover}"
    );

    stop_server(&mut child, &mut stdin, &receiver, 3);
    drop(stdin);
    wait_for_exit(&mut child);
    let _ = fs::remove_dir_all(&base);
}

#[test]
#[cfg(unix)]
fn an_external_unreadable_file_still_publishes_its_scan_error() {
    let base = std::env::temp_dir().join(format!("grund-lsp-external-io-{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    let root = base.join("repo");
    let shared = base.join("shared");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n[scan]\ninclude = [\"../shared\"]\nextensions = [\"md\"]\n",
    );
    fs::create_dir_all(&shared).expect("create shared include");
    symlink("missing.md", shared.join("broken.md")).expect("create broken symlink");

    let (mut child, mut stdin, receiver) = start_server_with_workspace_folders(&root, &[&root]);
    let diagnostics = recv_diagnostics(&receiver, &mut child, "broken.md");
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic["code"].as_str() == Some("io")),
        "the external scan error must not be discarded for lacking a scanned-file owner: {diagnostics:?}"
    );

    stop_server(&mut child, &mut stdin, &receiver, 2);
    drop(stdin);
    wait_for_exit(&mut child);
    let _ = fs::remove_dir_all(&base);
}

/// The next `publishDiagnostics` for exactly `uri` that `accept` takes, or a
/// panic naming what was waited for — unlike `recv_diagnostics`, an empty
/// publish counts, which is how a cleared config diagnostic arrives.
fn recv_publish_for(
    receiver: &mpsc::Receiver<Value>,
    child: &mut Child,
    uri: &str,
    waiting_for: &str,
    accept: impl Fn(&[Value]) -> bool,
) -> Vec<Value> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        match receiver.recv_timeout(remaining) {
            Ok(message)
                if message.get("method").and_then(Value::as_str)
                    == Some("textDocument/publishDiagnostics")
                    && message["params"]["uri"].as_str() == Some(uri) =>
            {
                let diagnostics = message["params"]["diagnostics"]
                    .as_array()
                    .cloned()
                    .unwrap_or_default();
                if accept(&diagnostics) {
                    return diagnostics;
                }
            }
            Ok(_) => {}
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                let mut stderr = String::new();
                if let Some(child_stderr) = child.stderr.as_mut() {
                    use std::io::Read;
                    let _ = child_stderr.read_to_string(&mut stderr);
                }
                panic!("no publish for {uri} with {waiting_for}; server stderr: {stderr}");
            }
        }
    }
}

const BROKEN_CONFIG: &str = "grund_config_version = 1\n[scan\n";

/// §FS-lsp.2.2.4.1: the load error `grund check` prints for `[scan` on line 2,
/// as one `error` from `grund` on `grund.toml` at line index 1.
fn assert_config_diagnostic(diagnostics: &[Value]) {
    assert_eq!(
        diagnostics.len(),
        1,
        "one config diagnostic: {diagnostics:?}"
    );
    let diagnostic = &diagnostics[0];
    assert_eq!(diagnostic["severity"], 1, "an error: {diagnostic}");
    assert_eq!(diagnostic["source"], "grund", "from grund: {diagnostic}");
    assert_eq!(
        diagnostic["range"]["start"]["line"], 1,
        "at line 2: {diagnostic}"
    );
    let message = diagnostic["message"].as_str().unwrap_or_default();
    assert!(
        message.contains("expected `key = value`"),
        "the CLI's error text: {message}"
    );
}

fn changed_watched_files(stdin: &mut ChildStdin) {
    send_message(
        stdin,
        json!({
            "jsonrpc": "2.0",
            "method": "workspace/didChangeWatchedFiles",
            "params": { "changes": [] }
        }),
    );
}

#[test]
fn a_config_that_stops_loading_is_published_on_grund_toml_until_it_loads_again() {
    let base = std::env::temp_dir().join(format!("grund-lsp-config-diag-{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    let root = base.join("project");
    let (spec, source) = write_project(&root, "Use");
    let config = root.join("grund.toml");
    let healthy = fs::read_to_string(&config).expect("read config");
    let config_uri = file_uri(&config);

    let (mut child, mut stdin, receiver) = start_server_with_workspace_folders(&base, &[&root]);
    assert!(
        cites(
            &references(&mut child, &mut stdin, &receiver, 2, &spec),
            &source
        ),
        "the healthy config's scan is the last good one before the config breaks"
    );

    write(&config, BROKEN_CONFIG);
    changed_watched_files(&mut stdin);
    let diagnostics = recv_publish_for(
        &receiver,
        &mut child,
        &config_uri,
        "the config load error",
        |diagnostics| !diagnostics.is_empty(),
    );
    assert_config_diagnostic(&diagnostics);
    let message = diagnostics[0]["message"].as_str().unwrap_or_default();
    assert!(
        message.contains("last good scan"),
        "says the other diagnostics are from the last good scan: {message}"
    );

    write(&config, &healthy);
    changed_watched_files(&mut stdin);
    recv_publish_for(
        &receiver,
        &mut child,
        &config_uri,
        "an empty publish clearing the config error",
        <[Value]>::is_empty,
    );

    stop_server(&mut child, &mut stdin, &receiver, 3);
    drop(stdin);
    wait_for_exit(&mut child);
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn a_server_started_on_a_broken_config_publishes_it_on_grund_toml() {
    let base = std::env::temp_dir().join(format!("grund-lsp-config-start-{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    let root = base.join("project");
    write_project(&root, "Use");
    let config = root.join("grund.toml");
    write(&config, BROKEN_CONFIG);
    let config_uri = file_uri(&config);

    let (mut child, mut stdin, receiver) = start_server_with_workspace_folders(&base, &[&root]);

    let diagnostics = recv_publish_for(
        &receiver,
        &mut child,
        &config_uri,
        "the config load error at startup",
        |diagnostics| !diagnostics.is_empty(),
    );
    assert_config_diagnostic(&diagnostics);

    stop_server(&mut child, &mut stdin, &receiver, 2);
    drop(stdin);
    wait_for_exit(&mut child);
    let _ = fs::remove_dir_all(&base);
}
