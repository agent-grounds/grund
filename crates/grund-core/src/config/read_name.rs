//! The one rule that picks the name a file's reading is taken from (§AR-config.1.6,
//! §FS-scan.identity.reading): it takes a spelling and the `Spellings` table the walk publishes,
//! and returns the name to read. A [`ReadName`] is the only thing `FileReading` is built from and
//! nothing outside this file can make one, so no caller gets a reading from a bare spelling.
//!
//! Only what the file is read as moves. The hidden and listed tests keep asking the spelling
//! handed (§FS-scan.identity.reading.entry), which is what `FileReading` keeps as its path.

use std::path::{Path, PathBuf};

use super::file_reading::FileReading;
use super::frame::Frame;
use super::project::{Schema, file_extension};
use super::sources::Language;
use crate::model::ValueFormat;

/// The name a file is read by, under the spelling that was handed
/// (§FS-scan.identity.reading). Its fields are this file's alone, which is the guard.
#[derive(Clone, Copy)]
pub(crate) struct ReadName<'a> {
    spelling: &'a Path,
    language: Option<&'a Language>,
    extension: Option<&'a str>,
    /// Whether a version-2 name no admitted language claims is read by its name, as grund
    /// 0.22.0 read it, which only `grund fetch`'s view of its output asks
    /// (§DF-one-reading-answer.2.6).
    unclaimed_by_name: bool,
}

impl<'a> ReadName<'a> {
    /// The spelling handed, which the entry tests ask (§FS-scan.identity.reading.entry).
    pub(super) fn spelling(&self) -> &'a Path {
        self.spelling
    }

    /// The chosen name's language, `None` for a v1 file or one no admitted language claims.
    pub(super) fn language(&self) -> Option<&'a Language> {
        self.language
    }

    /// The chosen name's extension, which keys a v1 file's reading.
    pub(super) fn extension(&self) -> Option<&'a str> {
        self.extension
    }

    /// Whether a version-2 name no admitted language claims is read by its name
    /// (§DF-one-reading-answer.2.6).
    pub(super) fn unclaimed_by_name(&self) -> bool {
        self.unclaimed_by_name
    }

    /// The spelling read by its own name, which is every file the rule leaves alone.
    fn as_spelled(schema: &'a Schema, spelling: &'a Path) -> Self {
        let model = schema.sources.model.as_ref();
        Self {
            spelling,
            language: model.and_then(|model| model.language_of(spelling)),
            extension: spelling.extension().and_then(|ext| ext.to_str()),
            unclaimed_by_name: false,
        }
    }

    /// The name of this reading: its language's under version 2; under version 1 its extension,
    /// `markdown` for `md`, and `source with no extension` for none
    /// (§FS-scan.checks.reading-conflict).
    fn label(&self) -> String {
        match (self.language, self.extension) {
            (Some(language), _) => language.name.clone(),
            (None, Some("md")) => "markdown".to_string(),
            (None, Some(extension)) if !extension.is_empty() => extension.to_string(),
            (None, _) => "source with no extension".to_string(),
        }
    }
}

/// What a name reads, where it claims a reading (§FS-scan.identity.reading.claims): an admitted
/// language under version 2, a listed extension under version 1. Only the name counts.
fn claim<'a>(schema: &'a Schema, name: &Path) -> Option<(Option<&'a Language>, Option<&'a str>)> {
    let sources = &schema.sources;
    match sources.model.as_ref() {
        Some(model) => model
            .language_of(name)
            .map(|language| (Some(language), None)),
        None if sources.lists_extension(name) => {
            let listed = file_extension(name)
                .and_then(|ext| sources.extensions.iter().find(|listed| *listed == ext));
            Some((None, listed.map(String::as_str)))
        }
        None => None,
    }
}

/// The JSON or YAML name a value source goes by, which a value-named spelling takes from its
/// target whatever the lists say: `values/a.json -> limits.yaml` is YAML
/// (§FS-scan.identity.reading.every-caller).
fn value_name(name: &Path) -> Option<&'static str> {
    match file_extension(name)? {
        "json" => Some("json"),
        "yaml" => Some("yaml"),
        "yml" => Some("yml"),
        _ => None,
    }
}

/// The extension a name reads by, as the configuration or the reader spells it, so the reading can
/// outlive a name the table lent (§FS-scan.identity.reading.plain). One neither of them reads is no
/// extension to a reader.
fn read_extension<'a>(schema: &'a Schema, name: &Path) -> Option<&'a str> {
    let extension = file_extension(name)?;
    let read = value_name(name).or((extension == "md").then_some("md"));
    read.or_else(|| {
        let listed = schema.sources.extensions.iter();
        listed
            .map(String::as_str)
            .find(|listed| *listed == extension)
    })
}

/// How a run reads the file `spelling` reached, against the names it knows
/// (§FS-scan.checks.reading-conflict, §FS-scan.checks.reading-moved).
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum NamesVerdict {
    /// One reading, and the name listed first read that way before 0.22.0 too.
    Agree,
    /// Two in-tree names read it differently, and its own name decides nothing.
    Conflict { readings: Vec<String> },
    /// The name listed first is a link that read it another way before 0.22.0.
    Moved { was: String, now: String },
}

impl<'f> Frame<'f> {
    /// The rule of §FS-scan.identity.reading: a file inside the config root is read as its own
    /// name reads, when that name claims a reading, and any other keeps the reaching name's.
    fn read_name<'b>(&self, schema: &'b Schema, spelling: &'b Path) -> ReadName<'b> {
        let as_spelled = ReadName::as_spelled(schema, spelling);
        let Some(physical) = self.owning_name(spelling) else {
            return self.as_the_plain_run_reads(schema, as_spelled);
        };
        if let Some((language, extension)) = claim(schema, &physical) {
            return ReadName {
                language,
                extension,
                ..as_spelled
            };
        }
        // §FS-scan.identity.reading.every-caller: JSON or YAML, from the file's own name.
        match (value_name(spelling), value_name(&physical)) {
            (Some(_), Some(extension)) => ReadName {
                language: None,
                extension: Some(extension),
                ..as_spelled
            },
            _ => self.as_the_plain_run_reads(schema, as_spelled),
        }
    }

    /// A name with no say keeps the reading of the name that has one
    /// (§FS-scan.identity.reading.plain), which every other file skips in one empty lookup.
    fn as_the_plain_run_reads<'b>(&self, schema: &'b Schema, name: ReadName<'b>) -> ReadName<'b> {
        let Some(plain) = self.run.spellings.plain_name_of(name.spelling) else {
            return name;
        };
        let model = schema.sources.model.as_ref();
        ReadName {
            language: model.and_then(|model| model.language_of(&plain)),
            extension: read_extension(schema, &plain),
            ..name
        }
    }

    /// The names of `physical` that have a say in how it is read: the ones a plain run `reaches`,
    /// when it reaches any, and otherwise every name this run met. Each other name is published
    /// with the first of them, which is the name it is read by, so a refusal, a moved reading and
    /// a file's reading all rest on this one answer (§FS-scan.identity.reading.plain).
    pub(crate) fn names_with_a_say(
        &self,
        physical: &Path,
        reaches: impl Fn(&Path) -> bool,
    ) -> Vec<PathBuf> {
        let spellings = &self.run.spellings;
        let names = spellings.names_at(physical);
        let (reached, others): (Vec<_>, Vec<_>) = names.iter().cloned().partition(|n| reaches(n));
        let Some(first) = reached.first() else {
            return names;
        };
        for other in &others {
            spellings.publish_plain_name(other, first);
        }
        reached
    }

    /// Where `spelling` physically is, where that is another name inside the config root: the
    /// only name that can decide the reading (§FS-scan.identity.reading.outside).
    fn owning_name(&self, spelling: &Path) -> Option<PathBuf> {
        let spellings = &self.run.spellings;
        let physical = spellings.physical(spelling)?;
        (physical != spelling && physical.starts_with(spellings.physical_root(self.root())))
            .then_some(physical)
    }

    /// How `path` is read under this project (§FS-config-v2.schema.sources,
    /// §FS-scan.identity.reading.every-caller): the one answer every reader asks.
    pub(crate) fn reading<'b>(&self, schema: &'b Schema, path: &'b Path) -> FileReading<'b>
    where
        'f: 'b,
    {
        let name = ReadName {
            unclaimed_by_name: self.run.unclaimed_read_by_name,
            ..self.read_name(schema, path)
        };
        FileReading::of(self.compiled, schema, name)
    }

    /// The format a value home takes `path` by, where it takes it (§FS-values.2.2): the spelling's
    /// own name decides, and its one resolution comes after that, so a name no home takes pays for
    /// none. This is the entry answer, not the file's reading: the rule then picks the format, or
    /// says the file reads as no value source at all, which earns the name a located finding
    /// rather than silence, and a failed status keeps this format (§FS-scan.identity.reading.entry,
    /// §FS-scan.checks.reading-moved, §FS-values.5.3.1).
    pub(crate) fn enters_as_value(&self, schema: &Schema, path: &Path) -> Option<ValueFormat> {
        let as_spelled = ReadName::as_spelled(schema, path);
        let entered = FileReading::of(self.compiled, schema, as_spelled).value_format();
        if entered.is_some() {
            self.run.spellings.resolve(path);
        }
        entered
    }

    /// Whether the file `spelling` reached is read by its own name, which no second name can
    /// contradict (§FS-scan.checks.reading-conflict). A narrow run asks this before it pays for
    /// the project walk that collects a file's other names
    /// (§FS-scan.identity.reading.every-caller).
    pub(crate) fn own_name_decides(&self, schema: &Schema, physical: &Path) -> bool {
        physical.starts_with(self.run.spellings.physical_root(self.root()))
            && claim(schema, physical).is_some()
    }

    /// What the names of one file come to (§FS-scan.checks.reading-conflict,
    /// §FS-scan.checks.reading-moved), each read through the rule, in the names' order.
    pub(crate) fn judge_names(&self, schema: &Schema, names: &[PathBuf]) -> NamesVerdict {
        let mut readings = Vec::new();
        for name in names {
            let label = self.read_name(schema, name).label();
            if !readings.contains(&label) {
                readings.push(label);
            }
        }
        if readings.len() > 1 {
            return NamesVerdict::Conflict { readings };
        }
        let Some(first) = names.first() else {
            return NamesVerdict::Agree;
        };
        let was = ReadName::as_spelled(schema, first).label();
        match readings.pop() {
            Some(now) if now != was => NamesVerdict::Moved { was, now },
            _ => NamesVerdict::Agree,
        }
    }
}

#[cfg(all(test, unix))]
#[path = "tests_read_name.rs"]
mod tests;
