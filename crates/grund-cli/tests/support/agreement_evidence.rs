//! Test-only observation of completed agreement entrypoints (§REQ-one-answer.proof.evidence).
//! Discovery and execution belong to the corpus driver; this records completed
//! boundaries and the independently checked expectation attached to each call.

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
    pub context: Option<serde_json::Value>,
    pub request: serde_json::Value,
    pub result: serde_json::Value,
    pub expected: serde_json::Value,
    pub seconds: Option<f64>,
}

thread_local! {
    static CALLS: RefCell<Option<Vec<Call>>> = const { RefCell::new(None) };
    static CONTEXT: RefCell<Option<serde_json::Value>> = const { RefCell::new(None) };
}

fn record(mut call: Call) {
    CONTEXT.with(|context| call.context = context.borrow().clone());
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
        context: None,
        request: serde_json::json!({"cwd":path,"args":args}),
        result: serde_json::Value::Null,
        expected: serde_json::Value::Null,
        seconds: None,
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
        context: None,
        request: serde_json::Value::Null,
        result: serde_json::Value::Null,
        expected: serde_json::Value::Null,
        seconds: None,
    });
    result
}

/// Scope copied input attribution to real call boundaries (§REQ-one-answer.proof.evidence).
pub fn context<T>(value: serde_json::Value, run: impl FnOnce() -> T) -> T {
    CONTEXT.with(|context| assert!(context.replace(Some(value)).is_none()));
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            CONTEXT.with(|context| *context.borrow_mut() = None);
        }
    }
    let _reset = Reset;
    run()
}

/// The independent witness is attached to the completed call it checks, never
/// manufactured by another surface (§REQ-one-answer.proof.evidence).
pub fn witness(expected: serde_json::Value) {
    CALLS.with(|calls| {
        if let Some(calls) = &mut *calls.borrow_mut() {
            calls.last_mut().expect("completed call").expected = expected;
        }
    });
}

/// Preserve the actual stdin bytes of an invocation (§FS-show.1.8).
pub fn stdin(input: Option<&str>) {
    CALLS.with(|calls| {
        if let Some(calls) = &mut *calls.borrow_mut() {
            calls.last_mut().expect("completed CLI").request["stdin"] = serde_json::json!(input);
        }
    });
}

/// Cost attribution includes entrypoint startup; it is not CI gate payment.
pub fn seconds(seconds: f64) {
    CALLS.with(|calls| {
        if let Some(calls) = &mut *calls.borrow_mut() {
            calls.last_mut().expect("completed entrypoint").seconds = Some(seconds);
        }
    });
}

/// A completed host/snapshot or verified materialization/applicability receipt.
/// `result` is the actual response; classification is independently checked by
/// acceptance from `request` (§REQ-one-answer.proof.matrix).
pub fn receipt(surface: &str, path: &Path, request: serde_json::Value, result: serde_json::Value) {
    record(Call {
        path: path.to_path_buf(),
        surface: surface.into(),
        args: Vec::new(),
        overlay: request
            .get("overlays")
            .is_some_and(|v| v.as_object().is_some_and(|o| !o.is_empty())),
        exit: None,
        stdout: Vec::new(),
        stderr: Vec::new(),
        context: None,
        request,
        result,
        expected: serde_json::Value::Null,
        seconds: None,
    });
}

/// Adopt the calls another thread captured, in the order given, into this
/// thread's observer (§REQ-one-answer.proof.evidence).
pub fn adopt(adopted: Vec<Call>) {
    CALLS.with(|calls| {
        if let Some(calls) = &mut *calls.borrow_mut() {
            calls.extend(adopted);
        }
    });
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
