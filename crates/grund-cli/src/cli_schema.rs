/// §FS-schema.1: the three printers of `grund schema`; `table` is the default.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SchemaFormat {
    Table,
    Mermaid,
    Dot,
}

/// `grund schema [PATH] [--format table|mermaid|dot]` (§FS-schema). Every usage
/// error is answered before the configuration is read (§FS-schema.12); the view
/// is lowered by the engine from the configuration alone (§AR-config.6.3) and
/// printed here (§DA-engine-renders-nothing).
fn command_schema(args: &[String]) -> ExitCode {
    let mut format = None;
    let mut path: Option<PathBuf> = None;
    let mut words = args.iter();
    while let Some(arg) = words.next() {
        if let Some(value) = arg.strip_prefix("--format=") {
            format = Some(value.to_string());
        } else if arg == "--format" {
            let Some(value) = words.next() else {
                eprintln!("error: --format requires a value");
                return ExitCode::from(2);
            };
            format = Some(value.clone());
        } else if arg.starts_with('-') {
            eprintln!("error: unknown flag `{arg}`");
            return ExitCode::from(2);
        } else if path.is_some() {
            // §FS-cli.3.2: at most one path.
            eprintln!("error: schema takes at most one path argument");
            return ExitCode::from(2);
        } else {
            path = Some(PathBuf::from(arg));
        }
    }
    // §FS-schema.13: no `text` and no `json`; `[output] format` does not apply.
    let format = match format.as_deref() {
        None | Some("table") => SchemaFormat::Table,
        Some("mermaid") => SchemaFormat::Mermaid,
        Some("dot") => SchemaFormat::Dot,
        Some(other) => {
            eprintln!("error: unsupported schema format `{other}`");
            eprintln!("known formats: table, mermaid, dot");
            return ExitCode::from(2);
        }
    };
    let path = path.unwrap_or_else(|| ".".into());
    // §FS-schema.12: the located error `grund config show` prints, exit 2.
    let view = match schema_view(&path) {
        Ok(view) => view,
        Err(err) => {
            eprintln!("error: {err:#}");
            return ExitCode::from(2);
        }
    };
    let text = match format {
        SchemaFormat::Table => schema_table(&view),
        SchemaFormat::Mermaid => schema_mermaid(&view),
        SchemaFormat::Dot => schema_dot(&view),
    };
    print!("{text}");
    ExitCode::SUCCESS
}

/// §FS-schema.8: the header line, the same in every format.
fn schema_header(view: &SchemaView) -> String {
    format!("{} · {} · v{}", view.project, view.config_file, view.version)
}

/// §FS-schema.8: the legend's three lines; the graph formats print the first two
/// and their own third (§FS-schema.9).
const SCHEMA_LEGEND: [&str; 3] = [
    "must = check error · warn = check warning · should = suggestion (check --suggestions)",
    "may = allowed · must not / warn not / should not = forbidden at that strength",
    "A|B = one obligation, met by citing either · — = no direction rule names it",
];

/// §FS-schema.8.3: the table legend's fourth line, where some scanned place or
/// the complement has an obligation and no `grounded` badge — an obligation
/// never judges a file there that cites nothing, and grounding is what does
/// (§FS-config-v2.rules.citations.uncited, §FS-config.3.9.1.4). The graph
/// formats never print it.
fn schema_uncited_line(view: &SchemaView) -> Option<String> {
    let obligation = |level: CitationLevel| {
        matches!(level, CitationLevel::Must | CitationLevel::Warn | CitationLevel::Should)
    };
    let skipping = view.nodes.iter().enumerate().any(|(index, node)| {
        matches!(node.class, NodeClass::Place | NodeClass::Complement)
            && node.grounding.is_empty()
            && view.edges.iter().any(|e| e.subject == index && obligation(e.level))
    });
    let remedy = match view.version {
        1 => "require_grounding",
        _ => "[rules.citations.<ROW>.grounding]",
    };
    skipping.then(|| {
        format!(
            "must/warn/should skip an uncited file of a place or the complement · {remedy} makes it a finding"
        )
    })
}

/// §FS-schema.9: the third comment line of the graph formats.
const SCHEMA_GRAPH_ANY_OF: &str = "any of = one obligation, met by citing any one of its arrows";

/// §FS-schema.2.1: the line naming a workspace root's members, `None` elsewhere.
fn schema_members_line(view: &SchemaView) -> Option<String> {
    (!view.members.is_empty()).then(|| {
        format!(
            "members: {} — run grund schema <member>",
            view.members.join(", ")
        )
    })
}

/// §FS-schema.5: the project-wide `default` line, where it is not `may`.
fn schema_default_line(view: &SchemaView) -> Option<String> {
    view.default.map(|level| {
        format!(
            "everything else, for every subject: {}",
            schema_strength(level)
        )
    })
}

/// §FS-schema.4: a node's class, always a word.
fn schema_class_word(class: NodeClass) -> &'static str {
    match class {
        NodeClass::Kind | NodeClass::HomelessKind => "kind",
        NodeClass::Place => "place",
        NodeClass::Unscanned => "place, not scanned",
        NodeClass::Complement => "complement",
    }
}

/// §FS-schema.4: a node's places, comma-separated in written order; a folder
/// with a trailing `/`, a homeless kind as `no home`.
fn schema_places(node: &SchemaNode) -> String {
    if node.places.is_empty() {
        return "no home".to_string();
    }
    let places: Vec<String> = node
        .places
        .iter()
        .map(|extent| match extent {
            Extent::Folder(path) => format!("{}/", path.trim_end_matches('/')),
            Extent::File(path) => path.clone(),
            Extent::Complement => "every scanned file no other row claims".to_string(),
        })
        .collect();
    places.join(", ")
}

/// §FS-schema.4.1: a slot's name and its facts, in the one wording core keeps.
fn schema_slot(slot: &SchemaSlot) -> (String, String) {
    slot.field_line()
}

/// §FS-schema.6: a badge's ladder, `<strength> · <unit>` rungs joined by `, `.
fn schema_badge(node: &SchemaNode) -> Option<String> {
    if node.grounding.is_empty() {
        return None;
    }
    let rungs: Vec<String> = node
        .grounding
        .iter()
        .map(|rung| format!("{} · {}", rung.strength.as_str(), rung.unit()))
        .collect();
    Some(rungs.join(", "))
}

/// §FS-schema.5.2: a strength as a word, a prohibition's hyphen a space.
fn schema_strength(level: CitationLevel) -> String {
    level.as_str().replace('-', " ")
}
