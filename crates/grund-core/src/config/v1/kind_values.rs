//! What the two value-related `[[kinds]]` keys require of the row that sets
//! them (§FS-config.3.4.9, §FS-config.3.4.13), beside `kind_table.rs`'s
//! whole-list rules (§AR-core-module-layout.1). Both are structural: neither
//! reads a declaration's content, and each refusal names a state in which no
//! value coordinate could ever exist. The v1 reader asks them per entry, in
//! the same pass as the row refusals they are interleaved with, because the
//! home they check is read off the filesystem under the config root
//! (§AR-config.4). The v2 reader asks the same ones of `form` and a values
//! field, so one spelling has one set of prerequisites (§FS-fields.forms).

use anyhow::Result;
use std::fs;
use std::path::Path;

use super::values::bail_config;
use crate::config::kind::KindConfig;
use crate::config::project::Sources;
use crate::model::normalize_path_lexically;

/// The lines a row's value refusals are located at: the key that wrote each,
/// or the row's header (§FS-config.4.3).
pub(in crate::config) struct ValueLines {
    pub(in crate::config) header: usize,
    pub(in crate::config) values: Option<usize>,
    pub(in crate::config) value_chapter: Option<usize>,
}

/// Every relationship `values` and `value_chapter` need of their own row
/// (§FS-config.3.4.9, §FS-config.3.4.13, §FS-fields.forms). Located at the
/// key's own line, so the message opens the line that has to change and the
/// editor marks that same line (§FS-lsp.2.2.4.1). `sources` is what the project
/// reads, which an absent home's hint asks (§FS-fetch.4.1).
pub(in crate::config) fn validate_kind_value_keys(
    path: &Path,
    k: &KindConfig,
    lines: &ValueLines,
    root: &Path,
    named_sections: bool,
    sources: &Sources,
) -> Result<()> {
    if k.values {
        let line = lines.values.unwrap_or(lines.header);
        if !k.citable {
            return bail_config(
                path,
                line,
                format!(
                    "kind `{}` sets `values = true` with `citable = false`",
                    k.kind
                ),
            );
        }
        let (home, expects_file) = match (&k.file, &k.folder) {
            (Some(home), None) => (home, true),
            (None, Some(home)) => (home, false),
            _ => {
                return bail_config(
                    path,
                    line,
                    format!(
                        "kind `{}` sets `values = true` without exactly one `file` or `folder` home",
                        k.kind
                    ),
                );
            }
        };
        validate_value_home(path, line, root, k, home, expects_file, sources)?;
    }
    // §FS-config.3.4.13: the chapter key's own relationships. Each is a
    // state in which no coordinate could ever exist, so the config is
    // refused rather than left to produce a chapter nothing reads.
    if k.value_chapter.is_some() {
        let line = lines.value_chapter.unwrap_or(lines.header);
        // §FS-config.3.2.7 is a prerequisite, not a consequence: without
        // named sections the handle this key names is not even a section.
        if !named_sections {
            return bail_config(
                path,
                line,
                "[[kinds]] sets `value_chapter` but [id] named_sections is not true".to_string(),
            );
        }
        if !k.citable {
            return bail_config(
                path,
                line,
                format!(
                    "kind `{}` sets `value_chapter` with `citable = false`",
                    k.kind
                ),
            );
        }
        if !k.scan {
            return bail_config(
                path,
                line,
                format!(
                    "kind `{}` sets `value_chapter` with `scan = false` (an unwalked place is never read, so its chapter could never be seen)",
                    k.kind
                ),
            );
        }
        // §FS-values.2.1: a whole-declaration value's immediate children are
        // a contiguous numeric run, so a named chapter cannot legally exist
        // inside one — the pair is refused rather than merged.
        if k.values {
            return bail_config(
                path,
                line,
                format!(
                    "kind `{}` sets `value_chapter` and `values = true` (a whole-declaration value admits no named chapter)",
                    k.kind
                ),
            );
        }
        let (home, expects_file) = match (&k.file, &k.folder) {
            (Some(home), None) => (home, true),
            (None, Some(home)) => (home, false),
            _ => {
                return bail_config(
                    path,
                    line,
                    format!(
                        "kind `{}` sets `value_chapter` without exactly one `file` or `folder` home",
                        k.kind
                    ),
                );
            }
        };
        validate_value_home(path, line, root, k, home, expects_file, sources)?;
    }
    Ok(())
}

/// §FS-config-v2.schema.sources.rules: the name test on a row's first `file` home:
/// version 1's whole answer, because `.md` is the one suffix it reads as Markdown, and
/// version 2's only for an entry a `source` language reads (§DF-v2-rule-home-by-language.2.3).
pub(in crate::config) fn file_home_is_named_markdown(k: &KindConfig) -> bool {
    k.file
        .as_deref()
        .is_none_or(|file| Path::new(file).extension().and_then(|ext| ext.to_str()) == Some("md"))
}

/// §FS-config.3.4.12, §FS-fields.forms: why a rule kind's row cannot hold
/// rules, if it cannot — the refusal v1 writes unlocated and v2 at `form`.
/// `file_homes_are_markdown` is the reader's answer for the row's `file` homes,
/// which each version gives its own way (§FS-config-v2.schema.sources.rules).
pub(in crate::config) fn rule_kind_refusal(
    k: &KindConfig,
    file_homes_are_markdown: bool,
) -> Option<String> {
    let refused = k.rules
        && (!k.citable
            || !k.scan
            || (k.folder.is_none() && k.file.is_none())
            || !file_homes_are_markdown
            || k.values
            || k.fetch.is_some());
    refused.then(|| {
        format!(
            "kind `{}` sets `rules = true` but rule kinds must be citable, scanned Markdown kinds with a `file` or `folder` home and without `values` or `fetch`",
            k.kind
        )
    })
}

/// §FS-fetch.4.1: whether the project reads what `grund fetch` writes at this home as
/// Markdown. In v2 the language decides, of the `file` home or of the `<ID>.md` a `folder`
/// home gets (§FS-fetch.5), and a name no language claims is read by nothing. v1 has no
/// language to ask: the name must be Markdown's, and its effective `[scan] extensions`
/// must list `md`, or nothing reads the file (§FS-scan.scope.extensions).
fn snapshot_read_as_markdown(k: &KindConfig, sources: &Sources) -> bool {
    let Some(model) = &sources.model else {
        return file_home_is_named_markdown(k) && sources.extensions.iter().any(|ext| ext == "md");
    };
    // Any stem will do: a language goes by the final extension.
    let written = k.file.as_deref().unwrap_or("<ID>.md");
    model
        .language_of(Path::new(written))
        .is_some_and(|language| language.reads_markdown())
}

/// §FS-fetch.4.1: the one step past an absent home. Outside the root no step helps, so
/// that hint names another path. Inside it, `grund fetch` writes Markdown, so it is the
/// way through only for a kind that sets it and a home whose snapshot is read as Markdown.
fn absent_value_home_hint(
    k: &KindConfig,
    shape: &str,
    inside_root: bool,
    sources: &Sources,
) -> String {
    if !inside_root {
        "hint: a value kind's home must be inside the project root; name a path inside it"
            .to_string()
    } else if k.fetch.is_some() && snapshot_read_as_markdown(k, sources) {
        format!(
            "hint: a value kind's home must exist; create the empty {shape}, then run `grund fetch <ID>` to write the first snapshot"
        )
    } else {
        format!("hint: a value kind's home must exist; create the {shape}")
    }
}

/// A value home is mandatory, existing, and physically inside the project root
/// (§FS-config.3.4.9, §FS-values.1). Resolving both paths closes `..`, absolute,
/// and symlink escapes through the same located config error, with its named
/// line preserved for the editor (§FS-lsp.2.2.4.1).
fn validate_value_home(
    config_path: &Path,
    line: usize,
    root: &Path,
    k: &KindConfig,
    home: &str,
    expects_file: bool,
    sources: &Sources,
) -> Result<()> {
    let kind = &k.kind;
    let shape = if expects_file { "file" } else { "folder" };
    let candidate = normalize_path_lexically(&root.join(home));
    let root = fs::canonicalize(root).unwrap_or_else(|_| normalize_path_lexically(root));
    // §FS-fetch.4.1: the first line stays as it is; the hint follows it.
    let resolved = fs::canonicalize(&candidate).or_else(|_| {
        let inside_root = candidate.starts_with(&root);
        bail_config(
            config_path,
            line,
            format!(
                "value home for kind `{kind}` does not exist: {home}\n{}",
                absent_value_home_hint(k, shape, inside_root, sources)
            ),
        )
    })?;
    if !resolved.starts_with(&root) {
        return bail_config(
            config_path,
            line,
            format!("value home for kind `{kind}` must normalize inside the project root: {home}"),
        );
    }
    if (expects_file && !resolved.is_file()) || (!expects_file && !resolved.is_dir()) {
        return bail_config(
            config_path,
            line,
            format!("value home for kind `{kind}` must be an existing {shape}: {home}"),
        );
    }
    Ok(())
}
