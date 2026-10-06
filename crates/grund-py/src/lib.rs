//! Python conversion only; all behavior is core-owned (§AR-bindings.6).

use grund_core::{EmbeddingRequest, embedding_call};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

/// The private native transport releases the GIL for the entire engine call
/// and delivers pending interrupts when it returns (§FS-distribution.3.3.4).
#[pyfunction]
fn call(
    py: Python<'_>,
    operation: String,
    root: String,
    explicit: bool,
    args: String,
    options: String,
) -> PyResult<String> {
    let request = EmbeddingRequest {
        operation,
        root: root.into(),
        explicit,
        args: serde_json::from_str(&args).map_err(|e| PyValueError::new_err(e.to_string()))?,
        options: serde_json::from_str(&options)
            .map_err(|e| PyValueError::new_err(e.to_string()))?,
    };
    let result = py.detach(move || embedding_call(request));
    py.check_signals()?;
    Ok(result.to_string())
}

/// Reuse engine vocabularies for host-side option validation (§FS-distribution.3.3.5).
#[pyfunction]
fn valid_option(name: &str, value: &str) -> bool {
    match name {
        "code" => grund_core::CHECK_FINDING_CODES
            .binary_search(&value)
            .is_ok(),
        "client" => grund_core::IntegrationClient::from_name(value).is_some(),
        "conversation" => grund_core::ConversationRendering::from_name(value).is_some(),
        "conversation_target" => grund_core::ConversationTarget::from_name(value).is_some(),
        "agent" => grund_core::known_agent(value).is_some(),
        _ => false,
    }
}

/// Private extension, deliberately without a competing CLI entrypoint
/// (§FS-distribution.3.3.7).
#[pymodule]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(call, module)?)?;
    module.add_function(wrap_pyfunction!(valid_option, module)?)?;
    Ok(())
}
