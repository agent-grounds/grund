//! The resolved v2 source model (§FS-config-v2.schema.sources): which languages a
//! v2 project admits, the suffixes that reach each, its comment hosts and doc
//! profile, the declaration mode and the scan's glob exclusions. Built once from
//! the configuration by the v2 reader and read by the walk, the file pass and
//! `config show`, so none of them derives a file's language twice and none of
//! them reads a source file to do it (§FS-config-v2.schema.sources.effective).
//! A v1 project has no model: its `[scan]` keys keep their meaning (§FS-config.5.2).

use std::path::Path;

use crate::grammar::{DocCommentRule, DocProfile, extension_rule};

/// §FS-config-v2.schema.sources.definitions: which reader a language's files take.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LanguageClass {
    /// The Markdown reader.
    Document,
    /// The JSON value-file reader, never the text scan.
    Data,
    /// The source reader, through the language's comment hosts.
    Source,
}

impl LanguageClass {
    pub(crate) const ALL: [LanguageClass; 3] = [Self::Document, Self::Data, Self::Source];

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Data => "data",
            Self::Source => "source",
        }
    }
}

/// §FS-config-v2.schema.sources.declarations: which comment blocks may declare.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DeclarationMode {
    DocComments,
    AnyComment,
}

impl DeclarationMode {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::DocComments => "doc-comments",
            Self::AnyComment => "any-comment",
        }
    }
}

/// One language, built-in or defined: its reader, its comment hosts, its doc
/// profile and whether it reads Python docstrings (§FS-config-v2.schema.sources.epoch).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Language {
    pub(crate) name: String,
    pub(crate) class: LanguageClass,
    /// The epoch suffixes of a built-in row; a definition has none of its own.
    pub(crate) suffixes: Vec<String>,
    pub(crate) comment_prefixes: Vec<String>,
    pub(crate) doc_comments: DocProfile,
    pub(crate) docstrings: bool,
    pub(crate) builtin: bool,
}

impl Language {
    /// The recognizer the inline-style measurement asks (§FS-inline-citation-style.1.1.2):
    /// a built-in row keeps the v1 table's for its own suffix, which may widen, and a
    /// definition takes its profile's.
    pub(crate) fn measurement_rule(&self) -> DocCommentRule {
        match self.suffixes.first() {
            Some(suffix) if self.builtin => extension_rule(suffix),
            _ => self.doc_comments.rule(),
        }
    }

    /// §FS-config-v2.schema.sources.declarations.eligible: whether a comment block
    /// of this language may declare, given whether its profile calls it a doc
    /// comment. A profile of `none` has no notion of documentation, so every block
    /// behind its hosts may.
    pub(crate) fn declares_in(&self, mode: DeclarationMode, doc: bool) -> bool {
        mode == DeclarationMode::AnyComment || self.doc_comments == DocProfile::None || doc
    }
}

/// §FS-config-v2.schema.sources: the resolved model of one v2 file.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SourceModel {
    /// The admitted languages, resolved, in `languages` order.
    pub(crate) admitted: Vec<Language>,
    /// `[schema.sources.extensions]`, suffix to language, in authored order.
    pub(crate) extensions: Vec<(String, String)>,
    /// `[schema.sources.definitions.*]`, in authored order.
    pub(crate) definitions: Vec<Language>,
    pub(crate) declarations: DeclarationMode,
    /// `[schema.sources] exclude`: gitignore-style globs at the config root.
    pub(crate) exclude: Vec<String>,
}

impl SourceModel {
    /// §FS-config-v2.schema.sources: the language `path`'s final extension maps to —
    /// a `[schema.sources.extensions]` entry first, then an admitted row's epoch
    /// suffix — or `None` when no admitted language reads it.
    pub(crate) fn language_of(&self, path: &Path) -> Option<&Language> {
        let extension = path.extension()?.to_str()?;
        if let Some((_, name)) = self.extensions.iter().find(|(key, _)| key == extension) {
            return self.admitted.iter().find(|language| language.name == *name);
        }
        self.admitted.iter().find(|language| {
            language.builtin && language.suffixes.iter().any(|suffix| suffix == extension)
        })
    }

    /// Every suffix the text scan reads: each one an admitted document or source
    /// language takes. A data language's files are read by the value reader alone.
    pub(crate) fn text_suffixes(&self) -> Vec<String> {
        let reads = |name: &str| {
            self.admitted
                .iter()
                .any(|language| language.name == name && language.class != LanguageClass::Data)
        };
        let mut suffixes: Vec<String> = Vec::new();
        for (suffix, name) in &self.extensions {
            if reads(name) {
                suffixes.push(suffix.clone());
            }
        }
        for language in self.admitted.iter().filter(|language| language.builtin) {
            for suffix in &language.suffixes {
                let remapped = self.extensions.iter().any(|(key, _)| key == suffix);
                if !remapped && language.class != LanguageClass::Data {
                    suffixes.push(suffix.clone());
                }
            }
        }
        suffixes
    }
}

/// One built-in row of the epoch table (§FS-config-v2.schema.sources.epoch).
struct Row {
    name: &'static str,
    default: bool,
    class: LanguageClass,
    suffixes: &'static [&'static str],
    prefixes: &'static [&'static str],
    profile: DocProfile,
}

const C_HOSTS: &[&str] = &["//", "/*", "*"];
/// The prefixes a Markdown file reads a comment-led line through, v1's defaults:
/// its own hosts, so a v2 `.md` reads as a v1 one does and no source language's
/// hosts reach it (§FS-languages.prefixes, §FS-config-v2.schema.sources.definitions).
const MARKDOWN_HOSTS: &[&str] = &["//", "#", ";", "--", "*", "/*"];

const fn source(
    name: &'static str,
    suffixes: &'static [&'static str],
    prefixes: &'static [&'static str],
    profile: DocProfile,
) -> Row {
    Row {
        name,
        default: true,
        class: LanguageClass::Source,
        suffixes,
        prefixes,
        profile,
    }
}

const fn opt_in(row: Row) -> Row {
    Row {
        default: false,
        ..row
    }
}

/// §FS-config-v2.schema.sources.epoch: the v2 built-in languages, fixed on
/// 2026-10-06. The default rows come first, in the order an omitted `languages`
/// admits and `config show` prints them; a row is never changed once published.
const EPOCH: &[Row] = &[
    Row {
        name: "markdown",
        default: true,
        class: LanguageClass::Document,
        suffixes: &["md"],
        prefixes: MARKDOWN_HOSTS,
        profile: DocProfile::None,
    },
    Row {
        name: "json",
        default: true,
        class: LanguageClass::Data,
        suffixes: &["json"],
        prefixes: &[],
        profile: DocProfile::None,
    },
    source("rust", &["rs"], C_HOSTS, DocProfile::CFamily),
    source("c", &["c"], C_HOSTS, DocProfile::CFamily),
    source("cpp", &["cpp"], C_HOSTS, DocProfile::CFamily),
    source("java", &["java"], C_HOSTS, DocProfile::CFamily),
    source("kotlin", &["kt"], C_HOSTS, DocProfile::CFamily),
    source("scala", &["scala"], C_HOSTS, DocProfile::CFamily),
    source("swift", &["swift"], C_HOSTS, DocProfile::CFamily),
    source("javascript", &["js"], C_HOSTS, DocProfile::CFamily),
    source("typescript", &["ts", "tsx"], C_HOSTS, DocProfile::CFamily),
    source("csharp", &["cs"], C_HOSTS, DocProfile::CFamily),
    source(
        "php",
        &["php"],
        &["//", "/*", "*", "#"],
        DocProfile::CFamily,
    ),
    source("python", &["py"], &["#"], DocProfile::Python),
    source("go", &["go"], &["//"], DocProfile::Go),
    source("ruby", &["rb"], &["#"], DocProfile::Ruby),
    source("sql", &["sql"], &["--"], DocProfile::Sql),
    source("lua", &["lua"], &["--"], DocProfile::Ldoc),
    source("haskell", &["hs", "lhs"], &["--"], DocProfile::Haddock),
    source("lisp", &["lisp"], &[";"], DocProfile::None),
    source("scheme", &["scm"], &[";"], DocProfile::None),
    source("clojure", &["clj"], &[";"], DocProfile::None),
    source("ada", &["ada", "adb", "ads"], &["--"], DocProfile::None),
    opt_in(source(
        "shell",
        &["sh", "bash", "zsh"],
        &["#"],
        DocProfile::Shell,
    )),
    opt_in(source("r", &["r", "R"], &["#'", "#"], DocProfile::Roxygen)),
    opt_in(source(
        "objectivec",
        &["m", "mm"],
        C_HOSTS,
        DocProfile::CFamily,
    )),
    opt_in(source("dart", &["dart"], C_HOSTS, DocProfile::CFamily)),
];

/// The built-in language `name`, or `None` when the epoch has no such row.
pub(crate) fn builtin(name: &str) -> Option<Language> {
    let row = EPOCH.iter().find(|row| row.name == name)?;
    let strings = |values: &[&str]| values.iter().map(|value| value.to_string()).collect();
    Some(Language {
        name: row.name.to_string(),
        class: row.class,
        suffixes: strings(row.suffixes),
        comment_prefixes: strings(row.prefixes),
        doc_comments: row.profile,
        docstrings: row.profile == DocProfile::Python,
        builtin: true,
    })
}

/// §FS-config-v2.schema.sources.languages: the names an omitted `languages` admits.
pub(crate) fn default_languages() -> impl Iterator<Item = &'static str> {
    EPOCH.iter().filter(|row| row.default).map(|row| row.name)
}
