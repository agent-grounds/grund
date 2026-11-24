//! Issue #1487: the opener rule of §FS-citations.zones.link-destination, shared
//! by fmt, scanner exemptions and editor links. The first two tests port the
//! triage fixture and its byte/proposal assertions (§FS-fmt.6.4, §FS-fmt.6.3).

use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[path = "../../crates/grund-cli/tests/support/fixture_cleanup.rs"]
mod fixture_cleanup;

const CONFIG: &str = r#"grund_config_version = 2
[schema]
marker = "§"
id_format = "{kind}-{slug}"
slug_pattern = "[a-z][a-z0-9-]*"
[schema.kinds.FS]
folders = ["docs"]
"#;
const INPUT: &str = "# FS-b: Beta\n\n\
Line one with `x` code then \u{a7}FS-a here.\n\n\
Line two with `](` code then \u{a7}FS-a here and \u{a7}FS-a.terms there.\n\n\
Line three with `(` code then \u{a7}FS-a here.\n\n\
Line four with `]` code then \u{a7}FS-a here.\n\n\
Protected `]( \u{a7}FS-a and \u{a7}FS-a.terms` code only.\n\n\
Closed [authored](keep/\u{a7}FS-a/\u{a7}FS-a.terms) then \u{a7}FS-a here and \u{a7}FS-a.terms there.\n\n\
Unclosed [authored](keep/\u{a7}FS-a then \u{a7}FS-a.terms there.\n";
const EXPECTED: &str = "# FS-b: Beta\n\n\
Line one with `x` code then [\u{a7}FS-a](FS-a.md#fs-a-alpha) here.\n\n\
Line two with `](` code then [\u{a7}FS-a](FS-a.md#fs-a-alpha) here and [\u{a7}FS-a.terms](FS-a.md#terms-terms) there.\n\n\
Line three with `(` code then [\u{a7}FS-a](FS-a.md#fs-a-alpha) here.\n\n\
Line four with `]` code then [\u{a7}FS-a](FS-a.md#fs-a-alpha) here.\n\n\
Protected `]( \u{a7}FS-a and \u{a7}FS-a.terms` code only.\n\n\
Closed [authored](keep/\u{a7}FS-a/\u{a7}FS-a.terms) then [\u{a7}FS-a](FS-a.md#fs-a-alpha) here and [\u{a7}FS-a.terms](FS-a.md#terms-terms) there.\n\n\
Unclosed [authored](keep/\u{a7}FS-a then \u{a7}FS-a.terms there.\n";

struct Project(PathBuf);

impl Project {
    fn new(test: &str, config: &str, input: &str) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1487-{test}-{}", std::process::id()));
        fs::create_dir(&root).expect("exclusively create fixture");
        let project = Self(root);
        fs::create_dir(project.0.join("docs")).expect("create docs");
        project.write("grund.toml", config);
        project.write("docs/FS-a.md", "# FS-a: Alpha\n\n## terms: Terms\n");
        project.write("docs/FS-b.md", input);
        project
    }

    fn write(&self, file: &str, text: &str) {
        fs::write(self.0.join(file), text).expect("write fixture");
    }

    fn run(&self, args: &[&str], code: i32) -> String {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .env("NO_COLOR", "1")
            .current_dir(&self.0)
            .output()
            .expect("run grund");
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.status.code(), Some(code), "{args:?}: {text}");
        text
    }

    fn written(&self) -> String {
        fs::read_to_string(self.0.join("docs/FS-b.md")).expect("read written document")
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        fixture_cleanup::remove_fixture_tree(&self.0, "remove issue-1487 fixture");
    }
}

#[test]
fn issue_1487_check_proposes_both_citations_after_code() {
    let project = Project::new("proposals", CONFIG, INPUT);
    let report = project.run(&["fmt", "--cross-refs", "--check"], 1);
    for line in [3, 7, 9, 13] {
        assert!(report.contains(&format!("docs/FS-b.md:{line}: markdown link:")));
    }
    let expected = "docs/FS-b.md:5: markdown link: \u{a7}FS-a → [\u{a7}FS-a](FS-a.md#fs-a-alpha), \u{a7}FS-a.terms → [\u{a7}FS-a.terms](FS-a.md#terms-terms)";
    assert!(
        report.contains(expected),
        "missing both target proposals:\n{report}"
    );
    assert_eq!(project.written(), INPUT, "--check must not write");
}

#[test]
fn issue_1487_write_preserves_bytes_and_is_idempotent() {
    let project = Project::new("write", CONFIG, INPUT);
    project.run(&["fmt", "--cross-refs", "--write"], 0);
    assert_eq!(project.written(), EXPECTED);
    assert_eq!(project.run(&["fmt", "--cross-refs", "--check"], 0), "");
    project.run(&["fmt", "--cross-refs", "--write"], 0);
    assert_eq!(project.written(), EXPECTED, "second write changes no byte");
}

/// §REQ-no-missed-citation.2, §FS-citations.recognized: off strict mode only
/// real destinations suppress bare citations; a missed dangling one must fail.
#[test]
fn issue_1487_scanner_finds_bare_dangling_prose() {
    let config = "grund_config_version = 1\n[reference]\nstrict = false\n\
        [id]\nformat = \"{kind}-{slug}\"\nslug_pattern = \"[a-z][a-z0-9-]*\"\n\
        [[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n";
    let input = "# FS-b: Beta\n\nClosed [link](FS-protected).\n\n\
        Unclosed [link](FS-protected-too\n\nAfter `]( FS-code-missing` then FS-missing here.\n";
    let project = Project::new("bare", config, input);
    let report = project.run(&["check", "--only", "dangling"], 1);
    assert!(
        report.contains("docs/FS-b.md:7:")
            && report.contains("FS-missing")
            && report.contains("FS-code-missing"),
        "{report}"
    );
    assert!(
        !report.contains("FS-protected"),
        "real destinations are still skipped: {report}"
    );
}

/// §FS-check.3.13.1: scanner rewrite eligibility must not extend the code zone
/// into prose. The same exemption controls fmt's shorthand expansion.
#[test]
fn issue_1487_shorthand_in_prose_is_actionable() {
    let config = CONFIG.replace("{kind}-{slug}", "{kind}-{number}-{slug}");
    let input = "# FS-002-beta: Beta\n\nAfter `](` then \u{a7}FS-001 here.\n";
    let project = Project::new("shorthand", &config, input);
    project.write("docs/FS-a.md", "# FS-001-alpha: Alpha\n");
    let report = project.run(&["check", "--only", "shorthand-citation"], 1);
    assert!(
        report.contains("docs/FS-b.md:3:") && report.contains("FS-001-alpha"),
        "{report}"
    );
    project.run(&["fmt", "--cross-refs", "--write"], 0);
    assert_eq!(
        project.written(),
        input.replace(
            "\u{a7}FS-001 here",
            "[\u{a7}FS-001-alpha](FS-a.md#fs-001-alpha-alpha) here"
        )
    );
}

/// §FS-check.3.17.4: a bare index entry after code uses the prose reading.
#[test]
fn issue_1487_index_entry_after_code_requires_a_link() {
    let config = format!("{CONFIG}\nindex = \"README.md\"\n");
    let project = Project::new("index", &config, "# FS-b: Beta\n");
    project.write("docs/README.md", "After `](` then \u{a7}FS-a.\n");
    let report = project.run(&["check", "--only", "unlinked-index-entry"], 1);
    assert!(
        report.contains("docs/README.md:1:") && report.contains("FS-a"),
        "{report}"
    );
    project.run(&["fmt", "--cross-refs", "--write"], 0);
    project.run(&["check", "--only", "unlinked-index-entry"], 0);
}

/// §FS-lsp.1.2.1: hover uses the same destination boundary as the formatter.
#[test]
fn issue_1487_hover_links_prose_after_code() {
    let input = "Code `]( @FS-a` then @FS-a. Closed [x](keep/@FS-a).";
    let targets = vec![("@FS-a".into(), "file:///alpha.md".into())];
    assert_eq!(
        grund_core::link_hover_citations(input, &targets),
        "Code `]( @FS-a` then [@FS-a](file:///alpha.md). Closed [x](keep/@FS-a)."
    );
}

/// §FS-lsp.1.4: the on-type trigger reads the same prose as fmt's trigger pass.
#[test]
fn issue_1487_trigger_in_editor_and_formatter_is_prose() {
    let input = "# FS-b: Beta\n\nAfter `](` then $$FS-a";
    let project = Project::new("trigger", CONFIG, input);
    let edits = grund_core::on_type_line_edits(
        &project.0.join("docs/FS-b.md"),
        input,
        2,
        input.lines().nth(2).unwrap().len(),
        &[],
    )
    .expect("on-type edits");
    assert_eq!(edits.len(), 1, "the trigger after code needs an edit");
    project.run(&["fmt", "--cross-refs", "--write"], 0);
    assert_eq!(
        project.written(),
        input.replace("$$FS-a", "[\u{a7}FS-a](FS-a.md#fs-a-alpha)")
    );
}

/// §FS-fmt.2.1, §FS-fmt.2.2: the trigger and marker passes must not inherit a
/// destination invented inside code. Code and genuine destination bytes stay put.
#[test]
fn issue_1487_formatter_trigger_and_marker_follow_code() {
    let input = "# FS-b: Beta\n\nCode `]( $$FS-a FS-a` then $$FS-a and FS-a.\n\n\
        Closed [x](keep/$$FS-a/FS-a) then FS-a.\n\nUnclosed [x](keep/$$FS-a/FS-a\n";
    let project = Project::new("passes", CONFIG, input);
    project.run(&["fmt", "--marker", "--cross-refs", "--write"], 0);
    assert_eq!(
        project.written(),
        "# FS-b: Beta\n\n\
        Code `]( $$FS-a FS-a` then [\u{a7}FS-a](FS-a.md#fs-a-alpha) and [\u{a7}FS-a](FS-a.md#fs-a-alpha).\n\n\
        Closed [x](keep/$$FS-a/FS-a) then [\u{a7}FS-a](FS-a.md#fs-a-alpha).\n\nUnclosed [x](keep/$$FS-a/FS-a\n"
    );
}

/// §FS-check.3.18.3: a canonical wrap after code enrolls an external source
/// declaration. Its unused finding must explain that an index is not a use.
#[test]
fn issue_1487_external_index_enrollment_after_code() {
    let config = "grund_config_version = 1\n[[kinds]]\nkind = \"AR\"\nfolder = \"docs\"\n\
        [scan]\ninclude = [\"docs\", \"src\"]\n";
    let project = Project::new("enrollment", config, "");
    fs::create_dir(project.0.join("src")).unwrap();
    project.write("src/bus.rs", "/// AR-001-bus: The bus\nfn bus() {}\n");
    project.write(
        "docs/README.md",
        "After `](` then [\u{a7}AR-001-bus](../src/bus.rs)\n",
    );
    let report = project.run(&["check", "--only", "unused"], 0);
    assert!(
        report.contains("its index entry at docs/README.md:1 does not count as a use"),
        "{report}"
    );
}
