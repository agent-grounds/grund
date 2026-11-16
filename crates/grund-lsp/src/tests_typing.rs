//! A keystroke's completion is answered from its own text and the project's
//! declarations, never from behind a rebuild (§FS-lsp.responsiveness.4), and the
//! diagnostics of a burst of edits arrive in version order, each publish naming
//! the version it was checked at (§FS-lsp.1.1).
//!
//! These cases drive the real event loop over an in-memory connection: the loop
//! runs on the test thread under the core's check-input observer, and a client
//! script runs on a thread of its own. The observer is thread-local, so it sees
//! what the loop thread reads and nothing a rebuild elsewhere reads. Nothing is
//! timed: the cost is the inputs the loop reads, counted, as
//! §FS-lsp.responsiveness asks.

use super::*;
use grund_core::{CheckInput, with_check_input_observer};
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// How long the client waits for one answer before it calls the server stuck.
const ANSWER_WAIT: Duration = Duration::from_secs(120);

/// One project: three numbered FS declarations and one AR, a Markdown note and a
/// Java source that cite nothing yet.
pub(super) fn fixture(root: &Path) {
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\nproject_name = \"typing\"\n\
         [reference]\nmarker = \"\u{a7}\"\nstrict = true\n\
         [id]\nformat = \"{kind}-{number}-{slug}\"\n\
         [[kinds]]\nkind = \"FS\"\nfolder = \"docs/functional-spec\"\ntitle = \"What\"\n\
         [[kinds]]\nkind = \"AR\"\nfolder = \"docs/architecture\"\ntitle = \"How\"\n\
         [scan]\ninclude = [\"docs\", \"src\"]\nextensions = [\"md\", \"java\"]\n",
    );
    for (path, id, title) in [
        ("docs/functional-spec", "FS-001-intro", "Intro"),
        ("docs/functional-spec", "FS-002-login", "Login"),
        ("docs/functional-spec", "FS-020-logout", "Logout"),
        ("docs/architecture", "AR-001-server", "Server"),
    ] {
        write(
            &root.join(format!("{path}/{id}.md")),
            &format!("# {id}: {title}\n\nThe lead.\n"),
        );
    }
    write(&root.join("docs/notes.md"), &note("Plain prose."));
    write(&root.join("src/Main.java"), &java("Plain prose."));
}

/// The Markdown note with `line` as its body; the body is line 2.
pub(super) fn note(line: &str) -> String {
    format!("# Notes\n\n{line}\n")
}

/// The Java source with `line` in a `//` comment; the comment is line 1.
fn java(line: &str) -> String {
    format!("class Main {{\n    // {line}\n}}\n")
}

pub(super) fn server_over(root: &Path) -> (Server, Connection) {
    let (connection, client) = Connection::memory();
    let folders = [(Url::from_directory_path(root).unwrap(), root.to_path_buf())];
    let server = Server::new(connection, folders.into_iter().collect(), true).unwrap();
    (server, client)
}

/// The client end of a session, run on its own thread by [`drive`].
pub(super) struct Client {
    pub(super) connection: Connection,
    /// Open while the loop thread's inputs are being recorded.
    window: Arc<AtomicBool>,
    pub(super) next: RefCell<i32>,
    /// How many messages the client has sent, for [`RebuildHold::taken`].
    pub(super) sent: std::cell::Cell<usize>,
    /// Every `publishDiagnostics` the client has read, in arrival order.
    pub(super) published: RefCell<Vec<PublishDiagnosticsParams>>,
}

impl Client {
    pub(super) fn notify(&self, method: &str, params: Value) {
        self.post(Notification::new(method.into(), params).into());
    }

    /// Send `message`, counted.
    pub(super) fn post(&self, message: Message) {
        self.sent.set(self.sent.get() + 1);
        self.connection.sender.send(message).unwrap();
    }

    /// Send `method` and read up to its answer, keeping every publish on the way.
    fn request(&self, method: &str, params: Value) -> Value {
        let id = {
            let mut next = self.next.borrow_mut();
            *next += 1;
            *next
        };
        self.post(Request::new(id.into(), method.into(), params).into());
        loop {
            let message = self
                .connection
                .receiver
                .recv_timeout(ANSWER_WAIT)
                .unwrap_or_else(|err| panic!("no answer to `{method}`: {err}"));
            match message {
                Message::Response(response) if response.id == id.into() => {
                    assert!(response.error.is_none(), "{method}: {:?}", response.error);
                    return response.result.unwrap_or(Value::Null);
                }
                Message::Notification(notification)
                    if notification.method == "textDocument/publishDiagnostics" =>
                {
                    let params = serde_json::from_value(notification.params).unwrap();
                    self.published.borrow_mut().push(params);
                }
                _ => {}
            }
        }
    }

    pub(super) fn open(&self, path: &Path, text: &str, version: i32) {
        let language = path.extension().unwrap().to_str().unwrap();
        self.notify(
            "textDocument/didOpen",
            json!({"textDocument": {"uri": uri(path), "languageId": language,
                                    "version": version, "text": text}}),
        );
    }

    pub(super) fn change(&self, path: &Path, text: &str, version: i32) {
        self.notify(
            "textDocument/didChange",
            json!({"textDocument": {"uri": uri(path), "version": version},
                   "contentChanges": [{"text": text}]}),
        );
    }

    /// The labels a completion at the end of `text`'s line `line` is offered, in
    /// wire order.
    pub(super) fn completion(&self, path: &Path, text: &str, line: u32) -> Vec<String> {
        let typed = text.lines().nth(line as usize).unwrap();
        let result = self.request(
            "textDocument/completion",
            json!({"textDocument": {"uri": uri(path)},
                   "position": {"line": line, "character": typed.encode_utf16().count()},
                   "context": {"triggerKind": 3}}),
        );
        assert_eq!(result["isIncomplete"], true, "{result}");
        result["items"]
            .as_array()
            .expect("completion items")
            .iter()
            .map(|item| item["label"].as_str().unwrap().to_string())
            .collect()
    }

    /// Wait until every edit sent so far is checked: a hover is a snapshot
    /// request, so it is answered only from a snapshot of the text it was sent
    /// against (§FS-lsp.responsiveness.3).
    pub(super) fn settle(&self, path: &Path) {
        self.request(
            "textDocument/hover",
            json!({"textDocument": {"uri": uri(path)},
                   "position": {"line": 0, "character": 0}}),
        );
    }

    /// Run `step` with the loop thread's inputs recorded.
    fn observed<T>(&self, step: impl FnOnce() -> T) -> T {
        self.window.store(true, Ordering::SeqCst);
        let out = step();
        self.window.store(false, Ordering::SeqCst);
        out
    }
}

pub(super) fn uri(path: &Path) -> Url {
    Url::from_file_path(path).unwrap()
}

/// Run `server`'s event loop on this thread under the check-input observer while
/// `script` drives it from another, then shut the session down. Returns what the
/// script returned and every input the loop thread read while a window was open.
pub(super) fn drive<T: Send + 'static>(
    mut server: Server,
    connection: Connection,
    script: impl FnOnce(&Client) -> T + Send + 'static,
) -> (T, Vec<CheckInput>) {
    let window = Arc::new(AtomicBool::new(false));
    let observed = Arc::new(Mutex::new(Vec::new()));
    let (open, sink) = (Arc::clone(&window), Arc::clone(&observed));
    let client = std::thread::spawn(move || {
        let client = Client {
            connection,
            window: open,
            next: RefCell::new(0),
            sent: std::cell::Cell::new(0),
            published: RefCell::new(Vec::new()),
        };
        let out = script(&client);
        // Closing the client's half ends the loop; read until the server is gone.
        let Connection { sender, receiver } = client.connection;
        drop(sender);
        while receiver.recv().is_ok() {}
        out
    });
    let recording = Arc::clone(&window);
    let looped = with_check_input_observer(
        Some(Arc::new(move |input| {
            if recording.load(Ordering::SeqCst) {
                sink.lock().unwrap().push(input);
            }
            true
        })),
        || server.event_loop(),
    );
    drop(server);
    let out = client
        .join()
        .unwrap_or_else(|panic| std::panic::resume_unwind(panic));
    looped.expect("the event loop");
    let observed = observed.lock().unwrap().clone();
    (out, observed)
}

/// Type the key that turns `before` into `after` on line `line` of `path` and ask
/// for completion there; return the labels offered and every input the loop
/// thread read between reading the `didChange` and answering the completion.
fn type_key(
    root: &Path,
    rel: &str,
    line: u32,
    (before, after): (String, String),
) -> (Vec<String>, Vec<CheckInput>) {
    let path = root.join(rel);
    let (server, connection) = server_over(root);
    drive(server, connection, move |client| {
        client.open(&path, &before, 1);
        client.settle(&path);
        client.observed(|| {
            client.change(&path, &after, 2);
            client.completion(&path, &after, line)
        })
    })
}

/// Every observed input that is not the edited document.
fn beyond<'a>(observed: &'a [CheckInput], edited: &Path) -> Vec<&'a Path> {
    let edited = fs::canonicalize(edited).unwrap();
    observed
        .iter()
        .map(|input| input.path.as_path())
        .filter(|path| !fs::canonicalize(path).is_ok_and(|path| path == edited))
        .collect()
}

/// What `$$FS-00` offers: the two declarations whose ID starts with `FS-00`.
pub(super) const AFTER_FS_00: [&str; 2] = ["FS-001-intro", "FS-002-login"];

fn assert_typing_answer(rel: &str, labels: Vec<String>, observed: &[CheckInput], root: &Path) {
    let mut sorted = labels.clone();
    sorted.sort();
    assert_eq!(
        sorted, AFTER_FS_00,
        "the completion after typing `0` in {rel} offers {labels:?}: it must offer exactly \
         what a full rebuild's catalog does for `$$FS-00` (\u{a7}FS-lsp.responsiveness.4)"
    );
    let read = beyond(observed, &root.join(rel));
    assert!(
        read.is_empty(),
        "between reading the didChange to {rel} and answering its completion, the loop thread \
         read {} inputs besides the edited document, {:?}…: the keystroke waited on a rebuild \
         of the tree (\u{a7}FS-lsp.responsiveness.4)",
        read.len(),
        read.iter().take(5).collect::<Vec<_>>()
    );
}

/// §FS-lsp.responsiveness.4: a key typed into Markdown body prose leaves the
/// project's declarations as they were, so its completion is answered from the
/// edited text and the catalog alone. Today the `didChange` rebuilds the project
/// on the loop thread first, reading every scanned file.
#[test]
fn a_keystroke_in_markdown_prose_is_completed_without_reading_the_tree() {
    let root = test_root("typing_markdown_prose");
    fixture(&root);
    let rel = "docs/notes.md";
    let edit = (
        note("Plain prose, see $$FS-0"),
        note("Plain prose, see $$FS-00"),
    );
    let (labels, observed) = type_key(&root, rel, 2, edit);
    assert_typing_answer(rel, labels, &observed, &root);
}

/// §FS-lsp.responsiveness.4: the same for a `//` comment in a Java source, which
/// the issue's GraalVM session types into as often as into a spec.
#[test]
fn a_keystroke_in_a_java_comment_is_completed_without_reading_the_tree() {
    let root = test_root("typing_java_comment");
    fixture(&root);
    let rel = "src/Main.java";
    let edit = (
        java("Plain prose, see $$FS-0"),
        java("Plain prose, see $$FS-00"),
    );
    let (labels, observed) = type_key(&root, rel, 1, edit);
    assert_typing_answer(rel, labels, &observed, &root);
}

/// The texts a burst types into the note and the spec, oldest first: each cites a
/// declaration that resolves and one that does not, so every version has a
/// diagnostic of its own.
fn burst() -> Vec<(i32, String, String)> {
    (2..=6)
        .map(|version| {
            let note = note(&format!(
                "See \u{a7}FS-001-intro and \u{a7}FS-9{version:02}-gone."
            ));
            let spec = format!("# FS-002-login: Login\n\nThe lead, \u{a7}FS-8{version:02}-gone.\n");
            (version, note, spec)
        })
        .collect()
}

/// The last publish for each URI, as diagnostics alone.
fn last_per_uri(published: &[PublishDiagnosticsParams]) -> BTreeMap<Url, Vec<Diagnostic>> {
    published
        .iter()
        .map(|publish| (publish.uri.clone(), publish.diagnostics.clone()))
        .collect()
}

/// §FS-lsp.1.1: once a burst of edits has drained, the last publish for each URI
/// is what a fresh server publishes for the final buffers; every publish for an
/// open document carries the version it was checked at, and none for an older
/// version follows one for a newer. Today every publish carries no version.
#[test]
fn a_drained_burst_publishes_what_a_fresh_server_does_in_version_order() {
    let root = test_root("typing_diagnostics_burst");
    fixture(&root);
    let notes = root.join("docs/notes.md");
    let spec = root.join("docs/functional-spec/FS-002-login.md");
    let (server, connection) = server_over(&root);
    let (paths, edits) = ((notes.clone(), spec.clone()), burst());
    let (published, _) = drive(server, connection, move |client| {
        let (notes, spec) = paths;
        client.open(&notes, &note("Plain prose."), 1);
        client.open(&spec, "# FS-002-login: Login\n\nThe lead.\n", 1);
        client.settle(&notes);
        for (version, note, spec_text) in &edits {
            client.change(&notes, note, *version);
            client.change(&spec, spec_text, *version);
        }
        client.settle(&notes);
        client.published.take()
    });

    let (final_version, final_note, final_spec) = burst().pop().unwrap();
    let (fresh_server, connection) = server_over(&root);
    let paths = (notes.clone(), spec.clone());
    let (fresh, _) = drive(fresh_server, connection, move |client| {
        let (notes, spec) = paths;
        client.open(&notes, &final_note, final_version);
        client.open(&spec, &final_spec, final_version);
        client.settle(&notes);
        client.published.take()
    });

    let open = [uri(&notes), uri(&spec)];
    let mut newest: BTreeMap<&Url, i32> = BTreeMap::new();
    for publish in published
        .iter()
        .filter(|publish| open.contains(&publish.uri))
    {
        let version = publish.version.unwrap_or_else(|| {
            panic!(
                "a publish for the open document {} carries no version: each publish names \
                 the document version it was checked at (\u{a7}FS-lsp.1.1)",
                publish.uri
            )
        });
        let seen = newest.entry(&publish.uri).or_insert(version);
        assert!(
            version >= *seen,
            "{} was published for version {version} after version {seen} (\u{a7}FS-lsp.1.1)",
            publish.uri
        );
        *seen = version;
    }
    for document in &open {
        assert_eq!(
            newest.get(document),
            Some(&final_version),
            "the last publish for {document} is not for its final version"
        );
    }
    let (mut drained, mut expected) = (last_per_uri(&published), last_per_uri(&fresh));
    for uri in drained
        .keys()
        .chain(expected.keys())
        .cloned()
        .collect::<Vec<_>>()
    {
        drained.entry(uri.clone()).or_default();
        expected.entry(uri).or_default();
    }
    assert_eq!(
        drained, expected,
        "after the burst drained, the last publish for each URI is not what a fresh server \
         publishes for the final buffers (\u{a7}FS-lsp.1.1)"
    );
}
