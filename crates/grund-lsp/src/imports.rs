// Import projects and the watch over their roots (§AR-lsp.2.1, §FS-imports.editor).

/// How the server hears of a change under an import root outside every open folder,
/// where the editor reports nothing on its own (§AR-lsp.2.1): a watcher registered
/// with a client that can register relative patterns, else the server's own.
struct ImportWatch {
    /// Whether the client registers `workspace/didChangeWatchedFiles` dynamically,
    /// relative patterns included.
    client: bool,
    /// Set once the handshake is done, before which the client takes no request.
    ready: bool,
    roots: BTreeSet<PathBuf>,
    registration: Option<String>,
    requests: u32,
    native: Option<grund_core::native_watch::notify::RecommendedWatcher>,
    events: (
        crossbeam_channel::Sender<()>,
        crossbeam_channel::Receiver<()>,
    ),
}

impl Default for ImportWatch {
    fn default() -> Self {
        Self {
            client: false,
            ready: false,
            roots: BTreeSet::new(),
            registration: None,
            requests: 0,
            native: None,
            events: crossbeam_channel::unbounded(),
        }
    }
}

fn client_registers_watchers(params: &InitializeParams) -> bool {
    params
        .capabilities
        .workspace
        .as_ref()
        .and_then(|workspace| workspace.did_change_watched_files.as_ref())
        .is_some_and(|watched| {
            watched.dynamic_registration == Some(true)
                && watched.relative_pattern_support == Some(true)
        })
}

/// What wakes the event loop: a client message, the native watcher, or the
/// client's channel closing.
enum Wake {
    Client(Message),
    Disk,
    Closed,
}

/// The quiet a burst of native events waits out before it is one rebuild.
const SETTLE: std::time::Duration = std::time::Duration::from_millis(50);

/// The project among `projects` whose root contains `path` most deeply.
fn deepest_root<'a>(projects: &'a [ProjectSnapshot], path: &Path) -> Option<&'a ProjectSnapshot> {
    projects
        .iter()
        .filter_map(|project| Some((project.containing_root_depth(path)?, project)))
        .max_by_key(|(depth, _)| *depth)
        .map(|(_, project)| project)
}

impl Server {
    fn wake(&self) -> Wake {
        crossbeam_channel::select! {
            recv(self.connection.receiver) -> message => message.map_or(Wake::Closed, Wake::Client),
            recv(self.watch.events.1) -> _ => {
                while self.watch.events.1.recv_timeout(SETTLE).is_ok() {}
                Wake::Disk
            }
        }
    }

    /// Every import root a folder's run mounted, transitive ones included.
    fn import_roots(&self) -> BTreeSet<PathBuf> {
        self.projects
            .iter()
            .filter_map(|project| project.show.as_ref())
            .flat_map(grund_core::LspShowContext::import_roots)
            .collect()
    }

    /// One import project per import root no folder's project holds, loaded as that
    /// root's standalone run and rebuilt where `changed` says a folder's would be.
    /// One that will not load keeps its last snapshot: its refusal is the
    /// consumer's to publish, on the import line (§FS-imports.editor).
    fn rebuild_imports(&mut self, changed: Option<&Path>, overlays: &BTreeMap<PathBuf, String>) {
        let roots = self.import_roots();
        let mut previous: BTreeMap<PathBuf, ProjectSnapshot> = std::mem::take(&mut self.imports)
            .into_iter()
            .map(|project| (project.root.clone(), project))
            .collect();
        for root in roots {
            if deepest_root(&self.projects, &root).is_some() {
                continue;
            }
            let existing = previous.remove(&root);
            if let (Some(path), Some(project)) = (changed, &existing)
                && !project.might_cover(path)
            {
                self.imports.extend(existing);
                continue;
            }
            let opts = LspSnapshotOpts {
                path: root.clone(),
                path_provided: true,
                open_documents: overlays.clone(),
            };
            match lsp_snapshot_with_completion(opts) {
                Ok(snapshot) => self
                    .imports
                    .push(ProjectSnapshot::with_completion(root, snapshot)),
                Err(_) => self.imports.extend(existing),
            }
        }
    }

    /// Whether the project answering `path` is an import's, which offers no edit.
    fn import_owns(&self, path: &Path) -> bool {
        self.project_for_path(path).is_some_and(|owner| {
            self.imports
                .iter()
                .any(|import| std::ptr::eq(import, owner))
        })
    }

    /// Watch the import roots outside every open folder, replacing the watch
    /// whenever that set changes (§AR-lsp.2.1).
    fn watch_imports(&mut self) {
        if !self.watch.ready {
            return;
        }
        let roots: BTreeSet<PathBuf> = self
            .import_roots()
            .into_iter()
            .filter(|root| {
                !self
                    .workspace_folders
                    .values()
                    .any(|folder| root.starts_with(folder))
            })
            .collect();
        if roots == self.watch.roots {
            return;
        }
        self.watch.roots = roots;
        if self.watch.client {
            if let Err(err) = self.register_watchers() {
                eprintln!("grund-lsp: registering the import watchers failed: {err:#}");
            }
            return;
        }
        // The old watch ends before the new one starts, so no root is watched twice.
        self.watch.native = None;
        if self.watch.roots.is_empty() {
            return;
        }
        let roots = self.watch.roots.iter().cloned().collect::<Vec<_>>();
        let events = self.watch.events.0.clone();
        match grund_core::native_watch::watch_roots(&roots, move || {
            let _ = events.send(());
        }) {
            Ok(watcher) => self.watch.native = Some(watcher),
            Err(err) => eprintln!("grund-lsp: watching the import roots failed: {err}"),
        }
    }

    /// Replace the client's watcher registration with one relative pattern per root.
    fn register_watchers(&mut self) -> Result<()> {
        const METHOD: &str = "workspace/didChangeWatchedFiles";
        if let Some(id) = self.watch.registration.take() {
            let gone = json!({"unregisterations": [{"id": id, "method": METHOD}]});
            self.client_request("client/unregisterCapability", gone)?;
        }
        if self.watch.roots.is_empty() {
            return Ok(());
        }
        let watchers = self
            .watch
            .roots
            .iter()
            .filter_map(|root| Url::from_directory_path(root).ok())
            .map(|base| json!({"globPattern": {"baseUri": base, "pattern": "**/*"}}))
            .collect::<Vec<_>>();
        let id = format!("grund-imports-{}", self.watch.requests);
        let registration =
            json!({"id": id, "method": METHOD, "registerOptions": {"watchers": watchers}});
        self.client_request(
            "client/registerCapability",
            json!({"registrations": [registration]}),
        )?;
        self.watch.registration = Some(id);
        Ok(())
    }

    /// Answer `shutdown` and wait for `exit`, as `Connection::handle_shutdown` does, but
    /// drop a client's answer to a registration that arrives in between rather than
    /// fail the session on it.
    fn shut_down(&self, id: lsp_server::RequestId) -> Result<()> {
        self.send_response(Response::new_ok(id, ()))?;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            let left = deadline.saturating_duration_since(std::time::Instant::now());
            match self.connection.receiver.recv_timeout(left)? {
                Message::Notification(notification) if notification.method == "exit" => {
                    return Ok(());
                }
                Message::Response(_) => {}
                message => anyhow::bail!("unexpected message during shutdown: {message:?}"),
            }
        }
    }

    /// Send the client a request whose answer the event loop drops.
    fn client_request(&mut self, method: &str, params: Value) -> Result<()> {
        self.watch.requests += 1;
        let id = lsp_server::RequestId::from(format!("grund-{}", self.watch.requests));
        let request = Request::new(id, method.to_string(), params);
        self.connection.sender.send(Message::Request(request))?;
        Ok(())
    }
}
