//! A v2 project written back out in v2 spelling (§FS-config.4.2): what
//! `grund config show` prints for a v2 config, every effective value under the
//! table that holds its concern. What it prints loads back as v2 to the same
//! effective values; a v1 project is never written in v2 spelling here, which
//! would be migration, agent-grounds/grund#461's.

use std::fmt::Write;

use super::rules::unit;
use crate::config::citations::{CitationDisjunction, render_citation_target};
use crate::config::kind::{KindIndex, KindResolution, escape_toml_basic};
use crate::config::project::{Project, Rung, Threshold};
use crate::config::rows::Extent;

fn string(value: &str) -> String {
    format!("\"{}\"", escape_toml_basic(value))
}

fn list<'a>(values: impl IntoIterator<Item = &'a String>) -> String {
    let values: Vec<String> = values.into_iter().map(|value| string(value)).collect();
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
        let _ = writeln!(
            out,
            "\n[schema.sources]\nlanguages = [\"markdown\"]\nrespect_gitignore = {}",
            schema.sources.respect_gitignore
        );
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
            for place in &row.places {
                let (key, path) = match &place.extent {
                    Extent::File(path) => ("files", path),
                    Extent::Folder(path) => ("folders", path),
                    Extent::Complement => continue,
                };
                let _ = writeln!(out, "{key} = {}", list([path]));
            }
            if row.kind.is_none() {
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
            for (kind, kind_rules) in &citations.per_kind {
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
        for (place, grounding) in &rules.grounding.kinds {
            if let Some(ladder) = &grounding.ladder {
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
        for (kind, title) in &presentation.kinds {
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
