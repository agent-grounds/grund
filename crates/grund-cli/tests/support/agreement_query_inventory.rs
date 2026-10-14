//! Query closure comes from input-authored physical facts, never a returned
//! catalog (§REQ-one-answer.proof.evidence, §FS-declarations.section.relative,
//! §FS-workspace.3). The byte partition makes deleting an inventory fact visible.

use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

fn text<'a>(value: &'a Value, name: &str) -> Result<&'a str, String> {
    value[name]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("missing inventory {name}"))
}

fn bytes(value: &Value) -> Result<Vec<u8>, String> {
    value
        .as_array()
        .ok_or("missing inventory bytes")?
        .iter()
        .map(|b| {
            b.as_u64()
                .filter(|n| *n <= 255)
                .map(|n| n as u8)
                .ok_or_else(|| "invalid inventory byte".into())
        })
        .collect()
}

/// Read immutable inputs, including the precisely authored saved edit. Full file
/// membership is checked independently of the facts' own partition membership.
pub fn validate_inputs(variant: &Value, original: &Value) -> Result<(), String> {
    if variant["absent"].is_string() {
        return Ok(());
    }
    let mut expected: BTreeMap<String, Value> = original
        .as_object()
        .ok_or("missing pinned original input files")?
        .iter()
        .filter_map(|(p, v)| {
            p.strip_prefix("repo/")
                .filter(|_| v.get("bytes").is_some())
                .map(|p| (p.into(), v["bytes"].clone()))
        })
        .collect();
    let context = &variant["context"];
    if let Some(relative) = context["overlayRelative"].as_str() {
        if !super::agreement_expected_bytes::safe_relative(relative) {
            return Err("unsafe query inventory saved-edit path".into());
        }
        let held = text(context, "heldBytes")?.as_bytes().to_vec();
        let authored = if let Some(before) = expected.get(relative) {
            if context["bufferOnly"] != false {
                return Err("existing query input cannot be buffer-only".into());
            }
            // The materializer reads text then appends one newline. Python's
            // universal newline input normalizes CRLF and CR before that edit.
            let before =
                String::from_utf8(bytes(before)?).map_err(|_| "overlay input must be UTF-8")?;
            format!("{}\n", before.replace("\r\n", "\n").replace('\r', "\n")).into_bytes()
        } else {
            if context["bufferOnly"] != true {
                return Err("new query input must be buffer-only".into());
            }
            b"Agreement buffer control.\n".to_vec()
        };
        if held != authored {
            return Err(
                "query inventory held bytes differ from the deterministic input edit".into(),
            );
        }
        expected.insert(relative.into(), json!(authored));
    } else if !context["heldBytes"].is_null() || !context["bufferOnly"].is_null() {
        return Err("unedited query input has unexpected held bytes".into());
    }
    let files = variant["queryInventory"]["files"]
        .as_object()
        .ok_or("missing independent physical query inventory")?;
    let actual: BTreeMap<_, _> = files
        .iter()
        .map(|(p, v)| (p.clone(), v["bytes"].clone()))
        .collect();
    if expected != actual {
        return Err(
            "physical query inventory omits or changes immutable input files/saved edit".into(),
        );
    }
    Ok(())
}

struct Fact<'a> {
    role: &'a Value,
    raw: Vec<u8>,
}

fn partition(inventory: &Value) -> Result<BTreeMap<String, Fact<'_>>, String> {
    let files = inventory["files"]
        .as_object()
        .ok_or("missing physical query inventory files")?;
    let mut facts = BTreeMap::new();
    for (file, value) in files {
        if !super::agreement_expected_bytes::safe_relative(file) {
            return Err(format!("unsafe query inventory path {file}"));
        }
        // Byte arrays are compared above to physical inputs; a separate source
        // string is unnecessary. This pass checks every interval in that array.
        let physical = bytes(&value["bytes"])?;
        let length = physical.len();
        let mut cursor = 0usize;
        for segment in value["segments"]
            .as_array()
            .ok_or("missing complete source partition")?
        {
            let start = segment["start"]
                .as_u64()
                .and_then(|n| usize::try_from(n).ok())
                .ok_or("invalid source start")?;
            let end = segment["end"]
                .as_u64()
                .and_then(|n| usize::try_from(n).ok())
                .ok_or("invalid source end")?;
            if start != cursor || end <= start || end > length {
                return Err(format!(
                    "{file}: incomplete/overlapping physical fact partition at {cursor}"
                ));
            }
            let role = &segment["role"];
            if !text(role, "point")?.starts_with('§')
                || text(role, "authoredFrom")?.trim().is_empty()
            {
                return Err(format!("{file}:{start}: missing defining point/provenance"));
            }
            // The exact authored slice is pinned to these physical byte offsets.
            // Binary exclusions use sourceBytes instead of a lossy UTF-8 decode.
            let raw = match (segment.get("source"), segment.get("sourceBytes")) {
                (Some(Value::String(s)), None) => s.as_bytes().to_vec(),
                (None, Some(b)) => bytes(b)?,
                _ => return Err("source partition needs exactly one physical slice".into()),
            };
            if raw != physical[start..end] {
                return Err(format!("{file}:{start}: authored source slice differs"));
            }
            facts.insert(format!("{file}:{start}"), Fact { role, raw });
            cursor = end;
        }
        if cursor != length {
            return Err(format!(
                "{file}: physical fact partition ends at {cursor}, input ends at {length}"
            ));
        }
    }
    Ok(facts)
}

fn source_has(fact: &Fact<'_>, token: &str) -> Result<(), String> {
    if token.is_empty()
        || !fact
            .raw
            .windows(token.len())
            .any(|part| part == token.as_bytes())
    {
        Err(format!(
            "authored query token {token:?} is absent from its physical fact"
        ))
    } else {
        Ok(())
    }
}

fn section<'a>(
    site: &str,
    facts: &'a BTreeMap<String, Fact<'a>>,
    visiting: &mut BTreeSet<String>,
) -> Result<(&'a str, String), String> {
    if !visiting.insert(site.into()) {
        return Err("cyclic authored section ancestry".into());
    }
    let fact = facts
        .get(site)
        .ok_or("section refers to a missing physical fact")?;
    if fact.role["kind"] != "section" {
        return Err("section parent must refer to a section fact".into());
    }
    let declaration = text(fact.role, "declaration")?;
    let component = text(fact.role, "component")?;
    source_has(fact, component)?;
    let path = if let Some(relative) = component.strip_prefix('.') {
        if relative.is_empty() {
            return Err("empty relative section component".into());
        }
        if let Some(parent) = fact.role["parent"].as_str() {
            let (owner, path) = section(parent, facts, visiting)?;
            if owner != declaration {
                return Err("section parent belongs to another declaration".into());
            }
            format!("{path}.{relative}")
        } else {
            relative.into()
        }
    } else {
        component.into()
    };
    visiting.remove(site);
    Ok((declaration, path))
}

/// Closure is a projection of the complete physical facts, with explicit absent
/// questions. A caller-supplied coordinate list is only checked against it.
pub fn closure(inventory: &Value) -> Result<BTreeSet<String>, String> {
    let facts = partition(inventory)?;
    let mut coordinates = BTreeSet::new();
    let mut projects: BTreeMap<Option<&str>, BTreeSet<String>> = BTreeMap::new();
    let mut aliases = Vec::new();
    for (site, fact) in &facts {
        let role = fact.role;
        match text(role, "kind")? {
            "declaration" => {
                let id = text(role, "id")?;
                source_has(fact, id)?;
                text(role, "sectionSeparator")?;
                project(role)?;
                coordinates.insert(id.into());
                projects
                    .entry(role["project"].as_str())
                    .or_default()
                    .insert(id.into());
            }
            "section" => {
                let (owner, path) = section(site, &facts, &mut BTreeSet::new())?;
                let declaration = facts
                    .get(owner)
                    .ok_or("section declaration fact is absent")?;
                if declaration.role["kind"] != "declaration" {
                    return Err("section owner is not a declaration".into());
                }
                let id = text(declaration.role, "id")?;
                let separator = text(declaration.role, "sectionSeparator")?;
                let query = format!("{id}{separator}{path}");
                coordinates.insert(query.clone());
                projects
                    .entry(declaration.role["project"].as_str())
                    .or_default()
                    .insert(query);
            }
            "citation" => {
                let query = text(role, "query")?;
                source_has(fact, query)?;
                coordinates.insert(query.into());
            }
            "alias" => {
                let alias = text(role, "alias")?;
                source_has(fact, alias)?;
                project(role)?;
                aliases.push((alias, role["project"].as_str()));
            }
            "excluded" => {
                text(role, "reason")?;
            }
            unknown => return Err(format!("unsupported physical query fact kind {unknown}")),
        }
    }
    let absent = text(inventory, "absentQuery")?;
    if coordinates.contains(absent) {
        return Err("authored absent control collides with an inventoried coordinate".into());
    }
    for (alias, project) in aliases {
        for query in projects.get(&project).into_iter().flatten() {
            coordinates.insert(format!("{alias}/{query}"));
        }
        coordinates.insert(format!("{alias}/{absent}"));
    }
    coordinates.insert(absent.into());
    Ok(coordinates)
}

fn project(role: &Value) -> Result<(), String> {
    match role.get("project") {
        Some(Value::Null) => Ok(()),
        Some(Value::String(s)) if !s.is_empty() => Ok(()),
        _ => Err("every declaration/alias needs its explicit project identity".into()),
    }
}
