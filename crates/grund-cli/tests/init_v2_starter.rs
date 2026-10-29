//! The version-2 starter `grund init` writes (§FS-init.2.4.3) as the shipped CLI
//! sees it, over the sequences a single-command e2e case cannot express: a fresh
//! tree passes its gates, a second run changes nothing, the starter's effective
//! display loads back to the same settings, and no written key is a default.

use std::fs;
use std::path::Path;
use std::process::Command;

#[path = "support/init_fixture.rs"]
mod init_fixture;

use init_fixture::{manifest_dir, run_grund, workdir};

const TEMPLATES: [&str; 2] = ["software-development", "none"];

const FRESH_SCAFFOLD_CHECK_ADVICE: &str = "run `grund check` — on a fresh scaffold it warns \"nothing recognized\" until the first declaration; this is expected (exit 0), no repair needed";

/// A real, empty git repository, so the commands follow the fresh-setup path.
fn fresh_git_repo(case: &str) -> std::path::PathBuf {
    let target = workdir(case);
    let git = Command::new("git")
        .args(["init", "-q"])
        .current_dir(&target)
        .output()
        .expect("git init");
    assert!(git.status.success(), "git init: {git:?}");
    target
}

/// Observe the actual warning before judging the advice that predicted it.
fn fresh_scaffold_warning(target: &Path) -> String {
    let check = run_grund(&["check"], target);
    let stdout = String::from_utf8_lossy(&check.stdout);
    let stderr = String::from_utf8_lossy(&check.stderr);
    assert_eq!(
        check.status.code(),
        Some(0),
        "grund check: {stdout}{stderr}"
    );
    assert!(stdout.is_empty(), "grund check stdout: {stdout}");
    assert!(
        stderr.starts_with("warning: nothing recognized — grund read ")
            && stderr.contains("found no declaration and no citation"),
        "fresh scaffold must trigger the existing warning: {stderr}"
    );
    stderr.into_owned()
}

fn assert_check_advice(report: &str, step: usize, warning: &str) {
    let expected = format!("  {step}. {FRESH_SCAFFOLD_CHECK_ADVICE}\n");
    assert!(
        report.contains(&expected),
        "init's step {step} must explain the expected fresh-scaffold warning\n\
         expected: {expected}\
         init stderr:\n{report}\
         grund check (exit 0) stderr:\n{warning}"
    );
    assert!(
        !report.contains("scaffolded tree is clean"),
        "init advice must not promise a clean verdict: {report}"
    );
}

/// §FS-init.2.2.2, §FS-init-fixtures.2: scaffold advice agrees with the actual
/// successful check's expected warning (§FS-check.4.5), before grounding begins.
#[test]
fn scaffold_check_advice_explains_the_expected_warning() {
    let target = fresh_git_repo("scaffold_check_advice_explains_the_expected_warning");
    let init = run_grund(
        &["init", "--scaffold", "--name", "demo", "--agents-md"],
        &target,
    );
    assert_eq!(init.status.code(), Some(0), "init: {init:?}");
    assert!(init.stdout.is_empty(), "init: {init:?}");
    let warning = fresh_scaffold_warning(&target);
    assert_check_advice(&String::from_utf8_lossy(&init.stderr), 1, &warning);
}

/// §FS-init.2.2.2, §FS-init-fixtures.1: non-scaffold advice predicts the same
/// expected warning when its re-run instruction is followed on a fresh tree.
#[test]
fn non_scaffold_check_advice_explains_the_expected_warning() {
    let target = fresh_git_repo("non_scaffold_check_advice_explains_the_expected_warning");
    let init = run_grund(&["init", "--name", "demo", "--agents-md"], &target);
    assert_eq!(init.status.code(), Some(0), "init: {init:?}");
    assert!(init.stdout.is_empty(), "init: {init:?}");
    let report = String::from_utf8_lossy(&init.stderr);
    assert!(
        report
            .contains("  1. re-run with --scaffold to scaffold the FS home (docs/functional-spec)"),
        "init: {report}"
    );
    let scaffold = run_grund(&["init", "--scaffold"], &target);
    assert_eq!(scaffold.status.code(), Some(0), "scaffold: {scaffold:?}");
    let warning = fresh_scaffold_warning(&target);
    assert_check_advice(&report, 2, &warning);
}

fn init(target: &Path, flags: &[&str]) -> (bool, String) {
    let mut args = vec!["init", target.to_str().unwrap(), "--name", "demo"];
    args.extend_from_slice(flags);
    let output = run_grund(&args, manifest_dir());
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// `grund config show` over `target`: its success and its stdout.
fn show(target: &Path) -> (bool, String) {
    let output = run_grund(
        &["config", "show", target.to_str().unwrap()],
        manifest_dir(),
    );
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
    )
}

/// A fresh starter for `template`, asserted to be version 2 (§FS-init.2.4.3) and to
/// open with the comment of §FS-config.4.2.3.1.
fn fresh_starter(case: &str, template: &str) -> (std::path::PathBuf, String) {
    let target = workdir(case);
    let (ok, stderr) = init(&target, &[&format!("--template={template}")]);
    assert!(ok, "{stderr}");
    let config = fs::read_to_string(target.join("grund.toml")).unwrap();
    let head = "# Only non-defaults; `grund config show` prints what applies, `grund config explain` every setting.\n\
                grund_config_version = 2\nproject_name = \"demo\"\n";
    assert!(config.starts_with(head), "{template} starter:\n{config}");
    (target, config)
}

/// Every file under `root`, relative and sorted, with its bytes.
fn tree(root: &Path) -> Vec<(String, Vec<u8>)> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) {
        for entry in fs::read_dir(dir).expect("read dir") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                walk(root, &path, out);
            } else {
                let rel = path.strip_prefix(root).expect("under root");
                out.push((rel.display().to_string(), fs::read(&path).expect("read")));
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out.sort();
    out
}

/// §FS-init-fixtures.2: a fresh `--scaffold` tree is version 2 and passes `grund
/// check` and `grund init --check` unmodified; so does a fresh `none` tree, whose
/// check still teaches the open step of declaring a kind (§FS-init.2.2.2.2).
#[test]
fn a_fresh_v2_tree_passes_check_and_init_check() {
    let scaffold = workdir("init_v2_scaffold_gates");
    let (ok, stderr) = init(&scaffold, &["--scaffold"]);
    assert!(ok, "{stderr}");
    let (none, _) = fresh_starter("init_v2_none_gates", "none");
    for (target, expected) in [
        (
            &scaffold,
            "exists AGENTS.md\nexists grund.toml\n".to_string(),
        ),
        (&none, rerun_of("none")),
    ] {
        let config = fs::read_to_string(target.join("grund.toml")).unwrap();
        assert!(config.contains("\ngrund_config_version = 2\n"), "{config}");
        let check = run_grund(&["check", target.to_str().unwrap()], manifest_dir());
        assert!(
            check.status.success(),
            "grund check: {}{}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );
        let (ok, stderr) = init(target, &["--check"]);
        assert!(ok, "grund init --check: {stderr}");
        assert_eq!(stderr, expected);
    }
}

/// What a re-run that changes nothing prints over a fresh `template` starter: both
/// files `exists`, and the `next:` block while the step it teaches is still open
/// (§FS-init.2.2.2): neither starter has its selected home on disk.
fn rerun_of(template: &str) -> String {
    let steps = match template {
        "none" => [
            "declare a first kind in grund.toml, then run `grund init` again so AGENTS.md lists it",
            "run `grund check`",
            "allocate an ID:  ID=$(grund id <KIND> \"…\")  then add it to that kind's home",
        ],
        _ => [
            "re-run with --scaffold to scaffold the FS home (docs/functional-spec), docs/, and tests/ (or create them yourself)",
            FRESH_SCAFFOLD_CHECK_ADVICE,
            "allocate an ID:  ID=$(grund id FS \"…\")  then add it under docs/functional-spec",
        ],
    };
    format!(
        "exists AGENTS.md\nexists grund.toml\n\nnext:\n  1. {}\n  2. {}\n  3. {}\n\
         see AGENTS.md for the full workflow.\n",
        steps[0], steps[1], steps[2]
    )
}

/// §FS-init.3.3, §FS-init-fixtures.1: a second `init` over a fresh starter reports
/// `exists` for both files and leaves the tree byte-identical; neither starter has
/// its selected home on disk, so it still prints the `next:` block.
#[test]
fn a_second_init_over_the_starter_changes_nothing() {
    for template in TEMPLATES {
        let (target, _) = fresh_starter(&format!("init_v2_repeat_{template}"), template);
        let before = tree(&target);
        let (ok, stderr) = init(&target, &[]);
        assert!(ok, "{stderr}");
        assert_eq!(stderr, rerun_of(template), "{template}");
        assert_eq!(
            tree(&target),
            before,
            "{template}: the second run changed the tree"
        );
    }
}

/// §FS-config.4.2, §FS-init.2.4.3: what `config show` prints for the starter,
/// written back as a config, shows the same settings again.
#[test]
fn the_starter_shown_loads_back_to_the_same_settings() {
    for template in TEMPLATES {
        let (target, _) = fresh_starter(&format!("init_v2_shown_{template}"), template);
        let (ok, shown) = show(&target);
        assert!(ok, "{template}: {shown}");
        assert!(shown.starts_with("grund_config_version = 2\n"), "{shown}");
        let reread = workdir(&format!("init_v2_shown_{template}_reread"));
        fs::write(reread.join("grund.toml"), &shown).unwrap();
        let (ok, again) = show(&reread);
        assert!(ok, "{template}: {again}");
        assert_eq!(
            again, shown,
            "{template}: the display does not load back to itself"
        );
    }
}

/// §FS-init.2.4.3: the starter is minimal — taking out any key it writes changes
/// the effective configuration `config show` prints (or makes the file refused).
#[test]
fn every_key_the_starter_writes_changes_the_effective_config() {
    for template in TEMPLATES {
        let (target, config) = fresh_starter(&format!("init_v2_minimal_{template}"), template);
        let (ok, baseline) = show(&target);
        assert!(ok, "{template}: {baseline}");
        let lines: Vec<&str> = config.lines().collect();
        let mut defaults = Vec::new();
        for (index, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('[') {
                continue;
            }
            let without: Vec<&str> = lines
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != index)
                .map(|(_, l)| *l)
                .collect();
            let probe = workdir(&format!("init_v2_minimal_{template}_probe"));
            fs::write(probe.join("grund.toml"), without.join("\n") + "\n").unwrap();
            let (ok, shown) = show(&probe);
            if ok && shown == baseline {
                defaults.push(*line);
            }
        }
        assert!(
            defaults.is_empty(),
            "{template}: these written keys are defaults, removing them changes nothing: {defaults:#?}"
        );
    }
}
