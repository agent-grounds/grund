//! Enrollment helpers for the members of a value source, read by either reader
//! (§FS-values.2.2.1, §AR-scanner.2.1.3, §AR-scanner.3): duplicates, ownership
//! and component validity are written here once for both formats.

use std::collections::BTreeMap;
use std::path::Path;

use super::json::{JsonMember, JsonNode, JsonSpan, json_line_column};
use crate::config::Frame;
use crate::grammar::parse_id_arg;
use crate::model::{
    Catalog, Declaration, DeclarationSource, Id, InvalidValueSite, SectionInfo, ValueComponent,
    ValueComponentKind, ValueFormat, component_text_is_valid,
};

pub(super) fn enroll_json_member(
    frame: Frame<'_>,
    path: &Path,
    owners: &[&str],
    text: &str,
    member: JsonMember,
    format: ValueFormat,
    findings: &mut Catalog,
) {
    let (name, (_, array)) = (format.name(), format.collections());
    let parsed = parse_id_arg(&member.key.decoded, frame.grammar());
    if let JsonNode::Refused(span, reason) = &member.value {
        // §FS-values.2.2.3: refused at its own site, named by the key it is under.
        let id = parsed
            .ok()
            .and_then(|(id, section)| section.is_none().then_some(id));
        push_json_invalid(findings, id, path, text, *span, reason, format);
        return;
    }
    let Ok((id, None)) = parsed else {
        push_json_invalid(
            findings,
            None,
            path,
            text,
            member.key.span,
            &format!("{name} value key must be a full unqualified local ID"),
            format,
        );
        return;
    };
    if !owners.contains(&id.kind.as_str()) {
        let expected = owners
            .iter()
            .map(|owner| format!("`{owner}`"))
            .collect::<Vec<_>>()
            .join(" or ");
        push_json_invalid(
            findings,
            Some(id),
            path,
            text,
            member.key.span,
            &format!("{name} value key must belong to owning kind {expected}"),
            format,
        );
        return;
    }
    let member_line = json_line_column(text, member.key.span.start).0;
    let key_column = json_line_column(text, member.key.span.start).1;
    let member_slice = text[member.span.start..member.span.end].to_string();
    let mut declaration = Declaration {
        authored_id: Some(member.key.decoded.clone()),
        id: id.clone(),
        file: path.to_path_buf(),
        line: member_line,
        heading_level: 1,
        sections: BTreeMap::new(),
        duplicate_sections: Vec::new(),
        is_stub: false,
        defined_in: None,
        stub_resolution: None,
        e2e_case: None,
        title: None,
        body_start: member_line,
        body_end: json_line_column(text, member.span.end).0,
        body_has_content: false,
        source: DeclarationSource::Json {
            member_slice,
            key_column,
            key_text: text[member.key.span.start..member.key.span.end].to_string(),
            format,
        },
        value_valid: Some(true),
        front_matter: None,
    };
    let JsonNode::Array(elements, _) = member.value else {
        declaration.value_valid = Some(false);
        push_json_invalid(
            findings,
            Some(id.clone()),
            path,
            text,
            member.value.span(),
            &format!("{name} value member must be a nonempty {array}"),
            format,
        );
        findings
            .declarations
            .entry(id)
            .or_default()
            .push(declaration);
        return;
    };
    if elements.is_empty() {
        declaration.value_valid = Some(false);
        push_json_invalid(
            findings,
            Some(id.clone()),
            path,
            text,
            member.span,
            &format!("{name} value {array} must not be empty"),
            format,
        );
    }
    let mut refusal = None;
    for (index, element) in elements.into_iter().enumerate() {
        let span = element.span();
        let (line, column) = json_line_column(text, span.start);
        let source_slice = text[span.start..span.end].to_string();
        let value = match element {
            JsonNode::Number(decoded, _) => Some(ValueComponent {
                decoded,
                kind: ValueComponentKind::Number,
                source_slice,
                column,
            }),
            JsonNode::String(string) if component_text_is_valid(&string.decoded) => {
                Some(ValueComponent {
                    decoded: string.decoded,
                    kind: ValueComponentKind::String,
                    source_slice,
                    column,
                })
            }
            JsonNode::Refused(_, reason) => {
                refusal = Some(reason);
                None
            }
            _ => None,
        };
        if value.is_none() {
            declaration.value_valid = Some(false);
            let generic = format!(
                "{name} value component must be a number or a nonempty edge-unspaced string without backticks or control characters"
            );
            let reason = refusal.take().unwrap_or(generic);
            push_json_invalid(
                findings,
                Some(id.clone()),
                path,
                text,
                span,
                &reason,
                format,
            );
        }
        declaration.sections.insert(
            (index + 1).to_string(),
            SectionInfo {
                title: value
                    .as_ref()
                    .map(|value| value.decoded.clone())
                    .unwrap_or_default(),
                line,
                heading_level: 2,
                value,
                value_root: None,
                relative: false,
                front_matter: None,
            },
        );
    }
    findings
        .declarations
        .entry(id)
        .or_default()
        .push(declaration);
}

pub(super) fn push_json_invalid(
    findings: &mut Catalog,
    id: Option<Id>,
    path: &Path,
    text: &str,
    span: JsonSpan,
    message: &str,
    format: ValueFormat,
) {
    let (line, column) = json_line_column(text, span.start);
    findings.invalid_value_declarations.push(InvalidValueSite {
        id,
        file: path.to_path_buf(),
        line,
        column: Some(column),
        message: message.to_string(),
        source: DeclarationSource::Json {
            member_slice: text[span.start..span.end].to_string(),
            key_column: column,
            key_text: text[span.start..span.end].to_string(),
            format,
        },
        binding_namespace: None,
        binding_section: None,
    });
}
