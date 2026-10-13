//! Test-only observation of completed agreement entrypoints (§REQ-one-answer.proof.evidence).
//! This observes the seed; it does not discover or execute additional questions.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::process::Output;

#[derive(Debug, serde::Serialize)]
pub struct Call {
    pub path: PathBuf,
    pub surface: String,
    pub args: Vec<String>,
    pub overlay: bool,
    pub exit: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

thread_local! {
    static CALLS: RefCell<Option<Vec<Call>>> = const { RefCell::new(None) };
}

fn record(call: Call) {
    CALLS.with(|calls| {
        if let Some(calls) = &mut *calls.borrow_mut() {
            calls.push(call);
        }
    });
}

/// Record the request and actual subprocess response only after the call completes.
pub fn cli(path: &Path, args: &[&str], result: &Output) {
    let command = args.first().copied().unwrap_or("show");
    let has = |flag| args.contains(&flag);
    let surface = match command {
        "list" if has("--size") => "list-size",
        "cover" if has("--lines") => "cover-lines",
        "show" if has("--batch") => "show-batch",
        "check" | "list" | "refs" | "cover" | "fmt" | "show" | "complete" | "id" => command,
        _ => "show",
    };
    record(Call {
        path: path
            .canonicalize()
            .expect("CLI cwd exists at call boundary"),
        surface: surface.to_string(),
        args: args.iter().map(|arg| arg.to_string()).collect(),
        overlay: false,
        exit: result.status.code(),
        stdout: result.stdout.clone(),
        stderr: result.stderr.clone(),
    });
}

/// Wrap the actual existing snapshot entrypoint, preserving its result unchanged.
pub fn editor<T>(
    opts: grund_core::LspSnapshotOpts,
    entrypoint: impl FnOnce(grund_core::LspSnapshotOpts) -> T,
) -> T {
    let path = opts
        .path
        .canonicalize()
        .expect("snapshot path exists at call boundary");
    let overlay = !opts.open_documents.is_empty();
    let result = entrypoint(opts);
    record(Call {
        path,
        surface: "editor".to_string(),
        args: Vec::new(),
        overlay,
        exit: None,
        stdout: Vec::new(),
        stderr: Vec::new(),
    });
    result
}

/// Isolated per libtest thread; disabled in the ordinary agreement tests.
pub fn capture(run: impl FnOnce()) -> Vec<Call> {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            CALLS.with(|calls| *calls.borrow_mut() = None);
        }
    }
    CALLS.with(|calls| {
        assert!(calls.borrow().is_none(), "nested agreement observer");
        *calls.borrow_mut() = Some(Vec::new());
    });
    let _reset = Reset;
    run();
    CALLS.with(|calls| calls.borrow_mut().take().expect("active observer"))
}
