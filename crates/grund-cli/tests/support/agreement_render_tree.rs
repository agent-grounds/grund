//! Tree questions rendered from the compact model into Python's envelope, the
//! schema every other producer's answer is mapped from
//! (§REQ-one-answer.proof.evidence.closed). A path argument narrows the report
//! and the listings, never what resolves (§FS-scan.path).

use super::agreement_render_model::{Model, measure};
use serde_json::{Value, json};
use std::cmp::Ordering;

/// One variant's reading of the model for one root role (`<repo>`, or
/// `<saved>` and `<held>` under an overlay): the selected path ("." for the
/// whole project), the report base and the `cover --lines` file, all relative
/// to the copied repository, and the variant's own pinned facts.
pub struct View<'a> {
    pub m: &'a Model,
    pub root: &'static str,
    pub selected: String,
    pub base: String,
    pub lines: Option<String>,
    pub variant: Value,
}

/// §FS-config.3.4.4: the kinds a config that declares none gets, in order.
const DEFAULT_KINDS: &[(&str, &str, &str)] = &[
    ("GRUND", "Why: project motivation", "docs/grund.md"),
    (
        "GOAL",
        "Where: project direction and outcomes",
        "docs/goals.md",
    ),
    (
        "FS",
        "What: behavior, requirements, and constraints",
        "docs/functional-spec",
    ),
    (
        "AR",
        "How: high-level implementation, structure, and design",
        "docs/architecture",
    ),
    (
        "DF",
        "Product behavior decisions and tradeoffs",
        "docs/decisions/functional",
    ),
    (
        "DA",
        "Architecture decisions and tradeoffs",
        "docs/decisions/architectural",
    ),
    ("RM", "Planned milestones and sequencing", "docs/roadmap.md"),
];

pub fn ok(result: Value) -> Value {
    json!({"failure":null,"result":result,"run_cautions":[]})
}

fn natural(a: &str, b: &str) -> Ordering {
    let parts = |s: &str| -> Vec<(u8, u64, String)> {
        s.split(['.', '-'])
            .map(|p| match p.parse::<u64>() {
                Ok(n) => (0, n, String::new()),
                Err(_) => (1, 0, p.to_string()),
            })
            .collect()
    };
    parts(a).cmp(&parts(b))
}

impl View<'_> {
    pub fn within(&self, file: &str) -> bool {
        self.selected == "."
            || file == self.selected
            || file.starts_with(&format!("{}/", self.selected))
    }

    /// §FS-scan.path.list: a declaration is at or under the path where it, or a
    /// stub that pairs with it, is.
    pub fn declared_within(&self, d: usize) -> bool {
        let decl = &self.m.decls[d];
        self.within(&decl.file) || decl.via.iter().any(|(f, ..)| self.within(f))
    }

    /// A fact a variant pins for itself, else the one the whole input pins.
    pub fn fact(&self, name: &str) -> &Value {
        self.variant.get(name).unwrap_or(&self.m.facts[name])
    }

    /// A report site as the comparison normalizes it: the root role joined
    /// with the file's path below the copied repository.
    pub fn site(&self, path: &str) -> String {
        let mut parts: Vec<&str> = Vec::new();
        for part in path.split('/') {
            match part {
                "" | "." => {}
                ".." => {
                    parts.pop();
                }
                other => parts.push(other),
            }
        }
        std::iter::once(self.root)
            .chain(parts)
            .collect::<Vec<_>>()
            .join("/")
    }

    /// The configured citable kinds, in `[[kinds]]` order.
    pub fn kinds(&self) -> Vec<Value> {
        match self.m.facts["kinds"].as_array() {
            Some(kinds) => kinds.clone(),
            None => DEFAULT_KINDS
                .iter()
                .map(|(kind, title, home)| json!({"kind":kind,"title":title,"home":home}))
                .collect(),
        }
    }

    pub fn kind_title(&self, id: &str) -> Value {
        let kind = id.rsplit('/').next().unwrap().split('-').next().unwrap();
        self.kinds()
            .into_iter()
            .find(|k| k["kind"] == kind)
            .map_or(Value::Null, |k| k["title"].clone())
    }

    /// Declarations by ID order (§FS-list.2.4), each followed by its sections.
    pub fn order(&self) -> Vec<(usize, Option<usize>)> {
        let mut decls: Vec<usize> = (0..self.m.decls.len()).collect();
        decls.sort_by(|&a, &b| {
            natural(&self.m.decls[a].id, &self.m.decls[b].id)
                .then(self.m.decls[a].file.cmp(&self.m.decls[b].file))
                .then(self.m.decls[a].line.cmp(&self.m.decls[b].line))
        });
        let mut rows = Vec::new();
        for d in decls {
            rows.push((d, None));
            let mut sections: Vec<usize> = (0..self.m.sections.len())
                .filter(|&s| self.m.sections[s].owner == d)
                .collect();
            sections.sort_by(|&a, &b| {
                natural(&self.m.sections[a].path, &self.m.sections[b].path)
                    .then(self.m.sections[a].line.cmp(&self.m.sections[b].line))
            });
            rows.extend(sections.into_iter().map(|s| (d, Some(s))));
        }
        rows
    }

    pub fn finding(&self, f: &Value) -> Value {
        json!({"severity":f["severity"],"channel":null,"code":f["code"],
            "path":f["path"].as_str().map(|p| self.site(p)),"line":f["line"],
            "column":null,"message":f["message"],"sites":self.sites(&f["sites"]),
            "authority":f.get("authority").cloned().unwrap_or(json!([]))})
    }

    pub fn sites(&self, sites: &Value) -> Value {
        json!(
            sites
                .as_array()
                .into_iter()
                .flatten()
                .map(|s| json!({"path":self.site(s["path"].as_str().unwrap()),"line":s["line"]}))
                .collect::<Vec<_>>()
        )
    }

    /// §FS-check.1.3.6.2: a finding is reported when its anchor or a site it
    /// names lies at or under the path; §FS-check.1.3.6.3: a file outside it
    /// that the walk could not read is a caution, never an error.
    pub fn findings(&self) -> Vec<Value> {
        let mut out = Vec::new();
        for f in self.fact("findings").as_array().into_iter().flatten() {
            let inside = f["path"].as_str().is_none_or(|p| self.within(p))
                || f["sites"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .any(|s| self.within(s["path"].as_str().unwrap()));
            if inside {
                out.push(f.clone());
            } else if f["code"] == "io" {
                let mut caution = f.clone();
                caution["severity"] = json!("warning");
                caution["message"] = json!(format!(
                    "{} \u{2014} outside the report scope, so a \
                    citation declared there may be reported as unresolved",
                    f["message"].as_str().unwrap()
                ));
                out.push(caution);
            }
        }
        out
    }

    fn report(&self) -> Value {
        let mut report = json!({"errors":[],"warnings":[],"suggestions":[]});
        for f in self.findings() {
            let key = match f["severity"].as_str().unwrap() {
                "error" => "errors",
                "warning" => "warnings",
                _ => "suggestions",
            };
            report[key].as_array_mut().unwrap().push(self.finding(&f));
        }
        report
    }

    /// The project's unread files (`project`), or only those at or under the
    /// path for a question whose path bounds its scan (§FS-scan.path.cover,
    /// §FS-scan.path.refs). They keep their reported relative paths: the
    /// comparison normalizes only report rows (`agreement_sites`).
    pub fn scan_errors_in(&self, project: bool) -> Vec<Value> {
        self.fact("scanErrors")
            .as_array()
            .into_iter()
            .flatten()
            .filter(|e| project || self.within(e["path"].as_str().unwrap()))
            .map(|e| json!({"path":self.raw(e["path"].as_str().unwrap()),"message":e["message"]}))
            .collect()
    }

    pub fn scan_errors(&self) -> Value {
        json!(self.scan_errors_in(true))
    }

    /// A path relative to the report base, as a result's unnormalized field holds it.
    pub fn raw(&self, path: &str) -> String {
        if self.base.is_empty() || self.base == "." {
            return path.into();
        }
        let base: Vec<_> = self.base.split('/').collect();
        let file: Vec<_> = path.split('/').collect();
        let common = base.iter().zip(&file).take_while(|(a, b)| a == b).count();
        let ups = std::iter::repeat_n("..", base.len() - common);
        ups.chain(file[common..].iter().copied())
            .collect::<Vec<_>>()
            .join("/")
    }

    /// §FS-check.2.4: only a file at or under the path makes the run incomplete.
    pub fn check(&self) -> Value {
        let report = self.report();
        ok(
            json!({"run_cautions":[],"report":report,"selected_report":report,
            "had_scan_errors":!self.scan_errors_in(false).is_empty(),
            "output_format":"text"}),
        )
    }

    fn ref_count(&self, id: &str) -> usize {
        self.m.cites.iter().filter(|c| c.id == id).count()
    }

    fn entry(&self, d: usize, s: Option<usize>, size: bool) -> Value {
        let decl = &self.m.decls[d];
        let stub = decl.facts.get("stub").filter(|v| !v.is_null());
        let mut row = json!({"project":decl.project,"id":decl.id,"section":null,
            "section_separator":".","kind":decl.kind,"path":self.site(&decl.file),
            "line":decl.line,"stub":stub.is_some(),"defines":stub,
            "duplicate":decl.facts["duplicate"].as_bool().unwrap_or(false)});
        let (lead, full) = match s {
            Some(s) => {
                let section = &self.m.sections[s];
                row["section"] = json!(section.path);
                row["line"] = json!(section.line);
                row["duplicate"] = json!(section.facts["duplicate"].as_bool().unwrap_or(false));
                (Some(section.lead.clone()), Some(section.full.clone()))
            }
            None => (decl.lead.clone(), decl.full.clone()),
        };
        if size {
            let (lead, full) = (lead.as_deref().map(measure), full.as_deref().map(measure));
            row["measurements"] = json!(["lines", "words", "bytes"].iter().enumerate().map(|(i, unit)|
                json!({"unit":unit,"lead":lead.map(|m| m[i]),"full":full.map(|m| m[i])})).collect::<Vec<_>>());
        } else {
            row["title"] = decl.facts["title"].clone();
            row["refs"] = json!(self.ref_count(&decl.id));
            row["value_roots"] = json!([]);
        }
        row
    }

    pub fn list(&self, size: bool) -> Value {
        let rows: Vec<_> = self
            .order()
            .into_iter()
            .filter(|(d, s)| self.declared_within(*d) && (size || s.is_none()))
            .collect();
        let mut result = json!({"run_cautions":[],"output_format":"text","workspace":false,
            "entries":rows.iter().map(|(d, s)| self.entry(*d, *s, size)).collect::<Vec<_>>(),
            "scan_errors":self.scan_errors()});
        if !size {
            result["summaries"] = json!(
                self.kinds()
                    .iter()
                    .filter_map(|k| {
                        let count = rows
                            .iter()
                            .filter(|(d, s)| {
                                s.is_none() && self.m.decls[*d].kind == k["kind"].as_str().unwrap()
                            })
                            .count();
                        (count > 0).then(|| {
                            // An untitled kind's summary reads "Declaration".
                            let title = k["title"].as_str().unwrap_or("Declaration");
                            json!({"project":null,"kind":k["kind"],"title":title,
                        "home":k["home"],"count":count})
                        })
                    })
                    .collect::<Vec<_>>()
            );
        }
        ok(result)
    }

    pub fn citation(&self, c: &super::agreement_render_model::Cite) -> Value {
        json!({"project":c.facts.get("project").cloned().unwrap_or(Value::Null),
            "path":self.site(&c.file),"line":c.line,"column":c.column,"id":c.id,
            "section":c.section,"marker":c.text.starts_with(self.marker()),"text":c.text,
            "enclosing_declaration":c.facts["enclosing"][0],
            "enclosing_section":c.facts["enclosing"][1]})
    }

    pub fn marker(&self) -> &str {
        self.m.facts["marker"].as_str().unwrap_or("§")
    }

    pub fn scanned(&self) -> Vec<String> {
        let mut files: Vec<String> = self
            .fact("scanned")
            .as_array()
            .into_iter()
            .flatten()
            .map(|f| f.as_str().unwrap().to_string())
            .collect();
        files.sort();
        files
    }

    pub fn cover(&self) -> Value {
        let entries: Vec<_> = self
            .scanned()
            .into_iter()
            .filter(|f| self.within(f))
            .map(|f| {
                let cites: Vec<_> = self
                    .m
                    .cites
                    .iter()
                    .filter(|c| c.file == f)
                    .map(|c| self.citation(c))
                    .collect();
                json!({"project":null,"path":self.site(&f),"citations":cites})
            })
            .collect();
        ok(
            json!({"run_cautions":[],"output_format":"text","entries":entries,
            "scan_errors":self.scan_errors_in(false)}),
        )
    }
}
