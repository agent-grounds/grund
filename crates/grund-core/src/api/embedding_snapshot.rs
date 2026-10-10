//! Strict scanner data and available partial facts (§FS-distribution.3.2.2.3).
use super::embedding::EmbeddingRequest;
use super::embedding_data::Data;
use super::embedding_failure::{error_data, failure};
use crate::config::{Config, display_path};
use crate::grammar::render_id;
use crate::*;
use serde_json::{Value, json};

pub(super) fn scan(r: &EmbeddingRequest) -> Result<Value, Value> {
    crate::workspace::preflight_embedding_paths(&r.root).map_err(|e| error_data(e, &[]))?;
    let config =
        crate::workspace::resolve_workspace_config(&r.root).map_err(|e| error_data(e, &[]))?;
    let cautions = config_run_warnings(&config);
    let (findings, errors) =
        crate::scanner::scan_tree(config.schema(), config.frame(), Some(&r.root), true)
            .map_err(|e| error_data(e, &cautions))?;
    let snapshot = snapshot(&config, &findings);
    if !errors.is_empty() {
        let mut f = failure("io", "filesystem", errors[0].1.clone(), &cautions);
        f["path"] = json!(display_path(&config, &errors[0].0));
        f["partial_output"] = snapshot;
        return Err(f);
    }
    Ok(json!({"snapshot":snapshot,"run_cautions":cautions.data()}))
}

fn snapshot(c: &Config, f: &Catalog) -> Value {
    let id = |i: &Id| render_id(&c.grammar, i);
    let path = |p: &std::path::Path| display_path(c, p);
    let section = |p: &str, s: &SectionInfo| {
        let mut v = s.data();
        v["path"] = json!(p);
        v
    };
    let mut declarations = f.declarations.iter().flat_map(|(i,ds)|ds.iter().map(|d|
        json!({"id":id(i),"kind":i.kind,"number":i.num,"slug":i.slug,
            "path":path(&d.file),"line":d.line,"heading_level":d.heading_level,"title":d.title,
            "sections":d.sections.iter().map(|(p,s)|section(p,s)).collect::<Vec<_>>(),
            "duplicate_sections":d.duplicate_sections.iter().map(|(p,s)|section(p,s)).collect::<Vec<_>>(),
            "stub":d.is_stub,"defines":d.defined_in.as_ref().map(|p|path(p)),
            "e2e_case":d.e2e_case.as_ref().map(Data::data),
            "body_start":d.body_start,"body_end":d.body_end,"body_has_content":d.body_has_content,
            "source":d.source.data(),"value_valid":d.value_valid}))).collect::<Vec<_>>();
    declarations.sort_by(|a, b| {
        a["id"]
            .as_str()
            .cmp(&b["id"].as_str())
            .then(a["path"].as_str().cmp(&b["path"].as_str()))
            .then(a["line"].as_u64().cmp(&b["line"].as_u64()))
    });
    let citation = |v: &Citation| {
        json!({"namespace":v.namespace,"id":id(&v.id),
        "section":v.section,"path":path(&v.file),"line":v.line,"column":v.column,
        "marker":v.has_marker,"shorthand":v.shorthand,"local_section":v.local_section,
        "shorthand_rewritable":v.shorthand_rewritable,"numeric_run":v.numeric_run,
        "text":v.text,"inline_site":v.inline_site.as_ref().map(Data::data),"source_kind":v.source_kind,
        "enclosing_declaration":v.enclosing_declaration.as_ref().map(id),
        "enclosing_section":v.enclosing_section})
    };
    let invalid = |v: &InvalidValueSite| {
        json!({"id":v.id.as_ref().map(id),
        "path":path(&v.file),"line":v.line,"column":v.column,"message":v.message,
        "source":v.source.data(),"binding_namespace":v.binding_namespace,"binding_section":v.binding_section})
    };
    json!({"declarations":declarations,
        "citations":f.citations.iter().map(citation).collect::<Vec<_>>(),
        "escaped_citations":f.escaped_citations.iter().map(citation).collect::<Vec<_>>(),
        "scanned_files":f.scanned_files.iter().map(|p|path(p)).collect::<Vec<_>>(),
        "walked_dirs":f.walked_dirs.iter().map(|p|path(p)).collect::<Vec<_>>(),
        "file_structures":f.file_structure.iter().map(|(p,s)|json!({"path":path(p),
            "headings":s.headings.iter().map(|h|json!({"line":h.line,"level":h.level,"text":h.text})).collect::<Vec<_>>(),
            "doc_comments":s.doc_comments.iter().map(|d|json!({"start":d.start,"end":d.end,"indented":d.indented})).collect::<Vec<_>>(),
            "total_lines":s.total_lines})).collect::<Vec<_>>(),
        "value_bindings":f.value_bindings.iter().map(|v|json!({"namespace":v.namespace,
            "id":id(&v.id),"section":v.section,"authored":v.authored.data(),"path":path(&v.file),
            "line":v.line,"column":v.column})).collect::<Vec<_>>(),
        "invalid_value_declarations":f.invalid_value_declarations.iter().map(invalid).collect::<Vec<_>>(),
        "invalid_value_bindings":f.invalid_value_bindings.iter().map(invalid).collect::<Vec<_>>(),
        "section_headings_outside_declarations":f.section_headings_outside_declarations.iter()
            .map(|v|json!({"path":path(&v.file),"line":v.line,"section":v.path})).collect::<Vec<_>>(),
        "unmarked_headings":f.unmarked_headings.iter().map(|v|json!({"path":path(&v.file),
            "line":v.line,"column":v.column,"heading":v.heading,"heading_level":v.heading_level,
            "title":v.title,"owner":id(&v.owner),"suggested_path":v.suggested_path})).collect::<Vec<_>>(),
        "near_miss_headings":f.near_miss_headings.iter().map(|v|json!({"path":path(&v.file),
            "line":v.line,"text":v.text,"format":v.format})).collect::<Vec<_>>()})
}
