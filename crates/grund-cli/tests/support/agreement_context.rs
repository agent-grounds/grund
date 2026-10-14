//! Stable invocation identities preserve held/saved roles and raw buffer text
//! (§REQ-one-answer.proof.matrix, §FS-scan.identity.buffers).

use serde_json::{Value, json};

fn windows(path: &str) -> bool {
    path.as_bytes().get(1) == Some(&b':') || path.starts_with("\\\\") || path.starts_with("//")
}

fn spelling(path: &str, windows_context: bool) -> String {
    if !windows_context && !windows(path) {
        return path.into(); // A Unix filename can contain a literal backslash.
    }
    let path = path.replace('\\', "/");
    if let Some(unc) = path.strip_prefix("//?/UNC/") {
        format!("//{unc}")
    } else {
        path.strip_prefix("//?/").unwrap_or(&path).into()
    }
}

/// Only filesystem fields call this function. Text and arbitrary map keys never
/// undergo separator, JSON or root substitution.
pub fn path(value: &str, context: &Value) -> String {
    let base = context["base"].as_str().unwrap_or_default();
    let saved = context["saved"].as_str().unwrap_or_default();
    let windows_context = windows(base) || windows(saved);
    let base = spelling(base, windows_context);
    let saved = spelling(saved, windows_context);
    let value = spelling(value, windows_context);
    let same = base == saved;
    let mut roots = vec![
        (base, if same { "<repo>" } else { "<held>" }),
        (saved, if same { "<repo>" } else { "<saved>" }),
    ];
    roots.sort_by_key(|(root, _)| std::cmp::Reverse(root.len()));
    for (root, role) in roots {
        if root.is_empty() {
            continue;
        }
        if value == root {
            return role.into();
        }
        if let Some(suffix) = value.strip_prefix(&root)
            && suffix.starts_with('/')
        {
            return format!("{role}{suffix}");
        }
    }
    value
}

fn field(value: &Value, context: &Value) -> Value {
    value
        .as_str()
        .map_or_else(|| value.clone(), |p| json!(path(p, context)))
}

/// Keep the exact schema (including nulls and unknown scope fields), mapping only
/// its named filesystem locations. Root-prefix aliases must not merge overlays.
pub fn identity(context: &Value) -> Result<Value, String> {
    let mut actual = json!({});
    for name in super::agreement_truth::CONTEXT {
        actual[*name] = context[*name].clone();
    }
    for name in ["relative", "path", "cwd", "editorPath", "overlayRelative"] {
        actual[name] = field(&actual[name], context);
    }
    if let Some(overlays) = actual["overlays"].as_object_mut() {
        let mut normalized = serde_json::Map::new();
        for (key, text) in overlays.iter() {
            if normalized
                .insert(path(key, context), text.clone())
                .is_some()
            {
                return Err("distinct overlay keys share one logical filesystem identity".into());
            }
        }
        *overlays = normalized;
    }
    if let Some(scopes) = actual["scopes"].as_object_mut() {
        for scope in scopes.values_mut().filter_map(Value::as_object_mut) {
            for name in ["projectRoot", "configFile", "invocationBase", "reportBase"] {
                if let Some(value) = scope.get_mut(name) {
                    *value = field(value, context);
                }
            }
        }
    }
    Ok(actual)
}
