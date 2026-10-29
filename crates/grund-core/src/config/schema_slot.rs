//! §FS-schema.4.1: one slot of a kind's shape as the schema view holds it, and
//! the one wording of its field line — presence, placement, content, title —
//! which `grund schema` pads into columns and the managed block prints as is
//! (§FS-init.2.3.5.12.2).

use super::fields::{FieldContent, FrontMatterKey, Placement};
use super::project::Strength;
use crate::model::schema_text;

/// §FS-schema.4.1: one slot of a kind's shape (§AR-config.6.2), owned.
#[derive(Clone, Debug, PartialEq)]
pub struct SchemaSlot {
    /// The field's name; `None` for a value kind's numbered children.
    pub name: Option<String>,
    pub presence: Strength,
    pub placement: Option<Placement>,
    pub content: FieldContent,
    /// The heading title the field's chapter must carry, where it sets one
    /// (§FS-schema.4.1, §FS-fields.identity.match).
    pub title: Option<String>,
}

impl SchemaSlot {
    /// §FS-schema.4.1: the slot's name and its facts — presence, placement where
    /// set, content where it is not prose, and last its title where it sets one
    /// — joined by ` · `. The one wording of a
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
            // §FS-fields.front-matter.record: its own `closed` where it is not `may`.
            FieldContent::FrontMatter(shape) if shape.closed != Strength::May => {
                facts.push(format!("closed {}", shape.closed.as_str()));
            }
            FieldContent::FrontMatter(_) => {}
        }
        if let Some(title) = &self.title {
            facts.push(format!("titled {}", code_span(&schema_text(title))));
        }
        (name, facts.join(" · "))
    }

    /// §FS-fields.front-matter.record: each front-matter key's name and its
    /// facts — its presence, then `one of …` where it has values — in header
    /// order; none for any other slot.
    pub fn key_lines(&self) -> Vec<(String, String)> {
        let FieldContent::FrontMatter(shape) = &self.content else {
            return Vec::new();
        };
        let line = |key: &FrontMatterKey| {
            let mut facts = vec![key.presence.as_str().to_string()];
            if let Some(values) = &key.one_of {
                facts.push(format!("one of {}", values.join(", ")));
            }
            (key.name.clone(), facts.join(" · "))
        };
        shape.keys.iter().map(line).collect()
    }
}

/// §FS-schema.4.1: `text` as a Markdown code span — fences one backtick longer
/// than its longest run of backticks, and one space inside each fence where it
/// begins or ends with a backtick, so the span shows the title's bytes as they are.
fn code_span(text: &str) -> String {
    let longest = text.split(|ch| ch != '`').map(str::len).max().unwrap_or(0);
    let fence = "`".repeat(longest + 1);
    let pad = if text.starts_with('`') || text.ends_with('`') {
        " "
    } else {
        ""
    };
    format!("{fence}{pad}{text}{pad}{fence}")
}
