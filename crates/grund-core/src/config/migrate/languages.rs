//! What a v1 `[scan]` source configuration becomes in version 2
//! (§FS-config-v2.schema.sources.migration): one `v1_<profile>` definition per
//! distinct v1 reading, every suffix mapped to its definition, the v1 names as
//! `<name>/` globs and `declarations = "any-comment"` — and the refusals where no
//! definition can read a suffix as v1 did.

use super::record::{MigrateRefusal, RefusalAnchor};
use crate::config::literal_suffix;
use crate::config::project::Project;
use crate::config::sources::{DeclarationMode, Language, LanguageClass, SourceModel};
use crate::grammar::{DocCommentRule, DocProfile, extension_rule};

/// The source model a v1 project's `[scan]` keys translate to, and what refuses.
pub(super) struct Languages {
    pub(super) model: SourceModel,
    pub(super) refusals: Vec<MigrateRefusal>,
}

/// One v1 reading: the definition a suffix is read through.
fn reading(suffix: &str, docstring_python: bool) -> (String, LanguageClass, DocProfile, bool) {
    if suffix == "md" {
        return (
            "v1_markdown".into(),
            LanguageClass::Document,
            DocProfile::None,
            false,
        );
    }
    let rule = extension_rule(suffix);
    let profile = match rule {
        DocCommentRule::Docstring if !docstring_python => DocProfile::None,
        DocCommentRule::None => DocProfile::None,
        rule => DocProfile::ALL
            .into_iter()
            .find(|profile| profile.rule() == rule)
            .unwrap_or(DocProfile::None),
    };
    let name = match profile {
        DocProfile::None => "v1_plain".to_string(),
        profile => format!("v1_{}", profile.name().replace('-', "_")),
    };
    let docstrings = profile == DocProfile::Python;
    (name, LanguageClass::Source, profile, docstrings)
}

/// §FS-config-v2.schema.sources.migration: the model for `v1`, whose
/// `extensions` and `comment_prefixes` keys sit at `extensions_line` and
/// `prefixes_line`; `values_home` says whether a kind reads `.json` as values.
pub(super) fn translate(
    v1: &Project,
    extensions_line: Option<usize>,
    prefixes_line: Option<usize>,
    values_home: bool,
) -> Languages {
    let sources = &v1.schema.sources;
    let mut refusals = Vec::new();
    let anchor = |line: Option<usize>| line.map_or(RefusalAnchor::Unset, RefusalAnchor::Line);
    let prefixes = &sources.comment_prefixes;
    for (at, prefix) in prefixes.iter().enumerate() {
        if prefix.trim().is_empty()
            || prefix.contains(['\n', '\r'])
            || prefixes[..at].contains(prefix)
        {
            refusals.push(MigrateRefusal {
                anchor: anchor(prefixes_line),
                what: format!(
                    "[scan] comment_prefixes holds `{prefix}`, which a version 2 definition cannot list: an empty or repeated prefix"
                ),
                details: Vec::new(),
                repair: format!("remove `{prefix}` from comment_prefixes under version 1"),
            });
        }
    }
    let mut definitions: Vec<Language> = Vec::new();
    let mut extensions = Vec::new();
    for suffix in &sources.extensions {
        if !literal_suffix(suffix) {
            refusals.push(MigrateRefusal {
                anchor: anchor(extensions_line),
                what: format!(
                    "[scan] extensions names `{suffix}`, which is not a final extension version 2 can map"
                ),
                details: Vec::new(),
                repair: format!("remove `{suffix}` from extensions under version 1"),
            });
            continue;
        }
        if suffix == "json" && values_home {
            refusals.push(MigrateRefusal {
                anchor: anchor(extensions_line),
                what: "[scan] extensions scans `json` as text while a values kind reads it as values, and version 2 gives a suffix one class".into(),
                details: Vec::new(),
                repair: "remove `json` from extensions under version 1".into(),
            });
            continue;
        }
        let (name, class, profile, docstrings) = reading(suffix, sources.docstring_python);
        if class == LanguageClass::Source && prefixes.is_empty() && !docstrings {
            refusals.push(MigrateRefusal {
                anchor: anchor(extensions_line.or(prefixes_line)),
                what: format!(
                    "[scan] extensions names `{suffix}`, which reads no comment with an empty comment_prefixes, and a version 2 definition must read one"
                ),
                details: Vec::new(),
                repair: "list the comment prefixes under version 1, or remove the extension".into(),
            });
            continue;
        }
        let hosts = profile.hosts();
        if !hosts.is_empty() && !hosts.iter().any(|host| prefixes.iter().any(|p| p == host)) {
            refusals.push(MigrateRefusal {
                anchor: anchor(extensions_line.or(prefixes_line)),
                what: format!(
                    "[scan] extensions names `{suffix}`, whose doc comments no prefix of [scan] comment_prefixes spells, and version 2 refuses an unreachable profile"
                ),
                details: Vec::new(),
                repair: format!("add `{}` to comment_prefixes, or remove `{suffix}`, under version 1", hosts[0]),
            });
            continue;
        }
        if !definitions.iter().any(|def| def.name == name) {
            definitions.push(Language {
                name: name.clone(),
                class,
                suffixes: Vec::new(),
                comment_prefixes: prefixes.clone(),
                doc_comments: profile,
                docstrings,
                builtin: false,
            });
        }
        extensions.push((suffix.clone(), name));
    }
    let model = SourceModel {
        admitted: definitions.clone(),
        extensions,
        definitions,
        declarations: DeclarationMode::AnyComment,
        exclude: sources
            .exclude
            .iter()
            .filter(|name| !name.contains('/'))
            .map(|name| format!("{}/", glob_escape(name)))
            .collect(),
    };
    Languages { model, refusals }
}

/// A v1 component name as a literal gitignore-style glob (§FS-config-v2.schema.sources.migration).
fn glob_escape(name: &str) -> String {
    let mut out = String::new();
    for c in name.chars() {
        if matches!(c, '*' | '?' | '[' | ']' | '\\' | '!' | '#') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}
