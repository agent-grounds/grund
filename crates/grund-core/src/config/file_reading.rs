//! How one file is read (§FS-config-v2.schema.sources): the one answer every
//! consumer asks of a path. That is its reader, the hosts and grammar its comments
//! are read through, the recognizers that measure and gate its comment blocks, and
//! the string forms it is read with. A v2 file takes all of them from the language
//! its name resolves to, so a remapped suffix is read as its target is, whole
//! (§FS-config-v2.schema.sources.extensions). A v1 file, or one no admitted
//! language claims, takes them from its extension and the project's `[scan]` keys
//! (§FS-config.5.2). Nothing else derives any of them from a path
//! (§AR-system.principle).

use std::path::Path;

use super::compiled::Compiled;
use super::project::Schema;
use super::sources::{DeclarationMode, Language, LanguageClass};
use crate::grammar::{DocCommentRule, LexicalSettings, MultiLineStrings, extension_rule};
use crate::model::ValueFormat;

/// How the text scan, the value homes, the formatter, the editor, the checker and
/// `show` read one file (§FS-config-v2.schema.sources).
#[derive(Clone, Copy)]
pub(crate) struct FileReading<'a> {
    /// Read with the Markdown reader.
    pub(crate) markdown: bool,
    /// Read for Python's docstring positions: a v1 `.py`, or a v2 language that
    /// reads docstrings.
    pub(crate) python: bool,
    /// The file's v2 language, `None` for a v1 file or one no language claims.
    pub(crate) language: Option<&'a Language>,
    /// `[schema.sources] declarations`, `None` for a v1 file.
    declarations: Option<DeclarationMode>,
    /// The final extension, which keys a v1 file's recognizer and string forms.
    extension: Option<&'a str>,
    /// The effective `[scan] extensions`, whose claim on a YAML file comes before
    /// a value home's (§FS-values.2.2).
    scanned_extensions: &'a [String],
    /// The marker, the hosts, the docstring reading and the grammar built over
    /// those hosts: a v2 file's are its language's, never another's
    /// (§FS-languages.prefixes).
    pub(crate) lexical: LexicalSettings<'a>,
}

impl<'a> FileReading<'a> {
    pub(super) fn of(compiled: &'a Compiled, schema: &'a Schema, path: &'a Path) -> Self {
        let extension = path.extension().and_then(|ext| ext.to_str());
        let mut lexical = compiled.lexical(schema);
        let model = schema.sources.model.as_ref();
        match model.and_then(|model| Some((model, model.language_of(path)?))) {
            Some((model, language)) => {
                lexical.comment_prefixes = &language.comment_prefixes;
                lexical.docstring_python = language.docstrings;
                lexical.grammar = compiled.grammar_for(&language.comment_prefixes);
                Self {
                    markdown: language.class == LanguageClass::Document,
                    python: language.docstrings,
                    language: Some(language),
                    declarations: Some(model.declarations),
                    extension,
                    scanned_extensions: &schema.sources.extensions,
                    lexical,
                }
            }
            None => Self {
                markdown: extension == Some("md"),
                python: extension == Some("py"),
                language: None,
                declarations: None,
                extension,
                scanned_extensions: &schema.sources.extensions,
                lexical,
            },
        }
    }

    /// The format a value home reads this file in, `None` where the file is no
    /// value source (§FS-values.terms, §FS-config-v2.schema.sources.values): a v2
    /// file whose language is a `data` one is JSON whatever its name; a v1 file,
    /// or one no admitted language claims, is JSON by `.json` and YAML by `.yaml`
    /// or `.yml`, unless the effective `[scan] extensions` claim it as scan input
    /// (§FS-values.2.2).
    pub(crate) fn value_format(&self) -> Option<ValueFormat> {
        match (self.language, self.extension) {
            (Some(language), _) => {
                (language.class == LanguageClass::Data).then_some(ValueFormat::Json)
            }
            (None, Some("json")) => Some(ValueFormat::Json),
            (None, Some(extension @ ("yaml" | "yml"))) => (!self
                .scanned_extensions
                .iter()
                .any(|listed| listed == extension))
            .then_some(ValueFormat::Yaml),
            (None, _) => None,
        }
    }

    /// The recognizer the inline-style measurement asks
    /// (§FS-inline-citation-style.1.1.2): a v2 file's language's, a v1 file's
    /// extension's.
    pub(crate) fn measurement_rule(&self) -> DocCommentRule {
        match self.language {
            Some(language) => language.measurement_rule(),
            None => self.extension.map_or(DocCommentRule::None, extension_rule),
        }
    }

    /// The frozen profile the declaration gate asks of a v2 block
    /// (§FS-inline-citation-style.1.1.4); `None` where nothing is gated.
    pub(crate) fn declaration_rule(&self) -> Option<DocCommentRule> {
        self.declarations?;
        self.language.map(|language| language.doc_comments.rule())
    }

    /// §FS-config-v2.schema.sources.declarations.eligible: whether a comment block
    /// the declaration rule calls `doc`, or not, may declare. Every v1 block may.
    pub(crate) fn declares(&self, doc: bool) -> bool {
        match (self.declarations, self.language) {
            (Some(mode), Some(language)) => language.declares_in(mode, doc),
            _ => true,
        }
    }

    /// §FS-config-v2.schema.sources.declarations.gate: whether a line in declaration
    /// position may declare, given whether the comment block it sits in may
    /// (`None` outside every block). A Markdown heading and every ungated line may.
    pub(crate) fn line_may_declare(&self, block_declares: Option<bool>) -> bool {
        self.markdown || self.declarations.is_none() || block_declares == Some(true)
    }

    /// The string literals that span lines, by the form row the file's language
    /// names (§FS-languages.multi-line-string.forms): a v2 built-in's own, none
    /// for a definition, and a v1 file's extension's.
    pub(crate) fn literals(&self, text: &str) -> MultiLineStrings {
        MultiLineStrings::of(self.source_forms(), text, self.lexical.docstring_python)
    }

    /// The resolved language suffix for bounded literal and qualified-comment
    /// reading (§FS-config-v2.schema.sources.extensions, §AR-scanner.2.3.2).
    pub(crate) fn source_forms(&self) -> Option<&'a str> {
        match self.language {
            Some(language) if language.builtin => language.suffixes.first().map(String::as_str),
            Some(_) => None,
            None => self.extension,
        }
    }
}
