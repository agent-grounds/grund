//! An import needs no shared parent: with nothing configured above either repository, the
//! consumer's own config reaches the sibling from the command line, run in the consumer or
//! below it, and from an editor holding only the consumer's folder or one inside it
//! (§FS-imports.editor, §FS-imports.declaration.path). Git sees no configuration but the
//! fixture's, and stops at the fixture root. Red until #1080 ships `[workspace.imports]`;
//! the change that ships it removes each `ignore`.

#[path = "binaries.rs"]
mod binaries;
#[path = "../../crates/grund-lsp/tests/support/mod.rs"]
mod support;

use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Output, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use support::{file_uri, send_message, wait_for_exit};

const GRAAL: &str = "grund_config_version = 2\nproject_name = \"native-image\"\n\n\
    [schema.kinds.FS]\nfolders = [\"docs/functional-spec\"]\n";
const CONSUMER: &str = "grund_config_version = 2\nproject_name = \"enterprise\"\n\n\
    [workspace.imports.native-image]\npath = \"../graal/substratevm\"\nproject = \"native-image\"\n";
const DECLARATION: &str = "graal/substratevm/docs/functional-spec/FS-security-providers.md";
const LEAD: &str = "Providers are registered at image build time.";
const EXAMPLE: &str = "// Registers providers as \u{a7}native-image/FS-security-providers requires.\nclass Example {}\n";

/// `checkout/graal/substratevm` and `checkout/enterprise`, under a base no config covers.
struct Checkout(PathBuf);

impl Checkout {
    fn new(name: &str, consumer_config: &str) -> Self {
        let root = support::fixture_base().join(format!(
            "grund-imports-no-parent-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create checkout");
        let checkout = Self(root.canonicalize().expect("canonical checkout"));
        assert_eq!(
            support::covering_config(&checkout.0),
            None,
            "a config covers the checkout"
        );
        checkout.write("graal/substratevm/grund.toml", GRAAL);
        checkout.write(
            DECLARATION,
            &format!("# FS-security-providers: Registered at build time\n\n{LEAD}\n"),
        );
        checkout.write(consumer_config, CONSUMER);
        checkout.write("enterprise/src/Example.java", EXAMPLE);
        checkout
    }

    fn write(&self, relative: &str, body: &str) {
        let path = self.0.join(relative);
        fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
        fs::write(path, body).expect("write fixture");
    }

    fn command(&self, program: &Path, cwd: &Path) -> Command {
        let mut command = Command::new(program);
        command
            .current_dir(cwd)
            .env("HOME", self.0.join("home"))
            .env("USERPROFILE", self.0.join("home"))
            .env("XDG_CONFIG_HOME", self.0.join("xdg"))
            .env("GIT_CONFIG_GLOBAL", self.0.join("xdg/git/config"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CEILING_DIRECTORIES", &self.0)
            .env_remove("GIT_CONFIG")
            .env_remove("GIT_CONFIG_SYSTEM")
            .env_remove("GIT_CONFIG_COUNT")
            .env_remove("GIT_CONFIG_PARAMETERS")
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE");
        command
    }

    fn grund(&self, cwd: &str, args: &[&str]) -> Output {
        self.command(&binaries::grund(), &self.0.join(cwd))
            .args(args)
            .output()
            .expect("run grund")
    }
}

impl Drop for Checkout {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// `check` and a query, from the consumer and from below it, under either config name.
#[test]
#[ignore = "red until #1080 ships [workspace.imports]; implement removes this"]
fn the_command_line_reaches_the_sibling_with_no_common_parent() {
    for (config, report, caution) in [
        ("enterprise/grund.toml", "grund.toml", ""),
        (
            "enterprise/.agents/grund.toml",
            ".agents/grund.toml",
            "warning: .agents/grund.toml is a deprecated config location \u{2014} move it to grund.toml\n",
        ),
    ] {
        let checkout = Checkout::new("cli", config);
        for cwd in ["enterprise", "enterprise/src"] {
            let check = checkout.grund(cwd, &["check"]);
            let live = format!(
                "{report}:4: warning: import `native-image` is read live from `../graal/substratevm`, unpinned\n"
            );
            assert_eq!(
                (
                    check.status.code(),
                    text(&check.stdout),
                    text(&check.stderr)
                ),
                (Some(0), live, caution.to_string()),
                "check from {cwd} with {config}"
            );
            let show = checkout.grund(cwd, &["native-image/FS-security-providers"]);
            assert_eq!(
                (show.status.code(), text(&show.stdout)),
                (Some(0), format!("{LEAD}\n")),
                "query from {cwd} with {config}: {show:?}"
            );
        }
    }
}

/// A server whose only folder is `folder`, started from the checkout root so no ambient
/// directory can stand in for discovery (§FS-lsp.2.2.1).
fn start(checkout: &Checkout, folder: &Path) -> (Child, ChildStdin, mpsc::Receiver<Value>) {
    let mut command = checkout.command(&support::server_binary(), &checkout.0);
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = support::spawn_child(&mut command, "spawn grund-lsp");
    let mut stdin = child.stdin.take().expect("child stdin");
    let receiver = support::read_messages(child.stdout.take().expect("child stdout"));
    let folders = json!([{ "uri": file_uri(folder), "name": "folder" }]);
    send_message(
        &mut stdin,
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
            "processId": std::process::id(), "workspaceFolders": folders,
            "capabilities": { "workspace": { "workspaceFolders": true } }}}),
    );
    support::recv_response_or_panic(&receiver, &mut child, 1);
    send_message(
        &mut stdin,
        json!({"jsonrpc": "2.0", "method": "initialized", "params": {}}),
    );
    (child, stdin, receiver)
}

/// Send one request and wait for its response, recording every diagnostic published on
/// the way, keyed by URI.
fn request(
    stdin: &mut ChildStdin,
    receiver: &mpsc::Receiver<Value>,
    diagnostics: &mut BTreeMap<String, Vec<Value>>,
    id: i64,
    method: &str,
    params: Value,
) -> Value {
    send_message(
        stdin,
        json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}),
    );
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let message = receiver
            .recv_timeout(remaining)
            .unwrap_or_else(|_| panic!("no response to {method}"));
        if message.get("id").and_then(Value::as_i64) == Some(id) {
            return message["result"].clone();
        }
        if message.get("method").and_then(Value::as_str) == Some("textDocument/publishDiagnostics")
        {
            let uri = message["params"]["uri"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            let found = message["params"]["diagnostics"]
                .as_array()
                .cloned()
                .unwrap_or_default();
            diagnostics.insert(uri, found);
        }
    }
}

/// Hover and definition through the import, and no diagnostic on the consumer's file,
/// from a session holding only the consumer's folder or only a folder inside it.
#[test]
#[ignore = "red until #1080 answers through imports in the editor; implement removes this"]
fn the_editor_answers_through_the_import_from_the_consumer_folder_alone() {
    let _ = support::SERVER_BINARY.set(binaries::grund_lsp());
    for folder in ["enterprise", "enterprise/src"] {
        let checkout = Checkout::new("lsp", "enterprise/grund.toml");
        let (mut child, mut stdin, receiver) = start(&checkout, &checkout.0.join(folder));
        let example = file_uri(&checkout.0.join("enterprise/src/Example.java"));
        send_message(
            &mut stdin,
            json!({"jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {
                "textDocument": {"uri": example, "version": 1, "languageId": "java", "text": EXAMPLE}}}),
        );
        let mut diagnostics = BTreeMap::new();
        // `§native-image/FS-security-providers` starts at UTF-16 column 26 of line 0.
        let position =
            json!({"textDocument": {"uri": example}, "position": {"line": 0, "character": 32}});
        let hover = request(
            &mut stdin,
            &receiver,
            &mut diagnostics,
            2,
            "textDocument/hover",
            position.clone(),
        );
        assert!(
            hover.to_string().contains(LEAD),
            "hover from {folder}: {hover}"
        );
        let definition = request(
            &mut stdin,
            &receiver,
            &mut diagnostics,
            3,
            "textDocument/definition",
            position,
        );
        assert!(
            definition
                .to_string()
                .contains(&file_uri(&checkout.0.join(DECLARATION))),
            "definition from {folder}: {definition}"
        );
        assert!(
            diagnostics.get(&example).is_none_or(Vec::is_empty),
            "diagnostics from {folder}: {diagnostics:?}"
        );
        request(
            &mut stdin,
            &receiver,
            &mut diagnostics,
            4,
            "shutdown",
            Value::Null,
        );
        send_message(
            &mut stdin,
            json!({"jsonrpc": "2.0", "method": "exit", "params": null}),
        );
        wait_for_exit(&mut child);
    }
}
