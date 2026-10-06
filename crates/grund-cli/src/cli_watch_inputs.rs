// Native subscriptions derived from shared checking inputs (§FS-check.6.1.3).

type WatchNotice = notify::Result<notify::Event>;

struct WatchSubscriptions {
    watcher: NativeWatchBackend,
    tx: std::sync::mpsc::SyncSender<WatchNotice>,
    lost: std::sync::Arc<std::sync::atomic::AtomicBool>,
    runtime_failure: std::sync::Arc<std::sync::Mutex<Option<String>>>,
    inputs: BTreeSet<grund_core::CheckInput>,
    desired: BTreeSet<grund_core::CheckInput>,
    handles: BTreeMap<PathBuf, bool>,
    failure: Option<String>,
}

/// A bounded native queue. Saturation records uncertainty instead of blocking
/// the backend or accumulating work (§FS-check.6.1.2, §FS-check.6.1.4).
fn native_watch_backend(
    tx: std::sync::mpsc::SyncSender<WatchNotice>,
    lost: std::sync::Arc<std::sync::atomic::AtomicBool>,
    runtime_failure: std::sync::Arc<std::sync::Mutex<Option<String>>>,
) -> Result<NativeWatchBackend, String> {
    let (closed, finished) = std::sync::mpsc::channel();
    struct Closed(std::sync::mpsc::Sender<()>);
    impl Drop for Closed {
        fn drop(&mut self) {
            let _ = self.0.send(());
        }
    }
    let closed = Closed(closed);
    let watcher = notify::recommended_watcher(move |event: WatchNotice| {
        let _alive = &closed;
        watch_queue_notice(&tx, &lost, &runtime_failure, event);
    })
    .map_err(|err| format!("setting up native watcher: {err}"))?;
    Ok(NativeWatchBackend {
        watcher: Some(watcher),
        finished,
    })
}

/// Shared with failure-injection builds so saturation exercises the real
/// callback policy (§FS-check.6.1.4, §FS-check.6.3.2).
fn watch_queue_notice(
    tx: &std::sync::mpsc::SyncSender<WatchNotice>,
    lost: &std::sync::atomic::AtomicBool,
    failure: &std::sync::Mutex<Option<String>>,
    event: WatchNotice,
) {
    if let Err(err) = &event {
        *failure
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(format!(
            "receiving native notifications for {:?}: {err}",
            err.paths
        ));
    }
    if event
        .as_ref()
        .is_ok_and(|e| matches!(e.kind, notify::EventKind::Access(_)))
    {
        return;
    }
    if let Err(std::sync::mpsc::TrySendError::Full(_)) = tx.try_send(event) {
        lost.store(true, std::sync::atomic::Ordering::Release);
    }
}

/// Some notify backends request asynchronous shutdown in Drop. Await destruction
/// of their callback, after their native handles close, before finishing our
/// session; macOS also joins its native worker (§FS-check.6.3.3).
struct NativeWatchBackend {
    watcher: Option<notify::RecommendedWatcher>,
    finished: std::sync::mpsc::Receiver<()>,
}

impl NativeWatchBackend {
    fn watch(&mut self, path: &Path, mode: notify::RecursiveMode) -> notify::Result<()> {
        use notify::Watcher;
        self.watcher.as_mut().unwrap().watch(path, mode)
    }
    fn unwatch(&mut self, path: &Path) -> notify::Result<()> {
        use notify::Watcher;
        self.watcher.as_mut().unwrap().unwatch(path)
    }
}
impl Drop for NativeWatchBackend {
    fn drop(&mut self) {
        drop(self.watcher.take());
        let _ = self.finished.recv();
    }
}

impl WatchSubscriptions {
    fn new(
        tx: std::sync::mpsc::SyncSender<WatchNotice>,
        lost: std::sync::Arc<std::sync::atomic::AtomicBool>,
        runtime_failure: std::sync::Arc<std::sync::Mutex<Option<String>>>,
    ) -> Result<Self, String> {
        Ok(Self {
            watcher: native_watch_backend(tx.clone(), lost.clone(), runtime_failure.clone())?,
            tx,
            lost,
            runtime_failure,
            inputs: BTreeSet::new(),
            desired: BTreeSet::new(),
            handles: BTreeMap::new(),
            failure: None,
        })
    }

    /// Parent anchors are shallow; only resolved trees recurse. Absent paths
    /// use their nearest existing directory (§FS-check.6.1.3).
    fn coverage(input: &grund_core::CheckInput) -> BTreeMap<PathBuf, bool> {
        let mut coverage = BTreeMap::new();
        if input.recursive && input.path.is_dir() {
            coverage.insert(input.path.clone(), true);
        }
        let mut parent = input.path.parent();
        while let Some(path) = parent {
            if path.is_dir() {
                coverage.entry(path.to_path_buf()).or_insert(false);
                // §FS-check.6.1.3: also anchor replacement of this directory,
                // including a root with invalid config and no ancestor climb.
                if let Some(anchor) = path.parent().filter(|anchor| anchor.is_dir()) {
                    coverage.entry(anchor.to_path_buf()).or_insert(false);
                }
                break;
            }
            parent = path.parent();
        }
        coverage
    }

    /// Called synchronously by the engine before reads, including Rayon reads.
    /// Additions happen immediately; retirement waits for rediscovery to finish
    /// (§FS-check.6.1.1).
    fn observe(&mut self, input: grund_core::CheckInput) -> bool {
        self.desired.insert(input.clone());
        self.inputs.insert(input.clone());
        if self.failure.is_some() {
            return false;
        }
        for (path, recursive) in Self::coverage(&input) {
            if let Err(err) = self.subscribe(path, recursive) {
                self.failure = Some(err);
                return false;
            }
        }
        watch_observe!(WatchObservation::Input(input));
        true
    }

    fn subscribe(&mut self, path: PathBuf, recursive: bool) -> Result<(), String> {
        if self
            .handles
            .get(&path)
            .is_some_and(|old| *old || !recursive)
        {
            return Ok(());
        }
        watch_inject_failure!("subscribe")?;
        let mode = if recursive {
            notify::RecursiveMode::Recursive
        } else {
            notify::RecursiveMode::NonRecursive
        };
        self.watcher
            .watch(&path, mode)
            .map_err(|err| format!("subscribing to {}: {err}", path.display()))?;
        self.handles.insert(path.clone(), recursive);
        watch_observe!(WatchObservation::Subscribed(path, recursive));
        Ok(())
    }

    fn relevant(&self, event: &notify::Event) -> bool {
        watch_event_relevant(&self.inputs, event)
    }

    /// Build a replacement backend under retained coverage before dropping the
    /// old one. This repairs stale inode watches as well as loss/overflow
    /// (§FS-check.6.1.4). Failure is fatal, never a stale resident loop.
    fn recover(&mut self) -> Result<(), String> {
        watch_inject_failure!("recover")?;
        let mut replacement = native_watch_backend(
            self.tx.clone(),
            self.lost.clone(),
            self.runtime_failure.clone(),
        )?;
        let mut handles = BTreeMap::new();
        for input in &self.inputs {
            for (path, recursive) in Self::coverage(input) {
                handles
                    .entry(path)
                    .and_modify(|old| *old |= recursive)
                    .or_insert(recursive);
            }
        }
        for (path, recursive) in &handles {
            let mode = if *recursive {
                notify::RecursiveMode::Recursive
            } else {
                notify::RecursiveMode::NonRecursive
            };
            replacement
                .watch(path, mode)
                .map_err(|err| format!("recovering watch for {}: {err}", path.display()))?;
            watch_observe!(WatchObservation::Subscribed(path.clone(), *recursive));
        }
        self.watcher = replacement;
        self.handles = handles;
        watch_observe!(WatchObservation::Recovered);
        Ok(())
    }

    fn reconcile(&mut self, usable: bool) -> Result<(), String> {
        if let Some(err) = self.failure.take() {
            return Err(err);
        }
        if usable {
            self.inputs = self.desired.clone();
        }
        let mut wanted = BTreeMap::new();
        for input in &self.inputs {
            for (path, recursive) in Self::coverage(input) {
                wanted
                    .entry(path)
                    .and_modify(|old| *old |= recursive)
                    .or_insert(recursive);
            }
        }
        // Observe already adds all new coverage before this retirement phase.
        for path in self
            .handles
            .keys()
            .filter(|path| !wanted.contains_key(*path))
            .cloned()
            .collect::<Vec<_>>()
        {
            self.watcher
                .unwatch(&path)
                .map_err(|err| format!("retiring watch for {}: {err}", path.display()))?;
            self.handles.remove(&path);
            watch_observe!(WatchObservation::Retired(path));
        }
        Ok(())
    }
}

/// Shallow ancestors only react to exact inputs or their replacement/creation;
/// unrelated siblings do not cause runs (§FS-check.6.1.3).
fn watch_event_relevant(inputs: &BTreeSet<grund_core::CheckInput>, event: &notify::Event) -> bool {
    if matches!(event.kind, notify::EventKind::Access(_)) {
        return false;
    }
    if event.need_rescan()
        || event.paths.is_empty()
        || matches!(
            event.kind,
            notify::EventKind::Any | notify::EventKind::Other
        )
    {
        return true;
    }
    event.paths.iter().any(|path| {
        inputs.iter().any(|input| {
            path == &input.path
                || input.path.starts_with(path)
                || (input.recursive && path.starts_with(&input.path))
        })
    })
}
