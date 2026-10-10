//! Schema-keyed configuration snapshots (§FS-distribution.3.3.1).

use super::embedding::EmbeddingRequest;
use super::embedding_data::Data;
use super::embedding_failure::error_data;
use crate::*;
// The façade itself, not the root's deprecated alias of it (§AR-config.5).
use crate::config::Config;
use serde_json::{Value, json};
use std::path::Path;

/// Preserve discovery cautions before workspace validation can refuse a member
/// (§FS-distribution.3.2.1, §FS-distribution.3.1). Supported Rust signatures stay intact.
pub(super) fn load(path: &Path, validate: bool) -> (Vec<Finding>, anyhow::Result<Config>) {
    let discovered = match effective_config(path) {
        Ok(config) => config,
        Err(error) => return (Vec::new(), Err(error)),
    };
    let earlier = cautions(&discovered);
    if !validate {
        return (earlier, Ok(discovered));
    }
    // §FS-distribution.3.2.3.1: refuse unsafe aliases while retaining discovery cautions.
    if let Err(error) = crate::workspace::preflight_embedding_paths(path) {
        return (earlier, Err(error));
    }
    let result = validate_config(path);
    let cautions = result.as_ref().map(cautions).unwrap_or(earlier);
    (cautions, result)
}

fn cautions(config: &Config) -> Vec<Finding> {
    let mut cautions = config_run_warnings(config);
    cautions.extend(
        super::config_findings::config_diagnostics(config).map(|d| Finding {
            severity: "warning",
            code: d.code,
            path: d.path.map(|p| crate::config::display_path(config, &p)),
            line: d.line,
            column: d.column,
            message: d.message,
            sites: Vec::new(),
            authority: Vec::new(),
        }),
    );
    cautions
}

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
    v.as_str()
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
pub(super) fn schema(c: &Config) -> Value {
    let mut citations = serde_json::Map::new();
    citations.insert(
        "default".into(),
        json!(c.citations.global_default.map(level)),
    );
    for (kind, r) in &c.citations.per_kind {
        let mut rules = json!({
        "default":r.default.map(level), "must":disjunctions(&r.must),"should":disjunctions(&r.should),
        "may":disjunctions(&r.may),"should_not":disjunctions(&r.should_not),"must_not":disjunctions(&r.must_not)});
        // §FS-config-v2.rules.citations: v2's warning lists, present only where
        // written, so a v1 config's schema keeps its bytes.
        for (key, list) in [("warn", &r.warn), ("warn_not", &r.warn_not)] {
            if !list.is_empty() {
                rules[key] = json!(disjunctions(list));
            }
        }
        citations.insert(kind.clone(), rules);
    }
    json!({"grund_config_version":c.project().version,"project_name":c.project_name,"project_description":c.project_description,
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
