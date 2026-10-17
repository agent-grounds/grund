//! Coordinate questions rendered from the compact model into Python's envelope:
//! a `--toc` read (§FS-show.2.1.2), the batch of every coordinate in scope
//! (§FS-show.1.9) and the citation set `refs` lists (§FS-refs.2). A refusal is
//! pinned once per coordinate and read by every producer
//! (§REQ-one-answer.proof.evidence.closed).

use super::agreement_render_tree::{View, ok};
use serde_json::{Value, json};

/// The coordinate a model row answers: `(declaration, section)`.
pub type Row = (usize, Option<usize>);

impl View<'_> {
    pub fn coordinate(&self, (d, s): Row) -> String {
        let id = &self.m.decls[d].id;
        s.map_or(id.clone(), |s| format!("{id}.{}", self.m.sections[s].path))
    }

    /// The row a coordinate resolves to, when exactly one model row is it.
    pub fn resolve(&self, coordinate: &str) -> Option<Row> {
        self.order()
            .into_iter()
            .find(|row| self.coordinate(*row) == coordinate)
    }

    pub fn refusal(&self, coordinate: &str, question: &str) -> Option<Value> {
        let refusal = &self.m.facts["refusals"][coordinate];
        let applies = refusal["questions"]
            .as_array()
            .is_none_or(|q| q.iter().any(|q| q == question));
        (!refusal.is_null() && applies).then(|| refusal.clone())
    }

    /// The `show --toc --format json` object, sites relative as written.
    pub fn show_object(&self, (d, s): Row) -> Value {
        let decl = &self.m.decls[d];
        // §FS-values.6.2: every read mode of a JSON value collapses to its slice.
        let sliced = decl.facts["slice"].is_string();
        let children: Vec<usize> = (0..self.m.sections.len())
            .filter(|_| !sliced)
            .filter(|&o| match s {
                Some(s) => self.m.descends(o, s),
                None => self.m.sections[o].owner == d,
            })
            .collect();
        let sections: Vec<_> = children
            .iter()
            .map(|&o| {
                let section = &self.m.sections[o];
                json!({"path":section.path,"title":section.facts["title"],
                    "depth":self.m.depth(o, s),"anchor":section.facts["anchor"]})
            })
            .collect();
        let (section, body, anchor, line) = match s {
            Some(s) => {
                let section = &self.m.sections[s];
                (
                    json!(section.path),
                    section.toc.clone(),
                    section.facts["anchor"].clone(),
                    section.line,
                )
            }
            None => (
                Value::Null,
                decl.toc.clone().unwrap_or_default(),
                decl.facts["anchor"].clone(),
                decl.line,
            ),
        };
        let mut object = json!({"id":decl.id,"section":section,"body":body,"sections":sections,
            "kind_title":self.kind_title(&decl.id),"anchor":anchor,"path":decl.file,"line":line});
        // §FS-config.3.4.3: an absent effective title omits the field.
        if object["kind_title"].is_null() {
            object.as_object_mut().unwrap().remove("kind_title");
        }
        object
    }

    /// Python's ShowResult, its serialized JSON normalized as the comparison does.
    pub fn show_result(&self, row: Row) -> Value {
        let mut object = self.show_object(row);
        object["path"] = json!(self.site(object["path"].as_str().unwrap()));
        let sections: Vec<_> = object["sections"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| json!({"path":s["path"],"title":s["title"],"depth":s["depth"]}))
            .collect();
        json!({"run_cautions":[],"body":object["body"],"path":object["path"],
            "line":object["line"],"json":object.to_string(),"sections":sections})
    }

    pub fn failure(&self, refusal: &Value, causes: Value, partial: Value) -> Value {
        json!({"run_cautions":[],"kind":refusal.get("kind").cloned().unwrap_or(json!("query")),
            "code":refusal["code"],"message":refusal["message"],
            "path":refusal["path"].as_str().map(|p| self.site(p)),"line":null,"column":null,
            "sites":self.sites(&refusal["sites"]),"authority":[],"causes":causes,
            "partial_output":partial,"details":refusal.get("details").cloned().unwrap_or(json!({}))})
    }

    /// §FS-show.2.3: a query over a project the walk could not read in full
    /// refuses with the first unread file; a batch names it in its own fields.
    pub fn unread(&self, batch: bool) -> Option<Value> {
        let errors = self.scan_errors_in(true);
        let first = errors.first()?;
        let (path, message) = (first["path"].as_str()?, first["message"].as_str()?);
        Some(if batch {
            json!({"kind":"filesystem","code":"io","message":message,"path":path})
        } else {
            json!({"kind":"filesystem","code":"io","message":format!("{path}: {message}"),
                "path":path,"details":{"scan_errors":errors}})
        })
    }

    fn absent(&self, coordinate: &str) -> Value {
        let (id, section) = coordinate
            .split_once('.')
            .map_or((coordinate, None), |(i, s)| (i, Some(s)));
        if self.m.decls.iter().any(|d| d.id == id) && section.is_some() {
            json!({"code":"missing-section","message":format!("section not found: {coordinate}")})
        } else {
            json!({"code":"not-found","message":format!("ID not found: {id}")})
        }
    }

    /// Python's answer to `show <coordinate> --toc`.
    pub fn show(&self, coordinate: &str) -> Value {
        let refusal = self
            .unread(false)
            .or_else(|| self.refusal(coordinate, "show"));
        match (refusal, self.resolve(coordinate)) {
            (None, Some(row)) => ok(self.show_result(row)),
            (refusal, _) => {
                let refusal = refusal.unwrap_or_else(|| self.absent(coordinate));
                json!({"failure":self.failure(&refusal, json!([]), Value::Null),
                    "result":null,"run_cautions":[]})
            }
        }
    }

    /// §FS-show.1.9: every coordinate in scope once, a duplicate refused.
    pub fn show_batch(&self) -> Value {
        if let Some(refusal) = self.unread(true) {
            return json!({"failure":self.failure(&refusal, json!([]), Value::Null),
                "result":null,"run_cautions":[]});
        }
        let mut records = Vec::new();
        let mut seen = std::collections::BTreeSet::new();
        for row in self.order() {
            let coordinate = self.coordinate(row);
            if !self.declared_within(row.0) || !seen.insert(coordinate.clone()) {
                continue;
            }
            let (id, section) = (
                &self.m.decls[row.0].id,
                row.1.map(|s| &self.m.sections[s].path),
            );
            let query = json!({"id":id,"section":section});
            records.push(match self.refusal(&coordinate, "show") {
                Some(refusal) => json!({"query":query,"result":null,
                    "failure":self.failure(&refusal, json!([]), Value::Null)}),
                None => json!({"query":query,"result":self.show_result(row),"failure":null}),
            });
        }
        ok(json!({"run_cautions":[],"records":records}))
    }

    /// §FS-refs.2: the citations of an ID, or of exactly one section of it.
    pub fn refs(&self, coordinate: &str) -> Value {
        let partial = json!({"hits":[],"kind_title":null,"note":null,"output_format":"text",
            "scan_errors":self.scan_errors_in(false),"workspace":false});
        if let Some(refusal) = self.refusal(coordinate, "refs") {
            // §FS-workspace.8.1.3: an unknown alias is refused before any scan.
            let (causes, partial) = if refusal["beforeScan"] == true {
                (json!([]), Value::Null)
            } else {
                (json!(["refs scan context"]), partial)
            };
            return json!({"failure":self.failure(&refusal, causes, partial),
                "result":null,"run_cautions":[]});
        }
        let (id, section) = coordinate
            .split_once('.')
            .map_or((coordinate, None), |(i, s)| (i, Some(s)));
        let hits: Vec<_> = self
            .m
            .cites
            .iter()
            .filter(|c| {
                c.id == id && section.is_none_or(|s| c.section == s) && self.within(&c.file)
            })
            .collect();
        let mut files: Vec<(String, usize)> = Vec::new();
        for c in &hits {
            match files.iter_mut().find(|(f, _)| *f == c.file) {
                Some((_, n)) => *n += 1,
                None => files.push((c.file.clone(), 1)),
            }
        }
        // §FS-scan.path.refs: the path's tree is what `refs` scans.
        let cited = self
            .m
            .cites
            .iter()
            .any(|c| c.id == id && self.within(&c.file));
        let declared =
            (0..self.m.decls.len()).any(|d| self.m.decls[d].id == id && self.declared_within(d));
        let note = (!cited && !declared).then(|| {
            format!("{id} is neither declared nor cited \u{2014} run `grund list` to see every declared ID")
        });
        ok(
            json!({"run_cautions":[],"output_format":"text","workspace":false,
            "hits":hits.iter().map(|c| self.citation(c)).collect::<Vec<_>>(),"note":note,
            "scan_errors":self.scan_errors_in(false),"kind_title":self.kind_title(id),
            "site_total":hits.len(),"file_total":files.len(),
            "file_summaries":files.iter().map(|(f, n)| json!({"project":null,"path":self.raw(f),"sites":n}))
                .collect::<Vec<_>>()}),
        )
    }

    /// The editor's `--toc` hover from the loaded snapshot (§FS-lsp.1.2.1).
    pub fn editor_show(&self, coordinate: &str) -> Value {
        let refusal = self
            .unread(false)
            .or_else(|| self.refusal(coordinate, "show"));
        match (refusal, self.resolve(coordinate)) {
            (None, Some(row)) => {
                let object = self.show_object(row);
                json!({"body":object["body"],"json":object.to_string(),"line":object["line"],
                    "path":self.site(object["path"].as_str().unwrap()),
                    "sections":object["sections"].as_array().unwrap().iter().map(|s|
                        json!({"path":s["path"],"title":s["title"],"depth":s["depth"]})).collect::<Vec<_>>()})
            }
            (refusal, _) => {
                let refusal = refusal.unwrap_or_else(|| self.absent(coordinate));
                json!({"failure":refusal["message"]})
            }
        }
    }
}
