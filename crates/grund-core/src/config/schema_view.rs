//! §AR-config.6.3: the schema view, lowered from a `Project` on demand. It is
//! the record `grund schema` prints (§FS-schema): the rows as nodes, the direction
//! rules as edges, an any-of entry through one junction, grounding as a badge. It
//! reads the rows, `Kind::slots()`, `rules.citations` and `rules.grounding` —
//! no scan, no catalog, no ID graph — and `compile` never builds it, so `check`
//! pays nothing for it. The printers are the frontend's (§DA-engine-renders-nothing).

use super::citations::NamespaceMatch;
use super::citations::{CitationDisjunction, CitationLevel, render_citation_target};
use super::discovery::config_file_in;
use super::fields::{FieldContent, Placement};
use super::grounding::{configured_row_grounding, grounding_unit, soft_grounding_rungs};
use super::project::{Project, Strength};
use super::record::Config;
use super::rows::{Extent, Row};
use super::schema_scaffold_note::ScaffoldNote;
use super::slots::{Content, Handle};

/// §AR-config.6.3: one project's schema view.
#[derive(Clone, Debug, PartialEq)]
pub struct SchemaView {
    /// The header (§FS-schema.8): the project's name, its config file spelled
    /// from the config root, and the format version that spelled it.
    pub project: String,
    pub config_file: String,
    pub version: u32,
    /// The nodes in row order, the complement after every written row unless a
    /// row names it (§FS-schema.7).
    pub nodes: Vec<SchemaNode>,
    /// One per distinct (any-of entry, strength), numbered in the order its first
    /// edge is printed (§FS-schema.5.1, §FS-schema.7).
    pub junctions: Vec<Junction>,
    /// In subject node order, then strength order, then written order (§FS-schema.7).
    pub edges: Vec<SchemaEdge>,
    /// The project-wide `default` where it is not `may` (§FS-schema.5).
    pub default: Option<CitationLevel>,
    /// The root's `[workspace] members`, as written (§FS-schema.2.1).
    pub members: Vec<String>,
    /// What `grund init --scaffold` would write, where no config file was read
    /// (§FS-schema.8.2). Beside the header and not part of the view: no node,
    /// row or edge comes from it (§FS-schema.3).
    pub scaffold: Option<ScaffoldNote>,
}

/// §FS-schema.4: what a node's row is, always printed as a word.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NodeClass {
    Kind,
    HomelessKind,
    Place,
    Unscanned,
    Complement,
}

/// §FS-schema.4: one row as the view shows it.
#[derive(Clone, Debug, PartialEq)]
pub struct SchemaNode {
    pub name: String,
    pub class: NodeClass,
    /// Every place of the row in written order (§FS-schema.4).
    pub places: Vec<Extent>,
    /// The kind's slots, in declared order (§FS-schema.4.1).
    pub slots: Vec<SchemaSlot>,
    /// The kind's `closed` strength where it is not `may` (§FS-schema.4.1).
    pub closed: Option<Strength>,
    /// The row's effective grounding ladder; empty where it need not be grounded
    /// (§FS-schema.6).
    pub grounding: Vec<BadgeRung>,
    /// The subject's own `default` where it is not `may` (§FS-schema.5).
    pub default: Option<CitationLevel>,
    /// Whether any direction rule names the node, as subject or target (§FS-schema.4.2).
    pub ruled: bool,
}

impl SchemaNode {
    /// §FS-schema.4.1: the lines `grund schema` prints under this node, one per
    /// slot and one for a `closed` that is not `may`, with a single space where the
    /// view pads columns — the managed block's declaration shape
    /// (§FS-init.2.3.5.12.2).
    pub fn shape_lines(&self) -> Vec<String> {
        let mut lines: Vec<String> = self
            .slots
            .iter()
            .map(|slot| {
                let (name, facts) = slot.field_line();
                format!("field {name} {facts}")
            })
            .collect();
        if let Some(closed) = self.closed {
            lines.push(format!("closed {}", closed.as_str()));
        }
        lines
    }
}

/// §FS-schema.4.1: one slot of a kind's shape (§AR-config.6.2), owned.
#[derive(Clone, Debug, PartialEq)]
pub struct SchemaSlot {
    /// The field's name; `None` for a value kind's numbered children.
    pub name: Option<String>,
    pub presence: Strength,
    pub placement: Option<Placement>,
    pub content: FieldContent,
}

impl SchemaSlot {
    /// §FS-schema.4.1: the slot's name and its facts — presence, placement where
    /// set, content where it is not prose — joined by ` · `. The one wording of a
    /// field line, which `grund schema` pads into columns and the managed block's
    /// `### Declaration shapes` prints as is (§FS-init.2.3.5.12.2).
    pub fn field_line(&self) -> (String, String) {
        let name = self.name.clone().unwrap_or_else(|| "<number>".to_string());
        let mut facts = vec![self.presence.as_str().to_string()];
        match &self.placement {
            Some(Placement::First) => facts.push("first".to_string()),
            Some(Placement::Last) => facts.push("last".to_string()),
            Some(Placement::After(field)) => facts.push(format!("after {field}")),
            None => {}
        }
        match &self.content {
            FieldContent::Prose => {}
            FieldContent::Values => facts.push("values".to_string()),
            FieldContent::OneOf(values) => facts.push(format!("one of {}", values.join(", "))),
        }
        (name, facts.join(" · "))
    }
}

/// §FS-schema.6: one rung of a grounding badge.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BadgeRung {
    pub strength: Strength,
    pub level: usize,
}

impl BadgeRung {
    /// The unit as `grund.toml` spells it: `file`, or `h2` to `h6`.
    pub fn unit(&self) -> String {
        grounding_unit(self.level)
    }
}

/// §FS-schema.5: one target of a direction, as written, and the node it names
/// when it names one of this project's rows.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EdgeTarget {
    pub label: String,
    pub node: Option<usize>,
}

/// §FS-schema.5.1: the one node an any-of entry is drawn through.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Junction {
    pub level: CitationLevel,
    pub targets: Vec<EdgeTarget>,
}

/// §FS-schema.5: one direction — a subject node, a strength, and its target, or
/// its junction when the entry is an any-of.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchemaEdge {
    pub subject: usize,
    pub level: CitationLevel,
    pub targets: Vec<EdgeTarget>,
    pub junction: Option<usize>,
}

impl SchemaEdge {
    /// §FS-schema.5.3: a single-target entry whose target is its own subject.
    pub fn is_self(&self) -> bool {
        self.junction.is_none() && self.targets.first().and_then(|t| t.node) == Some(self.subject)
    }
}

/// §FS-schema.5.2: the order strengths are printed in, which is not the order a
/// citation site is matched in.
const STRENGTH_ORDER: [CitationLevel; 7] = [
    CitationLevel::Must,
    CitationLevel::Warn,
    CitationLevel::Should,
    CitationLevel::May,
    CitationLevel::MustNot,
    CitationLevel::WarnNot,
    CitationLevel::ShouldNot,
];

impl SchemaView {
    /// §AR-config.6.3: lower `project` into its view. Reads the configuration
    /// alone (§FS-schema.3).
    pub fn of(project: &Project) -> Self {
        let mut nodes: Vec<SchemaNode> = project
            .schema
            .rows
            .iter()
            .map(|row| node(project, row))
            .collect();
        // §FS-schema.7: the implicit complement, after every written row.
        if !project.schema.rows.iter().any(Row::is_complement) {
            let name = project.schema.complement_name().to_string();
            let grounding = badge(project, &name);
            nodes.push(SchemaNode {
                name,
                class: NodeClass::Complement,
                places: vec![Extent::Complement],
                slots: Vec::new(),
                closed: None,
                grounding,
                default: None,
                ruled: false,
            });
        }
        let citations = &project.rules.citations;
        let index = |name: &str| nodes.iter().position(|node| node.name == name);
        let mut edges = Vec::new();
        let mut junctions: Vec<Junction> = Vec::new();
        let mut defaults = Vec::new();
        for (subject, node) in nodes.iter().enumerate() {
            let Some(rules) = citations.per_kind.get(&node.name) else {
                continue;
            };
            defaults.push((subject, rules.default.filter(|l| *l != CitationLevel::May)));
            let lists = rules.lists();
            for level in STRENGTH_ORDER {
                let (_, entries) = lists
                    .iter()
                    .find(|(l, _)| *l == level)
                    .expect("every level");
                for entry in entries.iter() {
                    let targets = targets(entry, &index);
                    let junction =
                        (targets.len() > 1).then(|| junction_for(&mut junctions, level, &targets));
                    edges.push(SchemaEdge {
                        subject,
                        level,
                        targets,
                        junction,
                    });
                }
            }
        }
        for (subject, default) in defaults {
            nodes[subject].default = default;
            nodes[subject].ruled |= default.is_some();
        }
        for edge in &edges {
            nodes[edge.subject].ruled = true;
            for target in edge.targets.iter().filter_map(|t| t.node) {
                nodes[target].ruled = true;
            }
        }
        let members = if project.workspace.declared {
            project.workspace.members.clone()
        } else {
            Vec::new()
        };
        Self {
            project: project.name.clone().unwrap_or_default(),
            config_file: "grund.toml".to_string(),
            version: project.version,
            nodes,
            junctions,
            edges,
            default: citations
                .global_default
                .filter(|l| *l != CitationLevel::May),
            members,
            scaffold: None,
        }
    }
}

impl SchemaView {
    /// §FS-schema.8: the view of a loaded config, its header filled in — the
    /// config file spelled from the config root whatever the run's `--path-base`
    /// (§FS-schema.1), and a project with no `project_name` named by its root.
    pub(crate) fn of_config(config: &Config) -> Self {
        let mut view = Self::of(config.project());
        view.config_file = match config_file_read(config) {
            Some(file) => {
                let relative = file.strip_prefix(&config.root).unwrap_or(&file);
                relative.to_string_lossy().replace('\\', "/")
            }
            None => "defaults, no grund.toml".to_string(),
        };
        if view.project.is_empty() {
            let root = config.root.canonicalize();
            let root = root.as_deref().unwrap_or(&config.root);
            let name = root
                .file_name()
                .map(|name| name.to_string_lossy().into_owned());
            view.project = name.unwrap_or_default();
        }
        view
    }
}

/// §FS-schema.8: the file the run read, never one beside a fallback root; `None`
/// where no config file was found and the defaults govern (§FS-schema.8.2).
pub(crate) fn config_file_read(config: &Config) -> Option<std::path::PathBuf> {
    config
        .run()
        .config_file
        .as_ref()
        .and(config_file_in(&config.root))
}

/// §FS-schema.5.1: the junction for an any-of entry at `level`, shared by every
/// subject that writes the same entry at the same strength. Edges are lowered in
/// print order, so a new junction's number is the order of its first edge.
fn junction_for(
    junctions: &mut Vec<Junction>,
    level: CitationLevel,
    targets: &[EdgeTarget],
) -> usize {
    if let Some(found) = junctions
        .iter()
        .position(|j| j.level == level && j.targets == targets)
    {
        return found;
    }
    junctions.push(Junction {
        level,
        targets: targets.to_vec(),
    });
    junctions.len() - 1
}

/// §FS-schema.5: an entry's targets as written, each with the local node it names.
fn targets(entry: &CitationDisjunction, index: &impl Fn(&str) -> Option<usize>) -> Vec<EdgeTarget> {
    entry
        .targets
        .iter()
        .map(|target| EdgeTarget {
            label: render_citation_target(target),
            node: match target.namespace {
                NamespaceMatch::Local => index(&target.kind),
                NamespaceMatch::Alias(_) | NamespaceMatch::Any => None,
            },
        })
        .collect()
}

/// §FS-schema.4: one written row as a node; whether a rule names it is settled
/// once every edge is known.
fn node(project: &Project, row: &Row) -> SchemaNode {
    let scanned = row.places.iter().all(|place| place.scanned);
    let class = match &row.kind {
        _ if !scanned => NodeClass::Unscanned,
        _ if row.is_complement() => NodeClass::Complement,
        Some(_) if row.places.is_empty() => NodeClass::HomelessKind,
        Some(_) => NodeClass::Kind,
        None => NodeClass::Place,
    };
    let slots = row
        .kind
        .iter()
        .flat_map(|kind| kind.slots())
        .map(|slot| SchemaSlot {
            name: match slot.handle {
                Handle::Named(name) => Some(name.to_string()),
                Handle::Numbered => None,
            },
            presence: slot.presence,
            placement: slot.placement.cloned(),
            content: match slot.content {
                Content::Prose => FieldContent::Prose,
                Content::Values => FieldContent::Values,
                Content::OneOf(values) => FieldContent::OneOf(values.to_vec()),
            },
        });
    SchemaNode {
        name: row.name.clone(),
        class,
        // §FS-schema.4: a `code` row that lists places shows them instead of the
        // root extent it carries first (§FS-config-v2.schema.places.complement).
        places: row
            .places
            .iter()
            .filter(|place| place.extent != Extent::Complement || row.places.len() == 1)
            .map(|place| place.extent.clone())
            .collect(),
        slots: slots.collect(),
        closed: row
            .kind
            .as_ref()
            .map(|kind| kind.closed)
            .filter(|s| *s != Strength::May),
        // §FS-schema.6: an unscanned row is never checked, so nothing grounds it.
        grounding: if scanned {
            badge(project, &row.name)
        } else {
            Vec::new()
        },
        default: None,
        ruled: false,
    }
}

/// §FS-schema.6: the effective grounding ladder of the row named `name`, read
/// from the resolvers check enforces (§REQ-one-answer): the `must` rung its pair
/// asks for, then the `warn` and `should` rungs. A `may` rung checks nothing, so
/// it is no rung of the badge.
fn badge(project: &Project, name: &str) -> Vec<BadgeRung> {
    let (require, level) = configured_row_grounding(&project.rules, name);
    let must = require.then_some(BadgeRung {
        strength: Strength::Must,
        level,
    });
    must.into_iter()
        .chain(
            soft_grounding_rungs(&project.rules, name)
                .into_iter()
                .map(|(strength, level)| BadgeRung { strength, level }),
        )
        .collect()
}
