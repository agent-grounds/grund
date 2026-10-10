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
/// (§FS-rules.3.5.4.5).
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
            // §FS-rules.3: a floor and an exact count spell one as the word
            // `one`; only a ceiling spells it as the numeral.
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
    // §FS-rules.3.5.4.5: no count to keep, so the canonical ones are listed alone.
    Err(refuse(
        "count is not accepted; the canonical counts are \"at least one\", \"at least N\", \"at most N\", \"exactly one\" and \"exactly N\" for a base-10 N",
        Form::None { kind: false },
    ))
}

/// Whether `raw` is written in base-10 digits alone.
pub(super) fn numeral(raw: &str) -> bool {
    !raw.is_empty() && raw.bytes().all(|byte| byte.is_ascii_digit())
}

/// A positive count. A refusal's form is the count without its leading zeros,
/// or none where it is zero or no numeral (§FS-rules.3.5.4.5).
pub(super) fn positive(raw: &str) -> Result<usize, Refusal> {
    let canonical = raw.bytes().all(|byte| byte.is_ascii_digit()) && !raw.starts_with('0');
    canonical
        .then(|| raw.parse::<usize>().ok())
        .flatten()
        .filter(|n| *n > 0)
        .ok_or_else(|| {
            let digits = raw.trim_start_matches('0');
            let form = if numeral(raw) && !digits.is_empty() && digits != raw {
                Form::One(digits.into())
            } else {
                Form::None { kind: false }
            };
            refuse("count must be a canonical positive base-10 integer", form)
        })
}
