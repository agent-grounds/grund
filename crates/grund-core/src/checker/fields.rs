//! §FS-fields.checks: every declaration of a kind held to the shape its fields
//! and `closed` state — a node-local pass of `conform` (§AR-checker.1.1), which
//! reads the shape through the kind's record (§FS-fields.record) — and the
//! pairing of a field with the chapter-presence rule it expresses, run where the
//! rule findings join the report (§FS-rules.6).

use crate::model::remedies;
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::config::{Field, FieldContent, Frame, Kind, Placement, Schema, Strength};
use crate::grammar::{render_declaration_id, section_display_name};
use crate::model::{Armed, Catalog, CheckReport, Code, Declaration, Diagnostic, Switch};

/// The channel a strength names (§FS-config-v2.rules.strengths); `None` is unchecked.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum Channel {
    Error,
    Warning,
    Suggestion,
}

pub(super) fn channel(strength: Strength) -> Option<Channel> {
    match strength {
        Strength::Must => Some(Channel::Error),
        Strength::Warn => Some(Channel::Warning),
        Strength::Should => Some(Channel::Suggestion),
        Strength::May => None,
    }
}

/// §FS-fields.strengths.may: a present field is held to its shape on its
/// presence's channel, and `may` waives absence only, so a present `may`
/// field's violation is an error.
fn shape_channel(field: &Field) -> Channel {
    channel(field.presence).unwrap_or(Channel::Error)
}

/// One finding about one declaration's shape, with the field it is about.
pub(super) struct FieldFinding<'a> {
    field: Option<&'a Field>,
    channel: Channel,
    diagnostic: Diagnostic,
}

/// §FS-fields.checks: the five field findings over every local declaration of
/// a kind that declares a field or is closed. Unsorted: the merge sorts.
pub(super) fn check_fields(
    catalog: &Catalog,
    schema: &Schema,
    frame: Frame<'_>,
    report: &mut CheckReport,
) {
    arm_fields(schema, &mut report.armed);
    for_each_finding(catalog, schema, frame, |_, finding| {
        let channel = match finding.channel {
            Channel::Error => &mut report.errors,
            Channel::Warning => &mut report.warnings,
            Channel::Suggestion => &mut report.suggestions,
        };
        channel.push(finding.diagnostic);
    });
}

/// §FS-check.1.4.1: the parts of a kind's shape that turn each field check on, as
/// the gate below reads them: a field a declaration can miss, its `one_of`, its
/// place, its title, and a kind that is `closed` at all.
fn arm_fields(schema: &Schema, armed: &mut Armed) {
    for (_, kind) in schema.kinds() {
        for field in &kind.fields {
            if field.presence != Strength::May {
                armed.arm(Switch::Field);
            }
            if matches!(field.content, FieldContent::OneOf(_)) {
                armed.arm(Switch::FieldOneOf);
            }
            if field.placement.is_some() {
                armed.arm(Switch::FieldPlacement);
            }
            if field.title.is_some() {
                armed.arm(Switch::FieldTitle);
            }
            // §FS-fields.front-matter.findings: its keys turn on the codes a field does.
            if let Some(shape) = field.front_matter() {
                armed.arm(Switch::FrontMatterField);
                if shape.keys.iter().any(|key| key.presence != Strength::May) {
                    armed.arm(Switch::Field);
                }
                if shape.keys.iter().any(|key| key.one_of.is_some()) {
                    armed.arm(Switch::FieldOneOf);
                }
                if shape.closed != Strength::May {
                    armed.arm(Switch::ClosedKind);
                }
            }
        }
        if kind.closed != Strength::May {
            armed.arm(Switch::ClosedKind);
        }
    }
}

/// Every field finding of every declaration, handed to `visit` with the
/// declaration it is about. A stub has no body to hold a field
/// (§FS-fields.identity.name).
fn for_each_finding<'a>(
    catalog: &'a Catalog,
    schema: &'a Schema,
    frame: Frame<'_>,
    mut visit: impl FnMut(&'a Declaration, FieldFinding<'a>),
) {
    for (id, declarations) in &catalog.declarations {
        let Some((_, kind)) = schema.kinds().find(|(name, _)| *name == id.kind) else {
            continue;
        };
        if kind.fields.is_empty() && kind.closed == Strength::May {
            continue;
        }
        let separator = schema.ids.section_separator.as_str();
        for declaration in declarations.iter().filter(|decl| !decl.is_stub) {
            let shape = Shape {
                kind: &id.kind,
                id: render_declaration_id(frame.grammar(), declaration),
                separator,
                declaration,
                direct: direct_sections(declaration),
            };
            for finding in shape.findings(kind) {
                visit(declaration, finding);
            }
        }
    }
}

/// §FS-fields.terms: a declaration's direct sections, `(name, line, display name)`
/// in the order they are written. The lead is not a section; a section written
/// twice counts once, at its first heading (§FS-fields.identity.name); the front
/// matter is no chapter (§FS-fields.front-matter.findings).
fn direct_sections(declaration: &Declaration) -> Vec<(&str, usize, Cow<'_, str>)> {
    let mut direct: Vec<(&str, usize, Cow<'_, str>)> = declaration
        .sections
        .iter()
        .filter(|(path, info)| !path.contains('.') && !info.in_front_matter())
        .map(|(path, info)| {
            let display = section_display_name(&info.title, path, info.relative);
            (path.as_str(), info.line, display)
        })
        .collect();
    direct.sort_by_key(|(_, line, _)| *line);
    direct
}

/// One declaration and the facts every field check reads of it.
pub(super) struct Shape<'a> {
    pub(super) kind: &'a str,
    pub(super) id: String,
    pub(super) separator: &'a str,
    pub(super) declaration: &'a Declaration,
    direct: Vec<(&'a str, usize, Cow<'a, str>)>,
}

impl<'a> Shape<'a> {
    fn findings(&self, kind: &'a Kind) -> Vec<FieldFinding<'a>> {
        let mut out = Vec::new();
        for field in &kind.fields {
            // §FS-fields.front-matter: the block, judged from what the scan read of it.
            if let Some(shape) = field.front_matter() {
                self.front_matter(field, shape, &mut out);
                continue;
            }
            match self.position(&field.name) {
                None => self.missing(field, &mut out),
                Some(at) => self.content(field, at, &mut out),
            }
        }
        self.order(kind, &mut out);
        self.closed(kind, &mut out);
        out
    }

    fn position(&self, name: &str) -> Option<usize> {
        self.direct
            .iter()
            .position(|(section, _, _)| *section == name)
    }

    pub(super) fn finding(
        &self,
        field: Option<&'a Field>,
        channel: Channel,
        code: Code,
        line: usize,
        message: String,
    ) -> FieldFinding<'a> {
        FieldFinding {
            field,
            channel,
            diagnostic: Diagnostic {
                code,
                path: Some(self.declaration.file.clone()),
                line: Some(line),
                column: None,
                message,
                sites: Vec::new(),
                authority: Vec::new(),
            },
        }
    }

    /// §FS-fields.checks.missing-field, at the declaration's title line.
    fn missing(&self, field: &'a Field, out: &mut Vec<FieldFinding<'a>>) {
        let Some(channel) = channel(field.presence) else {
            return;
        };
        let message = format!(
            "{} has no {} field; kind {} declares it; {}",
            self.id,
            field.name,
            self.kind,
            remedies::missing_field_chapter(&field.name)
        );
        let line = self.declaration.line;
        out.push(self.finding(Some(field), channel, Code::MISSING_FIELD, line, message));
    }

    /// §FS-fields.checks.mistitled-field, §FS-fields.checks.invalid-field-value:
    /// a present field's display name, compared byte for byte
    /// (§FS-fields.identity.match).
    fn content(&self, field: &'a Field, at: usize, out: &mut Vec<FieldFinding<'a>>) {
        let (_, line, ref display) = self.direct[at];
        let display = display.as_ref();
        let section = format!("{}{}{}", self.id, self.separator, field.name);
        let channel = shape_channel(field);
        if let Some(title) = &field.title
            && display != title
        {
            let message = format!(
                "{section} is titled \"{display}\"; kind {} titles it \"{title}\"; {}",
                self.kind,
                remedies::mistitled_field()
            );
            out.push(self.finding(Some(field), channel, Code::MISTITLED_FIELD, line, message));
        }
        if let FieldContent::OneOf(values) = &field.content
            && !values.iter().any(|value| value == display)
        {
            let message = format!(
                "{section} is \"{display}\"; kind {} admits {}; {}",
                self.kind,
                values.join(", "),
                remedies::invalid_field_value_heading()
            );
            out.push(self.finding(
                Some(field),
                channel,
                Code::INVALID_FIELD_VALUE,
                line,
                message,
            ));
        }
    }

    /// §FS-fields.checks.misplaced-field over each chain (§FS-fields.order): a
    /// present member is judged against its nearest present predecessor where it
    /// actually stands, or against its chain head's end when every predecessor is
    /// absent, so one field out of place is one finding (§FS-fields.order.nearest).
    fn order(&self, kind: &'a Kind, out: &mut Vec<FieldFinding<'a>>) {
        let count = self.direct.len();
        let successor = |name: &str| {
            kind.fields
                .iter()
                .find(|field| matches!(&field.placement, Some(Placement::After(p)) if p == name))
        };
        let heads = kind
            .fields
            .iter()
            .filter(|field| !matches!(field.placement, Some(Placement::After(_))));
        for head in heads {
            let mut previous: Option<(usize, &str)> = None;
            let mut member = Some(head);
            while let Some(field) = member {
                member = successor(&field.name);
                // §FS-fields.order.absent: an absent member is skipped.
                let Some(at) = self.position(&field.name) else {
                    continue;
                };
                let expected = match (previous, &head.placement) {
                    (Some((before, name)), _) => {
                        Some((before + 1, format!("directly after {name}")))
                    }
                    (None, Some(Placement::First)) => Some((0, "first".to_string())),
                    (None, Some(Placement::Last)) => Some((count - 1, "last".to_string())),
                    (None, _) => None,
                };
                if let Some((place, wording)) = expected
                    && place != at
                {
                    let message = format!(
                        "{}{}{} is section {} of {count}; kind {} places it {wording}; {}",
                        self.id,
                        self.separator,
                        field.name,
                        at + 1,
                        self.kind,
                        remedies::misplaced_field()
                    );
                    let line = self.direct[at].1;
                    out.push(self.finding(
                        Some(field),
                        shape_channel(field),
                        Code::MISPLACED_FIELD,
                        line,
                        message,
                    ));
                }
                previous = Some((at, &field.name));
            }
        }
    }

    /// §FS-fields.checks.undeclared-chapter: in a closed kind, a direct named
    /// section no field declares. Numbered sections are always admitted.
    fn closed(&self, kind: &'a Kind, out: &mut Vec<FieldFinding<'a>>) {
        let Some(channel) = channel(kind.closed) else {
            return;
        };
        let names: Vec<&str> = kind
            .fields
            .iter()
            .map(|field| field.name.as_str())
            .collect();
        for (section, line, _) in &self.direct {
            let numbered = section.bytes().all(|byte| byte.is_ascii_digit());
            if numbered || names.contains(section) {
                continue;
            }
            let message = format!(
                "{}{}{section} is not a field of kind {}, which is closed; its fields: {}; {}",
                self.id,
                self.separator,
                self.kind,
                if names.is_empty() {
                    "none".to_string()
                } else {
                    names.join(", ")
                },
                remedies::undeclared_chapter_chapter()
            );
            out.push(self.finding(None, channel, Code::UNDECLARED_CHAPTER, *line, message));
        }
    }
}

/// §FS-rules.6, §FS-fields.rules: drop a rule's `chapter-cardinality` where a
/// field that expresses it reports the same absence in the same declaration on
/// the same channel — the rule counted zero chapters of the field's name, and
/// the field is `missing-field` or `mistitled-field`. The field's finding
/// stands; a defect only one of them sees is kept. Runs after the chapter-rule
/// pass and before any `--only`/`--ignore` selection.
pub(crate) fn pair_fields_with_rules(
    catalog: &Catalog,
    schema: &Schema,
    frame: Frame<'_>,
    report: &mut CheckReport,
) {
    let mut claims: BTreeSet<(Channel, PathBuf, usize, String)> = BTreeSet::new();
    for_each_finding(catalog, schema, frame, |declaration, finding| {
        let Some(field) = finding.field else {
            return;
        };
        if matches!(
            finding.diagnostic.code,
            Code::MISSING_FIELD | Code::MISTITLED_FIELD
        ) {
            claims.insert((
                finding.channel,
                declaration.file.clone(),
                declaration.line,
                field.expressed_name().to_ascii_lowercase(),
            ));
        }
    });
    if claims.is_empty() {
        return;
    }
    for (channel, diagnostics) in [
        (Channel::Error, &mut report.errors),
        (Channel::Warning, &mut report.warnings),
        (Channel::Suggestion, &mut report.suggestions),
    ] {
        diagnostics.retain(|diagnostic| {
            let (Some(path), Some(line), Some(name)) = (
                diagnostic.path.as_ref(),
                diagnostic.line,
                absent_chapter(diagnostic),
            ) else {
                return true;
            };
            let claim = (channel, path.clone(), line, name.to_ascii_lowercase());
            !claims.contains(&claim)
        });
    }
}

/// The chapter name a `chapter-cardinality` finding counted zero of, read off
/// the message §FS-rules.7.2.1 fixes: `<ID> has 0 <NAME> chapters; …`.
fn absent_chapter(diagnostic: &Diagnostic) -> Option<&str> {
    if diagnostic.code != Code::CHAPTER_CARDINALITY {
        return None;
    }
    let (_, rest) = diagnostic.message.split_once(" has 0 ")?;
    rest.split_once(" chapters; ").map(|(name, _)| name)
}
