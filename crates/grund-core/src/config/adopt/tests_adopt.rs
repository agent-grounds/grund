//! Test module: adoption's gate and write, on the issue's project as migration leaves it
//! (§FS-config-adopt-builtins). The plan is built from the file alone, so a writer's mistake is
//! held to it and refused (§FS-config-adopt-builtins.6.2), and a write that cannot finish, or
//! finds the config changed, leaves the file as it found it (§FS-config-adopt-builtins.8.1).
//! Fixture builders retain their temporary-root owner (§FS-repository-maintenance.4).

use std::path::Path;

use super::plan::{Decision, plan};
use super::record::{AdoptFailure, AdoptOutcome, AdoptReport, KeptWhy};
use super::{Prepared, gate::gate, prepare};
use crate::api::BETWEEN_CHECKS;
use crate::config::discovery::config_with_spans;
use crate::config::file_reading::{SuffixReading, model_suffix_reading};
use crate::config::migrate::Located;
use crate::config::project::Project;
use crate::config::run::{Invocation, Selection};
use crate::config::sources::{DeclarationMode, builtin};
use crate::testing::{TemporaryRoot, test_root, write};
use crate::writers::{FAULT, Stage};
use crate::{AdoptOpts, config_adopt};

/// What `grund config migrate --to 2 --write` leaves for the issue's version-1 project: `.md`,
/// `.java`, `.gradle` and `.sh`, each through its `v1_*` definition, read through
/// `["//", "#", "*", "/*"]`.
const MIGRATED: &str = concat!(
    "grund_config_version = 2\n",
    "\n",
    "[schema]\n",
    "id_format = \"{kind}-{slug}\"\n",
    "\n",
    "[schema.sources]\n",
    "languages = [\"v1_markdown\", \"v1_c_family\", \"v1_plain\", \"v1_shell\"]\n",
    "declarations = \"any-comment\"\n",
    "\n",
    "[schema.sources.extensions]\n",
    "md = \"v1_markdown\"\n",
    "java = \"v1_c_family\"\n",
    "gradle = \"v1_plain\"\n",
    "sh = \"v1_shell\"\n",
    "\n",
    "[schema.sources.definitions.v1_markdown]\n",
    "class = \"document\"\n",
    "comment_prefixes = [\"//\", \"#\", \"*\", \"/*\"]\n",
    "doc_comments = \"none\"\n",
    "docstrings = \"off\"\n",
    "\n",
    "[schema.sources.definitions.v1_c_family]\n",
    "class = \"source\"\n",
    "comment_prefixes = [\"//\", \"#\", \"*\", \"/*\"]\n",
    "doc_comments = \"c-family\"\n",
    "docstrings = \"off\"\n",
    "\n",
    "[schema.sources.definitions.v1_plain]\n",
    "class = \"source\"\n",
    "comment_prefixes = [\"//\", \"#\", \"*\", \"/*\"]\n",
    "doc_comments = \"none\"\n",
    "docstrings = \"off\"\n",
    "\n",
    "[schema.sources.definitions.v1_shell]\n",
    "class = \"source\"\n",
    "comment_prefixes = [\"//\", \"#\", \"*\", \"/*\"]\n",
    "doc_comments = \"shell\"\n",
    "docstrings = \"off\"\n",
    "\n",
    "[schema.kinds.FS]\n",
    "folders = [\"docs\"]\n",
    "index = false\n",
    "\n",
    "[schema.kinds.code]\n",
    "folders = [\"src\"]\n",
);

/// The project under a fresh root: [`MIGRATED`], one spec and a source of each suffix citing it.
fn tree(name: &str) -> TemporaryRoot {
    let root = test_root(&format!("adopt-{name}"));
    write(&root.join("grund.toml"), MIGRATED);
    write(
        &root.join("docs/FS-login.md"),
        "# FS-login: Login\n\nText.\n",
    );
    write(
        &root.join("src/Login.java"),
        "// \u{a7}FS-login\nclass Login {}\n",
    );
    write(&root.join("src/build.gradle"), "// \u{a7}FS-login\n");
    write(&root.join("src/run.sh"), "# \u{a7}FS-login\n");
    root
}

fn located(root: &TemporaryRoot) -> Located {
    let root = root.to_path_buf();
    Located::new(root.clone(), root.join("grund.toml"), root)
}

/// [`MIGRATED`] loaded as the config at `root`, as adoption loads the file it plans from.
fn loaded(root: &Path) -> Project {
    let invocation = Invocation::new(root, false, Selection::Project);
    let (config, _) = config_with_spans(MIGRATED, Path::new("grund.toml"), root, &invocation)
        .expect("the migrated file loads");
    config.project().clone()
}

/// The built-in row `name`'s reading of `suffix`, under `any-comment`.
fn row_reading(name: &str, suffix: &str) -> SuffixReading {
    let row = builtin(name).expect("a built-in row");
    SuffixReading::of_language(&row, suffix, DeclarationMode::AnyComment)
}

/// Every name in `root`, sorted, so a temporary file left behind shows.
fn entries(root: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(root)
        .expect("read the root")
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

/// A `--write` run of [`config_adopt`] on `root`.
fn adopt(root: &Path) -> Result<AdoptOutcome, AdoptFailure> {
    config_adopt(&AdoptOpts {
        path: root.to_path_buf(),
        write: true,
        drop_comments: false,
    })
}

/// §FS-config-adopt-builtins.6.2: a writer that wrote `java = "shell"`, with `shell` admitted, is
/// refused with exit `1`, although the set of suffixes that change looks right. The plan was
/// built before the candidate existed, so it still expects `.java` read as built-in `java`, and
/// the difference names `.java` alone. The gate writes nothing, and a refused report never
/// reaches the write.
#[test]
fn a_forged_candidate_is_refused_against_the_plan() {
    let root = tree("forged");
    let Prepared::Planned { report, .. } =
        prepare(&located(&root), true, false).expect("the migrated file plans")
    else {
        panic!("the migrated file adopts");
    };
    assert!(report.readings.is_empty(), "{:?}", report.readings);
    let header = "[schema.sources.extensions]\n";
    assert!(report.candidate.contains(header), "{}", report.candidate);
    let forged = report
        .candidate
        .replacen(header, &format!("{header}java = \"shell\"\n"), 1);
    assert!(forged.contains("\"shell\""), "shell is admitted: {forged}");
    let project = loaded(&root);
    let plan = plan(
        project
            .schema
            .sources
            .model
            .as_ref()
            .expect("a version-2 model"),
    );
    let mut forged = AdoptReport {
        config: "grund.toml".into(),
        write: true,
        candidate: forged,
        ..AdoptReport::default()
    };
    gate(&mut forged, &project, &plan, &root);
    assert_eq!(forged.load_error, None);
    assert_eq!(
        forged.readings,
        [
            "differs: the readings adoption planned".to_string(),
            format!("- java: {:?}", row_reading("java", "java")),
            format!("+ java: {:?}", row_reading("shell", "java")),
        ]
    );
    assert!(forged.refused());
    assert_eq!(forged.exit(), 1);
    assert_eq!(forged.stdout(), None);
    assert_eq!(
        std::fs::read_to_string(root.join("grund.toml")).unwrap(),
        MIGRATED
    );
}

/// §FS-config-adopt-builtins.6.2, §FS-config-adopt-builtins.terms: the expected model is the
/// file's resolved model with the plan applied, built from the model and version 2's epoch alone,
/// with no writer and no candidate. `.md`, `.java` and `.sh` are adopted, `.bash` and `.zsh` are
/// added with `shell`, `.gradle` is kept, and `.py`, which no listed language reads, is not read
/// before or after.
#[test]
fn the_expected_model_needs_no_writer() {
    let root = tree("expected");
    let project = loaded(&root);
    let model = project
        .schema
        .sources
        .model
        .as_ref()
        .expect("a version-2 model");
    let plan = plan(model);
    let decisions: Vec<(&str, String)> = plan
        .entries
        .iter()
        .map(|entry| {
            let decision = match &entry.decision {
                Decision::Adopted(row) => format!("adopted into {}", row.name),
                Decision::Kept(KeptWhy::NoRow) => "kept, no row".into(),
                Decision::Kept(why) => format!("kept, {why:?}"),
            };
            (entry.suffix.as_str(), decision)
        })
        .collect();
    assert_eq!(
        decisions,
        [
            ("md", "adopted into markdown".to_string()),
            ("java", "adopted into java".to_string()),
            ("gradle", "kept, no row".to_string()),
            ("sh", "adopted into shell".to_string()),
        ]
    );
    let added: Vec<(&str, Vec<&str>)> = plan
        .added
        .iter()
        .map(|added| {
            let suffixes = added.suffixes.iter().map(String::as_str).collect();
            (added.row.as_str(), suffixes)
        })
        .collect();
    assert_eq!(added, [("shell", vec!["bash", "zsh"])]);
    let expected: Vec<&str> = plan.expected.keys().map(String::as_str).collect();
    assert_eq!(expected, ["bash", "gradle", "java", "md", "sh", "zsh"]);
    for (suffix, row) in [
        ("md", "markdown"),
        ("java", "java"),
        ("sh", "shell"),
        ("bash", "shell"),
        ("zsh", "shell"),
    ] {
        assert_eq!(plan.expected[suffix], row_reading(row, suffix), "{suffix}");
    }
    assert_eq!(
        Some(&plan.expected["gradle"]),
        model_suffix_reading(model, "gradle").as_ref()
    );
    assert_eq!(model_suffix_reading(model, "py"), None);
}

/// §FS-config-adopt-builtins.8.1: an edit made after the plan was built and before the rename is
/// exit `2` with adoption's message, and the edit stays.
#[test]
fn a_concurrent_edit_stops_the_write_and_keeps_the_edit() {
    let root = tree("concurrent");
    let edit = format!("{MIGRATED}# edited\n");
    FAULT.with(|fault| *fault.borrow_mut() = (None, Some(edit.as_bytes().to_vec())));
    let result = adopt(&root);
    FAULT.with(|fault| *fault.borrow_mut() = (None, None));
    let failure = result.expect_err("the write is stopped");
    assert_eq!(failure.exit(), 2);
    assert_eq!(
        format!("error: {failure}"),
        "error: grund.toml changed while its built-in languages were being adopted; nothing \
         written, run grund config adopt-builtins again"
    );
    assert_eq!(
        std::fs::read_to_string(root.join("grund.toml")).unwrap(),
        edit
    );
    assert_eq!(entries(&root), ["docs", "grund.toml", "src"]);
}

/// §FS-config-adopt-builtins.8.1: an I/O failure at the temporary write, the sync or the rename is
/// exit `2` with adoption's message, the config byte for byte as it was, and no temporary file.
#[test]
fn an_io_failure_leaves_the_config_and_no_temporary_file() {
    for stage in [Stage::Write, Stage::Sync, Stage::Rename] {
        let root = tree(&format!("io-{stage:?}"));
        FAULT.with(|fault| *fault.borrow_mut() = (Some(stage), None));
        let result = adopt(&root);
        FAULT.with(|fault| *fault.borrow_mut() = (None, None));
        let failure = result.expect_err("the write fails");
        assert!(
            matches!(failure, AdoptFailure::Write { .. }),
            "{stage:?}: {failure:?}"
        );
        assert_eq!(failure.exit(), 2, "{stage:?}");
        let message = format!("error: {failure}");
        assert!(
            message.starts_with("error: grund.toml: could not write the adopted config (")
                && message.contains(&format!("injected at {stage:?}"))
                && message
                    .ends_with("); grund.toml is unchanged, run grund config adopt-builtins again"),
            "{stage:?}: {message}"
        );
        assert_eq!(
            std::fs::read_to_string(root.join("grund.toml")).unwrap(),
            MIGRATED,
            "{stage:?}"
        );
        assert_eq!(entries(&root), ["docs", "grund.toml", "src"], "{stage:?}");
    }
}

/// §FS-config-adopt-builtins.8.1, §FS-config-adopt-builtins.6.4: a config rewritten between the
/// two `check` runs is found when its bytes are read again after them, exit `2` with the changed
/// message, and the candidate is never written.
#[test]
fn a_config_rewritten_between_the_checks_is_not_written() {
    let root = tree("between");
    let edit = format!("{MIGRATED}# edited between the checks\n");
    BETWEEN_CHECKS.with(|slot| *slot.borrow_mut() = Some(edit.clone()));
    let result = adopt(&root);
    BETWEEN_CHECKS.with(|slot| *slot.borrow_mut() = None);
    let failure = result.expect_err("the run stops");
    assert!(
        matches!(failure, AdoptFailure::Changed { .. }),
        "{failure:?}"
    );
    assert_eq!(failure.exit(), 2);
    assert_eq!(
        format!("error: {failure}"),
        "error: grund.toml changed while its built-in languages were being adopted; nothing \
         written, run grund config adopt-builtins again"
    );
    assert_eq!(
        std::fs::read_to_string(root.join("grund.toml")).unwrap(),
        edit
    );
    assert_eq!(entries(&root), ["docs", "grund.toml", "src"]);
}
