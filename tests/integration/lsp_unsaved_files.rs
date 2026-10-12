//! The editor judges what a save would write (§FS-scan.text.buffer), driven the
//! way an editor drives the server: open the files, hover the citation, and read
//! what the server published. Each tree is answered twice, once with the edit
//! held unsaved in the editor and once with the same bytes saved and nothing
//! open, and the two must be one (§FS-lsp.4.1). A new file the editor has not
//! saved is a file (§FS-scan.text.buffer-only), and a buffer is its file under
//! every spelling, a symlinked one included (§FS-scan.identity.buffers).
//! Every reader takes a file's text from one place, `sources` (§AR-system.2.13).

#[path = "binaries.rs"]
mod binaries;
#[path = "../../crates/grund-lsp/tests/support/mod.rs"]
mod support;

use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use support::{send_message, start_server, wait_for_exit};

/// One kind, `FS`, at `docs/`, scanning `docs` and whatever `extra` adds.
fn config(extra: &[&str]) -> String {
    let include: Vec<String> = ["docs"]
        .iter()
        .chain(extra)
        .map(|dir| format!("\"{dir}\""))
        .collect();
    format!(
        "grund_config_version = 1\n[reference]\nstrict = true\n[id]\nformat = \"{{kind}}-{{slug}}\"\n\
         [[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n\
         [scan]\ninclude = [{}]\n",
        include.join(", ")
    )
}

/// The citation every tree hovers: `<§>FS-a` on line 1 of `docs/use.md`.
const USE: &str = "See \u{a7}FS-a.\n";
const DECLARES: &str = "# FS-a: Alpha\n\nBody.\n";

/// One fixture tree under the worktree's own `target/`.
struct Tree(PathBuf);

impl Tree {
    fn new(name: &str, config: &str, files: &[(&str, &str)]) -> Self {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/lsp-unsaved-files")
            .join(format!("{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create fixture root");
        let tree = Self(root.canonicalize().expect("canonical fixture root"));
        tree.write("grund.toml", config);
        tree.write("docs/use.md", USE);
        for (path, body) in files {
            tree.write(path, body);
        }
        tree
    }

    fn write(&self, relative: &str, body: &str) {
        let path = self.0.join(relative);
        fs::create_dir_all(path.parent().expect("parent")).expect("create fixture parent");
        fs::write(path, body).expect("write fixture");
    }

    /// What the editor shows with `open` held as buffers, each under the spelling
    /// given: the diagnostics last published for each file, and the hover on the
    /// citation in `docs/use.md`.
    fn editor(&self, open: &[(&str, &str)]) -> Answers {
        let _ = support::SERVER_BINARY.set(binaries::grund_lsp());
        let (mut child, mut stdin, receiver) = start_server(&self.0);
        for (relative, text) in open {
            send_message(
                &mut stdin,
                json!({"jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {
                    "textDocument": {"uri": uri(&self.0.join(relative)), "version": 1,
                        "languageId": "markdown", "text": text}}}),
            );
        }
        // Messages are handled in order, so every diagnostic the opens published is
        // on the wire before the response to the hover sent after them.
        send_message(
            &mut stdin,
            json!({"jsonrpc": "2.0", "id": 2, "method": "textDocument/hover", "params": {
                "textDocument": {"uri": uri(&self.0.join("docs/use.md"))},
                "position": {"line": 0, "character": 5}}}),
        );
        let deadline = Instant::now() + Duration::from_secs(30);
        let mut diagnostics = BTreeMap::new();
        let hover = loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            let message = receiver
                .recv_timeout(remaining)
                .unwrap_or_else(|_| panic!("no hover response for {}", self.0.display()));
            if message.get("id").and_then(Value::as_i64) == Some(2) {
                break hover_text(&message["result"]);
            }
            if message.get("method").and_then(Value::as_str)
                == Some("textDocument/publishDiagnostics")
            {
                let file = self.relative(message["params"]["uri"].as_str().expect("uri"));
                let found: Vec<String> = message["params"]["diagnostics"]
                    .as_array()
                    .expect("diagnostics array")
                    .iter()
                    .map(|d| {
                        format!(
                            "{}: {}",
                            d["code"].as_str().unwrap_or(""),
                            d["message"].as_str().unwrap_or("")
                        )
                    })
                    .collect();
                diagnostics.insert(file, found);
            }
        };
        send_message(
            &mut stdin,
            json!({"jsonrpc": "2.0", "id": 3, "method": "shutdown", "params": null}),
        );
        send_message(
            &mut stdin,
            json!({"jsonrpc": "2.0", "method": "exit", "params": null}),
        );
        wait_for_exit(&mut child);
        diagnostics.retain(|_, found: &mut Vec<String>| !found.is_empty());
        Answers { diagnostics, hover }
    }

    /// `edited` held unsaved at `at`, then saved there with nothing open.
    fn unsaved_then_saved(
        &self,
        also_open: &[(&str, &str)],
        at: &str,
        edited: &str,
    ) -> (Answers, Answers) {
        let mut open = also_open.to_vec();
        open.push((at, edited));
        let unsaved = self.editor(&open);
        self.write(at, edited);
        let saved = self.editor(&[]);
        (unsaved, saved)
    }

    fn relative(&self, uri: &str) -> String {
        let path = url::Url::parse(uri)
            .ok()
            .and_then(|url| url.to_file_path().ok())
            .unwrap_or_else(|| panic!("diagnostic uri is not a file path: {uri}"));
        let path = fs::canonicalize(&path).unwrap_or(path);
        path.strip_prefix(&self.0)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/")
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// The URI of a path exactly as spelled, so a buffer opened through a symlink
/// reaches the server under the link's name.
fn uri(path: &Path) -> String {
    url::Url::from_file_path(path)
        .expect("absolute fixture path")
        .to_string()
}

fn hover_text(result: &Value) -> String {
    result["contents"]["value"]
        .as_str()
        .map(str::to_string)
        .unwrap_or_else(|| result.to_string())
}

/// What the editor shows about one tree: each file's diagnostics, then the hover.
#[derive(Debug, PartialEq)]
struct Answers {
    diagnostics: BTreeMap<String, Vec<String>>,
    hover: String,
}

impl Answers {
    fn clean_with_body(&self) -> bool {
        self.diagnostics.is_empty() && self.hover.contains("Body.")
    }

    /// Only the symlink tests, which are unix-only, ask this.
    #[cfg(unix)]
    fn broken_stub(&self) -> bool {
        self.diagnostics
            .get("docs/a.md")
            .is_some_and(|found| found.iter().any(|d| d.starts_with("broken-stub: ")))
    }
}

/// A stub and a new source file the editor has not saved: no diagnostic, and the
/// hover shows the body, as once the file is saved.
#[test]
fn an_unsaved_new_target_answers_as_saved() {
    let stub = "# FS-a: [a](../src/a.rs)\n";
    let tree = Tree::new("new-target", &config(&["src"]), &[("docs/a.md", stub)]);
    let source = "/// FS-a: Alpha\n///\n/// Body.\npub fn a() {}\n";
    let (unsaved, saved) = tree.unsaved_then_saved(&[("docs/a.md", stub)], "src/a.rs", source);
    assert!(saved.clean_with_body(), "the saved tree: {saved:?}");
    assert_eq!(
        unsaved, saved,
        "the unsaved new target against the saved one"
    );
}

/// `docs/a.md` stubs `../alias/a.md`, `alias -> notes`, and `notes/a.md` holds
/// `before`; the edit to `after` is opened as `at`, one spelling of that file.
#[cfg(unix)]
fn behind_a_symlink(
    name: &str,
    scanned: &[&str],
    before: &str,
    at: &str,
    after: &str,
) -> (Answers, Answers) {
    let stub = "# FS-a: [a](../alias/a.md)\n";
    let tree = Tree::new(
        name,
        &config(scanned),
        &[("docs/a.md", stub), ("notes/a.md", before)],
    );
    std::os::unix::fs::symlink("notes", tree.0.join("alias")).expect("create fixture symlink");
    tree.unsaved_then_saved(&[("docs/a.md", stub)], at, after)
}

/// An unsaved edit that adds the declaration to a target behind a symlink
/// repairs the stub at once, opened under the target's path or the stub's.
#[cfg(unix)]
#[test]
fn an_unsaved_edit_behind_a_symlink_repairs_the_stub_under_either_spelling() {
    for (name, scanned, at) in [
        ("adds-physical", &[][..], "notes/a.md"),
        ("adds-link", &["alias"][..], "alias/a.md"),
    ] {
        let (unsaved, saved) = behind_a_symlink(name, scanned, "No declaration.\n", at, DECLARES);
        assert!(saved.clean_with_body(), "{name}, the saved tree: {saved:?}");
        assert_eq!(
            unsaved, saved,
            "{name}: the unsaved edit at {at} against the saved one"
        );
    }
}

/// An unsaved edit that removes the declaration from a target behind a symlink
/// breaks the stub at once, opened under the target's path or the stub's.
#[cfg(unix)]
#[test]
fn an_unsaved_edit_behind_a_symlink_breaks_the_stub_under_either_spelling() {
    for (name, at) in [
        ("removes-physical", "notes/a.md"),
        ("removes-link", "alias/a.md"),
    ] {
        let (unsaved, saved) = behind_a_symlink(name, &[], DECLARES, at, "No declaration.\n");
        assert!(saved.broken_stub(), "{name}, the saved tree: {saved:?}");
        assert_eq!(
            unsaved, saved,
            "{name}: the unsaved edit at {at} against the saved one"
        );
    }
}

/// A new file the editor has not saved, in a directory `.git/info/exclude`
/// ignores, is not read, as the saved file is not: its declaration is no
/// duplicate of the one the walk reads.
#[test]
fn an_unsaved_file_the_walk_ignores_is_not_read() {
    let tree = Tree::new("ignored", &config(&[]), &[("docs/fs.md", DECLARES)]);
    let git = std::process::Command::new("git")
        .args(["init", "-q", "."])
        .current_dir(&tree.0)
        .output()
        .expect("spawn git init");
    assert!(git.status.success(), "git init {}", tree.0.display());
    tree.write(".git/info/exclude", "gen/\n");
    let (unsaved, saved) = tree.unsaved_then_saved(&[], "docs/gen/b.md", "# FS-a: Other\n\nX.\n");
    assert!(saved.clean_with_body(), "the saved tree: {saved:?}");
    assert_eq!(
        unsaved, saved,
        "the unsaved ignored file against the saved one"
    );
}
