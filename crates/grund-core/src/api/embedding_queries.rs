//! Delegation for the complete read/query inventory (§FS-distribution.3.3.5).

use super::cover::cover_text_with_globs;
use super::embedding::EmbeddingRequest;
use super::embedding_data::Data;
use super::embedding_failure::{error_data, failure};
use crate::config::display_path;
use crate::*;
use serde_json::{Value, json};

/// A batch's explicit queries, or `None` for the generated batch
/// (§FS-distribution.3.2.1).
pub(super) fn batch_queries(r: &EmbeddingRequest) -> Option<Vec<BatchShowQuery>> {
    r.args
        .first()
        .filter(|v| !v.is_null())
        .and_then(Value::as_array)
        .map(|queries| {
            queries
                .iter()
                .map(|q| BatchShowQuery {
                    id: q
                        .as_str()
                        .or_else(|| q["id"].as_str())
                        .unwrap_or_default()
                        .to_owned(),
                    section: q["section"].as_str().map(str::to_owned),
                })
                .collect()
        })
}

/// The one `show_batch` result shape, which the empty batch, Python's and Node's
/// projections all build, so none of them leaves a field out (§FS-distribution.3.0.2).
pub(super) fn batch_result(records: Vec<Value>, note: Option<String>, cautions: Value) -> Value {
    json!({"records": records, "note": note, "run_cautions": cautions})
}

pub(super) fn show_opts(r: &EmbeddingRequest) -> ShowOpts {
    ShowOpts {
        path: r.root.clone(),
        section: r.string("section"),
        mode: match r.string("mode").as_deref() {
            Some("brief") => ShowMode::Brief,
            Some("toc") => ShowMode::Toc,
            Some("full") => ShowMode::Full,
            _ => ShowMode::Lead,
        },
        format: match r.string("format").as_deref() {
            Some("md") => ShowFormat::Markdown,
            Some("json") => ShowFormat::Json,
            _ => ShowFormat::Text,
        },
    }
}

/// The scanner snapshot owns catalog/citation/diagnostic data, independently
/// of a check verdict (§FS-distribution.3.3.1).
pub(super) fn scan_data(r: &EmbeddingRequest) -> Result<Value, Value> {
    let config =
        crate::workspace::resolve_workspace_config(&r.root).map_err(|e| error_data(e, &[]))?;
    let (findings, errors) =
        crate::scanner::scan_tree(config.schema(), config.frame(), Some(&r.root), true)
            .map_err(|e| error_data(e, &config_run_warnings(&config)))?;
    // §FS-config.3.2.8: a declaration's spelling can differ from its numeric map key.
    let catalog = findings.declarations.values().flat_map(|decls| {
        decls.iter().map(|d| json!({"id": crate::grammar::render_declaration_id(&config.grammar, d),
            "path": display_path(&config, &d.file), "line": d.line,
            "heading_level": d.heading_level, "title": d.title, "stub": d.is_stub,
            "defines": d.defined_in.as_ref().map(|p| crate::model::format_path(p)),
            "body_start": d.body_start, "body_end": d.body_end,
            "body_has_content": d.body_has_content, "value_valid": d.value_valid,
            "sections": d.sections.iter().map(|(path,s)| json!({"path":path,
                "title":s.title, "line":s.line, "heading_level":s.heading_level})).collect::<Vec<_>>(),
            "duplicate_sections": d.duplicate_sections.iter().map(|(path,s)| json!({"path":path,
                "title":s.title, "line":s.line, "heading_level":s.heading_level})).collect::<Vec<_>>() }))
    }).collect::<Vec<_>>();
    let citations = findings.citations.iter().map(|c| json!({
        "project": c.namespace, "id": crate::grammar::render_citation_id(&config.grammar,&findings.declarations,c,&config.marker),
        "section": c.section, "path": display_path(&config,&c.file),
        "line": c.line, "column": c.column, "marker": c.has_marker, "text": c.text,
        "shorthand": c.shorthand, "local_section": c.local_section,
        "shorthand_rewritable": c.shorthand_rewritable, "numeric_run": c.numeric_run,
        "source_kind": c.source_kind,
        "enclosing_declaration": c.enclosing_declaration.as_ref().map(|id|crate::grammar::render_catalog_id(&config.grammar,&findings.declarations,id)),
        "enclosing_section": c.enclosing_section,
    })).collect::<Vec<_>>();
    Ok(json!({"catalog":catalog, "citations":citations,
        "scanned_files": findings.scanned_files.iter().map(|p|display_path(&config,p)).collect::<Vec<_>>(),
        // §FS-check.2.4: the one record every other operation publishes for a failure.
        "scan_errors":errors.iter().map(|(p,m)|crate::scanner::api_scan_error(config.frame(),p,m))
            .collect::<Vec<_>>().data(),
        "run_cautions":config_run_warnings(&config).data()}))
}

/// What a query refuses before config discovery: options that conflict (§FS-refs.1.2,
/// §FS-cover.6.4), then a pattern the grammar alone refuses
/// (§FS-coordinate-globs.errors.shape), in the order the CLI answers them.
pub(super) fn refuse_before_discovery(r: &EmbeddingRequest) -> Result<(), Value> {
    match r.operation.as_str() {
        "refs" => drop(super::embedding_refs::refs_sites(r)?),
        "cover" if !r.strings("lines").is_empty() => cover_lines_conflict(r)?,
        _ => {}
    }
    crate::queries::read_globs(&r.strings("globs"))
        .map(drop)
        .map_err(|e| error_data(e, &[]))
}

/// §FS-cover.6.4: `lines` prints no grouping for `globs` to filter.
fn cover_lines_conflict(r: &EmbeddingRequest) -> Result<(), Value> {
    match r.strings("globs").is_empty() {
        true => Ok(()),
        false => {
            let message = "globs cannot be combined with lines".to_string();
            Err(failure("input", "conflicting-options", message, &[]))
        }
    }
}

pub(super) fn query(r: &EmbeddingRequest) -> Result<Value, Value> {
    match r.operation.as_str() {
        "show" => {
            let (cautions, result) = show_with_scope(r.arg(0), show_opts(r), r.explicit);
            let output = result.map_err(|e| error_data(e, &cautions))?;
            let mut data = output.data();
            // §FS-distribution.3.3.3: output paths are logical, never native absolutes.
            let config = crate::workspace::resolve_workspace_config(&r.root)
                .map_err(|e| error_data(e, &cautions))?;
            data["path"] = json!(display_path(&config, &output.path));
            data["run_cautions"] = cautions.data();
            Ok(data)
        }
        "show_batch" => {
            let (cautions, result) = crate::queries::show_batch_data(
                batch_queries(r),
                &r.strings("globs"),
                show_opts(r),
                r.explicit,
            );
            let (records, note) = result.map_err(|e| error_data(e, &cautions))?;
            let records = records
                .into_iter()
                .map(|record| {
                    let (result, refusal) = match record.result {
                        Ok(value) => {
                            let mut data = value.data();
                            data["run_cautions"] = cautions.data();
                            (data, Value::Null)
                        }
                        Err(e) => (Value::Null, error_data(e, &cautions)),
                    };
                    json!({"query":{"id":record.query.id,"section":record.query.section},
                    "result":result,"failure":refusal})
                })
                .collect::<Vec<_>>();
            Ok(batch_result(records, note, cautions.data()))
        }
        "refs" => refs_data(r),
        "list_ids" => {
            let (cautions, result) = list_with_run_warnings(ListOpts {
                path: r.root.clone(),
                path_provided: r.explicit,
                kind_filter: r.strings("kinds").into_iter().collect(),
                project_filter: r.strings("projects").into_iter().collect(),
                unused_only: r.flag("unused"),
                selector: r.string("selector"),
                globs: r.strings("globs"),
            });
            let out = result.map_err(|e| error_data(e, &cautions))?;
            Ok(
                json!({"output_format":out.output_format,"workspace":out.workspace,
                "entries":out.entries.data(),"summaries":out.summaries.data(),
                "scan_errors":out.scan_errors.data(),"note":out.note,
                "run_cautions":cautions.data()}),
            )
        }
        "list_sizes" => {
            let mut opts = ListSizeOpts {
                path: r.root.clone(),
                path_provided: r.explicit,
                kind_filter: r.strings("kinds").into_iter().collect(),
                project_filter: r.strings("projects").into_iter().collect(),
                unused_only: r.flag("unused"),
                selector: r.string("selector"),
                globs: r.strings("globs"),
                top: r.options["top"].as_u64().map(|n| n as usize),
                ..Default::default()
            };
            if r.options["units"].is_array() {
                opts.units = r
                    .strings("units")
                    .iter()
                    .filter_map(|u| PointSizeUnit::parse(u))
                    .collect();
            }
            let (cautions, result) = list_sizes_with_run_warnings(opts);
            let out = result.map_err(|e| error_data(e, &cautions))?;
            Ok(
                json!({"output_format":out.output_format,"workspace":out.workspace,
                "entries":out.entries.data(),"scan_errors":out.scan_errors.data(),
                "note":out.note,"run_cautions":out.warnings.data()}),
            )
        }
        // §FS-cover.6: non-empty `lines` asks for line ownership instead.
        "cover" if !r.strings("lines").is_empty() => {
            let (cautions, result) = cover_lines_with_run_warnings(CoverLinesOpts {
                path: r.root.clone(),
                path_provided: r.explicit,
                lines: r.strings("lines"),
            });
            let out = result.map_err(|e| error_data(e, &cautions))?;
            Ok(
                json!({"output_format":out.output_format,"records":out.records.data(),"scan_errors":out.scan_errors.data(),"run_cautions":out.warnings.data()}),
            )
        }
        "cover" => {
            let opts = CoverOpts {
                path: r.root.clone(),
                path_provided: r.explicit,
            };
            // §FS-cover.globs: every entry stays, holding the citations `globs` select.
            let globs = r.strings("globs");
            if r.flag("text") {
                let (cautions, result) = cover_text_with_globs(opts, &globs);
                let (out, note) = result.map_err(|e| error_data(e, &cautions))?;
                Ok(
                    json!({"output_format":out.output_format,"entries":out.entries.data(),"scan_errors":out.scan_errors.data(),"note":note,"run_cautions":out.warnings.data()}),
                )
            } else {
                let (cautions, result) = cover_with_globs(opts, &globs);
                let (out, note) = result.map_err(|e| error_data(e, &cautions))?;
                Ok(
                    json!({"output_format":out.output_format,"entries":out.entries.data(),"scan_errors":out.scan_errors.data(),"note":note,"run_cautions":out.warnings.data()}),
                )
            }
        }
        "complete_ids" => {
            let (cautions, result) = complete_ids_with_run_warnings(CompleteIdsOpts {
                path: r.root.clone(),
                path_provided: r.explicit,
                prefix: r.arg(0).to_owned(),
                sections: r.flag("sections"),
            });
            Ok(
                json!({"candidates":result.map_err(|e|error_data(e,&cautions))?,"run_cautions":cautions.data()}),
            )
        }
        "propose_id" => {
            let (cautions, result) = propose_id_with_run_warnings(
                r.arg(0),
                r.arg(1),
                IdOpts {
                    path: r.root.clone(),
                    path_provided: r.explicit,
                    width: r.options["width"].as_u64().unwrap_or(3) as usize,
                },
            );
            match result.map_err(|e| error_data(e, &cautions))? {
                IdProposalOutcome::Proposed(out) => {
                    let mut data = out.data();
                    data["run_cautions"] = cautions.data();
                    Ok(data)
                }
                IdProposalOutcome::UnknownKind { headline, known } => {
                    let mut f = failure("operation", "unknown-kind", headline, &cautions);
                    f["details"] = json!({"known":known});
                    Err(f)
                }
                IdProposalOutcome::Rejected { message } => {
                    Err(failure("query", "query-failed", message, &cautions))
                }
            }
        }
        _ => unreachable!("read operation selected by core dispatch"),
    }
}

fn refs_data(r: &EmbeddingRequest) -> Result<Value, Value> {
    // §FS-coordinate-globs.bindings.options: `globs` or `from_` select the sites instead.
    if let Some(sites) = super::embedding_refs::refs_sites(r)? {
        return super::embedding_refs::refs_sites_data(r, sites);
    }
    let mut details = json!({});
    let mut cautions = Vec::new();
    let meta = super::refs_query::refs_impl_with_details(
        RefsOpts {
            path: r.root.clone(),
            path_provided: r.explicit,
            id: r.arg(0).to_owned(),
            section: r.string("section"),
            descendants: r.flag("descendants"),
        },
        &mut details,
        &mut cautions,
        &mut None,
    )
    .map_err(|e| error_data(e, &cautions))?;
    let out = meta.outcome.output;
    if let Some(e) = meta.outcome.query_failure {
        let mut f = failure("query", e.kind.code(), e.message, &out.warnings);
        // The resolver carrier retains structured candidates at its source.
        f["details"] = details;
        if let Some(hint) = e.format_hint {
            f["details"]["format_hint"] = json!(hint);
        }
        return Err(f);
    }
    Ok(super::embedding_refs::refs_result(&out, meta.kind_title))
}
