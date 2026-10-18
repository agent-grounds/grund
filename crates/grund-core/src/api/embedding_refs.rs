//! Refs transport folds preserve the CLI's path order and unique lines (§FS-distribution.3.2.2.2),
//! and the host's `globs` and `from_` select the sites `refs --glob` and `refs --from` list
//! (§FS-coordinate-globs.bindings.options).
use super::embedding::EmbeddingRequest;
use super::embedding_data::Data;
use super::embedding_failure::{error_data, failure};
use crate::{RefsOutput, RefsSites, RefsSitesOpts, refs_sites_with_run_warnings};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
pub(super) fn summaries(hits: &Value) -> Value {
    let mut by_file = BTreeMap::<String, (Value, usize, BTreeSet<u64>)>::new();
    for h in hits.as_array().into_iter().flatten() {
        let row = by_file
            .entry(h["path"].as_str().unwrap_or("").into())
            .or_insert_with(|| (h["project"].clone(), 0, BTreeSet::new()));
        row.1 += 1;
        row.2.insert(h["line"].as_u64().unwrap_or(0));
    }
    json!(by_file.into_iter().map(|(path,(project,count,lines))|
        json!({"project":project,"path":path,"count":count,"lines":lines})).collect::<Vec<_>>())
}

/// What a host's `refs` call selects its sites by instead of one ID: `None` in ID mode,
/// and a refusal where it names two selectors, or `section` or `descendants` beside
/// one (§FS-coordinate-globs.bindings.options, §FS-refs.1.2).
pub(super) fn refs_sites(r: &EmbeddingRequest) -> Result<Option<RefsSites>, Value> {
    let globs = r.strings("globs");
    let from = r.string("from_");
    let selector = match (globs.is_empty(), from.is_some()) {
        (true, false) => return Ok(None),
        (true, true) => "from_",
        (false, false) => "globs",
        (false, true) => return Err(conflict("globs cannot be combined with from_")),
    };
    let id = r.args.first().is_some_and(|operand| !operand.is_null());
    for (given, option) in [
        (id, "id"),
        (r.string("section").is_some(), "section"),
        (r.flag("descendants"), "descendants"),
    ] {
        if given {
            return Err(conflict(&format!(
                "{option} cannot be combined with {selector}"
            )));
        }
    }
    Ok(Some(match from {
        Some(source) => RefsSites::From(source),
        None => RefsSites::Globs(globs),
    }))
}

fn conflict(message: &str) -> Value {
    failure("input", "conflicting-options", message.to_string(), &[])
}

/// `refs` under `globs` or `from_`: the CLI's records with no kind title, and the
/// patterns' note (§FS-coordinate-globs.bindings.note, §FS-refs.3.2).
pub(super) fn refs_sites_data(r: &EmbeddingRequest, sites: RefsSites) -> Result<Value, Value> {
    let (cautions, result) = refs_sites_with_run_warnings(RefsSitesOpts {
        path: r.root.clone(),
        path_provided: r.explicit,
        sites,
    });
    let out = result.map_err(|e| error_data(e, &cautions))?;
    Ok(refs_result(&out, None))
}

/// One `refs` result with its per-file folds, whichever mode listed its hits
/// (§FS-distribution.3.2.2.2).
pub(super) fn refs_result(out: &RefsOutput, kind_title: Option<String>) -> Value {
    let mut files = BTreeMap::<(Option<String>, String), usize>::new();
    for hit in &out.hits {
        *files
            .entry((hit.project.clone(), hit.path.clone()))
            .or_default() += 1;
    }
    json!({"output_format":out.output_format,"workspace":out.workspace,"hits":out.hits.data(),
    "note":out.note,"scan_errors":out.scan_errors.data(),"run_cautions":out.warnings.data(),
    "kind_title":kind_title,"site_total":out.hits.len(),"file_total":files.len(),
    "file_summaries":files.into_iter().map(|((project,path),sites)|json!({"project":project,"path":path,"sites":sites})).collect::<Vec<_>>() })
}
