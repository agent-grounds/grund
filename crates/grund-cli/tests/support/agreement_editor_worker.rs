//! Real snapshot/hover entrypoints in a process with the fixture's cwd
//! (§FS-lsp.2.2.1, §REQ-one-answer.proof.matrix). The parent never changes cwd.

use grund_core::{
    LspSnapshotOpts, ShowFormat, ShowMode, lsp_snapshot_with_completion, show_from_snapshot,
};
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

const PREFIX: &str = "GRUND-EDITOR ";

pub struct Worker {
    child: Child,
    input: Option<ChildStdin>,
    output: BufReader<ChildStdout>,
    pub last_seconds: f64,
}

impl Worker {
    pub fn new() -> Self {
        let start = std::time::Instant::now();
        let mut child = Command::new(std::env::current_exe().expect("test executable"))
            .args(["--exact", "agreement_editor_worker::serve", "--nocapture"])
            .env("GRUND_AGREEMENT_EDITOR_WORKER", "1")
            .current_dir(super::agreement_host::repository())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("isolated editor entrypoint");
        let input = child.stdin.take();
        let output = BufReader::new(child.stdout.take().unwrap());
        let mut worker = Self {
            child,
            input,
            output,
            last_seconds: 0.0,
        };
        assert_eq!(worker.read(), json!({"ready":true}));
        super::agreement_evidence::receipt(
            "editor-provenance",
            &super::agreement_host::repository(),
            json!({}),
            json!({"startupSeconds":start.elapsed().as_secs_f64()}),
        );
        worker
    }

    fn read(&mut self) -> Value {
        loop {
            let mut line = String::new();
            assert!(
                self.output.read_line(&mut line).expect("editor response") > 0,
                "editor worker terminated: {:?}",
                self.child.try_wait()
            );
            if let Some((_, response)) = line.split_once(PREFIX) {
                return serde_json::from_str(response).expect("structured editor response");
            }
        }
    }

    pub fn call(&mut self, request: Value) -> Value {
        let start = std::time::Instant::now();
        let input = self.input.as_mut().unwrap();
        serde_json::to_writer(&mut *input, &request).unwrap();
        writeln!(input).unwrap();
        input.flush().unwrap();
        let result = self.read();
        self.last_seconds = start.elapsed().as_secs_f64();
        result
    }

    pub fn release(&mut self) {
        assert_eq!(self.call(json!({"operation":"release"})), json!(null));
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        drop(self.input.take());
        if std::thread::panicking() {
            let _ = self.child.kill();
        }
        assert!(
            self.child.wait().expect("editor worker exit").success() || std::thread::panicking()
        );
    }
}

/// Selected only by Worker; normal test execution is a no-op. Isolation is a
/// process property, so its cwd cannot race another test's fixture or engine.
#[test]
fn serve() {
    if std::env::var_os("GRUND_AGREEMENT_EDITOR_WORKER").is_none() {
        return;
    }
    let send = |value: Value| {
        println!("{PREFIX}{value}");
        std::io::stdout().flush().unwrap();
    };
    send(json!({"ready":true}));
    let mut loaded = None;
    for line in std::io::stdin().lock().lines() {
        let request: Value = serde_json::from_str(&line.unwrap()).unwrap();
        if request["operation"] == "release" {
            loaded = None;
            std::env::set_current_dir(super::agreement_host::repository()).unwrap();
            send(Value::Null);
            continue;
        }
        std::env::set_current_dir(request["cwd"].as_str().unwrap()).unwrap();
        if request["operation"] == "show" {
            let result = loaded
                .as_ref()
                .map(|s: &grund_core::LspSnapshotWithCompletion| {
                    show_from_snapshot(
                        &s.show,
                        request["coordinate"].as_str().unwrap(),
                        ShowMode::Toc,
                        ShowFormat::Json,
                    )
                });
            send(match result {
                Some(Ok(out)) => json!({"body":out.body,"path":out.path,"line":out.line,
                    "json":out.json,"sections":out.sections.iter().map(|s|json!({"path":s.path,
                        "title":s.title,"depth":s.depth})).collect::<Vec<_>>()}),
                Some(Err(error)) => json!({"failure":format!("{error:#}")}),
                None => panic!("show without a completed snapshot"),
            });
            continue;
        }
        let opts = LspSnapshotOpts {
            path: PathBuf::from(request["path"].as_str().unwrap()),
            path_provided: request["explicit"].as_bool().unwrap(),
            open_documents: request["overlays"]
                .as_object()
                .unwrap()
                .iter()
                .map(|(path, text)| (PathBuf::from(path), text.as_str().unwrap().to_string()))
                .collect(),
        };
        let result = lsp_snapshot_with_completion(opts);
        send(match &result {
            Ok(s) => super::agreement_snapshot::facts(s),
            Err(error) => json!({"failure":format!("{error:#}")}),
        });
        if request["operation"] == "snapshot" {
            loaded = result.ok();
        }
    }
}
