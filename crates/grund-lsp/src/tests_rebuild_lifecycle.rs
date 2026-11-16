//! What the loop does at the edges of its rebuilds (§AR-lsp.2.1.1, §AR-lsp.2.1.2): an `exit` sent
//! behind a held `shutdown`, a rebuild that panics, and a save that carries no text. Each case
//! drives the real event loop, its worker behind the gate of `tests_rebuild_gate.rs` where the
//! case needs a rebuild held.

use super::held_client::{
    LOGIN, QUIET, SIGNIN, SPEC, answer, completion_params, fresh, labels, opened_then_held, result,
    send, unanswered,
};
use super::typing::{Client, drive, fixture, note, server_over, uri};
use super::*;
use std::time::Duration;

/// How long a session may take to end once nothing holds it: far under the 30 s `shut_down`
/// waits for an `exit`, so a session that waits for one that already came fails here.
const ENDS_WITHIN: Duration = Duration::from_secs(10);

/// Run `server`'s loop on a thread of its own; the receiver hears once it has returned, with
/// what it returned.
fn looping(server: Server) -> crossbeam_channel::Receiver<std::result::Result<(), String>> {
    let (ended, ends) = crossbeam_channel::bounded(1);
    std::thread::spawn(move || {
        let mut server = server;
        let outcome = server.event_loop().map_err(|err| format!("{err:#}"));
        let _ = ended.send(outcome);
    });
    ends
}

fn notify(client: &Connection, method: &str, params: Value) {
    let message = Notification::new(method.into(), params);
    client.sender.send(message.into()).unwrap();
}

fn open(client: &Connection, path: &Path, text: &str) {
    let document = json!({"uri": uri(path), "languageId": "markdown", "version": 1, "text": text});
    notify(
        client,
        "textDocument/didOpen",
        json!({ "textDocument": document }),
    );
}

/// The response to `id`, skipping every publish on the way.
fn response(client: &Connection, id: i32) -> Response {
    loop {
        match client.receiver.recv_timeout(ENDS_WITHIN) {
            Ok(Message::Response(response)) if response.id == id.into() => return response,
            Ok(_) => {}
            Err(err) => panic!("no answer to request {id}: {err}"),
        }
    }
}

/// Wait for the loop to return, at most [`ENDS_WITHIN`] after nothing holds it any more.
fn ended(ends: &crossbeam_channel::Receiver<std::result::Result<(), String>>) {
    match ends.recv_timeout(ENDS_WITHIN) {
        Ok(outcome) => assert_eq!(outcome, Ok(()), "the session ended in an error"),
        Err(err) => panic!("the session had not ended {ENDS_WITHIN:?} after it could: {err}"),
    }
}

/// §AR-lsp.2.1.2: an `exit` the client sends right behind a held `shutdown`, its stream left
/// open, ends the session once the shutdown is answered, with no wait for another `exit`.
#[test]
fn an_exit_behind_a_held_shutdown_ends_the_session_once_it_is_answered() {
    let root = test_root("rebuild_lifecycle_exit_behind_shutdown");
    fixture(&root);
    let notes = root.join("docs/notes.md");
    let (mut server, client) = server_over(&root);
    let hold = server.hold_rebuilds();
    let ends = looping(server);
    open(&client, &notes, &note("Plain prose."));
    hold.started();
    let shutdown = Request::new(1.into(), "shutdown".into(), Value::Null);
    client.sender.send(shutdown.into()).unwrap();
    notify(&client, "exit", Value::Null);
    hold.taken(3);
    assert!(
        ends.recv_timeout(QUIET).is_err(),
        "the session ended before the rebuild of the open was let through"
    );
    hold.open();
    let answered = response(&client, 1);
    assert!(answered.error.is_none(), "{:?}", answered.error);
    ended(&ends);
    drop(client);
}

/// §AR-lsp.2.1.2: a `shutdown` sent with nothing outstanding is answered at once, and the session
/// ends at the `exit` the client sends after reading that answer, its stream still open.
#[test]
fn a_shutdown_answered_at_once_ends_at_the_exit_after_it() {
    let root = test_root("rebuild_lifecycle_shutdown_then_exit");
    fixture(&root);
    let notes = root.join("docs/notes.md");
    let (server, client) = server_over(&root);
    let ends = looping(server);
    open(&client, &notes, &note("Plain prose."));
    let settle = json!({"textDocument": {"uri": uri(&notes)},
                        "position": {"line": 0, "character": 0}});
    let hover = Request::new(1.into(), "textDocument/hover".into(), settle);
    client.sender.send(hover.into()).unwrap();
    response(&client, 1);
    let shutdown = Request::new(2.into(), "shutdown".into(), Value::Null);
    client.sender.send(shutdown.into()).unwrap();
    let answered = response(&client, 2);
    assert!(answered.error.is_none(), "{:?}", answered.error);
    assert!(
        ends.is_empty(),
        "the session ended before the client's `exit`"
    );
    notify(&client, "exit", Value::Null);
    ended(&ends);
    drop(client);
}

/// §AR-lsp.2.1.1, §FS-lsp.responsiveness.4: a rebuild of a heading rename that panics leaves the
/// last snapshot standing without answering typing from it. The completion held on that rebuild
/// is refused rather than offered the old name, and the next one waits for the failed rebuild to
/// run again and is then what a fresh server offers.
#[test]
fn a_panicked_rebuild_of_a_rename_never_offers_the_old_name() {
    let root = test_root("rebuild_lifecycle_panic");
    fixture(&root);
    let (notes, spec) = (root.join("docs/notes.md"), root.join(SPEC));
    let typed = note("see $$FS-00");
    let (mut server, connection) = server_over(&root);
    let hold = server.hold_rebuilds();
    let (paths, text) = ((notes.clone(), spec.clone()), typed.clone());
    let ((refused, later), _) = drive(server, connection, move |client: &Client| {
        let (notes, spec) = paths;
        opened_then_held(client, &hold, &[(&notes, &text), (&spec, LOGIN)]);
        hold.panic_next();
        client.change(&spec, SIGNIN, 2);
        let params = completion_params(&notes, &text, 2);
        let held = send(client, "textDocument/completion", params.clone());
        hold.taken(client.sent.get());
        hold.release(1);
        let refused = answer(client, held, &mut Vec::new());
        // A hover is answered from the last snapshot, so only after the failed rebuild is.
        client.settle(&notes);
        let later = send(client, "textDocument/completion", params);
        hold.taken(client.sent.get());
        unanswered(client, &[later], "the failed rebuild runs again first");
        hold.open();
        let later = labels(&result(answer(client, later, &mut Vec::new())));
        (refused, later)
    });
    let error = refused
        .error
        .expect("the completion held on the failed rebuild is refused");
    assert_eq!(error.code, -32803, "RequestFailed: {error:?}");
    let at = completion_params(&notes, &typed, 2);
    let open = [(&*notes, &*typed), (&*spec, SIGNIN)];
    let fresh = labels(&fresh(&[&root], &open, "textDocument/completion", &[at])[0]);
    assert_eq!(
        later, fresh,
        "a fresh server offers those buffers otherwise"
    );
    assert!(
        !later.iter().any(|label| label == "FS-002-login"),
        "{later:?}"
    );
}

/// §FS-lsp.1.1: a save that carries no text leaves the document open at the version the saved
/// file holds, so each publish for it after the save still names that version.
#[test]
fn a_save_without_text_keeps_naming_the_open_documents_version() {
    let root = test_root("rebuild_lifecycle_save_version");
    fixture(&root);
    let notes = root.join("docs/notes.md");
    let (server, connection) = server_over(&root);
    let path = notes.clone();
    let (published, _) = drive(server, connection, move |client| {
        client.open(&path, &note("See \u{a7}FS-9zz-gone."), 1);
        client.settle(&path);
        client.change(&path, &note("See \u{a7}FS-9yy-gone."), 2);
        client.settle(&path);
        write(&path, &note("See \u{a7}FS-9yy-gone."));
        client.notify(
            "textDocument/didSave",
            json!({"textDocument": {"uri": uri(&path)}}),
        );
        client.settle(&path);
        client.published.take()
    });
    let versions: Vec<Option<i32>> = published
        .iter()
        .filter(|publish| publish.uri == uri(&notes))
        .map(|publish| publish.version)
        .collect();
    assert_eq!(
        versions,
        [Some(1), Some(2), Some(2)],
        "each publish for the open notes names its version"
    );
}
