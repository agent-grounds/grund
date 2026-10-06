//! Independent frozen CLI check projection (§FS-distribution.3.0.3).

use serde_json::Value;

fn scalar(value: &Value) -> String {
    if let Value::String(s) = value {
        let mut out = String::from("\"");
        for c in s.chars() {
            match c {
                '\\' => out.push_str("\\\\"),
                '"' => out.push_str("\\\""),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
                c => out.push(c),
            }
        }
        out.push('"');
        out
    } else {
        value.to_string()
    }
}

/// Preserve CLI channel order, authority-last keys, and stream placement.
pub(super) fn project(data: &Value) -> (String, String) {
    let mut stdout = String::new();
    let mut stderr = String::new();
    if !data["failure"].is_null() {
        stderr.push_str(&format!(
            "error: {}\n",
            data["failure"]["message"].as_str().unwrap_or("")
        ));
        return (stdout, stderr);
    }
    if let Some(cautions) = data["run_cautions"].as_array() {
        for f in cautions {
            stderr.push_str(&format!(
                "warning: {}\n",
                f["message"].as_str().unwrap_or("")
            ));
        }
    }
    let report = &data["result"]["report"];
    let mut rows = Vec::new();
    for (channel, group) in [
        ("warning", "warnings"),
        ("error", "errors"),
        ("suggestion", "suggestions"),
    ] {
        if let Some(findings) = report[group].as_array() {
            rows.extend(findings.iter().map(|f| (channel, f)));
        }
    }
    rows.sort_by(|(_, a), (_, b)| {
        a["path"]
            .as_str()
            .cmp(&b["path"].as_str())
            .then_with(|| {
                a["line"]
                    .as_u64()
                    .unwrap_or(0)
                    .cmp(&b["line"].as_u64().unwrap_or(0))
            })
            .then_with(|| a["message"].as_str().cmp(&b["message"].as_str()))
    });
    for (channel, f) in rows {
        let tag = if channel == "suggestion" {
            "channel"
        } else {
            "severity"
        };
        let nullable = |key: &str| {
            if f[key].as_array().is_some_and(Vec::is_empty) {
                "null".into()
            } else {
                scalar(&f[key])
            }
        };
        let line = format!(
            "{{\"{tag}\":\"{channel}\",\"path\":{},\"line\":{},\"code\":{},\"message\":{},\"sites\":{},\"authority\":{}}}\n",
            scalar(&f["path"]),
            scalar(&f["line"]),
            scalar(&f["code"]),
            scalar(&f["message"]),
            if f["sites"].as_array().is_some_and(Vec::is_empty) {
                "null".into()
            } else {
                format!(
                    "[{}]",
                    f["sites"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|s| format!(
                            "{{\"path\":{},\"line\":{}}}",
                            scalar(&s["path"]),
                            scalar(&s["line"])
                        ))
                        .collect::<Vec<_>>()
                        .join(",")
                )
            },
            nullable("authority")
        );
        if f["line"].is_null() {
            stderr.push_str(&line);
        } else {
            stdout.push_str(&line);
        }
    }
    (stdout, stderr)
}
