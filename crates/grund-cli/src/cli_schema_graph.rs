/// The two graph printers of `grund schema` (§FS-schema.9, §FS-schema.10). They
/// share one walk over the view and differ in syntax alone, so the Mermaid and
/// DOT sources cannot carry different facts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SchemaGraph {
    Mermaid,
    Dot,
}

/// §FS-schema.9: `--format mermaid`, a `flowchart LR`.
fn schema_mermaid(view: &SchemaView) -> String {
    schema_graph(view, SchemaGraph::Mermaid)
}

/// §FS-schema.10: `--format dot`, one `digraph schema`.
fn schema_dot(view: &SchemaView) -> String {
    schema_graph(view, SchemaGraph::Dot)
}

/// §FS-schema.11: one label line escaped for the printer it reaches. Control
/// characters go through the table's escape first, so no line can end early.
fn schema_graph_text(graph: SchemaGraph, raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        match (graph, ch) {
            (SchemaGraph::Mermaid, '"') => out.push_str("#quot;"),
            (SchemaGraph::Mermaid, '<') => out.push_str("#lt;"),
            (SchemaGraph::Mermaid, '>') => out.push_str("#gt;"),
            (SchemaGraph::Mermaid, '&') => out.push_str("#amp;"),
            (SchemaGraph::Dot, '"') => out.push_str("\\\""),
            (SchemaGraph::Dot, '\\') => out.push_str("\\\\"),
            (SchemaGraph::Dot, '\n') => out.push_str("\\n"),
            (SchemaGraph::Dot, other) if other.is_control() => {
                out.push_str(&schema_table_text(&other.to_string()).replace('\\', "\\\\"));
            }
            (SchemaGraph::Mermaid, other) if other.is_control() => {
                out.push_str(&schema_table_text(&other.to_string()));
            }
            (_, other) => out.push(other),
        }
    }
    out
}

/// §FS-schema.9, §FS-schema.10: a node's label lines — its name, class word,
/// places, field lines, `closed`, its own `default`, and its badge — each escaped
/// and joined by the printer's line break.
fn schema_graph_label(graph: SchemaGraph, node: &SchemaNode) -> String {
    let mut lines = vec![
        node.name.clone(),
        schema_class_word(node.class).to_string(),
        schema_places(node),
    ];
    for slot in &node.slots {
        let (name, facts) = schema_slot(slot);
        lines.push(format!("field {name}: {facts}"));
        // §FS-fields.front-matter.record: its keys follow their field's line.
        for (key, facts) in slot.key_lines() {
            lines.push(format!("key {key}: {facts}"));
        }
    }
    if let Some(closed) = node.closed {
        lines.push(format!("closed {}", closed.as_str()));
    }
    if let Some(level) = node.default {
        lines.push(format!("everything else: {}", schema_strength(level)));
    }
    if let Some(badge) = schema_badge(node) {
        lines.push(format!("grounded: {badge}"));
    }
    let escaped: Vec<String> = lines.iter().map(|line| schema_graph_text(graph, line)).collect();
    escaped.join(match graph {
        SchemaGraph::Mermaid => "<br/>",
        SchemaGraph::Dot => "\\n",
    })
}

/// §FS-schema.5.2: the line style a strength is drawn with, besides its word.
fn schema_graph_style(level: CitationLevel) -> (&'static str, bool) {
    match level {
        CitationLevel::Must => ("bold", false),
        CitationLevel::Warn => ("solid", false),
        CitationLevel::Should => ("dashed", false),
        CitationLevel::May => ("dotted", false),
        CitationLevel::MustNot => ("bold", true),
        CitationLevel::WarnNot => ("solid", true),
        CitationLevel::ShouldNot => ("dashed", true),
    }
}

/// One Mermaid or DOT edge from `from` to `to`: `label` where the edge carries
/// the strength's word, `head` false for a subject's line into its junction.
fn schema_graph_edge(
    graph: SchemaGraph,
    from: &str,
    to: &str,
    level: CitationLevel,
    label: bool,
    head: bool,
) -> String {
    let (style, tee) = schema_graph_style(level);
    let word = schema_strength(level);
    match graph {
        SchemaGraph::Dot => {
            let mut attrs = Vec::new();
            if label {
                attrs.push(format!("label=\"{word}\""));
            }
            attrs.push(format!("style={style}"));
            if !head {
                attrs.push("arrowhead=none".to_string());
            } else if tee {
                attrs.push("arrowhead=tee".to_string());
            }
            format!("  {from} -> {to} [{}];", attrs.join(", "))
        }
        SchemaGraph::Mermaid => {
            // §FS-schema.5.2: thick `==`, solid `--`, dotted `-.`; a prohibition's head is `x`.
            let (open, close, bare) = match style {
                "bold" => ("==", "==", "=="),
                "solid" => ("--", "--", "--"),
                _ => ("-.", ".-", "-.-"),
            };
            let tip = match (head, tee) {
                (false, _) => "",
                (true, true) => "x",
                (true, false) => ">",
            };
            match (label, head) {
                (true, false) if style == "bold" => format!("  {from} {open} {word} === {to}"),
                (true, false) if style == "solid" => format!("  {from} {open} {word} --- {to}"),
                (true, _) => format!("  {from} {open} {word} {close}{tip} {to}"),
                (false, _) => format!("  {from} {bare}{tip} {to}"),
            }
        }
    }
}

/// §FS-schema.9, §FS-schema.10: the comments, the ruled nodes, the junctions,
/// the unruled box, and the edges in view order, each junction's arrows right
/// after the first edge into it. Identifiers are generated, never user text: `n`
/// with the row's position, `j` with the junction's, and `x` for a target in
/// another namespace, which no row of this project draws.
fn schema_graph(view: &SchemaView, graph: SchemaGraph) -> String {
    let mut out = String::new();
    let mut comments = vec![
        schema_header(view),
        SCHEMA_LEGEND[0].to_string(),
        SCHEMA_LEGEND[1].to_string(),
        SCHEMA_GRAPH_ANY_OF.to_string(),
    ];
    comments.extend(schema_default_line(view));
    comments.extend(schema_members_line(view));
    let comment = match graph {
        SchemaGraph::Mermaid => "%%",
        SchemaGraph::Dot => "//",
    };
    out.push_str(match graph {
        SchemaGraph::Mermaid => "flowchart LR\n",
        SchemaGraph::Dot => "digraph schema {\n",
    });
    for line in &comments {
        out.push_str(&format!("  {comment} {}\n", schema_table_text(line)));
    }
    match graph {
        SchemaGraph::Mermaid => {
            out.push_str("  classDef place stroke-dasharray:5 5\n");
            out.push_str("  classDef unscanned stroke-dasharray:2 2\n");
        }
        SchemaGraph::Dot => out.push_str("  rankdir=LR;\n  node [shape=box];\n"),
    }
    let node_line = |index: usize, node: &SchemaNode, indent: &str| {
        let label = schema_graph_label(graph, node);
        match graph {
            SchemaGraph::Mermaid => format!("{indent}n{index}[\"{label}\"]\n"),
            SchemaGraph::Dot => {
                let style = match node.class {
                    NodeClass::Place => ", style=dashed",
                    NodeClass::Unscanned => ", style=dotted",
                    _ => "",
                };
                format!("{indent}n{index} [label=\"{label}\"{style}];\n")
            }
        }
    };
    for (index, node) in view.nodes.iter().enumerate().filter(|(_, n)| n.ruled) {
        out.push_str(&node_line(index, node, "  "));
    }
    for index in 0..view.junctions.len() {
        out.push_str(&match graph {
            SchemaGraph::Mermaid => format!("  j{index}((\"any of\"))\n"),
            SchemaGraph::Dot => format!("  j{index} [label=\"any of\", shape=circle];\n"),
        });
    }
    let mut foreign: Vec<&str> = Vec::new();
    let targets = view.edges.iter().flat_map(|e| &e.targets);
    for target in targets.filter(|t| t.node.is_none()) {
        if !foreign.contains(&target.label.as_str()) {
            foreign.push(&target.label);
        }
    }
    for (index, label) in foreign.iter().enumerate() {
        let label = schema_graph_text(graph, label);
        out.push_str(&match graph {
            SchemaGraph::Mermaid => format!("  x{index}[\"{label}\"]\n"),
            SchemaGraph::Dot => format!("  x{index} [label=\"{label}\"];\n"),
        });
    }
    // §FS-schema.4.2: one box for every node no direction rule names.
    if view.nodes.iter().any(|n| !n.ruled) {
        out.push_str(match graph {
            SchemaGraph::Mermaid => "  subgraph unruled[\"no direction rule names these\"]\n",
            SchemaGraph::Dot => {
                "  subgraph cluster_unruled {\n    label=\"no direction rule names these\";\n"
            }
        });
        for (index, node) in view.nodes.iter().enumerate().filter(|(_, n)| !n.ruled) {
            out.push_str(&node_line(index, node, "    "));
        }
        out.push_str(match graph {
            SchemaGraph::Mermaid => "  end\n",
            SchemaGraph::Dot => "  }\n",
        });
    }
    if graph == SchemaGraph::Mermaid {
        for (class, name) in [(NodeClass::Place, "place"), (NodeClass::Unscanned, "unscanned")] {
            let ids: Vec<String> = (view.nodes.iter().enumerate())
                .filter(|(_, n)| n.class == class)
                .map(|(index, _)| format!("n{index}"))
                .collect();
            if !ids.is_empty() {
                out.push_str(&format!("  class {} {name}\n", ids.join(",")));
            }
        }
    }
    let target_id = |target: &EdgeTarget| match target.node {
        Some(index) => format!("n{index}"),
        None => {
            let position = foreign.iter().position(|label| *label == target.label);
            format!("x{}", position.unwrap_or_default())
        }
    };
    let mut fanned = vec![false; view.junctions.len()];
    for edge in &view.edges {
        let from = format!("n{}", edge.subject);
        let line = |to: &str, label, head| schema_graph_edge(graph, &from, to, edge.level, label, head);
        let Some(junction) = edge.junction else {
            let to = edge.targets.first().map(&target_id).unwrap_or_default();
            out.push_str(&line(&to, true, true));
            out.push('\n');
            continue;
        };
        out.push_str(&line(&format!("j{junction}"), true, false));
        out.push('\n');
        if !std::mem::replace(&mut fanned[junction], true) {
            let id = format!("j{junction}");
            for target in &view.junctions[junction].targets {
                let to = target_id(target);
                out.push_str(&schema_graph_edge(graph, &id, &to, edge.level, false, true));
                out.push('\n');
            }
        }
    }
    if graph == SchemaGraph::Dot {
        out.push_str("}\n");
    }
    out
}
