//! Source-classified operational errors for embedders (§FS-distribution.3.1).

use serde_json::{Value, json};

/// Additive context: Display stays verbatim, while locations and candidates
/// cross the binding boundary as data (§FS-distribution.3.3.2).
#[derive(Clone, Debug)]
pub struct OperationDiagnostic {
    pub class: &'static str,
    pub code: &'static str,
    pub message: String,
    pub path: Option<String>,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub details: Value,
    source: Option<std::sync::Arc<anyhow::Error>>,
}

impl OperationDiagnostic {
    /// A refusal without a source location (§FS-distribution.3.3.2).
    pub fn new(class: &'static str, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            class,
            code,
            message: message.into(),
            path: None,
            line: None,
            column: None,
            details: json!({}),
            source: None,
        }
    }

    /// Replace only the original error's head, keeping its Display and source
    /// chain verbatim for existing callers (§FS-distribution.3.1).
    pub(crate) fn from_error(
        class: &'static str,
        code: &'static str,
        error: anyhow::Error,
    ) -> Self {
        let mut diagnostic = Self::new(class, code, error.to_string());
        if let Some(io) = error.downcast_ref::<std::io::Error>() {
            diagnostic.details = json!({"os_error": io.raw_os_error()});
        }
        diagnostic.source = Some(std::sync::Arc::new(error));
        diagnostic
    }

    /// Preserve OS classification where a writer still has the original error
    /// (§FS-distribution.3.3.2).
    pub(crate) fn filesystem(
        path: &std::path::Path,
        error: &std::io::Error,
        message: String,
    ) -> Self {
        let mut diagnostic = Self::new("filesystem", "io", message);
        diagnostic.path = Some(super::paths::format_path(path));
        diagnostic.details = json!({"os_error":error.raw_os_error()});
        diagnostic
    }
}

impl std::fmt::Display for OperationDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for OperationDiagnostic {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .and_then(|error| error.as_ref().source())
    }
}

/// Run metadata beside a late refusal, independent of host exception policy
/// (§FS-distribution.3.1, §FS-distribution.3.3.2).
#[derive(Debug)]
pub(crate) struct OperationContext {
    pub(crate) message: String,
    pub(crate) cautions: Value,
    pub(crate) partial_output: Value,
}

impl std::fmt::Display for OperationContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for OperationContext {}
