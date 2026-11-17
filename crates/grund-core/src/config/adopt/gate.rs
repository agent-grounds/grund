//! §AR-config.10.3: adoption's gate on its candidate, from migration's parts. The candidate is
//! loaded back through `pending_config` (§FS-config-adopt-builtins.6.1), its readings are held to
//! the plan by the one per-suffix reading (§FS-config-adopt-builtins.6.2), and its other records
//! to the file's by the records gate's sections (§FS-config-adopt-builtins.6.3).

use std::collections::BTreeSet;
use std::path::Path;

use super::plan::{Plan, read_suffixes};
use super::record::AdoptReport;
use crate::config::discovery::pending_config;
use crate::config::file_reading::{SuffixReading, model_suffix_reading};
use crate::config::migrate::compare_but_readings;
use crate::config::project::Project;
use crate::config::run::{Invocation, Selection};

/// §FS-config-adopt-builtins.6.2: the heading of the readings' difference.
const READINGS: &str = "differs: the readings adoption planned";
/// §FS-config-adopt-builtins.6.3: the heading of the other records' difference.
const RECORDS: &str = "differs: the records";

/// §FS-config-adopt-builtins.6.1–6.3: hold `report`'s candidate to `plan` and to `file`, the
/// project the plan was built from, recording what each part says in `report`. A candidate that
/// does not load stops the gate there.
pub(crate) fn gate(report: &mut AdoptReport, file: &Project, plan: &Plan, root: &Path) {
    let invocation = Invocation::new(root, false, Selection::Project);
    let loaded = match pending_config(&report.candidate, root, &invocation) {
        Ok(config) => config,
        Err(error) => {
            report.load_error = Some(format!("{error:#}"));
            return;
        }
    };
    let candidate = loaded.project();
    report.readings = readings(plan, candidate);
    let records = compare_but_readings(file, candidate, root);
    if !records.is_empty() {
        report.records = std::iter::once(RECORDS.to_string())
            .chain(records)
            .collect();
    }
}

/// §FS-config-adopt-builtins.6.2: every suffix the file or the candidate reads, whose candidate
/// reading is not the one the plan expects, as the heading and a `- `/`+ ` pair each.
fn readings(plan: &Plan, candidate: &Project) -> Vec<String> {
    let model = candidate.schema.sources.model.as_ref();
    let mut suffixes: BTreeSet<String> = plan.expected.keys().cloned().collect();
    suffixes.extend(model.map(read_suffixes).unwrap_or_default());
    let mut lines = Vec::new();
    for suffix in suffixes {
        let planned = plan.expected.get(&suffix);
        let read = model.and_then(|model| model_suffix_reading(model, &suffix));
        if planned != read.as_ref() {
            lines.push(format!("- {suffix}: {}", shown(planned)));
            lines.push(format!("+ {suffix}: {}", shown(read.as_ref())));
        }
    }
    if !lines.is_empty() {
        lines.insert(0, READINGS.to_string());
    }
    lines
}

/// A reading as the records gate prints one, and `none` for a suffix not read.
fn shown(reading: Option<&SuffixReading>) -> String {
    reading.map_or_else(|| "none".to_string(), |reading| format!("{reading:?}"))
}
