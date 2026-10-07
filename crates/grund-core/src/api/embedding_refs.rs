//! Refs transport folds preserve the CLI's path order and unique lines (§FS-distribution.3.2.2.2).
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
pub(super) fn summaries(hits: &Value) -> Value {
    let mut by_file = BTreeMap::<String, (Value, usize, BTreeSet<u64>)>::new();
    for h in hits.as_array().into_iter().flatten() {
        let row = by_file
            .entry(h["path"].as_str().unwrap_or("").into())
            .or_insert_with(|| (h["project"].clone(), 0, BTreeSet::new()));
        row.1 += 1;
        row.2.insert(h["line"].as_u64().unwrap_or(0));
    }
    json!(by_file.into_iter().map(|(path,(project,count,lines))|
        json!({"project":project,"path":path,"count":count,"lines":lines})).collect::<Vec<_>>())
}
