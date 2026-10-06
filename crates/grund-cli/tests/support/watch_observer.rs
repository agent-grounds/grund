//! Bounded acknowledgements/barriers for the real coordinator (§AR-bindings.3).

use grund::{
    WatchObservation,
    watch_test::{self, Control, Snapshot},
};
use grund_core::{CheckFindingSelection, CheckOpts};
use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex, mpsc},
    thread,
    time::Duration,
};

pub const CONFIG: &str = "grund_config_version = 1\n[id]\nformat = \"{kind}-{slug}\"\n";
pub const CLEAN: &str = "// \u{a7}FS-live\n";
pub const BAD: &str = "// \u{a7}FS-live\n// \u{a7}FS-missing\n";
const LIMIT: Duration = Duration::from_secs(10);
static SERIAL: Mutex<()> = Mutex::new(());
pub fn serial() -> std::sync::MutexGuard<'static, ()> {
    SERIAL
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub struct Fixture(pub PathBuf);
impl Fixture {
    pub fn new() -> Self {
        let home = std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .expect("scratch home");
        let mut f = Self(PathBuf::from(home).join("ag/tmp").join(format!(
            "grund-watch-observer-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        )));
        fs::create_dir_all(&f.0).unwrap();
        f.0 = fs::canonicalize(&f.0).unwrap();
        f.write("grund.toml", CONFIG);
        f.write("docs/functional-spec/FS-live.md", "# FS-live: Live\n");
        f.write(
            "docs/functional-spec/README.md",
            "# FS index\n\n- [\u{a7}FS-live](FS-live.md#fs-live-live)\n",
        );
        f.write("src/main.rs", CLEAN);
        f
    }
    pub fn write(&self, path: &str, body: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }
    pub fn opts(&self) -> CheckOpts {
        CheckOpts {
            path: self.0.clone(),
            ..Default::default()
        }
    }
    pub fn ordinary(&self) -> Snapshot {
        watch_test::one_shot(self.opts(), &CheckFindingSelection::default(), None)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct Gate {
    key: String,
    resume: mpsc::Receiver<()>,
}
pub struct Harness {
    pub control: Arc<Control>,
    events: mpsc::Receiver<WatchObservation>,
    gates: Arc<Mutex<Vec<Gate>>>,
    resumes: Vec<mpsc::Sender<()>>,
    done: mpsc::Receiver<u8>,
    worker: Option<thread::JoinHandle<()>>,
    pub history: Vec<WatchObservation>,
}

pub fn key(event: &WatchObservation) -> String {
    match event {
        WatchObservation::Scanning => "scanning".into(),
        WatchObservation::Scanned => "scanned".into(),
        WatchObservation::Publishing => "publishing".into(),
        WatchObservation::StdoutFlushed => "stdout-flushed".into(),
        WatchObservation::StderrFlushed => "stderr-flushed".into(),
        WatchObservation::Input(input) => format!("input:{}", input.path.display()),
        _ => String::new(),
    }
}

impl Harness {
    pub fn start(f: &Fixture, format: Option<&str>, pause: Option<&str>) -> Self {
        Self::options(
            f.opts(),
            CheckFindingSelection::default(),
            format,
            pause,
            None,
        )
    }
    pub fn options(
        opts: CheckOpts,
        selection: CheckFindingSelection,
        format: Option<&str>,
        pause: Option<&str>,
        fail: Option<&str>,
    ) -> Self {
        Self::construct(opts, selection, format, pause, fail, None)
    }
    pub fn screen(f: &Fixture, eligible: bool) -> Self {
        Self::construct(
            f.opts(),
            CheckFindingSelection::default(),
            None,
            None,
            None,
            Some(eligible),
        )
    }
    fn construct(
        opts: CheckOpts,
        selection: CheckFindingSelection,
        format: Option<&str>,
        pause: Option<&str>,
        fail: Option<&str>,
        terminal: Option<bool>,
    ) -> Self {
        let (tx, events) = mpsc::channel();
        let gates: Arc<Mutex<Vec<Gate>>> = Arc::new(Mutex::new(Vec::new()));
        let gate_observer = gates.clone();
        let control = Control::new(move |event| {
            let key = key(&event);
            let gate = {
                let mut gates = gate_observer.lock().unwrap();
                gates
                    .iter()
                    .position(|gate| gate.key == key)
                    .map(|index| gates.remove(index))
            };
            tx.send(event).unwrap();
            if let Some(gate) = gate {
                gate.resume
                    .recv_timeout(LIMIT)
                    .expect("bounded barrier release");
            }
        });
        if let Some(fail) = fail {
            control.fail_next(fail);
        }
        if let Some(terminal) = terminal {
            control.terminal(terminal);
        }
        let (finished, done) = mpsc::channel();
        let mut harness = Self {
            control,
            events,
            gates,
            resumes: Vec::new(),
            done,
            worker: None,
            history: Vec::new(),
        };
        if let Some(pause) = pause {
            harness.pause(pause);
        }
        let control = harness.control.clone();
        let format = format.map(str::to_string);
        harness.worker = Some(thread::spawn(move || {
            finished
                .send(watch_test::run(opts, selection, format, control))
                .unwrap();
        }));
        harness
    }
    pub fn pause(&mut self, key: &str) -> mpsc::Sender<()> {
        let (tx, resume) = mpsc::channel();
        self.gates.lock().unwrap().push(Gate {
            key: key.to_string(),
            resume,
        });
        self.resumes.push(tx.clone());
        tx
    }
    pub fn release(&self) {
        for resume in &self.resumes {
            let _ = resume.send(());
        }
    }
    pub fn event(&mut self) -> WatchObservation {
        let event = self
            .events
            .recv_timeout(LIMIT)
            .expect("bounded watch acknowledgement");
        self.history.push(event.clone());
        event
    }
    pub fn stage(&mut self, wanted: &str) {
        loop {
            if key(&self.event()) == wanted {
                return;
            }
        }
    }
    pub fn completed(&mut self) -> Snapshot {
        loop {
            if let WatchObservation::Completed(status, stdout, stderr) = self.event() {
                return Snapshot {
                    status,
                    stdout,
                    stderr,
                    format: String::new(),
                };
            }
        }
    }
    pub fn matches(&mut self, expected: &Snapshot) {
        loop {
            let actual = self.completed();
            if actual.stdout == expected.stdout
                && actual.stderr == expected.stderr
                && actual.status == expected.status
            {
                return;
            }
        }
    }
    pub fn stop(&mut self, status: u8) {
        self.control.interrupt();
        self.release();
        self.finish(status);
    }
    pub fn finish(&mut self, status: u8) {
        assert_eq!(
            self.done
                .recv_timeout(LIMIT)
                .expect("bounded watch shutdown"),
            status
        );
        self.worker.take().unwrap().join().unwrap();
        while let Ok(event) = self.events.try_recv() {
            self.history.push(event);
        }
        assert!(matches!(
            self.history.last(),
            Some(WatchObservation::Stopped)
        ));
    }
}
impl Drop for Harness {
    fn drop(&mut self) {
        if self.worker.is_some() {
            self.control.interrupt();
            self.release();
            if self.done.recv_timeout(LIMIT).is_ok() {
                let _ = self.worker.take().unwrap().join();
            }
        }
    }
}

pub fn changed_event(path: PathBuf) -> notify::Event {
    notify::Event::new(notify::EventKind::Modify(notify::event::ModifyKind::Data(
        notify::event::DataChange::Any,
    )))
    .add_path(path)
}

/// Bound isolated children and drain both pipes concurrently (§AR-bindings.3).
pub fn bounded_child(command: &mut std::process::Command) -> std::process::Output {
    use std::io::Read;
    use std::process::Stdio;
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    fn reader(mut input: impl Read + Send + 'static) -> thread::JoinHandle<Vec<u8>> {
        thread::spawn(move || {
            let mut bytes = Vec::new();
            input.read_to_end(&mut bytes).unwrap();
            bytes
        })
    }
    let stdout = reader(child.stdout.take().unwrap());
    let stderr = reader(child.stderr.take().unwrap());
    let deadline = std::time::Instant::now() + LIMIT;
    let mut timed_out = false;
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if std::time::Instant::now() >= deadline {
            timed_out = true;
            let _ = child.kill();
            break child.wait().unwrap();
        }
        thread::sleep(Duration::from_millis(10));
    };
    let output = std::process::Output {
        status,
        stdout: stdout.join().unwrap(),
        stderr: stderr.join().unwrap(),
    };
    assert!(
        !timed_out,
        "isolated watch test exceeded its deadline: {output:?}"
    );
    output
}
