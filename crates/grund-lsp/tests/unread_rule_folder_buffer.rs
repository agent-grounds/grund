//! Real-server regression for a buffer in a rule folder the run reads nothing else of: a buffer
//! with no file behind it is a file of the run (§FS-scan.text.buffer-only), so it counts for the
//! folder's `unread-scan-root` verdict exactly as the saved file would
//! (§FS-scan.checks.unread-scan-root.rule-folder), for and against.

mod support;

use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;
use support::*;

/// A version-2 project that reads `.md` as `json`, whose rule folder is `rules`.
const MD_AS_JSON: &str = "grund_config_version = 2\nproject_name = \"c\"\n\n\
     [schema.sources]\nlanguages = [\"markdown\", \"json\", \"rust\"]\n\n\
     [schema.sources.extensions]\nmd = \"json\"\n\n\
     [schema.kinds.FS]\nfiles = [\"src/a.rs\"]\n\n\
     [schema.kinds.RULE]\nfolders = [\"rules\"]\nform = \"rule\"\n";
const SPEC: &str = "/// FS-a: A spec\n///\n/// Body \u{a7}FS-a.\npub fn a() {}\n";
const RULE_MD: &str = "# RULE-fresh: Each FS must cite at least one RULE.\n\nBody.\n";
const RULE_RS: &str =
    "/// RULE-fresh: Each FS must cite at least one RULE.\n///\n/// Body.\npub fn r() {}\n";

/// The last publication for each file until the server falls quiet, the answer to the last
/// change it took. A file the server never published for is absent, not empty.
fn settled(receiver: &mpsc::Receiver<Value>) -> BTreeMap<String, Vec<Value>> {
    let mut last = BTreeMap::new();
    while let Ok(message) = receiver.recv_timeout(Duration::from_millis(1500)) {
        if message.get("method").and_then(Value::as_str) == Some("textDocument/publishDiagnostics")
        {
            let diagnostics = message["params"]["diagnostics"]
                .as_array()
                .cloned()
                .unwrap_or_default();
            let uri = message["params"]["uri"].as_str().unwrap_or_default();
            last.insert(uri.to_string(), diagnostics);
        }
    }
    last
}

/// What the snapshot published for the file whose URI ends in `suffix`, by `code`.
fn messages(published: &BTreeMap<String, Vec<Value>>, suffix: &str, code: &str) -> Vec<String> {
    published
        .iter()
        .filter(|(uri, _)| uri.ends_with(suffix))
        .flat_map(|(_, diagnostics)| diagnostics)
        .filter(|diagnostic| diagnostic["code"].as_str() == Some(code))
        .filter_map(|diagnostic| diagnostic["message"].as_str().map(String::from))
        .collect()
}

/// What a server over `root` publishes once it has settled, with each of `buffers` open, in
/// order, as text no file on disk holds.
fn published_with(root: &Path, buffers: &[(&str, &str, &str)]) -> BTreeMap<String, Vec<Value>> {
    let (mut child, mut stdin, receiver) = start_server(root);
    let mut published = settled(&receiver);
    for (file, language, text) in buffers {
        send_message(
            &mut stdin,
            json!({"jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {
                "textDocument": {"uri": file_uri(&root.join(file)), "languageId": language,
                                 "version": 1, "text": text}}}),
        );
        published.extend(settled(&receiver));
    }
    send_message(
        &mut stdin,
        json!({"jsonrpc": "2.0", "id": 2, "method": "shutdown", "params": null}),
    );
    recv_response_or_panic(&receiver, &mut child, 2);
    send_message(
        &mut stdin,
        json!({"jsonrpc": "2.0", "method": "exit", "params": null}),
    );
    drop(stdin);
    wait_for_exit(&mut child);
    published
}

/// A project of `config` and `src/a.rs`, with `dirs` made and `files` written.
fn project(name: &str, config: &str, dirs: &[&str], files: &[(&str, &str)]) -> std::path::PathBuf {
    let root = test_root(name);
    fs::write(root.join("grund.toml"), config).expect("write config");
    fs::create_dir_all(root.join("src")).expect("create src");
    fs::write(root.join("src/a.rs"), SPEC).expect("write spec");
    for dir in dirs {
        fs::create_dir_all(root.join(dir)).expect("create dir");
    }
    for (file, text) in files {
        let path = root.join(file);
        fs::create_dir_all(path.parent().expect("a file has a parent")).expect("create parent");
        fs::write(path, text).expect("write file");
    }
    root
}

/// §FS-scan.checks.unread-scan-root.rule-folder: a folder holding any file the run read earns
/// nothing, and the buffer is read: the rule in it is enforced in the same snapshot.
#[test]
fn a_read_buffer_clears_the_rule_folder_it_enforces_a_rule_from() {
    let root = project(
        "unread_rule_folder_read_buffer",
        MD_AS_JSON,
        &[],
        &[("rules/rules.md", RULE_MD)],
    );
    let before = published_with(&root, &[]);
    assert_eq!(
        messages(&before, "grund.toml", "unread-scan-root").len(),
        1,
        "the folder is unread before the buffer: {before:#?}"
    );
    let after = published_with(&root, &[("rules/new.rs", "rust", RULE_RS)]);
    assert_eq!(
        messages(&after, "grund.toml", "unread-scan-root"),
        Vec::<String>::new(),
        "§FS-scan.text.buffer-only: {after:#?}"
    );
    assert_eq!(
        messages(&after, "src/a.rs", "missing-citation"),
        vec!["FS-a must cite RULE (RULE-fresh); add such a citation within it".to_string()],
        "the rule in the buffer is in force, so the snapshot read it: {after:#?}"
    );
    let _ = fs::remove_dir_all(root);
}

/// §AR-scanner.1.13: a buffer is answered as the saved file would be, so one its extension
/// refuses makes an otherwise empty folder unread, in the saved file's words.
#[test]
fn a_refused_buffer_alone_in_a_rule_folder_reads_as_the_saved_file() {
    let root = project(
        "unread_rule_folder_refused_buffer",
        MD_AS_JSON,
        &["rules"],
        &[],
    );
    let buffer = published_with(&root, &[("rules/new.md", "markdown", RULE_MD)]);
    let saved_root = project(
        "unread_rule_folder_refused_saved",
        MD_AS_JSON,
        &[],
        &[("rules/new.md", RULE_MD)],
    );
    let saved = published_with(&saved_root, &[]);
    let saved = messages(&saved, "grund.toml", "unread-scan-root");
    assert_eq!(saved.len(), 1, "the saved file is refused: {saved:#?}");
    assert_eq!(
        messages(&buffer, "grund.toml", "unread-scan-root"),
        saved,
        "§FS-scan.text.buffer-only: {buffer:#?}"
    );
    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_dir_all(saved_root);
}

/// §FS-scan.checks.unread-scan-root.rule-folder: a subdirectory `exclude` prunes earns nothing,
/// and the walk would never yield the saved file, whatever its name test would answer. The
/// second buffer shows the server took the first.
#[test]
fn a_refused_buffer_in_an_excluded_subdirectory_earns_nothing() {
    let config = MD_AS_JSON.replace(
        "languages = [\"markdown\", \"json\", \"rust\"]\n",
        "languages = [\"markdown\", \"json\", \"rust\"]\nexclude = [\"drafts\"]\n",
    );
    let root = project(
        "unread_rule_folder_excluded_buffer",
        &config,
        &["rules"],
        &[],
    );
    let published = published_with(
        &root,
        &[
            ("rules/drafts/new.md", "markdown", RULE_MD),
            (
                "src/b.rs",
                "rust",
                "/// FS-b: Another\n///\n/// Body.\npub fn b() {}\n",
            ),
        ],
    );
    assert!(
        published.keys().any(|uri| uri.ends_with("src/b.rs")),
        "the server took the buffers: {published:#?}"
    );
    assert_eq!(
        messages(&published, "grund.toml", "unread-scan-root"),
        Vec::<String>::new(),
        "§FS-scan.scope.roots: {published:#?}"
    );
    let _ = fs::remove_dir_all(root);
}

/// §FS-scan.text.buffer-only: a buffer in a rule folder no directory on disk stands behind, which
/// a version-1 `include` does not reach, still answers as the saved file would.
#[test]
fn a_refused_buffer_in_a_rule_folder_not_on_disk_reads_as_the_saved_file() {
    let config = "[id]\nformat = \"{kind}-{slug}\"\n\n[[kinds]]\nkind = \"FS\"\nfile = \"src/a.rs\"\n\n\
                  [[kinds]]\nkind = \"RULE\"\nfolder = \"rules\"\nrules = true\n\n\
                  [scan]\ninclude = [\"src\"]\nextensions = [\"rs\"]\n";
    let root = project("unread_rule_folder_absent_buffer", config, &[], &[]);
    let buffer = published_with(&root, &[("rules/new.md", "markdown", RULE_MD)]);
    let saved_root = project(
        "unread_rule_folder_absent_saved",
        config,
        &[],
        &[("rules/new.md", RULE_MD)],
    );
    let saved = published_with(&saved_root, &[]);
    let saved = messages(&saved, "grund.toml", "unread-scan-root");
    assert_eq!(saved.len(), 1, "the saved file is refused: {saved:#?}");
    assert_eq!(
        messages(&buffer, "grund.toml", "unread-scan-root"),
        saved,
        "§FS-scan.text.buffer-only: {buffer:#?}"
    );
    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_dir_all(saved_root);
}
