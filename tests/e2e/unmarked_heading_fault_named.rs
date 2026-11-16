//! Black-box: a heading whose path is already written in place, but that lacks only a title or
//! has a `_` in a named component, is told that fault and shown the heading that repairs it with
//! its own path kept (§FS-declarations.checks.unmarked-heading.2.3), after the unchanged message
//! of §FS-declarations.checks.unmarked-heading.2. The controls hold what must not move: a titled,
//! hyphenated heading draws nothing, a dotted heading keeps its relative-form clause alone
//! (§FS-declarations.checks.unmarked-heading.2.1), and a heading the guard turns away keeps the
//! message exactly as before.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

const CONFIG: &str = "grund_config_version = 2\nproject_name = \"grund-1372\"\n\n\
                      [schema.kinds.FS]\nfolders = [\"docs/fs\"]\nindex = false\n\
                      id_format = \"{kind}-{number}-{slug}\"\n\n\
                      [schema.kinds.FS.fields.requirements]\ntitle = \"Requirements\"\n";

/// The fixture of agent-grounds/grund#1372: four files of cases beside their controls, one with
/// both faults on one heading, and one of headings the guard must turn away.
const FILES: &[(&str, &str)] = &[
    (
        "FS-009-bare.md",
        "# FS-009-bare: Bare paths\n\n## requirements: Requirements\n\n\
         ### requirements.1\n\nBody one.\n\n\
         ### requirements.2: Two\n\nBody two.\n\n\
         ### requirements.3:\n\nBody three.\n",
    ),
    (
        "FS-010-num.md",
        "# FS-010-num: A bare number\n\n## requirements: Requirements\n\nBody.\n\n\
         ## 1.\n\nBody one.\n\n## 2. Second\n\nBody two.\n",
    ),
    (
        "FS-011-handle.md",
        "# FS-011-handle: Handles with an underscore\n\n## requirements: Requirements\n\nBody.\n\n\
         ### requirements.size_format: Size format\n\nBody.\n\n\
         ### requirements.page-size: Page size\n\nBody.\n\n\
         ## open_questions: Open Questions\n\nBody.\n\n\
         ## open-issues: Open issues\n\nBody.\n",
    ),
    (
        "FS-012-relative.md",
        "# FS-012-relative: Relative form\n\n## requirements: Requirements\n\n### .2\n\nBody.\n",
    ),
    (
        "FS-013-both.md",
        "# FS-013-both: Both faults\n\n## requirements: Requirements\n\nBody.\n\n\
         ### requirements.size_format:\n\nBody.\n",
    ),
    (
        "FS-014-guard.md",
        "# FS-014-guard: Guards\n\n## requirements: Requirements\n\nBody.\n\n\
         ### requirements.1: One\n\nBody.\n\n\
         ### requirements.1\n\nBody.\n\n\
         ### extra.1\n\nBody.\n\n\
         ## requirements.page_size: Page size\n\nBody.\n\n\
         ## open_questions\n\nBody.\n\n\
         ## Notes_here: Notes\n\nBody.\n\n\
         ## 1\n\nBody.\n",
    ),
];

struct Tree(PathBuf);

impl Tree {
    fn new(test: &str) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1372-{test}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("docs/fs")).expect("create the kind folder");
        fs::write(root.join("grund.toml"), CONFIG).expect("write the config");
        for (name, body) in FILES {
            fs::write(root.join("docs/fs").join(name), body).expect("write a declaration");
        }
        Self(root)
    }

    /// Every `unmarked-heading` finding, as `(path:line, message)`, in report order.
    fn findings(&self) -> Vec<(String, String)> {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(["check", "--only", "unmarked-heading"])
            .env("NO_COLOR", "1")
            .current_dir(&self.0)
            .output()
            .expect("run grund");
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        assert_eq!(
            output.status.code(),
            Some(1),
            "stdout:\n{stdout}\nstderr:\n{stderr}"
        );
        stdout
            .lines()
            .map(|line| {
                let (site, message) = line
                    .split_once(": error: ")
                    .unwrap_or_else(|| panic!("not an error line: {line}"));
                (site.to_string(), message.to_string())
            })
            .collect()
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// The message §FS-declarations.checks.unmarked-heading.2 fixes today, byte for byte.
fn prefix(id: &str, suggested: &str, coordinate: &str) -> String {
    format!(
        "unmarked heading inside {id}; number it ({suggested}) as {id}.{coordinate}, declare an ID, \
         or use a bold label; this became an error in grund 0.16.0"
    )
}

const NO_TITLE: &str = "needs a title after it";
const UNDERSCORE: &str = "has a _, which a section name ([a-z][a-z0-9-]*) never takes";

/// The message at `site`, failing with the whole report when there is none.
fn message_at(findings: &[(String, String)], site: &str) -> String {
    findings
        .iter()
        .find(|(at, _)| at == site)
        .map(|(_, message)| message.clone())
        .unwrap_or_else(|| panic!("no unmarked-heading at {site}; found {findings:#?}"))
}

fn assert_message(findings: &[(String, String)], site: &str, expected: &str) {
    assert_eq!(message_at(findings, site), expected, "at {site}");
}

#[test]
#[ignore = "red until #1372 appends the fault clause of FS-declarations.checks.unmarked-heading.2.3; implement removes this"]
fn a_path_with_no_title_is_told_a_title_must_follow_it() {
    let findings = Tree::new("no-title").findings();
    assert_message(
        &findings,
        "docs/fs/FS-009-bare.md:5",
        &format!(
            "{}; the section path requirements.1 {NO_TITLE}, written ### requirements.1: Untitled",
            prefix(
                "FS-009-bare",
                "### requirements.3: requirements.1",
                "requirements.3"
            )
        ),
    );
    assert_message(
        &findings,
        "docs/fs/FS-009-bare.md:13",
        &format!(
            "{}; the section path requirements.3 {NO_TITLE}, written ### requirements.3: Untitled",
            prefix(
                "FS-009-bare",
                "### requirements.4: requirements.3:",
                "requirements.4"
            )
        ),
    );
    assert_message(
        &findings,
        "docs/fs/FS-010-num.md:7",
        &format!(
            "{}; the section path 1 {NO_TITLE}, written ## 1. Untitled",
            prefix("FS-010-num", "## 3. 1.", "3")
        ),
    );
}

#[test]
#[ignore = "red until #1372 appends the fault clause of FS-declarations.checks.unmarked-heading.2.3; implement removes this"]
fn an_underscore_in_a_handle_is_named_and_shown_hyphenated() {
    let findings = Tree::new("underscore").findings();
    assert_message(
        &findings,
        "docs/fs/FS-011-handle.md:7",
        &format!(
            "{}; the section path requirements.size_format {UNDERSCORE}, written \
             ### requirements.size-format: Size format",
            prefix(
                "FS-011-handle",
                "### requirements.1: requirements.size_format: Size format",
                "requirements.1"
            )
        ),
    );
    assert_message(
        &findings,
        "docs/fs/FS-011-handle.md:15",
        &format!(
            "{}; the section path open_questions {UNDERSCORE}, written \
             ## open-questions: Open Questions",
            prefix("FS-011-handle", "## 1. open_questions: Open Questions", "1")
        ),
    );
}

#[test]
#[ignore = "red until #1372 appends the fault clause of FS-declarations.checks.unmarked-heading.2.3; implement removes this"]
fn both_faults_on_one_heading_are_named_in_one_clause() {
    let findings = Tree::new("both").findings();
    assert_message(
        &findings,
        "docs/fs/FS-013-both.md:7",
        &format!(
            "{}; the section path requirements.size_format {UNDERSCORE}, and {NO_TITLE}, \
             written ### requirements.size-format: Untitled",
            prefix(
                "FS-013-both",
                "### requirements.1: requirements.size_format:",
                "requirements.1"
            )
        ),
    );
}

/// A control: it passes today and must keep passing. The titled, hyphenated forms beside each
/// case declare their sections, so only the cases, the contrast and the guard file are findings.
#[test]
fn a_titled_hyphenated_heading_draws_no_finding() {
    let findings = Tree::new("controls").findings();
    let sites = findings
        .iter()
        .map(|(site, _)| site.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        sites,
        [
            "docs/fs/FS-009-bare.md:5",
            "docs/fs/FS-009-bare.md:13",
            "docs/fs/FS-010-num.md:7",
            "docs/fs/FS-011-handle.md:7",
            "docs/fs/FS-011-handle.md:15",
            "docs/fs/FS-012-relative.md:5",
            "docs/fs/FS-013-both.md:7",
            "docs/fs/FS-014-guard.md:11",
            "docs/fs/FS-014-guard.md:15",
            "docs/fs/FS-014-guard.md:19",
            "docs/fs/FS-014-guard.md:23",
            "docs/fs/FS-014-guard.md:27",
            "docs/fs/FS-014-guard.md:31",
        ]
    );
}

/// A control: a dotted heading is §FS-declarations.checks.unmarked-heading.2.1's, and its message
/// ends at the relative-form clause, with no fault clause after it.
#[test]
fn a_dotted_heading_keeps_only_its_relative_clause() {
    let findings = Tree::new("relative").findings();
    assert_message(
        &findings,
        "docs/fs/FS-012-relative.md:5",
        &format!(
            "{}; under FS-012-relative.requirements a relative heading is written ### .1: Untitled",
            prefix(
                "FS-012-relative",
                "### requirements.1: .2",
                "requirements.1"
            )
        ),
    );
}

/// A control: where the shown heading would not declare a recognized section, the guard of
/// §FS-declarations.checks.unmarked-heading.2.3 leaves the message exactly as it is today.
#[test]
fn a_repair_that_would_not_declare_a_section_is_not_shown() {
    let findings = Tree::new("guard").findings();
    let unchanged = [
        // the corrected path is already a section of the declaration
        (11, "### requirements.2: requirements.1", "requirements.2"),
        // its parent `extra` is no section of the declaration
        (15, "### requirements.3: extra.1", "requirements.3"),
        // two components at one level deep
        (19, "## 1. requirements.page_size: Page size", "1"),
        // a lone component with nothing after it reads as a title
        (23, "## 2. open_questions", "2"),
        // `Notes-here` is still no section name
        (27, "## 3. Notes_here: Notes", "3"),
        // a lone number with nothing after it reads as a title
        (31, "## 4. 1", "4"),
    ];
    for (line, suggested, coordinate) in unchanged {
        assert_message(
            &findings,
            &format!("docs/fs/FS-014-guard.md:{line}"),
            &prefix("FS-014-guard", suggested, coordinate),
        );
    }
}
