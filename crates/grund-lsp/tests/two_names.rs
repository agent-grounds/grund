//! Real-server: a file reached by two names is one file in the editor as well as in the report. A
//! finding about what it holds reaches whichever name is open, and a refused file takes no on-type
//! edit under any name, its own included (§FS-scan.identity.findings, §FS-scan.identity.buffers).
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

const CONFIG: &str = "grund_config_version = 2\nproject_name = \"names\"\n\
    [schema.sources]\nlanguages = [\"markdown\", \"rust\"]\n\
    [schema.kinds.FS]\nfolders = [\"docs/fs\"]\nindex = false\n";

/// The URI an editor opens `path` under, the link itself: the suite's `file_uri` resolves it, which
/// would open the target instead and prove nothing about the second name.
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

/// What the editor ends up showing, per URI: the last publish of each before the server answers
/// `shutdown`, which it handles after every notification sent before it. Then the server exits.
fn shown_then_exit(
    stdin: &mut impl std::io::Write,
    receiver: &mpsc::Receiver<Value>,
    child: &mut std::process::Child,
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

#[test]
fn a_content_finding_reaches_the_second_name_once_it_is_open() {
    let root = test_root("two-names-content");
    fs::write(root.join("grund.toml"), CONFIG).expect("write config");
    fs::create_dir_all(root.join("docs/fs")).expect("create home");
    fs::create_dir_all(root.join("notes")).expect("create notes");
    let text = "# FS-a: A\n\nCites \u{a7}FS-gone and \u{a7}FS-a.\n";
    fs::write(root.join("docs/fs/a.md"), text).expect("write file");
    let link = root.join("notes/link.md");
    symlink("../docs/fs/a.md", &link).expect("link");
    let (mut child, mut stdin, receiver) = start_server(&root);

    let first = recv_diagnostics(&receiver, &mut child, "docs/fs/a.md");
    assert_eq!(codes(&first, "dangling"), 1, "{first:?}");

    open(&mut stdin, &link, text);
    let shown = shown_then_exit(&mut stdin, &receiver, &mut child);
    let second = &shown[&spelled_uri(&link)];
    assert_eq!(codes(second, "dangling"), 1, "exactly once: {second:?}");
    assert_eq!(second[0]["range"]["start"]["line"], json!(2), "at its line");
    let at_first = &shown[&spelled_uri(&root.join("docs/fs/a.md"))];
    assert_eq!(codes(at_first, "dangling"), 1, "exactly once: {at_first:?}");
}

/// Sites in another file are not another name: each file keeps what it published before.
#[test]
fn a_duplicate_between_two_files_stays_where_it_was() {
    let root = test_root("two-names-duplicate");
    fs::write(root.join("grund.toml"), CONFIG).expect("write config");
    fs::create_dir_all(root.join("docs/fs")).expect("create home");
    let text = "# FS-a: A\n\nCites \u{a7}FS-a.\n";
    fs::write(root.join("docs/fs/a.md"), text).expect("write first");
    fs::write(root.join("docs/fs/b.md"), text).expect("write second");
    let (mut child, mut stdin, receiver) = start_server(&root);

    let first = recv_diagnostics(&receiver, &mut child, "docs/fs/a.md");
    assert_eq!(codes(&first, "duplicate"), 1, "{first:?}");
    open(&mut stdin, &root.join("docs/fs/a.md"), text);
    open(&mut stdin, &root.join("docs/fs/b.md"), text);
    let shown = shown_then_exit(&mut stdin, &receiver, &mut child);
    let second = shown
        .get(&spelled_uri(&root.join("docs/fs/b.md")))
        .cloned()
        .unwrap_or_default();
    assert_eq!(codes(&second, "duplicate"), 0, "{second:?}");
}

/// Where the file sits is said at the spelling that sits there, however many of them are open.
#[test]
fn a_place_finding_stays_at_its_own_spelling() {
    let root = test_root("two-names-place");
    let config = "grund_config_version = 2\nproject_name = \"names\"\n\
        [schema.sources]\nlanguages = [\"markdown\"]\n\
        [schema.kinds.FS]\nfolders = [\"docs/fs\"]\nindex = false\n\
        [schema.kinds.AR]\nfolders = [\"docs/ar\"]\nindex = false\n";
    fs::write(root.join("grund.toml"), config).expect("write config");
    fs::create_dir_all(root.join("docs/fs")).expect("create fs");
    fs::create_dir_all(root.join("docs/ar")).expect("create ar");
    let text = "# FS-x: X\n\nCites \u{a7}FS-x.\n";
    fs::write(root.join("docs/fs/x.md"), text).expect("write file");
    let link = root.join("docs/ar/x.md");
    symlink("../fs/x.md", &link).expect("link");
    let (mut child, mut stdin, receiver) = start_server(&root);

    let at_link = recv_diagnostics(&receiver, &mut child, "docs/ar/x.md");
    assert_eq!(codes(&at_link, "misplaced-declaration"), 1, "{at_link:?}");
    let target = root.join("docs/fs/x.md");
    open(&mut stdin, &target, text);
    open(&mut stdin, &link, text);
    let shown = shown_then_exit(&mut stdin, &receiver, &mut child);
    let again = &shown[&spelled_uri(&link)];
    assert_eq!(codes(again, "misplaced-declaration"), 1, "{again:?}");
    let at_target = shown
        .get(&spelled_uri(&target))
        .cloned()
        .unwrap_or_default();
    assert_eq!(
        codes(&at_target, "misplaced-declaration"),
        0,
        "{at_target:?}"
    );
}

/// The edits the server offers for a trigger typed into the buffer at `path`.
fn on_type(
    stdin: &mut impl std::io::Write,
    receiver: &mpsc::Receiver<Value>,
    id: i64,
    path: &Path,
    line: &str,
) -> usize {
    open(stdin, path, line);
    send_message(
        stdin,
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "textDocument/onTypeFormatting",
            "params": {
                "textDocument": {"uri": spelled_uri(path)},
                "position": {"line": 0, "character": line.trim_end().len()},
                "ch": "a",
                "options": {"tabSize": 4, "insertSpaces": true},
            },
        }),
    );
    let response = recv_response(receiver, id).expect("on-type response");
    response["result"].as_array().map_or(0, Vec::len)
}

/// No name escapes the refusal: not either link, and not the target's own name, which no finding
/// lists. A file with one name is the control that the trigger converts at all.
#[test]
fn a_refused_file_takes_no_on_type_edit_under_any_name() {
    let root = test_root("two-names-refused");
    fs::write(root.join("grund.toml"), CONFIG).expect("write config");
    fs::create_dir_all(root.join("docs/fs")).expect("create home");
    fs::write(
        root.join("docs/fs/a.md"),
        "# FS-a: A\n\nCites \u{a7}FS-a.\n",
    )
    .expect("write file");
    fs::write(root.join("docs/other.md"), "\n").expect("write control");
    fs::write(root.join("LICENSE"), "\n").expect("write license");
    symlink("../LICENSE", root.join("docs/license.md")).expect("link md");
    symlink("../LICENSE", root.join("docs/license.rs")).expect("link rs");
    let (mut child, mut stdin, receiver) = start_server(&root);
    let _ = recv_diagnostics(&receiver, &mut child, "docs/license.md");

    let line = "$$FS-a";
    let control = on_type(&mut stdin, &receiver, 11, &root.join("docs/other.md"), line);
    assert_eq!(control, 1, "the trigger converts in a file with one name");
    for (id, name) in [
        (12, "docs/license.md"),
        (13, "docs/license.rs"),
        (14, "LICENSE"),
    ] {
        let edits = on_type(&mut stdin, &receiver, id, &root.join(name), line);
        assert_eq!(edits, 0, "{name}");
    }
    shown_then_exit(&mut stdin, &receiver, &mut child);
}
