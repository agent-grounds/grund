//! §AR-config.10.1: adoption's plan, built from the file's resolved source model and version 2's
//! epoch alone, before any candidate text exists (§FS-config-adopt-builtins.3). [`plan`] takes no
//! candidate and calls no writer, so the expected model cannot follow a writer's mistake
//! (§FS-config-adopt-builtins.6.2). Every reading it compares is the one per-suffix reading
//! (§AR-config.1.6), and the rows are the epoch's own (`claiming`).

use std::collections::BTreeMap;

use super::record::{AddedSuffixes, AdoptedSuffix, KeptSuffix, KeptWhy};
use crate::config::file_reading::{SuffixReading, model_suffix_reading};
use crate::config::sources::{Language, SourceModel, claiming, opt_in_row};

/// §FS-config-adopt-builtins.3: what adoption does with each suffix in play, and the reading
/// every suffix the file reads, or would read, is expected to have.
pub(crate) struct Plan {
    /// Each suffix in play, in entry order.
    pub(crate) entries: Vec<Entry>,
    /// Each adopted row that adds suffixes, in the order of its first adopted entry.
    pub(crate) added: Vec<AddedSuffixes>,
    /// The expected model: each suffix's planned reading (§FS-config-adopt-builtins.terms).
    pub(crate) expected: BTreeMap<String, SuffixReading>,
}

/// §FS-config-adopt-builtins.2: one suffix the file reads through a definition.
pub(crate) struct Entry {
    pub(crate) suffix: String,
    pub(crate) definition: Language,
    pub(crate) decision: Decision,
}

/// §FS-config-adopt-builtins.3.1, §FS-config-adopt-builtins.3.2: adopted into a row, or kept.
pub(crate) enum Decision {
    Adopted(Language),
    Kept(KeptWhy),
}

impl Plan {
    /// Whether any suffix is adopted; a plan with none is a no-op (§FS-config-adopt-builtins.9).
    pub(crate) fn adopts(&self) -> bool {
        self.entries
            .iter()
            .any(|entry| matches!(entry.decision, Decision::Adopted(_)))
    }

    /// The `adopts:` lines' records, in entry order (§FS-config-adopt-builtins.5.2).
    pub(crate) fn adopted(&self, model: &SourceModel) -> Vec<AdoptedSuffix> {
        self.entries
            .iter()
            .filter_map(|entry| {
                let Decision::Adopted(row) = &entry.decision else {
                    return None;
                };
                let now = model_suffix_reading(model, &entry.suffix)?;
                let then = self.expected.get(&entry.suffix)?;
                Some(AdoptedSuffix {
                    suffix: entry.suffix.clone(),
                    definition: entry.definition.name.clone(),
                    row: row.name.clone(),
                    opt_in: opt_in_row(&row.name),
                    differences: differences(&now, then, &entry.definition, row),
                })
            })
            .collect()
    }

    /// The `kept:` lines' records, in entry order (§FS-config-adopt-builtins.5.2).
    pub(crate) fn kept(&self) -> Vec<KeptSuffix> {
        self.entries
            .iter()
            .filter_map(|entry| match &entry.decision {
                Decision::Kept(why) => Some(KeptSuffix {
                    suffix: entry.suffix.clone(),
                    definition: entry.definition.name.clone(),
                    why: why.clone(),
                }),
                Decision::Adopted(_) => None,
            })
            .collect()
    }
}

/// §FS-config-adopt-builtins.3: the plan for `model`, from it and the epoch alone.
pub(crate) fn plan(model: &SourceModel) -> Plan {
    let mut entries = Vec::new();
    for (suffix, name) in &model.extensions {
        // §FS-config-adopt-builtins.2: in play only through an admitted definition.
        let admitted = model
            .admitted
            .iter()
            .any(|language| !language.builtin && language.name == *name);
        let Some(definition) = model.definitions.iter().find(|def| def.name == *name) else {
            continue;
        };
        if !admitted {
            continue;
        }
        let decision = match claiming(suffix) {
            None => Decision::Kept(KeptWhy::NoRow),
            Some(row) if row.class != definition.class => Decision::Kept(KeptWhy::Class {
                row: row.name.clone(),
                class: row.class.name().to_string(),
            }),
            Some(row) => Decision::Adopted(row),
        };
        entries.push(Entry {
            suffix: suffix.clone(),
            definition: definition.clone(),
            decision,
        });
    }
    // §FS-config-adopt-builtins.3.2, §FS-config-adopt-builtins.3.4: what the file reads now.
    let mut expected: BTreeMap<String, SuffixReading> = read_suffixes(model)
        .into_iter()
        .filter_map(|suffix| {
            let reading = model_suffix_reading(model, &suffix)?;
            Some((suffix, reading))
        })
        .collect();
    let mut added: Vec<AddedSuffixes> = Vec::new();
    for entry in &entries {
        let Decision::Adopted(row) = &entry.decision else {
            continue;
        };
        // §FS-config-adopt-builtins.3.1: the row's reading of the suffix.
        let reading = SuffixReading::of_language(row, &entry.suffix, model.declarations);
        expected.insert(entry.suffix.clone(), reading);
        if added.iter().any(|known| known.row == row.name) {
            continue;
        }
        // §FS-config-adopt-builtins.3.3: unmapped, and not read now.
        let suffixes: Vec<String> = row
            .suffixes
            .iter()
            .filter(|suffix| {
                !model.extensions.iter().any(|(key, _)| key == *suffix)
                    && model_suffix_reading(model, suffix).is_none()
            })
            .cloned()
            .collect();
        for suffix in &suffixes {
            let reading = SuffixReading::of_language(row, suffix, model.declarations);
            expected.insert(suffix.clone(), reading);
        }
        added.push(AddedSuffixes {
            row: row.name.clone(),
            suffixes,
        });
    }
    added.retain(|row| !row.suffixes.is_empty());
    Plan {
        entries,
        added,
        expected,
    }
}

/// §FS-config-adopt-builtins.6.2: every suffix `model` reads, text and data alike: each entry
/// whose language is admitted, and each admitted row's suffixes no entry remaps.
pub(crate) fn read_suffixes(model: &SourceModel) -> Vec<String> {
    let mut suffixes: Vec<String> = model
        .extensions
        .iter()
        .map(|(suffix, _)| suffix.clone())
        .collect();
    for language in model.admitted.iter().filter(|language| language.builtin) {
        suffixes.extend(language.suffixes.iter().cloned());
    }
    suffixes.sort();
    suffixes.dedup();
    suffixes.retain(|suffix| model_suffix_reading(model, suffix).is_some());
    suffixes
}

/// §FS-config-adopt-builtins.5.3: what reads differently once `definition`'s reading `now`
/// becomes `row`'s reading `then`, one line's text per part, in the spec's order.
fn differences(
    now: &SuffixReading,
    then: &SuffixReading,
    definition: &Language,
    row: &Language,
) -> Vec<String> {
    let mut lines = Vec::new();
    let lost: Vec<&str> = hosts(now.hosts.difference(&then.hosts));
    let gained: Vec<&str> = hosts(then.hosts.difference(&now.hosts));
    match lost.as_slice() {
        [] => {}
        [one] => lines.push(format!("`{one}` is no longer a comment host")),
        many => lines.push(format!("{} are no longer comment hosts", ticked(many))),
    }
    if !gained.is_empty() {
        lines.push(format!("comments behind {} are now read", ticked(&gained)));
    }
    if now.docstrings != then.docstrings {
        let state = if then.docstrings {
            "are now read"
        } else {
            "are no longer read"
        };
        lines.push(format!("triple-quoted docstrings {state}"));
    }
    if now.strings != then.strings {
        lines.push(format!(
            "multi-line strings and quotes are now read as built-in {} reads them",
            row.name
        ));
    }
    if now.measurement != then.measurement || now.any_comment != then.any_comment {
        lines.push(format!(
            "doc comments are now recognized by the `{}` profile, not `{}`",
            row.doc_comments.name(),
            definition.doc_comments.name()
        ));
    }
    lines
}

/// §FS-config-adopt-builtins.5.3: hosts in the order `//`, `/*`, `*`, `#`, `;`, `--`, then any
/// other in byte order.
fn hosts<'a>(set: impl Iterator<Item = &'a String>) -> Vec<&'a str> {
    const ORDER: [&str; 6] = ["//", "/*", "*", "#", ";", "--"];
    let mut hosts: Vec<&str> = set.map(String::as_str).collect();
    hosts.sort_by_key(|host| {
        let rank = ORDER
            .iter()
            .position(|known| known == host)
            .unwrap_or(ORDER.len());
        (rank, *host)
    });
    hosts
}

/// Each host in backticks, joined `A, B and C`.
fn ticked(hosts: &[&str]) -> String {
    let ticked: Vec<String> = hosts.iter().map(|host| format!("`{host}`")).collect();
    match ticked.as_slice() {
        [] => String::new(),
        [one] => one.clone(),
        [init @ .., last] => format!("{} and {last}", init.join(", ")),
    }
}
