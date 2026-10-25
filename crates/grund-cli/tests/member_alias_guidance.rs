//! §FS-workspace.5.1.1: a member's refusal explains a verified enclosing root,
//! with a usable location and command. These black-box checks port #823's
//! root-success/member-failure reproducer without enabling workspace resolution.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Value, json};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let scratch = std::env::temp_dir().join(format!(
            "grund-823-member-alias-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&scratch).expect("create isolated fixture");
        let fixture = Self(scratch);
        let source =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/member_alias_guidance/repo");
        for file in [
            "grund.toml",
            "requirements.md",
            "packages/billing/grund.toml",
            "packages/billing/requirements.md",
        ] {
            let target = fixture.root().join(file);
            fs::create_dir_all(target.parent().unwrap()).unwrap();
            fs::copy(source.join(file), target).unwrap();
        }
        fixture
    }

    fn root(&self) -> PathBuf {
        self.0.join("repo")
    }

    fn member(&self) -> PathBuf {
        self.root().join("packages/billing")
    }

    fn set_citation(&self, target: &str) {
        fs::write(
            self.member().join("requirements.md"),
            format!(
                "# Requirements\n\n## FS-001-invoice: Invoice\n\n\
                 Invoice implements \u{a7}{target}. Local use: \u{a7}FS-001-invoice.\n"
            ),
        )
        .unwrap();
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove isolated fixture");
    }
}

fn run(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .current_dir(cwd)
        .args(args)
        .output()
        .expect("run grund")
}

fn text(output: &Output, code: i32) -> String {
    assert_eq!(
        output.status.code(),
        Some(code),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty(), "{output:?}");
    String::from_utf8(output.stdout.clone()).unwrap()
}

fn assert_root_success(fixture: &Fixture) {
    assert_eq!(text(&run(&fixture.root(), &["check"]), 0), "success\n");
}

fn guidance(message: &str, cwd: &Path, root: &Path, location: &str) {
    let explanation = message
        .strip_prefix("unknown project alias shop")
        .expect("retain the unknown-alias finding");
    for meaning in ["workspace", "member", "root", "project", "grund check"] {
        assert!(
            explanation.contains(meaning),
            "missing {meaning:?} in verified enclosing-workspace guidance: {message}"
        );
    }
    assert!(
        explanation.split_whitespace().any(|word| {
            word.trim_matches(['`', '\'', '"', ',', ';', ':', '(', ')']) == location
        }),
        "missing usable root location {location:?} from {}: {message}",
        cwd.display()
    );
    assert_eq!(
        cwd.join(location).canonicalize().unwrap(),
        root.canonicalize().unwrap(),
        "the advertised location must reach the workspace root"
    );
}

fn member_text(fixture: &Fixture, cwd: &Path, args: &[&str], location: &str) {
    let output = text(&run(cwd, args), 1);
    let message = output
        .strip_prefix("requirements.md:5: error: ")
        .expect("one error at the citation site")
        .trim_end();
    assert!(
        !message.contains('\n'),
        "unexpected extra findings: {output}"
    );
    guidance(message, cwd, &fixture.root(), location);
}

#[test]
fn configured_member_cwd_explains_enclosing_workspace() {
    let fixture = Fixture::new();
    assert_root_success(&fixture);
    member_text(&fixture, &fixture.member(), &["check"], "../..");
}

#[test]
fn explicit_member_paths_offer_a_root_location_from_the_invocation() {
    let fixture = Fixture::new();
    assert_root_success(&fixture);
    // Collect both forms before asserting, so a red first form does not hide
    // whether the explicit-path discovery seam ran.
    let dot = run(&fixture.member(), &["check", ".", "--format=json"]);
    let from_root = run(
        &fixture.root(),
        &["check", "packages/billing", "--format=json"],
    );
    for (output, cwd, location) in [
        (dot, fixture.member(), "../.."),
        (from_root, fixture.root(), "."),
    ] {
        let stdout = text(&output, 1);
        let findings: Vec<Value> = stdout
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(findings.len(), 1, "{stdout}");
        let finding = &findings[0];
        let message = finding["message"].as_str().unwrap();
        assert_eq!(
            finding,
            &json!({"severity":"error", "path":"requirements.md", "line":5,
                    "code":"unknown-project", "message":message,
                    "sites":null, "authority":null}),
            "the diagnostic gains no fields or different severity/code/site"
        );
        guidance(message, &cwd, &fixture.root(), location);
    }
}

#[test]
fn configless_member_discovery_offers_the_same_root_remedy() {
    let fixture = Fixture::new();
    fs::remove_file(fixture.member().join("grund.toml")).unwrap();
    assert_root_success(&fixture);
    let member = fixture.member();
    let member_arg = member.to_str().unwrap();
    let from_cwd = run(&member, &["check"]);
    let explicit = run(&fixture.0, &["check", member_arg]);
    for (output, cwd, location) in [
        (from_cwd, member, "../.."),
        (explicit, fixture.0.clone(), "repo"),
    ] {
        let stdout = text(&output, 1);
        let message = stdout
            .strip_prefix("requirements.md:5: error: ")
            .unwrap()
            .trim_end();
        guidance(message, &cwd, &fixture.root(), location);
    }
}

#[test]
fn root_remedy_validates_a_target_without_promising_it_resolves() {
    let fixture = Fixture::new();
    fixture.set_citation("shop/FS-001-missing");
    let root = text(&run(&fixture.root(), &["check"]), 1);
    assert!(
        root.contains("unknown reference shop/FS-001-missing"),
        "{root}"
    );
    member_text(&fixture, &fixture.member(), &["check"], "../..");
}

#[test]
fn genuinely_unknown_alias_does_not_claim_to_be_the_enclosing_project() {
    let fixture = Fixture::new();
    fixture.set_citation("ghost/FS-001-checkout");
    assert_eq!(
        text(&run(&fixture.member(), &["check"]), 1),
        "requirements.md:5: error: unknown project alias ghost\n"
    );
}

/// §FS-workspace.5.1.1 needs the root project's actual alias, and an ancestor
/// above `repo` whose `members` text cannot be obtained may list it
/// (§FS-workspace.6.1.7.5) — then `shop` names a member, not the root, so the
/// remedy is withheld. A readable ancestor that names nothing here verifies it.
#[test]
#[ignore = "red until #1010 the root remedy is withheld under an undecidable ancestor; implement removes this"]
fn an_undecidable_ancestor_above_the_root_withholds_the_root_remedy() {
    let fixture = Fixture::new();
    let outer = |members: &str| {
        fs::write(
            fixture.0.join("grund.toml"),
            format!(
                "grund_config_version = 2\nproject_name = \"outer\"\n\n\
                 [workspace]\nmembers = {members}\n"
            ),
        )
        .unwrap();
    };
    outer("[\"elsewhere\"]");
    member_text(&fixture, &fixture.member(), &["check"], "../..");
    outer("[\"repo\"");
    assert_eq!(
        text(&run(&fixture.member(), &["check"]), 1),
        "requirements.md:5: error: unknown project alias shop\n"
    );
}

#[test]
fn standalone_and_unlisted_projects_do_not_claim_workspace_membership() {
    let fixture = Fixture::new();
    assert_root_success(&fixture);
    for project in [
        fixture.0.join("standalone"),
        fixture.root().join("packages/unlisted"),
    ] {
        fs::create_dir_all(&project).unwrap();
        for file in ["grund.toml", "requirements.md"] {
            fs::copy(fixture.member().join(file), project.join(file)).unwrap();
        }
        assert_eq!(
            text(&run(&project, &["check"]), 1),
            "requirements.md:5: error: unknown project alias shop\n"
        );
    }
}
