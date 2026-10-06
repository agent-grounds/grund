// Private, feature-gated observation of production watch boundaries (§AR-bindings.3).

#[derive(Clone, Debug)]
pub enum WatchObservation {
    Input(grund_core::CheckInput),
    Subscribed(PathBuf, bool),
    Retired(PathBuf),
    Recovered,
    Scanning,
    Scanned,
    Publishing,
    StdoutFlushed,
    StderrFlushed,
    Completed(u8, Vec<u8>, Vec<u8>),
    Stopped,
}

static WATCH_TEST_CONTROL: std::sync::Mutex<Option<std::sync::Arc<watch_test::Control>>> =
    std::sync::Mutex::new(None);
static WATCH_TEST_SESSION: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn watch_observe_event(mut event: WatchObservation) {
    let control = WATCH_TEST_CONTROL.lock().unwrap().clone();
    if let Some(control) = control {
        if let WatchObservation::Completed(_, stdout, stderr) = &mut event {
            let captured = control.take_output();
            *stdout = captured[0].clone();
            *stderr = captured[1].clone();
        }
        (control.observer)(event);
    }
}

fn watch_inject_failure_event(operation: &str) -> Result<(), String> {
    let control = WATCH_TEST_CONTROL.lock().unwrap().clone();
    if let Some(control) = control {
        let mut failure = control.failure.lock().unwrap();
        if failure.as_deref() == Some(operation) {
            failure.take();
            return Err(format!("{operation}: injected native watcher failure"));
        }
    }
    Ok(())
}

/// Accessible only to test-watch builds; no environment variables, production
/// control channels, files, or output markers (§AR-bindings.3).
pub mod watch_test {
    use super::*;
    use std::sync::{Arc, Mutex};

    pub struct Control {
        pub(super) observer: Box<dyn Fn(WatchObservation) + Send + Sync>,
        pub(super) failure: Mutex<Option<String>>,
        pub(super) notices: Mutex<Option<std::sync::mpsc::SyncSender<WatchNotice>>>,
        pub(super) loss: Mutex<Option<Arc<std::sync::atomic::AtomicBool>>>,
        pub(super) runtime_failure: Mutex<Option<Arc<Mutex<Option<String>>>>>,
        pub(super) output: Mutex<[Vec<u8>; 2]>,
        pub(super) terminal: Mutex<Option<bool>>,
        pub(super) retained: Mutex<Vec<Arc<dyn std::any::Any + Send + Sync>>>,
    }

    impl Control {
        pub fn new(observer: impl Fn(WatchObservation) + Send + Sync + 'static) -> Arc<Self> {
            Arc::new(Self {
                observer: Box::new(observer),
                failure: Mutex::new(None),
                notices: Mutex::new(None),
                loss: Mutex::new(None),
                runtime_failure: Mutex::new(None),
                output: Mutex::new(Default::default()),
                terminal: Mutex::new(None),
                retained: Mutex::new(Vec::new()),
            })
        }
        pub fn fail_next(&self, operation: &str) {
            *self.failure.lock().unwrap() = Some(operation.to_string());
        }
        pub fn terminal(&self, eligible: bool) {
            *self.terminal.lock().unwrap() = Some(eligible);
        }
        pub fn take_output(&self) -> [Vec<u8>; 2] {
            std::mem::take(&mut *self.output.lock().unwrap())
        }
        pub fn interrupt(&self) {
            #[cfg(unix)]
            watch_sigint(libc::SIGINT);
            #[cfg(windows)]
            unsafe {
                watch_console_interrupt(windows_sys::Win32::System::Console::CTRL_C_EVENT);
            }
        }
        pub fn notify(&self, notice: WatchNotice) {
            let tx = self
                .notices
                .lock()
                .unwrap()
                .as_ref()
                .expect("watcher ready")
                .clone();
            let loss = self
                .loss
                .lock()
                .unwrap()
                .as_ref()
                .expect("watcher ready")
                .clone();
            let failure = self
                .runtime_failure
                .lock()
                .unwrap()
                .as_ref()
                .expect("watcher ready")
                .clone();
            watch_queue_notice(&tx, &loss, &failure, notice);
        }
        pub fn saturate(&self) {
            self.loss
                .lock()
                .unwrap()
                .as_ref()
                .expect("watcher ready")
                .store(true, std::sync::atomic::Ordering::Release);
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    pub struct Snapshot {
        pub stdout: Vec<u8>,
        pub stderr: Vec<u8>,
        pub status: u8,
        pub format: String,
    }

    pub fn one_shot(
        opts: CheckOpts,
        selection: &CheckFindingSelection,
        format: Option<&str>,
    ) -> Snapshot {
        let report = prepare_check_run(opts, selection, format);
        Snapshot {
            stdout: report.stdout,
            stderr: report.stderr,
            status: report.status,
            format: report.format,
        }
    }

    pub fn run(
        opts: CheckOpts,
        selection: CheckFindingSelection,
        format: Option<String>,
        control: Arc<Control>,
    ) -> u8 {
        let _session = WATCH_TEST_SESSION
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *WATCH_TEST_CONTROL.lock().unwrap() = Some(control);
        struct Reset;
        impl Drop for Reset {
            fn drop(&mut self) {
                *WATCH_TEST_CONTROL.lock().unwrap() = None;
            }
        }
        let _reset = Reset;
        run_check_watch(opts, selection, format)
    }

    /// The actual idle policy with supplied clock values, not a test replica.
    pub fn debounce_deadline(events_ms: &[u64], now_ms: u64) -> Option<u64> {
        let mut debounce = WatchDebounce::default();
        for event in events_ms {
            debounce.event(std::time::Duration::from_millis(*event));
        }
        debounce
            .remaining(std::time::Duration::from_millis(now_ms))
            .map(|wait| wait.as_millis() as u64)
    }

    pub fn coverage(input: grund_core::CheckInput) -> BTreeMap<PathBuf, bool> {
        WatchSubscriptions::coverage(&input)
    }
    pub fn event_relevant(inputs: &[grund_core::CheckInput], event: &notify::Event) -> bool {
        watch_event_relevant(&inputs.iter().cloned().collect(), event)
    }
}

fn watch_capture_write(stream: usize, bytes: &[u8]) {
    if let Some(control) = WATCH_TEST_CONTROL.lock().unwrap().as_ref() {
        control.output.lock().unwrap()[stream].extend_from_slice(bytes);
    }
}

/// An injected `release` keeps the next backend's callback alive past its
/// release, as an orphaned native request does (§FS-check.6.3.3).
fn watch_retain_callback(callback: std::sync::Arc<dyn std::any::Any + Send + Sync>) {
    if watch_inject_failure_event("release").is_err() {
        if let Some(control) = WATCH_TEST_CONTROL.lock().unwrap().as_ref() {
            control.retained.lock().unwrap().push(callback);
        }
    }
}

fn watch_test_terminal() -> Option<bool> {
    WATCH_TEST_CONTROL
        .lock()
        .unwrap()
        .as_ref()
        .and_then(|control| *control.terminal.lock().unwrap())
}

fn watch_attach_control(
    tx: std::sync::mpsc::SyncSender<WatchNotice>,
    lost: std::sync::Arc<std::sync::atomic::AtomicBool>,
    runtime_failure: std::sync::Arc<std::sync::Mutex<Option<String>>>,
) {
    if let Some(control) = WATCH_TEST_CONTROL.lock().unwrap().as_ref() {
        *control.notices.lock().unwrap() = Some(tx);
        *control.loss.lock().unwrap() = Some(lost);
        *control.runtime_failure.lock().unwrap() = Some(runtime_failure);
    }
}
