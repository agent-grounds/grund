//! Every question's answer, per producer, mapped from one Python envelope the
//! compact facts render (§REQ-one-answer.proof.evidence.closed): the command
//! line's streams and exit, and Node's envelope through the projection the
//! bindings define (§FS-distribution.3.2.2.1), with its show metadata and its
//! one refusal, `cover --lines` (§FS-distribution.3.2.1.1).

use super::agreement_render_cli::{fact, text};
use super::agreement_render_tree::{View, ok};
use serde_json::{Value, json};

fn node(python: &Value, operation: &str) -> Result<Value, String> {
    let mut errors = Vec::new();
    let value =
        super::agreement_host_projection::from_python(python.clone(), operation, &mut errors);
    // The refs fold check reads raw hit paths; these are already normalized, and
    // Node's summaries are rendered from the raw file summaries below.
    if errors.is_empty() || operation == "refs" {
        Ok(value)
    } else {
        Err(errors.join("; "))
    }
}

impl View<'_> {
    /// Node's metadata on a show answer: the query it resolved (§FS-distribution.3.3.1).
    fn show_metadata(&self, result: &mut Value) {
        if !result.is_object() {
            return;
        }
        let object: Value = serde_json::from_str(result["json"].as_str().unwrap()).unwrap();
        result["id"] = object["id"].clone();
        result["section"] = object["section"].clone();
        result["kind_title"] = object["kind_title"].clone();
        result["manifest"] = Value::Null;
    }

    /// §FS-refs.2: Node folds hits into per-file summaries, at reported paths.
    fn refs_summaries(&self, result: &mut Value, python: &Value) {
        if let Some(out) = result.get_mut("result").filter(|v| v.is_object()) {
            let summaries: Vec<_> = python["result"]["file_summaries"].as_array().unwrap().iter()
                .map(|s| {
                    let lines: Vec<_> = python["result"]["hits"].as_array().unwrap().iter()
                        .filter(|h| h["path"] == json!(self.reported_site(s["path"].as_str().unwrap())))
                        .map(|h| h["line"].clone())
                        .collect();
                    json!({"project":s["project"],"path":s["path"],"count":s["sites"],"lines":lines})
                })
                .collect();
            out["summaries"] = json!(summaries);
        }
    }

    /// §FS-fmt.3.1: a scan that could not read a file under the path rewrites
    /// nothing and says which file.
    fn fmt_answer(&self, project: bool, changes: &[Value], written: bool) -> Value {
        let errors = self.scan_errors_in(project);
        if let Some(first) = errors.first() {
            let message = format!(
                "nothing was rewritten: {}: {}",
                first["path"].as_str().unwrap(),
                first["message"].as_str().unwrap()
            );
            let lines: String = errors
                .iter()
                .map(|e| {
                    format!(
                        "error: nothing was rewritten: {}: {}\n",
                        e["path"].as_str().unwrap(),
                        e["message"].as_str().unwrap()
                    )
                })
                .collect();
            let python = json!({"failure":{"run_cautions":[],"kind":"filesystem","code":"io",
                "message":message,"path":null,"line":null,"column":null,"sites":[],"authority":[],
                "causes":[],"partial_output":{"changes":[],"refused_writes":[],"scan_errors":errors},
                "details":{}},"result":null,"run_cautions":[]});
            return json!({"python":python,"cli":fact(2, json!([]), text(&lines))});
        }
        let python =
            ok(json!({"run_cautions":[],"changes":changes,"scan_errors":[],"refused_writes":[]}));
        let cli = if written {
            let mut files: Vec<(String, usize)> = Vec::new();
            for c in changes {
                let path = c["path"].as_str().unwrap();
                match files.iter_mut().find(|(f, _)| f == path) {
                    Some((_, n)) => *n += 1,
                    None => files.push((path.into(), 1)),
                }
            }
            let plural = if changes.len() == 1 { "line" } else { "lines" };
            let head = if changes.is_empty() {
                "rewrote 0 lines\n".to_string()
            } else {
                format!("rewrote {} {plural}:{}\n", changes.len(), totals(changes))
            };
            let body: String = files
                .iter()
                .map(|(f, n)| format!("  {f} ({n})\n"))
                .collect();
            fact(0, text(&format!("{head}{body}")), json!([]))
        } else {
            let lines: String = changes
                .iter()
                .map(|c| {
                    format!(
                        "{}:{}: {}\n",
                        c["path"].as_str().unwrap(),
                        c["line"],
                        c["label"].as_str().unwrap()
                    )
                })
                .collect();
            fact(i32::from(!changes.is_empty()), text(&lines), json!([]))
        };
        json!({"python":python,"cli":cli})
    }

    fn changes(&self, key: &str, within: bool) -> Vec<Value> {
        self.fact("fmt")[key]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|c| !within || self.within(c["path"].as_str().unwrap()))
            .map(|c| {
                let mut counts = serde_json::Map::new();
                for (kind, ..) in KINDS {
                    counts.insert(kind.into(), json!(c["counts"][kind].as_u64().unwrap_or(0)));
                }
                json!({"path":self.raw(c["path"].as_str().unwrap()),"line":c["line"],
                    "label":c["label"],"counts":counts,"lock_file":null})
            })
            .collect()
    }

    /// §FS-fmt.6.2: the written bytes, as patches over the saved input.
    pub fn written(&self) -> Value {
        if !self.scan_errors_in(true).is_empty() {
            return json!({});
        }
        let mut patches = serde_json::Map::new();
        for (path, text) in self.fact("fmt")["written"]
            .as_object()
            .into_iter()
            .flatten()
        {
            if self.within(path) {
                patches.insert(
                    path.clone(),
                    json!({"bytes":text.as_str().unwrap().as_bytes()}),
                );
            }
        }
        json!(patches)
    }

    /// §FS-cover.6.2: who owns line 1 of the one file `--lines` reads.
    fn cover_lines(&self) -> Value {
        let Some(file) = self.lines.as_deref() else {
            return json!({"refused":true});
        };
        let pinned = self.fact("coverLines");
        let records = if !pinned.is_null() {
            pinned.clone()
        } else if !self.scanned().iter().any(|f| f == file) {
            json!([])
        } else {
            let owners: Vec<_> = self
                .m
                .decls
                .iter()
                // A stub's heading line is its declaration's (§FS-cover.6.2).
                .filter(|d| {
                    (d.file == file && d.line == 1) || d.via.iter().any(|v| v.0 == file && v.1 == 1)
                })
                .map(|d| {
                    json!({"declaration":d.id,"start":1,"end":1,
                    "sections":[{"section":null,"start":1,"end":1}]})
                })
                .collect();
            // §FS-scan.projects.narrowed: a file inside a member is a member run.
            json!([{"project":null,"path":self.site(file),"start":1,"end":1,"owners":owners}])
        };
        ok(json!({"run_cautions":[],"output_format":"text","records":records,"scan_errors":[]}))
    }

    /// One tree question's answer for every producer.
    pub fn answer(&self, key: &str) -> Result<Value, String> {
        let (python, operation) = match key {
            "check" => (self.check(), "check"),
            "list" => (self.list(false), "list"),
            "list-size" => (self.list(true), "listSizes"),
            "cover" => (self.cover(), "cover"),
            "show-batch" => (self.show_batch(), "showBatch"),
            "cover-lines" => {
                let python = self.cover_lines();
                let records = python["result"]["records"].as_array().unwrap();
                let rows: Vec<_> = records
                    .iter()
                    .map(|r| {
                        let mut row = r.clone();
                        row.as_object_mut().unwrap().remove("project");
                        row
                    })
                    .collect();
                let node = json!({"failure":{"path":null,"line":null,"column":null,"sites":[],
                    "authority":[],"causes":[],"details":{},"partial":null,"operation":"cover",
                    "kind":"input","code":"unknown-option","message":"unknown option lines"},
                    "result":null,"run_cautions":[]});
                return Ok(
                    json!({"cli":fact(0, json!(rows), json!([])),"python":python,"node":node}),
                );
            }
            "fmt" | "fmt-write" => {
                let write = key == "fmt-write";
                let changes = self.changes(if write { "writeChanges" } else { "changes" }, true);
                let mut answer = self.fmt_answer(write, &changes, write);
                answer["node"] = node(&answer["python"], "fmt")?;
                return Ok(answer);
            }
            other => return Err(format!("no rendering for {other}")),
        };
        let mut node = node(&python, operation)?;
        if key == "show-batch" {
            for record in node["result"]["records"]
                .as_array_mut()
                .into_iter()
                .flatten()
            {
                if let Some(result) = record.get_mut("result") {
                    self.show_metadata(result);
                }
            }
        }
        let cli = match key {
            "check" => self.cli_check(),
            "list" | "list-size" => self.cli_list(&python, key == "list-size"),
            "cover" => self.cli_cover(&python),
            _ => self.cli_show_batch(&python),
        };
        Ok(json!({"cli":cli,"python":python,"node":node}))
    }

    /// `show <c> --toc` or `refs <c>`, for every producer.
    pub fn coordinate_answer(&self, surface: &str, coordinate: &str) -> Result<Value, String> {
        if surface == "show" {
            let python = self.show(coordinate);
            let mut node = node(&python, "show")?;
            if let Some(result) = node.get_mut("result") {
                self.show_metadata(result);
            }
            return Ok(
                json!({"cli":self.cli_show(coordinate, &python),"python":python,"node":node}),
            );
        }
        let python = self.refs(coordinate);
        let mut node = node(&python, "refs")?;
        self.refs_summaries(&mut node, &python);
        Ok(json!({"cli":self.cli_refs(coordinate, &python),"python":python,"node":node}))
    }
}

/// §FS-fmt.3.7: each kind's count, in the report's order, with its singular and plural.
const KINDS: [(&str, &str, &str); 6] = [
    ("triggers_marked", "trigger marked", "triggers marked"),
    ("citations_marked", "citation marked", "citations marked"),
    (
        "local_sections_expanded",
        "local section expanded",
        "local sections expanded",
    ),
    (
        "shorthands_expanded",
        "shorthand expanded",
        "shorthands expanded",
    ),
    ("links_written", "link written", "links written"),
    (
        "section_lock_entries",
        "section-lock entry",
        "section-lock entries",
    ),
];

/// §FS-fmt.3.7: the write summary's totals, a zero left out, or nothing at all.
fn totals(changes: &[Value]) -> String {
    let named: Vec<String> = KINDS
        .iter()
        .filter_map(|(kind, one, many)| {
            let n: u64 = changes
                .iter()
                .map(|c| c["counts"][kind].as_u64().unwrap())
                .sum();
            (n > 0).then(|| format!("{n} {}", if n == 1 { one } else { many }))
        })
        .collect();
    if named.is_empty() {
        String::new()
    } else {
        format!(" {}", named.join(", "))
    }
}
