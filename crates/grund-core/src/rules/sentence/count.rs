//! Canonical cardinality spellings for controlled-English rules (§FS-rules.3).

use super::{Cardinality, RuleParseError, error};

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
    AtMost(usize),
    Exactly(usize),
}

pub(super) fn count_prefix(
    text: &str,
) -> Result<(Cardinality, &str, CountSpelling), RuleParseError> {
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
    for prefix in ["at most ", "exactly "] {
        if let Some(rest) = text.strip_prefix(prefix) {
            let (raw, object) = rest
                .split_once(' ')
                .ok_or_else(|| error("count has no object"))?;
            let n = positive(raw)?;
            if prefix == "exactly " && n == 1 {
                return Err(error(
                    "numeric \"exactly 1\" is not canonical; accepted form: Each FS must cite exactly one GOAL.",
                ));
            }
            let card = if prefix == "at most " {
                Cardinality {
                    minimum: None,
                    maximum: Some(n),
                }
            } else {
                Cardinality {
                    minimum: Some(n),
                    maximum: Some(n),
                }
            };
            let spelling = if prefix == "at most " {
                CountSpelling::AtMost(n)
            } else {
                CountSpelling::Exactly(n)
            };
            return Ok((card, object, spelling));
        }
    }
    Err(error(
        "count is not accepted; accepted form: Each FS must cite at least one GOAL.",
    ))
}

pub(super) fn positive(raw: &str) -> Result<usize, RuleParseError> {
    let canonical = raw.bytes().all(|byte| byte.is_ascii_digit()) && !raw.starts_with('0');
    canonical
        .then(|| raw.parse::<usize>().ok())
        .flatten()
        .filter(|n| *n > 0)
        .ok_or_else(|| {
            error(
                "count must be a canonical positive base-10 integer; accepted form: Each FS must cite exactly 2 GOAL.",
            )
        })
}
