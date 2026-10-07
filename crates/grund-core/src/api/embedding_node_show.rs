//! Node's show and batch records: the shared query, plus the metadata Node's
//! records carry, read from the same loaded context (§FS-distribution.3.2.2.2).
//! The shared delegate keeps Python's records exactly as they are.

use super::embedding::EmbeddingRequest;
use super::embedding_data::Data;
use super::embedding_failure::error_data;
use super::embedding_queries::show_opts;
use crate::BatchShowQuery;
use serde_json::{Value, json};

/// Node's read/query delegate: show and batch add metadata, the rest is shared.
pub(super) fn query(r: &EmbeddingRequest) -> Result<Value, Value> {
    match r.operation.as_str() {
        "show" => show(r),
        "show_batch" => show_batch(r),
        _ => super::embedding_queries::query(r),
    }
}

fn show(r: &EmbeddingRequest) -> Result<Value, Value> {
    let (cautions, result) = super::show::show_data(r.arg(0), show_opts(r), r.explicit);
    let (output, metadata) = result.map_err(|e| error_data(e, &cautions))?;
    let mut data = output.data();
    extend(&mut data, metadata);
    data["run_cautions"] = cautions.data();
    Ok(data)
}

fn show_batch(r: &EmbeddingRequest) -> Result<Value, Value> {
    let queries = r
        .args
        .first()
        .filter(|v| !v.is_null())
        .and_then(Value::as_array)
        .map(|queries| {
            queries
                .iter()
                .map(|q| BatchShowQuery {
                    id: q
                        .as_str()
                        .or_else(|| q["id"].as_str())
                        .unwrap_or_default()
                        .to_owned(),
                    section: q["section"].as_str().map(str::to_owned),
                })
                .collect()
        });
    let (cautions, result) = crate::queries::show_batch_data(queries, show_opts(r), r.explicit);
    let records = result
        .map_err(|e| error_data(e, &cautions))?
        .into_iter()
        .map(|record| {
            let (result, refusal) = match record.result {
                Ok(value) => {
                    let mut data = value.data();
                    extend(&mut data, record.metadata);
                    data["run_cautions"] = cautions.data();
                    (data, Value::Null)
                }
                Err(e) => (Value::Null, error_data(e, &cautions)),
            };
            json!({"query":{"id":record.query.id,"section":record.query.section},
            "result":result,"failure":refusal})
        })
        .collect::<Vec<_>>();
    Ok(json!({"records":records,"run_cautions":cautions.data()}))
}

fn extend(data: &mut Value, metadata: Value) {
    if let (Some(data), Value::Object(metadata)) = (data.as_object_mut(), metadata) {
        data.extend(metadata);
    }
}
