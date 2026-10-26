//! Black-box contract for opt-in named section coordinates. The fixture is the
//! validated issue-178 reproducer expanded across every CLI consumer
//! (§FS-config.3.2.7, §FS-config.3.3).

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_ROOT: AtomicUsize = AtomicUsize::new(0);

struct Repo(PathBuf);

impl Repo {
    fn new(name: &str) -> Self {
        let serial = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/named-sections-tests")
            .join(format!("{name}-{}-{serial}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("docs")).expect("create named-section fixture");
        Self(root)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn write(&self, relative: &str, body: &str) {
        let path = self.0.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create fixture parent");
        }
        fs::write(path, body).expect("write fixture file");
    }

    fn enabled(&self, strict: bool) {
        self.write(
            "grund.toml",
            &format!(
                "grund_config_version = 1\n\n[reference]\nstrict = {strict}\n\n\
                 [id]\nformat = \"{{kind}}-{{slug}}\"\nnamed_sections = true\n\n\
                 [scan]\ninclude = [\"docs\"]\n"
            ),
        );
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn grund(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(root)
        .output()
        .expect("run grund")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn expect_success(output: &Output, context: &str) {
    assert!(
        output.status.success(),
        "{context}: expected success, got {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        stdout(output),
        stderr(output)
    );
}

fn doc() -> &'static str {
    "# FS-doc: Document\n\nDeclaration lead.\n\n\
     ## goals: Scope\n\nGoals lead.\n\n\
     ### goals.performance: Performance\n\nThis nested named section is the decisive target.\n\n\
     #### goals.performance.latency: Latency\n\nNested child.\n\n\
     ### goals.3: Ordered child\n\nThe legal numeric child.\n\n\
     ## 1. Numeric\n\nExisting numeric section.\n"
}

/// §FS-config.3.3.1: the fixture is the named grammar at every depth — the
/// explicit `<complete-path>: <title>` colon form, an all-name path
/// (`goals.performance.latency`), and a numeric component under a named prefix
/// (`goals.3`) — read back by `show`, `refs`, the JSON surface and completion.
/// §FS-config.4.2.3: an enabled repository's `config show` prints
/// `named_sections = true`.
#[test]
fn named_sections_drive_show_refs_json_and_completion() {
    let repo = Repo::new("readers");
    repo.enabled(true);
    repo.write("docs/FS-doc.md", doc());
    repo.write(
        "docs/FS-reader.md",
        "# FS-reader: Reader\n\nUses \u{a7}FS-doc.goals.performance and \u{a7}FS-doc.goals.3.\n",
    );

    let direct = grund(repo.path(), &["show", "FS-doc.goals.performance"]);
    expect_success(&direct, "nested name query");
    let direct_text = stdout(&direct);
    assert!(direct_text.contains("### goals.performance: Performance"));
    assert!(direct_text.contains("decisive target"));
    assert!(
        !direct_text.contains("Nested child"),
        "default cuts at child"
    );

    let declaration = grund(repo.path(), &["show", "FS-doc"]);
    expect_success(&declaration, "named heading cuts declaration lead");
    assert_eq!(stdout(&declaration), "Declaration lead.\n");

    for args in [
        vec!["show", "FS-doc.goals.performance", "--brief"],
        vec!["show", "FS-doc.goals.performance", "--toc"],
        vec!["show", "FS-doc.goals.performance", "--full"],
        vec!["show", "FS-doc", "--section", "goals.performance"],
        vec!["show", "FS-doc.goals.3"],
    ] {
        let output = grund(repo.path(), &args);
        expect_success(&output, &format!("named read mode: {args:?}"));
        if args.contains(&"--full") {
            assert!(stdout(&output).contains("Nested child"));
        }
    }

    let markdown = grund(
        repo.path(),
        &["show", "FS-doc.goals.performance", "--format", "md"],
    );
    expect_success(&markdown, "named Markdown query");
    assert!(
        stdout(&markdown).starts_with("# FS-doc: Document\n### goals.performance: Performance")
    );

    let toc = grund(repo.path(), &["show", "FS-doc", "--toc"]);
    expect_success(&toc, "whole named TOC");
    let toc = stdout(&toc);
    for heading in [
        "## goals: Scope",
        "### goals.performance: Performance",
        "#### goals.performance.latency: Latency",
        "### goals.3: Ordered child",
        "## 1. Numeric",
    ] {
        assert!(toc.contains(heading), "TOC omitted {heading:?}: {toc}");
    }

    let json = grund(
        repo.path(),
        &["show", "FS-doc.goals.performance", "--format", "json"],
    );
    expect_success(&json, "named JSON query");
    let json: Value = serde_json::from_slice(&json.stdout).expect("show JSON");
    assert_eq!(json["section"], "goals.performance");
    assert!(json["body"].as_str().unwrap().contains("decisive target"));
    let json_toc = grund(
        repo.path(),
        &["show", "FS-doc", "--toc", "--format", "json"],
    );
    expect_success(&json_toc, "named JSON TOC");
    let json_toc: Value = serde_json::from_slice(&json_toc.stdout).expect("TOC JSON");
    let paths = json_toc["sections"]
        .as_array()
        .expect("section array")
        .iter()
        .map(|section| section["path"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(paths.contains(&"goals.performance"));
    assert!(paths.contains(&"goals.3"));

    let refs = grund(
        repo.path(),
        &["refs", "FS-doc", "--section", "goals.performance"],
    );
    expect_success(&refs, "named refs filter");
    let refs = stdout(&refs);
    assert!(refs.contains("FS-reader.md"));
    assert!(refs.contains("\u{a7}FS-doc.goals.performance"));
    assert!(!refs.contains("\u{a7}FS-doc.goals.3"));

    let complete = grund(
        repo.path(),
        &["complete", "ids", "--prefix", "FS-doc.goals", "--sections"],
    );
    expect_success(&complete, "named completion");
    let complete = stdout(&complete);
    for candidate in [
        "FS-doc.goals",
        "FS-doc.goals.performance",
        "FS-doc.goals.performance.latency",
        "FS-doc.goals.3",
    ] {
        assert!(complete.lines().any(|line| line == candidate));
    }
    assert!(!complete.contains("FS-doc.1.goals"));

    let shown = grund(repo.path(), &["config", "show"]);
    expect_success(&shown, "enabled config inspection");
    assert!(stdout(&shown).contains("named_sections = true"));
}

/// §FS-config.3.3.3: `missing.performance` is an orphan because the prefix
/// `missing` is recorded nowhere in the declaration, however the Markdown
/// nests it. §FS-config.3.3.1: a mixed path — `FS-doc.1.goals` — is one
/// address and never an alias for section `1`, so where no `1.goals` heading
/// exists the query fails instead of truncating to the numeric section.
///
/// §FS-check.1.1.2 (§FS-citations.recognized.named-candidates): the fixture runs at `strict = false` and holds all three
/// candidate readings at once — the marker-prefixed `<§>FS-doc.absent` is a
/// citation even though the named section is missing, the unmarked
/// `FS-doc.absent` is one prose token suppressed whole rather than falling
/// back to a bare-ID citation, and the mixed `<§>FS-doc.1.goals` is never
/// truncated to its numeric prefix: it is a citation of that missing section,
/// whose finding is an ordinary error (§FS-citations.checks.mixed-path).
#[test]
fn named_sections_report_missing_reserved_orphan_depth_and_duplicates() {
    let repo = Repo::new("diagnostics");
    repo.enabled(false);
    repo.write(
        "docs/FS-doc.md",
        "# FS-doc: Document\n\n\
         ## goals: First\n\nFirst.\n\n\
         ## goals: Second\n\nSecond.\n\n\
         ## missing.performance: Wrong depth and orphan\n\nOrphan.\n\n\
         ## 1. Numeric\n\nNumeric.\n",
    );
    repo.write(
        "docs/FS-reader.md",
        "# FS-reader: Reader\n\n\
         Marked missing \u{a7}FS-doc.absent.\n\
         Unmarked FS-doc.absent must stay prose.\n\
         Mixed \u{a7}FS-doc.1.goals must not truncate.\n",
    );

    let check = grund(repo.path(), &["check"]);
    assert_eq!(check.status.code(), Some(1), "stderr: {}", stderr(&check));
    let findings = stdout(&check);
    assert!(findings.contains("section not found") && findings.contains("FS-doc.absent"));
    assert!(
        findings.contains("<§>"),
        "missing-name escape hint: {findings}"
    );
    assert!(findings.contains("duplicate section FS-doc.goals"));
    assert!(findings.contains("orphan") && findings.contains("missing.performance"));
    assert!(
        findings.contains("FS-doc.md:11"),
        "orphan heading line: {findings}"
    );
    assert!(findings.contains("heading level mismatch"));
    assert!(
        findings.contains(
            "FS-reader.md:5: error: section not found: FS-doc.1.goals; write <§> before it \
             to show the shape without citing it"
        ),
        "the missing mixed citation errors whole: {findings}"
    );

    let ambiguous = grund(repo.path(), &["show", "FS-doc.goals"]);
    assert_eq!(ambiguous.status.code(), Some(1));
    assert!(stderr(&ambiguous).contains("ambiguous section: FS-doc.goals"));

    let toc = grund(repo.path(), &["show", "FS-doc", "--toc"]);
    expect_success(&toc, "whole-declaration TOC despite duplicate");
    assert_eq!(stdout(&toc).matches("## goals:").count(), 2);

    let reserved = grund(repo.path(), &["show", "FS-doc.1.goals"]);
    assert!(!reserved.status.success());
    assert!(!stdout(&reserved).contains("## 1. Numeric"));

    let refs = grund(repo.path(), &["refs", "FS-doc"]);
    expect_success(&refs, "refs with suppressed prose tokens");
    let refs = stdout(&refs);
    assert!(refs.contains("\u{a7}FS-doc.absent"));
    assert!(!refs.contains("Unmarked FS-doc.absent"));
    assert!(refs.contains("\u{a7}FS-doc.1.goals"));
}

#[test]
fn named_sections_preserve_full_ids_before_shorthand_and_findings_compose() {
    let repo = Repo::new("shorthand");
    repo.write(
        "grund.toml",
        "grund_config_version = 1\n\n[reference]\nstrict = true\n\n\
         [id]\nformat = \"{kind}-{number}-{slug}\"\nnamed_sections = true\n\n\
         [scan]\ninclude = [\"docs\"]\n",
    );
    repo.write(
        "docs/FS-042-doc.md",
        "# FS-042-doc: Document\n\n## goals: Goals\n\nBody.\n",
    );
    repo.write(
        "docs/FS-043-reader.md",
        "# FS-043-reader: Reader\n\n\u{a7}FS-042.missing\n\u{a7}FS-042-doc.missing\n",
    );

    let check = grund(repo.path(), &["check"]);
    assert_eq!(check.status.code(), Some(1), "stderr: {}", stderr(&check));
    let findings = stdout(&check);
    assert!(findings.contains("FS-042-doc.missing"));
    assert!(
        findings.contains("shorthand"),
        "canonical-form finding: {findings}"
    );
    assert_eq!(findings.matches("section not found").count(), 2);
}

/// §FS-fmt.6.2.4, §FS-declarations.heading.rendered-text: the rendered text of
/// `## goals: Scope` carries the handle and colon, so the wrapper points at
/// `#goals-scope`; retitling the heading to `## goals: Intent` refreshes that
/// wrapper to `#goals-intent` on the next pass while the stored handle and the
/// citation text stay `goals`.
#[test]
fn named_section_formatting_uses_rendered_anchor_and_never_changes_handles() {
    let repo = Repo::new("formatting");
    repo.enabled(true);
    repo.write("docs/FS-doc.md", doc());
    repo.write(
        "docs/FS-reader.md",
        "# FS-reader: Reader\n\n\u{a7}FS-doc.goals and unmarked FS-doc.goals.\n",
    );

    let first = grund(repo.path(), &["fmt", "--cross-refs", "--write"]);
    expect_success(&first, "wrap named citation");
    let reader = fs::read_to_string(repo.path().join("docs/FS-reader.md")).unwrap();
    assert!(reader.contains("[\u{a7}FS-doc.goals](FS-doc.md#goals-scope)"));
    assert!(reader.contains("unmarked FS-doc.goals"));
    let named = fs::read_to_string(repo.path().join("docs/FS-doc.md")).unwrap();
    assert!(named.contains("## goals: Scope"));

    repo.write(
        "docs/FS-doc.md",
        &doc().replace("## goals: Scope", "## goals: Intent"),
    );
    let second = grund(repo.path(), &["fmt", "--cross-refs", "--write"]);
    expect_success(&second, "refresh named anchor after retitle");
    let reader = fs::read_to_string(repo.path().join("docs/FS-reader.md")).unwrap();
    assert!(reader.contains("[\u{a7}FS-doc.goals](FS-doc.md#goals-intent)"));
    assert!(reader.contains("\u{a7}FS-doc.goals"));
}

/// §FS-init.2.4.4: the v2 starter writes no section switch and its block
/// teaches named section paths. §FS-config.4.2.3: in v1, `named_sections`
/// prints only when enabled — an absent key and an explicit `false` produce
/// byte-identical `config show` output, so a repository that never opted in
/// keeps the bytes it had.
#[test]
fn init_writes_no_switch_and_the_disabled_gate_is_an_operational_guard() {
    let initialized = Repo::new("init");
    let init = grund(initialized.path(), &["init", "--no-vcs"]);
    expect_success(&init, "initialize the v2 starter");
    let generated = fs::read_to_string(initialized.path().join("grund.toml")).unwrap();
    assert!(!generated.contains("named_sections"), "{generated}");
    let default_guidance = fs::read_to_string(initialized.path().join("AGENTS.md")).unwrap();
    assert!(default_guidance.contains("goals.performance"));

    let absent = Repo::new("absent");
    absent.write(
        "grund.toml",
        "grund_config_version = 1\n[id]\nformat = \"{kind}-{slug}\"\n[scan]\ninclude = [\"docs\"]\n",
    );
    absent.write(
        "docs/FS-doc.md",
        "# FS-doc: Document\n\n## 1. Numeric\n\nBody.\n",
    );
    let explicit_false = Repo::new("false");
    explicit_false.write(
        "grund.toml",
        "grund_config_version = 1\n[id]\nformat = \"{kind}-{slug}\"\n\
         named_sections = false\n[scan]\ninclude = [\"docs\"]\n",
    );
    explicit_false.write(
        "docs/FS-doc.md",
        "# FS-doc: Document\n\n## 1. Numeric\n\nBody.\n",
    );
    for args in [
        vec!["check"],
        vec!["show", "FS-doc.1", "--full"],
        vec!["refs", "FS-doc", "--section", "1"],
        vec!["complete", "ids", "--sections"],
        vec!["fmt", "--check"],
        vec!["config", "show"],
    ] {
        let left = grund(absent.path(), &args);
        let right = grund(explicit_false.path(), &args);
        assert_eq!(
            left.stdout, right.stdout,
            "absent and false stdout differ for {args:?}"
        );
        assert_eq!(
            left.stderr, right.stderr,
            "absent and false stderr differ for {args:?}"
        );
        assert_eq!(
            left.status.code(),
            right.status.code(),
            "absent and false exit differ for {args:?}"
        );
    }

    let enabled = Repo::new("enabled-inspection");
    enabled.enabled(true);
    let validate = grund(enabled.path(), &["config", "validate"]);
    expect_success(&validate, "enabled config must be accepted");
    let init_enabled = grund(enabled.path(), &["init", "--no-vcs"]);
    expect_success(&init_enabled, "render enabled named-section guidance");
    let enabled_guidance = fs::read_to_string(enabled.path().join("AGENTS.md")).unwrap();
    assert!(enabled_guidance.contains("goals.performance"));
    assert!(enabled_guidance.contains("goals.3"));
    assert!(enabled_guidance.contains(
        "and `name.number` and `number.name` are both legal; \
         a cited address with a name under a number is recorded in the section lock."
    ));
    // §FS-init.2.3.4.5: from v23 the block teaches both forms for doc-comments too.
    assert!(enabled_guidance.contains(
        "In a declaration body, Markdown or doc-comment, a heading may instead write only \
         the component it adds"
    ));
    assert!(enabled_guidance.contains("## Grounding with grund (block v43)"));
    assert!(!enabled_guidance.contains("`number.name` is reserved"));
}

/// §FS-citations.recognized.named-candidates (§FS-config.3.3.1): with a
/// `### 1.goals: Goals` heading under `## 1. Overview`, a marked
/// `<§>FS-x.1.goals` is one citation of section `1.goals` — neither suppressed
/// nor truncated to `FS-x.1` — beside a plain citation of section `1`.
#[test]
fn a_marked_mixed_path_is_one_citation_of_the_whole_address() {
    let repo = Repo::new("mixed-citation");
    repo.enabled(true);
    repo.write(
        "docs/FS-x.md",
        "# FS-x: Example\n\n## 1. Overview\n\nOverview lead.\n\n\
         ### 1.goals: Goals\n\nGoals target.\n",
    );
    repo.write(
        "docs/FS-reader.md",
        "# FS-reader: Reader\n\nUses \u{a7}FS-x.1.goals here and \u{a7}FS-x.1 there.\n",
    );

    let refs = grund(repo.path(), &["refs", "FS-x", "--format", "json"]);
    expect_success(&refs, "refs of a mixed citation");
    let records = stdout(&refs)
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("refs NDJSON"))
        .map(|record| {
            let field = |key: &str| record[key].as_str().unwrap_or_default().to_string();
            (record["column"].as_i64(), field("section"), field("text"))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        records,
        vec![
            (
                Some(6),
                "1.goals".to_string(),
                "\u{a7}FS-x.1.goals".to_string()
            ),
            (Some(30), "1".to_string(), "\u{a7}FS-x.1".to_string()),
        ],
        "one citation per token, the mixed one whole"
    );
}
