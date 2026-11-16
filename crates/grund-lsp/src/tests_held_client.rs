//! How a case talks to the event loop while its rebuilds are held (§AR-lsp.2.1.2): a request
//! sent without waiting for its answer, that answer read past every publish, a check that none
//! came while it should not, and what a fresh server with no gate answers for the same buffers
//! (§FS-lsp.responsiveness.4). Shared by the held-typing and rebuild-lifecycle cases.

use super::hover_snapshot::server_over_folders;
use super::typing::{Client, drive, uri};
use super::*;
use std::time::Duration;

/// How long a case listens for an answer it expects not to come.
pub(super) const QUIET: Duration = Duration::from_millis(300);

/// How long a case waits for an answer it expects.
pub(super) const ANSWER: Duration = Duration::from_secs(120);

pub(super) const SPEC: &str = "docs/functional-spec/FS-002-login.md";
pub(super) const LOGIN: &str = "# FS-002-login: Login\n\nThe lead.\n";
pub(super) const SIGNIN: &str = "# FS-002-signin: Sign in\n\nThe lead.\n";

/// Send a request without waiting for its answer; returns its id.
pub(super) fn send(client: &Client, method: &str, params: Value) -> i32 {
    let id = {
        let mut next = client.next.borrow_mut();
        *next += 1;
        *next
    };
    let request = Request::new(id.into(), method.into(), params);
    client.connection.sender.send(request.into()).unwrap();
    id
}

/// The next response within `wait`, keeping a publish where the harness keeps it.
pub(super) fn next_response(client: &Client, wait: Duration) -> Option<Response> {
    loop {
        match client.connection.receiver.recv_timeout(wait).ok()? {
            Message::Response(response) => return Some(response),
            Message::Notification(notification)
                if notification.method == "textDocument/publishDiagnostics" =>
            {
                let params = serde_json::from_value(notification.params).unwrap();
                client.published.borrow_mut().push(params);
            }
            _ => {}
        }
    }
}

/// Read up to the answer to `id`, keeping every other response in `strays`.
pub(super) fn answer(client: &Client, id: i32, strays: &mut Vec<Response>) -> Response {
    loop {
        match next_response(client, ANSWER) {
            Some(response) if response.id == id.into() => return response,
            Some(response) => strays.push(response),
            None => panic!("no answer to request {id}"),
        }
    }
}

/// Listen briefly and fail if any of `ids` is answered meanwhile.
pub(super) fn unanswered(client: &Client, ids: &[i32], why: &str) {
    while let Some(response) = next_response(client, QUIET) {
        assert!(
            !ids.iter().any(|id| response.id == (*id).into()),
            "request {:?} was answered while the rebuild was held: {why}",
            response.id
        );
    }
}

/// The position `line`, `character` (UTF-16 units) in `path`.
pub(super) fn position(path: &Path, line: u32, character: usize) -> Value {
    json!({"textDocument": {"uri": uri(path)},
           "position": {"line": line, "character": character}})
}

/// A completion at the end of `text`'s line `line`.
pub(super) fn completion_params(path: &Path, text: &str, line: u32) -> Value {
    let typed = text.lines().nth(line as usize).unwrap();
    let mut params = position(path, line, typed.encode_utf16().count());
    params["context"] = json!({"triggerKind": 3});
    params
}

pub(super) fn labels(result: &Value) -> Vec<String> {
    let mut labels: Vec<String> = result["items"]
        .as_array()
        .expect("completion items")
        .iter()
        .map(|item| item["label"].as_str().unwrap().to_string())
        .collect();
    labels.sort();
    labels
}

pub(super) fn result(response: Response) -> Value {
    assert!(response.error.is_none(), "{:?}", response.error);
    response.result.unwrap_or(Value::Null)
}

/// What a fresh server, with no gate, answers for `method` at each of `at` once `open` are open.
pub(super) fn fresh(
    folders: &[&PathBuf],
    open: &[(&Path, &str)],
    method: &str,
    at: &[Value],
) -> Vec<Value> {
    let (server, connection) = server_over_folders(folders);
    let open: Vec<(PathBuf, String)> = open
        .iter()
        .map(|(path, text)| (path.to_path_buf(), text.to_string()))
        .collect();
    let (method, at) = (method.to_string(), at.to_vec());
    let (results, _) = drive(server, connection, move |client| {
        for (path, text) in &open {
            client.open(path, text, 1);
        }
        client.settle(&open[0].0);
        at.into_iter()
            .map(|params| {
                let id = send(client, &method, params);
                result(answer(client, id, &mut Vec::new()))
            })
            .collect()
    });
    results
}

/// Open `open` and settle with every rebuild let through, then close the gate and drop what the
/// worker announced so far.
pub(super) fn opened_then_held(client: &Client, hold: &RebuildHold, open: &[(&Path, &str)]) {
    hold.open();
    for (path, text) in open {
        client.open(path, text, 1);
    }
    client.settle(open[0].0);
    hold.close();
    hold.rest();
}
