//! Structured editor facts, preserving every field (§FS-lsp.1,
//! §REQ-one-answer.contract, §REQ-one-answer.proof.matrix).

use grund_core::{Finding, LspDeclaration, LspSnapshotWithCompletion};
use serde_json::{Value, json};

fn path(path: &std::path::Path) -> String {
    path.to_string_lossy().into_owned()
}

pub fn finding(f: &Finding) -> Value {
    json!({"severity":f.severity,"code":f.code,"path":f.path,"line":f.line,
        "column":f.column,"message":f.message,"authority":f.authority,
        "sites":f.sites.iter().map(|s|json!({"path":s.path,"line":s.line})).collect::<Vec<_>>()})
}

fn declaration(d: &LspDeclaration) -> Value {
    json!({"project":d.project,"path":path(&d.path),"display_path":d.display_path,
        "line":d.line,"column":d.column,"text":d.text,"query_id":d.query_id,
        "section_separator":d.section_separator,"hover_text":d.hover_text})
}

pub fn facts(loaded: &LspSnapshotWithCompletion) -> Value {
    let s = &loaded.metadata.snapshot;
    json!({"kind_titles":loaded.metadata.kind_titles,"snapshot":{
        "root":path(&s.root),"marker":s.marker,"trigger":s.trigger,"workspace":s.workspace,
        "report":{"errors":s.report.errors.iter().map(finding).collect::<Vec<_>>(),
            "warnings":s.report.warnings.iter().map(finding).collect::<Vec<_>>(),
            "suggestions":s.report.suggestions.iter().map(finding).collect::<Vec<_>>()},
        "run_warnings":s.run_warnings.iter().map(finding).collect::<Vec<_>>(),
        "declarations":s.declarations.iter().map(declaration).collect::<Vec<_>>(),
        "sections":s.sections.iter().map(declaration).collect::<Vec<_>>(),
        "finding_ranges":s.finding_ranges.iter().map(|r|json!({"code":r.code,
            "path":path(&r.path),"line":r.line,"column":r.column,"text":r.text})).collect::<Vec<_>>(),
        "stubs":s.stubs.iter().map(|r|json!({"project":r.project,"path":path(&r.path),
            "display_path":r.display_path,"line":r.line,"column":r.column,"text":r.text,
            "query_id":r.query_id,"section_separator":r.section_separator,
            "target_path":path(&r.target_path),"target_line":r.target_line})).collect::<Vec<_>>(),
        "citations":s.citations.iter().map(|r|json!({"project":r.project,"path":path(&r.path),
            "display_path":r.display_path,"line":r.line,"column":r.column,"text":r.text,
            "query_id":r.query_id,"declaration_query_id":r.declaration_query_id,
            "section_separator":r.section_separator,"target_path":r.target_path.as_deref().map(path),
            "target_line":r.target_line})).collect::<Vec<_>>(),
        "scanned_files":s.scanned_files.iter().map(|p|path(p)).collect::<Vec<_>>(),
        "scan_errors":s.scan_errors.iter().map(|e|json!({"path":e.path,
            "message":e.message})).collect::<Vec<_>>()},
        "readFiles":loaded.show.read_files().iter().map(|p|path(p)).collect::<Vec<_>>()})
}
