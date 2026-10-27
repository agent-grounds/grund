//! The front-matter field's findings (§FS-fields.front-matter.findings): the
//! four field codes at front-matter sites, `unreadable-front-matter` and
//! `front-matter-heading`, judged from what the scan recorded of the block
//! (§FS-fields.front-matter.reading), never from a second read of it. A sibling
//! of `fields.rs`, whose shape and channels it shares.

use std::collections::BTreeSet;

use super::fields::{Channel, FieldFinding, Shape, channel};
use crate::config::{FRONT_MATTER, Field, FrontMatterShape};
use crate::model::{
    Code, FrontMatterAbsence, FrontMatterBlock, FrontMatterKeySite, FrontMatterUnreadable, remedies,
};

impl<'a> Shape<'a> {
    /// §FS-fields.front-matter.findings: every finding of the front-matter
    /// `field`, whose shape is `shape`, on this declaration.
    pub(super) fn front_matter(
        &self,
        field: &'a Field,
        shape: &'a FrontMatterShape,
        out: &mut Vec<FieldFinding<'a>>,
    ) {
        let Some(recorded) = &self.declaration.front_matter else {
            return;
        };
        // §FS-fields.checks.front-matter-heading: the heading alone, always an error.
        for (path, line) in &recorded.headings {
            let message = format!(
                "{} is a heading, but kind {} reads {FRONT_MATTER} from the file's front matter; {}",
                self.section(path),
                self.kind,
                remedies::front_matter_heading()
            );
            out.push(self.finding(
                Some(field),
                Channel::Error,
                Code::FRONT_MATTER_HEADING,
                *line,
                message,
            ));
        }
        match &recorded.block {
            FrontMatterBlock::Absent(absence) => self.absent(field, *absence, out),
            FrontMatterBlock::Unreadable(reason) => self.unreadable(field, reason, out),
            FrontMatterBlock::Read(keys) => {
                self.keys(field, shape, keys, out);
                self.undeclared(shape, keys, out);
            }
        }
    }

    /// `<ID>.front-matter`, or one of its keys, in the project's separator.
    fn section(&self, path: &str) -> String {
        format!(
            "{}{}{}",
            self.id,
            self.separator,
            path.replace('.', self.separator)
        )
    }

    /// §FS-fields.front-matter.findings: no front matter, at the title line, the
    /// remedy saying why there is none.
    fn absent(
        &self,
        field: &'a Field,
        absence: FrontMatterAbsence,
        out: &mut Vec<FieldFinding<'a>>,
    ) {
        let Some(channel) = channel(field.presence) else {
            return;
        };
        let remedy = match absence {
            FrontMatterAbsence::NoBlock => remedies::missing_field_block(),
            FrontMatterAbsence::ByteOrderMark(delimiter) => {
                remedies::missing_field_byte_order_mark(delimiter)
            }
            FrontMatterAbsence::NotFirst => remedies::missing_field_not_first(),
            FrontMatterAbsence::DocComment => remedies::missing_field_doc_comment(),
        };
        let message = format!(
            "{} has no {FRONT_MATTER} field; kind {} declares it; {remedy}",
            self.id, self.kind
        );
        let line = self.declaration.line;
        out.push(self.finding(Some(field), channel, Code::MISSING_FIELD, line, message));
    }

    /// §FS-fields.checks.unreadable-front-matter: at the opening delimiter,
    /// always an error, and no key judged.
    fn unreadable(
        &self,
        field: &'a Field,
        reason: &FrontMatterUnreadable,
        out: &mut Vec<FieldFinding<'a>>,
    ) {
        let (language, remedy) = match reason {
            FrontMatterUnreadable::Yaml { .. } => {
                ("YAML", remedies::unreadable_front_matter_yaml())
            }
            FrontMatterUnreadable::Toml { .. } => {
                ("TOML", remedies::unreadable_front_matter_toml())
            }
        };
        let (FrontMatterUnreadable::Yaml { reason, line }
        | FrontMatterUnreadable::Toml { reason, line }) = reason;
        let message = format!(
            "{} does not parse as {language}: {reason} at line {line}; {remedy}",
            self.section(FRONT_MATTER)
        );
        let code = Code::UNREADABLE_FRONT_MATTER;
        out.push(self.finding(Some(field), Channel::Error, code, 1, message));
    }

    /// §FS-fields.front-matter.findings: each declared key, absent at the opening
    /// delimiter on its own presence, or present and held to its `one_of` at its
    /// first occurrence, a `may` key's violation an error
    /// (§FS-fields.strengths.may).
    fn keys(
        &self,
        field: &'a Field,
        shape: &'a FrontMatterShape,
        keys: &[FrontMatterKeySite],
        out: &mut Vec<FieldFinding<'a>>,
    ) {
        for key in &shape.keys {
            let site = keys
                .iter()
                .find(|site| site.section && site.name == key.name);
            let Some(site) = site else {
                if let Some(channel) = channel(key.presence) {
                    let message = format!(
                        "{} has no {} key; kind {} declares it; {}",
                        self.section(FRONT_MATTER),
                        key.name,
                        self.kind,
                        remedies::missing_field_key()
                    );
                    // A key is no chapter a chapter-presence rule could count (§FS-fields.rules).
                    out.push(self.finding(None, channel, Code::MISSING_FIELD, 1, message));
                }
                continue;
            };
            let Some(values) = &key.one_of else {
                continue;
            };
            let section = self.section(&format!("{FRONT_MATTER}.{}", key.name));
            let admits = values.join(", ");
            let message = match &site.one_line {
                Some(value) if values.contains(value) => continue,
                Some(value) => format!(
                    "{section} is \"{value}\"; kind {} admits {admits}; {}",
                    self.kind,
                    remedies::invalid_field_value_value()
                ),
                None => format!(
                    "{section} is not a one-line value; kind {} admits {admits}; {}",
                    self.kind,
                    remedies::invalid_field_value_one_line()
                ),
            };
            let channel = channel(key.presence).unwrap_or(Channel::Error);
            let code = Code::INVALID_FIELD_VALUE;
            out.push(self.finding(Some(field), channel, code, site.line, message));
        }
    }

    /// §FS-fields.checks.undeclared-chapter: in a closed field, each key it does
    /// not declare, once, at its first occurrence; one that cannot be declared
    /// gets the shorter remedy (§FS-fields.front-matter.keys).
    fn undeclared(
        &self,
        shape: &'a FrontMatterShape,
        keys: &[FrontMatterKeySite],
        out: &mut Vec<FieldFinding<'a>>,
    ) {
        let Some(channel) = channel(shape.closed) else {
            return;
        };
        let names: Vec<&str> = shape.keys.iter().map(|key| key.name.as_str()).collect();
        let listed = if names.is_empty() {
            "none".to_string()
        } else {
            names.join(", ")
        };
        let mut seen = BTreeSet::new();
        for site in keys {
            if (site.section && names.contains(&site.name.as_str())) || !seen.insert(&site.name) {
                continue;
            }
            let remedy = if site.section {
                remedies::undeclared_chapter_key()
            } else {
                remedies::undeclared_chapter_undeclarable_key()
            };
            let message = format!(
                "{}{}{} is not a key of kind {}'s front matter, which is closed; its keys: {listed}; {remedy}",
                self.section(FRONT_MATTER),
                self.separator,
                site.name,
                self.kind
            );
            let code = Code::UNDECLARED_CHAPTER;
            out.push(self.finding(None, channel, code, site.line, message));
        }
    }
}
