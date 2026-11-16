//! §FS-citations.checks.mixed-path and §FS-rules.7.12: mixed paths use ordinary
//! findings; independent warning windows, suggestions and strengths survive.
//! The configured-prohibition and count fixtures port issue-710's triage probe.
//! A count's list names each citation as written (§FS-rules.7.3.1), so the
//! mixed and ordinary runs agree up to that list, and the mixed run's names
//! `GOAL-g.1.goals`.

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);
/// The fixtures stay version 1 for `[fmt.cross_refs] enabled = false`, which
/// has no version 2 spelling (§FS-check.checks.config-version-1).
const CONFIG_VERSION_1: &str = "config-version-1";
const MIXED: &str = "a citation of a name under a number is now checked";
/// Where a count row's list of the citations it counted begins (§FS-rules.7.3.1).
const COUNTED: &str = "; counted at ";
const CONFIG: &str = "grund_config_version = 1\n\
    [reference]\nstrict = true\nrequire_grounding = false\n\
    [id]\nformat = \"{kind}-{slug}\"\nnamed_sections = true\n\
    [[kinds]]\nkind = \"GOAL\"\nfolder = \"docs/goals\"\nindex = false\n\
    [[kinds]]\nkind = \"FS\"\nfolder = \"docs/fs\"\nindex = false\n\
    [fmt.cross_refs]\nenabled = false\n[scan]\ninclude = [\"docs\"]\n";

struct Repo(PathBuf);

impl Repo {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "mixed-path-contract-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        Self(root)
    }

    fn write(&self, path: &str, text: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    /// A check ignores the version 1 caution: `[fmt.cross_refs] enabled = false`
    /// has no version 2 spelling, and the caution is not these cases' subject.
    fn run(&self, args: &[&str]) -> Output {
        let caution: &[&str] = if args.first() == Some(&"check") {
            &["--ignore", CONFIG_VERSION_1]
        } else {
            &[]
        };
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .args(caution)
            .current_dir(&self.0)
            .output()
            .unwrap()
    }

    fn prepared(direction: &str, citations: &str) -> Self {
        let repo = Self::new();
        repo.write("grund.toml", &format!("{CONFIG}{direction}"));
        repo.write("docs/fs/x.md", &format!("# FS-x: X\n\n{citations}\n"));
        repo.write(
            "docs/goals/g.md",
            "# GOAL-g: G\n\nSee \u{a7}FS-x.\n\n## 1. One\n\nOne.\n\n### 1.goals: Goals\n\nGoals.\n",
        );
        let fmt = repo.run(&["fmt", "--write"]);
        assert!(fmt.status.success(), "{fmt:?}");
        repo
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn json(output: &Output) -> Vec<Value> {
    assert!(output.stderr.is_empty(), "{output:?}");
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| serde_json::from_str(line).expect("check NDJSON"))
        .collect()
}

/// Each finding with its message cut before the counted list, which names every
/// citation as written and so differs between a mixed and an ordinary spelling
/// (§FS-rules.7.3.1, §FS-rules.7.12); every other field is kept whole.
fn before_the_list(mut ds: Vec<Value>) -> Vec<Value> {
    for d in &mut ds {
        let message = d["message"].as_str().unwrap();
        let head = message.split(COUNTED).next().unwrap().to_string();
        d["message"] = Value::String(head);
    }
    ds
}

fn ordinary_error(repo: &Repo, args: &[&str], code: &str) {
    let output = repo.run(args);
    let ds = json(&output);
    assert_eq!(output.status.code(), Some(1), "{ds:#?}");
    assert_eq!(ds.len(), 1, "{ds:#?}");
    assert_eq!(ds[0]["code"], code, "{ds:#?}");
    assert_eq!(ds[0]["severity"], "error", "{ds:#?}");
    assert!(!ds[0]["message"].as_str().unwrap().contains(MIXED));
    let path = repo.0.join("docs/fs/x.md");
    let source = fs::read_to_string(&path).unwrap();
    fs::write(
        &path,
        source.replace("\u{a7}GOAL-g.1.goals", "\u{a7}GOAL-g"),
    )
    .unwrap();
    let ordinary = repo.run(args);
    assert_eq!(ordinary.status.code(), Some(1), "{ordinary:?}");
    assert_eq!(
        before_the_list(ds),
        before_the_list(json(&ordinary)),
        "ordinary codes, messages up to the counted list, sites and authority"
    );
}

/// The one count finding `--rule <sentence>` raises names, in its list, every
/// citation it counted as written, `lines` of `docs/fs/x.md` in order, and its
/// `sites` are those lines (§FS-rules.7.3.1).
fn names_as_written(repo: &Repo, sentence: &str, code: &str, list: &str, lines: &[usize]) {
    let output = repo.run(&["check", "--format", "json", "--rule", sentence]);
    let ds = json(&output);
    assert_eq!(output.status.code(), Some(1), "{ds:#?}");
    assert_eq!(ds.len(), 1, "{ds:#?}");
    assert_eq!(ds[0]["code"], code, "{ds:#?}");
    let message = ds[0]["message"].as_str().unwrap();
    assert!(message.ends_with(list), "{sentence}: {message}");
    let sites: Vec<Value> = lines
        .iter()
        .map(|line| serde_json::json!({"path": "docs/fs/x.md", "line": line}))
        .collect();
    assert_eq!(ds[0]["sites"], Value::Array(sites), "{ds:#?}");
}

/// §FS-rules.7.12 and §FS-rules.7.3.1: a count's ordinary message carries the
/// counted list, which names the mixed citation as written, `GOAL-g.1.goals`,
/// where the ordinary run's names `GOAL-g`; it is what shows the author which
/// citation counted. It covers every count case `ordinary_error` runs below.
#[test]
fn the_counted_list_names_a_mixed_citation_as_written() {
    let two = "; counted at docs/fs/x.md:3 (GOAL-g), docs/fs/x.md:5 (GOAL-g.1.goals)";
    for (sentence, code) in [
        ("Each FS must cite at most 1 GOAL.", "citation-cardinality"),
        (
            "Each FS must cite exactly one GOAL.",
            "citation-cardinality",
        ),
        (
            "Each FS must cite each GOAL at most once.",
            "citation-cardinality",
        ),
        (
            "Each FS must cite each GOAL exactly once.",
            "citation-cardinality",
        ),
        ("Each GOAL must be cited by at most 1 FS.", "uncited-unit"),
        ("Each GOAL must be cited by exactly one FS.", "uncited-unit"),
    ] {
        let repo = Repo::prepared("", "See \u{a7}GOAL-g.\n\nAgain \u{a7}GOAL-g.1.goals.");
        names_as_written(&repo, sentence, code, two, &[3, 5]);
    }
    let repo = Repo::prepared(
        "",
        "See \u{a7}GOAL-g.\n\nAgain \u{a7}GOAL-g.\n\nMixed \u{a7}GOAL-g.1.goals.",
    );
    names_as_written(
        &repo,
        "Each FS must cite at most 1 GOAL.",
        "citation-cardinality",
        "; counted at docs/fs/x.md:3 (GOAL-g), docs/fs/x.md:5 (GOAL-g), \
         docs/fs/x.md:7 (GOAL-g.1.goals)",
        &[3, 5, 7],
    );
}

#[test]
fn unresolved_mixed_text_has_exact_ordinary_messages_and_channels() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/e2e/cases/check-number-name-citation-missing/repo");
    let output = Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(["check", root.to_str().unwrap()])
        .args(["--ignore", CONFIG_VERSION_1])
        .output()
        .unwrap();
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "docs/use.md:5: error: section not found: FS-x.1.nope; write <§> before it to show the shape without citing it\n\
         docs/use.md:7: error: unknown reference FS-nope\n"
    );
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn unresolved_mixed_ndjson_keeps_codes_sites_and_filters() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/e2e/cases/check-number-name-citation-missing/repo");
    for (flags, codes, exit) in [
        (vec![], vec!["missing-section", "dangling"], 1),
        (
            vec!["--only", "missing-section"],
            vec!["missing-section"],
            1,
        ),
        (
            vec!["--ignore", "missing-section", "--ignore", "dangling"],
            vec![],
            0,
        ),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(["check", root.to_str().unwrap(), "--format", "json"])
            .args(["--ignore", CONFIG_VERSION_1])
            .args(flags)
            .output()
            .unwrap();
        let ds = json(&output);
        assert_eq!(
            ds.iter()
                .map(|d| d["code"].as_str().unwrap())
                .collect::<Vec<_>>(),
            codes
        );
        for d in &ds {
            assert_eq!(d["path"], "docs/use.md");
            assert_eq!(d["line"], if d["code"] == "dangling" { 7 } else { 5 });
            assert_eq!(d["severity"], "error", "{d:#?}");
            assert!(d["sites"].is_null(), "{d:#?}");
            assert!(!d["message"].as_str().unwrap().contains(MIXED));
        }
        assert_eq!(output.status.code(), Some(exit), "{ds:#?}");
    }
}

#[test]
fn configured_mixed_prohibition_is_an_ordinary_error() {
    let repo = Repo::prepared(
        "\n[citations]\ndefault = \"may\"\n[citations.FS]\nmust-not = [\"GOAL\"]\n",
        "See \u{a7}GOAL-g.1.goals.",
    );
    ordinary_error(&repo, &["check", "--format", "json"], "forbidden-citation");
}

macro_rules! upper_count {
    ($name:ident, $sentence:literal, $code:literal) => {
        #[test]
        fn $name() {
            let repo = Repo::prepared("", "See \u{a7}GOAL-g.\n\nAgain \u{a7}GOAL-g.1.goals.");
            ordinary_error(
                &repo,
                &["check", "--format", "json", "--rule", $sentence],
                $code,
            );
        }
    };
}

upper_count!(
    outbound_at_most_mixed_is_error,
    "Each FS must cite at most 1 GOAL.",
    "citation-cardinality"
);
upper_count!(
    outbound_exactly_mixed_is_error,
    "Each FS must cite exactly one GOAL.",
    "citation-cardinality"
);
upper_count!(
    per_target_at_most_mixed_is_error,
    "Each FS must cite each GOAL at most once.",
    "citation-cardinality"
);

/// §FS-rules.3.3: the new spelling retains mixed-path physical-site enforcement.
/// The upper-count case above uses the same canonical spelling and enforcement assertion.
#[test]
fn per_target_once_mixed_sites_keep_the_ordinary_enforcement_assertion() {
    let repo = Repo::prepared("", "See \u{a7}GOAL-g.\n\nAgain \u{a7}GOAL-g.1.goals.");
    ordinary_error(
        &repo,
        &[
            "check",
            "--format",
            "json",
            "--rule",
            "Each FS must cite each GOAL at most once.",
        ],
        "citation-cardinality",
    );
}
upper_count!(
    per_target_exactly_mixed_is_error,
    "Each FS must cite each GOAL exactly once.",
    "citation-cardinality"
);
upper_count!(
    inbound_at_most_mixed_is_error,
    "Each GOAL must be cited by at most 1 FS.",
    "uncited-unit"
);
upper_count!(
    inbound_exactly_mixed_is_error,
    "Each GOAL must be cited by exactly one FS.",
    "uncited-unit"
);

#[test]
fn ordinary_sites_that_already_break_a_count_still_error() {
    let repo = Repo::prepared(
        "",
        "See \u{a7}GOAL-g.\n\nAgain \u{a7}GOAL-g.\n\nMixed \u{a7}GOAL-g.1.goals.",
    );
    ordinary_error(
        &repo,
        &[
            "check",
            "--format",
            "json",
            "--rule",
            "Each FS must cite at most 1 GOAL.",
        ],
        "citation-cardinality",
    );
}

#[test]
fn recommended_findings_stay_opt_in_suggestions() {
    for direction in [
        "\n[citations]\ndefault = \"may\"\n[citations.FS]\nshould-not = [\"GOAL\"]\n",
        "",
    ] {
        let repo = Repo::prepared(direction, "See \u{a7}GOAL-g.1.goals.");
        let mut args = vec!["check", "--format", "json"];
        if direction.is_empty() {
            args.extend(["--rule", "Each FS should not cite any GOAL."]);
        }
        let hidden = repo.run(&args);
        assert!(hidden.status.success());
        assert!(json(&hidden).is_empty());
        args.push("--suggestions");
        let shown = repo.run(&args);
        let ds = json(&shown);
        assert!(shown.status.success(), "{ds:#?}");
        assert_eq!(ds.len(), 1);
        assert_eq!(ds[0]["channel"], "suggestion");
        assert_eq!(ds[0]["code"], "discouraged-citation");
        assert!(!ds[0]["message"].as_str().unwrap().contains(MIXED));
    }
}

#[test]
fn v2_warn_and_warn_not_stay_standing_warnings() {
    for strength in ["warn", "warn-not"] {
        let repo = Repo::new();
        repo.write(
            "grund.toml",
            &format!(
                "grund_config_version = 2\nproject_name = \"example\"\n\
             [schema.sources]\nlanguages = [\"markdown\"]\n\
             [schema.kinds.FS]\nfiles = [\"docs/fs.md\"]\n\
             [schema.kinds.GOAL]\nfiles = [\"docs/goals.md\"]\n\
             [rules.citations.FS]\n{strength} = [\"GOAL\"]\n"
            ),
        );
        repo.write(
            "docs/fs.md",
            if strength == "warn" {
                "# FS-x: X\n\nNo citation.\n"
            } else {
                "# FS-x: X\n\nSee \u{a7}GOAL-g.1.goals.\n"
            },
        );
        repo.write(
            "docs/goals.md",
            "# GOAL-g: G\n\n## 1. One\n\nOne.\n\n### 1.goals: Goals\n\nGoals.\n",
        );
        assert!(repo.run(&["fmt", "--write"]).status.success());
        let output = repo.run(&[
            "check",
            "--format",
            "json",
            "--only",
            if strength == "warn" {
                "missing-citation"
            } else {
                "forbidden-citation"
            },
        ]);
        let ds = json(&output);
        assert!(output.status.success(), "{ds:#?}");
        assert_eq!(ds.len(), 1, "{ds:#?}");
        assert_eq!(ds[0]["severity"], "warning");
        assert_eq!(
            ds[0]["code"],
            if strength == "warn" {
                "missing-citation"
            } else {
                "forbidden-citation"
            }
        );
        assert!(!ds[0]["message"].as_str().unwrap().contains(MIXED));
    }
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let dest = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &dest);
        } else {
            fs::copy(entry.path(), dest).unwrap();
        }
    }
}

#[test]
fn configured_lent_mixed_overlap_is_an_ordinary_error() {
    // §FS-rules.7.11 and §FS-rules.7.12 have both closed, so no window relieves it.
    let repo = Repo::new();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../tests/e2e/cases/check-stub-unscanned-target-citations-prohibition-errors/repo",
    );
    copy_tree(&fixture, &repo.0);
    let source = fs::read_to_string(repo.0.join("src/x.rs")).unwrap();
    repo.write(
        "src/x.rs",
        &source.replace("\u{a7}GOAL-g.", "\u{a7}GOAL-g.1.goals."),
    );
    let goal = fs::read_to_string(repo.0.join("docs/goals/g.md")).unwrap();
    repo.write(
        "docs/goals/g.md",
        &format!("{goal}\n## 1. One\n\nOne.\n\n### 1.goals: Goals\n\nGoals.\n"),
    );
    assert!(repo.run(&["fmt", "--write"]).status.success());
    let output = repo.run(&["check", "--format", "json"]);
    let ds = json(&output);
    assert_eq!(output.status.code(), Some(1), "{ds:#?}");
    assert_eq!(ds.len(), 1, "{ds:#?}");
    assert_eq!(ds[0]["code"], "forbidden-citation");
    assert_eq!(ds[0]["severity"], "error");
    assert_eq!(
        ds[0]["message"],
        "FS must not cite GOAL (citation direction) \u{2014} re-point the citation or downgrade it to a plain Markdown link",
        "{ds:#?}"
    );
}
