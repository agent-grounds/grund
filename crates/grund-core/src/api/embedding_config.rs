//! Schema-keyed configuration snapshots (§FS-distribution.3.3.1).

use super::embedding::EmbeddingRequest;
use super::embedding_data::Data;
use super::embedding_failure::error_data;
use crate::*;
use serde_json::{Value, json};

pub(super) fn config(r: &EmbeddingRequest) -> Result<Value, Value> {
    let c = if r.operation == "validate_config" {
        validate_config(&r.root)
    } else {
        effective_config(&r.root)
    }
    .map_err(|e| error_data(e, &[]))?;
    let cautions = config_run_warnings(&c).data();
    if r.operation == "reference_style" {
        let s = reference_style(&r.root).map_err(|e| error_data(e, &[]))?;
        return Ok(json!({"marker":s.marker,"trigger":s.trigger,"run_cautions":cautions}));
    }
    Ok(json!({"config": schema(&c), "warnings": config_warnings(&c), "run_cautions": cautions}))
}

fn level(v: CitationLevel) -> &'static str {
    match v {
        CitationLevel::Must => "must",
        CitationLevel::Should => "should",
        CitationLevel::May => "may",
        CitationLevel::ShouldNot => "should-not",
        CitationLevel::MustNot => "must-not",
    }
}
fn disjunctions(values: &[CitationDisjunction]) -> Vec<String> {
    values
        .iter()
        .map(|v| {
            v.targets
                .iter()
                .map(|t| match &t.namespace {
                    NamespaceMatch::Local => t.kind.clone(),
                    NamespaceMatch::Alias(a) => format!("{a}/{}", t.kind),
                    NamespaceMatch::Any => format!("*/{}", t.kind),
                })
                .collect::<Vec<_>>()
                .join(" | ")
        })
        .collect()
}

/// Every persisted setting, with derived engine caches excluded from the public
/// schema so #466 can change those caches independently (§FS-distribution.3.1).
fn schema(c: &Config) -> Value {
    let mut citations = serde_json::Map::new();
    citations.insert(
        "default".into(),
        json!(c.citations.global_default.map(level)),
    );
    for (kind, r) in &c.citations.per_kind {
        citations.insert(kind.clone(),json!({
        "default":r.default.map(level), "must":disjunctions(&r.must),"should":disjunctions(&r.should),
        "may":disjunctions(&r.may),"should_not":disjunctions(&r.should_not),"must_not":disjunctions(&r.must_not)}));
    }
    json!({"grund_config_version":1,"project_name":c.project_name,"project_description":c.project_description,
        "reference": {"marker":c.marker,"trigger":c.trigger,"strict":c.strict,
            "shorthand":c.shorthand.as_str(),"require_grounding":c.require_grounding,
            "grounding_level":c.grounding_level,"conversation":c.conversation,
            "lead_size_warning":c.lead_size_warning.as_ref().map(|s|json!({"max":s.max,"unit":s.unit.as_str()})),
            "inline_style":c.inline_style,"inline_note_suggested_lines":c.inline_note_suggested_lines,
            "inline_note_max_lines":c.inline_note_max_lines,"inline_note_max_columns":c.inline_note_max_columns,
            "inline_note_layout":c.inline_note_layout,"inline_note_layout_check":c.inline_note_layout_check,
            "warn_on_suggested":c.warn_on_suggested},
        "id":{"format":c.id_format,"section_separator":c.section_separator,
            "number_pattern":c.number_pattern,"slug_pattern":c.slug_pattern,
            "named_sections":c.named_sections,"section_heading_levels":c.section_heading_levels},
        "scan":{"include":c.include,"exclude":c.exclude,"extensions":c.extensions,
            "comment_prefixes":c.comment_prefixes,"docstring_python":c.docstring_python,
            "respect_gitignore":c.respect_gitignore},
        "output":{"format":c.output_format,"relative_paths":c.relative_paths,"color":"auto"},
        "fmt":{"exclude":c.fmt_exclude,"cross_refs":{"enabled":c.fmt_cross_refs_enabled,
            "anchor_format":c.cross_ref_anchor_format}},
        "workspace":{"members":c.workspace_members,"optional_members":c.workspace_optional_members,
            "include_root":c.workspace_include_root},
        "kinds":c.kinds.iter().map(|k|json!({"kind":k.kind,"folder":k.folder,"file":k.file,
            "title":k.title,"index":match &k.index {KindIndex::Default=>json!("README.md"),KindIndex::Disabled=>json!(false),KindIndex::Named(s)=>json!(s)},
            "citable":k.citable,"scan":k.scan,"require_grounding":k.require_grounding,
            "grounding_level":k.grounding_level,"values":k.values,"value_chapter":k.value_chapter,
            "rules":k.rules,"format":k.format,"resolve":k.resolve.map(|r|match r{KindResolution::Must=>"must",KindResolution::Should=>"should"}),"fetch":k.fetch})).collect::<Vec<_>>(),
        "citations":citations})
}
