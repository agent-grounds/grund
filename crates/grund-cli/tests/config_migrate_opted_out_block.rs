//! `grund config migrate --to 2` on a version-1 block with `include_root = false`
//! (§FS-config-migrate.8.1): the tree of agent-grounds/grund#1142, a version-2
//! root listing `group`, the block at `group/grund.toml`, and a version-2 member
//! at `group/a/grund.toml`. Each run is the reported command, `grund config
//! migrate --to 2 group`, from the root, and every stream is compared whole.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

/// The reported block: lines 2 and 3 are `include_root` and `members`.
const BARE: &str = "[workspace]\ninclude_root = false\nmembers = [\"a\"]\n";

/// The block after the two old repairs, on lines 2 and 5.
const REPAIRED: &str = "[id]\nnamed_sections = true\n\n[scan]\ninclude = []\n\n\
[workspace]\ninclude_root = false\nmembers = [\"a\"]\n";

/// §FS-config-migrate.8.1.3: the version line and the `[workspace]` table.
const CANDIDATE: &str =
    "grund_config_version = 2\n\n[workspace]\nmembers = [\"a\"]\ninclude_root = false\n";

/// §FS-config-migrate.3.2: the one notice for the defaults left unwritten.
const DEFAULTS: &str = "dropped: grund.toml: version 1's defaults: include_root = false, \
so no project scans this block's own files; it takes version 2's defaults\n";

const MAPPED: &str = "mapped: grund.toml:2: [workspace] include_root → [workspace] include_root\n\
mapped: grund.toml:3: [workspace] members → [workspace] members\n";

const COMPARED: &str = "compared: the records are equal, and grund check --suggestions on this \
tree gives the same findings and the same exit before and after\n";

const AFTER_WRITE: &str = "to get version 1 back: it is in version control, e.g. git restore grund.toml\n\
to see this candidate again: restore version 1 and run grund config migrate --to 2\n\
note: existing AGENTS.md managed instructions may still describe version 1; \
run grund init in the migrated project to refresh them\n";

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// The reported tree under the system temp directory, where no config above it
/// is discovered, with `block` as `group/grund.toml`.
fn tree(name: &str, block: &str) -> PathBuf {
    static SERIAL: AtomicUsize = AtomicUsize::new(0);
    let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir()
        .join(format!("grund-migrate-opted-out-{}", std::process::id()))
        .join(format!("{name}-{serial}"));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("group/a")).expect("create fixture");
    fs::write(
        root.join("grund.toml"),
        "grund_config_version = 2\n\n[workspace]\nmembers = [\"group\"]\n",
    )
    .expect("write root config");
    fs::write(root.join("group/grund.toml"), block).expect("write block config");
    fs::write(
        root.join("group/a/grund.toml"),
        "grund_config_version = 2\n",
    )
    .expect("write member config");
    root
}

fn migrate(root: &Path, write: bool) -> Output {
    let mut args = vec!["config", "migrate", "--to", "2", "group"];
    if write {
        args.push("--write");
    }
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(&args)
        .current_dir(root)
        .output()
        .expect("spawn grund")
}

fn block(root: &Path) -> String {
    fs::read_to_string(root.join("group/grund.toml")).expect("read block config")
}

fn assert_run(out: &Output, code: i32, stdout: &str, stderr: &str) {
    assert_eq!(
        (out.status.code(), text(&out.stdout), text(&out.stderr)),
        (Some(code), stdout.to_owned(), stderr.to_owned()),
        "exit, stdout and stderr"
    );
}

/// §FS-config-migrate.8.1.5: the bare block previews its compact candidate,
/// writes exactly those bytes, and a second run finds version 2.
#[test]
fn bare_block_previews_and_writes_its_envelope_and_workspace_table() {
    let root = tree("bare", BARE);
    let preview = migrate(&root, false);
    let report = format!(
        "grund.toml: version 1 → 2, preview, nothing written\n{DEFAULTS}{MAPPED}{COMPARED}"
    );
    assert_run(&preview, 0, CANDIDATE, &report);
    assert_eq!(block(&root), BARE, "a preview writes nothing");

    let write = migrate(&root, true);
    let report = format!(
        "wrote grund.toml: version 1 (3 lines) replaced by version 2\n{DEFAULTS}{AFTER_WRITE}"
    );
    assert_run(&write, 0, "", &report);
    assert_eq!(
        block(&root),
        CANDIDATE,
        "--write writes the preview's candidate"
    );

    let again = migrate(&root, true);
    assert_run(
        &again,
        0,
        "",
        "grund.toml is already version 2; nothing to migrate\n",
    );
}

/// §FS-config-migrate.8.1.2, §FS-config-migrate.3.2: the two old repairs have no
/// force on the block, so they are dropped with a notice each and the file is the
/// same compact candidate, far under the report's 20-line bound.
#[test]
fn repaired_block_drops_both_repairs_and_writes_the_same_candidate() {
    let root = tree("repaired", REPAIRED);
    let write = migrate(&root, true);
    let report = format!(
        "wrote grund.toml: version 1 (9 lines) replaced by version 2\n\
dropped: grund.toml:2: [id] named_sections: include_root = false, so no project scans this block's own files\n\
dropped: grund.toml:5: [scan] include: include_root = false, so no project scans this block's own files\n\
{DEFAULTS}{AFTER_WRITE}"
    );
    let written = block(&root);
    assert_eq!(written, CANDIDATE);
    assert!(written.lines().count() < 20, "{written}");
    assert_run(&write, 0, "", &report);
}

/// §FS-config-migrate.8.1.4: a block holding a file version 2's whole-root scope
/// reads, and version 1's default scope never did, reaches the report gate in one
/// run (§FS-check.4.10) rather than an `include` repair. The `differs:` lines are
/// the unread-block warning, its unread tree the block root named `.`
/// (§FS-check.4.10.5, agent-grounds/grund#1145).
#[test]
fn block_holding_a_readme_or_agents_md_stops_at_the_report_gate_in_one_run() {
    let warning = "+ run-warning warning unread-workspace-block grund.toml:2:- grund.toml:2: \
no project scans `.`, so its citations are never checked. Set include_root = true, \
or configure another project's source scope to read it.\n";
    for file in ["README.md", "AGENTS.md"] {
        let root = tree("unread-file", BARE);
        fs::write(root.join("group").join(file), "# Group\n").expect("write unread file");
        let write = migrate(&root, true);
        let report = format!(
            "grund.toml: version 1 → 2, --write\n{DEFAULTS}{MAPPED}\
differs: grund check --suggestions on this tree\n{warning}\
not written: grund check --suggestions differs (inspect the comparison above); grund.toml is unchanged\n"
        );
        assert_run(&write, 1, "", &report);
        assert_eq!(
            block(&root),
            BARE,
            "{file}: a refused write leaves the block"
        );
    }
}

/// Control, passing before and after: a block whose default scope holds `docs`
/// keeps the conservative path, and its missing-`include` refusal keeps the bytes
/// §FS-config-migrate.3.1.3 gives it.
#[test]
fn block_with_a_default_scope_root_keeps_the_include_refusal() {
    let root = tree("docs", BARE);
    fs::create_dir_all(root.join("group/docs")).expect("create docs");
    fs::write(root.join("group/docs/notes.md"), "# Notes\n").expect("write docs file");
    let write = migrate(&root, true);
    let stderr = text(&write.stderr);
    assert_eq!(write.status.code(), Some(1), "stderr: {stderr}");
    assert!(write.stdout.is_empty(), "stdout: {}", text(&write.stdout));
    let refusal = "refused: grund.toml: [scan] include is not set, so version 1 reads its default, \
which names `requirements.md`, `e2e` and `src`, which do not exist, and version 2 must know whether \
each is a folder or a file\n  repair: add [scan] include = [\"docs\"] under version 1; grund check \
shows its effect\n";
    assert!(stderr.contains(refusal), "stderr: {stderr}");
    assert_eq!(block(&root), BARE);
}

/// Control, passing before and after: an ordinary version-1 project with named
/// sections unset is still refused (§FS-config-migrate.3.1.1); the exception is
/// the block's alone.
#[test]
fn ordinary_project_is_still_refused_on_named_sections() {
    let root = tree("project", "[scan]\ninclude = [\"docs\"]\n");
    fs::create_dir_all(root.join("group/docs")).expect("create docs");
    fs::write(root.join("group/docs/notes.md"), "# Notes\n").expect("write docs file");
    let write = migrate(&root, true);
    let stderr = text(&write.stderr);
    assert_eq!(write.status.code(), Some(1), "stderr: {stderr}");
    let refusal = "refused: grund.toml: [id] named_sections is not set, so version 1 reads no named \
sections, and version 2 always reads them\n  repair: add [id] named_sections = true under version 1; \
grund check shows its effect\n";
    assert!(stderr.contains(refusal), "stderr: {stderr}");
}
