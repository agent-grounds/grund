//! Thin, owned AsyncTask frontend (§AR-bindings.5, §FS-distribution.3.2.3.1).
mod ffi;
mod runtime;
#[cfg(feature = "test-node-runtime")]
mod test_runtime;

use grund_core::{EmbeddingRequest, node_embedding_call};
use napi::bindgen_prelude::AsyncTask;
#[cfg(not(test))]
use napi::bindgen_prelude::{FromNapiValue, Object};
use napi::{Env, Error, Result, Task};
#[cfg(not(test))]
use napi_derive::module_init;
use napi_derive::napi;
use runtime::{Context, Guard, WriterLease};
use serde_json::{Value, json};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, atomic::Ordering};

// §AR-bindings.5: Node owns module registration; Cargo's test executable has no host.
#[cfg(not(test))]
#[module_init]
fn register_metadata() {
    unsafe fn register(
        raw_env: napi::sys::napi_env,
        raw_exports: napi::sys::napi_value,
    ) -> Result<()> {
        caught(||{
            let env=Env::from_raw(raw_env);
            runtime::context(&env)?;
            // SAFETY: napi-rs supplies the current environment's exports object.
            let mut exports=unsafe {Object::from_napi_value(raw_env,raw_exports)}?;
            exports.set("metadata",json!({
                "apiSchemaVersion":1,"engineVersion":env!("CARGO_PKG_VERSION"),
                "packageVersion":env!("CARGO_PKG_VERSION"),"target":env!("GRUND_NODE_TARGET"),"napiVersion":8
            }))?;
            Ok(())
        }).unwrap_or_else(|_|Err(Error::from_reason("native module initialization panicked")))
    }
    napi::bindgen_prelude::register_module_exports(register);
}

pub(crate) fn failure(operation: &str, kind: &str, code: &str, message: &str) -> Value {
    json!({"result":null,"run_cautions":[],"failure":{"operation":operation,
        "kind":kind,"code":code,"message":message,"path":null,"line":null,"column":null,
        "sites":[],"authority":[],"causes":[],"details":{},"partial":null}})
}
pub(crate) fn caught<T>(f: impl FnOnce() -> T) -> std::result::Result<T, ()> {
    let _guard = Guard::enter();
    catch_unwind(AssertUnwindSafe(f)).map_err(|payload| {
        // Drop ordinary payloads; contain a payload destructor that itself panics.
        if let Err(secondary) = catch_unwind(AssertUnwindSafe(|| drop(payload))) {
            std::mem::forget(secondary);
        }
    })
}

/// Native private request transport (§FS-distribution.3.2.3.2).
#[napi]
pub fn invoke(env: Env, request: ffi::Input) -> Result<ffi::Dispatch> {
    runtime::install_hook();
    let value: Value =
        serde_json::from_str(&request.0).map_err(|e| Error::from_reason(e.to_string()))?;
    let operation = value["operation"].as_str().unwrap_or("").to_owned();
    let context = runtime::context(&env)?;
    let mut initial = None;
    let parsed = caught(|| {
        fault("convert");
        EmbeddingRequest {
            operation: operation.clone(),
            root: value["root"].as_str().unwrap_or(".").into(),
            explicit: value["explicit"].as_bool().unwrap_or(false),
            args: value["args"].as_array().cloned().unwrap_or_default(),
            options: value["options"].clone(),
        }
    });
    let request = match parsed {
        Ok(r) => Some(r),
        Err(()) => {
            initial = Some(failure(
                &operation,
                "native",
                "native-panic",
                "native conversion panicked",
            ));
            None
        }
    };
    let writer = request.as_ref().is_some_and(|r| match operation.as_str() {
        "fmt" | "integrations" => r.options["write"] == true,
        "init" => r.options["dry_run"] != true && r.options["check"] != true,
        "fetch" => true,
        _ => false,
    });
    let lease = if writer {
        match WriterLease::acquire() {
            Some(lease) => Some(lease),
            None => {
                initial = Some(failure(
                    &operation,
                    "busy",
                    "writer-busy",
                    "another writer is active",
                ));
                None
            }
        }
    } else {
        None
    };
    if !context.accepting.load(Ordering::SeqCst) {
        initial = Some(failure(
            &operation,
            "worker",
            "worker-failed",
            "environment is closing",
        ));
    }
    runtime::JOBS.fetch_add(1, Ordering::SeqCst);
    let name = operation.clone();
    Ok(ffi::Dispatch(
        AsyncTask::new(Job {
            operation,
            request,
            initial,
            context,
            lease,
        }),
        name,
    ))
}

pub struct Job {
    operation: String,
    request: Option<EmbeddingRequest>,
    initial: Option<Value>,
    context: Arc<Context>,
    lease: Option<WriterLease>,
}
impl Drop for Job {
    fn drop(&mut self) {
        self.lease.take();
        runtime::JOBS.fetch_sub(1, Ordering::SeqCst);
    }
}
impl Task for Job {
    type Output = String;
    type JsValue = ffi::Wire;
    fn compute(&mut self) -> Result<String> {
        #[cfg(feature = "test-node-runtime")]
        if test_runtime::worker_failure() || test_runtime::reject_fault() {
            return Err(Error::new(
                napi::Status::GenericFailure,
                "native worker completion failed",
            ));
        }
        let result: std::result::Result<std::result::Result<Value, String>, ()> = caught(|| {
            if let Some(initial) = self.initial.take() {
                return Ok(initial);
            }
            #[cfg(feature = "test-node-runtime")]
            test_runtime::started(&self.operation, self.lease.is_some())?;
            fault("compute");
            self.context
                .pool
                .install(|| {
                    fault("rayon");
                    #[cfg(feature = "test-node-runtime")]
                    let _write_guard = test_runtime::write_guard(&self.operation);
                    let value = node_embedding_call(self.request.take().expect("owned request"));
                    grund_core::ApiOutcome::from_envelope(value)
                        .expect("core envelope schema")
                        .into_envelope()
                })
                .pipe(Ok)
        });
        let output = match result {
            Ok(Ok(v)) => v,
            Ok(Err(message)) => failure(&self.operation, "operation", "operation-failed", &message),
            Err(()) => failure(
                &self.operation,
                "native",
                "native-panic",
                "native compute panicked",
            ),
        };
        serde_json::to_string(&output).map_err(|e| Error::from_reason(e.to_string()))
    }
    fn resolve(&mut self, _env: Env, output: String) -> Result<ffi::Wire> {
        match caught(|| {
            fault("resolve");
            output
        }) {
            Ok(out) => Ok(out),
            Err(()) => Ok(failure(
                &self.operation,
                "native",
                "native-panic",
                "native resolve panicked",
            )
            .to_string()),
        }
        .map(|text| ffi::Wire(text, self.operation.clone()))
    }
    fn reject(&mut self, _env: Env, error: Error) -> Result<ffi::Wire> {
        match caught(|| {
            fault("reject");
            failure(&self.operation, "worker", "worker-failed", &error.reason).to_string()
        }) {
            Ok(out) => Ok(out),
            Err(()) => Ok(failure(
                &self.operation,
                "native",
                "native-panic",
                "native reject panicked",
            )
            .to_string()),
        }
        .map(|text| ffi::Wire(text, self.operation.clone()))
    }
}
fn fault(_stage: &str) {
    #[cfg(feature = "test-node-runtime")]
    test_runtime::fault(_stage);
}
trait Pipe: Sized {
    fn pipe<T>(self, f: impl FnOnce(Self) -> T) -> T {
        f(self)
    }
}
impl<T> Pipe for T {}
