//! Sentence-specific configuration guidance, separate from shared kind errors
//! (§FS-rule-refusals.sentences.rewrite.no-form.place).

use std::collections::BTreeMap;

use super::RuleVocabulary;
use crate::config::{
    CitationPlace, KindEntry, Rules, Run, Schema, citation_places, escape_toml_basic,
    grounding_setting, has_dot_path_component, row_grounds_at_any_strength,
};

/// Non-citable subject kinds and their optional place routes. Selectors keep
/// only the kind entries; rules add their project's routes once
/// (§FS-rule-refusals.sentences.rewrite.no-form.place).
#[derive(Clone, Default)]
pub(crate) struct SubjectKinds {
    pub(super) entries: BTreeMap<String, KindEntry>,
    places: Vec<CitationPlace>,
    /// Each place whose row grounds nothing, with the setting that makes its
    /// uncited files findings (§FS-init.2.3.5.2.2).
    ungrounded: BTreeMap<String, String>,
}

impl From<BTreeMap<String, KindEntry>> for SubjectKinds {
    /// Selectors share the kind reasons without a sentence obligation
    /// (§FS-rule-refusals.sentences.rewrite.no-form.place).
    fn from(entries: BTreeMap<String, KindEntry>) -> Self {
        Self {
            entries,
            places: Vec::new(),
            ungrounded: BTreeMap::new(),
        }
    }
}

impl SubjectKinds {
    /// Attach only this rule project's configured routes, never workspace
    /// object routes, and the grounding each route lacks
    /// (§FS-rule-refusals.sentences.rewrite.no-form.place).
    pub(crate) fn for_rules(
        entries: BTreeMap<String, KindEntry>,
        schema: &Schema,
        rules: &Rules,
        run: &Run,
        version: u32,
    ) -> Self {
        let places = citation_places(schema, version);
        let ungrounded = places
            .iter()
            .filter(|place| !row_grounds_at_any_strength(rules, run, &place.name))
            .map(|place| {
                let setting = grounding_setting(&place.name, version, str::to_string);
                (place.name.clone(), setting)
            })
            .collect();
        Self {
            entries,
            places,
            ungrounded,
        }
    }

    /// Recognize only a complete canonical place obligation. Exact lexical
    /// matching deliberately avoids path normalization and predicate recovery
    /// (§FS-rule-refusals.sentences.rewrite.no-form.place).
    pub(super) fn guidance(&self, title: &str, vocabulary: &RuleVocabulary) -> Option<String> {
        let sentence = title.strip_suffix('.')?;
        let (subject, target) = sentence.split_once(" must cite at least one ")?;
        if target.contains(char::is_whitespace)
            || target.contains('/')
            || !vocabulary.kinds.contains(target)
        {
            return None;
        }
        let mut matches = self.places.iter().filter(|place| {
            if let Some(path) = subject.strip_prefix("Each file in ") {
                if has_dot_path_component(path)
                    || path.starts_with('/')
                    || path.contains(['*', '?', '[', ']'])
                {
                    return false;
                }
                place.files.iter().any(|file| file == path)
                    || place.folders.iter().any(|folder| {
                        path == folder || path.strip_suffix('/') == Some(folder.as_str())
                    })
            } else {
                subject.strip_prefix("Each ") == Some(place.name.as_str())
            }
        });
        let place = matches.next()?;
        if matches.next().is_some() {
            return None;
        }
        let target = escape_toml_basic(target);
        // §FS-init.2.3.5.2.2: the entry alone never judges a file that cites nothing.
        let grounding = self
            .ungrounded
            .get(&place.name)
            .map_or_else(String::new, |setting| {
                format!(
                    ", and {setting}, without which a file there that cites nothing is not checked"
                )
            });
        Some(format!(
            "; for configured place \"{}\", express this obligation in grund.toml: [{}] with must = [\"{target}\"]{grounding}",
            place.name, place.table
        ))
    }
}
