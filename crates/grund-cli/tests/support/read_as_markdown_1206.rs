//! Issue #1206's first pull request, its cases run alone: whether a file is read as
//! Markdown is the configuration's one answer, never its name (§FS-fmt.2.3.1,
//! §AR-config.1.6); every home of a row is judged by it (§FS-config-v2.schema.sources.rules,
//! §FS-config.3.4.9); and what 1.0.0 refuses warns first, naming that release
//! (§FS-config-v2.checks). They are held out of the full pass until the fix lands, and
//! implement removes the hold.

use super::case_runner::CaseKind::E2e;
use super::case_runner::{assert_every_case_passed, run_case};
use super::repo_root;

/// Red until #1206: the full pass skips these; implement removes this.
pub const RED_UNTIL_1206: [&str; 15] = CASES;

const CASES: [&str; 15] = [
    "check-index-default-not-read-as-markdown-quiet",
    "check-index-not-read-as-markdown-mapped",
    "check-index-not-read-as-markdown-unadmitted",
    "check-rule-file-home-read-as-source-beside",
    "check-rule-file-home-read-as-source-first",
    "check-rule-file-home-read-as-source-mapped",
    "check-rule-folder-home-read-as-source",
    "config-index-named-not-markdown-warns",
    "config-index-read-as-markdown-loads",
    "config-md-not-markdown-data",
    "config-md-not-markdown-source",
    "config-rule-folder-not-markdown-gitkeep",
    "config-rule-folder-not-markdown-source",
    "config-value-home-absent-files",
    "config-value-home-absent-folders",
];

#[test]
#[ignore = "red until #1206 reads every file as the configuration reads it and warns about what 1.0.0 refuses; implement removes this"]
fn read_as_markdown_1206_contract() {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let outcomes = CASES
        .iter()
        .map(|name| run_case(&root, &cases.join(name), E2e))
        .collect::<Vec<_>>();
    assert_every_case_passed("read as Markdown (#1206)", &outcomes);
}
