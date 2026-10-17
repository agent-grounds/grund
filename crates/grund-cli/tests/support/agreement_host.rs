//! Persistent actual same-source hosts (§REQ-one-answer.proof.matrix), with
//! completed request/result receipts (§REQ-one-answer.proof.evidence).

use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

pub struct Host {
    child: Child,
    input: Option<ChildStdin>,
    output: BufReader<ChildStdout>,
    pub provenance: Value,
}

pub fn repository() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository")
}

/// The repository spelled as a plain path: `canonicalize` gives Windows' verbatim
/// `\\?\D:\...`, which Node's module loader refuses as a script path, so the host
/// that stages and loads the Node package is started at the drive-letter spelling
/// (§REQ-one-answer.proof.matrix).
pub fn plain(path: PathBuf) -> PathBuf {
    match path.to_str().and_then(|p| p.strip_prefix(r"\\?\")) {
        Some(rest) if !rest.starts_with(r"UNC\") => PathBuf::from(rest),
        _ => path,
    }
}

impl Host {
    pub fn new() -> Self {
        let start = std::time::Instant::now();
        let root = plain(repository());
        let mut child = Command::new(std::env::var_os("PYTHON").unwrap_or_else(|| "python".into()))
            .arg(root.join("tests/bindings/agreement_host.py"))
            .current_dir(&root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("start same-source hosts");
        let input = child.stdin.take();
        let output = BufReader::new(child.stdout.take().expect("host stdout"));
        let mut host = Self {
            child,
            input,
            output,
            provenance: Value::Null,
        };
        host.provenance = host.read()["ready"].clone();
        // §REQ-one-answer.proof.delivery: preparation includes copies/imports
        // and process startup, beyond setupCommands' individual build timers.
        host.provenance["hostStartupSeconds"] = json!(start.elapsed().as_secs_f64());
        let sha = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(root)
            .output()
            .expect("source SHA");
        assert_eq!(
            host.provenance["sourceSha"].as_str(),
            Some(String::from_utf8_lossy(&sha.stdout).trim())
        );
        host
    }

    fn read(&mut self) -> Value {
        let mut line = String::new();
        assert!(
            self.output.read_line(&mut line).expect("host response") > 0,
            "host terminated without a response: {:?}",
            self.child.try_wait()
        );
        serde_json::from_str(&line).unwrap_or_else(|error| panic!("host response {error}: {line}"))
    }

    pub fn request(&mut self, request: Value) -> Value {
        let input = self.input.as_mut().expect("live host");
        serde_json::to_writer(&mut *input, &request).expect("encode host request");
        writeln!(input).expect("request delimiter");
        input.flush().expect("send request");
        self.read()
    }

    pub fn call(&mut self, surface: &str, request: Value) -> Value {
        let start = std::time::Instant::now();
        let response = self.request(json!({"op":surface,"request":request}));
        assert_eq!(
            response["request"], request,
            "host response belongs to another call"
        );
        let cwd = PathBuf::from(request["cwd"].as_str().unwrap());
        super::agreement_evidence::receipt(surface, &cwd, request, response["envelope"].clone());
        // §REQ-one-answer.proof.delivery: include IPC and first Node startup;
        // the remote operation timer excludes these costs.
        super::agreement_evidence::seconds(start.elapsed().as_secs_f64());
        response["envelope"].clone()
    }
}

impl Drop for Host {
    fn drop(&mut self) {
        drop(self.input.take());
        if std::thread::panicking() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        } else {
            assert!(
                self.child.wait().expect("host exit").success(),
                "host failed on shutdown"
            );
        }
    }
}
