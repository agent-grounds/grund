//! The editor snapshot rendered from the compact model: the located findings
//! the server publishes (§FS-lsp.1.1), the declarations, sections and
//! citations it navigates (§FS-lsp.1.3.1), and every file its run read
//! (§FS-lsp.responsiveness.3). Site-less CLI-level config cautions are
//! `check`'s alone (§FS-check.checks.config-version-1).

use super::agreement_render_tree::View;
use serde_json::{Value, json};

impl View<'_> {
    fn absolute(&self, file: &str) -> String {
        format!("<repo>/{file}")
    }

    fn located(&self, file: &str, line: usize, column: usize, text: &str, query: &str) -> Value {
        json!({"column":column,"display_path":file,"line":line,"path":self.absolute(file),
            "project":null,"query_id":query,"section_separator":".","text":text})
    }

    fn target(&self, id: &str, section: &Value) -> Option<(String, usize)> {
        let coordinate = match section.as_str() {
            Some(s) => format!("{id}.{s}"),
            None => id.to_string(),
        };
        let (d, s) = self.resolve(&coordinate)?;
        let decl = &self.m.decls[d];
        Some(match s {
            Some(s) => (decl.file.clone(), self.m.sections[s].line),
            None => (decl.file.clone(), decl.line),
        })
    }

    fn editor_finding(&self, f: &Value) -> Value {
        json!({"authority":f.get("authority").cloned().unwrap_or(json!([])),"code":f["code"],
            "column":null,"line":f["line"],"message":f["message"],
            "path":self.absolute(f["path"].as_str().unwrap()),"severity":f["severity"],
            "sites":f["sites"].as_array().into_iter().flatten().map(|s|
                json!({"path":s["path"],"line":s["line"]})).collect::<Vec<_>>()})
    }

    pub fn snapshot(&self) -> Value {
        if let Some(failure) = self.m.facts["editor"].get("failure") {
            return json!({"failure":failure});
        }
        let mut report = json!({"errors":[],"warnings":[],"suggestions":[]});
        for f in self.m.facts["findings"].as_array().into_iter().flatten() {
            if f["path"].is_null() {
                continue;
            }
            let key = if f["severity"] == "error" {
                "errors"
            } else {
                "warnings"
            };
            report[key]
                .as_array_mut()
                .unwrap()
                .push(self.editor_finding(f));
        }
        let mut declarations: Vec<_> = self.m.decls.iter().collect();
        declarations.sort_by(|a, b| (&a.file, a.line).cmp(&(&b.file, b.line)));
        let mut sections: Vec<_> = self.m.sections.iter().collect();
        sections.sort_by(|a, b| (&a.file, a.line).cmp(&(&b.file, b.line)));
        let mut cites: Vec<_> = self.m.cites.iter().collect();
        cites.sort_by(|a, b| (&a.file, a.line, a.column).cmp(&(&b.file, b.line, b.column)));
        let mut kind_titles = serde_json::Map::new();
        for row in self.order() {
            let coordinate = self.coordinate(row);
            kind_titles.insert(coordinate.clone(), self.kind_title(&coordinate));
        }
        let scanned: Vec<_> = self.scanned().iter().map(|f| self.absolute(f)).collect();
        // §FS-lsp.responsiveness.3: the run also reads the agent entrypoint and
        // each folder kind's index, present or not.
        let probes: Vec<String> = match self.m.facts["editor"]["probes"].as_array() {
            Some(probes) => probes
                .iter()
                .map(|p| p.as_str().unwrap().to_string())
                .collect(),
            None => std::iter::once("AGENTS.md".to_string())
                .chain(self.kinds().iter().filter_map(|k| {
                    let home = k["home"].as_str().unwrap();
                    (!home.ends_with(".md")).then(|| format!("{home}/README.md"))
                }))
                .collect(),
        };
        let mut read: Vec<String> = self
            .scanned()
            .into_iter()
            .chain(probes)
            .map(|f| self.absolute(&f))
            .collect();
        read.sort();
        read.dedup();
        let ranges: Vec<_> = self.m.facts["editor"]["findingRanges"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|r| {
                let file = r["file"].as_str().unwrap();
                let token = r["token"].as_str().unwrap();
                let nth = r["nth"].as_u64().unwrap_or(1) as usize;
                let (line, column, _) = super::agreement_render_model::locate(
                    self.m.text(file).unwrap(),
                    token,
                    nth,
                    1,
                )
                .unwrap();
                json!({"code":r["code"],"path":self.absolute(file),"line":line,"column":column,
                    "text":token})
            })
            .collect();
        json!({"kind_titles":kind_titles,"readFiles":read,"snapshot":{
            "citations":cites.iter().map(|c| {
                let target = self.target(&c.id, &c.section);
                let query = match c.section.as_str() {
                    Some(s) => format!("{}.{s}", c.id),
                    None => c.id.clone(),
                };
                let mut row = self.located(&c.file, c.line, c.column, &c.text, &query);
                row["declaration_query_id"] = json!(c.id);
                row["target_path"] = json!(target.as_ref().map(|(f, _)| self.absolute(f)));
                row["target_line"] = json!(target.map(|(_, l)| l));
                row
            }).collect::<Vec<_>>(),
            "declarations":declarations.iter().map(|d| {
                let mut row = self.located(&d.file, d.line, d.column, &d.text, &d.id);
                row["hover_text"] = Value::Null;
                row
            }).collect::<Vec<_>>(),
            "finding_ranges":ranges,"marker":self.marker(),"report":report,
            "root":"<repo>","run_warnings":[],"scan_errors":self.scan_errors(),
            "scanned_files":scanned,
            "sections":sections.iter().map(|s| {
                let query = format!("{}.{}", self.m.decls[s.owner].id, s.path);
                let mut row = self.located(&s.file, s.line, s.column, &s.text, &query);
                row["hover_text"] = Value::Null;
                row
            }).collect::<Vec<_>>(),
            "stubs":self.m.facts["editor"]["stubs"].as_array().cloned().unwrap_or_default(),
            "trigger":self.m.facts["trigger"].as_str().unwrap_or("$$"),"workspace":false}})
    }
}
