// Client messages against the rebuilds behind them (§AR-lsp.2.1.2): a typing request is answered
// from its own text while the declarations stand (§FS-lsp.responsiveness.4), a snapshot request
// waits for the snapshot of its edits (§FS-lsp.responsiveness.3), and `shutdown` waits for both.

/// Whether the session goes on after a message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Flow {
    Continue,
    Exit,
}

/// How an edited document's latest text stands against the installed catalog
/// (§FS-lsp.responsiveness.4): judged when the edit arrives, and again whenever another catalog
/// is installed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Verdict {
    Stands,
    Changed,
    /// A close or a save, or a document with no path: no reading of one file judges it.
    Unreadable,
}

/// A document edited this session: its latest edit, and how that text stands against the
/// installed catalog. One the installed snapshot reflects stands, and stays in the map, so a
/// keystroke finds the same entries whether or not a rebuild landed before it (§AR-lsp.2.1.1).
struct Edited {
    seq: u64,
    verdict: Verdict,
}

/// A request that waits on the rebuild ending at `seq`.
struct Held {
    seq: u64,
    request: Request,
    kind: HeldKind,
}

enum HeldKind {
    /// Answered from the snapshot of the edits sent before it (§FS-lsp.responsiveness.3).
    Snapshot,
    /// Answered against the text it was sent with (§FS-lsp.responsiveness.4).
    Typing(Option<String>),
    /// A request no snapshot answers, refused once the edits before it are checked, so its
    /// error never overtakes their publishes (§FS-lsp.1.6.4). It seals nothing: a later edit
    /// may still merge into the rebuild it waits on.
    Unsupported,
    Shutdown,
}

impl HeldKind {
    /// Whether the request sealed the rebuild it waits on.
    fn seals(&self) -> bool {
        matches!(self, HeldKind::Snapshot | HeldKind::Typing(_))
    }
}

/// A keystroke's message: the edit, or the typing request after it.
enum Key {
    Change(Notification),
    Request(Request),
}

/// The requests a snapshot answers (§FS-lsp.responsiveness.3).
const SNAPSHOT_REQUESTS: [&str; 5] = [
    "textDocument/hover",
    "textDocument/definition",
    "textDocument/references",
    "textDocument/documentHighlight",
    "textDocument/documentLink",
];

/// The requests answered as the user types (§FS-lsp.responsiveness.4).
const TYPING_REQUESTS: [&str; 2] = ["textDocument/completion", "textDocument/onTypeFormatting"];

impl Server {
    /// One client message, in the order the client sent it.
    fn accept(&mut self, message: Message) -> Result<Flow> {
        match message {
            Message::Request(request) => self.request(request),
            Message::Notification(notification) => {
                let method = notification.method.clone();
                // A malformed notification (params that do not deserialize, say) or a failing
                // one goes to stderr and is skipped: one bad message cannot tear the session
                // down (§FS-lsp.1).
                if let Err(err) = self.notification(notification) {
                    eprintln!("grund-lsp: notification `{method}` failed: {err:#}");
                }
                Ok(Flow::Continue)
            }
            Message::Response(_) => Ok(Flow::Continue),
        }
    }

    fn request(&mut self, request: Request) -> Result<Flow> {
        let method = request.method.as_str();
        if TYPING_REQUESTS.contains(&method) {
            // Sent past the counted keystroke: a send meets the writer thread (§AR-lsp.2.1.1).
            if let Some(response) = self.keystroke(Key::Request(request))? {
                self.send_response(response)?;
            }
        } else if method == "shutdown" {
            // §AR-lsp.2.1.2: the session ends after every edit sent before it is checked.
            if self.rebuilds.current() {
                self.shut_down(request.id)?;
                return Ok(Flow::Exit);
            }
            self.hold(request, HeldKind::Shutdown);
        } else if !self.rebuilds.current() {
            // §AR-lsp.2.1.2: one queue, so any rebuild outstanding holds every other request.
            let kind = match SNAPSHOT_REQUESTS.contains(&method) {
                true => HeldKind::Snapshot,
                false => HeldKind::Unsupported,
            };
            self.hold(request, kind);
        } else {
            self.respond(request)?;
        }
        Ok(Flow::Continue)
    }

    fn notification(&mut self, notification: Notification) -> Result<()> {
        match notification.method.as_str() {
            "textDocument/didChange" => self.keystroke(Key::Change(notification)).map(drop),
            "$/cancelRequest" => self.cancel(notification),
            // §AR-lsp.2.1.2: an `exit` behind a held `shutdown` ends the session once it is
            // answered, with no wait for an `exit` that has already come.
            "exit" if self.shutdown_held() => {
                self.rebuilds.exited = true;
                Ok(())
            }
            _ => self.handle_change(notification),
        }
    }

    fn shutdown_held(&self) -> bool {
        self.rebuilds
            .held
            .iter()
            .any(|held| matches!(held.kind, HeldKind::Shutdown))
    }

    /// The loop's whole share of a keystroke (§AR-lsp.2.1.1): store the edit, judge it against the
    /// installed catalog and record it for the worker, or answer a typing request from its own
    /// text by the verdicts already reached. Nothing in it branches on the worker: `dispatch`
    /// merges the recorded edit into the queue and seals for a held request, and the caller sends
    /// the answer, both outside it, as installing a rebuild, publishing and dropping the snapshot
    /// it replaced are. Every pattern it reaches was settled for this thread as its snapshot was
    /// installed, so a key after an install compiles none and allocates no search cache the key
    /// before it did not. Never inlined, so a profiler can count it alone in an optimized build
    /// (§AR-benchmarks.7.3).
    #[inline(never)]
    fn keystroke(&mut self, key: Key) -> Result<Option<Response>> {
        match key {
            Key::Change(notification) => {
                let params: lsp_types::DidChangeTextDocumentParams =
                    serde_json::from_value(notification.params)?;
                let uri = params.text_document.uri;
                if let Some(text) = full_change_text(params.content_changes) {
                    self.open_docs.insert(uri.clone(), text);
                }
                self.versions
                    .insert(uri.clone(), params.text_document.version);
                self.edit(uri, true);
                Ok(None)
            }
            Key::Request(request) => {
                let uri = typing_uri(&request);
                let text = uri.as_ref().and_then(|uri| self.document_text(uri));
                if self.typing_waits(uri.as_ref()) {
                    self.hold(request, HeldKind::Typing(text));
                    return Ok(None);
                }
                Ok(Some(self.typing_answer(request, text)))
            }
        }
    }

    /// Count an edit to `uri`, recorded for `dispatch` to merge into the rebuild still to start
    /// (§AR-lsp.2.1.2), and judged now against the installed catalog where one file's reading can
    /// judge it (§AR-lsp.3.2).
    fn edit(&mut self, uri: Url, readable: bool) {
        let path = uri.to_file_path().ok();
        let verdict = match path.is_some() && readable {
            true => self.judge(&uri),
            false => Verdict::Unreadable,
        };
        let scope = path.map_or(Scope::All, |path| Scope::Paths(BTreeSet::from([path])));
        let seq = self.rebuilds.record(scope);
        self.rebuilds.edited.insert(uri, Edited { seq, verdict });
    }

    /// Count the full scan `initialized` asks for (§AR-lsp.2.1). No edit comes with it, so no
    /// typing request waits for it (§FS-lsp.responsiveness.4): one sent meanwhile is judged
    /// against the startup snapshot, and again against the scan once it is installed.
    fn rescan(&mut self) {
        self.rebuilds.record(Scope::All);
    }

    /// Count an edit that can change any project: everything is rebuilt, and every typing request
    /// waits for it.
    fn stale(&mut self) {
        let seq = self.rebuilds.record(Scope::All);
        self.rebuilds.stale_all = Some(seq);
    }

    /// Whether a typing request in `uri` waits for a rebuild (§FS-lsp.responsiveness.4): after an
    /// edit that changed the declarations of a file the document's project covers, or one no
    /// project covers, or one that can change any project. Every project whose rebuild covers
    /// the edited file holds, not only the one that owns it. It reads the verdicts reached when
    /// each edit arrived and when each catalog was installed, and judges nothing itself.
    fn typing_waits(&self, uri: Option<&Url>) -> bool {
        if self.rebuilds.stale_all.is_some() {
            return true;
        }
        let mut changed = self
            .rebuilds
            .edited
            .iter()
            .filter(|(_, edit)| edit.verdict != Verdict::Stands)
            .peekable();
        if changed.peek().is_none() {
            return false;
        }
        let path = uri.and_then(|uri| uri.to_file_path().ok());
        let owner = path.and_then(|path| self.project_for_path(&path));
        changed.any(|(uri, _)| {
            let Some(path) = uri.to_file_path().ok().map(normalize_path) else {
                return true;
            };
            let covers = |project: &Arc<ProjectSnapshot>| project.might_cover(&path);
            owner.is_none_or(|owner| owner.might_cover(&path))
                || !self.projects.iter().chain(&self.imports).any(covers)
        })
    }

    /// How `uri`'s latest text stands against the installed catalog, by one file's reading.
    fn judge(&self, uri: &Url) -> Verdict {
        match (uri.to_file_path(), self.open_docs.get(uri)) {
            (Ok(path), Some(text)) if self.declarations_stand(&path, text) => Verdict::Stands,
            (Ok(_), _) => Verdict::Changed,
            (Err(()), _) => Verdict::Unreadable,
        }
    }

    /// Judge again, against the catalog just installed from the rebuild ending at `seq`, every
    /// edit it does not reflect, and hold every edit it does reflect as standing (R4). Outside the
    /// counted keystroke, as every install is.
    fn judge_again(&mut self, seq: u64) {
        let verdicts: Vec<Verdict> = self
            .rebuilds
            .edited
            .iter()
            .map(|(uri, edit)| match edit {
                Edited { seq: edited, .. } if *edited <= seq => Verdict::Stands,
                Edited {
                    verdict: Verdict::Unreadable,
                    ..
                } => Verdict::Unreadable,
                _ => self.judge(uri),
            })
            .collect();
        for (edit, verdict) in self.rebuilds.edited.values_mut().zip(verdicts) {
            edit.verdict = verdict;
        }
    }

    /// Hold `request` for the rebuild of every edit sent before it, recording a seal on that
    /// rebuild where the request is answered from its text or its snapshot, which `dispatch`
    /// applies before the next message.
    fn hold(&mut self, request: Request, kind: HeldKind) {
        let seq = self.rebuilds.seq;
        if kind.seals() {
            self.rebuilds.recorded.push(Recorded::Seal(seq));
        }
        self.rebuilds.held.push(Held { seq, request, kind });
    }

    /// `$/cancelRequest` for a held request: answered `RequestCancelled` now, and its rebuild
    /// unsealed, so a client that cancels what it no longer needs leaves no backlog
    /// (§AR-lsp.2.1.2).
    fn cancel(&mut self, notification: Notification) -> Result<()> {
        let params: lsp_types::CancelParams = serde_json::from_value(notification.params)?;
        let id = match params.id {
            lsp_types::NumberOrString::Number(id) => lsp_server::RequestId::from(id),
            lsp_types::NumberOrString::String(id) => lsp_server::RequestId::from(id),
        };
        self.fold();
        let Some(index) = self.rebuilds.held.iter().position(|held| {
            held.request.id == id && !matches!(held.kind, HeldKind::Shutdown)
        }) else {
            return Ok(());
        };
        let held = self.rebuilds.held.remove(index);
        if held.kind.seals() {
            self.rebuilds.unseal(held.seq);
        }
        self.send_response(Response::new_err(
            id,
            lsp_server::ErrorCode::RequestCanceled as i32,
            "request cancelled".into(),
        ))
    }

    /// Answer a request the rebuild ending at its edit released; `rebuilt` is false where that
    /// rebuild panicked, which left the last snapshot standing (§AR-lsp.2.1.1).
    fn answer_held(&mut self, held: Held, rebuilt: bool) -> Result<Flow> {
        match held.kind {
            HeldKind::Snapshot | HeldKind::Unsupported => self.respond(held.request)?,
            // The failed rebuild reflects none of the edits the request waited on: refused, not
            // answered from a catalog they outgrew (§FS-lsp.responsiveness.4).
            HeldKind::Typing(_) if !rebuilt && self.typing_waits(typing_uri(&held.request).as_ref()) => {
                self.send_response(Response::new_err(
                    held.request.id,
                    lsp_server::ErrorCode::RequestFailed as i32,
                    "the rebuild of the edits before this request failed".into(),
                ))?;
            }
            HeldKind::Typing(text) => {
                let response = self.typing_answer(held.request, text);
                self.send_response(response)?;
            }
            HeldKind::Shutdown if self.rebuilds.exited => {
                self.send_response(Response::new_ok(held.request.id, ()))?;
                return Ok(Flow::Exit);
            }
            HeldKind::Shutdown => {
                self.shut_down(held.request.id)?;
                return Ok(Flow::Exit);
            }
        }
        Ok(Flow::Continue)
    }

    /// The answer to a typing request against `text`, the document as it stood when the request
    /// came.
    fn typing_answer(&self, request: Request, text: Option<String>) -> Response {
        let outcome = match request.method.as_str() {
            "textDocument/completion" => self.completion(request.params, text).and_then(to_value),
            _ => self
                .on_type_formatting(request.params, text)
                .and_then(to_value),
        };
        self.outcome_response(request.id, &request.method, outcome)
    }
}

impl Installed {
    /// Whether `text` in place of `path` leaves every catalog that reads the file as it is
    /// (§FS-lsp.responsiveness.4): a project that walked the file reads it again, one that names
    /// it otherwise cannot judge it, and a file no project reads is no file the one-file reading
    /// judges (§AR-lsp.3.2).
    fn declarations_stand(&self, path: &Path, text: &str) -> bool {
        let mut read = false;
        for project in self.projects.iter().chain(&self.imports) {
            let judged = match (&project.show, &project.judge) {
                (Some(show), Some(judge)) => judge.declarations_stand(show, path, text),
                _ => project.might_cover(path).then_some(false),
            };
            match judged {
                Some(false) => return false,
                Some(true) => read = true,
                None => {}
            }
        }
        read
    }
}

/// The document a typing request is about.
fn typing_uri(request: &Request) -> Option<Url> {
    let uri = request.params.get("textDocument")?.get("uri")?.as_str()?;
    Url::parse(uri).ok()
}
