//! The whose clause a quantified subject may end in (§FS-rules.2.2), and the
//! reasons a clause the grammar does not accept is refused for
//! (§FS-rule-refusals.sentences.whose).
//!
//! The clause is read from the subject the modality split left (§FS-rules.3.6),
//! so a rule and a selector read it alike: `parse_subject` is the one caller.

use super::subjects::{SubjectRefusal, parse_subject};
use super::{RuleSubject, RuleVocabulary, Whose};
use crate::config::KindField;

/// What opens a whose clause after a quantified subject's kind (§FS-rules.2.2).
pub(super) const WHOSE: &str = " whose ";

/// The suffix every reason the window of §FS-errors.3.10 moves ends in.
const WINDOW_SUFFIX: &str = "; this wording changes in grund 0.22.0";

/// §FS-errors.3.10: a refusal of a subject spelled with a whose clause keeps the
/// reason it printed before clauses existed, `unknown kind "<legacy>"`, as a
/// verbatim prefix, then ` — `, the reason that replaces it, and the suffix.
pub(super) fn windowed(legacy: &str, message: &str) -> String {
    format!("unknown kind \"{legacy}\" — {message}{WINDOW_SUFFIX}")
}

/// Why a whose clause is refused, in the order §FS-rule-refusals.sentences.whose
/// checks it: the first that fails is the one kept.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum WhoseFault {
    /// A version-1 config, which declares no scalar field.
    VersionOne,
    /// The clause, without `whose `, has no ` is `.
    NoIs(String),
    NoField(String),
    /// The clause names no value; `one_of` is the field's, where it is a scalar,
    /// for the modal-word append.
    NoValue {
        clause: String,
        field: Field,
    },
    UnknownField {
        kind: String,
        field: String,
        scalars: Vec<String>,
    },
    NotScalar {
        kind: String,
        field: String,
        scalars: Vec<String>,
    },
    Whitespace {
        kind: String,
        field: String,
        value: String,
    },
    NotValue {
        kind: String,
        field: String,
        value: String,
        one_of: Vec<String>,
    },
}

/// A scalar field a malformed clause still named, kept for the modal-word append.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Field {
    pub(super) one_of: Option<Vec<String>>,
}

impl WhoseFault {
    /// The final reason (§FS-rule-refusals.sentences.whose), before any modal-word
    /// append and before the window of §FS-errors.3.10.
    pub(super) fn reason(&self) -> String {
        let shape = |clause: &str, what: &str| {
            format!("whose clause \"whose {clause}\" {what}; write whose FIELD is VALUE")
        };
        match self {
            Self::VersionOne => "whose clauses require grund_config_version = 2".into(),
            Self::NoIs(clause) => shape(clause, "has no \" is \""),
            Self::NoField(clause) => shape(clause, "names no field"),
            Self::NoValue { clause, .. } => shape(clause, "names no value"),
            Self::UnknownField {
                kind,
                field,
                scalars,
            } => format!(
                "\"{field}\" is not a field of kind {kind}; {}",
                scalar_list(kind, scalars)
            ),
            Self::NotScalar {
                kind,
                field,
                scalars,
            } => format!(
                "field \"{field}\" of kind {kind} is not a scalar: it sets no one_of; {}",
                scalar_list(kind, scalars)
            ),
            Self::Whitespace { kind, field, value } => format!(
                "value \"{value}\" of {kind}.{field} has surrounding whitespace, which no one_of \
                 value can hold"
            ),
            Self::NotValue {
                kind,
                field,
                value,
                one_of,
            } => format!(
                "\"{value}\" is not a value of {kind}.{field}; kind {kind} admits {}",
                one_of.join(", ")
            ),
        }
    }

    /// The value the modality split may have cut, and the field's values to
    /// look for the cut one in (§FS-rule-refusals.sentences.whose).
    pub(super) fn cut_value(&self) -> Option<(&str, &[String])> {
        match self {
            Self::NoValue {
                field: Field {
                    one_of: Some(one_of),
                },
                ..
            } => Some(("", one_of)),
            Self::NotValue { value, one_of, .. } => Some((value, one_of)),
            _ => None,
        }
    }

    /// One subject per value of a value joined by ` or `, the subject's text up
    /// to the value being `head`: the only row offered a form
    /// (§FS-rule-refusals.sentences.whose).
    pub(super) fn or_forms(&self, subject: &str) -> Option<Vec<String>> {
        let Self::NotValue { value, .. } = self else {
            return None;
        };
        let head = subject.strip_suffix(value.as_str())?;
        value.contains(" or ").then(|| {
            value
                .split(" or ")
                .map(|part| format!("{head}{part}"))
                .collect()
        })
    }
}

/// `its scalar fields: …`, or that the kind declares none.
fn scalar_list(kind: &str, scalars: &[String]) -> String {
    if scalars.is_empty() {
        format!("kind {kind} declares no scalar field")
    } else {
        format!("its scalar fields: {}", scalars.join(", "))
    }
}

/// The ` — value "<entry>" holds the modal word …` append: where a `one_of`
/// entry starts at the cut value's first byte and runs past the modality the
/// split took (§FS-rule-refusals.sentences.whose). `modality` is the split's
/// own text, spaces included; after an empty value the space before it ended
/// the clause's ` is`.
pub(super) fn modal_value_append(value: &str, one_of: &[String], modality: &str) -> Option<String> {
    let tail = if value.is_empty() {
        modality.trim_start()
    } else {
        modality
    };
    let cut = format!("{value}{tail}");
    let entry = one_of.iter().find(|entry| entry.starts_with(&cut))?;
    Some(format!(
        " — value \"{entry}\" holds the modal word \"{}\", which ends the subject, so a rule \
         cannot name it",
        modality.trim()
    ))
}

/// Read the clause after ` whose ` of a subject whose kind `kind` is configured
/// (§FS-rules.2.2), checking in the order §FS-rule-refusals.sentences.whose gives.
pub(super) fn parse_whose(
    kind: &str,
    clause: &str,
    vocabulary: &RuleVocabulary,
) -> Result<Whose, WhoseFault> {
    let fields = match vocabulary.fields.get(kind) {
        Some(None) => return Err(WhoseFault::VersionOne),
        Some(Some(fields)) => fields.as_slice(),
        None => &[],
    };
    // §FS-rules.2.2: the first ` is `, or an ` is` that ends the subject, divides it.
    let (field, value) = match clause.split_once(" is ") {
        Some(split) => split,
        None => match clause.strip_suffix(" is") {
            Some(field) => (field, ""),
            None => return Err(WhoseFault::NoIs(clause.into())),
        },
    };
    let declared = fields.iter().find(|declared| declared.name == field);
    if field.is_empty() {
        return Err(WhoseFault::NoField(clause.into()));
    }
    if value.is_empty() {
        let one_of = declared.and_then(|declared| declared.one_of.clone());
        return Err(WhoseFault::NoValue {
            clause: clause.into(),
            field: Field { one_of },
        });
    }
    let scalars = || {
        fields
            .iter()
            .filter(|field| field.one_of.is_some())
            .map(|field| field.name.clone())
            .collect()
    };
    let one_of = match declared {
        None => {
            return Err(WhoseFault::UnknownField {
                kind: kind.into(),
                field: field.into(),
                scalars: scalars(),
            });
        }
        Some(KindField { one_of: None, .. }) => {
            return Err(WhoseFault::NotScalar {
                kind: kind.into(),
                field: field.into(),
                scalars: scalars(),
            });
        }
        Some(KindField {
            one_of: Some(one_of),
            ..
        }) => one_of,
    };
    // §FS-fields.identity.match: no one_of entry holds surrounding whitespace.
    if value.trim() != value {
        return Err(WhoseFault::Whitespace {
            kind: kind.into(),
            field: field.into(),
            value: value.into(),
        });
    }
    if !one_of.iter().any(|entry| entry == value) {
        return Err(WhoseFault::NotValue {
            kind: kind.into(),
            field: field.into(),
            value: value.into(),
            one_of: one_of.clone(),
        });
    }
    Ok(Whose {
        field: field.into(),
        value: value.into(),
    })
}

/// A quantified subject's kind and the whose clause after it, if any
/// (§FS-rules.2.2): the clause opens at the first ` whose `, unless the whole
/// text is a configured kind.
pub(super) fn split_whose<'a>(text: &'a str, vocab: &RuleVocabulary) -> (&'a str, Option<&'a str>) {
    if vocab.kinds.contains(text) {
        return (text, None);
    }
    match text.split_once(WHOSE) {
        Some((kind, clause)) => (kind, Some(clause)),
        None => (text, None),
    }
}

/// A refusal of a quantified subject that carried a whose clause keeps what
/// followed the quantifier as the legacy unknown kind (§FS-errors.3.10).
pub(super) fn with_clause(
    mut refusal: SubjectRefusal,
    after_each: &str,
    clause: Option<&str>,
) -> SubjectRefusal {
    if clause.is_some() {
        refusal.legacy = Some(after_each.into());
    }
    refusal
}

/// What followed `Each ` or `of each ` in a quantified subject: the text a
/// sentence whose subject carried a whose clause was refused for as an unknown
/// kind before clauses existed, which every refusal of it is windowed after
/// (§FS-errors.3.10).
pub(super) fn legacy_kind(subject: &str) -> Option<&str> {
    subject.strip_prefix("Each ").or_else(|| {
        let rest = subject.strip_prefix("The ")?;
        rest.split_once(" chapter of each ")
            .map(|(_, after_each)| after_each)
    })
}

/// Whether `text` is an exact ID followed by a whose clause
/// (§FS-rule-refusals.sentences.whose).
pub(super) fn exact_with_clause(text: &str, vocab: &RuleVocabulary) -> bool {
    text.split_once(WHOSE).is_some_and(|(head, _)| {
        matches!(
            parse_subject(head, vocab),
            Ok(RuleSubject::ExactDeclaration(_) | RuleSubject::ExactChapter { .. })
        )
    })
}

#[cfg(test)]
#[path = "tests_whose.rs"]
mod tests_whose;
