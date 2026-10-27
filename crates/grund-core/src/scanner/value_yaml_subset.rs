//! The value subset of YAML, a pass over the span tree (§FS-values.2.2.3,
//! §DA-yaml-value-reader.2.2): it reads a YAML value source's tree into the node
//! shape the JSON reader gives, so both formats enroll through one path, and
//! names every construct the subset refuses at its own site.

use super::json::{JsonMember, JsonNode, JsonSpan, JsonStringValue, decode_json_string};
use super::yaml::{ScalarStyle, YamlKind, YamlNode, YamlSpan, YamlStream};
use crate::model::JSON_NUMBER_RE;

/// The source's root as the value pass reads it: a mapping becomes an object of
/// members, anything else keeps its span and is refused or shaped wrong there.
pub(super) fn yaml_value_root(text: &str, stream: &YamlStream) -> JsonNode {
    let bom = if text.starts_with('\u{feff}') { 3 } else { 0 };
    let mut documents = stream.documents.iter();
    let Some(first) = documents.next() else {
        return JsonNode::Null(JsonSpan {
            start: bom,
            end: bom,
        });
    };
    if let Some(second) = documents.next() {
        let start = second.explicit_start.unwrap_or(second.root.span.start);
        return refused(
            JsonSpan {
                start,
                end: second.root.span.end.max(start),
            },
            "a second document",
        );
    }
    let root = &first.root;
    if let Some(refusal) = node_refusal(root) {
        return refusal;
    }
    let YamlKind::Mapping { entries, .. } = &root.kind else {
        return JsonNode::Null(span(root.span));
    };
    let members = entries
        .iter()
        .map(|(key, value)| member(text, key, value))
        .collect();
    JsonNode::Object(members, span(root.span))
}

fn member(text: &str, key: &YamlNode, value: &YamlNode) -> JsonMember {
    let raw = &text[key.span.start..key.span.end];
    let mut node = match key_refusal(text, key) {
        Some(refusal) => refusal,
        None => member_value(text, value),
    };
    let decoded = match scalar(text, key) {
        JsonNode::String(string) => string.decoded,
        JsonNode::Number(number, _) => number,
        _ => raw.to_string(),
    };
    if !matches!(node, JsonNode::Refused(_, _)) && decoded == "<<" && is_plain(key) {
        node = refused(span(key.span), "a merge key");
    }
    JsonMember {
        key: JsonStringValue {
            decoded,
            span: span(key.span),
        },
        span: JsonSpan {
            start: key.span.start,
            end: value.span.end.max(key.span.end),
        },
        value: node,
    }
}

/// A key the subset refuses, reported in its member's value's place so the
/// enrollment names it at the key.
fn key_refusal(text: &str, key: &YamlNode) -> Option<JsonNode> {
    if let Some(refusal) = node_refusal(key) {
        return Some(refusal);
    }
    match &key.kind {
        YamlKind::Scalar { .. } => match scalar(text, key) {
            JsonNode::Refused(span, reason) => Some(JsonNode::Refused(span, reason)),
            _ => None,
        },
        _ => Some(refused(span(key.span), "a nested collection")),
    }
}

fn member_value(text: &str, value: &YamlNode) -> JsonNode {
    if let Some(refusal) = node_refusal(value) {
        return refusal;
    }
    match &value.kind {
        YamlKind::Sequence { items, .. } => JsonNode::Array(
            items.iter().map(|item| element(text, item)).collect(),
            span(value.span),
        ),
        // A mapping is the wrong shape for a member, not a refused construct.
        YamlKind::Mapping { .. } => JsonNode::Object(Vec::new(), span(value.span)),
        YamlKind::Scalar { .. } => match scalar(text, value) {
            refusal @ JsonNode::Refused(_, _) => refusal,
            _ => JsonNode::Null(span(value.span)),
        },
        YamlKind::Alias => refused(span(value.span), "an alias"),
    }
}

fn element(text: &str, item: &YamlNode) -> JsonNode {
    if let Some(refusal) = node_refusal(item) {
        return refusal;
    }
    match &item.kind {
        YamlKind::Scalar { .. } => scalar(text, item),
        _ => refused(span(item.span), "a nested collection"),
    }
}

/// An alias, an anchor or a tag on any node (§FS-values.2.2.3).
fn node_refusal(node: &YamlNode) -> Option<JsonNode> {
    let construct = if matches!(node.kind, YamlKind::Alias) {
        "an alias"
    } else if node.anchored {
        "an anchor"
    } else if node.tag.is_some() {
        "a tag"
    } else {
        return None;
    };
    Some(refused(span(node.span), construct))
}

/// A scalar's kind from how it is written, never from YAML's type resolution
/// (§FS-values.2.2.3): quoted is a string, plain is a number exactly when it is
/// JSON number grammar, and plain `true`, `false`, `null`, `~` and an empty
/// element are invalid.
fn scalar(text: &str, node: &YamlNode) -> JsonNode {
    let YamlKind::Scalar {
        style, single_line, ..
    } = &node.kind
    else {
        return refused(span(node.span), "a nested collection");
    };
    let at = span(node.span);
    let raw = &text[node.span.start..node.span.end];
    let string = |decoded: String| JsonNode::String(JsonStringValue { decoded, span: at });
    match style {
        ScalarStyle::Literal | ScalarStyle::Folded => refused(at, "a block scalar"),
        _ if !single_line => refused(at, "a multi-line scalar"),
        ScalarStyle::Plain => match raw {
            "" | "null" | "~" => JsonNode::Null(at),
            "true" | "false" => JsonNode::Bool(at),
            _ if JSON_NUMBER_RE.is_match(raw) => JsonNode::Number(raw.to_string(), at),
            _ => string(raw.to_string()),
        },
        ScalarStyle::SingleQuoted => string(raw[1..raw.len() - 1].replace("''", "'")),
        ScalarStyle::DoubleQuoted => match yaml_only_escape(raw) {
            Some(escape) => refused(at, &format!("the escape `{escape}`")),
            // A raw control character JSON forbids is no valid component either.
            None => decode_json_string(raw).map_or(JsonNode::InvalidScalar(at), string),
        },
    }
}

/// The first escape in a double-quoted scalar that JSON does not have.
fn yaml_only_escape(raw: &str) -> Option<String> {
    let mut chars = raw.char_indices();
    while let Some((index, c)) = chars.next() {
        if c != '\\' {
            continue;
        }
        let (_, escaped) = chars.next()?;
        let hex = raw.get(index + 2..index + 6);
        let json = match escaped {
            '"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't' => true,
            'u' => hex.is_some_and(|hex| hex.bytes().all(|byte| byte.is_ascii_hexdigit())),
            _ => false,
        };
        if !json {
            return Some(format!("\\{escaped}"));
        }
    }
    None
}

fn is_plain(node: &YamlNode) -> bool {
    matches!(
        node.kind,
        YamlKind::Scalar {
            style: ScalarStyle::Plain,
            ..
        }
    )
}

fn refused(span: JsonSpan, construct: &str) -> JsonNode {
    JsonNode::Refused(span, format!("a YAML value source refuses {construct}"))
}

fn span(span: YamlSpan) -> JsonSpan {
    JsonSpan {
        start: span.start,
        end: span.end,
    }
}
