//! A project cites a sibling project its build places beside it, read live from that tree
//! (§FS-imports). The runners below name the import cases of `tests/e2e/cases/` by the
//! contract each pins, and the rest build what a committed case cannot hold.
#![allow(dead_code)]
#[path = "support/case_runner.rs"]
mod case_runner;

use case_runner::CaseKind::E2e;
use case_runner::{assert_every_case_passed, run_case};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn run(label: &str, names: &[&str]) {
    let root = repo_root();
    let cases = root.join("tests/e2e/cases");
    let outcomes = names
        .iter()
        .map(|name| run_case(&root, &cases.join(name), E2e))
        .collect::<Vec<_>>();
    assert_every_case_passed(label, &outcomes);
}

const HEADLINE: &[&str] = &[
    "imports-sibling-check",
    "imports-sibling-check-json",
    "imports-sibling-show-brief",
    "imports-sibling-list",
    "imports-sibling-list-json",
    "imports-sibling-refs",
    "imports-sibling-refs-json",
    "imports-sibling-cover-json",
    "imports-sibling-fmt-write",
    "imports-sibling-fmt-check",
    "imports-sibling-init",
];

/// The headline: a consumer cites a sibling it imports, and every surface answers
/// through it (§FS-imports.declaration.table, §FS-imports.mount,
/// §FS-imports.checks.import-live), reporting the imported files where they are
/// (§FS-imports.paths), counting them (§FS-imports.findings), and writing only the
/// consumer's own files (§FS-imports.writes).
#[test]
fn imports_headline_surfaces_contract() {
    run("imports headline", HEADLINE);
}

const GRAPH: &[&str] = &[
    "imports-alias-differs-check",
    "imports-alias-differs-refs-json",
    "imports-nested-member-show",
    "imports-transitive-check",
    "imports-transitive-show",
    "imports-same-name-two-trees-check",
    "imports-same-name-two-trees-list-json",
    "imports-diamond-check",
    "imports-diamond-list-json",
    "imports-diamond-show",
    "imports-diamond-refs-json",
    "imports-diamond-cover-json",
    "imports-diamond-refs-glob",
    "imports-diamond-cover-glob-json",
    "imports-member-and-import-check",
    "imports-cycle",
];

/// The alias is the consumer's name for the target (§FS-imports.declaration.alias);
/// imports nest beneath their importer (§FS-imports.graph.transitive), a cycle is a
/// config error (§FS-imports.graph.cycle), and one tree reached under two alias paths
/// is one tree, reported once under its canonical alias path (§FS-imports.graph.diamond).
#[test]
fn imports_alias_and_graph_contract() {
    run("imports graph", GRAPH);
}

const LOCATED: &[&str] = &[
    "imports-missing-check",
    "imports-missing-check-json",
    "imports-missing-only-dangling",
    "imports-missing-show",
    "imports-missing-fmt-keeps-section-locks",
    "imports-live-ignore-selector",
    "imports-live-only-selector",
    "imports-missing-config",
    "imports-malformed-target",
    "imports-unsupported-target-version",
    "imports-wrong-identity",
    "imports-no-project-name",
    "imports-unknown-key",
    "imports-missing-key",
    "imports-not-a-string",
    "imports-collision-member",
    "imports-collision-optional-member",
    "imports-collision-root",
    "imports-path-self",
    "imports-path-ancestor",
    "imports-path-inside",
    "imports-path-absolute",
    "imports-path-backslash",
    "imports-target-claimed-check",
    "imports-v1-target-check",
    "imports-agents-location-check",
    "imports-target-agents-location-check",
];

/// Every outcome is located: a missing import is one unhideable error at its table
/// line (§FS-imports.checks.import-missing), and every config failure names the
/// line that causes it (§FS-imports.failures) — the path (§FS-imports.declaration.path),
/// the target's own root (§FS-imports.declaration.root), its project
/// (§FS-imports.declaration.identity), and the dependency's own findings and cautions
/// stay its own (§FS-imports.findings).
#[test]
fn imports_located_outcomes_contract() {
    run("imports located outcomes", LOCATED);
}

const UNCHANGED: &[&str] = &[
    "imports-none-matching-sibling",
    "imports-none-malformed-sibling",
    "imports-none-malformed-sibling-json",
    "imports-dependency-standalone",
    "imports-v1-consumer-refused",
    "imports-v1-consumer-migrate-refused",
    "imports-adopted-block-init-check",
    "imports-adopted-block-check",
];

/// What an import leaves as it was: a config with no import reads no sibling, however
/// its `project_name` or its config reads (§FS-imports.declaration.none); the
/// dependency run on its own never learns its consumer (§FS-imports.graph.reverse); a
/// version-1 consumer keeps today's refusal of the table; and a managed block rendered
/// before an import was adopted stays current (§FS-imports.declaration.table).
#[test]
fn imports_unchanged_ground_contract() {
    run("imports unchanged ground", UNCHANGED);
}

/// No import case of the corpus is left out of the runners above, so each stays named
/// by the contract it pins (§FS-examples.5).
#[test]
fn every_import_case_is_run() {
    let cases = fs::read_dir(repo_root().join("tests/e2e/cases"))
        .expect("read tests/e2e/cases")
        .map(|entry| entry.expect("case entry").path())
        .filter(|path| path.is_dir())
        .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
        .filter(|name| name.starts_with("imports-"))
        .collect::<BTreeSet<_>>();
    let run = HEADLINE
        .iter()
        .chain(GRAPH)
        .chain(LOCATED)
        .chain(UNCHANGED)
        .map(|name| name.to_string())
        .collect::<BTreeSet<_>>();
    assert_eq!(cases, run, "an import case no runner names, or the reverse");
}

// The fixtures the cases below build by hand, because each needs something a
// committed case cannot hold: a `.git` directory, a moved root, a link, an edit.

const GRAAL: &str = "grund_config_version = 2\nproject_name = \"native-image\"\n\n\
    [schema.kinds.FS]\nfolders = [\"docs/functional-spec\"]\n";
const CONSUMER: &str = "grund_config_version = 2\nproject_name = \"enterprise\"\n\n\
    [workspace.imports.native-image]\npath = \"../graal/substratevm\"\nproject = \"native-image\"\n";
const PROVIDERS: &str = "# FS-security-providers: Security providers are registered at build time\n\n\
    Providers are registered at image build time.\n";
const EXAMPLE: &str = "// Registers providers as \u{a7}native-image/FS-security-providers requires.\nclass Example {}\n";
const LIVE: &str = "grund.toml:4: warning: import `native-image` is read live from `../graal/substratevm`, unpinned\n";
static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Tree(PathBuf);

impl Tree {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "grund-imports-{name}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create fixture root");
        Self(root.canonicalize().expect("canonical fixture root"))
    }

    /// `checkout/graal/substratevm` and `checkout/enterprise`, nothing above them.
    fn sibling(name: &str) -> Self {
        let tree = Self::new(name);
        tree.write("graal/substratevm/grund.toml", GRAAL);
        tree.write(
            "graal/substratevm/docs/functional-spec/FS-security-providers.md",
            PROVIDERS,
        );
        tree.write("enterprise/grund.toml", CONSUMER);
        tree.write("enterprise/src/Example.java", EXAMPLE);
        tree
    }

    fn write(&self, relative: &str, body: &str) {
        let path = self.0.join(relative);
        fs::create_dir_all(path.parent().expect("parent")).expect("create fixture parent");
        fs::write(path, body).expect("write fixture");
    }

    /// A command with no Git configuration but the fixture's own.
    fn command(&self, program: &str, cwd: &str) -> Command {
        let mut command = Command::new(program);
        command
            .current_dir(self.0.join(cwd))
            .env("HOME", self.0.join("home"))
            .env("USERPROFILE", self.0.join("home"))
            // Git joins `/git/ignore` onto it, which a verbatim Windows root cannot take.
            .env("XDG_CONFIG_HOME", git_path(&self.0.join("xdg")))
            .env("GIT_CONFIG_GLOBAL", self.0.join("xdg/git/config"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CEILING_DIRECTORIES", &self.0)
            .env_remove("GIT_CONFIG")
            .env_remove("GIT_CONFIG_SYSTEM")
            .env_remove("GIT_CONFIG_COUNT")
            .env_remove("GIT_CONFIG_PARAMETERS")
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE");
        command
    }

    fn grund(&self, cwd: &str, args: &[&str]) -> Output {
        self.command(env!("CARGO_BIN_EXE_grund"), cwd)
            .args(args)
            .output()
            .expect("run grund")
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// `path` spelled as Git takes a path from its environment: forward slashes, and no
/// `\\?\` prefix, inside which a `/` Git appends would not separate anything.
fn git_path(path: &Path) -> String {
    let path = path.to_string_lossy().replace('\\', "/");
    path.strip_prefix("//?/").unwrap_or(&path).to_string()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn assert_run(output: &Output, code: i32, stdout: &str, stderr: &str) {
    assert_eq!(
        (
            output.status.code(),
            text(&output.stdout),
            text(&output.stderr)
        ),
        (Some(code), stdout.to_string(), stderr.to_string())
    );
}

/// An import is walked with the ignore files at and below its root and none above
/// it: not its repository's `.gitignore` above the target, not `.git/info/exclude`,
/// not the global excludes file, and not the consumer's (§FS-imports.mount).
#[test]
fn ignore_files_above_the_target_hide_nothing_of_it() {
    let tree = Tree::sibling("ignore");
    for id in ["FS-a", "FS-b", "FS-c"] {
        tree.write(
            &format!("graal/substratevm/docs/functional-spec/{id}.md"),
            &format!("# {id}: Hidden by its own repository\n"),
        );
    }
    let init = tree
        .command("git", "graal")
        .args(["init", "-q", "."])
        .output();
    assert!(init.expect("git init").status.success());
    tree.write(
        "graal/.gitignore",
        "substratevm/docs/functional-spec/FS-a.md\n",
    );
    tree.write("graal/.git/info/exclude", "FS-b.md\n");
    tree.write("xdg/git/ignore", "FS-c.md\n");
    tree.write("enterprise/.gitignore", "*.md\n");
    tree.write(
        "enterprise/src/Example.java",
        "// \u{a7}native-image/FS-a \u{a7}native-image/FS-b \u{a7}native-image/FS-c\n",
    );
    for id in ["FS-a", "FS-b", "FS-c"] {
        let path = format!("substratevm/docs/functional-spec/{id}.md");
        let ignored = tree
            .command("git", "graal")
            .args(["check-ignore", "-q", &path])
            .status();
        assert!(
            ignored.expect("git check-ignore").success(),
            "{path} is not ignored"
        );
    }
    // The dependency's own run hides all three: that is the premise.
    let own = text(&tree.grund("graal/substratevm", &["list"]).stdout);
    assert!(
        ["FS-a", "FS-b", "FS-c"].iter().all(|id| !own.contains(id)),
        "{own}"
    );
    assert_run(&tree.grund("enterprise", &["check"]), 0, LIVE, "");
}

/// A sibling layout whose dependency fetches `TICKET` IDs through a script that would
/// leave a mark if it ran (§FS-fetch.2).
#[cfg(unix)]
fn fetching_sibling(name: &str) -> Tree {
    use std::os::unix::fs::PermissionsExt;
    let tree = Tree::sibling(name);
    tree.write(
        "graal/substratevm/grund.toml",
        &format!(
            "{GRAAL}\n[schema.kinds.TICKET]\nfiles = [\"docs/tickets.md\"]\n\
             id_format = \"{{kind}}-{{number}}\"\nfetch = \"scripts/fetch-ticket\"\n"
        ),
    );
    let script = tree.0.join("graal/substratevm/scripts/fetch-ticket");
    tree.write(
        "graal/substratevm/scripts/fetch-ticket",
        "#!/bin/sh\ntouch \"$(dirname \"$0\")/../FETCHED\"\n",
    );
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
    tree
}

/// `grund fetch` of an ID in an import is refused before its integration runs, naming
/// the target as the place to run it (§FS-imports.writes).
#[cfg(unix)]
#[test]
fn fetch_into_an_import_is_refused_before_its_integration_runs() {
    let tree = fetching_sibling("fetch");
    let output = tree.grund("enterprise", &["fetch", "native-image/TICKET-1"]);
    assert_run(
        &output,
        2,
        "",
        "error: `native-image/TICKET-1` is in import `native-image`, read-only here \u{2014} \
         run grund fetch TICKET-1 in ../graal/substratevm\n",
    );
    assert!(!tree.0.join("graal/substratevm/FETCHED").exists());
    assert!(!tree.0.join("graal/substratevm/docs/tickets.md").exists());
}

/// Run from the root of a workspace one of whose members declares the import, the
/// refusal names the target from where the run started, not from the member that
/// wrote the path (§FS-imports.writes, §FS-errors.4).
#[cfg(unix)]
#[test]
fn fetch_into_a_members_import_names_the_target_from_the_run() {
    let tree = fetching_sibling("fetch-member");
    tree.write(
        "grund.toml",
        "grund_config_version = 2\nproject_name = \"checkout\"\n\n\
         [workspace]\nmembers = [\"enterprise\", \"graal/substratevm\"]\ninclude_root = false\n",
    );
    let output = tree.grund(".", &["fetch", "enterprise/native-image/TICKET-1"]);
    assert_run(
        &output,
        2,
        "",
        "error: `enterprise/native-image/TICKET-1` is in import `enterprise/native-image`, \
         read-only here \u{2014} run grund fetch TICKET-1 in graal/substratevm\n",
    );
    assert!(!tree.0.join("graal/substratevm/FETCHED").exists());
}

/// A dependency defect that breaks a consumer citation fails at the consumer's site,
/// and nothing is reported inside the import (§FS-imports.findings). The wording of
/// that site's finding is the implementing change's to choose.
#[test]
fn a_duplicate_in_the_dependency_fails_at_the_consumer_site() {
    let tree = Tree::sibling("duplicate");
    tree.write(
        "graal/substratevm/docs/functional-spec/FS-security-providers-again.md",
        "# FS-security-providers: Declared twice\n",
    );
    let output = tree.grund("enterprise", &["check"]);
    let stdout = text(&output.stdout);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let lines = stdout.lines().collect::<Vec<_>>();
    assert!(lines.contains(&LIVE.trim_end()), "{stdout}");
    let site = lines
        .iter()
        .filter(|line| line.starts_with("src/Example.java:1: error:"));
    assert_eq!(
        site.filter(|line| line.contains("FS-security-providers"))
            .count(),
        1,
        "{stdout}"
    );
    assert_eq!(lines.len(), 2, "{stdout}");
}

/// `grund config show` prints the table as written, with no resolved path, and
/// `grund config explain` answers for the table and both keys
/// (§FS-imports.declaration.table).
#[test]
fn config_show_and_explain_answer_for_the_import_table() {
    let tree = Tree::sibling("config");
    let show = tree.grund("enterprise", &["config", "show"]);
    let stdout = text(&show.stdout);
    assert_eq!(show.status.code(), Some(0), "{show:?}");
    assert!(
        stdout.contains(
            "[workspace.imports.native-image]\npath = \"../graal/substratevm\"\nproject = \"native-image\"\n"
        ),
        "{stdout}"
    );
    assert!(!stdout.contains(&*tree.0.to_string_lossy()), "{stdout}");
    for (target, expected) in [
        ("workspace.imports.native-image", "project"),
        ("workspace.imports.native-image.path", "Values: string"),
        ("workspace.imports.native-image.project", "Values: string"),
    ] {
        let explain = tree.grund("enterprise", &["config", "explain", target]);
        assert_eq!(explain.status.code(), Some(0), "{target}: {explain:?}");
        assert!(
            text(&explain.stdout).contains(expected),
            "{target}: {explain:?}"
        );
    }
}

/// An import is read from the target's tree as it stands when the run reads it: one
/// byte the dependency changes changes the consumer's verdict, and changing it back
/// restores it (§FS-imports.mount).
#[test]
fn one_changed_dependency_byte_moves_the_consumer_verdict() {
    let tree = Tree::sibling("live");
    assert_run(&tree.grund("enterprise", &["check"]), 0, LIVE, "");
    let declaration = "graal/substratevm/docs/functional-spec/FS-security-providers.md";
    tree.write(
        declaration,
        &PROVIDERS.replacen("providers:", "provider:", 1),
    );
    let moved = tree.grund("enterprise", &["check"]);
    assert_eq!(moved.status.code(), Some(1), "{moved:?}");
    assert!(
        text(&moved.stdout).starts_with("src/Example.java:1: error:"),
        "{moved:?}"
    );
    assert!(text(&moved.stdout).ends_with(LIVE), "{moved:?}");
    tree.write(declaration, PROVIDERS);
    assert_run(&tree.grund("enterprise", &["check"]), 0, LIVE, "");
}

/// The same composition at two roots, one with a space and a non-ASCII name, prints
/// the same bytes: an imported file is reported relative to the report base, never
/// absolute (§FS-imports.paths, §REQ-deterministic-output.2).
#[test]
fn a_relocated_composition_prints_the_same_bytes() {
    let plain = Tree::sibling("relocated");
    let spaced = Tree::sibling("relocated \u{e9}t\u{e9} root");
    for args in [
        &["check"][..],
        &["list", "--format", "json"],
        &[
            "refs",
            "native-image/FS-security-providers",
            "--format",
            "json",
        ],
        &["cover", "--format", "json"],
    ] {
        let a = plain.grund("enterprise", args);
        let b = spaced.grund("enterprise", args);
        assert_eq!(a.status.code(), Some(0), "{args:?}: {a:?}");
        assert_eq!(
            (&a.status.code(), &a.stdout, &a.stderr),
            (&b.status.code(), &b.stdout, &b.stderr)
        );
        assert!(
            !text(&a.stdout).contains(&*plain.0.to_string_lossy()),
            "{args:?}: {a:?}"
        );
    }
    let list = plain.grund("enterprise", &["list"]);
    assert_run(
        &list,
        0,
        "native-image/FS-security-providers  \
         ../graal/substratevm/docs/functional-spec/FS-security-providers.md:1  \
         Security providers are registered at build time\n",
        "",
    );
    // A link `fmt --write` writes into the consumer climbs from the consumer's document
    // to the imported file, wherever the composition sits (§FS-imports.writes).
    let notes = "enterprise/docs/notes.md";
    let linked = "See [\u{a7}native-image/FS-security-providers](../../graal/substratevm/docs/\
        functional-spec/FS-security-providers.md#fs-security-providers-security-providers-are-\
        registered-at-build-time).\n";
    for tree in [&plain, &spaced] {
        tree.write(notes, "See \u{a7}native-image/FS-security-providers.\n");
        let fmt = tree.grund("enterprise", &["fmt", "--write"]);
        assert_run(
            &fmt,
            0,
            "rewrote 1 line: 1 link written\n  docs/notes.md (1)\n",
            "",
        );
        assert_eq!(fs::read_to_string(tree.0.join(notes)).unwrap(), linked);
    }
}

/// A symlinked `path` is followed to the directory whose identity is checked, the
/// warning names the path as the config wrote it, and two spellings of one canonical
/// root are one target, whose sites are listed once (§FS-imports.declaration.path).
#[cfg(unix)]
#[test]
fn a_symlinked_import_path_is_one_target() {
    let tree = Tree::sibling("symlink");
    fs::create_dir_all(tree.0.join("links")).unwrap();
    std::os::unix::fs::symlink("../graal/substratevm", tree.0.join("links/svm")).unwrap();
    tree.write(
        "enterprise/grund.toml",
        &format!(
            "{CONSUMER}\n[workspace.imports.svm]\npath = \"../links/svm\"\nproject = \"native-image\"\n"
        ),
    );
    tree.write(
        "enterprise/src/Other.java",
        "// \u{a7}svm/FS-security-providers\nclass Other {}\n",
    );
    let check = tree.grund("enterprise", &["check"]);
    let svm = "grund.toml:8: warning: import `svm` is read live from `../links/svm`, unpinned\n";
    assert_run(&check, 0, &format!("{LIVE}{svm}"), "");
    let refs = tree.grund("enterprise", &["refs", "svm/FS-security-providers"]);
    assert_run(
        &refs,
        0,
        "src/Example.java:1: \u{a7}native-image/FS-security-providers\n\
         src/Other.java:1: \u{a7}svm/FS-security-providers\n",
        "",
    );
}

/// A missing import is never answered by another tree: neither a sibling beside the
/// consumer nor one beside the missing path that declares the expected `project_name` and the
/// cited ID is searched, so the run fails at the import and the query names it
/// (§FS-imports.graph.reverse, §FS-imports.checks.import-missing).
#[test]
fn a_missing_import_never_falls_back_to_a_decoy_with_its_name() {
    let tree = Tree::sibling("decoy");
    fs::remove_dir_all(tree.0.join("graal/substratevm")).unwrap();
    for decoy in ["decoy", "graal/other"] {
        tree.write(&format!("{decoy}/grund.toml"), GRAAL);
        tree.write(
            &format!("{decoy}/docs/functional-spec/FS-security-providers.md"),
            PROVIDERS,
        );
    }
    let missing = "import `native-image` is missing: `../graal/substratevm` is not a directory";
    assert_run(
        &tree.grund("enterprise", &["check"]),
        1,
        &format!(
            "grund.toml:4: error: {missing} \u{2014} citations into namespace `native-image` were not checked\n"
        ),
        "",
    );
    let show = tree.grund("enterprise", &["native-image/FS-security-providers"]);
    assert_run(&show, 2, "", &format!("error: {missing}\n"));
}

/// With no import declared, nothing changes: a sibling whose `project_name` matches
/// the alias a citation spells is never searched, and the citation keeps today's
/// finding (§FS-imports.declaration.none).
#[test]
fn no_import_declared_never_reads_a_matching_sibling() {
    let tree = Tree::sibling("none");
    tree.write(
        "enterprise/grund.toml",
        "grund_config_version = 2\nproject_name = \"enterprise\"\n",
    );
    let unknown = "src/Example.java:1: error: unknown project alias native-image\n";
    assert_run(&tree.grund("enterprise", &["check"]), 1, unknown, "");
    fs::remove_dir_all(tree.0.join("graal")).unwrap();
    assert_run(&tree.grund("enterprise", &["check"]), 1, unknown, "");
}
