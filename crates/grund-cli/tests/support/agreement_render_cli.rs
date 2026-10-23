//! Each question's command-line answer mapped from the same compact facts as
//! its host envelopes: JSON rows on stdout, a refusal as one error row on
//! stderr, CLI-level config cautions on stderr (§FS-check.2.1.1), and the exit
//! each command defines (§FS-check.2, §FS-fmt.1, §FS-refs.4).

use super::agreement_render_show::Row;
use super::agreement_render_tree::View;
use serde_json::{Value, json};

pub fn fact(exit: i32, stdout: Value, stderr: Value) -> Value {
    json!({"exit":exit,"stdout":stdout,"stderr":stderr})
}

/// A stream the comparison reads as JSON rows when every line parses.
pub fn text(lines: &str) -> Value {
    if lines.is_empty() {
        json!([])
    } else {
        json!(lines)
    }
}

impl View<'_> {
    fn sites_row(&self, sites: &Value) -> Value {
        match sites.as_array() {
            Some(s) if !s.is_empty() => self.sites(sites),
            _ => Value::Null,
        }
    }

    /// A refusal's row from Python's failure, whose sites are already reported.
    pub fn error_row(&self, failure: &Value) -> Value {
        let sites = match failure["sites"].as_array() {
            Some(s) if !s.is_empty() => failure["sites"].clone(),
            _ => Value::Null,
        };
        json!({"severity":"error","path":null,"line":null,"code":failure["code"],
            "message":failure["message"],"sites":sites,"authority":null})
    }

    fn finding_row(&self, f: &Value) -> Value {
        json!({"severity":f["severity"],"path":f["path"].as_str().map(|p| self.site(p)),
            "line":f["line"],"code":f["code"],"message":f["message"],
            "sites":self.sites_row(&f["sites"]),"authority":null})
    }

    /// §FS-check.2.1.1: a finding with no line is a CLI-level message on
    /// stderr, the site-less ones first; §FS-check.2.4: an unread file under
    /// the path exits 2.
    pub fn cli_check(&self) -> Value {
        let findings = self.findings();
        let mut out: Vec<_> = findings
            .iter()
            .filter(|f| !f["line"].is_null())
            .map(|f| self.finding_row(f))
            .collect();
        // §FS-errors.4.1: JSON stdout sorts across channels using rendered paths;
        // equal (path, line, message) keys keep their authored order.
        out.sort_by(|a, b| {
            a["path"]
                .as_str()
                .cmp(&b["path"].as_str())
                .then_with(|| {
                    a["line"]
                        .as_u64()
                        .unwrap_or(0)
                        .cmp(&b["line"].as_u64().unwrap_or(0))
                })
                .then_with(|| {
                    a["message"]
                        .as_str()
                        .unwrap_or("")
                        .cmp(b["message"].as_str().unwrap_or(""))
                })
        });
        let err = findings.iter().filter(|f| f["path"].is_null()).chain(
            findings
                .iter()
                .filter(|f| f["line"].is_null() && !f["path"].is_null()),
        );
        let failed = findings.iter().any(|f| f["severity"] == "error");
        let exit = if !self.scan_errors_in(false).is_empty() {
            2
        } else {
            i32::from(failed)
        };
        fact(
            exit,
            json!(out),
            json!(err.map(|f| self.finding_row(f)).collect::<Vec<_>>()),
        )
    }

    /// §FS-scan.path: an incomplete walk prints what it read, one `error:`
    /// line per unread file, and exits 2.
    pub fn incomplete(&self, project: bool, rows: Value, prefix: &str) -> Value {
        let errors = self.scan_errors_in(project);
        let lines: String = errors
            .iter()
            .map(|e| {
                format!(
                    "error: {}: {}\n",
                    e["path"].as_str().unwrap(),
                    e["message"].as_str().unwrap()
                )
            })
            .collect();
        let exit = if errors.is_empty() { 0 } else { 2 };
        fact(exit, rows, text(&format!("{prefix}{lines}")))
    }

    pub fn cli_list(&self, python: &Value, size: bool) -> Value {
        let rows: Vec<_> = python["result"]["entries"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| {
                let mut row = json!({"id":e["id"],"kind":e["kind"],"path":e["path"],
                    "line":e["line"],"stub":e["stub"],"defines":e["defines"],
                    "duplicate":e["duplicate"]});
                if size {
                    row["section"] = e["section"].clone();
                    for m in e["measurements"].as_array().unwrap() {
                        let unit = m["unit"].as_str().unwrap();
                        row[format!("lead_{unit}")] = m["lead"].clone();
                        row[format!("full_{unit}")] = m["full"].clone();
                    }
                } else {
                    row["title"] = e["title"].clone();
                    row["refs"] = e["refs"].clone();
                    // §FS-list.3.2.2: omitted, not empty, without value roots.
                    if e["value_roots"].as_array().is_some_and(|r| !r.is_empty()) {
                        row["value_roots"] = e["value_roots"].clone();
                    }
                }
                if !e["project"].is_null() {
                    row["project"] = e["project"].clone();
                }
                row
            })
            .collect();
        self.incomplete(true, json!(rows), "")
    }

    /// §FS-cover.3.2, §FS-workspace.8.2.4: `project` leads a row in workspace
    /// mode and is absent outside it.
    fn citation_row(c: &Value) -> Value {
        let mut row = c.clone();
        if row["project"].is_null() {
            row.as_object_mut().unwrap().remove("project");
        }
        row
    }

    pub fn cli_cover(&self, python: &Value) -> Value {
        let rows: Vec<_> = python["result"]["entries"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| {
                Self::citation_row(&json!({"project":e["project"],"path":e["path"],
                    "citations":e["citations"].as_array().unwrap()
                .iter().map(Self::citation_row).collect::<Vec<_>>()}))
            })
            .collect();
        self.incomplete(false, json!(rows), "")
    }

    pub fn show_row(&self, row: Row) -> Value {
        let mut object = self.show_object(row);
        object["path"] = json!(self.site(object["path"].as_str().unwrap()));
        object
    }

    /// A refusal the command line words for itself, such as an alias outside a
    /// workspace (§FS-workspace.8.1.3): its own exit and stderr lines.
    fn cli_refusal(&self, coordinate: &str, question: &str) -> Option<Value> {
        let cli = self.refusal(coordinate, question)?.get("cli")?.clone();
        let stderr = text(cli["stderr"].as_str().unwrap_or(""));
        Some(fact(
            cli["exit"].as_i64().unwrap() as i32,
            json!([]),
            stderr,
        ))
    }

    pub fn cli_show(&self, coordinate: &str, python: &Value) -> Value {
        if self.unread(false).is_some() {
            return self.incomplete(true, json!([]), "");
        }
        if let Some(cli) = self.cli_refusal(coordinate, "show") {
            return cli;
        }
        match self.resolve(coordinate) {
            Some(row) if python["failure"].is_null() => {
                fact(0, json!([self.show_row(row)]), json!([]))
            }
            _ => fact(1, json!([]), json!([self.error_row(&python["failure"])])),
        }
    }

    pub fn cli_show_batch(&self, python: &Value) -> Value {
        if self.unread(true).is_some() {
            return self.incomplete(true, json!([]), "");
        }
        let mut failed = false;
        let rows: Vec<_> = python["result"]["records"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| {
                let query = &r["query"];
                let coordinate = match query["section"].as_str() {
                    Some(s) => format!("{}.{s}", query["id"].as_str().unwrap()),
                    None => query["id"].as_str().unwrap().to_string(),
                };
                if r["failure"].is_null() {
                    let row = self.resolve(&coordinate).unwrap();
                    json!({"query":query,"ok":true,"result":self.show_row(row),"error":null})
                } else {
                    failed = true;
                    json!({"query":query,"ok":false,"result":null,
                        "error":self.error_row(&r["failure"])})
                }
            })
            .collect();
        fact(i32::from(failed), json!(rows), json!([]))
    }

    pub fn cli_refs(&self, coordinate: &str, python: &Value) -> Value {
        if let Some(cli) = self.cli_refusal(coordinate, "refs") {
            return cli;
        }
        if !python["failure"].is_null() {
            return fact(1, json!([]), json!([self.error_row(&python["failure"])]));
        }
        let result = &python["result"];
        let rows: Vec<_> = result["hits"]
            .as_array()
            .unwrap()
            .iter()
            .map(|h| {
                let mut row = Self::citation_row(h);
                // §FS-config.3.4.3: an absent effective title omits the field.
                if !result["kind_title"].is_null() {
                    row["kind_title"] = result["kind_title"].clone();
                }
                row
            })
            .collect();
        let note = result["note"]
            .as_str()
            .map_or(String::new(), |n| format!("note: {n}\n"));
        self.incomplete(false, json!(rows), &note)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agreement_render_model::Model;

    /// §FS-errors.4.1: the CLI projection orders rendered rows independently of
    /// the authored report channels, retaining ties without a severity/code key.
    /// The `./` spelling makes sorting unrendered fact paths observably wrong.
    /// §FS-check.2.1.1: pathless, lineless cautions stay on stderr in their order.
    #[test]
    fn cli_check_orders_rendered_stdout_stably_across_channels() {
        let facts = json!({"findings":[
            {"severity":"error","path":"docs/a.md","line":10,
                "code":"late","message":"Ten","sites":[]},
            {"severity":"error","path":"docs/b.md","line":1,
                "code":"missing-index-entry","message":"Beta","sites":[
                    {"path":"docs/b.md","line":1},{"path":"docs/c.md","line":3}]},
            {"severity":"warning","path":"docs/a.md","line":2,
                "code":"z-warning","message":"Shared","sites":[]},
            {"severity":"error","path":"docs/a.md","line":2,
                "code":"a-error","message":"Shared","sites":[]},
            {"severity":"warning","path":"./docs/a.md","line":1,
                "code":"early","message":"é","sites":[]},
            {"severity":"warning","path":"docs/a.md","line":1,
                "code":"bytewise","message":"Z","sites":[]},
            {"severity":"warning","path":null,"line":null,
                "code":"empty-scan","message":"nothing to scan","sites":[]},
            {"severity":"warning","path":null,"line":null,
                "code":"config-version-1","message":"grund.toml is version 1","sites":[]}
        ]});
        let model = Model::new(&facts, Default::default()).unwrap();
        let view = View {
            m: &model,
            root: "<repo>",
            selected: ".".into(),
            base: ".".into(),
            lines: None,
            explicit_path: None,
            variant: json!({}),
        };

        assert_eq!(
            view.cli_check(),
            json!({"exit":1,"stdout":[
                {"severity":"warning","path":"<repo>/docs/a.md","line":1,
                    "code":"bytewise","message":"Z","sites":null,"authority":null},
                {"severity":"warning","path":"<repo>/docs/a.md","line":1,
                    "code":"early","message":"é","sites":null,"authority":null},
                {"severity":"warning","path":"<repo>/docs/a.md","line":2,
                    "code":"z-warning","message":"Shared","sites":null,"authority":null},
                {"severity":"error","path":"<repo>/docs/a.md","line":2,
                    "code":"a-error","message":"Shared","sites":null,"authority":null},
                {"severity":"error","path":"<repo>/docs/a.md","line":10,
                    "code":"late","message":"Ten","sites":null,"authority":null},
                {"severity":"error","path":"<repo>/docs/b.md","line":1,
                    "code":"missing-index-entry","message":"Beta","sites":[
                        {"path":"<repo>/docs/b.md","line":1},
                        {"path":"<repo>/docs/c.md","line":3}],"authority":null}
            ],"stderr":[
                {"severity":"warning","path":null,"line":null,
                    "code":"empty-scan","message":"nothing to scan","sites":null,
                    "authority":null},
                {"severity":"warning","path":null,"line":null,
                    "code":"config-version-1","message":"grund.toml is version 1",
                    "sites":null,"authority":null}
            ]})
        );
        assert_eq!(view.findings(), *facts["findings"].as_array().unwrap());
    }
}
