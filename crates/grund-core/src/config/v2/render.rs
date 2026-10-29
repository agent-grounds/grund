//! A v2 project written back out in v2 spelling (§FS-config.4.2): what
//! `grund config show` prints for a v2 config, every effective value under the
//! table that holds its concern. What it prints loads back as v2 to the same
//! effective values. `grund config show` never writes a v1 project in v2
//! spelling; migration does, by converting it to v2-shaped records first
//! (§FS-config-migrate).

use std::fmt::Write;

use crate::config::citations::{CitationDisjunction, render_citation_target};
use crate::config::fields::{FieldContent, Placement};
use crate::config::grounding::grounding_unit as unit;
use crate::config::kind::{KindIndex, KindResolution, escape_toml_basic};
use crate::config::project::Strength;
use crate::config::project::{Project, Rung, Threshold};
use crate::config::rows::{Form, Kind};

/// `value` as a TOML basic string. Beyond the `\` and `"` the shared escaper
/// takes, a line feed, a tab and a carriage return are written as the escapes the
/// reader reads back, so no string this prints runs over a line and what it prints
/// loads to the same value (§FS-config.4.2, §FS-config-v2.reader.5.strings). The
/// shared escaper keeps its bytes for version 1's writers
/// (§DF-v2-string-closes-on-its-line.2.5).
pub(super) fn string(value: &str) -> String {
    let escaped = escape_toml_basic(value)
        .replace('\n', "\\n")
        .replace('\t', "\\t")
        .replace('\r', "\\r");
    format!("\"{escaped}\"")
}

pub(super) fn list(values: impl IntoIterator<Item = impl AsRef<str>>) -> String {
    let values: Vec<String> = values
        .into_iter()
        .map(|value| string(value.as_ref()))
        .collect();
    format!("[{}]", values.join(", "))
}

impl Project {
    /// §FS-config.4.2: this project as a v2 `grund.toml`, or `None` when a v1
    /// file spelled it — a v1 config prints the bytes it always printed.
    pub fn v2_toml(&self) -> Option<String> {
        (self.version == 2).then(|| {
            let mut out = String::new();
            self.envelope(&mut out);
            self.schema_tables(&mut out);
            self.rules_tables(&mut out);
            self.presentation_tables(&mut out);
            out
        })
    }

    fn envelope(&self, out: &mut String) {
        out.push_str("grund_config_version = 2\n");
        if let Some(name) = &self.name {
            let _ = writeln!(out, "project_name = {}", string(name));
        }
        let workspace = &self.workspace;
        if workspace.declared {
            let _ = writeln!(out, "\n[workspace]\nmembers = {}", list(&workspace.members));
            if !workspace.optional_members.is_empty() {
                let _ = writeln!(
                    out,
                    "optional_members = {}",
                    list(&workspace.optional_members)
                );
            }
            let _ = writeln!(out, "include_root = {}", workspace.include_root);
        }
    }

    fn schema_tables(&self, out: &mut String) {
        let schema = &self.schema;
        let ids = &schema.ids;
        let depth = match ids.section_heading_levels.as_str() {
            "warn" => "warn",
            "suggest" => "should",
            "loose" => "may",
            _ => "must",
        };
        let shorthand = match schema.citation.shorthand {
            crate::config::record::ShorthandPolicy::Canonical => "canonical",
            crate::config::record::ShorthandPolicy::Accepted => "accepted",
        };
        let _ = writeln!(
            out,
            "\n[schema]\nmarker = {}\nshorthand = \"{shorthand}\"\nid_format = {}\nslug_pattern = {}\nnumber_pattern = {}\nsection_separator = {}\nheading_depth = \"{depth}\"",
            string(&schema.citation.marker),
            string(&ids.format),
            string(&ids.slug_pattern),
            string(&ids.number_pattern),
            string(&ids.section_separator),
        );
        super::sources_model::render(out, &schema.sources);
        let notes = &schema.notes;
        let _ = writeln!(out, "\n[schema.notes.lines]\nmust = {}", notes.max_lines);
        if notes.warn_on_suggested {
            let _ = writeln!(out, "warn = {}", notes.suggested_lines);
        }
        thresholds(out, &notes.extra_lines);
        let _ = writeln!(
            out,
            "\n[schema.notes.columns]\nmust = {}",
            notes.max_columns
        );
        thresholds(out, &notes.extra_columns);
        let text = notes.inline_style != "citation-only";
        let _ = writeln!(out, "\n[schema.notes.text]\nmust = {text}");
        if notes.layout != "any" || notes.layout_check != "off" {
            let strength = match notes.layout_check.as_str() {
                "error" => "must",
                "warn" => "warn",
                "suggest" => "should",
                _ => "may",
            };
            let _ = writeln!(
                out,
                "\n[schema.notes.layout]\n{strength} = {}",
                string(&notes.layout)
            );
        }
        if schema.leads.is_some() || !schema.lead_thresholds.is_empty() {
            out.push_str("\n[schema.leads.words]\n");
            if let Some(leads) = &schema.leads {
                let _ = writeln!(out, "warn = {}", leads.max);
            }
            thresholds(out, &schema.lead_thresholds);
        }
        for row in &schema.rows {
            let _ = writeln!(out, "\n[schema.kinds.{}]", row.name);
            // §FS-config-v2.schema.places.ownership: one `files` and one `folders`
            // list, in that order, each normalized and in written order.
            let files: Vec<&str> = row.files().collect();
            let folders: Vec<&str> = row.folders().collect();
            for (key, paths) in [("files", files), ("folders", folders)] {
                if !paths.is_empty() {
                    let _ = writeln!(out, "{key} = {}", list(paths));
                }
            }
            // §FS-config-v2.schema.places.complement: `code` is non-citable by name.
            if row.kind.is_none() && !row.is_complement() {
                out.push_str("citable = false\n");
            }
            if row.places.iter().any(|place| !place.scanned) {
                out.push_str("scan = false\n");
            }
            let Some(kind) = &row.kind else {
                continue;
            };
            match &kind.index {
                KindIndex::Default => {}
                KindIndex::Disabled => out.push_str("index = false\n"),
                KindIndex::Named(name) => {
                    let _ = writeln!(out, "index = {}", string(name));
                }
            }
            if let Some(format) = &kind.id_format {
                let _ = writeln!(out, "id_format = {}", string(format));
            }
            if let crate::config::rows::Origin::External { fetch } = &kind.origin {
                let _ = writeln!(out, "fetch = {}", string(fetch));
            }
            fields(out, &row.name, kind);
        }
    }

    fn rules_tables(&self, out: &mut String) {
        let rules = &self.rules;
        let citations = &rules.citations;
        if citations.declared {
            out.push_str("\n[rules.citations]\n");
            if let Some(default) = citations.global_default {
                let _ = writeln!(out, "default = \"{}\"", default.as_str());
            }
            let written = citations.written.0.iter().map(String::as_str);
            for kind in self.row_order(citations.per_kind.keys().map(String::as_str), written) {
                let kind_rules = &citations.per_kind[kind];
                let _ = writeln!(out, "\n[rules.citations.{kind}]");
                if let Some(default) = kind_rules.default {
                    let _ = writeln!(out, "default = \"{}\"", default.as_str());
                }
                for (level, disjunctions) in kind_rules.lists() {
                    if !disjunctions.is_empty() {
                        let _ = writeln!(out, "{} = {}", level.as_str(), entries(disjunctions));
                    }
                }
            }
        }
        if let Some(ladder) = &rules.grounding.ladder {
            out.push_str("\n[rules.citations.grounding]\n");
            rungs(out, ladder);
        }
        let places = rules.grounding.kinds.keys().map(String::as_str);
        for place in self.row_order(places, std::iter::empty()) {
            if let Some(ladder) = &rules.grounding.kinds[place].ladder {
                let _ = writeln!(out, "\n[rules.citations.{place}.grounding]");
                rungs(out, ladder);
            }
        }
        if !rules.resolution.is_empty() {
            out.push_str("\n[rules.resolution]\n");
            for (kind, resolution) in &rules.resolution {
                let strength = match resolution {
                    KindResolution::Must => "must",
                    KindResolution::Should => "warn",
                };
                let _ = writeln!(out, "{kind} = \"{strength}\"");
            }
        }
    }

    /// §FS-config.4.2.4: the names of tables keyed by a row name, in the order
    /// the file writes its `[schema.kinds]` rows, then each name that is no row
    /// in `written` order, then any left over, in name order. `config migrate`'s
    /// candidate prints in this order too (§FS-config-migrate.2.1).
    fn row_order<'a>(
        &'a self,
        names: impl Iterator<Item = &'a str>,
        written: impl Iterator<Item = &'a str>,
    ) -> Vec<&'a str> {
        let names: Vec<&str> = names.collect();
        let rows = self.schema.rows.iter().map(|row| row.name.as_str());
        let mut ordered: Vec<&str> = Vec::with_capacity(names.len());
        for name in rows.chain(written).chain(names.iter().copied()) {
            if names.contains(&name) && !ordered.contains(&name) {
                ordered.push(name);
            }
        }
        ordered
    }

    fn presentation_tables(&self, out: &mut String) {
        let presentation = &self.presentation;
        out.push_str("\n[presentation]\n");
        if let Some(description) = &presentation.description {
            let _ = writeln!(out, "description = {}", string(description));
        }
        let _ = writeln!(out, "trigger = {}", string(&presentation.trigger));
        if let Some(conversation) = &presentation.conversation {
            let _ = writeln!(out, "conversation = {}", string(conversation));
        }
        // §FS-config-v2.presentation: `home` is the default, so only `root` is spelled.
        if presentation.rules_at_root {
            out.push_str("rules = \"root\"\n");
        }
        let kinds = presentation.kinds.keys().map(String::as_str);
        for kind in self.row_order(kinds, std::iter::empty()) {
            let title = &presentation.kinds[kind];
            let _ = writeln!(
                out,
                "\n[presentation.kinds.{kind}]\ntitle = {}",
                string(title)
            );
        }
        let _ = writeln!(
            out,
            "\n[presentation.fmt]\nanchors = \"github\"\nlinks = \"index\"\nexclude = {}",
            list(&presentation.fmt.exclude)
        );
    }
}

/// §FS-fields.declare: a kind's `form`, `closed` and fields, each field under
/// its own header in header order (§FS-config-v2.reader.2).
fn fields(out: &mut String, name: &str, kind: &Kind) {
    match kind.form {
        Form::Prose => {}
        Form::Value => out.push_str("form = \"value\"\n"),
        Form::Rule => out.push_str("form = \"rule\"\n"),
    }
    if kind.closed != Strength::May {
        let _ = writeln!(out, "closed = \"{}\"", kind.closed.as_str());
    }
    for field in &kind.fields {
        let _ = writeln!(out, "\n[schema.kinds.{name}.fields.{}]", field.name);
        if let Some(title) = &field.title {
            let _ = writeln!(out, "title = {}", string(title));
        }
        let _ = writeln!(out, "presence = \"{}\"", field.presence.as_str());
        match &field.placement {
            Some(Placement::First) => out.push_str("position = \"first\"\n"),
            Some(Placement::Last) => out.push_str("position = \"last\"\n"),
            Some(Placement::After(after)) => {
                let _ = writeln!(out, "after = {}", string(after));
            }
            None => {}
        }
        match &field.content {
            FieldContent::Prose => out.push_str("content = \"prose\"\n"),
            FieldContent::Values => out.push_str("content = \"values\"\n"),
            FieldContent::OneOf(values) => {
                let _ = writeln!(out, "one_of = {}", list(values));
            }
            // §FS-fields.front-matter.config: its `closed`, then a table per key.
            FieldContent::FrontMatter(shape) => {
                let _ = writeln!(out, "closed = \"{}\"", shape.closed.as_str());
                for key in &shape.keys {
                    let _ = writeln!(
                        out,
                        "\n[schema.kinds.{name}.fields.{}.keys.{}]",
                        field.name, key.name
                    );
                    let _ = writeln!(out, "presence = \"{}\"", key.presence.as_str());
                    if let Some(values) = &key.one_of {
                        let _ = writeln!(out, "one_of = {}", list(values));
                    }
                }
            }
        }
    }
}

fn thresholds(out: &mut String, thresholds: &[Threshold]) {
    for threshold in thresholds {
        let _ = writeln!(out, "{} = {}", threshold.strength.as_str(), threshold.value);
    }
}

fn rungs(out: &mut String, rungs: &[Rung]) {
    for rung in rungs {
        let _ = writeln!(out, "{} = \"{}\"", rung.strength.as_str(), unit(rung.level));
    }
}

fn entries(disjunctions: &[CitationDisjunction]) -> String {
    let rendered: Vec<String> = disjunctions
        .iter()
        .map(|disjunction| {
            let targets: Vec<String> = disjunction
                .targets
                .iter()
                .map(render_citation_target)
                .collect();
            targets.join("|")
        })
        .collect();
    list(&rendered)
}
