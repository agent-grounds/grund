//! §FS-schema.4.1: one slot of a kind's shape as the schema view holds it, and
//! the one wording of its shape line — the heading to write, its presence and
//! placement, and what the chapter holds — which `grund schema` pads into columns,
//! labels Mermaid and DOT nodes with, and the managed block prints as is
//! (§FS-init.2.3.5). No view words a slot itself (§AR-system.principle).

use super::fields::{FieldContent, FrontMatterKey, FrontMatterShape, Placement};
use super::project::Strength;
use crate::model::{ValueFormat, schema_text};

/// §FS-schema.4.1: the declaration form every heading a shape line shows is
/// written under, which the block's lead and the table's legend name.
pub const SHAPE_DECLARATION: &str = "# <ID>: …";

/// §FS-schema.4.1: where a field's heading sits, said once by the block's lead
/// and once by the table's legend (§FS-schema.8) rather than on every line.
pub const FIELD_DEPTH: &str = "a field always sits one level below its declaration's heading";

/// §FS-schema.4.1: the placeholder a value part is shown as.
const PART: &str = "<number or word>";

/// §FS-schema.4.1: where a value part's unit goes.
const UNIT: &str = "a unit is a part of its own";

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
    /// The files a value kind's numbered children are written in; `None` for a
    /// named field (§FS-schema.4.1).
    pub files: Option<ValueFiles>,
}

/// §FS-schema.4.1: where a value kind's values are written — by `grund fetch`,
/// or by hand in each home in the forms the scan reads there.
#[derive(Clone, Debug, PartialEq)]
pub struct ValueFiles {
    /// The kind sets `fetch`, so its integration writes the file (§FS-fetch).
    pub fetched: bool,
    pub homes: Vec<ValueHome>,
}

/// §FS-schema.4.1: one value home and the forms its files take.
#[derive(Clone, Debug, PartialEq)]
pub struct ValueHome {
    /// The home as the view spells a place: a folder with a trailing `/`.
    pub place: String,
    pub folder: bool,
    /// The forms the scan reads the home's files in, Markdown, then JSON, then YAML.
    pub forms: Vec<ValueForm>,
}

/// §FS-schema.4.1: the form one value file takes, by the reader the scan picks
/// for it (§FS-values.2.2): a value source's reader, or the text scan's Markdown.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ValueForm {
    Markdown,
    Read(ValueFormat),
}

impl From<Option<ValueFormat>> for ValueForm {
    fn from(format: Option<ValueFormat>) -> Self {
        format.map_or(Self::Markdown, Self::Read)
    }
}

impl ValueForm {
    /// The language's name in `in a <name> file in` (§FS-schema.4.1).
    fn name(self) -> &'static str {
        match self {
            Self::Markdown => "Markdown",
            Self::Read(format) => format.name(),
        }
    }

    /// The form a value file takes, as code spans with placeholders: a member of
    /// a JSON object or a YAML mapping (§FS-values.2.2.1), or a Markdown
    /// declaration whose parts are numbered headings (§FS-values.2.1).
    fn shape(self) -> String {
        match self {
            Self::Read(ValueFormat::Json) => code_span(&format!("{{\"<ID>\": [{PART}, …], …}}")),
            Self::Read(ValueFormat::Yaml) => code_span(&format!("<ID>: [{PART}, …]")),
            Self::Markdown => format!(
                "{} with only {}, {} under it",
                code_span("# <ID>: <title>"),
                code_span(&format!("## 1. {PART}")),
                code_span("## 2. …"),
            ),
        }
    }
}

/// §FS-schema.4.1: one line a view prints under a kind's node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShapeLine {
    /// 0 for a field line and the kind's `closed`, 1 for what nests under a
    /// front-matter field (§FS-schema.4.1).
    pub depth: usize,
    /// What a view prints before the text: `field <name>`, `key <name>` or `closed`.
    pub key: String,
    /// The shape line, which the managed block's bullet is (§FS-init.2.3.5).
    pub text: String,
}

impl ShapeLine {
    fn new(depth: usize, key: String, text: String) -> Self {
        Self { depth, key, text }
    }
}

impl SchemaSlot {
    /// §FS-schema.4.1: the slot's shape lines — its own, then, for a front-matter
    /// field, one per key and its own `closed` one level deeper (§FS-schema.4.1).
    pub fn shape_lines(&self) -> Vec<ShapeLine> {
        let Some(name) = &self.name else {
            let text = value_text(self.files.as_ref());
            return vec![ShapeLine::new(0, "field <number>".to_string(), text)];
        };
        let key = format!("field {name}");
        if let FieldContent::FrontMatter(shape) = &self.content {
            return front_matter_lines(key, &self.parentheses(), shape);
        }
        let title = match &self.content {
            FieldContent::OneOf(values) => one_of(values),
            _ => self
                .title
                .as_deref()
                .map_or("<title>".to_string(), schema_text),
        };
        let heading = code_span(&format!("## {name}: {title}"));
        let mut text = format!("{heading} {}", self.parentheses());
        match &self.content {
            // §FS-fields.content.one-of: a scalar's heading title is its value.
            FieldContent::OneOf(_) => text.push_str(": the title is the value"),
            // §FS-fields.content.values, §FS-values.2.5: named roots, numbered parts.
            FieldContent::Values => text.push_str(&format!(
                ": only values, each {} with parts {}; {UNIT}",
                code_span(&format!("### {name}.<name>: <title>")),
                code_span(&format!("#### .1: {PART}")),
            )),
            FieldContent::Prose | FieldContent::FrontMatter(_) => {}
        }
        vec![ShapeLine::new(0, key, text)]
    }

    /// §FS-schema.4.1: the presence, then the placement where one is set, in
    /// check's words (§FS-fields.order.after), joined by `, ` in parentheses.
    fn parentheses(&self) -> String {
        let placement = match &self.placement {
            Some(Placement::First) => ", first".to_string(),
            Some(Placement::Last) => ", last".to_string(),
            Some(Placement::After(field)) => format!(", directly after {field}"),
            None => String::new(),
        };
        format!("({}{placement})", self.presence.as_str())
    }
}

/// §FS-schema.4.1: a kind's `closed` line where it is not `may`
/// (§FS-fields.declare.closed).
pub(crate) fn kind_closed_line(closed: Strength) -> ShapeLine {
    let text = format!(
        "no other chapter except numbered ones ({})",
        closed.as_str()
    );
    ShapeLine::new(0, "closed".to_string(), text)
}

/// §FS-schema.4.1: front matter is the block at the top of the file and no
/// heading; each key follows as the line to write, then its own `closed` where
/// it is not `may` (§FS-fields.front-matter.record).
fn front_matter_lines(key: String, parentheses: &str, shape: &FrontMatterShape) -> Vec<ShapeLine> {
    let text = format!(
        "{} YAML or {} TOML front matter at the top of the file {parentheses}",
        code_span("---"),
        code_span("+++"),
    );
    let mut lines = vec![ShapeLine::new(0, key, text)];
    let key_line = |key: &FrontMatterKey| {
        let value = key.one_of.as_deref().map_or("<value>".to_string(), one_of);
        let line = code_span(&format!("{}: {value}", key.name));
        let text = format!("{line} ({})", key.presence.as_str());
        ShapeLine::new(1, format!("key {}", key.name), text)
    };
    lines.extend(shape.keys.iter().map(key_line));
    if shape.closed != Strength::May {
        let text = format!("no other key ({})", shape.closed.as_str());
        lines.push(ShapeLine::new(1, "closed".to_string(), text));
    }
    lines
}

/// §FS-schema.4.1: a value kind's line — the form each home holds and where,
/// joined by `, ` and by `, or ` before the last, then the citation its first
/// element answers to; a fetched kind's names the file `grund fetch` writes. A
/// homeless kind's values are Markdown declarations, which need no home.
fn value_text(files: Option<&ValueFiles>) -> String {
    let cite = format!("part 1 is {}", code_span("\u{a7}<ID>.1"));
    let homes = files.map_or(&[][..], |files| &files.homes);
    if files.is_some_and(|files| files.fetched) {
        let homes: Vec<String> = homes.iter().map(|home| code_span(&home.place)).collect();
        let fetch = code_span("grund fetch");
        return format!("written by {fetch} into {}: {cite}", homes.join(", "));
    }
    let mut forms = Vec::new();
    for home in homes {
        let place = code_span(&home.place);
        for form in &home.forms {
            forms.push(if home.folder {
                format!("{} in a {} file in {place}", form.shape(), form.name())
            } else {
                format!("{} in {place}", form.shape())
            });
        }
    }
    let forms = match forms.split_last() {
        None => ValueForm::Markdown.shape(),
        Some((only, [])) => only.clone(),
        Some((last, rest)) => format!("{}, or {last}", rest.join(", ")),
    };
    format!("{forms}: {cite}; {UNIT}")
}

/// §FS-schema.4.1: the placeholder of a scalar's values, `<one of A, B>`.
fn one_of(values: &[String]) -> String {
    let values: Vec<String> = values.iter().map(|value| schema_text(value)).collect();
    format!("<one of {}>", values.join(", "))
}

/// §FS-schema.4.1: `text` as a Markdown code span — fences one backtick longer
/// than its longest run of backticks, and one space inside each fence where it
/// begins or ends with a backtick, so the span shows its bytes as they are.
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
