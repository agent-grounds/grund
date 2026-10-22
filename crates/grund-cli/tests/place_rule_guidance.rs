//! Black-box place-subject guidance (§FS-rule-refusals.sentences.rewrite.no-form.place).
//! No production parser is used: both refusal and suggested configuration run
//! through the shipped executable, including the existing direction check.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Repo(PathBuf);

impl Repo {
    fn new(version: u8) -> Self {
        let root = std::env::temp_dir().join(format!(
            "grund-828-place-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("tests/e2e")).unwrap();
        fs::create_dir_all(root.join("tests/integration")).unwrap();
        let repo = Self(root);
        let init = Command::new("git")
            .args(["init", "-q"])
            .current_dir(&repo.0)
            .output()
            .unwrap();
        assert!(init.status.success(), "git init: {init:?}");
        repo.write("grund.toml", &config(version));
        repo.write("requirements.md", "# FS-demo: Demo\n");
        repo.write("architecture.md", "# AR-demo: Demo\n");
        // No FS citation: AR evidence puts this file in the existing direction
        // check's domain (§FS-config.3.9.2.4); wholly uncited files are exempt.
        repo.write("tests/e2e/uncited.rs", "// \u{a7}AR-demo\nfn main() {}\n");
        repo
    }

    fn write(&self, path: &str, text: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .output()
            .unwrap()
    }

    fn rule(&self, sentence: &str) -> Output {
        self.run(&["check", "--rule", sentence, "--only-rule"])
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn config(version: u8) -> String {
    let schema = if version == 2 {
        r#"[schema]
id_format = "{kind}-{slug}"
[schema.kinds.AR]
files = ["architecture.md"]
[schema.kinds.FS]
files = ["requirements.md"]
[schema.kinds.e2e]
folders = ["tests/e2e"]
citable = false
[schema.kinds.integration]
folders = ["tests/integration"]
citable = false
"#
    } else {
        r#"[id]
format = "{kind}-{slug}"
[[kinds]]
kind = "AR"
file = "architecture.md"
[[kinds]]
kind = "FS"
file = "requirements.md"
[[kinds]]
kind = "e2e"
folder = "tests/e2e"
citable = false
[[kinds]]
kind = "integration"
folder = "tests/integration"
citable = false
[scan]
include = ["requirements.md", "architecture.md", "tests", "docs"]
"#
    };
    format!("grund_config_version = {version}\nproject_name = \"demo\"\n{schema}")
}

fn table(version: u8) -> &'static str {
    if version == 2 {
        "rules.citations.e2e"
    } else {
        "citations.e2e"
    }
}

/// The setting that makes an uncited e2e file a finding, as the hint spells it
/// (§FS-init.2.3.5.2.2).
fn grounding(version: u8) -> &'static str {
    if version == 2 {
        "[rules.citations.e2e.grounding] must = \"file\""
    } else {
        "require_grounding = true on its [[kinds]] row"
    }
}

/// The config with the e2e row grounded, in the version's own spelling.
fn grounded_config(version: u8) -> String {
    if version == 2 {
        config(version) + "\n[rules.citations.e2e.grounding]\nmust = \"file\"\n"
    } else {
        config(version).replace(
            "folder = \"tests/e2e\"\ncitable = false",
            "folder = \"tests/e2e\"\ncitable = false\nrequire_grounding = true",
        )
    }
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).unwrap()
}

fn refused(output: &Output, prefix: &str) -> String {
    let error = stderr(output);
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert!(error.starts_with(prefix), "{error}");
    assert!(error.contains("\nknown kinds: AR, FS\n"), "{error}");
    assert!(!error.contains("accepted form"), "{error}");
    error
}

fn guidance_contract(version: u8, subject: &str) {
    let repo = Repo::new(version);
    let sentence = format!("{subject} must cite at least one FS.");
    let prefix = if subject == "Each e2e" {
        // §FS-errors.3.8.1: the non-citable reason alone, since the window closed.
        "error: kind \"e2e\" declares no IDs — tests/e2e/ is not a citable home"
    } else {
        "error: path subjects are not accepted in phase 1"
    };
    let absent = refused(&repo.rule(&sentence), prefix);

    // Prove the proposed configuration's existing semantics before checking
    // the missing hint, so the red test cannot hide a broken fixture.
    let obligation = format!("\n[{}]\nmust = [\"FS\"]\n", table(version));
    repo.write("grund.toml", &(config(version) + &obligation));
    let check = repo.run(&["check"]);
    let report = String::from_utf8(check.stdout.clone()).unwrap();
    assert_eq!(check.status.code(), Some(1), "{check:?}");
    assert!(
        report.contains(
            "tests/e2e/uncited.rs:1: error: tests/e2e/ must cite FS (citation direction)"
        ),
        "{check:?}"
    );
    repo.write(
        "tests/e2e/uncited.rs",
        "// \u{a7}AR-demo \u{a7}FS-demo\nfn main() {}\n",
    );
    let fixed = repo.run(&["check"]);
    assert_eq!(fixed.status.code(), Some(0), "{fixed:?}");
    let present = refused(&repo.rule(&sentence), prefix);

    for (entry, error) in [("absent", absent), ("present", present)] {
        assert!(
            error.contains(&format!("[{}]", table(version))),
            "v{version}, citation entry {entry}: missing configuration table in refusal:\n{error}"
        );
        assert!(
            error.contains("must = [\"FS\"]"),
            "missing obligation: {error}"
        );
        assert!(error.contains("e2e"), "missing configured place: {error}");
        assert!(!error.contains("citable = true"), "{error}");
        // §FS-rule-refusals.sentences.rewrite.no-form.place: the row grounds
        // nothing, so the entry alone would leave an uncited file unchecked.
        assert!(
            error.contains(&format!(
                ", and {}, without which a file there that cites nothing is not checked",
                grounding(version)
            )),
            "v{version}, citation entry {entry}: missing grounding setting:\n{error}"
        );
    }
}

/// §FS-rule-refusals.sentences.rewrite.no-form.place, #908: with no grounding
/// row the hint names the grounding setting, and with one it names the entry
/// alone, since that entry then judges a file that cites nothing as well.
#[test]
fn place_hint_names_grounding_only_where_the_row_grounds_nothing() {
    for version in [1, 2] {
        let repo = Repo::new(version);
        for subject in ["Each e2e", "Each file in tests/e2e/"] {
            let sentence = format!("{subject} must cite at least one FS.");
            repo.write("grund.toml", &config(version));
            let error = stderr(&repo.rule(&sentence));
            assert!(
                error.contains(grounding(version)),
                "v{version} {subject}: {error}"
            );

            repo.write("grund.toml", &grounded_config(version));
            let output = repo.rule(&sentence);
            assert_eq!(output.status.code(), Some(2), "{output:?}");
            let error = stderr(&output);
            assert!(
                error.contains(&format!("[{}] with must = [\"FS\"]\n", table(version))),
                "v{version} {subject}: the hint ends at the entry:\n{error}"
            );
            assert!(
                !error.contains("grounding"),
                "v{version} {subject}: {error}"
            );
        }
    }
}

macro_rules! guidance_test {
    ($name:ident, $version:literal, $subject:literal) => {
        #[test]
        fn $name() {
            guidance_contract($version, $subject);
            if $subject.ends_with('/') {
                guidance_contract($version, $subject.trim_end_matches('/'));
            }
        }
    };
}

guidance_test!(v1_named_place_guidance, 1, "Each e2e");
guidance_test!(v1_path_place_guidance, 1, "Each file in tests/e2e/");
guidance_test!(v2_named_place_guidance, 2, "Each e2e");
guidance_test!(v2_path_place_guidance, 2, "Each file in tests/e2e/");

/// §FS-rule-refusals.sentences.rewrite.no-form.place: no guessed row or
/// obligation, including an ancestor containing two configured places.
#[test]
fn unrelated_ambiguous_and_non_equivalent_inputs_have_no_place_hint() {
    for version in [1, 2] {
        let repo = Repo::new(version);
        for subject in [
            "Each file in vendor/",
            "Each file in tests/",
            "Each file in tests/e2e/child/",
            "Each file in tests/e2e-other/",
            "Each file in tests/*/",
            "Each file in ./tests/e2e/",
            "Each file in tests/e2e/../e2e/",
            "Each file in /tests/e2e/",
            "Each file in requirements.md",
        ] {
            let output = repo.rule(&format!("{subject} must cite at least one FS."));
            let error = refused(&output, "error: path subjects are not accepted in phase 1");
            assert_no_hint(&error);
        }
        for predicate in [
            "must have exactly one terms chapter",
            "must be cited by at least one FS",
            "must not cite any FS",
            "must cite exactly one FS",
            "must cite at least two FS",
            "must cite at least one FS or AR",
            "must cite at least one other/FS",
            "must cite at least one UNKNOWN",
            "must cite at least one FS and must cite at least one AR",
            "should cite at least one FS",
            "must dance at least one FS",
            "must cite at least 1 FS",
        ] {
            for subject in ["Each e2e", "Each file in tests/e2e/"] {
                let output = repo.rule(&format!("{subject} {predicate}."));
                assert_eq!(output.status.code(), Some(2), "{output:?}");
                assert_no_hint(&stderr(&output));
            }
        }
        let output = repo.rule("Each E2E must cite at least one FS.");
        assert_no_hint(&refused(&output, "error: unknown kind \"E2E\""));

        for row_config in [
            config(version).replace("citable = false", "citable = true"),
            config(version).replace("citable = false", "citable = false\nscan = false"),
        ] {
            repo.write("grund.toml", &row_config);
            let output = repo.rule("Each file in tests/e2e/ must cite at least one FS.");
            assert_eq!(output.status.code(), Some(2), "{output:?}");
            let error = stderr(&output);
            assert!(
                error.starts_with("error: path subjects are not accepted in phase 1"),
                "{error}"
            );
            assert_no_hint(&error);
        }
    }
}

fn assert_no_hint(error: &str) {
    assert!(!error.contains("[rules.citations."), "{error}");
    assert!(!error.contains("[citations."), "{error}");
    assert!(!error.contains("must = ["), "{error}");
}

/// §FS-rule-refusals.sentences.order and §FS-errors.2.5.1: place guidance
/// cannot leak into object/selector errors or displace an earlier refusal.
#[test]
fn place_hint_preserves_other_surfaces_and_refusal_precedence() {
    for version in [1, 2] {
        let repo = Repo::new(version);
        for subject in ["Each e2e", "Each file in tests/e2e/"] {
            let output = repo.rule(&format!("{subject} must cite at least one FS"));
            assert_eq!(output.status.code(), Some(2), "{output:?}");
            let error = stderr(&output);
            assert!(
                error.starts_with("error: rule must end with \".\""),
                "{error}"
            );
            assert_no_hint(&error);
        }
        for (args, prefix) in [
            (
                vec!["list", "--selector", "e2e"],
                "error: kind \"e2e\" declares no IDs — tests/e2e/ is not a citable home",
            ),
            (
                vec![
                    "check",
                    "--rule",
                    "Each FS must cite at least one e2e.",
                    "--only-rule",
                ],
                "error: kind \"e2e\" declares no IDs — tests/e2e/ is not a citable home",
            ),
        ] {
            let output = repo.run(&args);
            let error = refused(&output, prefix);
            assert!(
                error.contains("kind \"e2e\" declares no IDs — tests/e2e/ is not a citable home"),
                "{error}"
            );
            assert_no_hint(&error);
        }
    }
}

/// §FS-rule-refusals.sentences.rewrite.no-form.place: rule declarations use
/// the same refusal guidance without CLI-only known-kinds lines.
#[test]
fn rule_declarations_carry_place_guidance_in_both_versions() {
    let mut missing = Vec::new();
    for version in [1, 2] {
        let repo = Repo::new(version);
        let rule_row = if version == 2 {
            "\n[schema.kinds.RULE]\nfolders = [\"docs/rules\"]\nindex = false\nform = \"rule\"\n"
        } else {
            "\n[[kinds]]\nkind = \"RULE\"\nfolder = \"docs/rules\"\nindex = false\nrules = true\n"
        };
        repo.write("grund.toml", &(config(version) + rule_row));
        for subject in ["Each e2e", "Each file in tests/e2e/"] {
            repo.write(
                "docs/rules/RULE-place.md",
                &format!("# RULE-place: {subject} must cite at least one FS.\n"),
            );
            let output = repo.run(&["check"]);
            let report = String::from_utf8(output.stdout).unwrap();
            assert_eq!(output.status.code(), Some(1), "{report}");
            assert!(report.contains("is not a valid rule: "), "{report}");
            assert!(!report.contains("known kinds:"), "{report}");
            assert!(!report.contains("accepted form"), "{report}");
            if !report.contains(&format!("[{}]", table(version)))
                || !report.contains("must = [\"FS\"]")
            {
                missing.push(format!("v{version} {subject}: {report}"));
            }
        }
    }
    assert!(
        missing.is_empty(),
        "missing place guidance:\n{}",
        missing.join("\n")
    );
}

/// §FS-rule-refusals.sentences.rewrite.no-form.place: use the actual row
/// and target, including a configured file rather than only a folder.
#[test]
fn configured_file_place_guidance_uses_actual_row_and_target() {
    let mut missing = Vec::new();
    for version in [1, 2] {
        let repo = Repo::new(version);
        let config = config(version)
            .replace(
                "folder = \"tests/integration\"",
                "file = \"tests/integration.rs\"",
            )
            .replace(
                "folders = [\"tests/integration\"]",
                "files = [\"tests/integration.rs\"]",
            );
        repo.write("grund.toml", &config);
        repo.write("tests/integration.rs", "fn main() {}\n");
        let table = table(version).replace("e2e", "integration");
        for subject in ["Each integration", "Each file in tests/integration.rs"] {
            let output = repo.rule(&format!("{subject} must cite at least one AR."));
            assert_eq!(output.status.code(), Some(2), "{output:?}");
            let error = stderr(&output);
            assert!(error.contains("known kinds: AR, FS"), "{error}");
            assert!(!error.contains("accepted form"), "{error}");
            if !error.contains(&format!("[{table}]")) || !error.contains("must = [\"AR\"]") {
                missing.push(format!("v{version} {subject}: {error}"));
            }
        }
    }
    assert!(
        missing.is_empty(),
        "missing file-place guidance:\n{}",
        missing.join("\n")
    );
}
