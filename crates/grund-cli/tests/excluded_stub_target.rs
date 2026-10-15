//! Excluded or ignored stub targets are broken on every surface
//! (§FS-declarations.stubs.target.3). Git-backed tests exercise ignore rules
//! that a committed fixture cannot carry.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const CONFIG: &str = "grund_config_version = 2\n\n\
     [schema]\nid_format = \"{kind}-{slug}\"\n\n\
     [schema.sources]\nrespect_gitignore = true\n\n\
     [schema.kinds.FS]\nfolders = [\"docs/fs\"]\nindex = false\n\n\
     [schema.kinds.code]\nfolders = [\"docs\", \"notes\"]\n";

/// The cases about `[scan] include` and `exclude`, version 1 keys, stay on version 1;
/// they read only stdout, where its caution never lands (§FS-check.checks.config-version-1).
const V1: &str = "grund_config_version = 1\n\n\
     [reference]\nstrict = true\nrequire_grounding = false\n\n\
     [id]\nformat = \"{kind}-{slug}\"\n\n\
     [[kinds]]\nkind = \"FS\"\nfolder = \"docs/fs\"\nindex = false\n\n\
     [scan]\ninclude = [\"docs\", \"notes\"]\nrespect_gitignore = true\n";

/// The tree of the ticket's example: `docs/fs/x.md` stubs `notes/gen/a.md`, which
/// declares `FS-x`, and `docs/refs.md` cites it so nothing else is reported.
fn tree(name: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/excluded-stub-target")
        .join(format!("{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create fixture root");
    let git = Command::new("git")
        .args(["init", "-q", "."])
        .current_dir(&root)
        .output()
        .expect("spawn git init");
    assert!(git.status.success(), "git init {}", root.display());
    write(&root, "grund.toml", CONFIG);
    write(&root, "docs/fs/refs.md", "Keeps \u{a7}FS-x live.\n");
    write(&root, "docs/fs/x.md", "# FS-x: [x](../../notes/gen/a.md)\n");
    write(&root, "notes/gen/a.md", "# FS-x: X\n\nLead.\n");
    root
}

fn write(root: &Path, relative: &str, body: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().expect("parent")).expect("create fixture parent");
    fs::write(path, body).expect("write fixture");
}

fn grund(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .env("GIT_CONFIG_GLOBAL", root.join("xdg/git/config"))
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("XDG_CONFIG_HOME", root.join("xdg"))
        .current_dir(root)
        .output()
        .expect("run grund")
}

fn check(root: &Path, extra: &[&str]) -> Output {
    let mut args = vec!["check"];
    args.extend_from_slice(extra);
    grund(root, &args)
}

fn refusal(ignore_file: &str, pattern: &str) -> String {
    format!(
        "docs/fs/x.md:1: error: stub link target ../../notes/gen/a.md is not scanned: \
         {ignore_file} ignores it by {pattern}; this became an error in grund 0.19.0 — \
         move the target out of what {ignore_file} ignores, or stop ignoring it there\n"
    )
}

/// §FS-declarations.stubs.target.3: the former warning is now an actionable error.
#[test]
fn a_git_ignored_target_errors_at_the_stub() {
    let root = tree("gitignore");
    write(&root, ".gitignore", "notes/gen/\n");
    let output = check(&root, &[]);
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        refusal(".gitignore", "notes/gen/"),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "an ignored declaring target is broken"
    );
    let _ = fs::remove_dir_all(root);
}

/// `.git/info/exclude` is an ignore file too, and is named by its path from the
/// config root.
#[test]
fn a_target_in_git_info_exclude_names_that_file() {
    let root = tree("info-exclude");
    write(&root, ".git/info/exclude", "gen/\n");
    let output = check(&root, &[]);
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        refusal(".git/info/exclude", "gen/"),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "an ignored declaring target is broken"
    );
    let _ = fs::remove_dir_all(root);
}

/// With `respect_gitignore = false` the ignore files are no rule of the walk, so they
/// are no reason either: the run is silent.
#[test]
fn ignore_files_are_no_reason_when_the_walk_does_not_respect_them() {
    let root = tree("no-respect");
    write(&root, ".gitignore", "notes/gen/\n");
    write(
        &root,
        "grund.toml",
        &CONFIG.replace("respect_gitignore = true", "respect_gitignore = false"),
    );
    let output = check(&root, &[]);
    assert_eq!(String::from_utf8_lossy(&output.stdout), "success\n");
    assert_eq!(output.status.code(), Some(0));
    let _ = fs::remove_dir_all(root);
}

/// Port of triage's nine-case probe (§FS-declarations.stubs.target.3).
/// Compare check and show together so a warning cannot mask body leakage.
fn contract(root: &Path, broken: bool) {
    let checked = check(root, &["--format", "json"]);
    let shown = grund(root, &["show", "FS-x"]);
    let findings: Vec<serde_json::Value> = String::from_utf8_lossy(&checked.stdout)
        .lines()
        .map(|line| serde_json::from_str(line).expect("JSON diagnostic"))
        .collect();
    let codes: Vec<_> = findings
        .iter()
        .map(|f| (f["code"].clone(), f["severity"].clone()))
        .collect();
    let expected = if broken {
        vec![(serde_json::json!("broken-stub"), serde_json::json!("error"))]
    } else {
        vec![]
    };
    assert_eq!(
        (codes, checked.status.code(), shown.status.code()),
        (expected, Some(i32::from(broken)), Some(i32::from(broken))),
        "check: {}; show: {}{}",
        String::from_utf8_lossy(&checked.stdout),
        String::from_utf8_lossy(&shown.stdout),
        String::from_utf8_lossy(&shown.stderr)
    );
    if broken {
        assert_eq!(findings[0]["path"], "docs/fs/x.md");
        assert_eq!(findings[0]["line"], 1);
        assert!(!String::from_utf8_lossy(&shown.stdout).contains("Lead."));
        assert!(String::from_utf8_lossy(&shown.stderr).contains("broken stub: FS-x"));
    } else {
        assert!(String::from_utf8_lossy(&shown.stdout).contains("Lead."));
    }
}

fn outside(root: &Path, scan: &str) {
    write(
        root,
        "grund.toml",
        &V1.replace(
            "include = [\"docs\", \"notes\"]",
            &format!("include = [\"docs\"]\n{scan}"),
        ),
    );
}

#[test]
fn v1_exclude_breaks_an_outside_target_and_refuses_show() {
    let root = tree("v1-exclude");
    outside(&root, "exclude = [\"gen\"]");
    contract(&root, true);
}

#[test]
fn gitignore_breaks_an_outside_target_and_refuses_show() {
    let root = tree("outside-gitignore");
    outside(&root, "");
    write(&root, ".gitignore", "notes/gen/\n");
    contract(&root, true);
}

#[test]
fn ignore_breaks_an_outside_target_and_refuses_show() {
    let root = tree("outside-ignore");
    outside(&root, "");
    write(&root, ".ignore", "notes/gen/\n");
    contract(&root, true);
}

#[test]
fn outside_include_alone_keeps_the_target_readable() {
    let root = tree("outside-only");
    outside(&root, "");
    contract(&root, false);
}

#[test]
fn an_explicit_root_outruns_exclude() {
    let root = tree("root-outruns");
    write(
        &root,
        "grund.toml",
        &V1.replace(
            "include = [\"docs\", \"notes\"]",
            "include = [\"docs\", \"notes/gen\"]\nexclude = [\"gen\"]",
        ),
    );
    contract(&root, false);
}

#[test]
fn disabled_ignore_keeps_an_outside_target_readable() {
    let root = tree("outside-disabled");
    outside(&root, "");
    let config = fs::read_to_string(root.join("grund.toml")).unwrap();
    write(
        &root,
        "grund.toml",
        &config.replace("respect_gitignore = true", "respect_gitignore = false"),
    );
    write(&root, ".gitignore", "notes/gen/\n");
    contract(&root, false);
}

const V2: &str = "grund_config_version = 2\nproject_name = \"probe\"\n\
    [schema.sources]\nlanguages = [\"markdown\"]\nexclude = [\"gen\"]\n\
    [schema.kinds.FS]\nfolders = [\"docs/fs\"]\nindex = false\n";

#[test]
fn v2_exclude_breaks_the_target_and_refuses_show() {
    let root = tree("v2-exclude");
    write(&root, "grund.toml", V2);
    contract(&root, true);
}

#[test]
fn a_v2_explicit_root_outruns_exclude() {
    let root = tree("v2-root");
    write(
        &root,
        "grund.toml",
        &format!("{V2}\n[schema.kinds.code]\nfolders = [\"notes/gen\"]\n"),
    );
    contract(&root, false);
}

#[test]
fn an_already_broken_target_keeps_its_error_and_show_refusal() {
    let root = tree("already-broken");
    outside(&root, "exclude = [\"gen\"]");
    write(&root, "notes/gen/a.md", "# FS-y: Other\n");
    contract(&root, true);
}

/// §FS-declarations.checks.excluded-stub-target: selectors keep parsing but the
/// retired code cannot select or suppress the replacement error.
#[test]
fn spent_warning_only_selects_no_findings() {
    let root = tree("spent-selectors");
    outside(&root, "exclude = [\"gen\"]");
    let only = check(
        &root,
        &["--only", "excluded-stub-target", "--format", "json"],
    );
    assert!(only.status.success());
    assert!(
        only.stdout.is_empty(),
        "spent selector: {}",
        String::from_utf8_lossy(&only.stdout)
    );
}

/// §FS-declarations.checks.excluded-stub-target: ignoring the spent code cannot
/// suppress an error with the replacement code.
#[test]
fn spent_warning_ignore_leaves_the_replacement_visible() {
    let root = tree("spent-ignore");
    outside(&root, "exclude = [\"gen\"]");
    let ignored = check(
        &root,
        &["--ignore", "excluded-stub-target", "--format", "json"],
    );
    assert_eq!(
        ignored.status.code(),
        Some(1),
        "replacement remains visible"
    );
    assert!(String::from_utf8_lossy(&ignored.stdout).contains("\"code\":\"broken-stub\""));
}

/// §FS-declarations.stubs.target.3: global git ignores use the same matcher,
/// and name the absolute ignore file plus the authored pattern. The pattern is
/// unanchored because the walk roots the global matcher at the working directory,
/// which on Windows never carries the `\\?\` spelling of the canonical paths it walks.
#[test]
fn a_global_ignore_refuses_and_names_its_file() {
    let root = tree("global-ignore").canonicalize().unwrap();
    let global = root.with_extension("ignore");
    fs::write(&global, "gen/\n").unwrap();
    write(
        &root,
        "xdg/git/config",
        &format!("[core]\nexcludesFile = {}\n", global.display()),
    );
    let output = check(&root, &[]);
    // §FS-errors.4: the report spells the file with `/` on every platform.
    let file = global.to_string_lossy().replace('\\', "/");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        refusal(&file, "gen/")
    );
    assert_eq!(output.status.code(), Some(1));
}

/// §FS-declarations.stubs.target.3: first rule descending, exclude before ignore
/// at one step, and name/extension refusal before either.
#[test]
fn the_first_rule_descending_is_reported_under_full() {
    let root = tree("precedence");
    outside(&root, "exclude = [\"gen\"]");
    write(&root, ".ignore", "notes/\n");
    let first = check(&root, &["--full"]);
    assert!(
        String::from_utf8_lossy(&first.stdout)
            .contains("is not scanned: .ignore ignores it by notes/; this became an error"),
        "{}",
        String::from_utf8_lossy(&first.stdout)
    );
}

/// §FS-declarations.stubs.target.3: exclude wins over ignore at the same step;
/// reporting a path does not change target eligibility.
#[test]
fn exclude_precedes_ignore_at_the_same_step_in_a_narrowed_report() {
    let root = tree("same-step");
    outside(&root, "exclude = [\"gen\"]");
    write(&root, ".ignore", "notes/gen/\n");
    let same_step = check(&root, &["docs/fs/x.md"]);
    assert!(
        String::from_utf8_lossy(&same_step.stdout)
            .contains("is not scanned: [scan] exclude names gen; this became an error"),
        "{}",
        String::from_utf8_lossy(&same_step.stdout)
    );
}

/// §FS-declarations.stubs.target.3: name and extension refusal keep precedence
/// over a simultaneously matching exclude and ignore rule.
#[test]
fn name_and_extension_refusals_precede_exclusion_and_ignore() {
    let root = tree("name-precedence");
    outside(&root, "exclude = [\"gen\"]");
    write(&root, ".ignore", "notes/gen/\n");
    write(&root, "notes/gen/a.zz", "# FS-x: X\n");
    write(&root, "docs/fs/x.md", "# FS-x: [x](../../notes/gen/a.zz)\n");
    let extension = check(&root, &[]);
    let text = String::from_utf8_lossy(&extension.stdout);
    assert!(
        text.contains("extension zz is not in [scan] extensions"),
        "{text}"
    );
    assert!(!text.contains("this became an error"));
    write(&root, "notes/gen/.a.zz", "# FS-x: X\n");
    write(
        &root,
        "docs/fs/x.md",
        "# FS-x: [x](../../notes/gen/.a.zz)\n",
    );
    let hidden = check(&root, &[]);
    let text = String::from_utf8_lossy(&hidden.stdout);
    assert!(text.contains("stub link target lacks FS-x"), "{text}");
    assert!(!text.contains("is not scanned"));
}

/// §FS-declarations.stubs.target.3: nested ignore negation can keep the declaring
/// target readable, and turning ignore off must not turn exclude off.
#[test]
fn nested_negation_and_disabled_ignore_preserve_exclude() {
    let root = tree("negation");
    outside(&root, "");
    write(&root, ".ignore", "notes/gen/*.md\n");
    write(&root, "notes/.ignore", "!gen/\n!gen/a.md\n");
    contract(&root, false);
    outside(&root, "exclude = [\"gen\"]");
    let config = fs::read_to_string(root.join("grund.toml")).unwrap();
    write(
        &root,
        "grund.toml",
        &config.replace("respect_gitignore = true", "respect_gitignore = false"),
    );
    contract(&root, true);
}

/// §FS-declarations.stubs.target.3: an explicit scan root also outruns ignore.
#[test]
fn an_explicit_root_outruns_ignore() {
    let root = tree("root-outruns-ignore");
    write(
        &root,
        "grund.toml",
        &V1.replace(
            "include = [\"docs\", \"notes\"]",
            "include = [\"docs\", \"notes/gen\"]",
        ),
    );
    write(&root, ".gitignore", "notes/gen/\n");
    contract(&root, false);
}
