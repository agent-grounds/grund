// Synchronous terminal coordinator (§FS-check.6.1.2, §AR-bindings.3).

/// Keep time policy independent of native event counts. During work the bounded
/// backend queue is drained into a single pending burst (§FS-check.6.1.2).
#[derive(Default)]
struct WatchDebounce {
    first: Option<std::time::Duration>,
    last: std::time::Duration,
}

impl WatchDebounce {
    fn event(&mut self, now: std::time::Duration) {
        self.first.get_or_insert(now);
        self.last = now;
    }
    fn remaining(&self, now: std::time::Duration) -> Option<std::time::Duration> {
        let first = self.first?;
        let deadline = (first + std::time::Duration::from_millis(500))
            .min(self.last + std::time::Duration::from_millis(100));
        Some(deadline.saturating_sub(now))
    }
}

fn command_check_watch(
    opts: CheckOpts,
    selection: CheckFindingSelection,
    format: Option<String>,
) -> ExitCode {
    let status = run_check_watch(opts, selection, format);
    ExitCode::from(status)
}

/// All checking and publication stays on this thread. Native callbacks only
/// enqueue bounded notifications; shutdown drops/joins the backend before
/// reporting completion (§FS-check.6.3.3).
fn run_check_watch(
    mut opts: CheckOpts,
    selection: CheckFindingSelection,
    format: Option<String>,
) -> u8 {
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    };
    use std::time::{Duration, Instant};
    struct Stopped;
    impl Drop for Stopped {
        fn drop(&mut self) {
            watch_observe!(WatchObservation::Stopped);
        }
    }
    let _stopped = Stopped;
    let mut screen = WatchScreen::new();
    // §FS-check.6.1.3: retain the lexical location across deletion/recreation
    // of the process's working directory; a relative `.` would name its old inode.
    if !opts.path.is_absolute() {
        opts.path = match std::env::current_dir() {
            Ok(cwd) => cwd.join(&opts.path),
            Err(err) => return watch_fatal(&mut screen, format!("locating watched path: {err}")),
        };
    }
    let signal = match WatchSignal::install() {
        Ok(signal) => signal,
        Err(err) => return watch_fatal(&mut screen, err),
    };
    let (tx, rx) = std::sync::mpsc::sync_channel(128);
    let lost = Arc::new(AtomicBool::new(false));
    let runtime_failure = Arc::new(Mutex::new(None));
    #[cfg(feature = "test-watch")]
    watch_attach_control(tx.clone(), lost.clone(), runtime_failure.clone());
    let subscriptions = match watch_inject_failure!("setup")
        .and_then(|_| WatchSubscriptions::new(tx, lost.clone(), runtime_failure.clone()))
    {
        Ok(subscriptions) => Arc::new(Mutex::new(subscriptions)),
        Err(err) => return watch_fatal(&mut screen, err),
    };
    let inputs = subscriptions.clone();
    let observer: grund_core::CheckInputObserver = Arc::new(move |input| {
        inputs
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .observe(input)
    });
    let clock = Instant::now();
    let mut last = None;
    let mut failure = None;
    let mut debounce = WatchDebounce::default();
    let mut run = true;
    let mut recovery = false;
    loop {
        if let Some(err) = runtime_failure
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
        {
            failure = Some(err);
            break;
        }
        if signal.interrupted() {
            break;
        }
        if run {
            let refreshed = {
                let mut guard = subscriptions
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                guard.desired.clear();
                if recovery { guard.recover() } else { Ok(()) }
            };
            if let Err(err) = refreshed {
                failure = Some(err);
                break;
            }
            debounce = WatchDebounce::default();
            recovery = false;
            watch_observe!(WatchObservation::Scanning);
            let output = grund_core::with_check_input_observer(Some(observer.clone()), || {
                prepare_check_run(opts.clone(), &selection, format.as_deref())
            });
            watch_observe!(WatchObservation::Scanned);
            if let Some(err) = runtime_failure
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take()
            {
                failure = Some(err);
                break;
            }
            if signal.interrupted() {
                break;
            }
            let reconciled = subscriptions
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .reconcile(output.status != 2);
            if let Err(err) = reconciled {
                failure = Some(err);
                break;
            }
            if signal.interrupted() {
                break;
            }
            if let Err(err) = screen.before_publication(&output.format) {
                failure = Some(err.to_string());
                break;
            }
            // Interrupts after this boundary finish both streams and flushes.
            watch_observe!(WatchObservation::Publishing);
            if let Err(err) = publish_check_run(&output) {
                failure = Some(format!("publishing check report: {err}"));
                break;
            }
            last = Some(output.status);
            watch_observe!(WatchObservation::Completed(
                output.status,
                output.stdout,
                output.stderr,
            ));
        }
        if signal.interrupted() {
            break;
        }
        if lost.swap(false, Ordering::AcqRel) {
            recovery = true;
            debounce.event(clock.elapsed());
        }
        // The timeout observes the signal flag only, never polls filesystem inputs.
        let wait = debounce
            .remaining(clock.elapsed())
            .unwrap_or(Duration::from_millis(25))
            .min(Duration::from_millis(25));
        match rx.recv_timeout(wait) {
            Ok(Ok(event)) => {
                if subscriptions
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .relevant(&event)
                {
                    recovery |= event.need_rescan()
                        || !matches!(
                            event.kind,
                            notify::EventKind::Modify(notify::event::ModifyKind::Data(_))
                        );
                    debounce.event(clock.elapsed());
                }
            }
            Ok(Err(err)) => {
                failure = Some(format!(
                    "receiving native notifications for {:?}: {err}",
                    err.paths
                ));
                break;
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                failure = Some("native notification channel disconnected".to_string());
                break;
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
        }
        // Drain existing events before scheduling to coalesce a startup/scan burst.
        for _ in 0..128 {
            let Ok(event) = rx.try_recv() else { break };
            match event {
                Ok(event)
                    if subscriptions
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .relevant(&event) =>
                {
                    recovery |= event.need_rescan()
                        || !matches!(
                            event.kind,
                            notify::EventKind::Modify(notify::event::ModifyKind::Data(_))
                        );
                    debounce.event(clock.elapsed());
                }
                Err(err) => {
                    failure = Some(format!("receiving native notifications: {err}"));
                    break;
                }
                _ => {}
            }
        }
        if failure.is_some() {
            break;
        }
        run = debounce
            .remaining(clock.elapsed())
            .is_some_and(|wait| wait.is_zero());
    }
    drop(observer);
    // §FS-check.6.3.3: a backend that keeps its subscriptions is fatal, not a hang.
    let released = subscriptions
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .watcher
        .close();
    drop(subscriptions);
    failure = failure.or(released.err());
    if let Some(err) = runtime_failure
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take()
    {
        failure = Some(err);
    }
    if let Some(err) = failure {
        return watch_fatal(&mut screen, err);
    }
    if let Err(err) = screen.restore() {
        return watch_fatal(&mut screen, format!("restoring terminal: {err}"));
    }
    last.unwrap_or_else(|| {
        let _ = watch_error_line("error: interrupted before the first check completed");
        2
    })
}

fn watch_fatal(screen: &mut WatchScreen, err: String) -> u8 {
    let _ = screen.restore();
    let _ = watch_error_line(&format!("error: watch: {err}"));
    2
}

fn watch_error_line(line: &str) -> std::io::Result<()> {
    use std::io::Write;
    let bytes = format!("{line}\n");
    let mut err = std::io::stderr().lock();
    err.write_all(bytes.as_bytes())?;
    err.flush()?;
    #[cfg(feature = "test-watch")]
    watch_capture_write(1, bytes.as_bytes());
    Ok(())
}
