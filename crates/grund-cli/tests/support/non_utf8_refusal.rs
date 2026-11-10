//! Linux filesystem fault injection for §AR-ci.10.6; never linked into grund.

#[path = "fixture_cleanup.rs"]
mod fixture_cleanup;
use fixture_cleanup::remove_fixture_tree;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Probe(PathBuf);

pub struct Result {
    pub success: bool,
    pub output: String,
    pub trace: String,
}

impl Probe {
    pub fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "grund-1281-refusal-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).expect("claim refusal probe root");
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/support/non_utf8_refusal.c");
        let built = Command::new("cc")
            .args(["-shared", "-fPIC", "-Wall", "-Wextra", "-Werror"])
            .arg(source)
            .args(["-ldl", "-o"])
            .arg(root.join("refusal.so"))
            .output()
            .expect("the Linux refusal regression requires a C compiler");
        assert!(built.status.success(), "compile refusal seam: {built:?}");
        Self(root)
    }

    pub fn run(&self, filter: &str, mode: &str) -> Result {
        let trace = self.0.join("trace");
        fs::write(&trace, "").unwrap();
        let child = Command::new(std::env::current_exe().unwrap())
            .args([filter, "--nocapture", "--test-threads=1"])
            .env("LD_PRELOAD", self.0.join("refusal.so"))
            .env("GRUND_TEST_NAME_REFUSAL", mode)
            .env("GRUND_TEST_NAME_TRACE", &trace)
            .env("TMPDIR", &self.0)
            .output()
            .expect("run existing cases with controlled filename refusal");
        Result {
            success: child.status.success(),
            output: format!(
                "{}{}",
                String::from_utf8_lossy(&child.stdout),
                String::from_utf8_lossy(&child.stderr)
            ),
            trace: fs::read_to_string(trace).unwrap(),
        }
    }
}

impl Result {
    pub fn assert_available(&self, mode: &str) {
        assert!(
            self.trace.contains(&format!("refused:{mode}")),
            "seam did not refuse a name: {}",
            self.trace
        );
        assert!(
            self.success,
            "fixtures must survive filename refusal:\n{}",
            self.output
        );
        let explanation = self.output.to_lowercase();
        assert!(
            explanation.contains("filesystem") && explanation.contains("utf-8"),
            "report the unavailable disk-name coverage: {}",
            self.output
        );
    }

    pub fn assert_invoked(&self, argv: &[u8]) {
        let encoded: String = argv.iter().map(|byte| format!("{byte:02x}")).collect();
        assert!(
            self.trace
                .lines()
                .any(|line| line.starts_with("exec:") && line.contains(&encoded)),
            "independent invocation {argv:?} was skipped:\n{}",
            self.trace
        );
    }
}

impl Drop for Probe {
    fn drop(&mut self) {
        remove_fixture_tree(&self.0, "strict refusal probe cleanup");
    }
}
