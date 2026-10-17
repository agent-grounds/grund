//! Logical copy identities and explicit report bases (§FS-cli.3.4,
//! §REQ-one-answer.proof.matrix). Normalize before serialization, never Debug.

use super::agreement_oracles::text;
use serde_json::Value;
use std::path::{Path, PathBuf};

fn boundary(c: Option<char>) -> bool {
    c.is_none_or(|c| {
        c.is_whitespace() || matches!(c, '/' | '\\' | ':' | '"' | '\'' | ')' | ']' | ',')
    })
}

/// Replace only a complete copy-root prefix, retaining the owner-relative suffix.
/// Both native and slash-rendered spellings occur in the published contracts.
pub fn logical_text(value: &str, context: &Value) -> String {
    let mut result = value.to_string();
    let mut roots: Vec<_> = ["base", "saved"]
        .into_iter()
        .filter_map(|field| context[field].as_str())
        .flat_map(|root| {
            let mut aliases = vec![root.to_string()];
            if root.as_bytes().get(1) == Some(&b':') {
                aliases.push(format!("\\\\?\\{root}"));
            } else if root.starts_with("\\\\") && !root.starts_with("\\\\?\\") {
                aliases.push(format!("\\\\?\\UNC\\{}", &root[2..]));
            } else if let Some(short) = root.strip_prefix("\\\\?\\") {
                aliases.push(
                    short
                        .strip_prefix("UNC\\")
                        .map_or_else(|| short.to_string(), |unc| format!("\\\\{unc}")),
                );
            }
            aliases.into_iter().flat_map(|root| {
                let mut spellings = vec![root.clone(), root.replace('\\', "/")];
                // A root written with `/` is spelled with `\` once Windows
                // joins a site under it.
                if cfg!(windows) {
                    spellings.push(root.replace('/', "\\"));
                }
                spellings
            })
        })
        .collect();
    roots.sort_by_key(|root| std::cmp::Reverse(root.len()));
    roots.dedup();
    for root in roots {
        if root.is_empty() {
            continue;
        }
        let mut replaced = String::new();
        let mut rest = result.as_str();
        while let Some(index) = rest.find(&root) {
            let end = index + root.len();
            replaced.push_str(&rest[..index]);
            let before = rest[..index].chars().next_back();
            if boundary(before) && boundary(rest[end..].chars().next()) {
                replaced.push_str("<repo>");
            } else {
                replaced.push_str(&rest[index..end]);
            }
            rest = &rest[end..];
        }
        replaced.push_str(rest);
        result = replaced;
    }
    result
}

pub fn logical_path(path: &str, context: &Value) -> String {
    logical_text(path, context).replace('\\', "/")
}

/// Lossless envelopes contain nested serialized show JSON. Decode it before
/// path mapping so JSON escaping cannot hide an absolute copy identity.
pub fn logical_value(mut value: Value, context: &Value) -> Value {
    fn visit(value: &mut Value, context: &Value) {
        match value {
            Value::String(s) => {
                if let Ok(mut nested) = serde_json::from_str::<Value>(s)
                    && (nested.is_object() || nested.is_array())
                {
                    visit(&mut nested, context);
                    *s = nested.to_string();
                } else {
                    *s = logical_text(s, context);
                }
            }
            Value::Array(values) => values.iter_mut().for_each(|v| visit(v, context)),
            Value::Object(values) => values.values_mut().for_each(|v| visit(v, context)),
            _ => {}
        }
    }
    visit(&mut value, context);
    value
}

/// A producer's report base is evidence, not a guess based on which file exists.
pub fn physical(context: &Value, path: &str) -> PathBuf {
    let path = Path::new(path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        PathBuf::from(text(context, "reportBase")).join(path)
    }
}

/// Remap physical saved/held copies component-wise for snapshot comparisons.
pub fn saved_path(context: &Value, path: &Path) -> PathBuf {
    let base = text(context, "base");
    // Windows canonical paths carry a verbatim prefix. This is the same root's
    // filesystem identity, not an alternative owner selected by file existence.
    let canonical_base = Path::new(&base)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(&base));
    match path
        .strip_prefix(&base)
        .or_else(|_| path.strip_prefix(&canonical_base))
    {
        Ok(relative) => PathBuf::from(text(context, "saved")).join(relative),
        Err(_) => path.to_path_buf(),
    }
}
