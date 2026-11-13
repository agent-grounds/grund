//! Process-level coverage of grund#1207, one spelling for a reported path and one rendering for a
//! scan failure (§FS-errors.4, §FS-cli.3.4, §FS-workspace.8.7.3), kept with the end-to-end tests.

#[path = "../../../tests/e2e/report_path_one_spelling.rs"]
mod contract;
