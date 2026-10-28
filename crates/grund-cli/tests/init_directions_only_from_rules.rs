//! Binary regressions for agent-grounds/grund#1075: the `### Citation directions`
//! section states the rules a config declares and no others (§FS-init.2.3.4.10).
//! A config that declares no citation rule renders no section, `kinds = []`
//! included (§FS-init.2.3.4.4.2). Where grounding lands, the section is the
//! heading and the grounding sentence alone (§FS-init.2.3.5.7), at the root and in
//! a scoped block (§FS-init.2.3.5.12.1). Every shape is one tree that `init`,
//! `init --check` and `check` agree on (§FS-init.2.3.5.8).
//!
//! These are binary tests rather than `tests/e2e/cases`, because a red case there
//! cannot be ignored on its own. Each fixture is a fresh git repository under
//! `target/init-tests/`, laid as the issue's reproducer lays it.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[path = "support/init_fixture.rs"]
mod init_fixture;

use init_fixture::{run_grund, workdir};

const V1_PLAIN: &str = "grund_config_version = 1\nproject_name = \"Repro\"\n\n[[kinds]]\n\
                        kind = \"GRUND\"\nfile = \"docs/grund.md\"\ntitle = \"Why this project exists\"\n";

const V1_GROUNDING: &str = "grund_config_version = 1\nproject_name = \"Repro\"\n\n[reference]\n\
                            require_grounding = true\n\n[[kinds]]\nkind = \"GRUND\"\n\
                            file = \"docs/grund.md\"\ntitle = \"Why this project exists\"\n";

const V1_NO_KINDS: &str = "grund_config_version = 1\nproject_name = \"Repro\"\nkinds = []\n";

const V2_PLAIN: &str = "grund_config_version = 2\nproject_name = \"Repro\"\n\n\
                        [schema.kinds.GRUND]\nfiles = [\"docs/grund.md\"]\n";

const V2_GROUNDING: &str = "grund_config_version = 2\nproject_name = \"Repro\"\n\n\
                            [schema.kinds.GRUND]\nfiles = [\"docs/grund.md\"]\n\n\
                            [rules.citations.grounding]\nmust = \"file\"\n";

const V2_NO_KINDS: &str = "grund_config_version = 2\nproject_name = \"Repro\"\n";

/// Grounding on one non-citable home alone, so every grounding clause lands in
/// its scoped block and none at the root.
const V2_SCOPED_GROUNDING: &str = "grund_config_version = 2\nproject_name = \"Repro\"\n\n\
     [schema.kinds.GRUND]\nfiles = [\"docs/grund.md\"]\n\n\
     [schema.kinds.e2e]\nfolders = [\"tests/e2e\"]\ncitable = false\n\n\
     [rules.citations.e2e.grounding]\nmust = \"file\"\n";

/// Scoped only by a field: a scoped block with a declaration shape, no rule and
/// no grounding anywhere.
const V2_SCOPED_FIELD: &str = "grund_config_version = 2\nproject_name = \"Repro\"\n\n\
     [schema.kinds.FS]\nfolders = [\"docs/fs\"]\n\n\
     [schema.kinds.FS.fields.front-matter]\npresence = \"must\"\n";

const V1_RULE: &str = "grund_config_version = 1\nproject_name = \"Repro\"\n\n[[kinds]]\n\
                       kind = \"GRUND\"\nfile = \"docs/grund.md\"\ntitle = \"Why this project exists\"\n\n\
                       [citations.code]\nshould = [\"GRUND\"]\n";

const V2_RULE: &str = "grund_config_version = 2\nproject_name = \"Repro\"\n\n\
                       [schema.kinds.GRUND]\nfiles = [\"docs/grund.md\"]\n\n\
                       [rules.citations.code]\nshould = [\"GRUND\"]\n";

/// The sentence a config without rules rendered before #1075 (§DF-directions-only-from-rules).
const STATIC_SENTENCE: &str = "Specs cite goals, architecture cites specs, code and executable \
    tests cite the specs they realize. In a citation rule array, entries are all required; `|` \
    inside one entry means any one alternative. See \
    https://github.com/agent-grounds/grund/blob/main/docs/user-facing/citation-directions.md for \
    the levels and examples.";

const BEGIN: &str = "<!-- BEGIN GRUND MANAGED BLOCK -->";
const END: &str = "<!-- END GRUND MANAGED BLOCK -->";

/// A fresh git repository with `config` and the one declaration its kind holds,
/// plus each of `dirs`, so a scoped destination exists (§FS-init.2.3.5.11.4).
fn tree(name: &str, config: &str, dirs: &[&str]) -> PathBuf {
    let target = workdir(&format!("directions_only_from_rules_{name}"));
    let git = Command::new("git")
        .args(["init", "-q"])
        .current_dir(&target)
        .output()
        .expect("git init");
    assert!(git.status.success(), "git init: {git:?}");
    let id = if config.starts_with("grund_config_version = 1") {
        "GRUND-1-why"
    } else {
        "GRUND-why"
    };
    fs::create_dir_all(target.join("docs")).unwrap();
    fs::write(
        target.join("docs/grund.md"),
        format!("# {id}: Why\n\nWhy.\n"),
    )
    .unwrap();
    fs::write(target.join("grund.toml"), config).unwrap();
    for dir in dirs {
        fs::create_dir_all(target.join(dir)).unwrap();
    }
    target
}

fn grund(target: &Path, args: &[&str]) -> (Option<i32>, String, String) {
    let out = run_grund(args, target);
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn init(target: &Path) {
    let (code, stdout, stderr) = grund(target, &["init"]);
    assert_eq!(code, Some(0), "grund init: {stdout}{stderr}");
}

/// The managed block of `file`, delimiters included.
fn block(target: &Path, file: &str) -> String {
    let text = fs::read_to_string(target.join(file)).expect("read entrypoint");
    let start = text.find(BEGIN).expect("a managed block");
    let end = text.find(END).expect("the block's end") + END.len();
    text[start..end].to_string()
}

/// The block's `### Citation directions` section up to the next heading or the
/// end delimiter, without its trailing newlines; `None` when it has none.
fn section(target: &Path, file: &str) -> Option<String> {
    let block = block(target, file);
    let start = block.find("### Citation directions")?;
    let rest = &block[start..];
    let end = rest[1..]
        .find("\n### ")
        .map(|at| at + 1)
        .unwrap_or_else(|| rest.find(END).unwrap());
    Some(rest[..end].trim_end_matches('\n').to_string())
}

/// One tree the commands agree on (§FS-init.2.3.5.8): no doubled blank line in a
/// block, a second `init` finds every file current, and `init --check` and
/// `check` exit 0.
fn assert_settled(target: &Path, files: &[&str]) {
    for file in files {
        let block = block(target, file);
        assert!(
            !block.contains("\n\n\n"),
            "{file}: a doubled blank line:\n{block}"
        );
    }
    let (code, _, stderr) = grund(target, &["init"]);
    assert_eq!(code, Some(0), "second grund init: {stderr}");
    let report: Vec<&str> = stderr.lines().take_while(|line| !line.is_empty()).collect();
    assert!(
        !report.is_empty() && report.iter().all(|line| line.starts_with("exists ")),
        "a second grund init changed something:\n{stderr}"
    );
    let (code, _, stderr) = grund(target, &["init", "--check"]);
    assert_eq!(code, Some(0), "grund init --check: {stderr}");
    let (code, stdout, stderr) = grund(target, &["check"]);
    assert_eq!(code, Some(0), "grund check: {stdout}{stderr}");
}

/// §FS-init.2.3.4.10: the issue's config, and its version-2 spelling, declare
/// kinds and no rule: no section, and one blank line where it stood.
#[test]
#[ignore = "red until #1075 renders Citation directions only from declared rules; implement removes this"]
fn kinds_and_no_rules_render_no_section() {
    for (name, config) in [("v1_plain", V1_PLAIN), ("v2_plain", V2_PLAIN)] {
        let target = tree(name, config, &[]);
        init(&target);
        assert_eq!(section(&target, "AGENTS.md"), None, "{name}");
        let block = block(&target, "AGENTS.md");
        assert!(
            block.contains(
                "- **Always cite the most-specific point.**\n\n### Clickable citations\n"
            ),
            "{name}: Rules runs into Clickable citations:\n{block}"
        );
        assert_settled(&target, &["AGENTS.md"]);
    }
}

/// §FS-init.2.3.4.4.2: `kinds = []` declares no rule either, so its "No kind is
/// declared yet" paragraph goes with the section.
#[test]
#[ignore = "red until #1075 renders Citation directions only from declared rules; implement removes this"]
fn no_kinds_render_no_section() {
    for (name, config) in [("v1_no_kinds", V1_NO_KINDS), ("v2_no_kinds", V2_NO_KINDS)] {
        let target = tree(name, config, &[]);
        init(&target);
        assert_eq!(section(&target, "AGENTS.md"), None, "{name}");
        assert_settled(&target, &["AGENTS.md"]);
    }
}

/// §FS-init.2.3.4.10, §FS-init.2.3.5.7: grounding without rules is the heading
/// and the grounding sentence alone, in either config version.
#[test]
#[ignore = "red until #1075 renders Citation directions only from declared rules; implement removes this"]
fn grounding_without_rules_is_the_grounding_sentence_alone() {
    for (name, config) in [
        ("v1_grounding", V1_GROUNDING),
        ("v2_grounding", V2_GROUNDING),
    ] {
        let target = tree(name, config, &[]);
        init(&target);
        assert_eq!(
            section(&target, "AGENTS.md").as_deref(),
            Some(
                "### Citation directions\n\n\
                 Every source file must cite a declared ID or declare one inline."
            ),
            "{name}"
        );
        assert_settled(&target, &["AGENTS.md"]);
    }
}

/// §FS-init.2.3.5.12.1: grounding on one scoped home alone leaves the root
/// without a section and without a doubled blank line where the swap took it out,
/// and the scoped block holds its grounding sentence with no legend and no
/// closing line.
#[test]
#[ignore = "red until #1075 renders Citation directions only from declared rules; implement removes this"]
fn grounding_only_in_a_scoped_home_leaves_the_root_without_a_section() {
    let target = tree("v2_scoped_grounding", V2_SCOPED_GROUNDING, &["tests/e2e"]);
    init(&target);
    assert_eq!(section(&target, "AGENTS.md"), None);
    let root = block(&target, "AGENTS.md");
    assert!(
        root.contains("- **Always cite the most-specific point.**\n\n### Scoped guidance\n"),
        "{root}"
    );
    assert!(
        root.contains("- `tests/e2e/AGENTS.md`: tests/e2e/\n"),
        "{root}"
    );
    assert_eq!(
        section(&target, "tests/e2e/AGENTS.md").as_deref(),
        Some("### Citation directions\n\nEvery file under tests/e2e/ must cite one.")
    );
    assert_settled(&target, &["AGENTS.md", "tests/e2e/AGENTS.md"]);
}

/// §FS-init.2.3.4.10: a config scoped only by a field renders no section in
/// either block, and the root block is whole from its first byte.
#[test]
#[ignore = "red until #1075 renders Citation directions only from declared rules; implement removes this"]
fn a_config_scoped_only_by_a_field_renders_no_section_anywhere() {
    let target = tree("v2_scoped_field", V2_SCOPED_FIELD, &["docs/fs"]);
    init(&target);
    assert_eq!(section(&target, "AGENTS.md"), None);
    assert_eq!(section(&target, "docs/fs/AGENTS.md"), None);
    let agents = fs::read_to_string(target.join("AGENTS.md")).unwrap();
    assert!(
        agents.starts_with(&format!(
            "# Repro — agent instructions\n\n{BEGIN}\n## Grounding with grund (block v"
        )),
        "{agents}"
    );
    assert!(block(&target, "docs/fs/AGENTS.md").contains("### Declaration shapes\n"));
    assert_settled(&target, &["AGENTS.md", "docs/fs/AGENTS.md"]);
}

/// §FS-init.2.3.5.8: a block that still carries the static sentence differs from
/// the render, so `check` names its citation directions section and `init
/// --check` would rewrite it.
#[test]
#[ignore = "red until #1075 renders Citation directions only from declared rules; implement removes this"]
fn a_block_still_carrying_the_static_sentence_is_stale() {
    for (name, config) in [("v1_stale", V1_PLAIN), ("v2_stale", V2_PLAIN)] {
        let target = tree(name, config, &[]);
        init(&target);
        let path = target.join("AGENTS.md");
        let text = fs::read_to_string(&path).unwrap();
        let clickable = text.find("### Clickable citations").unwrap();
        let start = text.find("### Citation directions").unwrap_or(clickable);
        let old = format!(
            "{}### Citation directions\n\n{STATIC_SENTENCE}\n\n{}",
            &text[..start],
            &text[clickable..]
        );
        fs::write(&path, old).unwrap();
        let (_, stdout, _) = grund(&target, &["check", "--format", "json"]);
        assert!(
            stdout
                .lines()
                .any(|line| line.contains("\"code\":\"agents-init\"")
                    && line.contains("\"path\":\"AGENTS.md\"")
                    && line.contains("the citation directions section differs")),
            "{name}: no stale citation directions finding:\n{stdout}"
        );
        let (code, _, stderr) = grund(&target, &["init", "--check"]);
        assert_eq!(code, Some(1), "{name}: grund init --check: {stderr}");
    }
}

/// §DF-directions-only-from-rules: a config that declares a rule renders the
/// section §FS-init.2.3.5.1 lays out, as before. Passes today and must keep
/// passing.
#[test]
fn a_config_with_a_rule_keeps_the_legend_and_the_closing_line() {
    for (name, config) in [("v1_rule", V1_RULE), ("v2_rule", V2_RULE)] {
        let target = tree(name, config, &[]);
        init(&target);
        assert_eq!(
            section(&target, "AGENTS.md").as_deref(),
            Some(
                "### Citation directions\n\n\
                 `must`/`never` are `grund check` errors; `should`/`avoid` are suggestions \
                 (`grund check --suggestions`).\n\n\
                 - Each source file outside the Project map (**code**) that cites anything \
                 should cite GRUND.\nAnything not listed above is allowed."
            ),
            "{name}"
        );
        assert_settled(&target, &["AGENTS.md"]);
    }
}
