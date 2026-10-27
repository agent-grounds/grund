/// One line of the schema table (§FS-schema.8): the cells before the last
/// column, each with the column it starts at, and the last cell where the line has one.
struct SchemaTableLine {
    cells: Vec<(usize, String)>,
    last: Option<String>,
}

impl SchemaTableLine {
    /// Where the widest cell before the last column ends.
    fn end(&self) -> usize {
        self.cells
            .iter()
            .map(|(at, text)| at + text.chars().count())
            .max()
            .unwrap_or(0)
    }

    /// The line padded with spaces, the last cell at `last_at`, nothing trailing.
    fn render(&self, last_at: usize) -> String {
        let mut out = String::new();
        let mut put = |at: usize, text: &str| {
            // A cell wider than its column is still two spaces from the next.
            let width = out.chars().count();
            let pad = match at.saturating_sub(width) {
                pad if width > 0 && pad < 2 => 2,
                pad => pad,
            };
            out.push_str(&" ".repeat(pad));
            out.push_str(text);
        };
        for (at, text) in &self.cells {
            put(*at, text);
        }
        if let Some(last) = &self.last {
            put(last_at, last);
        }
        out
    }
}

/// §FS-schema.11: the table prints a control character as a TOML basic string
/// writes its escape, so no configured name can break a line.
fn schema_table_text(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        match ch {
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other if other.is_control() => out.push_str(&format!("\\u{:04X}", other as u32)),
            other => out.push(other),
        }
    }
    out
}

/// §FS-schema.8: the default format — the header, `kinds and places`, `citations`
/// with the grounding badges, the unscanned rows, the legend, and at a workspace
/// root the members line (§FS-schema.2.1). Columns are two spaces apart, and the
/// last column starts at one offset in both sections.
fn schema_table(view: &SchemaView) -> String {
    let text = |raw: &str| schema_table_text(raw);
    let width = |raw: &str| text(raw).chars().count();
    let name_w = view.nodes.iter().map(|n| width(&n.name)).max().unwrap_or(0);
    let strength_w = view
        .edges
        .iter()
        .map(|e| schema_strength(e.level).len())
        .chain(view.nodes.iter().filter_map(|n| n.default).map(|l| schema_strength(l).len()))
        .max()
        .unwrap_or(0);
    let (class_at, strength_at) = (2 + name_w + 2, 2 + name_w + 2);
    let target_at = strength_at + strength_w + 2;
    let field_at = class_at + 2;

    let mut kinds = Vec::new();
    for node in &view.nodes {
        kinds.push(SchemaTableLine {
            cells: vec![(2, text(&node.name)), (class_at, schema_class_word(node.class).to_string())],
            last: Some(text(&schema_places(node))),
        });
        for slot in &node.slots {
            let (name, facts) = schema_slot(slot);
            kinds.push(SchemaTableLine {
                cells: vec![(field_at, format!("field {}", text(&name)))],
                last: Some(text(&facts)),
            });
            // §FS-fields.front-matter.record: each key two columns deeper.
            for (key, facts) in slot.key_lines() {
                kinds.push(SchemaTableLine {
                    cells: vec![(field_at + 2, format!("key {}", text(&key)))],
                    last: Some(text(&facts)),
                });
            }
        }
        if let Some(closed) = node.closed {
            kinds.push(SchemaTableLine {
                cells: vec![(field_at, format!("closed {}", closed.as_str()))],
                last: None,
            });
        }
    }

    // §FS-schema.8: no badge anywhere, no badge column.
    let badged = view.nodes.iter().any(|n| !n.grounding.is_empty());
    let mut citations = vec![SchemaTableLine {
        cells: vec![(0, "citations".to_string())],
        last: badged.then(|| "grounded".to_string()),
    }];
    if let Some(line) = schema_default_line(view) {
        citations.push(SchemaTableLine { cells: vec![(2, line)], last: None });
    }
    let scanned = view.nodes.iter().enumerate().filter(|(_, n)| n.class != NodeClass::Unscanned);
    for (index, node) in scanned {
        let mut rows: Vec<(String, String)> = view
            .edges
            .iter()
            .filter(|edge| edge.subject == index)
            .map(|edge| (schema_strength(edge.level), schema_table_target(edge)))
            .collect();
        if let Some(level) = node.default {
            rows.push((schema_strength(level), "everything else".to_string()));
        }
        let badge = schema_badge(node).map(|badge| text(&badge));
        if rows.is_empty() {
            citations.push(SchemaTableLine {
                cells: vec![(2, text(&node.name)), (strength_at, "—".to_string())],
                last: badge,
            });
            continue;
        }
        for (position, (strength, target)) in rows.into_iter().enumerate() {
            let name = if position == 0 { text(&node.name) } else { String::new() };
            citations.push(SchemaTableLine {
                cells: vec![(2, name), (strength_at, strength), (target_at, text(&target))],
                last: if position == 0 { badge.clone() } else { None },
            });
        }
    }
    for node in view.nodes.iter().filter(|n| n.class == NodeClass::Unscanned) {
        citations.push(SchemaTableLine {
            cells: vec![
                (2, text(&node.name)),
                (class_at, "not scanned: cites nothing and is never checked".to_string()),
            ],
            last: None,
        });
    }

    let last_at = kinds
        .iter()
        .chain(&citations)
        .filter(|line| line.last.is_some())
        .map(SchemaTableLine::end)
        .max()
        .unwrap_or(0)
        + 2;
    let mut out = String::new();
    out.push_str(&text(&schema_header(view)));
    if let Some(note) = &view.scaffold {
        out.push('\n');
        out.push_str(&text(&schema_scaffold_note(note)));
    }
    out.push_str("\n\nkinds and places\n");
    for line in kinds.iter() {
        out.push_str(&line.render(last_at));
        out.push('\n');
    }
    out.push('\n');
    for line in citations.iter() {
        out.push_str(&line.render(last_at));
        out.push('\n');
    }
    out.push('\n');
    for line in SCHEMA_LEGEND {
        out.push_str(line);
        out.push('\n');
    }
    if let Some(line) = schema_uncited_line(view) {
        out.push_str(&line);
        out.push('\n');
    }
    if let Some(members) = schema_members_line(view) {
        out.push('\n');
        out.push_str(&text(&members));
        out.push('\n');
    }
    out
}

/// §FS-schema.8.2: the no-config note — what the defaults are, and the version
/// and added kinds of the config `grund init --scaffold` writes, each kind's places
/// spelled as the `kinds and places` section spells them.
fn schema_scaffold_note(note: &ScaffoldNote) -> String {
    let mut line = format!(
        "note: these are the built-in defaults a tree with no grund.toml is held to; \
         grund init --scaffold writes a version-{} grund.toml",
        note.version
    );
    if !note.added.is_empty() {
        let added: Vec<String> = note
            .added
            .iter()
            .map(|node| format!("{} in {}", node.name, schema_places(node)))
            .collect();
        line.push_str(" that also declares ");
        line.push_str(&added.join(", "));
    }
    line
}

/// §FS-schema.5.1, §FS-schema.5.3: a direction's target as written — an any-of
/// entry as one cell, a kind citing its own kind marked `(itself)`.
fn schema_table_target(edge: &SchemaEdge) -> String {
    let labels: Vec<&str> = edge.targets.iter().map(|t| t.label.as_str()).collect();
    if edge.is_self() {
        return format!("{} (itself)", labels.join("|"));
    }
    labels.join("|")
}
