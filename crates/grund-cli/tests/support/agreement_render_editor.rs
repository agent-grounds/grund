//! The editor snapshot rendered from the compact model: the located findings
//! the server publishes (§FS-lsp.1.1), the declarations, sections and
//! citations it navigates (§FS-lsp.1.3.1), and every file its run read
//! (§FS-lsp.responsiveness.3). Site-less CLI-level config cautions are
//! `check`'s alone (§FS-check.checks.config-version-1).

use super::agreement_render_model::Decl;
use super::agreement_render_tree::View;
use serde_json::{Value, json};

impl View<'_> {
    fn absolute(&self, file: &str) -> String {
        format!("<repo>/{file}")
    }

    /// §FS-lsp.1.1: the editor routes a site by its file's filesystem identity,
    /// a link's target, while the files it read keep the spelling it read them by.
    fn identity(&self, file: &str) -> String {
        self.absolute(self.m.facts["links"][file].as_str().unwrap_or(file))
    }

    fn located(&self, file: &str, line: usize, column: usize, text: &str, query: &str) -> Value {
        json!({"column":column,"display_path":self.raw(file),"line":line,"path":self.identity(file),
            "project":self.project_of(file),"query_id":query,"section_separator":".","text":text})
    }

    fn target(&self, id: &str, section: &Value) -> Option<(String, usize)> {
        let coordinate = match section.as_str() {
            Some(s) => format!("{id}.{s}"),
            None => id.to_string(),
        };
        let id = coordinate.split('.').next().unwrap();
        // §FS-lsp.1.3: a citation jumps to its declaration's line, which a
        // section its declaration lacks still has.
        let (d, s) = self.resolve(&coordinate).or_else(|| self.resolve(id))?;
        let decl = &self.m.decls[d];
        Some(match s {
            Some(s) => (decl.file.clone(), self.m.sections[s].line),
            None => (decl.file.clone(), decl.line),
        })
    }

    fn editor_finding(&self, f: &Value) -> Value {
        json!({"authority":f.get("authority").cloned().unwrap_or(json!([])),"code":f["code"],
            "column":self.column(f),"line":f["line"],"message":f["message"],
            "path":self.identity(f["path"].as_str().unwrap()),"severity":f["severity"],
            "sites":f["sites"].as_array().into_iter().flatten().map(|s|
                json!({"path":self.identity(s["path"].as_str().unwrap()),"line":s["line"]}))
                .collect::<Vec<_>>()})
    }

    pub fn snapshot(&self) -> Value {
        if let Some(failure) = self.m.facts["editor"].get("failure") {
            return json!({"failure":failure});
        }
        let mut report = json!({"errors":[],"warnings":[],"suggestions":[]});
        // §FS-scan.path.check: an explicit path narrows the published report too.
        for f in &self.findings() {
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
        // A stub stands in as the title of a target the walk did not read.
        let title = |d: &Decl| match d.via.first() {
            Some(v) if d.facts["standIn"] == true => (v.0.clone(), v.1, v.2, v.3.clone()),
            _ => (d.file.clone(), d.line, d.column, d.text.clone()),
        };
        let mut declarations: Vec<_> = (0..self.m.decls.len())
            .map(|d| (d, title(&self.m.decls[d])))
            .filter(|(_, (file, ..))| self.loaded(file))
            .collect();
        declarations.sort_by(|a, b| (&a.1.0, a.1.1).cmp(&(&b.1.0, b.1.1)));
        // §FS-declarations.checks.duplicate-section.3: a later claimant of a
        // path is no section target, so the editor navigates to the first only.
        let mut sections: Vec<_> = self
            .m
            .sections
            .iter()
            .enumerate()
            .filter(|(i, s)| {
                !self.m.sections[..*i]
                    .iter()
                    .any(|o| o.owner == s.owner && o.path == s.path)
            })
            .map(|(_, s)| s)
            .filter(|s| self.loaded(&s.file))
            .collect();
        sections.sort_by(|a, b| (&a.file, a.line).cmp(&(&b.file, b.line)));
        let cites: Vec<_> = self
            .m
            .cites
            .iter()
            .filter(|c| self.loaded(&c.file))
            .collect();
        let mut kind_titles = serde_json::Map::new();
        for row in self.order() {
            let coordinate = self.coordinate(row);
            let title = self.kind_title(&coordinate);
            if !title.is_null() {
                kind_titles.insert(coordinate.clone(), title);
            }
        }
        let scanned: Vec<_> = self.scanned().iter().map(|f| self.identity(f)).collect();
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
                    let indexed = k["index"] != false && !home.ends_with(".md");
                    indexed.then(|| format!("{home}/README.md"))
                }))
                .collect(),
        };
        // A stub's target is read where its verdict reads it (§FS-declarations.stubs.target).
        let reads = self.m.facts["editor"]["reads"]
            .as_array()
            .into_iter()
            .flatten();
        let mut read: Vec<String> = self
            .scanned()
            .into_iter()
            .chain(probes)
            .chain(reads.map(|r| r.as_str().unwrap().to_string()))
            .filter(|f| self.loaded(f))
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
                json!({"code":r["code"],"path":self.identity(file),"line":line,"column":column,
                    "text":token})
            })
            .collect();
        json!({"kind_titles":kind_titles,"readFiles":read,"snapshot":{
            "citations":cites.iter().map(|c| {
                // §FS-values.7: a citation navigates by its written alias path.
                let id = self.citation(c, true)["id"].as_str().unwrap().to_string();
                let target = self.target(&id, &c.section);
                let query = match c.section.as_str() {
                    Some(s) => format!("{id}.{s}"),
                    None => id.clone(),
                };
                let mut row = self.located(&c.file, c.line, c.column, &c.text, &query);
                row["declaration_query_id"] = json!(id);
                row["target_path"] = json!(target.as_ref().map(|(f, _)| self.identity(f)));
                row["target_line"] = json!(target.map(|(_, l)| l));
                row
            }).collect::<Vec<_>>(),
            "declarations":declarations.iter().map(|(d, (file, line, column, text))| {
                let mut row = self.located(file, *line, *column, text, &self.qualified(*d));
                row["hover_text"] = Value::Null;
                row
            }).collect::<Vec<_>>(),
            "finding_ranges":ranges,"marker":self.marker(),"report":report,
            "root":self.member().map_or("<repo>".into(), |m| self.absolute(m)),"run_warnings":[],"scan_errors":self.scan_errors(),
            "scanned_files":scanned,
            "sections":sections.iter().map(|s| {
                let query = format!("{}.{}", self.qualified(s.owner), s.path);
                // §FS-values.2.4.2: a marked root's LSP title excludes its tag.
                let text = match s.facts["valueRoot"].as_str() {
                    Some("marked") => s.text.strip_suffix(" <!-- grund:value -->").unwrap_or(&s.text),
                    _ => &s.text,
                };
                // §FS-fields.front-matter.surfaces: with no heading to span, a
                // front-matter coordinate's title is its path at column 1.
                let (column, text) = if s.front_matter { (1, s.path.as_str()) } else { (s.column, text) };
                let mut row = self.located(&s.file, s.line, column, text, &query);
                row["hover_text"] = Value::Null;
                row
            }).collect::<Vec<_>>(),
            "stubs":self.m.facts["editor"]["stubs"].as_array().cloned().unwrap_or_default(),
            "trigger":self.m.facts["trigger"].as_str().unwrap_or("$$"),"workspace":self.workspace()}})
    }
}
