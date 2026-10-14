//! The judged `[schema.sources]` drafts lowered into the one source model
//! (§FS-config-v2.schema.sources, `config/sources.rs`) the walk, the file pass and
//! `config show` read, and that model rendered back out in v2 spelling
//! (§FS-config-v2.schema.sources.effective). `sources.rs` reads and judges them.

use super::render::{list, string};
use super::sources::admitted_names;
use super::walk::Reader;
use crate::config::project::Sources;
use crate::config::sources::{DeclarationMode, Language, LanguageClass, SourceModel, builtin};
use crate::grammar::DocProfile;

/// §FS-config-v2.schema.sources: the judged drafts as the source model, and the
/// v1-shaped fields the walk reads lowered from it: the suffixes the text scan
/// reads and no v1 component exclusion (§FS-config-v2.defaults.1). No prefix list
/// and no docstring reading are lowered: a v2 file has no global prefix list, and
/// each file reads through its own language's (§FS-languages.prefixes).
pub(super) fn lower(r: &mut Reader) {
    let draft = std::mem::take(&mut r.sources);
    let definitions: Vec<Language> = draft
        .definitions
        .iter()
        .map(|def| Language {
            name: def.name.clone(),
            class: def.class.map_or(LanguageClass::Source, |(class, _)| class),
            suffixes: Vec::new(),
            comment_prefixes: def.prefixes.clone().map(|(p, _)| p).unwrap_or_default(),
            doc_comments: def.doc_comments.map_or(DocProfile::None, |(p, _)| p),
            docstrings: def.docstrings.is_some_and(|(python, _)| python),
            builtin: false,
        })
        .collect();
    let admitted: Vec<Language> = admitted_names(&draft)
        .iter()
        .filter_map(|name| {
            builtin(name).or_else(|| definitions.iter().find(|d| d.name == *name).cloned())
        })
        .collect();
    let model = SourceModel {
        admitted,
        extensions: draft
            .extensions
            .into_iter()
            .map(|(suffix, name, _)| (suffix, name))
            .collect(),
        definitions,
        declarations: draft.declarations.unwrap_or(DeclarationMode::DocComments),
        exclude: draft.exclude,
    };
    let sources = &mut r.project.schema.sources;
    sources.extensions = model.text_suffixes();
    sources.comment_prefixes = Vec::new();
    sources.docstring_python = false;
    sources.exclude = Vec::new();
    sources.model = Some(model);
}

/// §FS-config-v2.schema.sources.effective: `[schema.sources]` and its tables in v2
/// spelling, from the model alone: `languages` written out in full, every key with
/// its default, and the built-in rows left to the epoch.
pub(super) fn render(out: &mut String, sources: &Sources) {
    use std::fmt::Write;
    let Some(model) = &sources.model else {
        return;
    };
    let names: Vec<String> = model.admitted.iter().map(|l| l.name.clone()).collect();
    let _ = writeln!(
        out,
        "\n[schema.sources]\nlanguages = {}\ndeclarations = \"{}\"\nexclude = {}\nrespect_gitignore = {}",
        list(&names),
        model.declarations.name(),
        list(&model.exclude),
        sources.respect_gitignore
    );
    if !model.extensions.is_empty() {
        let _ = writeln!(out, "\n[schema.sources.extensions]");
        for (suffix, name) in &model.extensions {
            let _ = writeln!(out, "{} = {}", key(suffix), string(name));
        }
    }
    for def in &model.definitions {
        let _ = writeln!(
            out,
            "\n[schema.sources.definitions.{}]\nclass = \"{}\"\ncomment_prefixes = {}\ndoc_comments = \"{}\"\ndocstrings = \"{}\"",
            def.name,
            def.class.name(),
            list(&def.comment_prefixes),
            def.doc_comments.name(),
            if def.docstrings { "python" } else { "off" },
        );
    }
}

/// A suffix as a key: bare where TOML allows it, quoted otherwise.
fn key(suffix: &str) -> String {
    let bare = suffix
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    if bare {
        suffix.to_string()
    } else {
        string(suffix)
    }
}
