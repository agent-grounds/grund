//! What `grund init` lays from a template (§FS-init.1.6) as the shipped CLI sees
//! it: `--scaffold` lays the starter files while `--docs`, its old spelling, is
//! refused as an unknown flag, and a fresh `--template=none` tree is one
//! `grund check` and `grund init --check` both pass. The single-command transcripts are e2e cases; these are the
//! sequences a case cannot express.

use std::fs;
use std::path::{Path, PathBuf};

#[path = "support/init_fixture.rs"]
mod init_fixture;

use init_fixture::{manifest_dir, run_grund, workdir};

/// Every file under `root`, relative and sorted, with its bytes.
fn tree(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<(PathBuf, Vec<u8>)>) {
        for entry in fs::read_dir(dir).expect("read dir") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                walk(root, &path, out);
            } else {
                let rel = path.strip_prefix(root).expect("under root").to_path_buf();
                out.push((rel, fs::read(&path).expect("read file")));
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out.sort();
    out
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

/// §FS-init.1.7: `--docs` was the deprecated spelling of `--scaffold` and is an
/// unknown flag from 0.19.0: refused with exit 2 before anything is written, while
/// `--scaffold` lays the starter files and warns about nothing.
#[test]
fn docs_is_an_unknown_flag_and_scaffold_lays_the_files() {
    let scaffold = workdir("init_template_scaffold_spelling");
    let docs = workdir("init_template_docs_spelling");
    let (ok, scaffold_stderr) = init(&scaffold, &["--scaffold"]);
    assert!(ok, "{scaffold_stderr}");
    assert!(!scaffold_stderr.contains("--docs"), "{scaffold_stderr}");
    assert!(scaffold.join("docs/functional-spec/README.md").is_file());

    let refused = run_grund(
        &["init", docs.to_str().unwrap(), "--name", "demo", "--docs"],
        manifest_dir(),
    );
    assert_eq!(refused.status.code(), Some(2), "{refused:?}");
    assert!(refused.stdout.is_empty(), "{refused:?}");
    assert_eq!(
        String::from_utf8_lossy(&refused.stderr),
        "error: unknown flag `--docs`\n"
    );
    assert!(tree(&docs).is_empty(), "a refused flag writes nothing");
}

/// §FS-init.1.6.2: a fresh `none` tree passes `grund check` and `grund init
/// --check` unmodified, before any kind is declared; the check still teaches the
/// open step, since a config with no kind is never a complete setup (§FS-init.2.2.2.2).
#[test]
fn a_fresh_none_tree_passes_check_and_init_check() {
    let target = workdir("init_template_none_passes_its_gates");
    fs::create_dir_all(target.join("docs/adr")).unwrap();
    fs::write(target.join("docs/adr/0001-postgres.md"), "# Use Postgres\n").unwrap();
    let (ok, stderr) = init(&target, &["--template=none"]);
    assert!(ok, "{stderr}");

    let check = run_grund(&["check", target.to_str().unwrap()], manifest_dir());
    assert!(
        check.status.success(),
        "grund check: {}{}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
    let (ok, stderr) = init(&target, &["--check"]);
    assert!(ok, "grund init --check: {stderr}");
    assert_eq!(
        stderr,
        "exists AGENTS.md\nexists grund.toml\n\nnext:\n  \
         1. declare a first kind in grund.toml, then run `grund init` again so AGENTS.md lists it\n  \
         2. run `grund check`\n  \
         3. allocate an ID:  ID=$(grund id <KIND> \"…\")  then add it to that kind's home\n\
         see AGENTS.md for the full workflow.\n"
    );
}

/// §FS-init.1.6.2, §FS-init.2.2.2.2: once the `none` config declares a first
/// kind, `init` renders it into the block and the guidance returns to the FS
/// steps — the hand-off the template's comment and block both teach.
#[test]
fn a_first_kind_replaces_the_empty_set() {
    let target = workdir("init_template_none_first_kind");
    let (ok, stderr) = init(&target, &["--template", "none"]);
    assert!(ok, "{stderr}");
    let mut config = fs::read_to_string(target.join("grund.toml")).unwrap();
    config.push_str(
        "\n[schema.kinds.FS]\nfolders = [\"docs/specs\"]\n\n[presentation.kinds.FS]\ntitle = \"Specs\"\n",
    );
    fs::write(target.join("grund.toml"), config).unwrap();
    let (ok, stderr) = init(&target, &[]);
    assert!(ok, "{stderr}");
    assert!(
        stderr.starts_with("updated AGENTS.md\nexists grund.toml\n"),
        "{stderr}"
    );
    let block = fs::read_to_string(target.join("AGENTS.md")).unwrap();
    assert!(block.contains("(`KIND ∈ {FS}`)"), "{block}");
    assert!(
        block.contains("### Project map\n\n- [FS](docs/specs): Specs\n"),
        "{block}"
    );
}

/// §FS-init.2.4.10, §FS-init-fixtures.2: a fresh `software-development`
/// scaffold declares `REQ` in `docs/requirements/` beside the `FS` row in
/// `docs/functional-spec/`, and passes both of its gates.
#[test]
fn a_fresh_scaffold_declares_req_in_docs_requirements() {
    let target = workdir("init_template_scaffold_declares_req");
    let (ok, stderr) = init(&target, &["--scaffold"]);
    assert!(ok, "{stderr}");
    assert!(
        stderr.contains(
            "wrote docs/functional-spec/README.md\nwrote docs/requirements/README.md\nwrote docs/grund.md\n"
        ),
        "{stderr}"
    );
    assert!(target.join("docs/requirements/README.md").is_file());

    let config = fs::read_to_string(target.join("grund.toml")).unwrap();
    let rows = "[schema.kinds.FS]\nfolders = [\"docs/functional-spec\"]\n\
                [schema.kinds.REQ]\nfolders = [\"docs/requirements\"]\n";
    assert!(config.contains(rows), "{config}");
    let titles = "[presentation.kinds.FS]\n\
                  title = \"What: behavior, requirements, and constraints\"\n\
                  [presentation.kinds.REQ]\n\
                  title = \"Hard requirements: what the project must never break\"\n";
    assert!(config.contains(titles), "{config}");
    // §FS-init.2.4.5: no commented citation ruleset; no direction is enforced.
    assert!(!config.contains("citations"), "{config}");

    let block = fs::read_to_string(target.join("AGENTS.md")).unwrap();
    assert!(
        block.contains("(`KIND ∈ {GRUND, GOAL, FS, REQ, AR, DF, DA, RM}`)"),
        "{block}"
    );
    let map = "- [FS](docs/functional-spec): What: behavior, requirements, and constraints\n\
               - [REQ](docs/requirements): Hard requirements: what the project must never break\n\
               - [AR](docs/architecture): How: high-level implementation, structure, and design\n";
    assert!(block.contains(map), "{block}");

    let check = run_grund(&["check", target.to_str().unwrap()], manifest_dir());
    assert!(
        check.status.success(),
        "grund check: {}{}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
    let (ok, stderr) = init(&target, &["--check"]);
    assert!(ok, "grund init --check: {stderr}");
}

/// §FS-init.2.1.3: `--scaffold` lays the `REQ` starter only where the governing
/// config declares `REQ`, in the folder its row names: a config an earlier
/// `init` wrote, with no `REQ` row, gets none.
#[test]
fn the_req_starter_follows_the_configs_req_row() {
    let fs_row =
        "grund_config_version = 1\n\n[[kinds]]\nkind = \"FS\"\nfile = \"requirements.md\"\n";

    let older = workdir("init_template_scaffold_without_req");
    fs::write(older.join("grund.toml"), fs_row).unwrap();
    let (ok, stderr) = init(&older, &["--scaffold"]);
    assert!(ok, "{stderr}");
    assert!(!stderr.contains("docs/requirements"), "{stderr}");
    assert!(!older.join("docs/requirements").exists(), "{stderr}");

    let moved = workdir("init_template_scaffold_req_elsewhere");
    let config = format!("{fs_row}\n[[kinds]]\nkind = \"REQ\"\nfolder = \"specs/hard\"\n");
    fs::write(moved.join("grund.toml"), config).unwrap();
    let (ok, stderr) = init(&moved, &["--scaffold"]);
    assert!(ok, "{stderr}");
    assert!(
        stderr.contains("wrote requirements.md\nwrote specs/hard/README.md\n"),
        "{stderr}"
    );
    assert!(!moved.join("docs/requirements").exists(), "{stderr}");
}
