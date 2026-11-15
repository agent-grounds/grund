// Client messages against the rebuilds behind them (§AR-lsp.2.1.2): a typing request is answered
// from its own text while the declarations stand (§FS-lsp.responsiveness.4), a snapshot request
// waits for the snapshot of its edits (§FS-lsp.responsiveness.3), and `shutdown` waits for both.

/// Whether the session goes on after a message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Flow {
    Continue,
    Exit,
}

/// How an edited document's latest text was judged against the installed catalog
/// (§FS-lsp.responsiveness.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Verdict {
    Unjudged,
    Stands,
    Changed,
    /// A close or a save, or a document with no path: no reading of one file judges it.
    Unreadable,
}

impl Verdict {
    /// The verdict once another catalog is installed: a reading is judged again against that
    /// catalog, so a rebuild of an older text never stands in for a later one.
    fn again(self) -> Self {
        match self {
            Verdict::Unreadable => Verdict::Unreadable,
            _ => Verdict::Unjudged,
        }
    }
}

/// A document edited since the installed snapshot: its latest edit, and how it was judged.
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
            self.keystroke(Key::Request(request))?;
        } else if method == "shutdown" {
            // §AR-lsp.2.1.2: the session ends after every edit sent before it is checked.
            if self.rebuilds.current() {
                self.shut_down(request.id)?;
                return Ok(Flow::Exit);
            }
            self.hold(request, HeldKind::Shutdown);
        } else if !self.rebuilds.current() {
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
            "textDocument/didChange" => self.keystroke(Key::Change(notification)),
            "$/cancelRequest" => self.cancel(notification),
            _ => self.handle_change(notification),
        }
    }

    /// The loop's whole share of a keystroke (§AR-lsp.2.1.1): store the edit and count it, or
    /// judge the edits a typing request follows and answer it from its own text. The same work
    /// whether the worker is idle or busy: handing a rebuild over, installing one, publishing and
    /// dropping the snapshot it replaced all happen outside it. Never inlined, so a profiler can
    /// count it alone in an optimized build (§AR-benchmarks.7.3).
    #[inline(never)]
    fn keystroke(&mut self, key: Key) -> Result<()> {
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
                self.edit(uri, Verdict::Unjudged);
                Ok(())
            }
            Key::Request(request) => {
                let uri = typing_uri(&request);
                let text = uri.as_ref().and_then(|uri| self.document_text(uri));
                if self.typing_waits(uri.as_ref()) {
                    self.hold(request, HeldKind::Typing(text));
                    return Ok(());
                }
                self.respond_typing(request, text)
            }
        }
    }

    /// Count an edit to `uri`, merged into the rebuild still to start (§AR-lsp.2.1.2).
    fn edit(&mut self, uri: Url, verdict: Verdict) {
        let path = uri.to_file_path().ok();
        let verdict = if path.is_some() {
            verdict
        } else {
            Verdict::Unreadable
        };
        let scope = path.map_or(Scope::All, |path| Scope::Paths(BTreeSet::from([path])));
        let seq = self.rebuilds.edit(scope);
        self.rebuilds.edited.insert(uri, Edited { seq, verdict });
    }

    /// Count the full scan `initialized` asks for (§AR-lsp.2.1). No edit comes with it, so no
    /// typing request waits for it (§FS-lsp.responsiveness.4): one sent meanwhile is judged
    /// against the startup snapshot, and again against the scan once it is installed.
    fn rescan(&mut self) {
        self.rebuilds.edit(Scope::All);
    }

    /// Count an edit that can change any project: everything is rebuilt, and every typing request
    /// waits for it.
    fn stale(&mut self) {
        let seq = self.rebuilds.edit(Scope::All);
        self.rebuilds.stale_all = Some(seq);
    }

    /// Whether a typing request in `uri` waits for a rebuild (§FS-lsp.responsiveness.4): after an
    /// edit that changed the declarations of a file the document's project covers, or one no
    /// project covers, or one that can change any project. Every project whose rebuild covers
    /// the edited file holds, not only the one that owns it.
    fn typing_waits(&mut self, uri: Option<&Url>) -> bool {
        if self.rebuilds.stale_all.is_some() {
            return true;
        }
        self.judge_edits();
        let mut changed = self
            .rebuilds
            .edited
            .iter()
            .filter(|(_, edit)| matches!(edit.verdict, Verdict::Changed | Verdict::Unreadable))
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

    /// Judge every edit not yet judged against the installed catalog, from the document's text.
    fn judge_edits(&mut self) {
        let unjudged: Vec<(Url, bool)> = self
            .rebuilds
            .edited
            .iter()
            .filter(|(_, edit)| edit.verdict == Verdict::Unjudged)
            .map(|(uri, _)| {
                let stands = match (uri.to_file_path(), self.open_docs.get(uri)) {
                    (Ok(path), Some(text)) => self.declarations_stand(&path, text),
                    _ => false,
                };
                (uri.clone(), stands)
            })
            .collect();
        for (uri, stands) in unjudged {
            if let Some(edit) = self.rebuilds.edited.get_mut(&uri) {
                edit.verdict = if stands {
                    Verdict::Stands
                } else {
                    Verdict::Changed
                };
            }
        }
    }

    /// Hold `request` for the rebuild of every edit sent before it, sealing that rebuild where
    /// the request is answered from its text or its snapshot.
    fn hold(&mut self, request: Request, kind: HeldKind) {
        let seq = self.rebuilds.seq;
        if kind.seals() {
            self.seal(seq);
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

    fn answer_held(&mut self, held: Held) -> Result<Flow> {
        match held.kind {
            HeldKind::Snapshot | HeldKind::Unsupported => self.respond(held.request)?,
            HeldKind::Typing(text) => self.respond_typing(held.request, text)?,
            HeldKind::Shutdown => {
                self.shut_down(held.request.id)?;
                return Ok(Flow::Exit);
            }
        }
        Ok(Flow::Continue)
    }

    /// Answer a typing request against `text`, the document as it stood when the request came.
    fn respond_typing(&self, request: Request, text: Option<String>) -> Result<()> {
        let outcome = match request.method.as_str() {
            "textDocument/completion" => self.completion(request.params, text).and_then(to_value),
            _ => self
                .on_type_formatting(request.params, text)
                .and_then(to_value),
        };
        self.send_outcome(request.id, &request.method, outcome)
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
