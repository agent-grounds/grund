//! `[schema.sources]` and its two tables (§FS-config-v2.schema.sources): each key
//! read into a draft at the line it was written, and the checks that need the
//! whole file judged in `judge`. `sources_model.rs` lowers the judged drafts into
//! the one source model (`config/sources.rs`) and renders it back out.

use anyhow::Result;
use ignore::gitignore::GitignoreBuilder;

use super::walk::Reader;
use crate::config::sources::{DeclarationMode, LanguageClass, builtin, default_languages};
use crate::config::v1::parse_bool;
use crate::grammar::DocProfile;

/// What `[schema.sources]` and its tables wrote, each value at its line.
#[derive(Default)]
pub(super) struct SourcesDraft {
    pub(super) languages: Option<(Vec<String>, usize)>,
    pub(super) declarations: Option<DeclarationMode>,
    pub(super) exclude: Vec<String>,
    /// `[schema.sources.extensions]`: suffix, language, line.
    pub(super) extensions: Vec<(String, String, usize)>,
    pub(super) definitions: Vec<DefinitionDraft>,
}

/// One `[schema.sources.definitions.<name>]` as written.
pub(super) struct DefinitionDraft {
    pub(super) name: String,
    line: usize,
    pub(super) class: Option<(LanguageClass, usize)>,
    pub(super) prefixes: Option<(Vec<String>, usize)>,
    pub(super) doc_comments: Option<(DocProfile, usize)>,
    pub(super) docstrings: Option<(bool, usize)>,
}

/// §FS-config-v2.schema.sources: one key of `[schema.sources]`.
pub(super) fn sources_key(r: &mut Reader, key: &str, value: &str, line: usize) -> Result<bool> {
    match key {
        // §FS-config-v2.schema.sources.languages: judged once the definitions are known.
        "languages" => {
            let languages = r.string_list(line, value)?;
            r.sources.languages = Some((languages, line));
        }
        "declarations" => {
            let mode = r.string(line, value)?;
            r.sources.declarations = Some(match mode.as_str() {
                "doc-comments" => DeclarationMode::DocComments,
                "any-comment" => DeclarationMode::AnyComment,
                _ => {
                    return r.fail(
                        line,
                        format!(
                            "unknown [schema.sources] declarations `{mode}` (expected doc-comments or any-comment)"
                        ),
                    );
                }
            });
        }
        // §FS-config-v2.schema.sources.exclude: the formatter's glob grammar, refused at its line.
        "exclude" => {
            let patterns = r.string_list(line, value)?;
            let mut builder = GitignoreBuilder::new("");
            for pattern in &patterns {
                let unclosed = unclosed_class(pattern).then_some("unclosed character class");
                if let Some(err) = unclosed.map(str::to_string).or_else(|| {
                    builder
                        .add_line(None, pattern)
                        .err()
                        .map(|err| err.to_string())
                }) {
                    let message =
                        format!("invalid [schema.sources] exclude pattern `{pattern}`: {err}");
                    return r.fail(line, message);
                }
            }
            if let Err(err) = builder.build() {
                return r.fail(line, format!("invalid [schema.sources] exclude: {err}"));
            }
            r.sources.exclude = patterns;
        }
        "respect_gitignore" => {
            r.project.schema.sources.respect_gitignore = parse_bool(r.path, line, value)?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

/// Whether `pattern` opens a `[` class it never closes. The glob builder reads
/// such a pattern as a literal, where git matches nothing with it, so the reader
/// refuses it as the malformed pattern it is (§FS-config-v2.schema.sources.exclude).
fn unclosed_class(pattern: &str) -> bool {
    let chars: Vec<char> = pattern.chars().collect();
    let mut at = 0;
    while at < chars.len() {
        match chars[at] {
            '\\' => at += 1,
            '[' => {
                let mut end = at + 1;
                if matches!(chars.get(end), Some('!' | '^')) {
                    end += 1;
                }
                // A `]` first in the class is a member, not its close.
                if chars.get(end) == Some(&']') {
                    end += 1;
                }
                match chars[end.min(chars.len())..].iter().position(|c| *c == ']') {
                    Some(close) => at = end + close,
                    None => return true,
                }
            }
            _ => {}
        }
        at += 1;
    }
    false
}

/// The glob characters a suffix may not hold (§FS-config-v2.schema.sources.extensions),
/// which also mark a place written as a pattern (§FS-check.3.1.4).
pub(crate) fn holds_glob(text: &str) -> bool {
    text.contains(['*', '?', '[', ']', '{', '}', '!'])
}

/// §FS-config-v2.schema.sources.extensions: a final extension, no dot, no
/// separator, no glob — the one judge of a suffix, at load and in migration.
pub(crate) fn literal_suffix(suffix: &str) -> bool {
    !suffix.is_empty() && !suffix.contains(['.', '/', '\\']) && !holds_glob(suffix)
}

/// §FS-config-v2.schema.sources.extensions: one suffix mapping. The suffix is
/// judged here; the language it names once every definition is read.
pub(super) fn extension_key(r: &mut Reader, key: &str, value: &str, line: usize) -> Result<bool> {
    if !literal_suffix(key) {
        return r.fail(
            line,
            format!(
                "invalid suffix `{key}` in [schema.sources.extensions] (expected a final extension with no dot, separator or glob)"
            ),
        );
    }
    let language = r.string(line, value)?;
    r.sources.extensions.push((key.to_string(), language, line));
    Ok(true)
}

/// §FS-config-v2.schema.sources.definitions: open `[schema.sources.definitions.<name>]`,
/// refusing a name outside the grammar or a built-in's at the header.
pub(super) fn open_definition(r: &mut Reader, name: &str, line: usize) -> Result<usize> {
    let grammar = name.starts_with(|c: char| c.is_ascii_lowercase())
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
    if !grammar {
        return r.fail(
            line,
            format!("invalid language name `{name}` (expected [a-z][a-z0-9_]*)"),
        );
    }
    if builtin(name).is_some() {
        return r.fail(
            line,
            format!(
                "[schema.sources.definitions.{name}] cannot redefine the built-in language `{name}`"
            ),
        );
    }
    r.sources.definitions.push(DefinitionDraft {
        name: name.to_string(),
        line,
        class: None,
        prefixes: None,
        doc_comments: None,
        docstrings: None,
    });
    Ok(r.sources.definitions.len() - 1)
}

/// §FS-config-v2.schema.sources.definitions: one key of the definition at `index`.
pub(super) fn definition_key(
    r: &mut Reader,
    index: usize,
    key: &str,
    value: &str,
    line: usize,
) -> Result<bool> {
    let table = format!(
        "[schema.sources.definitions.{}]",
        r.sources.definitions[index].name
    );
    let unknown = |what: &str, expected: &str| {
        format!("unknown {table} {key} `{what}` (expected {expected})")
    };
    match key {
        "class" => {
            let class = r.string(line, value)?;
            let Some(parsed) = LanguageClass::ALL.into_iter().find(|c| c.name() == class) else {
                return r.fail(line, unknown(&class, "document, data, or source"));
            };
            r.sources.definitions[index].class = Some((parsed, line));
        }
        "comment_prefixes" => {
            let prefixes = r.string_list(line, value)?;
            for (at, prefix) in prefixes.iter().enumerate() {
                if prefix.trim().is_empty() || prefix.contains(['\n', '\r']) {
                    let message = format!(
                        "{table} comment_prefixes takes nonempty, single-line prefixes, not `{prefix}`"
                    );
                    return r.fail(line, message);
                }
                if prefixes[..at].contains(prefix) {
                    let message = format!("{table} comment_prefixes lists `{prefix}` twice");
                    return r.fail(line, message);
                }
            }
            r.sources.definitions[index].prefixes = Some((prefixes, line));
        }
        "doc_comments" => {
            let profile = r.string(line, value)?;
            let Some(parsed) = DocProfile::parse(&profile) else {
                let names: Vec<&str> = DocProfile::ALL.iter().map(|p| p.name()).collect();
                return r.fail(line, unknown(&profile, &names.join(", ")));
            };
            r.sources.definitions[index].doc_comments = Some((parsed, line));
        }
        "docstrings" => {
            let docstrings = r.string(line, value)?;
            let python = match docstrings.as_str() {
                "off" => false,
                "python" => true,
                _ => return r.fail(line, unknown(&docstrings, "off or python")),
            };
            r.sources.definitions[index].docstrings = Some((python, line));
        }
        _ => return Ok(false),
    }
    Ok(true)
}

/// The checks that need the whole file (§FS-config-v2.schema.sources): every name
/// `languages` and `[schema.sources.extensions]` use against the built-ins and the
/// definitions, and each definition's keys together. Each lands at its line.
pub(super) fn judge(draft: &SourcesDraft, late: &mut Vec<(usize, String)>) {
    let defined = |name: &str| draft.definitions.iter().any(|def| def.name == name);
    let known = |name: &str| builtin(name).is_some() || defined(name);
    if let Some((languages, line)) = &draft.languages {
        for (at, name) in languages.iter().enumerate() {
            if !known(name) {
                let message = format!("unknown language `{name}` in [schema.sources] languages");
                late.push((*line, message));
            } else if languages[..at].contains(name) {
                let message =
                    format!("language `{name}` is listed twice in [schema.sources] languages");
                late.push((*line, message));
            }
        }
    }
    let admitted = admitted_names(draft);
    for (suffix, name, line) in &draft.extensions {
        if !known(name) {
            let message = format!("unknown language `{name}` in [schema.sources.extensions]");
            late.push((*line, message));
        } else if !admitted.iter().any(|admitted| admitted == name) {
            let message = format!(
                "[schema.sources.extensions] {suffix} maps to `{name}`, which [schema.sources] languages does not admit"
            );
            late.push((*line, message));
        }
    }
    for def in &draft.definitions {
        judge_definition(def, late);
    }
}

/// §FS-config-v2.schema.sources.definitions: what one definition's keys must say together.
fn judge_definition(def: &DefinitionDraft, late: &mut Vec<(usize, String)>) {
    let table = format!("[schema.sources.definitions.{}]", def.name);
    let Some((class, _)) = def.class else {
        late.push((def.line, format!("{table} needs `class`")));
        return;
    };
    let prefixes = def.prefixes.as_ref().map_or(&[][..], |(p, _)| p.as_slice());
    let docstrings = def.docstrings.is_some_and(|(python, _)| python);
    let refuse = |key: &str, line: usize| {
        let message = format!("{table} class = \"{}\" takes no {key}", class.name());
        (line, message)
    };
    if class != LanguageClass::Source {
        if let Some((profile, line)) = def.doc_comments
            && profile != DocProfile::None
        {
            late.push(refuse("doc_comments", line));
        }
        if let Some((true, line)) = def.docstrings {
            late.push(refuse("docstrings", line));
        }
        if let Some((prefixes, line)) = &def.prefixes
            && class == LanguageClass::Data
            && !prefixes.is_empty()
        {
            late.push(refuse("comment_prefixes", *line));
        }
        return;
    }
    if prefixes.is_empty() && !docstrings {
        let message = format!(
            "{table} reads no comment: give it comment_prefixes or docstrings = \"python\""
        );
        late.push((def.line, message));
        return;
    }
    let Some((profile, line)) = def.doc_comments else {
        return;
    };
    if profile == DocProfile::Python && !docstrings {
        let message = format!("{table} doc_comments = \"python\" needs docstrings = \"python\"");
        late.push((line, message));
    } else if !profile.hosts().is_empty()
        && !profile
            .hosts()
            .iter()
            .any(|host| prefixes.iter().any(|prefix| prefix == host))
    {
        let message = format!(
            "doc_comments = \"{}\" is unreachable: no comment prefix of {table} spells it",
            profile.name()
        );
        late.push((line, message));
    }
}

/// The names `languages` admits: the list as written, or the epoch's default rows.
pub(super) fn admitted_names(draft: &SourcesDraft) -> Vec<String> {
    match &draft.languages {
        Some((languages, _)) => languages.clone(),
        None => default_languages().map(str::to_string).collect(),
    }
}
