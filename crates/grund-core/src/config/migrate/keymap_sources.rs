//! §FS-config-migrate.mapping: the `mapped:` spellings of `[scan]`'s source keys, `extensions`,
//! `comment_prefixes` and `docstring_python`. Each names the built-in languages and the
//! definitions the candidate reads those suffixes through
//! (§FS-config-v2.schema.sources.migration), with full table addresses for comment guidance
//! (§FS-config-migrate.2.2).

use super::keymap::{ALWAYS, joined};
use crate::config::file_reading::model_language;
use crate::config::project::Project;

/// §FS-config-migrate.2.2: both reports name the same source definitions;
/// comment guidance spells their full table addresses for manual placement.
fn definitions(v2: &Project, full_addresses: bool) -> String {
    let names: Vec<String> = v2
        .schema
        .sources
        .model
        .iter()
        .flat_map(|model| model.definitions.iter())
        .map(|def| {
            if full_addresses {
                format!("[schema.sources.definitions.{}]", def.name)
            } else {
                def.name.clone()
            }
        })
        .collect();
    let word = if names.len() == 1 {
        "definition"
    } else {
        "definitions"
    };
    format!("{word} {}", joined(&names))
}

/// §FS-config-migrate.mapping `[scan] extensions` and `[scan] comment_prefixes`: the built-in
/// rows the candidate's `languages` names, in its order, as `built-in A` or `built-ins A and B`;
/// `None` when it names none, so a file with no built-in reads as it did before
/// agent-grounds/grund#1384.
fn builtins(v2: &Project) -> Option<String> {
    let names: Vec<String> = v2
        .schema
        .sources
        .model
        .iter()
        .flat_map(|model| model.admitted.iter())
        .filter(|language| language.builtin)
        .map(|language| language.name.clone())
        .collect();
    match names.len() {
        0 => None,
        1 => Some(format!("built-in {}", names[0])),
        _ => Some(format!("built-ins {}", joined(&names))),
    }
}

/// Whether the candidate writes any definition.
fn has_definitions(v2: &Project) -> bool {
    v2.schema
        .sources
        .model
        .as_ref()
        .is_some_and(|model| !model.definitions.is_empty())
}

/// §FS-config-migrate.mapping `[scan] extensions`: the built-ins, then the definitions.
pub(super) fn extensions(v2: &Project, full_addresses: bool) -> String {
    match builtins(v2) {
        None => format!(
            "[schema.sources] languages and extensions, {}",
            definitions(v2, full_addresses)
        ),
        Some(builtins) if !has_definitions(v2) => format!("[schema.sources] languages, {builtins}"),
        Some(builtins) => format!(
            "[schema.sources] languages and extensions, {builtins}, {}",
            definitions(v2, full_addresses)
        ),
    }
}

/// §FS-config-migrate.mapping `[scan] comment_prefixes`: each definition's list, and the comment
/// hosts of the built-ins, which equal it as a set.
pub(super) fn comment_prefixes(v2: &Project, full_addresses: bool) -> String {
    let definitions = format!("comment_prefixes of {}", definitions(v2, full_addresses));
    match builtins(v2) {
        None => definitions,
        Some(builtins) if !has_definitions(v2) => format!("the comment hosts of {builtins}"),
        Some(builtins) => format!("{definitions}, and the comment hosts of {builtins}"),
    }
}

/// `[scan] docstring_python`'s spelling: the definition `.py` maps to, which
/// reads docstrings only when the key was on, or nothing when `.py` is not read.
pub(super) fn python_docstrings(v2: &Project, full_addresses: bool) -> String {
    let Some(model) = &v2.schema.sources.model else {
        return ALWAYS.into();
    };
    let definition = model
        .extensions
        .iter()
        .find(|(suffix, _)| suffix == "py")
        .and_then(|(_, name)| model.definitions.iter().find(|def| def.name == *name));
    match definition {
        Some(def) => {
            let name = if full_addresses {
                format!("[schema.sources.definitions.{}]", def.name)
            } else {
                def.name.clone()
            };
            let state = if def.docstrings { "" } else { " (off)" };
            format!("docstrings of definition {name}{state}")
        }
        // §AR-config.1.6: whether `.py` is read is the file reading's answer, not a second one.
        None if model_language(model, std::path::Path::new("x.py")).is_some() => {
            "docstrings of built-in python".into()
        }
        None if full_addresses => ALWAYS.into(),
        None => "nothing, [scan] extensions does not read `py`".into(),
    }
}
