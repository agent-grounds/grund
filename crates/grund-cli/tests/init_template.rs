//! What `grund init` lays from a template (§FS-init.1.6) as the shipped CLI sees
//! it: `--scaffold` and its deprecated spelling `--docs` lay one tree, and a
//! fresh `--template=none` tree is one `grund check` and `grund init --check`
//! both pass. The single-command transcripts are e2e cases; these are the
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

/// §FS-init.1.7: `--docs` lays exactly what `--scaffold` lays and reports it the
/// same way, after one warning naming the replacement and the removal release.
#[test]
fn docs_and_scaffold_lay_one_tree() {
    let scaffold = workdir("init_template_scaffold_spelling");
    let docs = workdir("init_template_docs_spelling");
    let (ok, scaffold_stderr) = init(&scaffold, &["--scaffold"]);
    assert!(ok, "{scaffold_stderr}");
    let (ok, docs_stderr) = init(&docs, &["--docs"]);
    assert!(ok, "{docs_stderr}");
    assert_eq!(tree(&scaffold), tree(&docs));
    let warning = "warning: --docs is deprecated and is removed in grund 0.19.0; use --scaffold, which lays the same starter files\n";
    assert_eq!(docs_stderr, format!("{warning}{scaffold_stderr}"));
    assert!(!scaffold_stderr.contains("deprecated"), "{scaffold_stderr}");
}

/// §FS-init.1.6.2: a fresh `none` tree passes `grund check` and `grund init
/// --check` unmodified, before any kind is declared.
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
    assert_eq!(stderr, "exists AGENTS.md\nexists grund.toml\n");
}

/// §FS-init.1.6.2, §FS-init.2.2.2.2: once the `none` config declares a first
/// kind, `init` renders it into the block and the guidance returns to the FS
/// steps — the hand-off the template's comment and block both teach.
#[test]
fn a_first_kind_replaces_the_empty_set() {
    let target = workdir("init_template_none_first_kind");
    let (ok, stderr) = init(&target, &["--template", "none"]);
    assert!(ok, "{stderr}");
    let config = fs::read_to_string(target.join("grund.toml")).unwrap();
    let config = config.replace(
        "kinds = []\n",
        "[[kinds]]\nkind = \"FS\"\nfolder = \"docs/specs\"\ntitle = \"Specs\"\n",
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
