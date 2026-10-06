//! Stdio acceptance fixtures for §FS-lsp.1.6.4; no completion implementation.
#![allow(dead_code)]

#[path = "binaries.rs"]
mod binaries;
#[path = "../../crates/grund-lsp/tests/support/mod.rs"]
mod wire;

use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    mpsc,
};

pub fn file_uri(path: &Path) -> String {
    // Preserve the requested spelling so outward-symlink refusals are exercised.
    url::Url::from_file_path(path)
        .expect("absolute fixture URI")
        .to_string()
}
static NEXT_ROOT: AtomicUsize = AtomicUsize::new(0);

pub struct Fixture(pub PathBuf);
impl Fixture {
    pub fn new() -> Self {
        let root = PathBuf::from(
            std::env::var_os("HOME")
                .or_else(|| std::env::var_os("USERPROFILE"))
                .expect("user home"),
        )
        .join("ag/tmp")
        .join(format!(
            "grund-completion-{}-{}",
            std::process::id(),
            NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.parent().unwrap()).unwrap();
        fs::create_dir(&root).expect("claim exclusive fixture root");
        let fixture = Self(root);
        fixture.write("grund.toml", "grund_config_version = 1\n[id]\nformat = \"{kind}-{slug}\"\n[fmt.cross_refs]\nenabled = false\n");
        fixture.write(
            "docs/functional-spec/FS-login.md",
            "# FS-login: Login\n\nLead.\n\n## 1. Details\n\nDetails.\n",
        );
        fixture.write(
            "docs/functional-spec/FS-logout.md",
            "# FS-logout: Logout\n\nLead.\n",
        );
        fixture.write("docs/notes.md", "\n");
        fixture.write("src/lib.rs", "\n");
        fixture
    }
    pub fn write(&self, path: &str, text: &str) -> PathBuf {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, text).unwrap();
        path
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub struct Session {
    child: Child,
    input: ChildStdin,
    output: mpsc::Receiver<Value>,
    next: i64,
    version: i64,
    opened: std::collections::HashSet<PathBuf>,
    pub capabilities: Value,
}
impl Session {
    pub fn new(root: &Path) -> Self {
        let mut child = Command::new(binaries::grund_lsp())
            .current_dir(root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let input = child.stdin.take().unwrap();
        let output = wire::read_messages(child.stdout.take().unwrap());
        let mut session = Self {
            child,
            input,
            output,
            next: 0,
            version: 0,
            opened: Default::default(),
            capabilities: Value::Null,
        };
        let response = session.request(
            "initialize",
            json!({"processId": null,
            "rootUri": file_uri(root), "capabilities": {}}),
        );
        assert!(
            response.get("error").is_none(),
            "initialize failed: {response}"
        );
        session.capabilities = response["result"]["capabilities"].clone();
        session.notify("initialized", json!({}));
        session
    }
    pub fn notify(&mut self, method: &str, params: Value) {
        wire::send_message(
            &mut self.input,
            json!({"jsonrpc":"2.0", "method":method,"params":params}),
        );
    }
    pub fn request(&mut self, method: &str, params: Value) -> Value {
        self.next += 1;
        wire::send_message(
            &mut self.input,
            json!({"jsonrpc":"2.0", "id":self.next,"method":method,"params":params}),
        );
        wire::recv_response_or_panic(&self.output, &mut self.child, self.next)
    }
    pub fn text(&mut self, path: &Path, text: &str) {
        self.version += 1;
        if self.opened.insert(path.to_path_buf()) {
            self.notify("textDocument/didOpen", json!({"textDocument": {
                "uri":file_uri(path),"version":self.version,"languageId":path.extension().unwrap().to_str().unwrap(),"text":text}}));
        } else {
            self.notify(
                "textDocument/didChange",
                json!({"textDocument":{"uri":file_uri(path),"version":self.version},
                "contentChanges":[{"text":text}]}),
            );
        }
    }
    pub fn completion(
        &mut self,
        path: &Path,
        text: &str,
        line: usize,
        cursor_byte: usize,
    ) -> Vec<Value> {
        self.text(path, text);
        self.current_completion(path, text, line, cursor_byte)
    }
    pub fn current_completion(
        &mut self,
        path: &Path,
        text: &str,
        line: usize,
        cursor_byte: usize,
    ) -> Vec<Value> {
        let response = self.request(
            "textDocument/completion",
            position(path, text, line, cursor_byte),
        );
        assert!(
            response.get("error").is_none(),
            "completion request rejected: {response}"
        );
        assert_eq!(
            response["result"]["isIncomplete"], true,
            "further typing must refresh choices: {response}"
        );
        response["result"]["items"]
            .as_array()
            .expect("completion list items")
            .clone()
    }
    pub fn on_type(
        &mut self,
        path: &Path,
        text: &str,
        line: usize,
        cursor_byte: usize,
    ) -> Vec<Value> {
        self.text(path, text);
        let mut params = position(path, text, line, cursor_byte);
        params["ch"] = json!(
            text.lines().nth(line).unwrap()[..cursor_byte]
                .chars()
                .next_back()
                .unwrap()
                .to_string()
        );
        params["options"] = json!({"tabSize":4,"insertSpaces":true});
        let response = self.request("textDocument/onTypeFormatting", params);
        assert!(
            response.get("error").is_none(),
            "on-type request: {response}"
        );
        response["result"].as_array().cloned().unwrap_or_default()
    }
    pub fn close_document(&mut self, path: &Path) {
        self.opened.remove(path);
        self.notify(
            "textDocument/didClose",
            json!({"textDocument":{"uri":file_uri(path)}}),
        );
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub fn position(path: &Path, text: &str, line: usize, cursor_byte: usize) -> Value {
    json!({"textDocument":{"uri":file_uri(path)},"position":{"line":line,
        "character":text.lines().nth(line).unwrap()[..cursor_byte].encode_utf16().count()}})
}
pub fn chosen<'a>(items: &'a [Value], id: &str, marker: &str) -> &'a Value {
    items
        .iter()
        .find(|item| item["textEdit"]["newText"] == format!("{marker}{id}"))
        .unwrap_or_else(|| panic!("missing candidate {id}: {items:?}"))
}
pub fn apply(text: &str, edit: &Value) -> String {
    let start = &edit["range"]["start"];
    let end = &edit["range"]["end"];
    assert_eq!(start["line"], end["line"], "single-line edit");
    let line = start["line"].as_u64().unwrap() as usize;
    let lines: Vec<_> = text.split_inclusive('\n').collect();
    let base: usize = lines[..line].iter().map(|l| l.len()).sum();
    let byte = |units: u64| {
        let mut count = 0;
        for (offset, ch) in lines[line].char_indices() {
            if count == units {
                return offset;
            }
            count += ch.len_utf16() as u64;
        }
        assert_eq!(count, units, "invalid UTF-16 boundary");
        lines[line].len()
    };
    let a = base + byte(start["character"].as_u64().unwrap());
    let b = base + byte(end["character"].as_u64().unwrap());
    format!(
        "{}{}{}",
        &text[..a],
        edit["newText"].as_str().unwrap(),
        &text[b..]
    )
}
pub fn cli(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(binaries::grund())
        .current_dir(root)
        .args(args)
        .output()
        .unwrap()
}
