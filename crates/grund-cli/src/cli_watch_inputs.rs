// Native subscriptions derived from shared checking inputs (§FS-check.6.1.3).

type WatchNotice = notify::Result<notify::Event>;

struct WatchSubscriptions {
    watcher: NativeWatchBackend,
    tx: std::sync::mpsc::SyncSender<WatchNotice>,
    lost: std::sync::Arc<std::sync::atomic::AtomicBool>,
    runtime_failure: std::sync::Arc<std::sync::Mutex<Option<String>>>,
    inputs: BTreeSet<grund_core::CheckInput>,
    desired: BTreeSet<grund_core::CheckInput>,
    // §FS-check.6.1.3: native ownership is physical; inputs remain lexical too.
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
    // §FS-check.6.1: unsupported platforms must not select a polling backend.
    if !cfg!(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "windows"
    )) {
        return Err("native watch notifications are unsupported on this platform".to_string());
    }
    let (closed, finished) = std::sync::mpsc::channel();
    struct Closed(std::sync::mpsc::Sender<()>);
    impl Drop for Closed {
        fn drop(&mut self) {
            let _ = self.0.send(());
        }
    }
    let closed = std::sync::Arc::new(Closed(closed));
    #[cfg(feature = "test-watch")]
    watch_retain_callback(closed.clone());
    use notify::Watcher;
    // §FS-check.6.1.3: followed targets receive explicit physical coverage;
    // recursive traversal must not register their lexical aliases again.
    let watcher = notify::RecommendedWatcher::new(
        move |event: WatchNotice| {
            let _alive = &closed;
            watch_queue_notice(&tx, &lost, &runtime_failure, event);
        },
        notify::Config::default().with_follow_symlinks(false),
    )
    .map_err(|err| format!("setting up native watcher: {err}"))?;
    Ok(NativeWatchBackend {
        watcher: Some(watcher),
        finished,
        registered: BTreeSet::new(),
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
/// session; macOS also joins its native worker. The wait is bounded: a backend
/// that keeps its callback is a fatal failure, never a hang (§FS-check.6.3.3).
struct NativeWatchBackend {
    watcher: Option<notify::RecommendedWatcher>,
    finished: std::sync::mpsc::Receiver<()>,
    registered: BTreeSet<PathBuf>,
}

impl NativeWatchBackend {
    /// ReadDirectoryChangesW keys registrations by path, so a second one orphans
    /// the first request, and that request keeps the callback past release.
    /// Refuse it on every platform (§FS-check.6.1.3, §FS-check.6.3.3).
    fn watch(&mut self, path: &Path, mode: notify::RecursiveMode) -> notify::Result<()> {
        use notify::Watcher;
        if self.registered.contains(path) {
            return Err(notify::Error::generic("directory already registered on this backend")
                .add_path(path.to_path_buf()));
        }
        self.watcher.as_mut().unwrap().watch(path, mode)?;
        self.registered.insert(path.to_path_buf());
        Ok(())
    }

    fn close(&mut self) -> Result<(), String> {
        let Some(watcher) = self.watcher.take() else {
            return Ok(());
        };
        drop(watcher);
        match self.finished.recv_timeout(std::time::Duration::from_secs(5)) {
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Err(
                "releasing native subscriptions: the backend kept its callback past 5 seconds"
                    .to_string(),
            ),
            _ => Ok(()),
        }
    }
}
impl Drop for NativeWatchBackend {
    fn drop(&mut self) {
        let _ = self.close();
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
        // §FS-check.6.1.3: intermediate directory links can retarget while
        // their old physical directory remains. Keep each link's own parent.
        for path in input.path.ancestors() {
            if std::fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink()) {
                if let Some(anchor) = path.parent().filter(|anchor| anchor.is_dir()) {
                    coverage.entry(anchor.to_path_buf()).or_insert(false);
                }
            }
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
        let path = std::fs::canonicalize(&path)
            .map_err(|err| format!("resolving watch for {}: {err}", path.display()))?;
        if self.handles.iter().any(|(old, mode)| {
            (*mode && path.starts_with(old)) || (old == &path && !recursive)
        })
        {
            return Ok(());
        }
        watch_inject_failure!("subscribe")?;
        // §FS-check.6.1.3: a shallow anchor that becomes recursive coverage moves
        // to a complete replacement backend instead of registering its directory twice.
        if self.handles.contains_key(&path) {
            let mut handles = self.handles.clone();
            handles.insert(path, true);
            return self.replace_coverage(handles);
        }
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
        let handles = self.physical_coverage()?;
        self.replace_coverage(handles)?;
        watch_observe!(WatchObservation::Recovered);
        Ok(())
    }

    /// Deduplicate aliases and subsume shallow/recursive children under a
    /// recursive physical root (§FS-check.6.1.3). Recompute after retargeting.
    fn physical_coverage(&self) -> Result<BTreeMap<PathBuf, bool>, String> {
        let mut handles = BTreeMap::new();
        for input in &self.inputs {
            for (path, recursive) in Self::coverage(input) {
                let physical = std::fs::canonicalize(&path)
                    .map_err(|err| format!("resolving watch for {}: {err}", path.display()))?;
                handles
                    .entry(physical)
                    .and_modify(|old| *old |= recursive)
                    .or_insert(recursive);
            }
        }
        let recursive = handles.iter().filter(|(_, mode)| **mode)
            .map(|(path, _)| path.clone()).collect::<Vec<_>>();
        handles.retain(|path, _| !recursive.iter().any(|root| path != root && path.starts_with(root)));
        Ok(handles)
    }

    /// A backend can share handles even between recursive parents and children.
    /// Install the complete retained set before releasing its old owner instead
    /// of unwatching an alias or subtree still needed (§FS-check.6.1.1, §FS-check.6.1.4).
    fn replace_coverage(&mut self, handles: BTreeMap<PathBuf, bool>) -> Result<(), String> {
        let mut replacement = native_watch_backend(
            self.tx.clone(),
            self.lost.clone(),
            self.runtime_failure.clone(),
        )?;
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
        std::mem::replace(&mut self.watcher, replacement).close()?;
        for _path in self.handles.keys().filter(|path| !handles.contains_key(*path)) {
            watch_observe!(WatchObservation::Retired(_path.clone()));
        }
        self.handles = handles;
        Ok(())
    }

    fn reconcile(&mut self, usable: bool) -> Result<(), String> {
        if let Some(err) = self.failure.take() {
            return Err(err);
        }
        if usable {
            self.inputs = self.desired.clone();
        }
        let wanted = self.physical_coverage()?;
        if wanted != self.handles {
            self.replace_coverage(wanted)?;
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
