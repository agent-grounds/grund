// What `grund list` prints (§FS-list.3): the summary rows, the catalog rows in
// text and NDJSON, and the size-mode rows, split from the command's argument
// handling in `cli_list.rs`.

fn render_list_summary(summaries: &[grund_core::ListSummary], workspace: bool, format: &str) {
    // §FS-workspace.8.3.4: the alias column is sized to the widest alias among
    // the rows being rendered, capped like `render_list_text`'s `id_width`.
    let alias_width = summaries
        .iter()
        .filter_map(|summary| summary.project.as_deref())
        .map(|project| project.chars().count())
        .max()
        .unwrap_or(0)
        .min(40);
    for summary in summaries {
        if workspace {
            let project = summary.project.as_deref().unwrap_or("");
            if format == "json" {
                println!(
                    "{{\"project\":\"{}\",\"kind\":\"{}\",\"title\":\"{}\",\"home\":\"{}\",\"count\":{}}}",
                    json_escape(project),
                    json_escape(&summary.kind),
                    json_escape(&summary.title),
                    json_escape(&summary.home),
                    summary.count
                );
            } else {
                println!(
                    "{project:<alias_width$}  {:<4}  {:>3}  {}",
                    summary.kind, summary.count, summary.home
                );
            }
        } else if format == "json" {
            println!(
                "{{\"kind\":\"{}\",\"title\":\"{}\",\"home\":\"{}\",\"count\":{}}}",
                json_escape(&summary.kind),
                json_escape(&summary.title),
                json_escape(&summary.home),
                summary.count
            );
        } else {
            println!(
                "{:<4}  {:>3}  {}",
                summary.kind, summary.count, summary.home
            );
        }
    }
}

fn render_list_entry_json(entry: &ListEntry) -> String {
    let project_field = entry
        .project
        .as_deref()
        .map(|project| format!("\"project\":\"{}\",", json_escape(project)))
        .unwrap_or_default();
    let value_roots = if entry.value_roots.is_empty() {
        String::new()
    } else {
        let roots = entry
            .value_roots
            .iter()
            .map(|root| {
                format!(
                    "{{\"id\":\"{}\",\"valid\":{}}}",
                    json_escape(&root.id),
                    root.valid
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(",\"value_roots\":[{roots}]")
    };
    let section_field = entry.section.as_deref().map(|section| format!("\"section\":\"{}\",", json_escape(section))).unwrap_or_default();
    format!(
        "{{{}\"id\":\"{}\",{}\"kind\":\"{}\",\"path\":\"{}\",\"line\":{},\"title\":{},\"stub\":{},\"defines\":{},\"refs\":{},\"duplicate\":{}{}}}",
        project_field,
        json_escape(&entry.id),
        section_field,
        json_escape(&entry.kind),
        json_escape(&entry.path),
        entry.line,
        entry
            .title
            .as_deref()
            .map(|title| format!("\"{}\"", json_escape(title)))
            .unwrap_or_else(|| "null".to_string()),
        entry.stub,
        entry
            .defines
            .as_deref()
            .map(|target| format!("\"{}\"", json_escape(target)))
            .unwrap_or_else(|| "null".to_string()),
        entry.refs,
        entry.duplicate,
        value_roots,
    )
}

fn render_list_text(entries: &[ListEntry]) {
    let id_width = entries
        .iter()
        .map(|entry| list_coordinate(entry).chars().count())
        .max()
        .unwrap_or(0)
        .min(40);
    for entry in entries {
        let location = format!("{}:{}", entry.path, entry.line);
        let mut note = if entry.stub {
            entry
                .defines
                .as_ref()
                .map(|target| format!("→ {target}"))
                .unwrap_or_default()
        } else {
            entry.title.clone().unwrap_or_default()
        };
        if entry.duplicate {
            if note.is_empty() {
                note = "(duplicate declaration — grund check)".to_string();
            } else {
                note.push_str("  (duplicate declaration — grund check)");
            }
        }
        if !entry.value_roots.is_empty() {
            let roots = entry
                .value_roots
                .iter()
                .map(|root| {
                    if root.valid {
                        root.id.clone()
                    } else {
                        format!("{} (invalid)", root.id)
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            if note.is_empty() {
                note = format!("[value roots: {roots}]");
            } else {
                note.push_str(&format!(" [value roots: {roots}]"));
            }
        }
        if note.is_empty() {
            println!("{:<id_width$}  {location}", list_coordinate(entry));
        } else {
            println!("{:<id_width$}  {location}  {note}", list_coordinate(entry));
        }
    }
}

fn list_coordinate(entry: &ListEntry) -> String {
    entry
        .section
        .as_deref()
        .map(|section| format!("{}{}{}", entry.id, entry.section_separator, section))
        .unwrap_or_else(|| entry.id.clone())
}

/// Render the owning project's exact declaration/section coordinate
/// (§FS-list.3.4.3, §FS-workspace.8.3.3).
fn list_size_coordinate(entry: &ListSizeEntry) -> String {
    match &entry.section {
        Some(section) => format!("{}{}{}", entry.id, entry.section_separator, section),
        None => entry.id.clone(),
    }
}

/// §FS-list.3.4.4 / §FS-output-shapes.5.2: size-mode NDJSON has its own fixed
/// metadata prefix and appends only selected lead/full pairs in caller order.
fn render_list_size_entry_json(entry: &ListSizeEntry) -> String {
    let project_field = entry
        .project
        .as_deref()
        .map(|project| format!("\"project\":\"{}\",", json_escape(project)))
        .unwrap_or_default();
    let section = entry
        .section
        .as_deref()
        .map(|section| format!("\"{}\"", json_escape(section)))
        .unwrap_or_else(|| "null".to_string());
    let defines = entry
        .defines
        .as_deref()
        .map(|target| format!("\"{}\"", json_escape(target)))
        .unwrap_or_else(|| "null".to_string());
    let mut json = format!(
        "{{{}\"id\":\"{}\",\"section\":{},\"kind\":\"{}\",\"path\":\"{}\",\"line\":{},\"stub\":{},\"defines\":{},\"duplicate\":{}",
        project_field,
        json_escape(&entry.id),
        section,
        json_escape(&entry.kind),
        json_escape(&entry.path),
        entry.line,
        entry.stub,
        defines,
        entry.duplicate,
    );
    for measurement in &entry.measurements {
        let unit = measurement.unit.as_str();
        let lead = measurement
            .lead
            .map(|value| value.to_string())
            .unwrap_or_else(|| "null".to_string());
        let full = measurement
            .full
            .map(|value| value.to_string())
            .unwrap_or_else(|| "null".to_string());
        json.push_str(&format!(
            ",\"lead_{unit}\":{lead},\"full_{unit}\":{full}"
        ));
    }
    json.push('}');
    json
}

/// Headerless size rows with ordinary list alignment and site-local suffixes
/// (§FS-list.3.4.3).
fn render_list_size_text(entries: &[ListSizeEntry]) {
    let coordinates = entries.iter().map(list_size_coordinate).collect::<Vec<_>>();
    let width = coordinates
        .iter()
        .map(|coordinate| coordinate.chars().count())
        .max()
        .unwrap_or(0)
        .min(40);
    for (entry, coordinate) in entries.iter().zip(coordinates) {
        let location = format!("{}:{}", entry.path, entry.line);
        let sizes = entry
            .measurements
            .iter()
            .map(|measurement| {
                let unit = measurement.unit.as_str();
                match (measurement.lead, measurement.full) {
                    (Some(lead), Some(full)) => format!("{unit}={lead}/{full}"),
                    _ => format!("{unit}=-/-"),
                }
            })
            .collect::<Vec<_>>()
            .join(" ");
        let suffix = if entry.stub {
            format!(
                "  (broken stub → {})",
                entry.defines.as_deref().unwrap_or("")
            )
        } else if entry.duplicate {
            "  (duplicate — site-local)".to_string()
        } else {
            String::new()
        };
        println!("{coordinate:<width$}  {location}  {sizes}{suffix}");
    }
}
