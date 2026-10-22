//! The managed block's version heading labels its number as the block's revision,
//! `## Grounding with grund (block vN)`, so it no longer reads as a grund release
//! (§FS-init.2.3.9.3, §FS-init.2.3.7). Labelling it is fixed text, so both current
//! pairs move four: v31/v32 and v33/v34 (§FS-init.2.3.7.1). The reader keeps the
//! unlabelled spelling of v30 and earlier inside the delimiters, migrates it in
//! place (§FS-init.2.3.10.1) and refuses a newer version in either spelling
//! (§FS-init.2.3.10.2); the new spelling opens no legacy block (§FS-init.2.3.9.2).
//! The first test is the reproduction of #827 run as a regression.

use std::fs;
use std::path::{Path, PathBuf};

#[path = "support/init_fixture.rs"]
mod init_fixture;

use init_fixture::{manifest_dir, run_grund, workdir};

const BEGIN: &str = "<!-- BEGIN GRUND MANAGED BLOCK -->";
const END: &str = "<!-- END GRUND MANAGED BLOCK -->";

/// The heading `init` writes at version `n`.
fn heading(n: u32) -> String {
    format!("## Grounding with grund (block v{n})")
}

/// The repository #827 was reported from: an `FS` file and a non-citable `e2e` home.
const REPORTED: &str = r#"grund_config_version = 2
project_name = "demo"

[schema.kinds.FS]
files = ["requirements.md"]

[schema.kinds.e2e]
citable = false
folders = ["tests/e2e"]
"#;

/// The scoped config of `init_scoped_guidance.rs`: FS and a test row land in
/// `docs/spec/AGENTS.md` and `tests/AGENTS.md`.
const SCOPED: &str = r#"grund_config_version = 2
project_name = "scoped"
[schema.kinds.GOAL]
files = ["docs/goals.md"]
[schema.kinds.FS]
folders = ["docs/spec/core", "docs/spec/plugins"]
[schema.kinds.test]
citable = false
folders = ["tests/e2e", "tests/integration"]
[rules.citations.FS]
must = ["GOAL"]
[rules.citations.test]
must = ["FS"]
"#;

const RULE_KIND: &str =
    "[schema.kinds.RULE]\nfolders = [\"docs/rules\"]\nindex = false\nform = \"rule\"\n";

/// A declaration shape kept at the root renders the scoped pair without a scoped file.
const ROOT_SHAPE: &str = r#"grund_config_version = 2
project_name = "shape"
[schema.kinds.DF]
folders = ["docs/df"]
index = false
[schema.kinds.DF.fields.status]
one_of = ["Accepted", "Superseded"]
[presentation]
rules = "root"
"#;

fn tree(name: &str, config: &str, dirs: &[&str]) -> PathBuf {
    let target = workdir(name);
    for dir in dirs {
        fs::create_dir_all(target.join(dir)).expect("create place");
    }
    fs::write(target.join("grund.toml"), config).expect("write grund.toml");
    target
}

fn reported_tree(name: &str) -> PathBuf {
    let target = tree(name, REPORTED, &["tests/e2e"]);
    fs::write(
        target.join("requirements.md"),
        "# Requirements\n\n## FS-login: Login\n\nA user logs in.\n",
    )
    .expect("write requirements.md");
    target
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn init(target: &Path, extra: &[&str]) -> (Option<i32>, String) {
    let mut args = vec!["init", target.to_str().unwrap()];
    args.extend_from_slice(extra);
    let output = run_grund(&args, manifest_dir());
    (output.status.code(), text(&output.stderr))
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}

/// Every version heading in `file`, in order, whatever its spelling.
fn version_headings(file: &str) -> Vec<&str> {
    file.lines()
        .filter(|line| line.starts_with("## Grounding with grund ("))
        .collect()
}

/// `file` holds exactly one block, opened by `BEGIN` and the labelled heading at `n`.
fn assert_labelled(file: &str, n: u32, what: &str) {
    let opening = format!("{BEGIN}\n{}\n", heading(n));
    assert!(
        file.contains(&opening),
        "{what}: the block must open with `{}`, got headings {:?}",
        heading(n),
        version_headings(file)
    );
    assert_eq!(
        version_headings(file).len(),
        1,
        "{what}: one block:\n{file}"
    );
}

/// #827, reproduced: a fresh `init` labels the heading as the block's revision,
/// `grund -V` reports the CLI release on its own, and the refusal the agent misread
/// is byte for byte what it was — the label is the whole fix.
#[test]
fn reported_repository_gets_a_labelled_heading_beside_an_unchanged_refusal() {
    let target = reported_tree("block_label_reported_repository");
    fs::write(target.join("AGENTS.md"), "# demo — agent instructions\n").expect("AGENTS.md");
    fs::write(target.join("GEMINI.md"), "# Gemini notes\n").expect("standalone companion");
    let (code, stderr) = init(&target, &[]);
    assert_eq!(code, Some(0), "init failed: {stderr}");

    let agents = read(&target.join("AGENTS.md"));
    assert_labelled(&agents, 39, "AGENTS.md");
    let gemini = read(&target.join("GEMINI.md"));
    assert!(gemini.starts_with("# Gemini notes\n"), "{gemini}");
    assert_labelled(&gemini, 39, "the standalone companion GEMINI.md");

    let version = run_grund(&["-V"], &target);
    assert_eq!(
        text(&version.stdout),
        format!("grund {}\n", env!("CARGO_PKG_VERSION")),
        "`grund -V` is where the release is read"
    );

    let refused = run_grund(
        &[
            "check",
            "--rule",
            "Each e2e must cite at least one FS.",
            "--only-rule",
        ],
        &target,
    );
    assert_eq!(refused.status.code(), Some(2));
    assert_eq!(text(&refused.stdout), "");
    assert_eq!(
        text(&refused.stderr),
        "error: kind \"e2e\" declares no IDs — tests/e2e/ is not a citable home; for \
         configured place \"e2e\", express this obligation in grund.toml: \
         [rules.citations.e2e] with must = [\"FS\"]\n\
         known kinds: FS\n",
        "the refusal's bytes are §FS-errors.3.8.1's since the one-grammar window closed"
    );
}

/// What `init` writes, `check` accepts and `init --check` finds current, for each of
/// the four pairs a config selects; every scoped file carries the root's heading.
fn assert_selection(name: &str, target: &Path, n: u32, scoped: &[&str]) {
    let (code, stderr) = init(target, &[]);
    assert_eq!(code, Some(0), "{name}: init failed: {stderr}");
    assert_labelled(&read(&target.join("AGENTS.md")), n, name);
    for file in scoped {
        let scoped_file = read(&target.join(file));
        assert!(scoped_file.ends_with(&format!("{END}\n")), "{name}: {file}");
        assert_labelled(&scoped_file, n, &format!("{name}: {file}"));
    }
    let check = run_grund(&["check", "."], target);
    let stdout = text(&check.stdout);
    assert!(
        !stdout.contains("agents-init") && !stdout.contains("AGENTS.md"),
        "{name}: check must accept the block init wrote:\n{stdout}{}",
        text(&check.stderr)
    );
    let (code, stderr) = init(target, &["--check"]);
    assert_eq!(code, Some(0), "{name}: init --check: {stderr}");
}

#[test]
fn each_selected_pair_is_labelled_and_four_higher() {
    let base = reported_tree("block_label_base");
    assert_selection("base", &base, 39, &[]);

    let rules = reported_tree("block_label_rules");
    fs::write(rules.join("grund.toml"), format!("{REPORTED}{RULE_KIND}")).expect("rule kind");
    fs::create_dir_all(rules.join("docs/rules")).expect("docs/rules");
    assert_selection("rule kind", &rules, 40, &[]);

    let shape = tree("block_label_root_shape", ROOT_SHAPE, &["docs/df"]);
    assert_selection("root-only shape", &shape, 41, &[]);

    let dirs = [
        "docs/spec/core",
        "docs/spec/plugins",
        "tests/e2e",
        "tests/integration",
        "docs/rules",
    ];
    let scoped = tree(
        "block_label_scoped_rules",
        &format!("{SCOPED}{RULE_KIND}"),
        &dirs,
    );
    assert_selection(
        "scoped with a rule kind",
        &scoped,
        42,
        &["docs/spec/AGENTS.md", "tests/AGENTS.md"],
    );
}

/// A block of the last unlabelled revision, with unmanaged text on both sides.
fn unlabelled_block_tree(name: &str) -> (PathBuf, String, String) {
    let target = reported_tree(name);
    let (code, stderr) = init(&target, &[]);
    assert_eq!(code, Some(0), "init failed: {stderr}");
    let current = read(&target.join("AGENTS.md"));
    let start = current.find(BEGIN).expect("a delimited block");
    let block = current[start..]
        .lines()
        .skip(2)
        .map(|line| format!("{line}\n"))
        .collect::<String>();
    let (before, after) = (
        "# Title\n\nmine before\n\n",
        "\n## Local notes\n\nmine after\n",
    );
    let old = format!("{before}{BEGIN}\n## Grounding with grund (v27)\n{block}{after}");
    fs::write(target.join("AGENTS.md"), &old).expect("write the v27 block");
    (target, before.to_string(), after.to_string())
}

/// An unlabelled block is read, reported outdated, relabelled in place, and current after.
#[test]
fn an_unlabelled_block_is_relabelled_in_place_once() {
    let (target, before, after) = unlabelled_block_tree("block_label_migrates_in_place");
    let check = run_grund(&["check", "."], &target);
    assert!(
        text(&check.stdout).contains("AGENTS.md"),
        "check reports the v27 block outdated, not current:\n{}",
        text(&check.stdout)
    );
    let (code, stderr) = init(&target, &[]);
    assert_eq!(code, Some(0), "{stderr}");
    assert!(stderr.contains("updated AGENTS.md"), "{stderr}");
    let migrated = read(&target.join("AGENTS.md"));
    assert!(
        migrated.starts_with(&format!("{before}{BEGIN}\n")),
        "{migrated}"
    );
    assert!(migrated.ends_with(&format!("{END}\n{after}")), "{migrated}");
    assert_labelled(&migrated, 39, "the migrated block");

    let (code, stderr) = init(&target, &[]);
    assert_eq!(code, Some(0), "{stderr}");
    assert!(stderr.contains("exists AGENTS.md"), "{stderr}");
    assert_eq!(read(&target.join("AGENTS.md")), migrated, "idempotent");
}

/// A labelled block is a block: the current one is `exists`, a hand-edited older one
/// is outdated and re-rendered, never malformed.
#[test]
fn a_labelled_block_inside_the_delimiters_is_read() {
    let target = reported_tree("block_label_is_read");
    let file = format!("# Title\n\n{BEGIN}\n## Grounding with grund (block v30)\n\nold\n{END}\n");
    fs::write(target.join("AGENTS.md"), &file).expect("write a labelled v30 block");
    let (code, stderr) = init(&target, &[]);
    assert_eq!(code, Some(0), "a labelled block is not malformed: {stderr}");
    assert!(stderr.contains("updated AGENTS.md"), "{stderr}");
    let agents = read(&target.join("AGENTS.md"));
    assert!(agents.starts_with("# Title\n\n"), "{agents}");
    assert_labelled(&agents, 39, "the re-rendered block");
}

/// A newer revision is refused in the new spelling as in the old, with nothing written.
#[test]
fn a_newer_labelled_block_is_refused_without_writes() {
    let target = reported_tree("block_label_newer_refused");
    let file = format!("# Title\n\n{BEGIN}\n## Grounding with grund (block v99)\n\nnew\n{END}\n");
    fs::write(target.join("AGENTS.md"), &file).expect("write a v99 block");
    fs::write(target.join("GEMINI.md"), "# Gemini notes\n").expect("standalone companion");
    let (code, stderr) = init(&target, &[]);
    assert_eq!(code, Some(2), "{stderr}");
    assert!(
        stderr.contains("contains newer grund init block v99; this binary supports v42"),
        "the newer-version refusal, not a malformed block: {stderr}"
    );
    assert_eq!(read(&target.join("AGENTS.md")), file);
    assert_eq!(read(&target.join("GEMINI.md")), "# Gemini notes\n");
}

/// Outside the delimiters the labelled heading opens no legacy block (§FS-init.2.3.9.2):
/// the file has no block, so `init` appends one and keeps the text as it was.
#[test]
fn a_labelled_heading_without_delimiters_opens_no_legacy_block() {
    let target = reported_tree("block_label_no_legacy");
    let file = "# Title\n\n## Grounding with grund (block v3)\n\nmine\n";
    fs::write(target.join("AGENTS.md"), file).expect("write an undelimited heading");
    let (code, stderr) = init(&target, &[]);
    assert_eq!(code, Some(0), "{stderr}");
    let agents = read(&target.join("AGENTS.md"));
    assert!(
        agents.starts_with(file),
        "the user's text is kept whole:\n{agents}"
    );
    assert!(
        agents[file.len()..].contains(&format!("{BEGIN}\n{}\n", heading(39))),
        "a block is appended after it:\n{agents}"
    );
}
