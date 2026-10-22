//! Tree questions rendered from the compact model into Python's envelope, the
//! schema every other producer's answer is mapped from
//! (§REQ-one-answer.proof.evidence.closed). A path argument narrows the report
//! and the listings, never what resolves (§FS-scan.path).

use super::agreement_render_model::{Model, locate, measure};
use serde_json::{Value, json};
use std::cmp::Ordering;

fn parent(file: &str) -> &str {
    file.rsplit_once('/').map_or(".", |(p, _)| p)
}

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
    pub explicit_path: Option<&'a str>,
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

fn under(file: &str, dir: &str) -> bool {
    dir == "." || file == dir || file.starts_with(&format!("{dir}/"))
}

impl View<'_> {
    /// §FS-scan.projects.member: a member run reads its member alone, and a
    /// path narrows what it reports (§FS-scan.path).
    pub fn within(&self, file: &str) -> bool {
        self.loaded(file) && under(file, &self.selected)
    }

    /// §FS-scan.projects.member: the files a run loads, its member's alone in a
    /// member run.
    pub fn loaded(&self, file: &str) -> bool {
        self.member().is_none_or(|member| under(file, member))
    }

    pub fn member(&self) -> Option<&str> {
        self.variant["member"].as_str()
    }

    /// A path as the run's project root reads it: below its member in a member run.
    pub fn display(&self, file: &str) -> String {
        match self.member() {
            Some(member) => file
                .strip_prefix(&format!("{member}/"))
                .unwrap_or(file)
                .to_string(),
            None => file.to_string(),
        }
    }

    /// §FS-scan.projects.workspace-root: a run at the workspace root loads every
    /// member; a member run loads no workspace.
    pub fn workspace(&self) -> bool {
        self.fact("workspace") == true
    }

    /// §FS-workspace.3: the project whose config directory holds `file`, by the
    /// alias its `grund.toml` names, in a workspace run.
    pub fn project_of(&self, file: &str) -> Value {
        let aliases = self.m.facts["aliases"].as_array().into_iter().flatten();
        let owner = aliases
            .filter(|a| self.workspace() && under(file, parent(a["file"].as_str().unwrap())))
            .max_by_key(|a| a["file"].as_str().unwrap().len());
        owner.map_or(Value::Null, |a| a["project"].clone())
    }

    /// §FS-workspace.8.3.1, §FS-workspace.8.1.5: a workspace run names each
    /// declaration by its project's alias.
    pub fn qualified(&self, d: usize) -> String {
        let decl = &self.m.decls[d];
        match decl.project.as_str() {
            Some(project) if self.workspace() => format!("{project}/{}", decl.id),
            _ => decl.id.clone(),
        }
    }

    /// §FS-workspace.4: the project a citation's target lies in, its alias when
    /// written, else the citing project; and its bare ID.
    pub fn aim<'c>(&self, c: &'c super::agreement_render_model::Cite) -> (Value, &'c str) {
        let query = c.facts["query"]
            .as_str()
            .unwrap_or_else(|| c.text.trim_start_matches(self.marker()));
        match query.split_once('/') {
            Some((alias, _)) if self.workspace() => (json!(alias), &c.id),
            _ => (self.project_of(&c.file), &c.id),
        }
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

    /// Declarations and sections by coordinate, then physical site (§FS-list.2.7).
    pub fn order(&self) -> Vec<(usize, Option<usize>)> {
        let mut rows = Vec::new();
        for d in 0..self.m.decls.len() {
            rows.push((d, None));
            rows.extend(
                (0..self.m.sections.len())
                    .filter(|&s| self.m.sections[s].owner == d)
                    .map(|s| (d, Some(s))),
            );
        }
        // §FS-list.2.7: duplicate homes interleave by coordinate, then site;
        // every claimant's measurement stays local to its own physical body.
        rows.sort_by(|&(a, sa), &(b, sb)| {
            let section = |s: Option<usize>| s.map(|s| self.m.sections[s].path.as_str());
            natural(&self.m.decls[a].id, &self.m.decls[b].id)
                .then_with(|| match (section(sa), section(sb)) {
                    (Some(a), Some(b)) => natural(a, b),
                    (a, b) => a.cmp(&b),
                })
                .then(self.m.decls[a].file.cmp(&self.m.decls[b].file))
                .then_with(|| {
                    let line = |d: usize, s: Option<usize>| {
                        s.map_or(self.m.decls[d].line, |s| self.m.sections[s].line)
                    };
                    line(a, sa).cmp(&line(b, sb))
                })
        });
        rows
    }

    /// §FS-lsp.1.1: a finding about a citation starts at the citation's column,
    /// the `nth` occurrence of its pinned `token` on the finding's line.
    pub fn column(&self, f: &Value) -> Value {
        let (Some(token), Some(path), Some(line)) =
            (f["token"].as_str(), f["path"].as_str(), f["line"].as_u64())
        else {
            return Value::Null;
        };
        let nth = f["nth"].as_u64().unwrap_or(1) as usize;
        self.m
            .text(path)
            .ok()
            .and_then(|text| locate(text, token, nth, line as usize).ok())
            .map_or(Value::Null, |(_, column, _)| json!(column))
    }

    pub fn finding(&self, f: &Value) -> Value {
        json!({"severity":f["severity"],"channel":null,"code":f["code"],
            "path":f["path"].as_str().map(|p| self.site(p)),"line":f["line"],
            "column":self.column(f),"message":f["message"],"sites":self.sites(&f["sites"]),
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
    /// names lies at or under the path; §FS-check.1.3.6.1: the agent-entrypoint
    /// finding is a probe over the root, which the path never drops;
    /// §FS-check.1.3.6.3: a file outside it that the walk could not read is a
    /// caution, never an error.
    pub fn findings(&self) -> Vec<Value> {
        let mut out = Vec::new();
        for f in self.fact("findings").as_array().into_iter().flatten() {
            let inside = f["code"] == "agents-init"
                || f["path"].as_str().is_none_or(|p| self.within(p))
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

    /// §FS-workspace.8.3.5: the citations of a declaration under its project.
    fn ref_count(&self, d: usize) -> usize {
        let decl = &self.m.decls[d];
        let project = if self.workspace() {
            decl.project.clone()
        } else {
            Value::Null
        };
        self.m
            .cites
            .iter()
            .filter(|c| self.loaded(&c.file) && self.aim(c) == (project.clone(), &decl.id))
            .count()
    }

    fn entry(&self, d: usize, s: Option<usize>, size: bool) -> Value {
        let decl = &self.m.decls[d];
        let stub = decl.facts.get("stub").filter(|v| !v.is_null());
        let project = if self.workspace() {
            decl.project.clone()
        } else {
            Value::Null
        };
        let mut row = json!({"project":project,"id":self.qualified(d),"section":null,
            "section_separator":".","kind":decl.kind,"path":self.site(&decl.file),
            "line":decl.line,"stub":stub.is_some(),"defines":stub,
            "duplicate":decl.facts["duplicate"].as_bool().unwrap_or(false)});
        let (lead, full) = match s {
            Some(s) => {
                let section = &self.m.sections[s];
                row["section"] = json!(section.path);
                row["line"] = json!(section.line);
                row["duplicate"] = json!(
                    decl.facts["duplicate"].as_bool().unwrap_or(false)
                        || section.facts["duplicate"].as_bool().unwrap_or(false)
                );
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
            row["refs"] = json!(self.ref_count(d));
            row["value_roots"] = json!(self.value_roots(d));
        }
        row
    }

    /// §FS-list.3.2.2: a declaration's value roots of either origin, marked
    /// (§FS-values.2.4) or chapter-declared (§FS-values.2.5), in canonical
    /// section-path order.
    fn value_roots(&self, d: usize) -> Vec<Value> {
        let mut roots: Vec<_> = self
            .m
            .sections
            .iter()
            .filter(|s| s.owner == d && s.facts["valueRoot"].is_string())
            .collect();
        roots.sort_by(|a, b| natural(&a.path, &b.path));
        roots
            .into_iter()
            .map(|s| {
                json!({"id":format!("{}.{}", self.qualified(d), s.path),
                    "valid":s.facts["valueRootValid"].as_bool().unwrap_or(true)})
            })
            .collect()
    }

    pub fn list(&self, size: bool) -> Value {
        let rows: Vec<_> = self
            .order()
            .into_iter()
            .filter(|(d, s)| self.declared_within(*d) && (size || s.is_none()))
            .collect();
        // §FS-coordinate-globs.bindings.note: a run without globs prints no note.
        let mut result = json!({"run_cautions":[],"output_format":"text","workspace":self.workspace(),
            "entries":rows.iter().map(|(d, s)| self.entry(*d, *s, size)).collect::<Vec<_>>(),
            "scan_errors":self.scan_errors(),"note":null});
        if !size {
            // §FS-workspace.8.3.4: one summary per project and kind in a workspace.
            let mut projects: Vec<Value> = rows
                .iter()
                .map(|(d, _)| self.entry(*d, None, true)["project"].clone())
                .collect();
            projects.sort_by_key(Value::to_string);
            projects.dedup();
            result["summaries"] = json!(
                projects
                    .iter()
                    .flat_map(|project| self.kinds().into_iter().map(move |k| (project, k)))
                    .filter_map(|(project, k)| {
                        let count = rows
                            .iter()
                            .filter(|(d, s)| {
                                s.is_none()
                                    && self.m.decls[*d].kind == k["kind"].as_str().unwrap()
                                    && self.entry(*d, None, true)["project"] == *project
                            })
                            .count();
                        (count > 0).then(|| {
                            // An untitled kind's summary reads "Declaration".
                            let title = k["title"].as_str().unwrap_or("Declaration");
                            json!({"project":project,"kind":k["kind"],"title":title,
                        "home":k["home"],"count":count})
                        })
                    })
                    .collect::<Vec<_>>()
            );
        }
        ok(result)
    }

    /// A citation row; `cover` names a qualified target as written
    /// (§FS-workspace.8.6.4), `refs` leaves it to the query (§FS-workspace.8.2.4).
    pub fn citation(&self, c: &super::agreement_render_model::Cite, written: bool) -> Value {
        let query = c.facts["query"]
            .as_str()
            .unwrap_or_else(|| c.text.trim_start_matches(self.marker()));
        let id = match query.split_once('/') {
            Some((alias, _)) if written && self.workspace() => format!("{alias}/{}", c.id),
            _ => c.id.clone(),
        };
        json!({"project":self.project_of(&c.file),
            "path":self.site(&c.file),"line":c.line,"column":c.column,"id":id,
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
            .filter(|f| self.loaded(f))
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
                    .map(|c| self.citation(c, true))
                    .collect();
                json!({"project":self.project_of(&f),"path":self.site(&f),"citations":cites})
            })
            .collect();
        // §FS-cover.globs: a run without globs prints no note.
        ok(
            json!({"run_cautions":[],"output_format":"text","entries":entries,
            "scan_errors":self.scan_errors_in(false),"note":null}),
        )
    }
}
