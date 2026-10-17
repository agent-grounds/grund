//! The physical query inventory a compact model pins: every input file's bytes
//! partitioned into the declaration IDs, written section components, citation
//! tokens and aliases it holds, the rest excluded with a reason
//! (§REQ-one-answer.proof.evidence, §FS-declarations.section.relative).

use super::agreement_render_model::{Model, locate};
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn role(facts: &Value, kind: &str, extra: Value) -> Value {
    let mut role = json!({"kind":kind,"point":facts["point"],"authoredFrom":facts["authoredFrom"]});
    for (key, value) in extra.as_object().unwrap() {
        role[key] = value.clone();
    }
    role
}

/// The byte partition of every input file: each pinned declaration ID,
/// written section component and citation token, the rest excluded.
pub fn inventory(m: &Model, files: &BTreeMap<String, Vec<u8>>) -> Result<Value, String> {
    let mut segments: BTreeMap<String, Vec<(usize, usize, Value)>> = BTreeMap::new();
    let mut decl_sites = Vec::new();
    for d in &m.decls {
        let text = m.text(&d.file)?;
        let (_, _, at) = locate(
            text,
            &d.text,
            d.facts["nth"].as_u64().unwrap_or(1) as usize,
            1,
        )?;
        let start = at + d.text.find(&d.id).ok_or("heading lacks its ID")?;
        decl_sites.push(format!("{}:{start}", d.file));
        segments.entry(d.file.clone()).or_default().push((
            start,
            start + d.id.len(),
            role(
                &m.facts,
                "declaration",
                json!({"id":d.id,"project":d.project,"sectionSeparator":"."}),
            ),
        ));
    }
    let mut section_sites: Vec<String> = Vec::new();
    for s in &m.sections {
        let text = m.text(&s.file)?;
        let (_, _, at) = locate(
            text,
            &s.text,
            s.facts["nth"].as_u64().unwrap_or(1) as usize,
            m.decls[s.owner].line,
        )?;
        let component = s.facts["component"].as_str().unwrap_or(&s.path);
        let start = at
            + s.text
                .find(component)
                .ok_or("heading lacks its component")?;
        section_sites.push(format!("{}:{start}", s.file));
        segments.entry(s.file.clone()).or_default().push((
            start,
            start + component.len(),
            role(
                &m.facts,
                "section",
                json!({"declaration":decl_sites[s.owner],"component":component,
                "parent":s.parent.map(|p| section_sites[p].clone())}),
            ),
        ));
    }
    let marker = m.facts["marker"].as_str().unwrap_or("§");
    for c in &m.cites {
        let nth = c.facts["nth"].as_u64().unwrap_or(1) as usize;
        let (_, _, start) = locate(m.text(&c.file)?, &c.text, nth, 1)?;
        let query = c.facts["query"]
            .as_str()
            .unwrap_or_else(|| c.text.trim_start_matches(marker));
        segments.entry(c.file.clone()).or_default().push((
            start,
            start + c.text.len(),
            role(&m.facts, "citation", json!({"query":query})),
        ));
    }
    for a in m.facts["aliases"].as_array().into_iter().flatten() {
        let file = a["file"].as_str().ok_or("alias lacks file")?;
        let token = a["alias"].as_str().ok_or("alias lacks its name")?;
        let (_, _, start) = locate(
            m.text(file)?,
            token,
            a["nth"].as_u64().unwrap_or(1) as usize,
            1,
        )?;
        segments.entry(file.into()).or_default().push((
            start,
            start + token.len(),
            role(
                &m.facts,
                "alias",
                json!({"alias":token,"project":a["project"]}),
            ),
        ));
    }
    let mut out = serde_json::Map::new();
    for (file, bytes) in files {
        let mut pinned = segments.remove(file).unwrap_or_default();
        pinned.sort_by_key(|s| s.0);
        let mut partition = Vec::new();
        let mut cursor = 0;
        let excluded = role(
            &m.facts,
            "excluded",
            json!({"reason":"bytes that write no declaration ID, section component or citation"}),
        );
        let slice = |a: usize, b: usize| match std::str::from_utf8(&bytes[a..b]) {
            Ok(s) => json!({"start":a,"end":b,"source":s}),
            Err(_) => json!({"start":a,"end":b,"sourceBytes":bytes[a..b].to_vec()}),
        };
        for (start, end, role) in pinned {
            if start < cursor {
                return Err(format!("{file}:{start}: overlapping pinned facts"));
            }
            if start > cursor {
                let mut gap = slice(cursor, start);
                gap["role"] = excluded.clone();
                partition.push(gap);
            }
            let mut segment = slice(start, end);
            segment["role"] = role;
            partition.push(segment);
            cursor = end;
        }
        if cursor < bytes.len() {
            let mut gap = slice(cursor, bytes.len());
            gap["role"] = excluded;
            partition.push(gap);
        }
        out.insert(file.clone(), json!({"bytes":bytes,"segments":partition}));
    }
    Ok(json!({"files":out,"absentQuery":m.facts["absentQuery"].as_str().unwrap_or("FS-404")}))
}
