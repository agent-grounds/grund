//! §AR-config.1.3: the schema's rows — one per `[[kinds]]` row, in file order —
//! and the two views over them. A row has a list of places and at most one kind,
//! so the three states §DISC-core-concerns.10.1 keeps apart are distinct by
//! construction: a non-citable place has a place and no kind, a homeless citable
//! kind has a kind and no place, and the complement is a place whose extent is
//! `Complement`.
//!
//! The v1 row the `Config` façade still carries, `KindConfig`, is rebuilt from a
//! row and the per-kind facts the other two concerns hold, in `kind_config`
//! below — the one place that knows how the two shapes meet (§AR-config.5).

use super::fields::{Field, FieldContent};
use super::kind::{KindConfig, KindIndex};
use super::project::{Project, Schema, Strength};
use super::record::{CODE_SOURCE_KIND, ConfigLocation};

/// §AR-config.1.3: one `[[kinds]]` row.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub name: String,
    /// Where this row's files live. v1 lowers at most one (§AR-config.1.3).
    pub places: Vec<Place>,
    /// The ID namespace this row declares, `None` for a place that is cited
    /// from and never declared in (§FS-config.3.4).
    pub kind: Option<Kind>,
}

/// §AR-config.1.3: a place, and whether the walk reads it (§FS-config.3.4.7).
#[derive(Clone, Debug)]
pub struct Place {
    pub extent: Extent,
    pub scanned: bool,
    /// The line of the key that named the place, which a finding about the place
    /// itself anchors at (§FS-scan.checks.unread-scan-root.location); `None` for a
    /// built-in row.
    pub source: Option<ConfigLocation>,
}

/// Two places are equal when they mean the same: where each was written is not
/// part of it, so a file and the file it prints load to equal rows (§AR-config.2).
impl PartialEq for Place {
    fn eq(&self, other: &Self) -> bool {
        self.extent == other.extent && self.scanned == other.scanned
    }
}

/// §AR-config.1.3: what a place covers.
#[derive(Clone, Debug, PartialEq)]
pub enum Extent {
    Folder(String),
    File(String),
    /// Every file outside every other place (§FS-config.3.9.2).
    Complement,
}

/// §AR-config.1.3: an ID namespace — its format override, what its
/// declarations are, the index it keeps (§AR-config.1.4), where its facts
/// come from, and the shape its declarations are held to (§FS-fields.record).
#[derive(Clone, Debug, PartialEq)]
pub struct Kind {
    pub id_format: Option<String>,
    pub form: Form,
    pub index: KindIndex,
    pub origin: Origin,
    /// §FS-fields.declare.field: the declared fields, in header order. A value
    /// chapter is one of them (§FS-fields.content.values, §DF-kind-fields.2.4).
    pub fields: Vec<Field>,
    /// §FS-fields.declare.closed: `may` is an open kind.
    pub closed: Strength,
}

/// What a kind's declarations are (§FS-config.3.4.9, §FS-config.3.4.12,
/// §FS-fields.forms). The value chapter is not a form's payload: it is a field
/// (§DF-kind-fields.2.4), so a rule kind and a prose kind name one alike.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Form {
    Prose,
    /// `values = true`, `form = "value"`: the declarations are the values.
    Value,
    /// `rules = true`, `form = "rule"`.
    Rule,
}

/// Where a kind's declarations come from (§FS-config.3.4.10).
#[derive(Clone, Debug, PartialEq)]
pub enum Origin {
    Local,
    External { fetch: String },
}

/// §AR-config.3.2: what a place nested inside another row's place means, which
/// [`Schema::owner`] answers by (§AR-config.7.3).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Nesting {
    /// v1: the nested place's files go to the complement (§FS-config.3.9.2).
    NestedPlaceFallsToComplement,
    /// v2: a listed file, then the deepest folder, owns the file
    /// (§FS-config-v2.schema.places.ownership).
    DeepestPlaceWins,
}

impl Schema {
    /// §AR-config.1.3: every place of every row, citable or not, in file order.
    pub fn places(&self) -> impl Iterator<Item = (&str, &Place)> {
        self.rows
            .iter()
            .flat_map(|row| row.places.iter().map(|place| (row.name.as_str(), place)))
    }

    /// §AR-config.1.3: every ID namespace, in file order.
    pub fn kinds(&self) -> impl Iterator<Item = (&str, &Kind)> {
        self.rows
            .iter()
            .filter_map(|row| row.kind.as_ref().map(|kind| (row.name.as_str(), kind)))
    }

    /// §FS-values.2: the rows declared `values = true` — value kinds, whose
    /// homes hold the values themselves.
    pub fn value_rows(&self) -> impl Iterator<Item = &Row> {
        self.rows.iter().filter(|row| {
            row.kind
                .as_ref()
                .is_some_and(|kind| kind.form == Form::Value)
        })
    }

    /// The name of the complement (§FS-config.3.9.2): the row whose place is
    /// `Complement` when one is written, else `code` with no row added
    /// (§AR-config.3.2).
    pub fn complement_name(&self) -> &str {
        self.rows
            .iter()
            .find(|row| row.is_complement())
            .map_or(CODE_SOURCE_KIND, |row| row.name.as_str())
    }
}

impl Row {
    /// Whether this row's kind keeps fetched snapshots (§FS-check.3.1.3): what
    /// a missing snapshot and a value home's YAML source both ask
    /// (§FS-values.2.2.4).
    pub(crate) fn fetches(&self) -> bool {
        self.kind
            .as_ref()
            .is_some_and(|kind| matches!(kind.origin, Origin::External { .. }))
    }

    /// Whether this row is the complement (§FS-config.3.9.2.1). A v2 `code` row
    /// carries the complement place first and its extent's places after it
    /// (§FS-config-v2.schema.places.complement).
    pub fn is_complement(&self) -> bool {
        self.places
            .iter()
            .any(|place| place.extent == Extent::Complement)
    }

    /// The folder this row's first place names, if it names one (§AR-config.1.3):
    /// a v1 home. A v2 row may list more; [`Row::folders`] reads them all.
    pub fn folder(&self) -> Option<&str> {
        match &self.places.first()?.extent {
            Extent::Folder(path) => Some(path),
            Extent::File(_) | Extent::Complement => None,
        }
    }

    /// The file this row's first place names, if it names one (§AR-config.1.3);
    /// [`Row::files`] reads every one.
    pub fn file(&self) -> Option<&str> {
        match &self.places.first()?.extent {
            Extent::File(path) => Some(path),
            Extent::Folder(_) | Extent::Complement => None,
        }
    }
}

impl Kind {
    /// Whether this kind's declarations carry values: a value kind, or one
    /// with a value chapter (§FS-values.1, §FS-values.2.5).
    pub fn has_values(&self) -> bool {
        self.form == Form::Value || self.value_chapter().is_some()
    }

    /// The name of the kind's values field, its value chapter
    /// (§FS-fields.content.values).
    pub fn value_chapter(&self) -> Option<&str> {
        self.fields
            .iter()
            .find(|field| field.content == FieldContent::Values)
            .map(|field| field.name.as_str())
    }
}

impl Project {
    /// The `[[kinds]]` rows in the v1 shape the façade carries (§AR-config.5).
    pub(crate) fn kind_configs(&self) -> Vec<KindConfig> {
        self.schema
            .rows
            .iter()
            .map(|row| self.kind_config(row))
            .collect()
    }

    /// One row in the v1 shape, its title, grounding and resolution read back
    /// from the concern that holds each (§AR-config.3.1).
    fn kind_config(&self, row: &Row) -> KindConfig {
        let place = row.places.first();
        // The v1 shape has one `folder` and one `file`: a v2 row with several
        // places shows its first of each, which `grund id` keeps as `folder`
        // (§FS-id.2.2); every reader of all places asks the row instead.
        let folder = row.folders().next().map(str::to_string);
        let file = row.files().next().map(str::to_string);
        let kind = row.kind.as_ref();
        let form = kind.map(|kind| kind.form);
        let (values, rules) = (form == Some(Form::Value), form == Some(Form::Rule));
        let value_chapter = kind.and_then(Kind::value_chapter).map(str::to_string);
        let grounding = self.rules.grounding.kinds.get(&row.name);
        KindConfig {
            kind: row.name.clone(),
            folder,
            file,
            title: self.presentation.kinds.get(&row.name).cloned(),
            index: kind.map_or(KindIndex::Default, |kind| kind.index.clone()),
            citable: kind.is_some(),
            scan: place.is_none_or(|place| place.scanned),
            require_grounding: grounding.and_then(|grounding| grounding.require),
            grounding_level: grounding.and_then(|grounding| grounding.level),
            values,
            value_chapter,
            rules,
            format: kind.and_then(|kind| kind.id_format.clone()),
            resolve: self.rules.resolution.get(&row.name).copied(),
            fetch: kind.and_then(|kind| match &kind.origin {
                Origin::External { fetch } => Some(fetch.clone()),
                Origin::Local => None,
            }),
        }
    }
}
