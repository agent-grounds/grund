//! `grund config explain` (§FS-config-explain, §AR-config.9): what a table, a
//! key or a check code is, in the version the project's file is read as,
//! assembled from what the binary carries — the readers' gates
//! (`config/vocabulary.rs`), the rendered code catalog and the embedded schema.
//! Like migration it renders nothing: it returns one typed answer, which the
//! frontend prints as it prints `config show` (§AR-system.4).

mod schema;
mod target;
#[cfg(test)]
mod tests_answers;
#[cfg(test)]
mod tests_audit;
#[cfg(test)]
mod tests_dotted;
#[cfg(test)]
mod tests_page_parity;

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use super::discovery::{config_file_in, discovery_start_dir};
use super::v1::{is_header, read_sections, strip_comment};
use super::v2;
use super::vocabulary::admitted;
use crate::model::catalog::{self, CodeEnabled, CodeSeverity};
use crate::model::{Code, OperationDiagnostic, format_path};
use target::Resolved;

/// What one call asks (§FS-config-explain.1).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExplainAsk {
    /// No target: the index.
    Index,
    /// One table, key, inline member or code, as it was typed
    /// (§FS-config-explain.1.1).
    Target(String),
    /// `--codes`: every code the version can report.
    Codes,
    /// `--all`: every table with its keys, then every code.
    All,
}

/// One call's answer: in the version the file is read as, or with no file in
/// both, version 2 first (§FS-config-explain.2).
#[derive(Clone, Debug)]
pub struct Explained {
    /// The file the version was read from, spelled from its config root, or
    /// `None` where discovery found no file.
    pub file: Option<String>,
    /// The path the call asked about, spelled as every report spells a path
    /// (§FS-errors.4), for the `Next:` line that repeats it.
    pub path: String,
    /// Each answering version and what it says, in the order it is printed.
    pub answers: Vec<(u32, ExplainAnswer)>,
}

impl Explained {
    /// §FS-config-explain.1.3: no version that answers knows the target, a
    /// query that found nothing.
    pub fn found_nothing(&self) -> bool {
        self.answers
            .iter()
            .all(|(_, answer)| matches!(answer, ExplainAnswer::Unknown(_)))
    }
}

/// What one version says, by rung (§FS-config-explain.1.2).
#[derive(Clone, Debug)]
pub enum ExplainAnswer {
    /// Every table, the top level's first with the header `""`, and how many
    /// codes the version can report.
    Index {
        tables: Vec<ExplainTable>,
        codes: usize,
    },
    Table(ExplainTable),
    Key(ExplainKey),
    Code(ExplainCode),
    Codes(Vec<ExplainCode>),
    All {
        tables: Vec<ExplainTable>,
        codes: Vec<ExplainCode>,
    },
    /// §FS-config-explain.5: a target this version does not know.
    Unknown(ExplainUnknown),
}

/// A table the reader admits, `""` for the top level, what it is for, its keys
/// and the tables directly under it.
#[derive(Clone, Debug)]
pub struct ExplainTable {
    pub header: String,
    pub meaning: Option<String>,
    pub keys: Vec<ExplainKey>,
    pub tables: Vec<String>,
}

/// §FS-config-explain.3: a key or an inline member, under its table's header
/// (`""` at the top level), with its values in words (§FS-config-explain.3.1),
/// what it does, as the embedded schema writes them, and an inline table's
/// members, spelled `key.member`.
#[derive(Clone, Debug)]
pub struct ExplainKey {
    pub table: String,
    pub key: String,
    pub values: String,
    pub meaning: String,
    pub members: Vec<String>,
}

/// §FS-config-explain.4: a code's row of the catalog, read for one version,
/// and the tables its clauses name, which the answer's `Next:` line asks for.
#[derive(Clone, Debug)]
pub struct ExplainCode {
    pub code: String,
    pub meaning: String,
    pub severity: CodeSeverity,
    pub enabled: CodeEnabled,
    pub ramp: Option<String>,
    pub tables: Vec<String>,
}

/// §FS-config-explain.5: why a version has no answer for a target, and what
/// it names instead.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExplainUnknown {
    /// A version-1 table asked under version 2, and the version-2 tables its
    /// settings are written in (§FS-config-v2.reader.1.v1-tables).
    VersionOneTable { table: String, homes: Vec<String> },
    /// A version-2 table asked under version 1.
    VersionTwoTable { table: String },
    /// A key its table does not admit, and the keys it does.
    Key {
        table: String,
        key: String,
        admitted: Vec<String>,
    },
    /// Anything else, a misspelt or a planned code included.
    Other { target: String },
}

/// §FS-config-explain.2: discover the config as `config show` does, read its
/// version and nothing else of it, and answer in that version, or in both
/// with no file.
pub(crate) fn explain(path: &Path, ask: &ExplainAsk) -> Result<Explained> {
    let Some((read, report)) = discover(path) else {
        let answers = vec![(2, answer(2, ask)), (1, answer(1, ask))];
        return Ok(Explained {
            file: None,
            path: format_path(path),
            answers,
        });
    };
    let text =
        fs::read_to_string(&read).with_context(|| format!("read {}", format_path(&report)))?;
    let version = version_of(&text, &report)?;
    Ok(Explained {
        file: Some(format_path(&report)),
        path: format_path(path),
        answers: vec![(version, answer(version, ask))],
    })
}

/// The nearest config file walking up from `path` (§FS-config.4.2), and its
/// path spelled from its root, as every config refusal names it.
fn discover(path: &Path) -> Option<(PathBuf, PathBuf)> {
    let start = discovery_start_dir(path).to_path_buf();
    let start = fs::canonicalize(&start).unwrap_or(start);
    start.ancestors().find_map(|dir| {
        let file = config_file_in(dir)?;
        let report = file.strip_prefix(dir).unwrap_or(&file).to_path_buf();
        Some((file, report))
    })
}

/// §FS-config.5: the version every command reads `text` as, read the way they
/// read it. Only a file whose version cannot be read gets the refusal every
/// command gives it, which is the v1 reader's: a file that names a version this
/// grund does not read, or one the reader stops in at a line that is not TOML.
/// A file whose tables or keys the reader refuses still reads as version 1
/// (§FS-config-explain.2).
fn version_of(text: &str, report: &Path) -> Result<u32> {
    let version = v2::version(text);
    if version == Some("2") {
        return Ok(2);
    }
    match read_sections(text, report) {
        Err(refusal)
            if version.is_some_and(|version| version != "1")
                || refused_outside_toml(text, &refusal) =>
        {
            Err(refusal)
        }
        _ => Ok(1),
    }
}

/// Whether the v1 reader's `refusal` is at a line that is neither a table
/// header nor `key = value`, the two shapes it reads a line in. A value spread
/// over lines is refused at its first line, which holds the `=`.
fn refused_outside_toml(text: &str, refusal: &anyhow::Error) -> bool {
    let at = refusal
        .downcast_ref::<OperationDiagnostic>()
        .and_then(|refusal| refusal.line);
    let line = at.and_then(|at| text.lines().nth(at.checked_sub(1)?));
    line.map(|raw| strip_comment(raw).trim())
        .is_some_and(|line| !is_header(line) && !line.contains('='))
}

/// What `version` says to `ask`.
fn answer(version: u32, ask: &ExplainAsk) -> ExplainAnswer {
    let tables = admitted(version);
    let every_table = || {
        (0..tables.len())
            .map(|at| table(version, &tables, at))
            .collect()
    };
    match ask {
        ExplainAsk::Index => ExplainAnswer::Index {
            tables: every_table(),
            codes: reported(version).count(),
        },
        ExplainAsk::Codes => ExplainAnswer::Codes(
            reported(version)
                .map(|c| code(c, version, &tables))
                .collect(),
        ),
        ExplainAsk::All => ExplainAnswer::All {
            tables: every_table(),
            codes: reported(version)
                .map(|c| code(c, version, &tables))
                .collect(),
        },
        ExplainAsk::Target(asked) => match target::resolve(version, &tables, asked) {
            Resolved::Table(at) => ExplainAnswer::Table(table(version, &tables, at)),
            Resolved::Key(at, name) => ExplainAnswer::Key(key(version, &tables[at], name)),
            Resolved::Code(found) => ExplainAnswer::Code(code(found, version, &tables)),
            Resolved::Unknown(unknown) => ExplainAnswer::Unknown(unknown),
        },
    }
}

/// The codes a file read as `version` can report, in catalog order: every
/// code whose row does not say `not in version N` for it.
fn reported(version: u32) -> impl Iterator<Item = Code> {
    Code::ALL.iter().copied().filter(move |code| {
        !matches!(
            catalog::row(*code).side(version).enabled,
            CodeEnabled::Absent { .. }
        )
    })
}

/// The admitted table at `at`, its keys answered and its child tables named.
fn table(version: u32, tables: &[(String, Vec<String>)], at: usize) -> ExplainTable {
    let (header, keys) = &tables[at];
    let inner = header.trim_matches(['[', ']']);
    let children = tables.iter().map(|(child, _)| child).filter(|child| {
        let rest = child.trim_matches(['[', ']']).strip_prefix(inner);
        !inner.is_empty()
            && rest
                .and_then(|rest| rest.strip_prefix('.'))
                .is_some_and(|rest| !rest.contains('.'))
    });
    ExplainTable {
        header: header.clone(),
        meaning: schema::table_meaning(schema::embedded(), version, header),
        keys: keys
            .iter()
            .map(|name| key(version, &tables[at], name))
            .collect(),
        tables: children.cloned().collect(),
    }
}

/// §FS-config-explain.3: the key `name` of an admitted table, worded from the
/// embedded schema.
fn key(version: u32, (header, keys): &(String, Vec<String>), name: &str) -> ExplainKey {
    let schema = schema::embedded();
    let node = schema::key_node(schema, version, header, name);
    let members = keys.iter().filter(|member| is_member_of(member, name));
    ExplainKey {
        table: header.clone(),
        key: name.to_string(),
        values: node
            .map(|node| schema::value_words(schema, node))
            .unwrap_or_default(),
        meaning: node.and_then(schema::meaning).unwrap_or_default(),
        members: members.cloned().collect(),
    }
}

/// Whether `member`, one of a table's keys, is an inline member of its key
/// `key`, which the vocabulary spells `key.member` (§FS-config-explain.3).
fn is_member_of(member: &str, key: &str) -> bool {
    member
        .strip_prefix(key)
        .is_some_and(|rest| rest.starts_with('.'))
}

/// §FS-config-explain.4: `found`'s row, read for `version`, and the tables
/// its clauses and its severity name that the version admits, in order.
fn code(found: Code, version: u32, admitted: &[(String, Vec<String>)]) -> ExplainCode {
    let row = catalog::row(found);
    let side = row.side(version);
    let severity = match &side.severity {
        CodeSeverity::Strength(text) | CodeSeverity::Setting(text) => Some(text.as_str()),
        _ => None,
    };
    let mut tables: Vec<String> = Vec::new();
    let clauses = side
        .enabled
        .clauses()
        .iter()
        .map(|clause| clause.text.as_str());
    for spelled in clauses.chain(severity).flat_map(target::spelled_tables) {
        if target::admits(admitted, &spelled) && !tables.contains(&spelled) {
            tables.push(spelled);
        }
    }
    ExplainCode {
        code: row.code.clone(),
        meaning: row.meaning.clone(),
        severity: side.severity.clone(),
        enabled: side.enabled.clone(),
        ramp: row.ramp.clone(),
        tables,
    }
}
