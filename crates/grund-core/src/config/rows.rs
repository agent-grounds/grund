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

use std::path::{Path, PathBuf};

use super::kind::{DEFAULT_KIND_INDEX, KindConfig, KindIndex};
use super::project::{Project, Schema};
use super::record::CODE_SOURCE_KIND;

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
#[derive(Clone, Debug, PartialEq)]
pub struct Place {
    pub extent: Extent,
    pub scanned: bool,
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
/// declarations are, the index it keeps (§AR-config.1.4), and where its facts
/// come from.
#[derive(Clone, Debug, PartialEq)]
pub struct Kind {
    pub id_format: Option<String>,
    pub form: Form,
    pub index: KindIndex,
    pub origin: Origin,
}

/// What a kind's declarations are (§FS-config.3.4.9, §FS-config.3.4.12,
/// §FS-config.3.4.13). A rule kind may still name a value chapter in v1, so
/// the rule form carries one and the lowering keeps it (§AR-config.2 rule 3).
#[derive(Clone, Debug, PartialEq)]
pub enum Form {
    Prose,
    /// `values = true` (`chapter: None`), or the named chapter whose children
    /// are the values.
    Value {
        chapter: Option<String>,
    },
    Rule {
        value_chapter: Option<String>,
    },
}

/// Where a kind's declarations come from (§FS-config.3.4.10).
#[derive(Clone, Debug, PartialEq)]
pub enum Origin {
    Local,
    External { fetch: String },
}

/// §AR-config.3.2: what a place nested inside another row's place means. v1
/// has one answer, so it is the only variant; #457 may add another.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Nesting {
    /// The nested place's files go to the complement (§FS-config.3.9.2).
    NestedPlaceFallsToComplement,
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

    /// §FS-values.2: the rows declared `values = true` — value kinds with no
    /// named chapter, whose homes hold the values themselves.
    pub fn value_rows(&self) -> impl Iterator<Item = &Row> {
        self.rows.iter().filter(|row| {
            row.kind
                .as_ref()
                .is_some_and(|kind| kind.form == Form::Value { chapter: None })
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
    /// Whether this row is the complement (§FS-config.3.9.2.1).
    pub fn is_complement(&self) -> bool {
        self.places
            .iter()
            .any(|place| place.extent == Extent::Complement)
    }

    /// The folder this row's first place names, if it names one (§AR-config.1.3).
    pub fn folder(&self) -> Option<&str> {
        match &self.places.first()?.extent {
            Extent::Folder(path) => Some(path),
            Extent::File(_) | Extent::Complement => None,
        }
    }

    /// The file this row's first place names, if it names one (§AR-config.1.3).
    pub fn file(&self) -> Option<&str> {
        match &self.places.first()?.extent {
            Extent::File(path) => Some(path),
            Extent::Folder(_) | Extent::Complement => None,
        }
    }

    /// How this row's place is named in a finding: `<folder>/`, or its file
    /// (§FS-check.3.11); `None` for a row with no place to look at.
    pub fn place_label(&self) -> Option<String> {
        if let Some(folder) = self.folder() {
            return Some(format!("{folder}/"));
        }
        self.file().map(str::to_string)
    }

    /// The index file a citable folder row keeps, relative to the config root
    /// (§FS-config.3.4): `None` for a row with no kind, no folder, or
    /// `index = false`.
    pub fn index_path(&self) -> Option<PathBuf> {
        let name = match &self.kind.as_ref()?.index {
            KindIndex::Disabled => return None,
            KindIndex::Default => DEFAULT_KIND_INDEX,
            KindIndex::Named(name) => name.as_str(),
        };
        Some(Path::new(self.folder()?).join(name))
    }
}

impl Kind {
    /// Whether this kind's declarations carry values: a value kind, or a rule
    /// kind that names a value chapter (§FS-values.1, §FS-values.2.5).
    pub fn has_values(&self) -> bool {
        matches!(
            self.form,
            Form::Value { .. }
                | Form::Rule {
                    value_chapter: Some(_)
                }
        )
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
        let (folder, file) = match place.map(|place| &place.extent) {
            Some(Extent::Folder(path)) => (Some(path.clone()), None),
            Some(Extent::File(path)) => (None, Some(path.clone())),
            Some(Extent::Complement) | None => (None, None),
        };
        let kind = row.kind.as_ref();
        let (values, value_chapter, rules) = match kind.map(|kind| &kind.form) {
            Some(Form::Value { chapter: None }) => (true, None, false),
            Some(Form::Value { chapter }) => (false, chapter.clone(), false),
            Some(Form::Rule { value_chapter }) => (false, value_chapter.clone(), true),
            Some(Form::Prose) | None => (false, None, false),
        };
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
