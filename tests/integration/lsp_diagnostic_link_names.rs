//! §FS-lsp.1.1, §FS-scan.identity.link-path, §AR-lsp.5.1.1: port of #1337's CLI-versus-stdio
//! reproducer. Expected paths are authored independently of both surfaces. URI decoding never
//! canonicalizes the descendant: doing so would erase the distinction this test must observe.
#![cfg(unix)]

#[path = "binaries.rs"]
mod binaries;
#[path = "../../crates/grund-lsp/tests/support/mod.rs"]
mod support;

use serde_json::json;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

const CONFIG: &str = r#"grund_config_version = 2
project_name = "p"
[schema.sources]
languages = ["markdown"]
exclude = ["private"]
[schema.kinds.FS]
folders = ["docs/fs"]
index = false
[schema.kinds.AR]
folders = ["docs/ar"]
index = false
"#;

const DECODE: &str =
    "stream did not contain valid UTF-8; make the path readable, or keep it out of the scan";

struct Fixture(PathBuf);

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove diagnostic-link fixture");
    }
}

fn write(path: &Path, bytes: impl AsRef<[u8]>) {
    fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
    fs::write(path, bytes).expect("write fixture");
}

/// All initialized diagnostics before the ordered shutdown response, decoded without realpath.
fn published(root: &Path) -> Vec<(PathBuf, u64, String, u64)> {
    let _ = support::SERVER_BINARY.set(binaries::grund_lsp());
    // support::file_uri resolves the entire path, so supply the root's spelling ourselves.
    let uri = url::Url::from_file_path(root).expect("root URI");
    let (mut child, mut stdin, receiver) = support::start_server_with_initialize(
        root,
        json!({
            "processId": std::process::id(), "capabilities": {},
            "workspaceFolders": [{"uri": uri.as_str(), "name": "p"}]
        }),
    );
    support::send_message(
        &mut stdin,
        json!({"jsonrpc": "2.0", "id": 2, "method": "shutdown", "params": null}),
    );
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut findings = Vec::new();
    loop {
        let message = receiver
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .expect("initialized diagnostics and shutdown response");
        if message["id"].as_i64() == Some(2) {
            break;
        }
        if message["method"] != "textDocument/publishDiagnostics" {
            continue;
        }
        let path = url::Url::parse(message["params"]["uri"].as_str().expect("URI"))
            .expect("parse URI")
            .to_file_path()
            .expect("file URI");
        for diagnostic in message["params"]["diagnostics"].as_array().expect("array") {
            findings.push((
                path.clone(),
                diagnostic["range"]["start"]["line"].as_u64().expect("line") + 1,
                diagnostic["message"]
                    .as_str()
                    .expect("message")
                    .lines()
                    .next()
                    .expect("first line")
                    .to_owned(),
                diagnostic["severity"].as_u64().expect("severity"),
            ));
        }
    }
    support::send_message(
        &mut stdin,
        json!({"jsonrpc": "2.0", "method": "exit", "params": null}),
    );
    support::wait_for_exit(&mut child);
    findings.sort();
    findings
}

fn assert_link_names(case: &str, member: bool, linked_root: bool, external_files: bool) {
    let base = support::fixture_base().join(format!(
        "grund-1337-{case}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    fs::create_dir_all(&base).expect("create fixture base");
    let fixture = Fixture(base.canonicalize().expect("physical fixture base"));
    let root = fixture.0.join("project");
    let owner = if member { root.join("m") } else { root.clone() };
    if member {
        write(
            &root.join("grund.toml"),
            "grund_config_version = 2\nproject_name = \"w\"\n\
             [schema.kinds.doc]\ncitable = false\nfolders = [\"docs\"]\n\
             [workspace]\nmembers = [\"m\"]\n",
        );
        write(&root.join("docs/notes.md"), "See \u{a7}p/FS-real.\n");
    }
    write(&owner.join("grund.toml"), CONFIG);
    write(
        &owner.join("docs/fs/FS-real.md"),
        "# FS-real: Real\n\nSee \u{a7}AR-real.\n",
    );
    let targets = if external_files {
        fixture.0.join("shared")
    } else {
        owner.join("private")
    };
    write(&targets.join("a.md"), "A note\nSee \u{a7}FS-missing.\n");
    write(&targets.join("bad.md"), b"\xff\xfe");
    write(
        &owner.join("private/ar/x.md"),
        "# AR-real: R\n\nSee \u{a7}AR-missing and \u{a7}FS-real.\n",
    );
    symlink(targets.join("a.md"), owner.join("docs/fs/link.md")).expect("file link");
    symlink(targets.join("bad.md"), owner.join("docs/fs/bad.md")).expect("decode link");
    symlink("../private/ar", owner.join("docs/ar")).expect("directory link");

    let output = Command::new(binaries::grund())
        .arg("check")
        .current_dir(&root)
        .output()
        .expect("CLI check");
    let cli = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(2), "{cli}");
    let prefix = if member { "m/" } else { "" };
    let mut expected_cli = vec![
        format!("error: {prefix}docs/fs/bad.md: {DECODE}"),
        format!("{prefix}docs/fs/link.md:2: error: unknown reference FS-missing"),
        format!("{prefix}docs/ar/x.md:3: error: unknown reference AR-missing"),
    ];
    expected_cli.sort();
    let mut actual_cli: Vec<_> = cli.lines().map(str::to_owned).collect();
    actual_cli.sort();
    assert_eq!(
        actual_cli, expected_cli,
        "CLI must first demonstrate the promised link names"
    );

    let mut expected = vec![
        (owner.join("docs/fs/bad.md"), 1, DECODE.to_owned(), 1),
        (
            owner.join("docs/fs/link.md"),
            2,
            "unknown reference FS-missing".to_owned(),
            1,
        ),
        (
            owner.join("docs/ar/x.md"),
            3,
            "unknown reference AR-missing".to_owned(),
            1,
        ),
    ];
    expected.sort();
    let opened_root = if linked_root {
        let alias = fixture.0.join("linked-project");
        symlink(&root, &alias).expect("link the project root");
        alias
    } else {
        root
    };
    assert_eq!(
        published(&opened_root),
        expected,
        "{case}: published paths, one-based lines, messages, severity"
    );
}

#[test]
#[ignore = "red until #1337 diagnostics preserve sole link names; implement removes this"]
fn issue_1337_stdio_keeps_sole_link_names_with_excluded_targets() {
    assert_link_names("excluded", false, false, false);
}

#[test]
#[ignore = "red until #1337 diagnostics preserve external file links; implement removes this"]
fn issue_1337_stdio_keeps_sole_file_links_to_external_targets() {
    assert_link_names("external-files", false, false, true);
}

#[test]
#[ignore = "red until #1337 diagnostics preserve member link names; implement removes this"]
fn issue_1337_stdio_uses_the_members_root_and_link_names() {
    assert_link_names("member", true, false, false);
}

#[test]
#[ignore = "red until #1337 resolving the root preserves descendant links; implement removes this"]
fn issue_1337_stdio_resolves_a_linked_root_without_erasing_descendant_names() {
    assert_link_names("linked-root", false, true, false);
}
