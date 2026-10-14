//! §AR-config.7.4: a v1 `Project` converted into v2-shaped records, which
//! `Project::v2_toml` spells, so `config show` and `config migrate` cannot
//! disagree on v2 spelling. Each conversion is a row of
//! §FS-config-migrate.mapping; what a v1 file never wrote is written out
//! (§FS-config-migrate.3.3).

use crate::config::citations::CitationLevel;
use crate::config::grounding::configured_row_grounding;
use crate::config::kind::KindResolution;
use crate::config::project::{KindGrounding, Project, Rung, Strength, Threshold};
use crate::config::rows::{Extent, Nesting, Place, Row};
use crate::config::sources::SourceModel;

/// A positive default creates no obligation in v1, so it is written `may`
/// (§FS-config-v2.rules.citations.default).
pub(super) fn may_default(level: Option<CitationLevel>) -> Option<CitationLevel> {
    level.map(|level| match level {
        CitationLevel::Must | CitationLevel::Warn | CitationLevel::Should => CitationLevel::May,
        other => other,
    })
}

/// The v2-shaped records for `v1`, with `model` its sources and `code` the
/// places the complement gains.
pub(super) fn records(v1: &Project, model: SourceModel, code: Vec<Place>) -> Project {
    let mut p = v1.clone();
    p.version = 2;
    let sources = &mut p.schema.sources;
    sources.include = None;
    sources.include_source = None;
    sources.exclude = Vec::new();
    sources.extensions = model.text_suffixes();
    sources.comment_prefixes = Vec::new();
    sources.docstring_python = false;
    sources.model = Some(model);
    // §FS-config-v2.schema.places.complement: `code`, with a row only when it has places.
    if !code.is_empty() {
        let complement = Place {
            extent: Extent::Complement,
            scanned: true,
            source: None,
        };
        match p.schema.rows.iter_mut().find(|row| row.is_complement()) {
            Some(row) => row.places.extend(code),
            None => p.schema.rows.push(Row {
                name: "code".into(),
                places: std::iter::once(complement).chain(code).collect(),
                kind: None,
            }),
        }
    }
    // §FS-config-v2.schema.places.ownership: no two homes nest, or it refused.
    p.schema.nesting = Nesting::DeepestPlaceWins;
    // A suggested length is the `may` rung, or `warn` with `warn_on_suggested`.
    let notes = &mut p.schema.notes;
    if !notes.warn_on_suggested {
        notes.extra_lines.push(Threshold {
            strength: Strength::May,
            value: notes.suggested_lines,
            source: None,
        });
    }
    p.rules.grounding = ladders(v1);
    let citations = &mut p.rules.citations;
    citations.global_default = may_default(citations.global_default);
    for rules in citations.per_kind.values_mut() {
        rules.default = may_default(rules.default);
    }
    // *fetch resolves must*: v2's default for a fetched kind.
    p.rules
        .resolution
        .retain(|_, resolution| *resolution != KindResolution::Must);
    p
}

/// *kind grounding inherits*: the effective pair per row, as whole ladders. The
/// project's `must` rung is its pair when grounding is on; a row whose pair
/// differs gets a ladder of its own, `may = "file"` where it is off.
fn ladders(v1: &Project) -> crate::config::project::Grounding {
    let mut grounding = v1.rules.grounding.clone();
    let rung = |strength, level| Rung {
        strength,
        level,
        source: None,
    };
    let project = (grounding.require, grounding.level);
    grounding.ladder = project.0.then(|| vec![rung(Strength::Must, project.1)]);
    grounding.level_source = None;
    let mut names: Vec<String> = v1.schema.rows.iter().map(|row| row.name.clone()).collect();
    if !names.iter().any(|name| name == v1.schema.complement_name()) {
        names.push(v1.schema.complement_name().to_string());
    }
    grounding.kinds.clear();
    for name in names {
        let pair = configured_row_grounding(&v1.rules, &name);
        if pair == project || (!pair.0 && !project.0) {
            continue;
        }
        let ladder = match pair {
            (true, level) => vec![rung(Strength::Must, level)],
            (false, _) => vec![rung(Strength::May, 1)],
        };
        grounding.kinds.insert(
            name,
            KindGrounding {
                ladder: Some(ladder),
                ..KindGrounding::default()
            },
        );
    }
    grounding
}
