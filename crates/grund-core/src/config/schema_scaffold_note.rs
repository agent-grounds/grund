//! §FS-schema.8.2: the note a no-config schema view carries beside its header —
//! what `grund init --scaffold` would write, read from the scaffold's own lowered
//! project. Not part of the view (§FS-schema.3): no node, row or edge comes from it.

use super::project::Project;
use super::schema_view::{NodeClass, SchemaNode, SchemaView};

/// §FS-schema.8.2: the config `grund init --scaffold` writes, as the no-config
/// note names it — its version, and each kind it declares that the defaults do
/// not, in the template's order.
#[derive(Clone, Debug, PartialEq)]
pub struct ScaffoldNote {
    pub version: u32,
    pub added: Vec<SchemaNode>,
}

impl ScaffoldNote {
    /// §FS-schema.8.2: the note for `defaults`, read from the scaffold's own
    /// lowered project rather than from a second copy of its kinds.
    pub(crate) fn of(defaults: &SchemaView, scaffold: &Project) -> Self {
        let is_kind =
            |node: &SchemaNode| matches!(node.class, NodeClass::Kind | NodeClass::HomelessKind);
        let known = |node: &SchemaNode| {
            defaults
                .nodes
                .iter()
                .any(|own| is_kind(own) && own.name == node.name)
        };
        let view = SchemaView::of(scaffold);
        Self {
            version: view.version,
            added: view
                .nodes
                .into_iter()
                .filter(|node| is_kind(node) && !known(node))
                .collect(),
        }
    }
}
