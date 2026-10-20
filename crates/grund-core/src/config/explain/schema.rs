//! §AR-config.9.1: the schema `grund config explain` words a key from, the
//! published `schemas/grund.schema.json` without its spec metadata, which
//! `scripts/render_config_reference.py` renders into the asset embedded here.
//! It only words what the gates admit; it never decides what there is
//! (§FS-config-explain.3).

use once_cell::sync::Lazy;
use serde_json::Value;

const EMBEDDED: &str = include_str!("../../../assets/explain/grund.schema.json");

static SCHEMA: Lazy<Value> =
    Lazy::new(|| serde_json::from_str(EMBEDDED).expect("the embedded schema parses"));

/// The embedded schema.
pub(super) fn embedded() -> &'static Value {
    &SCHEMA
}

/// `node` with its local `$ref`s followed.
fn resolve<'a>(schema: &'a Value, mut node: &'a Value) -> &'a Value {
    while let Some(reference) = node.get("$ref").and_then(Value::as_str) {
        let name = reference
            .strip_prefix("#/definitions/")
            .unwrap_or(reference);
        node = &schema["definitions"][name];
    }
    node
}

/// A table's node with its `$ref` followed, an array of tables read as its rows.
fn body<'a>(schema: &'a Value, node: &'a Value) -> &'a Value {
    let node = resolve(schema, node);
    match node.get("items") {
        Some(items) if node.get("properties").is_none() => body(schema, items),
        _ => node,
    }
}

/// The child `name` of a table's node; a `<placeholder>` is the node's any-name
/// child, which its `x-grund-key` names.
fn child<'a>(schema: &'a Value, node: &'a Value, name: &str) -> Option<&'a Value> {
    let node = body(schema, node);
    if let Some(found) = node.get("properties").and_then(|props| props.get(name)) {
        return Some(found);
    }
    let any = node
        .get("additionalProperties")
        .filter(|any| any.is_object());
    any.filter(|_| name.starts_with('<'))
}

/// The node of the table `header` (`[schema.kinds.<NAME>]`, `[[kinds]]`, `""`
/// for the top level) under `version`'s definition, unresolved.
fn table_node<'a>(schema: &'a Value, version: u32, header: &str) -> Option<&'a Value> {
    let root = &schema["definitions"][if version >= 2 { "v2" } else { "v1" }];
    let path = header.trim_matches(['[', ']']);
    let mut segments = path.split('.').filter(|segment| !segment.is_empty());
    segments.try_fold(root, |node, segment| child(schema, node, segment))
}

/// The node of `key` in the table `header`, an inline member spelled
/// `key.member`, resolved.
pub(super) fn key_node<'a>(
    schema: &'a Value,
    version: u32,
    header: &str,
    key: &str,
) -> Option<&'a Value> {
    let table = table_node(schema, version, header)?;
    let node = key
        .split('.')
        .try_fold(table, |node, name| child(schema, node, name))?;
    Some(resolve(schema, node))
}

/// What the table `header` is for: the description on its node, or on its rows'
/// node for an array of tables.
pub(super) fn table_meaning(schema: &Value, version: u32, header: &str) -> Option<String> {
    let node = table_node(schema, version, header)?;
    let resolved = resolve(schema, node);
    [node, resolved, body(schema, node)]
        .into_iter()
        .find_map(meaning)
}

/// A node's description, as the schema writes it.
pub(super) fn meaning(node: &Value) -> Option<String> {
    node.get("description")
        .and_then(Value::as_str)
        .map(str::to_string)
}

/// §FS-config-explain.3.1: a key's values in words, the rendered page's words
/// with its Markdown aside (`scripts/render_config_reference.py`, `type_of`).
pub(super) fn value_words(schema: &Value, node: &Value) -> String {
    let node = resolve(schema, node);
    if node
        .get("not")
        .and_then(Value::as_object)
        .is_some_and(|not| not.is_empty())
    {
        return "refused".to_string();
    }
    if let Some(forms) = node.get("anyOf").and_then(Value::as_array) {
        let words = forms.iter().map(|form| match form.get("const") {
            Some(value) => toml_value(value),
            None => value_words(schema, form),
        });
        return words.collect::<Vec<_>>().join(" or ");
    }
    if let Some(values) = node.get("enum").and_then(Value::as_array) {
        return closed_set(values);
    }
    let kind = node.get("type").and_then(Value::as_str);
    let bound = |name: &str| node.get(name).and_then(Value::as_i64);
    match (kind, bound("minimum"), bound("maximum")) {
        (Some("integer"), Some(low), Some(high)) => format!("integer, {low}–{high}"),
        (Some("integer"), Some(low), None) => format!("integer, at least {low}"),
        (Some("array"), ..) => {
            let items = node.get("items").map(|items| resolve(schema, items));
            match items
                .and_then(|items| items.get("enum"))
                .and_then(Value::as_array)
            {
                Some(values) => format!("array of {}", closed_set(values)),
                // An array of tables, `[[kinds]]`, gets a heading of its own on the page.
                None => match items
                    .and_then(|items| items.get("type"))
                    .and_then(Value::as_str)
                {
                    Some("object") => "array of tables".to_string(),
                    kind => format!("array of {}s", kind.unwrap_or("value")),
                },
            }
        }
        (Some("object"), ..) if node.get("x-grund-inline").is_some_and(|v| v != false) => {
            let members = node.get("properties").and_then(Value::as_object);
            let names: Vec<&str> = members
                .into_iter()
                .flat_map(|m| m.keys())
                .map(String::as_str)
                .collect();
            format!("inline table {{ {} }}", names.join(", "))
        }
        (Some("object"), ..) => "table".to_string(),
        (Some(kind), ..) => kind.to_string(),
        (None, ..) => "value".to_string(),
    }
}

/// A closed set of values, each as TOML, joined by ` | `.
fn closed_set(values: &[Value]) -> String {
    values
        .iter()
        .map(toml_value)
        .collect::<Vec<_>>()
        .join(" | ")
}

/// A JSON value as TOML writes it; a TOML basic string is a JSON string.
fn toml_value(value: &Value) -> String {
    match value {
        Value::Array(items) => {
            let items: Vec<String> = items.iter().map(toml_value).collect();
            format!("[{}]", items.join(", "))
        }
        other => other.to_string(),
    }
}
