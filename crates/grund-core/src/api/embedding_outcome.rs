//! Shared typed envelope independent of a host runtime (§FS-distribution.3.2.2.1).
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiFailureSite {
    pub path: String,
    pub line: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiPartialOutput {
    pub operation: String,
    pub result: Value,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiFailure {
    pub kind: String,
    pub code: String,
    pub operation: String,
    pub message: String,
    pub path: Option<String>,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub sites: Vec<ApiFailureSite>,
    pub authority: Vec<String>,
    pub causes: Vec<String>,
    pub details: Value,
    pub partial: Option<ApiPartialOutput>,
}
/// Cautions remain beside both completed output and a later refusal.
pub struct ApiOutcome<T> {
    pub run_cautions: Vec<Value>,
    pub result: Result<T, ApiFailure>,
}
impl ApiOutcome<Value> {
    pub fn from_envelope(mut value: Value) -> Result<Self, serde_json::Error> {
        let run_cautions = value["run_cautions"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let result = if value["failure"].is_null() {
            Ok(value["result"].take())
        } else {
            Err(serde_json::from_value(value["failure"].take())?)
        };
        Ok(Self {
            run_cautions,
            result,
        })
    }
    pub fn into_envelope(self) -> Value {
        match self.result {
            Ok(result) => json!({"result":result,"failure":null,"run_cautions":self.run_cautions}),
            Err(failure) => {
                json!({"result":null,"failure":failure,"run_cautions":self.run_cautions})
            }
        }
    }
}
