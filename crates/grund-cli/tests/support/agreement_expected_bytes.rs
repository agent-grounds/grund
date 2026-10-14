//! Input/output inventories pin relevant bytes and every unchanged file
//! (§REQ-one-answer.proof.evidence, §REQ-one-answer.writes.anchors, §FS-fmt.6.2).

use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::Path};

pub fn inventory(root: &Path) -> Value {
    fn walk(root: &Path, dir: &Path, files: &mut BTreeMap<String, Value>) {
        for entry in fs::read_dir(dir).expect("independent physical byte inventory") {
            let path = entry.unwrap().path();
            let relative = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            let value = if path.is_symlink() {
                json!({"symlink":fs::read_link(&path).unwrap().to_string_lossy().replace('\\', "/")})
            } else if path.is_dir() {
                walk(root, &path, files);
                json!({"directory":true})
            } else {
                json!({"bytes":fs::read(&path).unwrap()})
            };
            files.insert(relative, value);
        }
    }
    let mut files = BTreeMap::new();
    walk(root, root, &mut files);
    json!(files)
}

/// The patches are authored output facts; the baseline is read before any writer
/// runs. An explicitly empty patch pins unchanged bytes rather than inferring
/// them from a successful or empty formatter answer.
pub fn resulting(before: &Value, changes: &Value) -> Result<Value, String> {
    let mut expected = before
        .as_object()
        .ok_or("missing before inventory")?
        .clone();
    for (path, value) in changes
        .as_object()
        .ok_or("missing authored writtenChanges")?
    {
        if !safe_relative(path) {
            return Err(format!("unsafe authored output path {path}"));
        }
        if value == &json!({"remove":true}) {
            if expected.remove(path).is_none() {
                return Err(format!("authored removal has no input file: {path}"));
            }
        } else if valid_entry(value) {
            expected.insert(path.clone(), value.clone());
        } else {
            return Err(format!("invalid authored bytes for {path}"));
        }
    }
    Ok(json!(expected))
}

pub fn safe_relative(path: &str) -> bool {
    !path.is_empty()
        && !path.contains('\\')
        && !path.contains(':')
        && Path::new(path)
            .components()
            .all(|part| matches!(part, std::path::Component::Normal(_)))
}

pub fn valid_entry(value: &Value) -> bool {
    value.as_object().is_some_and(|v| v.len() == 1)
        && (value == &json!({"directory":true})
            || value["symlink"].is_string()
            || value["bytes"]
                .as_array()
                .is_some_and(|bytes| bytes.iter().all(|b| b.as_u64().is_some_and(|n| n <= 255))))
}
