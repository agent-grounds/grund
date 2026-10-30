//! A composition the editor reaches through an import, and one editor session over it
//! (§FS-imports.editor, §AR-lsp.2.1): `graal/substratevm` is the dependency and
//! `enterprise` and `enterprise2` two consumers that import it as `native-image`, under a
//! base no config covers. The session answers every request the server sends it, so a
//! watcher registration is recorded rather than left waiting.
#![allow(dead_code)]

use crate::binaries;
use crate::support::{self, file_uri, send_message, wait_for_exit};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

pub const DEPENDENCY: &str = "graal/substratevm";
pub const DEPENDENCY_CONFIG: &str = "grund_config_version = 2\nproject_name = \"native-image\"\n\n\
    [schema.kinds.FS]\nfolders = [\"docs/functional-spec\"]\n\n\
    [schema.kinds.CONST]\nfolders = [\"values\"]\nform = \"value\"\nindex = false\n";
pub const DECLARATION: &str = "graal/substratevm/docs/functional-spec/FS-security-providers.md";
pub const MAIN: &str = "graal/substratevm/src/Main.java";
/// `<§>native-image/FS-security-providers` starts at UTF-16 column 26 of line 0.
pub const EXAMPLE: &str = "// Registers providers as \u{a7}native-image/FS-security-providers requires.\nclass Example {}\n";

pub fn consumer_config(name: &str) -> String {
    format!(
        "grund_config_version = 2\nproject_name = \"{name}\"\n\n\
         [workspace.imports.native-image]\npath = \"../graal/substratevm\"\nproject = \"native-image\"\n"
    )
}

pub fn declaration(title: &str) -> String {
    format!(
        "# FS-security-providers: {title}\n\n## 2. Two\n\nProviders are registered at image build time.\n"
    )
}

pub struct Composition(pub PathBuf);

impl Composition {
    pub fn new(name: &str) -> Self {
        let _ = support::SERVER_BINARY.set(binaries::grund_lsp());
        let root = support::fixture_base()
            .join(format!("grund-lsp-imports-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create composition");
        let composition = Self(root.canonicalize().expect("canonical composition"));
        composition.write("graal/substratevm/grund.toml", DEPENDENCY_CONFIG);
        composition.write(DECLARATION, &declaration("Registered at build time"));
        composition.write(
            "graal/substratevm/docs/functional-spec/README.md",
            "# Functional spec\n\n- [\u{a7}FS-security-providers](FS-security-providers.md#fs-security-providers-registered-at-build-time)\n",
        );
        composition.write(
            "graal/substratevm/values/prices.json",
            "{\n  \"CONST-price\": [1200, \"USD\"]\n}\n",
        );
        composition.write(
            MAIN,
            "// \u{a7}FS-missing and \u{a7}FS-security-providers\nclass Main {}\n",
        );
        for consumer in ["enterprise", "enterprise2"] {
            composition.write(
                &format!("{consumer}/grund.toml"),
                &consumer_config(consumer),
            );
            composition.write(&format!("{consumer}/src/Example.java"), EXAMPLE);
        }
        composition
    }

    pub fn path(&self, relative: &str) -> PathBuf {
        self.0.join(relative)
    }

    pub fn uri(&self, relative: &str) -> String {
        file_uri(&self.path(relative))
    }

    pub fn write(&self, relative: &str, body: &str) {
        let path = self.path(relative);
        fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
        fs::write(path, body).expect("write fixture");
    }

    /// A command with no Git or home configuration but the composition's own.
    pub fn command(&self, program: &Path, cwd: &Path) -> Command {
        let mut command = Command::new(program);
        command
            .current_dir(cwd)
            .env("HOME", self.0.join("home"))
            .env("USERPROFILE", self.0.join("home"))
            .env("XDG_CONFIG_HOME", self.0.join("xdg"))
            .env("GIT_CONFIG_GLOBAL", self.0.join("xdg/git/config"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CEILING_DIRECTORIES", &self.0)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE");
        command
    }

    /// `grund` run in `cwd` with `args`: its exit code and stdout.
    pub fn grund(&self, cwd: &str, args: &[&str]) -> (Option<i32>, String) {
        let output = self
            .command(&binaries::grund(), &self.path(cwd))
            .args(args)
            .output()
            .expect("run grund");
        (
            output.status.code(),
            String::from_utf8_lossy(&output.stdout).into_owned(),
        )
    }
}

impl Drop for Composition {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub struct Session<'a> {
    composition: &'a Composition,
    child: Child,
    stdin: ChildStdin,
    receiver: mpsc::Receiver<Value>,
    next: i64,
    /// The latest diagnostics published on each URI.
    pub diagnostics: BTreeMap<String, Vec<Value>>,
    /// Every request the server sent, in order.
    pub server_requests: Vec<Value>,
    /// The capabilities the server advertised in its `initialize` answer.
    pub capabilities: Value,
}

impl<'a> Session<'a> {
    /// A session over `folders`; `watchers` says whether the client can register
    /// `workspace/didChangeWatchedFiles` dynamically, relative patterns included.
    pub fn start(composition: &'a Composition, folders: &[&str], watchers: bool) -> Self {
        let mut command = composition.command(&support::server_binary(), &composition.0);
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        let mut child = support::spawn_child(&mut command, "spawn grund-lsp");
        let stdin = child.stdin.take().expect("child stdin");
        let receiver = support::read_messages(child.stdout.take().expect("child stdout"));
        let mut session = Self {
            composition,
            child,
            stdin,
            receiver,
            next: 1,
            diagnostics: BTreeMap::new(),
            server_requests: Vec::new(),
            capabilities: Value::Null,
        };
        let folders = folders
            .iter()
            .map(|folder| json!({"uri": composition.uri(folder), "name": folder}))
            .collect::<Vec<_>>();
        let watched = json!({"dynamicRegistration": watchers, "relativePatternSupport": watchers});
        let initialized = session.request(
            "initialize",
            json!({"processId": std::process::id(), "workspaceFolders": folders,
                "capabilities": {"workspace": {"workspaceFolders": true,
                    "didChangeWatchedFiles": watched}}}),
        );
        session.capabilities = initialized["capabilities"].clone();
        session.notify("initialized", json!({}));
        session
    }

    pub fn notify(&mut self, method: &str, params: Value) {
        send_message(
            &mut self.stdin,
            json!({"jsonrpc": "2.0", "method": method, "params": params}),
        );
    }

    /// Send one request and wait for its response, recording what arrives on the way.
    pub fn request(&mut self, method: &str, params: Value) -> Value {
        let id = self.next;
        self.next += 1;
        send_message(
            &mut self.stdin,
            json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}),
        );
        let deadline = Instant::now() + Duration::from_secs(60);
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            let message = self
                .receiver
                .recv_timeout(remaining)
                .unwrap_or_else(|_| panic!("no response to {method}"));
            if message.get("method").is_none() && message["id"].as_i64() == Some(id) {
                return message["result"].clone();
            }
            self.record(message);
        }
    }

    fn record(&mut self, message: Value) {
        match message["method"].as_str() {
            Some("textDocument/publishDiagnostics") => {
                let uri = message["params"]["uri"].as_str().unwrap_or_default();
                let found = message["params"]["diagnostics"].as_array().cloned();
                self.diagnostics
                    .insert(uri.to_string(), found.unwrap_or_default());
            }
            Some(_) if message.get("id").is_some() => {
                send_message(
                    &mut self.stdin,
                    json!({"jsonrpc": "2.0", "id": message["id"], "result": null}),
                );
                self.server_requests.push(message);
            }
            _ => {}
        }
    }

    pub fn open(&mut self, relative: &str, text: &str) {
        let uri = self.composition.uri(relative);
        self.open_uri(&uri, text);
    }

    pub fn open_uri(&mut self, uri: &str, text: &str) {
        self.notify(
            "textDocument/didOpen",
            json!({"textDocument": {"uri": uri, "version": 1, "languageId": "java", "text": text}}),
        );
    }

    /// The client's report that `changes`, each a relative path and an LSP change
    /// type (1 created, 2 changed, 3 deleted), happened.
    pub fn watched(&mut self, changes: &[(&str, u8)]) {
        let changes = changes
            .iter()
            .map(|(path, kind)| json!({"uri": self.composition.uri(path), "type": kind}))
            .collect::<Vec<_>>();
        self.notify(
            "workspace/didChangeWatchedFiles",
            json!({"changes": changes}),
        );
    }

    pub fn folders(&mut self, added: &[&str], removed: &[&str]) {
        let entry = |folder: &&str| json!({"uri": self.composition.uri(folder), "name": folder});
        let event = json!({"added": added.iter().map(entry).collect::<Vec<_>>(),
            "removed": removed.iter().map(entry).collect::<Vec<_>>()});
        self.notify(
            "workspace/didChangeWorkspaceFolders",
            json!({"event": event}),
        );
    }

    /// The hover text at `line`:`character` of `relative`, or `None`.
    pub fn hover(&mut self, relative: &str, line: u32, character: u32) -> Option<String> {
        let params = json!({"textDocument": {"uri": self.composition.uri(relative)},
            "position": {"line": line, "character": character}});
        let hover = self.request("textDocument/hover", params);
        hover["contents"]["value"].as_str().map(str::to_string)
    }

    /// The consumer citation's hover, once `expected` holds of it or the deadline passes.
    pub fn hover_until(&mut self, expected: impl Fn(Option<&str>) -> bool) -> Option<String> {
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            let hover = self.hover("enterprise/src/Example.java", 0, 32);
            if expected(hover.as_deref()) || Instant::now() > deadline {
                return hover;
            }
            std::thread::sleep(Duration::from_millis(200));
        }
    }

    /// The latest diagnostics on `relative`, after a round trip has drained the queue.
    pub fn diagnostics_on(&mut self, relative: &str) -> Vec<Value> {
        self.request("shutdown-probe/unknown", Value::Null);
        let uri = self.composition.uri(relative);
        self.diagnostics.get(&uri).cloned().unwrap_or_default()
    }

    pub fn registrations(&self) -> Vec<Value> {
        self.server_requests
            .iter()
            .filter(|request| request["method"] == "client/registerCapability")
            .cloned()
            .collect()
    }

    pub fn shutdown(mut self) {
        self.request("shutdown", Value::Null);
        self.notify("exit", Value::Null);
        wait_for_exit(&mut self.child);
    }
}
