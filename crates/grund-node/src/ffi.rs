//! Guard conversion and scheduling as well as worker stages (§FS-distribution.3.2.3.1).
use super::{Job, caught, failure};
use napi::bindgen_prelude::{AsyncTask, FromNapiValue, ToNapiValue, TypeName, ValidateNapiValue};
use napi::{Error, Result, ValueType};

pub struct Input(pub String);
impl TypeName for Input {
    fn type_name() -> &'static str {
        "String"
    }
    fn value_type() -> ValueType {
        ValueType::String
    }
}
impl ValidateNapiValue for Input {}
impl FromNapiValue for Input {
    unsafe fn from_napi_value(
        env: napi::sys::napi_env,
        value: napi::sys::napi_value,
    ) -> Result<Self> {
        super::runtime::install_hook();
        // SAFETY: napi-rs supplies an operand in this call's live environment;
        // String's converter checks its type and owns the copied bytes.
        caught(|| unsafe { String::from_napi_value(env, value) })
            .unwrap_or_else(|_| {
                Err(Error::from_reason(
                    failure(
                        "invoke",
                        "native",
                        "native-panic",
                        "native input conversion panicked",
                    )
                    .to_string(),
                ))
            })
            .map(Self)
    }
}
pub struct Dispatch(pub AsyncTask<Job>, pub String);
impl TypeName for Dispatch {
    fn type_name() -> &'static str {
        "Promise"
    }
    fn value_type() -> ValueType {
        ValueType::Object
    }
}
impl ToNapiValue for Dispatch {
    unsafe fn to_napi_value(
        env: napi::sys::napi_env,
        value: Self,
    ) -> Result<napi::sys::napi_value> {
        let Self(task, operation) = value;
        // SAFETY: this conversion runs on the originating live JS thread.
        caught(|| unsafe { AsyncTask::to_napi_value(env, task) }).unwrap_or_else(|_| {
            Err(Error::from_reason(
                failure(
                    &operation,
                    "native",
                    "native-panic",
                    "native dispatch panicked",
                )
                .to_string(),
            ))
        })
    }
}
pub struct Wire(pub String, pub String);
impl TypeName for Wire {
    fn type_name() -> &'static str {
        "String"
    }
    fn value_type() -> ValueType {
        ValueType::String
    }
}
impl ToNapiValue for Wire {
    unsafe fn to_napi_value(
        env: napi::sys::napi_env,
        value: Self,
    ) -> Result<napi::sys::napi_value> {
        let Self(text, operation) = value;
        // SAFETY: napi-rs only converts task values in its live completion callback.
        caught(|| unsafe { String::to_napi_value(env, text) }).unwrap_or_else(|_| {
            Err(Error::from_reason(
                failure(
                    &operation,
                    "native",
                    "native-panic",
                    "native result conversion panicked",
                )
                .to_string(),
            ))
        })
    }
}
