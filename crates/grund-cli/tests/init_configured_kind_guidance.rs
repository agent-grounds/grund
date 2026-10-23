//! Black-box guidance in a project without FS (§FS-init.2.2.2.3): port the #940
//! reproducer, a REQ-only adoption through `grund config migrate`, and pin each
//! configured-kind block §FS-init-fixtures.8 gives and the selection behind it.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

/// The reproducer's version-1 config: one kind, `REQ`, homed in `docs/req`.
const REQ_ONLY: &str = "[id]
format = \"{kind}-{slug}\"
named_sections = true

[[kinds]]
kind = \"REQ\"
folder = \"docs/req\"

[scan]
include = [\"docs\", \"README.md\"]
";

/// The reproducer's tree beside `REQ_ONLY` (§FS-init-fixtures.8).
const REQ_TREE: &[(&str, &str)] = &[
    (
        "docs/req/REQ-login.md",
        "# REQ-login: Users log in\n\nA user logs in with a name and a password.\n",
    ),
    (
        "docs/req/README.md",
        "# Requirements\n\n- [\u{a7}REQ-login](REQ-login.md): users log in\n",
    ),
    (
        "README.md",
        "# Demo\n\nLogin works as \u{a7}REQ-login says.\n",
    ),
];

const REFRESHED: &str = "updated AGENTS.md\nexists grund.toml\n";

/// §FS-init-fixtures.8.1: a first run over an existing home.
const HOME_EXISTS: &str = "wrote AGENTS.md
exists grund.toml

next:
  1. run `grund check`
  2. allocate an ID:  ID=$(grund id REQ \"…\")  then add it under docs/req
see AGENTS.md for the full workflow.
";

/// §FS-init-fixtures.8.2: the selected home is missing.
const HOME_MISSING_STEPS: &str = "
next:
  1. create the REQ home (docs/req)
  2. run `grund check`
  3. allocate an ID:  ID=$(grund id REQ \"…\")  then add it under docs/req
see AGENTS.md for the full workflow.
";

/// §FS-init-fixtures.8.3: the selected row has no home.
const NO_HOME: &str = "wrote AGENTS.md
exists grund.toml

next:
  1. run `grund check`
  2. allocate an ID:  ID=$(grund id REQ \"…\")  then declare it in a source doc-comment
see AGENTS.md for the full workflow.
";

struct Fixture(PathBuf);

impl Fixture {
    /// A Git tree at a fresh temp path holding `config` and `files`; every
    /// directory in `dirs` exists, empty or not.
    fn new(name: &str, config: &str, files: &[(&str, &str)], dirs: &[&str]) -> Self {
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("grund-940-{name}-{}-{serial}", std::process::id()));
        fs::create_dir_all(&dir).expect("create fixture");
        let git = Command::new("git")
            .args(["init", "-q"])
            .current_dir(&dir)
            .output()
            .expect("initialize fixture Git repository");
        assert!(git.status.success(), "git init: {}", text(&git.stderr));
        fs::write(dir.join("grund.toml"), config).unwrap();
        for (path, contents) in files {
            let path = dir.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, contents).unwrap();
        }
        for path in dirs {
            fs::create_dir_all(dir.join(path)).unwrap();
        }
        Self(dir)
    }

    fn req_only(name: &str) -> Self {
        Self::new(name, REQ_ONLY, REQ_TREE, &[])
    }

    fn grund(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .output()
            .expect("run grund")
    }

    /// Run `args`, require exit `code`, and return stderr.
    fn stderr_of(&self, args: &[&str], code: i32) -> String {
        let out = self.grund(args);
        let stderr = text(&out.stderr);
        assert_eq!(
            out.status.code(),
            Some(code),
            "grund {}: stdout:\n{}\nstderr:\n{stderr}",
            args.join(" "),
            text(&out.stdout)
        );
        stderr
    }

    /// Replace `from` with `to` in `grund.toml`, the way a user edits the config
    /// the block was rendered from, so the next `init` is a refresh (§FS-check.3.5).
    fn edit_config(&self, from: &str, to: &str) {
        let path = self.0.join("grund.toml");
        let config = fs::read_to_string(&path).unwrap();
        assert!(config.contains(from), "{from:?} in {config}");
        fs::write(path, config.replacen(from, to, 1)).unwrap();
    }

    fn snapshot(&self) -> Vec<(PathBuf, Vec<u8>)> {
        let mut files = Vec::new();
        let mut stack = vec![self.0.clone()];
        while let Some(dir) = stack.pop() {
            for entry in fs::read_dir(&dir).unwrap() {
                let path = entry.unwrap().path();
                if path.file_name().unwrap() == ".git" {
                    continue;
                }
                if path.is_dir() {
                    stack.push(path);
                } else {
                    files.push((path.clone(), fs::read(path).unwrap()));
                }
            }
        }
        files.sort();
        files
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// §FS-init.2.2.2.3: the project declares neither, so no guidance names them.
fn assert_names_no_fs(label: &str, stderr: &str) {
    for forbidden in ["grund id FS", "docs/functional-spec", "--scaffold"] {
        assert!(
            !stderr.contains(forbidden),
            "{label}: the guidance names `{forbidden}`, which this project does not declare:\n{stderr}"
        );
    }
}

/// The #940 reproducer: a REQ-only project adopts grund, migrates to version 2,
/// and refreshes the block `grund check` reports stale (§FS-init-fixtures.8.1).
#[test]
fn req_only_adoption_teaches_req_and_the_refresh_after_migrating_is_quiet() {
    let fixture = Fixture::req_only("migrate");
    let first = fixture.stderr_of(&["init"], 0);
    assert_names_no_fs("first init", &first);
    assert_eq!(first, HOME_EXISTS, "first init");
    fixture.stderr_of(&["check"], 0);
    fixture.stderr_of(&["config", "migrate", "--to", "2", "--write"], 0);

    let stale = fixture.grund(&["check"]);
    assert_eq!(stale.status.code(), Some(1), "{}", text(&stale.stderr));
    let report = text(&stale.stdout) + &text(&stale.stderr);
    assert!(
        report.contains("stale grund init block"),
        "check asks for the refresh: {report}"
    );

    // §FS-init.2.2.2.1: the previews see the same complete refresh and print no
    // block; --dry-run writes nothing and exits 0, a pending --check exits 1.
    let before = fixture.snapshot();
    let preview = "would-update AGENTS.md\nexists grund.toml\n";
    assert_eq!(fixture.stderr_of(&["init", "--dry-run"], 0), preview);
    assert_eq!(fixture.stderr_of(&["init", "--check"], 1), preview);
    assert_eq!(fixture.snapshot(), before, "the previews write nothing");

    assert_eq!(
        fixture.stderr_of(&["init"], 0),
        REFRESHED,
        "the refresh check asked for"
    );
    fixture.stderr_of(&["check"], 0);
    let id = fixture.grund(&["id", "REQ", "Something"]);
    assert_eq!(id.status.code(), Some(0), "{}", text(&id.stderr));
    assert_eq!(text(&id.stdout), "REQ-something\n");
}

/// §FS-init-fixtures.8.2: a missing selected home is asked for, on a first run
/// and on a refresh, and `--scaffold` is never offered for it.
#[test]
fn a_missing_home_is_asked_for_on_a_first_run_and_on_a_refresh() {
    let fixture = Fixture::new("missing", REQ_ONLY, &[REQ_TREE[2]], &["docs"]);
    let first = fixture.stderr_of(&["init"], 0);
    assert_names_no_fs("first init", &first);
    assert_eq!(
        first,
        format!("wrote AGENTS.md\nexists grund.toml\n{HOME_MISSING_STEPS}")
    );

    fixture.edit_config(
        "kind = \"REQ\"\n",
        "kind = \"REQ\"\ntitle = \"Requirements\"\n",
    );
    assert_eq!(
        fixture.stderr_of(&["init"], 0),
        format!("updated AGENTS.md\nexists grund.toml\n{HOME_MISSING_STEPS}"),
        "a refresh whose selected home is absent keeps the block"
    );
}

/// §FS-init-fixtures.8.3: a row with no home teaches a doc-comment declaration,
/// names no directory, and leaves a refresh with nothing to teach.
#[test]
fn a_row_with_no_home_teaches_a_doc_comment_and_its_refresh_is_quiet() {
    let config = REQ_ONLY.replace("folder = \"docs/req\"\n", "");
    let fixture = Fixture::new("no-home", &config, REQ_TREE, &[]);
    let first = fixture.stderr_of(&["init"], 0);
    assert_names_no_fs("first init", &first);
    assert_eq!(first, NO_HOME);

    fixture.stderr_of(&["config", "migrate", "--to", "2", "--write"], 0);
    assert_eq!(fixture.stderr_of(&["init"], 0), REFRESHED);
}

/// §FS-init-fixtures.8.4: `--scaffold` writes what it wrote before, and only the
/// allocation step names the selected kind.
#[test]
fn the_scaffold_form_names_the_selected_kind_and_writes_what_it_wrote() {
    let fixture = Fixture::req_only("scaffold");
    let stderr = fixture.stderr_of(&["init", "--scaffold"], 0);
    assert_eq!(
        stderr,
        "wrote AGENTS.md
exists grund.toml
wrote docs/functional-spec/README.md
exists docs/req/README.md
wrote docs/grund.md
wrote docs/goals.md
wrote docs/roadmap.md
wrote docs/changelog.md
wrote docs/architecture/README.md
wrote docs/decisions/architectural/README.md
wrote docs/decisions/functional/README.md
wrote tests/e2e/README.md
wrote tests/integration/.gitkeep

next:
  1. run `grund check` — on a fresh scaffold it warns \"nothing recognized\" until the first declaration; this is expected (exit 0), no repair needed
  2. allocate an ID:  ID=$(grund id REQ \"…\")  then add it under docs/req
     (H1: `# <ID>: <one-line statement of the behavior>`)
  3. cite it as §<ID> from the docs and e2e tests that depend on it, then `grund check` again
see AGENTS.md for the full workflow.
"
    );
}

/// §FS-init-fixtures.8.4, §FS-init.2.1.3: `--scaffold` lays the whole starter, so on the
/// tree with `docs/req` absent it writes the `REQ` home too, and the block stays the same.
#[test]
fn the_scaffold_lays_a_missing_req_home_with_the_rest_of_the_starter() {
    let files: Vec<_> = REQ_TREE
        .iter()
        .copied()
        .filter(|(path, _)| !path.starts_with("docs/req/"))
        .collect();
    let fixture = Fixture::new("scaffold-no-req", REQ_ONLY, &files, &[]);
    let stderr = fixture.stderr_of(&["init", "--scaffold"], 0);
    let present = Fixture::req_only("scaffold-req");
    let expected = present
        .stderr_of(&["init", "--scaffold"], 0)
        .replace("exists docs/req/README.md", "wrote docs/req/README.md");
    assert_eq!(stderr, expected);
    assert!(fixture.0.join("docs/req/README.md").is_file());
}

/// §FS-init.2.1.3: a one-kind `AR` project homed in `docs/architecture` gets that home
/// from the same starter, as one of its fixed places.
#[test]
fn the_scaffold_lays_an_ar_home_at_the_starters_place() {
    let config = "[[kinds]]\nkind = \"AR\"\nfolder = \"docs/architecture\"\n";
    let fixture = Fixture::new("scaffold-ar", config, &[], &[]);
    let stderr = fixture.stderr_of(&["init", "--scaffold"], 0);
    assert!(
        stderr.contains("wrote docs/functional-spec/README.md\n"),
        "{stderr}"
    );
    assert!(
        stderr.contains("wrote docs/architecture/README.md\n"),
        "{stderr}"
    );
}

/// §FS-init.2.2.2.3, §FS-init-fixtures.8: every place that gives the reason the
/// configured-kind form never offers `--scaffold` says the scaffold lays the whole
/// software-development starter rather than the selected home alone, and none says it
/// lays the FS starter alone or never lays the selected home (#974).
#[test]
fn the_scaffold_rationale_names_the_whole_starter_everywhere() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for place in [
        "docs/functional-spec/FS-init.md",
        "docs/functional-spec/FS-init-fixtures.md",
        "docs/decisions/functional/DF-init-configured-kind-guidance.md",
        "docs/user-facing/init.md",
        "skills/grund-init/SKILL.md",
        "crates/grund-core/assets/skills/grund-init/SKILL.md",
    ] {
        let prose = fs::read_to_string(root.join(place))
            .expect(place)
            .replace('\n', " ");
        for false_reason in [
            "FS starter and no",
            "lays only grund's `FS` starter",
            "lays grund's own `FS` starter",
            "not the selected kind's home",
            "is no way to lay the selected kind's home",
        ] {
            assert!(
                !prose.contains(false_reason),
                "{place} still says `{false_reason}`, but --scaffold lays the whole starter"
            );
        }
        assert!(
            prose.contains("whole software-development starter"),
            "{place} does not say --scaffold lays the whole software-development starter"
        );
        assert!(
            prose.contains("home alone"),
            "{place} does not say --scaffold lays more than the selected home alone"
        );
    }
}

/// The first `init` of a version-2 project declaring `rows`, over `dirs`.
fn first_block(name: &str, rows: &str, dirs: &[&str]) -> String {
    let config = format!("grund_config_version = 2\nproject_name = \"demo\"\n{rows}");
    let fixture = Fixture::new(name, &config, &[("README.md", "# Demo\n")], dirs);
    let stderr = fixture.stderr_of(&["init"], 0);
    assert_names_no_fs(name, &stderr);
    stderr
}

/// The `next:` block of a first run that asks for `home` of `kind` (§FS-init-fixtures.8.2).
fn asks_for(kind: &str, home: &str, under: &str) -> String {
    format!(
        "wrote AGENTS.md\nexists grund.toml\n\nnext:\n  1. create the {kind} home ({home})\n  \
         2. run `grund check`\n  3. allocate an ID:  ID=$(grund id {kind} \"…\")  then {under}\n\
         see AGENTS.md for the full workflow.\n"
    )
}

/// §FS-init.2.2.2.3: a row's first file comes before its first folder, whichever
/// key is written first, and an existing folder does not win over a missing file.
#[test]
fn a_rows_first_file_comes_before_its_folder_even_when_only_the_folder_exists() {
    let rows = "\n[schema.kinds.REQ]\nfolders = [\"docs/req\"]\nfiles = [\"requirements.md\"]\n";
    assert_eq!(
        first_block("file-first", rows, &["docs/req"]),
        asks_for("REQ", "requirements.md", "add it to requirements.md")
    );
}

/// §FS-init.2.2.2.3: the first place in written order is the one named, though a
/// later one exists.
#[test]
fn the_first_place_wins_though_a_later_one_exists() {
    let rows = "\n[schema.kinds.REQ]\nfolders = [\"docs/a\", \"docs/b\"]\n";
    assert_eq!(
        first_block("first-place", rows, &["docs/b"]),
        asks_for("REQ", "docs/a", "add it under docs/a")
    );
}

/// §FS-init.2.2.2.3: non-citable rows never qualify, the complement `code`
/// included, however early they are written.
#[test]
fn non_citable_rows_and_the_complement_are_skipped() {
    let rows = "\n[schema.kinds.code]\nfolders = [\"src\"]\n\n[schema.kinds.runbook]\n\
                citable = false\nfolders = [\"docs/runbooks\"]\n\n[schema.kinds.ADR]\n\
                folders = [\"docs/adr\"]\n";
    assert_eq!(
        first_block("non-citable", rows, &["src", "docs/runbooks", "docs/adr"]),
        "wrote AGENTS.md\nexists grund.toml\n\nnext:\n  1. run `grund check`\n  \
         2. allocate an ID:  ID=$(grund id ADR \"…\")  then add it under docs/adr\n\
         see AGENTS.md for the full workflow.\n"
    );
}

/// §FS-init.2.2.2.3: the first citable row with a home wins over an earlier
/// citable row without one.
#[test]
fn a_row_with_a_home_wins_over_an_earlier_row_without_one() {
    let rows = "\n[schema.kinds.NOTE]\n\n[schema.kinds.REQ]\nfolders = [\"docs/req\"]\n";
    assert_eq!(
        first_block("home-first", rows, &[]),
        asks_for("REQ", "docs/req", "add it under docs/req")
    );
}

/// §FS-init.2.2.2.3: a declared FS row with no home is not FS with a home, so
/// the guidance falls through to the first citable row that has one and never
/// names the built-in `docs/functional-spec`.
#[test]
fn an_fs_row_without_a_home_falls_through_to_a_row_with_one() {
    let rows = "\n[schema.kinds.FS]\n\n[schema.kinds.REQ]\nfolders = [\"docs/req\"]\n";
    let config = format!("grund_config_version = 2\nproject_name = \"demo\"\n{rows}");
    let fixture = Fixture::new("fs-homeless", &config, &[("README.md", "# Demo\n")], &[]);
    let stderr = fixture.stderr_of(&["init"], 0);
    assert!(!stderr.contains("docs/functional-spec"), "{stderr}");
    assert_eq!(stderr, asks_for("REQ", "docs/req", "add it under docs/req"));
}

/// §FS-init.2.2.2.3: where no citable row has a home, FS included, the first
/// citable row is selected and taught as a doc-comment declaration.
#[test]
fn an_fs_row_without_a_home_alone_teaches_a_doc_comment() {
    let config = "grund_config_version = 2\nproject_name = \"demo\"\n\n[schema.kinds.FS]\n";
    let fixture = Fixture::new("fs-alone", config, &[("README.md", "# Demo\n")], &[]);
    let stderr = fixture.stderr_of(&["init"], 0);
    assert!(!stderr.contains("docs/functional-spec"), "{stderr}");
    assert!(!stderr.contains("--scaffold"), "{stderr}");
    assert_eq!(
        stderr,
        "wrote AGENTS.md\nexists grund.toml\n\nnext:\n  1. run `grund check`\n  \
         2. allocate an ID:  ID=$(grund id FS \"…\")  then declare it in a source doc-comment\n\
         see AGENTS.md for the full workflow.\n"
    );
}
