//! Bounded CLI subprocess scaffolding for §FS-check.6.2.1 and §FS-check.6.3.3.

use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Child, Command, Output, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const DEADLINE: Duration = Duration::from_secs(10);
static NEXT: AtomicUsize = AtomicUsize::new(0);
pub const CLEAN: &str = "// \u{a7}FS-live\n";
pub const BAD: &str = "// \u{a7}FS-live\n// \u{a7}FS-missing\n";
pub const CONFIG: &str = "grund_config_version = 1\n\n[id]\nformat = \"{kind}-{slug}\"\n";

pub struct Fixture(pub PathBuf);

impl Fixture {
    pub fn new() -> Self {
        let root = PathBuf::from(std::env::var_os("HOME").expect("HOME for scratch fixtures"))
            .join("ag/tmp")
            .join(format!(
                "grund-473-watch-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir_all(root.join("docs/functional-spec")).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        let fixture = Self(root);
        fixture.write("grund.toml", CONFIG);
        fixture.write("docs/functional-spec/FS-live.md", "# FS-live: Live\n");
        fixture.write(
            "docs/functional-spec/README.md",
            "# FS index\n\n- [\u{a7}FS-live](FS-live.md#fs-live-live)\n",
        );
        fixture.write("src/main.rs", CLEAN);
        fixture
    }

    pub fn write(&self, path: &str, body: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }

    pub fn check(&self, args: &[&str]) -> Output {
        let child = Command::new(env!("CARGO_BIN_EXE_grund"))
            .arg("check")
            .args(args)
            .current_dir(&self.0)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut process = Watch::capture(child);
        let status = process.wait_exit();
        Output {
            status,
            stdout: std::mem::take(&mut process.bytes[0]),
            stderr: std::mem::take(&mut process.bytes[1]),
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub struct Watch {
    child: Child,
    rx: Receiver<(usize, Vec<u8>)>,
    readers: Vec<JoinHandle<()>>,
    bytes: [Vec<u8>; 2],
    previous: [Vec<u8>; 2],
}

impl Watch {
    fn capture(mut child: Child) -> Self {
        let (tx, rx) = mpsc::channel();
        let streams: [Box<dyn Read + Send>; 2] = [
            Box::new(child.stdout.take().unwrap()),
            Box::new(child.stderr.take().unwrap()),
        ];
        let readers = streams
            .into_iter()
            .enumerate()
            .map(|(stream, mut pipe)| {
                let tx = tx.clone();
                thread::spawn(move || {
                    let mut buf = [0; 4096];
                    while let Ok(n) = pipe.read(&mut buf) {
                        if n == 0 || tx.send((stream, buf[..n].to_vec())).is_err() {
                            break;
                        }
                    }
                })
            })
            .collect();
        Self {
            child,
            rx,
            readers,
            bytes: Default::default(),
            previous: Default::default(),
        }
    }

    pub fn start(fixture: &Fixture, args: &[&str]) -> Self {
        Self::capture(
            Command::new(env!("CARGO_BIN_EXE_grund"))
                .args(["check", "--watch"])
                .args(args)
                .current_dir(&fixture.0)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap(),
        )
    }

    fn receive(&mut self) {
        if let Ok((stream, bytes)) = self.rx.recv_timeout(Duration::from_millis(10)) {
            self.bytes[stream].extend(bytes);
        }
    }

    fn join_readers(&mut self) {
        for reader in self.readers.drain(..) {
            reader.join().unwrap();
        }
        while let Ok((stream, bytes)) = self.rx.try_recv() {
            self.bytes[stream].extend(bytes);
        }
    }

    pub fn report(&mut self, expected: &Output) {
        let wanted = [&expected.stdout, &expected.stderr];
        assert!(
            wanted.iter().any(|bytes| !bytes.is_empty()),
            "empty reports need the private completion observer"
        );
        let deadline = Instant::now() + DEADLINE;
        loop {
            self.receive();
            if let Some(status) = self.child.try_wait().unwrap() {
                self.join_readers();
                panic!(
                    "watch exited before the requested report: status={:?}, stdout={:?}, stderr={:?}; expected stdout={:?}, stderr={:?}",
                    status.code(),
                    String::from_utf8_lossy(&self.bytes[0]),
                    String::from_utf8_lossy(&self.bytes[1]),
                    String::from_utf8_lossy(wanted[0]),
                    String::from_utf8_lossy(wanted[1])
                );
            }
            // §FS-check.6.1.2: do not assume an exact count of native events/runs.
            // Consume repeated previous snapshots before matching the new one.
            for (stream, target) in wanted.iter().enumerate() {
                let old = &self.previous[stream];
                while !old.is_empty() && self.bytes[stream].starts_with(old) && old != *target {
                    self.bytes[stream].drain(..old.len());
                }
            }
            if wanted
                .iter()
                .enumerate()
                .all(|(s, w)| self.bytes[s].starts_with(w))
            {
                for (stream, target) in wanted.iter().enumerate() {
                    self.bytes[stream].drain(..target.len());
                    self.previous[stream] = (*target).clone();
                }
                return;
            }
            assert!(
                Instant::now() < deadline,
                "watch report deadline: expected={expected:?}, captured={:?}",
                self.bytes
            );
        }
    }

    fn wait_exit(&mut self) -> std::process::ExitStatus {
        let deadline = Instant::now() + DEADLINE;
        loop {
            self.receive();
            if let Some(status) = self.child.try_wait().unwrap() {
                self.join_readers();
                return status;
            }
            assert!(Instant::now() < deadline, "process exit deadline");
        }
    }

    pub fn interrupt(mut self, expected: i32) {
        assert!(
            Command::new("kill")
                .args(["-INT", &self.child.id().to_string()])
                .status()
                .unwrap()
                .success()
        );
        let status = self.wait_exit();
        assert_eq!(
            status.code(),
            Some(expected),
            "remaining output: {:?}",
            self.bytes
        );
        for stream in 0..2 {
            let old = &self.previous[stream];
            while !old.is_empty() && self.bytes[stream].starts_with(old) {
                self.bytes[stream].drain(..old.len());
            }
            assert!(
                self.bytes[stream].is_empty(),
                "unexpected bytes on stream {stream}: {:?}",
                self.bytes[stream]
            );
        }
    }
}

impl Drop for Watch {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        self.join_readers();
    }
}
