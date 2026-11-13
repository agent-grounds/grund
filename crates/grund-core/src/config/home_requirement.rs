//! What a form asks of one home, judged in one function for every `files` and `folders` entry of
//! a row, by both loaders (§AR-config.1.6): a rule kind's homes by how the configuration reads
//! them (§FS-config-v2.schema.sources.rules, §FS-config.3.4.12), and a value kind's by whether
//! they exist inside the root (§FS-config.3.4.9). The first entry is not the only one asked, and
//! the order of the entries moves no verdict but the one §FS-config.3.4.9 gives the first.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use super::fetch_key_runs_nothing;
use super::file_reading::{Readings, model_language, written_language};
use super::kind::KindConfig;
use super::project::Sources;
use super::sources::{LanguageClass, SourceModel};
use crate::model::{Code, normalize_path_lexically, remedies};
use crate::sources::io_reason;

/// The clause every load warning that grund 1.0.0 turns into a refusal ends with
/// (§FS-config-v2.checks.config-md-not-markdown, §REQ-backwards-compatibility.2).
pub(in crate::config) const UNTIL_1_0: &str = "; this becomes an error in grund 1.0.0";

/// One home of a row as written: its path under the config root, whether a `files` entry lists
/// it, whether it is the row's first, and the line of the key that lists it.
#[derive(Clone, Copy)]
pub(in crate::config) struct Home<'a> {
    pub(in crate::config) path: &'a str,
    pub(in crate::config) file: bool,
    pub(in crate::config) first: bool,
    pub(in crate::config) line: usize,
}

/// What a form asks of a home.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(in crate::config) enum Ask {
    /// `rules = true`, `form = "rule"`: a home whose declarations are Markdown.
    Rule,
    /// `values = true`, `form = "value"`, or a value chapter: a home that exists inside the root.
    Value,
}

/// The answer for one home.
pub(in crate::config) enum Verdict {
    /// The home is what the form asks.
    Holds,
    /// The row cannot hold rules, and is refused with the form's own refusal
    /// ([`rule_kind_message`]).
    RefusesRow,
    /// Refused at the line of the key that asked, with this message.
    Refused(String),
    /// Loads, with this warning at the home's line.
    Warns(Code, String),
}

/// §AR-config.1.6: what `ask` asks of `home`, on the row `k` under `sources` and `root`.
pub(in crate::config) fn home_requirement(
    ask: Ask,
    k: &KindConfig,
    home: Home<'_>,
    root: &Path,
    sources: &Sources,
) -> Verdict {
    match ask {
        Ask::Rule => rule_home(&k.kind, home, sources),
        Ask::Value => value_home(k, home, root, sources),
    }
}

/// §FS-config.3.4.12, §FS-fields.forms: the one refusal of a row that cannot hold rules, which
/// v1 writes unlocated and v2 at `form`.
pub(in crate::config) fn rule_kind_message(kind: &str) -> String {
    format!(
        "kind `{kind}` sets `rules = true` but rule kinds must be citable, scanned Markdown kinds with a `file` or `folder` home and without `values` or `fetch`"
    )
}

/// §FS-config-v2.schema.sources.rules: a `files` entry read as Markdown holds; one a `source`
/// language reads loads with `config-rule-file-not-markdown`; one a `data` language or no
/// admitted language reads refuses the row. A `folders` entry holds unless no suffix could be
/// read as Markdown there, which the configuration alone decides. Version 1 asks the same
/// reading, which is its name, and accepts a folder on its presence.
fn rule_home(kind: &str, home: Home<'_>, sources: &Sources) -> Verdict {
    let model = sources.model.as_ref();
    let path = Path::new(home.path);
    if !home.file {
        return match model {
            Some(model) if !can_hold_markdown(model) => Verdict::Warns(
                Code::CONFIG_RULE_FOLDER_NOT_MARKDOWN,
                format!(
                    "[schema.kinds.{kind}] folders entry `{}` can hold no file read as Markdown, \
                     because no admitted language reads Markdown; {}{UNTIL_1_0}",
                    home.path,
                    remedies::config_rule_folder_not_markdown()
                ),
            ),
            _ => Verdict::Holds,
        };
    }
    if Readings::of_written(sources, path).markdown {
        return Verdict::Holds;
    }
    match written_language(sources, path) {
        Some(language) if language.class == LanguageClass::Source => Verdict::Warns(
            Code::CONFIG_RULE_FILE_NOT_MARKDOWN,
            format!(
                "[schema.kinds.{kind}] files entry `{}` is read as `{}`, which does not read \
                 Markdown, and every home of a rule kind must be; until then a rule written as a \
                 Markdown heading there declares nothing; {}{UNTIL_1_0}",
                home.path,
                language.name,
                remedies::config_rule_file_not_markdown()
            ),
        ),
        _ => Verdict::RefusesRow,
    }
}

/// §FS-config-v2.checks.config-rule-folder-not-markdown: whether any name could be read as
/// Markdown under `model`, by a `[schema.sources.extensions]` entry or an admitted language's
/// epoch suffix, each asked through the one reading (§FS-fmt.2.3.1).
pub(crate) fn can_hold_markdown(model: &SourceModel) -> bool {
    let mapped = model.extensions.iter().map(|(suffix, _)| suffix);
    let epoch = model
        .admitted
        .iter()
        .flat_map(|language| &language.suffixes);
    mapped.chain(epoch).any(|suffix| {
        let name = format!("name.{suffix}");
        let language = model_language(model, Path::new(&name));
        language.is_some_and(|language| Readings::of(true, Some(language), None).markdown)
    })
}

/// §FS-config.3.4.9, §FS-values.1: a value home is mandatory, existing, and physically inside the
/// project root, and resolving both paths closes `..`, absolute and symlink escapes. A later
/// entry that does not exist loads with `config-value-home-absent` and is not read
/// (§FS-config-v2.checks.config-value-home-absent); one that fails any other of these loads with
/// `config-value-home-invalid` and is read as before (§FS-config-v2.checks.config-value-home-invalid).
fn value_home(k: &KindConfig, home: Home<'_>, root: &Path, sources: &Sources) -> Verdict {
    let kind = &k.kind;
    let shape = if home.file { "file" } else { "folder" };
    let candidate = normalize_path_lexically(&root.join(home.path));
    let canonical_root = fs::canonicalize(root).unwrap_or_else(|_| normalize_path_lexically(root));
    let resolved = match fs::canonicalize(&candidate) {
        Ok(resolved) => resolved,
        // §FS-config-v2.checks.config-value-home-absent: words of the first entry's refusal.
        Err(error) if error.kind() == ErrorKind::NotFound && !home.first => {
            return Verdict::Warns(
                Code::CONFIG_VALUE_HOME_ABSENT,
                format!(
                    "value home for kind `{kind}` does not exist: {}; until then the row reads \
                     its other homes; {}{UNTIL_1_0}",
                    home.path,
                    if home.file {
                        remedies::config_value_home_absent_file()
                    } else {
                        remedies::config_value_home_absent_folder()
                    }
                ),
            );
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            // §FS-fetch.4.1: the first line stays as it is; the hint follows it.
            let inside_root = candidate.starts_with(&canonical_root);
            return Verdict::Refused(format!(
                "value home for kind `{kind}` does not exist: {}\n{}",
                home.path,
                absent_value_home_hint(k, home, root, inside_root, sources)
            ));
        }
        // §FS-fetch.4.3: a lookup that fails otherwise is refused by its reason, as the scan
        // names one (§FS-check.2.4.1), never as absent.
        Err(error) => {
            let refusal = format!(
                "could not look up value home for kind `{kind}`: {}: {}",
                home.path,
                io_reason(&error)
            );
            let hint = remedies::config_value_home_invalid_lookup();
            let first = Verdict::Refused(format!("{refusal}\nhint: {hint}"));
            return later_invalid(home, refusal, remedies::config_value_home_invalid_lookup)
                .unwrap_or(first);
        }
    };
    let (refusal, remedy) = if !resolved.starts_with(&canonical_root) {
        let refusal = format!(
            "value home for kind `{kind}` must normalize inside the project root: {}",
            home.path
        );
        (refusal, remedies::config_value_home_invalid_outside())
    } else if (home.file && !resolved.is_file()) || (!home.file && !resolved.is_dir()) {
        let refusal = format!(
            "value home for kind `{kind}` must be an existing {shape}: {}",
            home.path
        );
        (refusal, remedies::config_value_home_invalid_shape(shape))
    } else {
        return Verdict::Holds;
    };
    let first = Verdict::Refused(refusal.clone());
    later_invalid(home, refusal, || remedy).unwrap_or(first)
}

/// §FS-config-v2.checks.config-value-home-invalid: a later home that fails a check the first is
/// refused for loads with this warning, in the refusal's words, and the walk reads it as grund
/// 0.22.0 did; the first entry is refused (`None`).
fn later_invalid(
    home: Home<'_>,
    refusal: String,
    remedy: impl FnOnce() -> String,
) -> Option<Verdict> {
    (!home.first).then(|| {
        Verdict::Warns(
            Code::CONFIG_VALUE_HOME_INVALID,
            format!(
                "{refusal}; until then it is read as before grund 0.23.0; {}{UNTIL_1_0}",
                remedy()
            ),
        )
    })
}

/// §FS-fetch.4.1: whether the project reads what `grund fetch` writes at this home as Markdown:
/// the `file` home itself, or the `<ID>.md` a `folder` home gets (§FS-fetch.5), each asked
/// through the one reading (§FS-fmt.2.3.1). Version 1 also needs its effective `[scan]
/// extensions` to list `md`, or nothing reads the file.
fn snapshot_read_as_markdown(home: Home<'_>, sources: &Sources) -> bool {
    // Any stem will do: a language goes by the final extension.
    let written = if home.file { home.path } else { "<ID>.md" };
    let read = Readings::of_written(sources, Path::new(written)).markdown;
    match sources.model {
        Some(_) => read,
        None => read && sources.extensions.iter().any(|ext| ext == "md"),
    }
}

/// §FS-fetch.4.1: the one step past an absent home. Outside the root no step helps, so that hint
/// names another path. Inside it, `grund fetch` writes Markdown, so it is the way through only
/// for a key with something to run (§FS-fetch.2.1), asked of the project's own `root`, and a home
/// whose snapshot is read as Markdown.
fn absent_value_home_hint(
    k: &KindConfig,
    home: Home<'_>,
    root: &Path,
    inside_root: bool,
    sources: &Sources,
) -> String {
    let shape = if home.file { "file" } else { "folder" };
    if !inside_root {
        "hint: a value kind's home must be inside the project root; name a path inside it"
            .to_string()
    } else if snapshot_read_as_markdown(home, sources)
        && k.fetch
            .as_deref()
            .is_some_and(|value| !fetch_key_runs_nothing(root, value))
    {
        format!(
            "hint: a value kind's home must exist; create the empty {shape}, then run `grund fetch <ID>` to write the first snapshot"
        )
    } else {
        format!("hint: a value kind's home must exist; create the {shape}")
    }
}

#[cfg(test)]
#[path = "tests_home_requirement.rs"]
mod tests;
