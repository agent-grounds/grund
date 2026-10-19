//! Missing-target repair guidance and its boundaries (§FS-declarations.stubs.verdict.repair).
//! These black-box tests invoke the built CLI on the issue #820 fixture.

use serde_json::{Value, json};
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const CONFIG: &str = "grund_config_version = 2\nproject_name = \"demo\"\n\
    [schema.kinds.AR]\nfolders = [\"docs/architecture\", \"zdocs\"]\nindex = false\n\
    [schema.kinds.code]\nfolders = [\"src\", \"asrc\"]\ncitable = false\n";
const DECLARATION: &str = "/// AR-cache: Cache\n///\n/// ## 1. Eviction\n///\n\
    /// Least recently used.\npub struct Cache;\n";
const STUB: &str = "docs/architecture/AR-cache.md";
const REAL: &str = "src/store/cache.rs";
const MISSING: &str = "stub link target missing: src/cache.rs";
const QUERY_PREFIX: &str =
    "ambiguous ID: AR-cache (declared at docs/architecture/AR-cache.md:1, src/store/cache.rs:1)";
static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Repo(PathBuf);

impl Repo {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "grund-820-repair-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).expect("create isolated fixture");
        let repo = Self(root.canonicalize().unwrap());
        repo.write("grund.toml", CONFIG);
        repo.write(STUB, "# AR-cache: [src/cache.rs](src/cache.rs)\n");
        repo.write(REAL, DECLARATION);
        repo.write("src/main.rs", "// Uses \u{a7}AR-cache\nfn main() {}\n");
        repo
    }

    fn write(&self, path: &str, body: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }

    fn run(&self, args: &[&str], exit: i32) -> Output {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .env("GIT_CONFIG_GLOBAL", self.0.join("no-global-config"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .output()
            .expect("run this checkout's CLI");
        assert_eq!(output.status.code(), Some(exit), "{args:?}: {output:?}");
        output
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn utf8(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).unwrap()
}

fn records(bytes: &[u8]) -> Vec<Value> {
    utf8(bytes)
        .lines()
        .map(|line| serde_json::from_str(line).expect("finding JSON"))
        .collect()
}

fn sites(paths: &[&str]) -> Value {
    json!(
        paths
            .iter()
            .map(|path| json!({"path": path, "line": 1}))
            .collect::<Vec<_>>()
    )
}

fn shape(finding: &Value) {
    let mut keys = finding
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>();
    keys.sort_unstable();
    assert_eq!(
        keys,
        [
            "authority",
            "code",
            "line",
            "message",
            "path",
            "severity",
            "sites"
        ]
    );
    assert_eq!(finding["severity"], "error");
    assert_eq!(finding["authority"], Value::Null);
}

fn unique_repair(message: &str, real: &str) {
    assert!(
        message.contains(real),
        "missing real declaration {real}: {message}"
    );
    assert!(
        message.contains("AR-cache is declared at"),
        "missing relationship: {message}"
    );
    assert!(
        message.contains("point the stub"),
        "missing repair instruction: {message}"
    );
}

#[test]
#[ignore = "red until #820 connects missing-target diagnostics to the repair; implement removes this"]
fn missing_stub_check_text_connects_the_repair() {
    let repo = Repo::new();
    let output = repo.run(&["check"], 1);
    assert!(output.stderr.is_empty(), "{output:?}");
    let lines = utf8(&output.stdout).lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert!(lines[0].starts_with("docs/architecture/AR-cache.md:1: error: duplicate declaration of AR-cache (also declared at src/store/cache.rs:1)"));
    assert!(lines[1].starts_with(
        "docs/architecture/AR-cache.md:1: error: stub link target missing: src/cache.rs"
    ));
    unique_repair(lines[1], REAL);
}

#[test]
#[ignore = "red until #820 connects missing-target diagnostics to the repair; implement removes this"]
fn missing_stub_check_json_keeps_located_finding_contract() {
    let repo = Repo::new();
    let text = repo.run(&["check"], 1);
    let output = repo.run(&["check", "--format=json"], 1);
    assert!(output.stderr.is_empty(), "{output:?}");
    let findings = records(&output.stdout);
    assert_eq!(findings.len(), 2, "{findings:?}");
    for (finding, line) in findings.iter().zip(utf8(&text.stdout).lines()) {
        shape(finding);
        assert_eq!(finding["path"], STUB);
        assert_eq!(finding["line"], 1);
        assert_eq!(
            line,
            format!("{STUB}:1: error: {}", finding["message"].as_str().unwrap())
        );
    }
    assert_eq!(findings[0]["code"], "duplicate");
    assert_eq!(findings[0]["sites"], sites(&[STUB, REAL]));
    assert_eq!(findings[1]["code"], "broken-stub");
    let message = findings[1]["message"].as_str().unwrap();
    assert!(message.starts_with(MISSING));
    unique_repair(message, REAL);
    // §FS-errors.5.1: actual sites named by this multi-site message, never the absent target.
    assert_eq!(findings[1]["sites"], sites(&[STUB, REAL]));
}

#[test]
#[ignore = "red until #820 connects missing-target diagnostics to the repair; implement removes this"]
fn missing_stub_all_text_queries_explain_the_ambiguity() {
    let repo = Repo::new();
    let mut gaps = Vec::new();
    for args in [
        vec!["AR-cache"],
        vec!["AR-cache", "--toc"],
        vec!["AR-cache.1"],
        vec!["refs", "AR-cache"],
    ] {
        let output = repo.run(&args, 1);
        assert!(output.stdout.is_empty(), "{args:?}: {output:?}");
        let message = utf8(&output.stderr);
        assert!(message.starts_with(QUERY_PREFIX), "{args:?}: {message}");
        let suffix = &message[QUERY_PREFIX.len()..];
        for required in [
            "broken stub",
            STUB,
            "src/cache.rs",
            REAL,
            "AR-cache is declared at",
            "point the stub",
        ] {
            if !suffix.contains(required) {
                gaps.push(format!(
                    "{args:?}: missing {required:?} after prefix: {message}"
                ));
            }
        }
    }
    assert!(gaps.is_empty(), "{}", gaps.join("\n"));
}

#[test]
#[ignore = "red until #820 connects missing-target diagnostics to the repair; implement removes this"]
fn missing_stub_query_json_preserves_original_sites_and_prefix() {
    let repo = Repo::new();
    // A second lexical order prevents a repair helper from assuming the first site is a stub.
    fs::remove_file(repo.0.join(STUB)).unwrap();
    fs::remove_file(repo.0.join(REAL)).unwrap();
    repo.write(
        "zdocs/cache.md",
        "# AR-cache: [src/cache.rs](src/cache.rs)\n",
    );
    repo.write("asrc/cache.rs", DECLARATION);
    let prefix = "ambiguous ID: AR-cache (declared at asrc/cache.rs:1, zdocs/cache.md:1)";
    for args in [vec!["AR-cache.1"], vec!["refs", "AR-cache"]] {
        let text = repo.run(&args, 1);
        let mut args = args;
        args.push("--format=json");
        let output = repo.run(&args, 1);
        assert!(output.stdout.is_empty(), "{output:?}");
        let findings = records(&output.stderr);
        assert_eq!(findings.len(), 1);
        let finding = &findings[0];
        shape(finding);
        assert_eq!(finding["code"], "ambiguous");
        assert_eq!(finding["path"], Value::Null);
        assert_eq!(finding["line"], Value::Null);
        assert_eq!(
            finding["sites"],
            sites(&["asrc/cache.rs", "zdocs/cache.md"])
        );
        let message = finding["message"].as_str().unwrap();
        assert_eq!(utf8(&text.stderr), format!("{message}\n"));
        assert!(message.starts_with(prefix), "{message}");
        let suffix = &message[prefix.len()..];
        assert!(
            suffix.contains("broken stub"),
            "missing broken-stub explanation: {message}"
        );
        assert!(
            suffix.contains("zdocs/cache.md") && suffix.contains("src/cache.rs"),
            "{message}"
        );
        unique_repair(suffix, "asrc/cache.rs");
    }
}

#[test]
#[ignore = "red until #820 connects missing-target diagnostics to the repair; implement removes this"]
fn missing_stub_several_real_homes_are_facts_without_a_chosen_repair() {
    let repo = Repo::new();
    repo.write("src/other/cache.rs", DECLARATION);
    let check = repo.run(&["check", "--format=json"], 1);
    let findings = records(&check.stdout);
    let broken = findings
        .iter()
        .find(|f| f["code"] == "broken-stub")
        .unwrap();
    let message = broken["message"].as_str().unwrap();
    assert!(message.starts_with(MISSING));
    assert!(
        !message.contains("point the stub"),
        "must not choose: {message}"
    );
    assert!(
        message.contains("src/other/cache.rs") && message.contains(REAL),
        "missing available homes: {message}"
    );
    assert_eq!(broken["sites"], sites(&[STUB, "src/other/cache.rs", REAL]));
    let query = repo.run(&["refs", "AR-cache", "--format=json"], 1);
    let query = records(&query.stderr);
    assert_eq!(
        query[0]["sites"],
        sites(&[STUB, "src/other/cache.rs", REAL])
    );
    let message = query[0]["message"].as_str().unwrap();
    assert!(message.starts_with("ambiguous ID: AR-cache (declared at docs/architecture/AR-cache.md:1, src/other/cache.rs:1, src/store/cache.rs:1)"));
    assert!(
        message.contains("broken stub") && message.contains("src/cache.rs"),
        "{message}"
    );
    assert!(
        !message.contains("point the stub"),
        "must not choose: {message}"
    );
}

#[test]
#[ignore = "red until #820 connects missing-target diagnostics to the repair; implement removes this"]
fn missing_stub_another_broken_stub_is_not_a_repair_destination() {
    let repo = Repo::new();
    repo.write(
        "docs/architecture/second.md",
        "# AR-cache: [src/also-missing.rs](src/also-missing.rs)\n",
    );
    let output = repo.run(&["check", "--format=json"], 1);
    let findings = records(&output.stdout);
    let broken = findings
        .iter()
        .filter(|f| f["code"] == "broken-stub")
        .collect::<Vec<_>>();
    assert_eq!(broken.len(), 2);
    for finding in broken {
        let message = finding["message"].as_str().unwrap();
        unique_repair(message, REAL);
        assert!(
            !message.contains("second.md"),
            "a broken stub is not a candidate: {message}"
        );
    }
    let duplicate = findings.iter().find(|f| f["code"] == "duplicate").unwrap();
    assert_eq!(
        duplicate["sites"],
        sites(&[STUB, "docs/architecture/second.md", REAL])
    );
}

#[test]
#[ignore = "red until #820 connects missing-target diagnostics to the repair; implement removes this"]
fn missing_stub_a_healthy_stub_can_represent_the_only_real_home() {
    let repo = Repo::new();
    fs::remove_file(repo.0.join(REAL)).unwrap();
    // The real home is deliberately outside the configured scan places.
    repo.write("unwalked/cache.rs", DECLARATION);
    repo.write(
        "docs/architecture/healthy.md",
        "# AR-cache: [unwalked/cache.rs](unwalked/cache.rs)\n",
    );
    let output = repo.run(&["check", "--format=json"], 1);
    let findings = records(&output.stdout);
    let duplicate = findings.iter().find(|f| f["code"] == "duplicate").unwrap();
    assert_eq!(duplicate["sites"], sites(&[STUB, "unwalked/cache.rs"]));
    let broken = findings
        .iter()
        .find(|f| f["code"] == "broken-stub")
        .unwrap();
    unique_repair(broken["message"].as_str().unwrap(), "unwalked/cache.rs");
    assert_eq!(broken["sites"], sites(&[STUB, "unwalked/cache.rs"]));
}

#[test]
fn missing_stub_without_real_homes_keeps_the_ordinary_check_message() {
    let repo = Repo::new();
    fs::remove_file(repo.0.join(REAL)).unwrap();
    let output = repo.run(&["check", "--format=json"], 1);
    let findings = records(&output.stdout);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0]["message"], MISSING);
    assert_eq!(findings[0]["sites"], Value::Null);
    repo.write(
        "docs/architecture/second.md",
        "# AR-cache: [src/also-missing.rs](src/also-missing.rs)\n",
    );
    let output = repo.run(&["check", "--format=json"], 1);
    for finding in records(&output.stdout)
        .iter()
        .filter(|f| f["code"] == "broken-stub")
    {
        assert!(
            !finding["message"]
                .as_str()
                .unwrap()
                .contains("point the stub")
        );
        assert_eq!(finding["sites"], Value::Null);
    }
}

#[test]
fn missing_stub_guidance_does_not_leak_to_other_verdicts() {
    for (target, contents, source_config, expected) in [
        ("src/cache.rs", DECLARATION, "", None),
        (
            "src/.cache.rs",
            DECLARATION,
            "",
            Some("stub link target lacks AR-cache: src/.cache.rs"),
        ),
        (
            "src/cache.zz",
            DECLARATION,
            "",
            Some("is not scanned: extension zz is not in [scan] extensions"),
        ),
        (
            "src/cache.rs",
            "pub struct Other;\n",
            "",
            Some("stub link target lacks AR-cache: src/cache.rs"),
        ),
        (
            "src/old/cache.rs",
            DECLARATION,
            "\n[schema.sources]\nexclude = [\"src/old/**\"]\n",
            Some("is not scanned:"),
        ),
    ] {
        let repo = Repo::new();
        repo.write("grund.toml", &format!("{CONFIG}{source_config}"));
        repo.write(STUB, &format!("# AR-cache: [{target}]({target})\n"));
        repo.write(target, contents);
        let output = repo.run(&["check", "--format=json"], 1);
        let findings = records(&output.stdout);
        let broken = findings.iter().find(|f| f["code"] == "broken-stub");
        if let Some(expected) = expected {
            let finding = broken.expect("non-missing broken verdict");
            assert!(
                finding["message"].as_str().unwrap().contains(expected),
                "{finding}"
            );
            assert_eq!(finding["sites"], Value::Null);
        } else {
            assert!(broken.is_none(), "healthy pair: {findings:?}");
        }
        let query = repo.run(&["AR-cache"], 1);
        let text = utf8(&query.stderr);
        assert!(
            text.starts_with("ambiguous ID: AR-cache"),
            "{target}: {text}"
        );
        assert!(
            !text.contains("point the stub") && !text.contains("broken stub"),
            "{target}: {text}"
        );
    }
}
