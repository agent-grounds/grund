/// Test module: the `grund schema` printers over a hand-built view (§FS-schema) —
/// escaping for each printer (§FS-schema.11), the prohibition heads and the
/// junction lines (§FS-schema.5.1, §FS-schema.5.2), and the table's columns where
/// a strength word is wider than `should` (§FS-schema.8).
#[cfg(test)]
mod tests_schema {
    use super::*;
    use grund_core::{BadgeRung, Junction, Strength};

    fn node(name: &str, class: NodeClass, places: Vec<Extent>) -> SchemaNode {
        SchemaNode {
            name: name.to_string(),
            class,
            places,
            slots: Vec::new(),
            closed: None,
            grounding: Vec::new(),
            default: None,
            ruled: true,
        }
    }

    fn target(label: &str, node: usize) -> EdgeTarget {
        EdgeTarget {
            label: label.to_string(),
            node: Some(node),
        }
    }

    fn edge(subject: usize, level: CitationLevel, targets: Vec<EdgeTarget>) -> SchemaEdge {
        SchemaEdge {
            subject,
            level,
            targets,
            junction: None,
        }
    }

    fn view(nodes: Vec<SchemaNode>, edges: Vec<SchemaEdge>) -> SchemaView {
        SchemaView {
            project: "p".to_string(),
            config_file: "grund.toml".to_string(),
            version: 2,
            nodes,
            junctions: Vec::new(),
            edges,
            default: None,
            members: Vec::new(),
            scaffold: None,
        }
    }

    /// A place whose folder name carries every character a printer must escape.
    fn hostile() -> SchemaView {
        let place = Extent::Folder("a\"b<c>&d\\e\nf".to_string());
        let mut place = node("test", NodeClass::Place, vec![place]);
        place.ruled = false;
        view(vec![place], Vec::new())
    }

    /// §FS-schema.11: DOT labels use DOT string escapes.
    #[test]
    fn dot_escapes_quotes_backslashes_and_newlines() {
        let dot = schema_dot(&hostile());
        assert!(
            dot.contains(r#"label="test\nplace\na\"b<c>&d\\e\nf/""#),
            "{dot}"
        );
    }

    /// §FS-schema.11: Mermaid labels are quoted, `"`, `<`, `>` and `&` as entities,
    /// and a newline never ends the node's line.
    #[test]
    fn mermaid_escapes_quotes_angles_and_ampersands() {
        let mermaid = schema_mermaid(&hostile());
        assert!(
            mermaid.contains(r#"n0["test<br/>place<br/>a#quot;b#lt;c#gt;#amp;d\e\nf/"]"#),
            "{mermaid}"
        );
    }

    /// §FS-schema.11: the table prints a control character as its escape, so a
    /// configured name cannot break a line.
    #[test]
    fn the_table_escapes_control_characters() {
        let table = schema_table(&hostile());
        assert!(table.contains("a\"b<c>&d\\e\\nf/"), "{table}");
        assert!(!table.contains("e\nf"), "{table}");
        assert_eq!(schema_table_text("a\tb\u{1}"), "a\\tb\\u0001");
    }

    /// §FS-schema.5.2: a prohibition ends in a tee in DOT and an `x` in Mermaid,
    /// in its strength's line style, and carries its word with a space.
    #[test]
    fn prohibitions_end_in_a_tee_or_a_cross() {
        let nodes = vec![
            node("FS", NodeClass::Kind, vec![Extent::Folder("fs".into())]),
            node("AR", NodeClass::Kind, vec![Extent::Folder("ar".into())]),
        ];
        let edges = vec![
            edge(0, CitationLevel::MustNot, vec![target("AR", 1)]),
            edge(0, CitationLevel::WarnNot, vec![target("AR", 1)]),
            edge(0, CitationLevel::ShouldNot, vec![target("AR", 1)]),
        ];
        let view = view(nodes, edges);
        let dot = schema_dot(&view);
        assert!(dot.contains(r#"n0 -> n1 [label="must not", style=bold, arrowhead=tee];"#));
        assert!(dot.contains(r#"n0 -> n1 [label="warn not", style=solid, arrowhead=tee];"#));
        assert!(dot.contains(r#"n0 -> n1 [label="should not", style=dashed, arrowhead=tee];"#));
        let mermaid = schema_mermaid(&view);
        assert!(mermaid.contains("n0 == must not ==x n1"), "{mermaid}");
        assert!(mermaid.contains("n0 -- warn not --x n1"), "{mermaid}");
        assert!(mermaid.contains("n0 -. should not .-x n1"), "{mermaid}");
        let table = schema_table(&view);
        assert!(table.contains("  FS  must not    AR\n"), "{table}");
        assert!(table.contains("      should not  AR\n"), "{table}");
    }

    /// §FS-schema.9, §FS-schema.10: a target outside the project is a generated
    /// `x<i>` node labelled as written.
    #[test]
    fn a_target_outside_the_project_is_an_x_node() {
        let nodes = vec![node("FS", NodeClass::Kind, vec![Extent::Folder("fs".into())])];
        let other = EdgeTarget {
            label: "other/FS".to_string(),
            node: None,
        };
        let view = view(nodes, vec![edge(0, CitationLevel::ShouldNot, vec![other])]);
        let dot = schema_dot(&view);
        assert!(dot.contains("  x0 [label=\"other/FS\"];\n"), "{dot}");
        assert!(dot.contains(r#"n0 -> x0 [label="should not", style=dashed, arrowhead=tee];"#));
        let mermaid = schema_mermaid(&view);
        assert!(mermaid.contains("  x0[\"other/FS\"]\n"), "{mermaid}");
        assert!(mermaid.contains("n0 -. should not .-x x0"), "{mermaid}");
    }

    /// §FS-schema.5.1: a subject's line into a `must` junction has no head, and
    /// the junction's unlabelled arrows keep the thick style; a second subject of
    /// the same junction draws only its own line.
    #[test]
    fn a_shared_junction_fans_out_once() {
        let mut nodes = vec![
            node("FS", NodeClass::Kind, vec![Extent::Folder("fs".into())]),
            node("AR", NodeClass::Kind, vec![Extent::Folder("ar".into())]),
        ];
        nodes[1].grounding = vec![BadgeRung {
            strength: Strength::Must,
            level: 1,
        }];
        let targets = vec![target("FS", 0), target("AR", 1)];
        let mut edges = vec![
            edge(0, CitationLevel::Must, targets.clone()),
            edge(1, CitationLevel::Must, targets.clone()),
        ];
        edges.iter_mut().for_each(|edge| edge.junction = Some(0));
        let mut view = view(nodes, edges);
        view.junctions = vec![Junction {
            level: CitationLevel::Must,
            targets,
        }];
        let mermaid = schema_mermaid(&view);
        let edges: Vec<&str> = mermaid.lines().skip_while(|l| !l.contains("==")).collect();
        assert_eq!(
            edges,
            [
                "  n0 == must === j0",
                "  j0 ==> n0",
                "  j0 ==> n1",
                "  n1 == must === j0"
            ]
        );
        assert!(schema_dot(&view).contains("n1 -> j0 [label=\"must\", style=bold, arrowhead=none];"));
        let table = schema_table(&view);
        assert!(table.contains("  AR  must  FS|AR  must · file\n"), "{table}");
    }
}
