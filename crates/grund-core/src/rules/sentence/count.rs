//! Canonical cardinality spellings for controlled-English rules (§FS-rules.3).

use super::Cardinality;
use super::forms::{Form, Refusal, refuse};

impl Cardinality {
    pub(crate) const AT_LEAST_ONE: Self = Self {
        minimum: Some(1),
        maximum: None,
    };
    pub(crate) const NONE: Self = Self {
        minimum: None,
        maximum: Some(0),
    };
    pub(crate) fn contains(self, count: usize) -> bool {
        self.minimum.is_none_or(|n| count >= n) && self.maximum.is_none_or(|n| count <= n)
    }
    pub(crate) fn wording(self) -> String {
        match (self.minimum, self.maximum) {
            (Some(1), None) => "at least one".into(),
            (Some(n), None) => format!("at least {n}"),
            (None, Some(n)) => format!("at most {n}"),
            (Some(1), Some(1)) => "exactly one".into(),
            (Some(n), Some(m)) if n == m => format!("exactly {n}"),
            _ => "the configured count".into(),
        }
    }
    pub(crate) fn times_wording(self) -> String {
        if self.minimum == Some(1) && self.maximum == Some(1) {
            "exactly once".into()
        } else {
            self.wording()
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum CountSpelling {
    AtLeastOne,
    ExactlyOne,
    /// §FS-rules.3.1: a floor spelled as a numeral, which is never one.
    AtLeast,
    AtMost(usize),
    /// An exact count spelled as a numeral, which is never one either.
    Exactly,
}

/// Read the count `text` opens with. A refusal's form is `text` with the count
/// written canonically, or none where it does not say which count it meant
/// (§FS-rule-refusals.sentences.rewrite.by-refusal).
pub(super) fn count_prefix(text: &str) -> Result<(Cardinality, &str, CountSpelling), Refusal> {
    if let Some(rest) = text.strip_prefix("at least one ") {
        return Ok((Cardinality::AT_LEAST_ONE, rest, CountSpelling::AtLeastOne));
    }
    if let Some(rest) = text.strip_prefix("exactly one ") {
        return Ok((
            Cardinality {
                minimum: Some(1),
                maximum: Some(1),
            },
            rest,
            CountSpelling::ExactlyOne,
        ));
    }
    for prefix in ["at least ", "at most ", "exactly "] {
        if let Some(rest) = text.strip_prefix(prefix) {
            let Some((raw, object)) = rest.split_once(' ') else {
                return Err(refuse("count has no object", Form::None { kind: false }));
            };
            let n = positive(raw)
                .map_err(|refusal| refusal.within(|n| format!("{prefix}{n} {object}")))?;
            // §FS-rules.3: outside per-target counts, a floor and an exact count
            // spell one as `one`; only a ceiling spells it as the numeral.
            let (card, spelling) = match prefix {
                "at least " if n == 1 => {
                    return Err(refuse(
                        "numeric \"at least 1\" is not canonical",
                        Form::One(format!("at least one {object}")),
                    ));
                }
                "exactly " if n == 1 => {
                    return Err(refuse(
                        "numeric \"exactly 1\" is not canonical",
                        Form::One(format!("exactly one {object}")),
                    ));
                }
                "at least " => (
                    Cardinality {
                        minimum: Some(n),
                        maximum: None,
                    },
                    CountSpelling::AtLeast,
                ),
                "at most " => (
                    Cardinality {
                        minimum: None,
                        maximum: Some(n),
                    },
                    CountSpelling::AtMost(n),
                ),
                _ => (
                    Cardinality {
                        minimum: Some(n),
                        maximum: Some(n),
                    },
                    CountSpelling::Exactly,
                ),
            };
            return Ok((card, object, spelling));
        }
    }
    // §FS-rule-refusals.sentences.rewrite.by-refusal: no count to keep, so the canonical ones are
    // listed alone.
    Err(refuse(
        "count is not accepted; the canonical counts are \"at least one\", \"at least N\", \"at most N\", \"exactly one\" and \"exactly N\" for a base-10 N",
        Form::None { kind: false },
    ))
}

/// Per-target coverage (§FS-rules.3.3): every bound spells one as `once` and
/// larger counts as `N times`. Return `None` where there is no per-target bound,
/// leaving the predicate to read an outbound count instead. Refusal forms keep
/// the typed kind and bound (§FS-rule-refusals.sentences.rewrite.by-refusal).
pub(super) fn per_target(rest: &str) -> Option<Result<(&str, Cardinality), Refusal>> {
    let (kind, bound, raw) = [" at least ", " at most ", " exactly "]
        .into_iter()
        .find_map(|bound| {
            let (kind, raw) = rest.split_once(bound)?;
            Some((kind, bound, raw))
        })?;
    Some((|| {
        let n = if raw == "once" {
            1
        } else {
            let Some(number) = raw.strip_suffix(" times") else {
                // §FS-rule-refusals.sentences.rewrite.by-refusal: reparsing repairs
                // the noun, leading zeros and numeric one in successive passes.
                let count = raw.split(' ').next().unwrap_or_default();
                let form = if numeral(count) {
                    Form::One(format!("{kind}{bound}{count} times"))
                } else {
                    Form::None { kind: false }
                };
                return Err(refuse("per-target counts must end in \"times\"", form));
            };
            let n = positive(number)
                .map_err(|refusal| refusal.within(|n| format!("{kind}{bound}{n} times")))?;
            if n == 1 {
                return Err(refuse(
                    format!("numeric \"{}1 times\" is not canonical", bound.trim_start()),
                    Form::One(format!("{kind}{bound}once")),
                ));
            }
            n
        };
        let card = match bound {
            " at least " => Cardinality {
                minimum: Some(n),
                maximum: None,
            },
            " at most " => Cardinality {
                minimum: None,
                maximum: Some(n),
            },
            _ => Cardinality {
                minimum: Some(n),
                maximum: Some(n),
            },
        };
        Ok((kind, card))
    })())
}

/// Whether `raw` is written in base-10 digits alone.
pub(super) fn numeral(raw: &str) -> bool {
    !raw.is_empty() && raw.bytes().all(|byte| byte.is_ascii_digit())
}

/// A positive count. A refusal's form is the count without its leading zeros,
/// or none where it is zero or no numeral (§FS-rule-refusals.sentences.rewrite.by-refusal),
/// and a numeral past `usize::MAX` is refused for its range (§FS-rule-refusals.sentences.count-range).
pub(super) fn positive(raw: &str) -> Result<usize, Refusal> {
    const REASON: &str = "count must be a canonical positive base-10 integer";
    let digits = raw.trim_start_matches('0');
    if !numeral(raw) || digits.is_empty() {
        return Err(refuse(REASON, Form::None { kind: false }));
    }
    // §FS-rule-refusals.sentences.count-range.leading-zeros: the value is read
    // after the zeros, so an overflow is refused for its range either way.
    let Ok(n) = digits.parse::<usize>() else {
        // §FS-rule-refusals.sentences.count-range.no-form: nothing in range to offer.
        return Err(refuse(
            format!(
                "{REASON}; the count is above the largest supported count, {}",
                usize::MAX
            ),
            Form::None { kind: false },
        ));
    };
    if digits != raw {
        return Err(refuse(REASON, Form::One(digits.into())));
    }
    Ok(n)
}
