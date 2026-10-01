use once_cell::sync::Lazy;
use regex::Regex;
use std::path::PathBuf;

use super::records::Id;

/// Exact component classification and equality for first-class values
/// (§FS-values.2, §FS-values.4). Decimals stay as normalized coefficient and
/// arbitrary-size decimal exponent strings; no binary float or exponent
/// expansion is involved.

#[derive(Debug, Clone)]
pub enum DeclarationSource {
    Text,
    Json {
        member_slice: String,
        key_column: usize,
        key_text: String,
    },
}

/// One authoritative component shared by Markdown and JSON value declarations
/// (§FS-values.2, §FS-values.4).
#[derive(Debug, Clone)]
pub struct ValueComponent {
    pub decoded: String,
    pub kind: ValueComponentKind,
    pub source_slice: String,
    pub column: usize,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ValueComponentKind {
    Number,
    String,
}

/// Which of the two enrollment routes made a section a value root
/// (§AR-scanner.2.2.8). `Marker` carries the authored, one-based byte column of
/// the exact suffix (§FS-values.2.4); `Chapter` carries nothing, because a
/// chapter-declared root has no marker bytes to point at (§FS-values.2.5).
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ValueRootOrigin {
    Marker { column: usize },
    Chapter,
}

/// Authority metadata attached to an ordinary section record
/// (§FS-values.2.4, §FS-values.2.5). Semantic title consumers omit a marker
/// while raw readers keep the source; downstream validation, resolution and
/// comparison read this record rather than the route that wrote it
/// (§AR-scanner.2.2.8).
#[derive(Debug, Clone)]
pub struct EmbeddedValueRoot {
    pub valid: bool,
    pub origin: ValueRootOrigin,
}

impl EmbeddedValueRoot {
    /// The authored marker column, or `None` for a chapter root — which is also
    /// the location a root-level failure has to fall back to the root heading
    /// for (§FS-values.2.4.3).
    pub fn marker_column(&self) -> Option<usize> {
        match self.origin {
            ValueRootOrigin::Marker { column } => Some(column),
            ValueRootOrigin::Chapter => None,
        }
    }
}

/// Whether a binding's cited section path is a value-binding shape at all: a
/// valid root path — every component numeric, or a named prefix as
/// §FS-config.3.3.1 permits — optionally followed by one positive numeric
/// immediate coordinate (§FS-values.3.1). A path with no coordinate is the root
/// itself (§FS-values.3.1.2), and a bare ID has no path to check. Which of the
/// two readings a path takes, and whether its root *is* a root, is the checker's
/// question (§FS-values.5.1); this is only the grammar.
pub(crate) fn value_binding_section_shape_is_valid(section: &str) -> bool {
    // §FS-config.3.3.1: a numeric component may follow a named prefix, never
    // the other way round, so `2.values` stays reserved.
    let mut seen_numeric = false;
    section.split('.').all(|part| {
        if positive_numeric_component(part) {
            seen_numeric = true;
            true
        } else {
            !seen_numeric && named_section_component(part)
        }
    })
}

/// Whether a binding path's final segment can name an immediate component: a
/// path that ends in a name has no coordinate, so it can only be a chapter root
/// aimed at whole (§FS-values.3.1, §FS-values.5.1).
pub(crate) fn value_binding_section_ends_in_coordinate(section: &str) -> bool {
    section
        .rsplit('.')
        .next()
        .is_some_and(positive_numeric_component)
}

fn positive_numeric_component(part: &str) -> bool {
    !part.is_empty() && !part.starts_with('0') && part.bytes().all(|byte| byte.is_ascii_digit())
}

/// One `[a-z][a-z0-9-]*` section handle — the fixed named-component grammar
/// `value_chapter` is also held to (§FS-config.3.2.7, §FS-config.3.4.13).
pub(crate) fn named_section_component(part: &str) -> bool {
    part.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && part
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

/// An exact authored binding beside the ordinary citation the scanner emits
/// for the same token (§FS-values.3, §AR-scanner.3). `section` is the cited
/// path as written, `None` for a bare ID: whether it names a root or one of a
/// root's components is resolved against the catalog, not read off the
/// spelling (§FS-values.5.1).
#[derive(Debug)]
pub struct ValueBinding {
    pub namespace: Option<String>,
    pub id: Id,
    pub section: Option<String>,
    pub authored: ValueComponent,
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
}

/// A readable declaration or attempted binding that violates the explicit
/// value grammar (§FS-values.5.2, §AR-scanner.3).
#[derive(Debug)]
pub struct InvalidValueSite {
    pub id: Option<Id>,
    pub file: PathBuf,
    pub line: usize,
    pub column: Option<usize>,
    pub message: String,
    pub source: DeclarationSource,
    /// Binding-only target metadata. Declaration-shape errors leave these
    /// fields empty; the checker uses them to keep malformed delimited prose
    /// inert unless it actually aims at configured or marked value authority
    /// (§FS-values.3.1.1, §FS-values.5.1).
    pub binding_namespace: Option<String>,
    pub binding_section: Option<String>,
}

pub(crate) static JSON_NUMBER_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^-?(?:0|[1-9][0-9]*)(?:\.[0-9]+)?(?:[eE][+-]?[0-9]+)?$").unwrap());

pub(crate) fn component_text_is_valid(text: &str) -> bool {
    !text.is_empty()
        && text.trim() == text
        && !text.contains('`')
        && !text.chars().any(char::is_control)
}

pub(crate) fn authored_component(text: &str, column: usize) -> ValueComponent {
    ValueComponent {
        decoded: text.to_string(),
        kind: if JSON_NUMBER_RE.is_match(text) {
            ValueComponentKind::Number
        } else {
            ValueComponentKind::String
        },
        source_slice: text.to_string(),
        column,
    }
}

/// The parts a root-aimed literal splits into: at every U+0020 and nowhere else,
/// keeping the empty part two adjacent spaces leave, each classified as an
/// authored component of its own (§FS-values.3.1.2). Columns stay byte columns
/// into the authored line, as the literal's own is.
pub(crate) fn root_literal_parts(literal: &ValueComponent) -> Vec<ValueComponent> {
    let mut offset = 0;
    literal
        .decoded
        .split(' ')
        .map(|part| {
            let component = authored_component(part, literal.column + offset);
            offset += part.len() + 1;
            component
        })
        .collect()
}

/// A root's components `.1` through `.N` joined by one ASCII space, each as a
/// declared component prints (§FS-values.3.1.2, §FS-values.5.2.2).
pub(crate) fn joined_value_components(components: &[&ValueComponent]) -> String {
    components
        .iter()
        .map(|component| component.decoded.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

/// The index of the first part that is unequal to its component, comparing in
/// declared order under §FS-values.4 and stopping there, so only that component
/// is reported (§FS-values.5.2.2). The caller has already matched the counts.
pub(crate) fn first_unequal_component(
    parts: &[ValueComponent],
    components: &[&ValueComponent],
) -> Option<usize> {
    parts
        .iter()
        .zip(components)
        .position(|(part, component)| !value_components_equal(part, component))
}

pub(crate) fn value_components_equal(left: &ValueComponent, right: &ValueComponent) -> bool {
    match (left.kind, right.kind) {
        (ValueComponentKind::Number, ValueComponentKind::Number) => {
            ExactDecimal::parse(&left.decoded) == ExactDecimal::parse(&right.decoded)
        }
        (ValueComponentKind::String, ValueComponentKind::String) => left.decoded == right.decoded,
        _ => false,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ExactDecimal {
    negative: bool,
    coefficient: String,
    exponent: SignedDigits,
}

impl ExactDecimal {
    fn parse(raw: &str) -> Option<Self> {
        if !JSON_NUMBER_RE.is_match(raw) {
            return None;
        }
        let (negative, unsigned) = raw
            .strip_prefix('-')
            .map_or((false, raw), |rest| (true, rest));
        let (mantissa, exponent_raw) = unsigned
            .split_once(['e', 'E'])
            .map_or((unsigned, "0"), |(mantissa, exponent)| (mantissa, exponent));
        let (whole, fraction) = mantissa
            .split_once('.')
            .map_or((mantissa, ""), |(whole, fraction)| (whole, fraction));
        let mut coefficient = format!("{whole}{fraction}");
        let first_nonzero = coefficient
            .find(|ch| ch != '0')
            .unwrap_or(coefficient.len());
        coefficient.drain(..first_nonzero);
        if coefficient.is_empty() {
            return Some(Self {
                negative: false,
                coefficient: "0".to_string(),
                exponent: SignedDigits::zero(),
            });
        }
        let trailing = coefficient.len() - coefficient.trim_end_matches('0').len();
        coefficient.truncate(coefficient.len() - trailing);
        let exponent = SignedDigits::parse(exponent_raw)?
            .add_signed_usize(false, fraction.len())
            .add_signed_usize(true, trailing);
        Some(Self {
            negative,
            coefficient,
            exponent,
        })
    }
}

/// Sign plus normalized base-10 magnitude. Only addition/subtraction by an
/// input-length-sized `usize` is needed to account for a decimal point and
/// stripped coefficient zeroes, so exponent length remains unbounded.
#[derive(Clone, Debug, Eq, PartialEq)]
struct SignedDigits {
    negative: bool,
    digits: String,
}

impl SignedDigits {
    fn zero() -> Self {
        Self {
            negative: false,
            digits: "0".to_string(),
        }
    }

    fn parse(raw: &str) -> Option<Self> {
        let (negative, digits) = if let Some(rest) = raw.strip_prefix('-') {
            (true, rest)
        } else if let Some(rest) = raw.strip_prefix('+') {
            (false, rest)
        } else {
            (false, raw)
        };
        if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
            return None;
        }
        let digits = digits.trim_start_matches('0');
        if digits.is_empty() {
            return Some(Self::zero());
        }
        Some(Self {
            negative,
            digits: digits.to_string(),
        })
    }

    fn add_signed_usize(mut self, positive: bool, amount: usize) -> Self {
        if amount == 0 {
            return self;
        }
        let amount = amount.to_string();
        if self.digits == "0" {
            self.negative = !positive;
            self.digits = amount;
            return self;
        }
        if self.negative == !positive {
            self.digits = add_magnitudes(&self.digits, &amount);
            return self;
        }
        match compare_magnitudes(&self.digits, &amount) {
            std::cmp::Ordering::Greater => {
                self.digits = subtract_magnitudes(&self.digits, &amount);
            }
            std::cmp::Ordering::Less => {
                self.digits = subtract_magnitudes(&amount, &self.digits);
                self.negative = !self.negative;
            }
            std::cmp::Ordering::Equal => return Self::zero(),
        }
        self
    }
}

fn compare_magnitudes(left: &str, right: &str) -> std::cmp::Ordering {
    left.len().cmp(&right.len()).then_with(|| left.cmp(right))
}

fn add_magnitudes(left: &str, right: &str) -> String {
    let mut carry = 0u8;
    let mut out = Vec::new();
    let mut left = left.bytes().rev();
    let mut right = right.bytes().rev();
    loop {
        let a = left.next().map(|byte| byte - b'0');
        let b = right.next().map(|byte| byte - b'0');
        if a.is_none() && b.is_none() && carry == 0 {
            break;
        }
        let sum = a.unwrap_or(0) + b.unwrap_or(0) + carry;
        out.push(b'0' + sum % 10);
        carry = sum / 10;
    }
    out.reverse();
    String::from_utf8(out).expect("decimal digits are UTF-8")
}

/// Subtract `right` from `left`; the caller proves `left >= right`.
fn subtract_magnitudes(left: &str, right: &str) -> String {
    let mut borrow = 0i8;
    let mut out = Vec::new();
    let mut right = right.bytes().rev();
    for byte in left.bytes().rev() {
        let mut digit = (byte - b'0') as i8 - borrow;
        let other = right.next().map(|byte| (byte - b'0') as i8).unwrap_or(0);
        if digit < other {
            digit += 10;
            borrow = 1;
        } else {
            borrow = 0;
        }
        out.push(b'0' + (digit - other) as u8);
    }
    while out.len() > 1 && out.last() == Some(&b'0') {
        out.pop();
    }
    out.reverse();
    String::from_utf8(out).expect("decimal digits are UTF-8")
}
