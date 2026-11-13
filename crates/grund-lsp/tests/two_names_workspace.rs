//! Real-server: a server opened at a workspace root tells an editor a member's finding at the name
//! it speaks of, because the member's table holds that name. A place finding stands at its own
//! spelling, and a content finding reaches each open name once (§FS-scan.identity.findings,
//! §FS-lsp.1.1, §AR-scanner.1.8).
//! Fixture builders retain their temporary-root owner (§FS-repository-maintenance.4.1).
#![cfg(unix)]

mod support;

use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;
use support::*;

const ROOT: &str = "grund_config_version = 1\nproject_name = \"root\"\n\
    [reference]\nstrict = true\n[id]\nformat = \"{kind}-{slug}\"\n\
    [scan]\ninclude = [\"docs\"]\n\
    [workspace]\nmembers = [\"apps/api\"]\ninclude_root = true\n";
const MEMBER: &str = "grund_config_version = 2\nproject_name = \"api\"\n\
    [schema.sources]\nlanguages = [\"markdown\", \"rust\"]\n\
    [schema.kinds.FS]\nfolders = [\"docs/fs\"]\nindex = false\n\
    [schema.kinds.AR]\nfolders = [\"docs/ar\"]\nindex = false\n";

/// A workspace whose one member holds `name`, with `text` in it, and whose root reads `docs/`.
fn workspace(case: &str, name: &str, text: &str) -> TemporaryRoot {
    let root = test_root(case);
    let write = |path: &str, text: &str| {
        let path = root.join(path);
        fs::create_dir_all(path.parent().expect("a parent")).expect("create the parent");
        fs::write(path, text).expect("write the file");
    };
    write("grund.toml", ROOT);
    write("docs/r.md", "root\n");
    write("apps/api/grund.toml", MEMBER);
    write(&format!("apps/api/{name}"), text);
    root
}

/// The URI an editor opens `path` under, the link itself.
fn spelled_uri(path: &Path) -> String {
    url::Url::from_file_path(path)
        .expect("file uri")
        .to_string()
}

fn open(stdin: &mut impl std::io::Write, path: &Path, text: &str) {
    send_message(
        stdin,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {"textDocument": {
                "uri": spelled_uri(path),
                "languageId": "markdown",
                "version": 1,
                "text": text,
            }},
        }),
    );
}

fn codes(diagnostics: &[Value], code: &str) -> usize {
    diagnostics.iter().filter(|d| d["code"] == code).count()
}

/// The last publish of each URI before the server answers `shutdown`, which it handles after every
/// notification sent before it. Then the server exits.
fn shown_then_exit(
    stdin: &mut impl std::io::Write,
    receiver: &mpsc::Receiver<Value>,
    child: &mut support::Child,
) -> BTreeMap<String, Vec<Value>> {
    send_message(
        stdin,
        json!({"jsonrpc": "2.0", "id": 2, "method": "shutdown"}),
    );
    let mut shown = BTreeMap::new();
    loop {
        let message = receiver
            .recv_timeout(Duration::from_secs(10))
            .expect("the server answers shutdown");
        if message["id"] == 2 {
            break;
        }
        if message["method"] == "textDocument/publishDiagnostics" {
            let uri = message["params"]["uri"].as_str().expect("uri").to_string();
            let diagnostics = message["params"]["diagnostics"].as_array();
            shown.insert(uri, diagnostics.cloned().unwrap_or_default());
        }
    }
    send_message(stdin, json!({"jsonrpc": "2.0", "method": "exit"}));
    wait_for_exit(child);
    shown
}

/// Below the member's kind homes a link in `docs/ar` earns the place finding the target does not,
/// and both names show the content finding once.
#[test]
fn a_members_place_finding_stands_at_its_own_spelling() {
    let text = "# FS-x: X\n\nCites \u{a7}FS-x and \u{a7}FS-gone.\n";
    let root = workspace("two-names-ws-place", "docs/fs/x.md", text);
    let target = root.join("apps/api/docs/fs/x.md");
    let link = root.join("apps/api/docs/ar/x.md");
    fs::create_dir_all(link.parent().expect("a parent")).expect("create the home");
    symlink("../fs/x.md", &link).expect("link");
    let (mut child, mut stdin, receiver) = start_server(&root);
    let at_link = recv_diagnostics(&receiver, &mut child, "apps/api/docs/ar/x.md");
    assert_eq!(codes(&at_link, "misplaced-declaration"), 1, "{at_link:?}");

    open(&mut stdin, &link, text);
    open(&mut stdin, &target, text);
    let shown = shown_then_exit(&mut stdin, &receiver, &mut child);
    let at_link = &shown[&spelled_uri(&link)];
    assert_eq!(codes(at_link, "misplaced-declaration"), 1, "{at_link:?}");
    assert_eq!(codes(at_link, "dangling"), 1, "exactly once: {at_link:?}");
    let at_target = &shown[&spelled_uri(&target)];
    assert_eq!(
        codes(at_target, "misplaced-declaration"),
        0,
        "{at_target:?}"
    );
    assert_eq!(
        codes(at_target, "dangling"),
        1,
        "exactly once: {at_target:?}"
    );
}

/// A link listed first in the member's own home, and the file's own name, each show it once.
#[test]
fn a_members_content_finding_reaches_each_open_name_once() {
    let text = "# FS-z: Z\n\nCites \u{a7}FS-z and \u{a7}FS-gone.\n";
    let root = workspace("two-names-ws-content", "docs/fs/z-real.md", text);
    let target = root.join("apps/api/docs/fs/z-real.md");
    let link = root.join("apps/api/docs/fs/a-link.md");
    symlink("z-real.md", &link).expect("link");
    let (mut child, mut stdin, receiver) = start_server(&root);
    let first = recv_diagnostics(&receiver, &mut child, "apps/api/docs/fs/a-link.md");
    assert_eq!(codes(&first, "dangling"), 1, "{first:?}");

    open(&mut stdin, &link, text);
    open(&mut stdin, &target, text);
    let shown = shown_then_exit(&mut stdin, &receiver, &mut child);
    for name in [&link, &target] {
        let at = &shown[&spelled_uri(name)];
        assert_eq!(codes(at, "dangling"), 1, "exactly once at {name:?}: {at:?}");
        assert_eq!(at[0]["range"]["start"]["line"], json!(2), "at its line");
    }
}
