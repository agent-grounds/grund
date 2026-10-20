//! The published schema, read the way [§FS-config-explain.3](../../../docs/functional-spec/FS-config-explain.md#3-a-keys-answer)
//! words a key: its description and its values.

#![allow(dead_code)]

use serde_json::Value;

use crate::explain_catalog::printed;

/// The published schema's node for one version.
pub fn schema_root(schema: &Value, version: u32) -> &Value {
    &schema["definitions"][if version == 2 { "v2" } else { "v1" }]
}

/// A node with its `$ref` followed.
pub fn resolve<'a>(schema: &'a Value, node: &'a Value) -> &'a Value {
    match node.get("$ref").and_then(Value::as_str) {
        Some(reference) => {
            let name = reference.trim_start_matches("#/definitions/");
            &schema["definitions"][name]
        }
        None => node,
    }
}

/// A table node with its `$ref` followed, and an array of tables read as its items.
pub fn deref<'a>(schema: &'a Value, node: &'a Value) -> &'a Value {
    let node = resolve(schema, node);
    match node.get("items") {
        Some(items) if node.get("properties").is_none() => deref(schema, items),
        _ => node,
    }
}

/// The child `name` of a table node, a `<placeholder>` read as any name.
pub fn child<'a>(schema: &'a Value, node: &'a Value, name: &str) -> Option<&'a Value> {
    let node = deref(schema, node);
    if let Some(found) = node.get("properties").and_then(|props| props.get(name)) {
        return Some(found);
    }
    if !name.starts_with('<') {
        return None;
    }
    let any = node
        .get("additionalProperties")
        .filter(|any| any.is_object());
    let pattern = node
        .get("patternProperties")
        .and_then(Value::as_object)
        .and_then(|patterns| patterns.values().next());
    any.or(pattern)
}

/// The schema node of `key` under `header`, `""` for the top level.
pub fn key_schema<'a>(
    schema: &'a Value,
    version: u32,
    header: &str,
    key: &str,
) -> Option<&'a Value> {
    let path = header.trim_start_matches('[').trim_end_matches(']');
    let mut node = schema_root(schema, version);
    let segments = path.split('.').filter(|segment| !segment.is_empty());
    for segment in segments.chain(key.split('.')) {
        node = child(schema, node, segment)?;
    }
    Some(resolve(schema, node))
}

/// A description as an answer prints it: its specification link dropped and
/// its backticks with it.
pub fn description_words(node: &Value) -> Option<String> {
    let description = node.get("description")?.as_str()?;
    let words = description.split(" ([§").next().unwrap_or(description);
    Some(printed(words))
}

/// §FS-config-explain.3.1: the words a key's values must be put in, each one
/// that the answer must carry.
pub fn value_words(schema: &Value, node: &Value) -> Vec<String> {
    let node = resolve(schema, node);
    if let Some(values) = node.get("enum").and_then(Value::as_array) {
        return values.iter().map(Value::to_string).collect();
    }
    for alternatives in ["oneOf", "anyOf"] {
        if let Some(forms) = node.get(alternatives).and_then(Value::as_array) {
            return forms
                .iter()
                .flat_map(|form| value_words(schema, form))
                .collect();
        }
    }
    if node.get("x-grund-inline").is_some() {
        return vec!["inline table".to_string()];
    }
    let types: Vec<&str> = match node.get("type") {
        Some(Value::String(one)) => vec![one.as_str()],
        Some(Value::Array(many)) => many.iter().filter_map(Value::as_str).collect(),
        _ => Vec::new(),
    };
    types
        .into_iter()
        .map(|kind| match kind {
            "array" => "array of".to_string(),
            "object" => "table".to_string(),
            other => other.to_string(),
        })
        .collect()
}
