//! Native-only deterministic runtime seam (§FS-distribution.3.2.3.1).
use napi::bindgen_prelude::AsyncTask;
use napi::{Env, Result, Task};
use napi_derive::napi;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Condvar, Mutex};
use std::time::Duration;

#[derive(Default)]
struct State {
    hold: Option<(String, String)>,
    started: bool,
    released: bool,
    panic: Option<String>,
    worker: bool,
    delay: Option<(String, u32)>,
    writes: Option<(String, usize)>,
}
static STATE: Mutex<State> = Mutex::new(State {
    hold: None,
    started: false,
    released: false,
    panic: None,
    worker: false,
    delay: None,
    writes: None,
});
static SIGNAL: Condvar = Condvar::new();
static HOOK_CALLS: AtomicUsize = AtomicUsize::new(0);

#[napi(namespace = "__test", js_name = "holdNext")]
pub fn hold_next(operation: String) {
    arm(operation, "success".into());
}
#[napi(namespace = "__test", js_name = "holdNextWriter")]
pub fn hold_next_writer(outcome: String) {
    arm("*writer".into(), outcome);
}
fn arm(operation: String, outcome: String) {
    let mut s = STATE.lock().unwrap_or_else(|e| e.into_inner());
    s.hold = Some((operation, outcome));
    s.started = false;
    s.released = false;
}
pub fn started(operation: &str, writer: bool) -> std::result::Result<(), String> {
    let mut s = STATE.lock().unwrap_or_else(|e| e.into_inner());
    let outcome = if s
        .hold
        .as_ref()
        .is_some_and(|(op, _)| op == operation || (op == "*writer" && writer))
    {
        let (_, outcome) = s.hold.take().unwrap();
        s.started = true;
        SIGNAL.notify_all();
        while !s.released {
            s = SIGNAL.wait(s).unwrap_or_else(|e| e.into_inner());
        }
        outcome
    } else {
        "success".into()
    };
    let delay = if s.delay.as_ref().is_some_and(|(op, _)| op == operation) {
        s.delay.take().map(|(_, ms)| ms)
    } else {
        None
    };
    drop(s);
    if let Some(ms) = delay {
        std::thread::sleep(Duration::from_millis(ms.into()));
    }
    match outcome.as_str() {
        "error" => Err("test native writer failure".into()),
        "panic" => panic!("test native writer panic"),
        _ => Ok(()),
    }
}
#[napi(namespace = "__test")]
pub fn release() {
    STATE.lock().unwrap_or_else(|e| e.into_inner()).released = true;
    SIGNAL.notify_all();
}
#[napi(namespace = "__test", js_name = "panicNext")]
pub fn panic_next(stage: String) {
    STATE.lock().unwrap_or_else(|e| e.into_inner()).panic = Some(stage);
}
pub fn fault(stage: &str) {
    let mut s = STATE.lock().unwrap_or_else(|e| e.into_inner());
    let panic = s.panic.as_deref() == Some(stage);
    if panic {
        s.panic.take();
    }
    drop(s);
    if panic {
        panic!("test native {stage} panic");
    }
}
pub fn reject_fault() -> bool {
    STATE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .panic
        .as_deref()
        == Some("reject")
}
#[napi(namespace = "__test", js_name = "failWorkerNext")]
pub fn fail_worker_next() {
    STATE.lock().unwrap_or_else(|e| e.into_inner()).worker = true;
}
pub fn worker_failure() -> bool {
    let mut s = STATE.lock().unwrap_or_else(|e| e.into_inner());
    std::mem::take(&mut s.worker)
}
#[napi(namespace = "__test", js_name = "delayNext")]
pub fn delay_next(operation: String, milliseconds: u32) {
    STATE.lock().unwrap_or_else(|e| e.into_inner()).delay = Some((operation, milliseconds));
}
#[napi(namespace = "__test", js_name = "installOtherHook")]
pub fn install_other_hook() {
    std::panic::set_hook(Box::new(|_| {
        HOOK_CALLS.fetch_add(1, Ordering::SeqCst);
    }));
}
#[napi(namespace = "__test", js_name = "hookCalls")]
pub fn hook_calls() -> u32 {
    HOOK_CALLS.load(Ordering::SeqCst) as u32
}
#[napi(namespace = "__test", js_name = "panicOutsideGuard")]
pub fn panic_outside_guard() {
    let _ = std::panic::catch_unwind(|| panic!("independent native caller"));
}
#[napi(namespace = "__test", js_name = "failWriteAfter")]
pub fn fail_write_after(operation: String, count: u32) {
    STATE.lock().unwrap_or_else(|e| e.into_inner()).writes = Some((operation, count as usize));
}
pub fn write_guard(operation: &str) -> grund_core::BindingWriteFaultGuard {
    let mut s = STATE.lock().unwrap_or_else(|e| e.into_inner());
    let count = if s.writes.as_ref().is_some_and(|(op, _)| op == operation) {
        s.writes.take().map(|(_, count)| count)
    } else {
        None
    };
    grund_core::binding_write_fault(count)
}
#[napi(object)]
pub struct Counts {
    pub jobs: u32,
    pub environments: u32,
    pub writers: u32,
}
#[napi(namespace = "__test", js_name = "resourceCounts")]
pub fn resource_counts() -> Counts {
    Counts {
        jobs: super::runtime::JOBS.load(Ordering::SeqCst) as u32,
        environments: super::runtime::ENVIRONMENTS.load(Ordering::SeqCst) as u32,
        writers: u32::from(super::runtime::WRITER.load(Ordering::SeqCst)),
    }
}
pub struct Wait(bool);
impl Task for Wait {
    type Output = ();
    type JsValue = ();
    fn compute(&mut self) -> Result<()> {
        if self.0 {
            let mut s = STATE.lock().unwrap_or_else(|e| e.into_inner());
            while !s.started {
                s = SIGNAL.wait(s).unwrap_or_else(|e| e.into_inner());
            }
        } else {
            while super::runtime::JOBS.load(Ordering::SeqCst) > 0 {
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        Ok(())
    }
    fn resolve(&mut self, _env: Env, _out: ()) -> Result<()> {
        Ok(())
    }
}
#[napi(namespace = "__test", js_name = "waitStarted")]
pub fn wait_started() -> AsyncTask<Wait> {
    AsyncTask::new(Wait(true))
}
#[napi(namespace = "__test", js_name = "waitIdle")]
pub fn wait_idle() -> AsyncTask<Wait> {
    AsyncTask::new(Wait(false))
}
