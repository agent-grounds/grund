//! A manifest's compact facts expanded, in memory and before any validation,
//! into the exact invocation context, physical query inventory and answer of
//! every question and producer (§REQ-one-answer.proof.evidence.closed). Each
//! fact is pinned once per input and mapped into each producer's schema; no
//! answer is read from an engine, a binding or a returned catalog.

use super::agreement_render_inventory::inventory;
use super::agreement_render_model::Model;
use super::agreement_render_tree::View;
use serde_json::{Value, json};
use std::collections::BTreeMap;

const TREE: &[(&str, &str)] = &[
    ("check", "§FS-check.1.1"),
    ("list", "§REQ-one-answer.identity.declarations"),
    ("list-size", "§REQ-one-answer.sections.map"),
    ("cover", "§FS-scan.path.cover"),
    ("cover-lines", "§FS-cover.6.1"),
    ("fmt", "§REQ-one-answer.writes.anchors"),
    ("show-batch", "§REQ-one-answer.sections.map"),
    ("fmt-write", "§REQ-one-answer.writes.anchors"),
];

fn join(root: &str, relative: &str) -> String {
    match relative {
        "." | "" => root.into(),
        relative => format!("{root}/{relative}"),
    }
}

fn parent(relative: &str) -> &str {
    relative.rsplit_once('/').map_or(".", |(p, _)| p)
}

pub fn input_files(document: &Value) -> BTreeMap<String, Vec<u8>> {
    document["inputFiles"]
        .as_object()
        .into_iter()
        .flatten()
        .filter_map(|(path, entry)| {
            let bytes = entry["bytes"].as_array()?;
            let relative = path.strip_prefix("repo/")?;
            Some((
                relative.to_string(),
                bytes.iter().map(|b| b.as_u64().unwrap() as u8).collect(),
            ))
        })
        .collect()
}

/// §FS-scan.text.buffer: the held edit is the input text plus one newline.
fn held(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    format!("{}\n", text.replace("\r\n", "\n").replace('\r', "\n"))
}

/// Python's `read_text(encoding='utf-8')` accepts the file, and nothing
/// about its name keeps it out of the readable inventory.
fn readable(path: &str, bytes: &[u8]) -> bool {
    !matches!(path.rsplit('/').next(), Some("grund.toml" | "AGENTS.md"))
        && !path.split('/').any(|p| p.starts_with('.'))
        && std::str::from_utf8(bytes).is_ok()
}

/// `pathlib` order: component by component, as the materializer sorts.
fn components(path: &str) -> Vec<&str> {
    path.split('/').collect()
}

/// The materializer's choice of path for each variant (`agreement_fixtures.py`),
/// read from the input inventory: the first directory holding a readable file
/// (else any directory), the first readable file (else any file), and the
/// first readable Markdown file the held edit lands in.
fn selection(
    document: &Value,
    files: &BTreeMap<String, Vec<u8>>,
    variant: &str,
) -> (String, Option<String>) {
    let mut readable: Vec<&String> = files
        .iter()
        .filter(|(f, b)| readable(f, b))
        .map(|(f, _)| f)
        .collect();
    readable.sort_by(|a, b| components(a).cmp(&components(b)));
    let mut subdirs: Vec<&str> = readable
        .iter()
        .filter(|f| f.contains('/'))
        .map(|f| parent(f))
        .collect();
    if subdirs.is_empty() {
        subdirs = document["inputFiles"]
            .as_object()
            .unwrap()
            .iter()
            .filter(|(_, v)| v["directory"] == true)
            .filter_map(|(p, _)| p.strip_prefix("repo/"))
            .collect();
    }
    subdirs.sort_by(|a, b| components(a).cmp(&components(b)));
    subdirs.dedup();
    let mut all: Vec<&String> = files.keys().collect();
    all.sort_by(|a, b| components(a).cmp(&components(b)));
    match variant {
        "narrowed" => (subdirs[0].into(), None),
        "path-argument" => (
            readable
                .first()
                .or(all.first())
                .map_or_else(|| subdirs[0].into(), |f| f.to_string()),
            None,
        ),
        "editor-overlay" => {
            let overlay = readable
                .iter()
                .find(|f| f.ends_with(".md"))
                .or(readable.first())
                .map_or_else(|| "agreement-overlay.md".into(), |f| f.to_string());
            (".".into(), Some(overlay))
        }
        _ => (".".into(), None),
    }
}

struct Variant<'a> {
    name: &'a str,
    relative: String,
    overlay: Option<String>,
    project: Value,
}

impl Variant<'_> {
    fn roles(&self) -> (&'static str, &'static str) {
        if self.overlay.is_some() {
            ("<held>", "<saved>")
        } else {
            ("<repo>", "<repo>")
        }
    }

    /// §FS-config.1: the nearest `grund.toml` (or `.agents/grund.toml`) at or
    /// above `start`, and the directory that owns it.
    fn discover(files: &BTreeMap<String, Vec<u8>>, start: &str) -> Option<(String, String)> {
        let mut dir = start.to_string();
        loop {
            for config in ["grund.toml", ".agents/grund.toml"] {
                let path = if dir == "." {
                    config.to_string()
                } else {
                    format!("{dir}/{config}")
                };
                if files.contains_key(&path) {
                    return Some((dir, path));
                }
            }
            if dir == "." {
                return None;
            }
            dir = parent(&dir).to_string();
        }
    }

    /// §FS-config.1, §FS-config.3.6.1: where each producer loads and reports.
    fn scope(&self, files: &BTreeMap<String, Vec<u8>>, root: &str, surface: &str) -> Value {
        let file = files.contains_key(&self.relative);
        let start = if file {
            parent(&self.relative)
        } else {
            self.relative.as_str()
        };
        let cwd = if self.name == "narrowed" {
            self.relative.as_str()
        } else {
            "."
        };
        let found = match self.project.get("root") {
            Some(owner) => self.project["config"]
                .as_str()
                .map(|c| (owner.as_str().unwrap().into(), c.into())),
            None => Self::discover(files, start),
        };
        let owner = match &found {
            Some((owner, _)) => owner.clone(),
            None if matches!(surface, "python" | "node") => start.into(),
            None if surface == "editor" && self.name != "narrowed" && !file => start.into(),
            None => cwd.into(),
        };
        let relative_paths = found.as_ref().is_none_or(|(_, config)| {
            !String::from_utf8_lossy(&files[config]).contains("relative_paths = false")
        });
        let report = if relative_paths {
            owner.clone()
        } else {
            start.into()
        };
        json!({"projectRoot":join(root, &owner),
            "configFile":found.map(|(_, c)| join(root, &c)),
            "invocationBase":join(root, start),"reportBase":join(root, &report),
            "parseError":self.project["parseError"]})
    }

    fn context(&self, files: &BTreeMap<String, Vec<u8>>) -> Value {
        let (held_root, saved_root) = self.roles();
        let path = join(saved_root, &self.relative);
        let held = self.overlay.as_ref().map(|o| {
            files
                .get(o)
                .map_or_else(|| "Agreement buffer control.\n".to_string(), |b| held(b))
        });
        let overlays = match (&self.overlay, &held) {
            (Some(o), Some(h)) => json!({join(held_root, o): h}),
            _ => json!({}),
        };
        json!({"relative":self.relative,"path":path,
            "cwd":if self.name == "narrowed" { path.clone() } else { saved_root.to_string() },
            "editorPath":join(held_root, &self.relative),"explicit":self.name != "narrowed",
            "overlays":overlays,"overlayRelative":self.overlay,"heldBytes":held,
            "bufferOnly":self.overlay.as_ref().map(|o| !files.contains_key(o)),
            "scopes":{"cli":self.scope(files, saved_root, "cli"),
                "python":self.scope(files, saved_root, "python"),
                "node":self.scope(files, saved_root, "node"),
                "editor":self.scope(files, held_root, "editor")}})
    }

    /// The file `cover --lines 1` reads: the held edit, else the first
    /// readable (then any) file at or under the path.
    fn lines(&self, files: &BTreeMap<String, Vec<u8>>) -> Option<String> {
        if let Some(o) = &self.overlay {
            return Some(o.clone());
        }
        let under = |f: &&String| {
            self.relative == "."
                || **f == self.relative
                || f.starts_with(&format!("{}/", self.relative))
        };
        files
            .iter()
            .filter(|(f, b)| readable(f, b))
            .map(|(f, _)| f)
            .find(under)
            .or_else(|| files.keys().find(under))
            .cloned()
    }
}

fn call(facts: &Value, point: &str, answers: Value) -> Value {
    let mut call = json!({"point":point,"authoredFrom":facts["authoredFrom"]});
    for (key, value) in answers.as_object().unwrap() {
        call[key] = value.clone();
    }
    call
}

fn base(scope: &Value, root: &str) -> String {
    let report = scope["reportBase"].as_str().unwrap();
    report
        .strip_prefix(root)
        .unwrap_or("")
        .trim_start_matches('/')
        .to_string()
}

/// §FS-fmt.6.2: a written file pinned as replaced lines over its saved input
/// bytes, each line keeping its own terminator, becomes the file's whole text.
fn patch(fmt: &mut Value, inputs: &BTreeMap<String, Vec<u8>>) -> Result<(), String> {
    for (path, written) in fmt["written"].as_object_mut().into_iter().flatten() {
        let Some(lines) = written.as_object() else {
            continue;
        };
        let saved = inputs
            .get(path)
            .map(|b| String::from_utf8_lossy(b).into_owned())
            .ok_or_else(|| format!("written patch of no input {path}"))?;
        let mut text = String::new();
        for (index, line) in saved.split_inclusive('\n').enumerate() {
            match lines.get(&(index + 1).to_string()) {
                Some(new) => {
                    let ending = &line[line.trim_end_matches(['\r', '\n']).len()..];
                    text.push_str(new.as_str().ok_or("a patched line is text")?);
                    text.push_str(ending);
                }
                None => text.push_str(line),
            }
        }
        *written = json!(text);
    }
    Ok(())
}

/// Expand every applicable variant of a manifest that carries compact facts.
pub fn expand(document: &mut Value) -> Result<(), String> {
    let Some(mut facts) = document.get("facts").cloned() else {
        return Ok(());
    };
    let inputs = input_files(document);
    patch(&mut facts["fmt"], &inputs)?;
    for name in super::agreement_corpus::VARIANTS {
        let mut compact = document["variants"][*name].clone();
        if compact["absent"].is_string() {
            continue;
        }
        if compact.get("fmt").is_some() {
            patch(&mut compact["fmt"], &inputs)?;
        }
        let (relative, overlay) = selection(document, &inputs, name);
        let variant = Variant {
            name,
            relative,
            overlay,
            project: compact
                .get("project")
                .or(facts.get("project"))
                .cloned()
                .unwrap_or(json!({})),
        };
        let mut files = inputs.clone();
        if let Some(o) = &variant.overlay {
            let held = inputs
                .get(o)
                .map_or_else(|| "Agreement buffer control.\n".into(), |b| held(b));
            files.insert(o.clone(), held.into_bytes());
        }
        let texts = files
            .iter()
            .map(|(p, b)| (p.clone(), String::from_utf8_lossy(b).into_owned()))
            .collect();
        let model = Model::new(&facts, texts)?;
        let context = variant.context(&inputs);
        let view = |root: &'static str, scope: &str| View {
            m: &model,
            root,
            selected: if *name == "path-argument" {
                variant.relative.clone()
            } else {
                ".".into()
            },
            base: base(&context["scopes"][scope], root),
            lines: variant.lines(&inputs),
            variant: compact.clone(),
        };
        let (host, editor) = (view("<repo>", "python"), view("<repo>", "editor"));
        let mut expanded = json!({"absent":null,"context":context,
            "queryInventory":inventory(&model, &files)?});
        let coordinates = super::agreement_query_inventory::closure(&expanded["queryInventory"])?;
        expanded["coordinates"] = json!(coordinates);
        let mut calls = serde_json::Map::new();
        for (key, point) in TREE {
            calls.insert(key.to_string(), call(&facts, point, host.answer(key)?));
        }
        for c in &coordinates {
            calls.insert(
                format!("show:{c}"),
                call(
                    &facts,
                    "§REQ-one-answer.sections.map",
                    host.coordinate_answer("show", c)?,
                ),
            );
            calls.insert(
                format!("refs:{c}"),
                call(&facts, "§FS-refs.2", host.coordinate_answer("refs", c)?),
            );
        }
        let snapshot = editor.snapshot();
        let refused = snapshot.get("failure").is_some();
        calls.insert(
            "editor:snapshot".into(),
            call(&facts, "§FS-lsp.1.1", json!({"editor":snapshot})),
        );
        if refused {
            expanded["editorUnavailable"] =
                json!({"reason":"snapshot refuses","point":"§FS-lsp.2.2.1"});
        } else {
            for c in &coordinates {
                calls.insert(
                    format!("editor:show:{c}"),
                    call(
                        &facts,
                        "§FS-lsp.1.2.1",
                        json!({"editor":editor.editor_show(c)}),
                    ),
                );
            }
        }
        expanded["calls"] = json!(calls);
        expanded["writtenChanges"] = host.written();
        document["variants"][*name] = expanded;
    }
    document.as_object_mut().unwrap().remove("facts");
    Ok(())
}
