//! The grounding sentence of the `### Citation directions` section
//! (§FS-init.2.3.5.7): which rows' clauses it carries, read from each row's
//! effective grounding, in a file of its own beside the bullets it opens
//! (§AR-core-module-layout.1). Placement asks the same question of one row, so
//! the predicate lives here too (§FS-init.2.3.5.7.1).

use super::citation_directions::join_english_list;
use crate::config::{Project, Row, Run, homeless_row_grounding, row_grounding};

/// The grounding sentence (§FS-init.2.3.5.7), generated from each row's
/// effective `require_grounding` and the configured non-citable homes. It claims
/// only what §FS-check.3.6 enforces, and it distinguishes *cite* from *declare*: a
/// source file grounds by citing a declared ID **or** by declaring one inline,
/// while a file in a non-citable home can only cite, because a declaration there
/// is misplaced (§FS-declarations.checks.misplaced-declaration.3). An unwalked
/// home (§FS-config.3.4.7.4) is left out — nothing in it is scanned, so the rule
/// never reaches it. Per-row grounding *levels* (§FS-config.3.4.8) are not this
/// sentence's. Each row's pair is read through [`row_grounding`] — the row's key
/// or v2 ladder, else the default OR the run's `--require-grounding`
/// (§FS-config.3.4.8.3) — so the drift `check --require-grounding` reports is
/// unchanged.
pub(super) fn citation_grounding_sentence(
    project: &Project,
    run: &Run,
    here: &dyn Fn(&str) -> bool,
) -> Option<String> {
    let base = "Every source file must cite a declared ID or declare one inline";
    let homeless = project.schema.complement_name();
    // §FS-config-v2.schema.places: every place of each such row; §FS-init.2.3.5.7.1:
    // under scoped guidance only the rows that land here.
    let homes = project
        .schema
        .rows
        .iter()
        .filter(|row| row_has_home_clause(project, run, row) && here(&row.name))
        .filter_map(Row::place_labels)
        .collect::<Vec<_>>();
    let homes = (!homes.is_empty()).then(|| join_english_list(&homes, "and"));
    // §FS-init.2.3.5.7: the source-file clause only where the homeless kind's row grounds.
    match (here(homeless) && homeless_grounds(project, run), homes) {
        (true, None) => Some(format!("{base}.")),
        (true, Some(homes)) => Some(format!("{base}; every file under {homes} must cite one.")),
        (false, Some(homes)) => Some(format!("Every file under {homes} must cite one.")),
        (false, None) => None,
    }
}

/// Whether the homeless kind's row has grounding on, which opens the grounding
/// sentence with the source-file clause (§FS-init.2.3.5.7).
fn homeless_grounds(project: &Project, run: &Run) -> bool {
    homeless_row_grounding(&project.schema, &project.rules, run).0
}

/// Whether `row` is a non-citable home other than the homeless kind's
/// (§FS-init.2.3.5.2.1).
fn is_home(row: &Row, homeless: &str) -> bool {
    row.kind.is_none() && row.name != homeless
}

/// Whether `row` is a scanned non-citable home whose own row has grounding on,
/// which the grounding sentence names (§FS-init.2.3.5.7, §FS-init.2.3.5.7.1); an
/// unwalked home is left out.
fn row_has_home_clause(project: &Project, run: &Run, row: &Row) -> bool {
    is_home(row, project.schema.complement_name())
        && row.places.first().is_none_or(|place| place.scanned)
        && row.place_labels().is_some()
        && row_grounding(&project.rules, run, &row.name).0
}

/// Whether `row` carries a clause of the grounding sentence: the complement's
/// source-file clause, or a scanned non-citable home's (§FS-init.2.3.5.7.1).
pub(crate) fn row_has_grounding_clause(project: &Project, run: &Run, row: &str) -> bool {
    if row == project.schema.complement_name() {
        return homeless_grounds(project, run);
    }
    project
        .schema
        .rows
        .iter()
        .any(|candidate| candidate.name == row && row_has_home_clause(project, run, candidate))
}
