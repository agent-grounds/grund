//! The editor-buffer trees `command_agreement.rs` compares: each asks the editor
//! snapshot and `show` the same question twice, once with an edit held as a
//! buffer and once with the same bytes saved, and the two answers must be one
//! (§FS-scan.text.buffer, §REQ-one-answer.text.targets). A buffer is the file
//! under every spelling (§FS-scan.identity.buffers), and a buffer with no file
//! behind it is a file the walk reads or skips as it would the saved file
//! (§FS-scan.text.buffer-only). R06a to R06d, the index check, the inline-code
//! hint, the `AGENTS.md` block check and the end-to-end manifest reader, closed
//! with §DF-four-editor-checks-read-saved-text, so their trees must agree too.

use super::agreement_tree::Verdict;
use grund_core::{
    Finding, LspSnapshotOpts, ShowOpts, lsp_snapshot_with_completion, show_from_snapshot,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[path = "agreement_buffer_ownership.rs"]
mod ownership;

/// One kind, `FS`, at `docs/`, scanning `docs` and whatever `extra` adds.
fn config(extra: &[&str]) -> String {
    let include: Vec<String> = ["docs"]
        .iter()
        .chain(extra)
        .map(|dir| format!("\"{dir}\""))
        .collect();
    format!(
        "grund_config_version = 1\n[reference]\nstrict = true\n[id]\nformat = \"{{kind}}-{{slug}}\"\n\
         [[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n\
         [scan]\ninclude = [{}]\n",
        include.join(", ")
    )
}

/// The invariant every tree here holds: a buffer answers as the same bytes saved.
pub const INVARIANT: &str = "a-buffer-answers-as-saved";

/// `(row, tree, the question asked of the tree)`. A tree with no row holds a
/// point the closed rows R04, R05 and R06a to R06d left behind, so its two
/// answers must agree.
pub const TREES: &[(Option<&str>, &str, fn() -> Verdict)] = &[
    (
        None,
        "r04-physical",
        a_physical_buffer_removes_what_the_stub_reads,
    ),
    (
        None,
        "r04-link",
        a_linked_buffer_removes_what_the_stub_reads,
    ),
    (
        None,
        "r04b-physical",
        a_physical_buffer_adds_what_the_stub_reads,
    ),
    (None, "r04b-link", a_linked_buffer_adds_what_the_stub_reads),
    (None, "r05", a_buffer_only_target_exists),
    (
        None,
        "r05b",
        a_buffer_only_file_the_walk_ignores_is_not_read,
    ),
    (None, "r06a", an_index_entry_is_judged_on_the_buffer),
    (None, "r06b", the_inline_code_hint_reads_the_buffer),
    (None, "r06c", the_agents_block_is_judged_on_the_buffer),
    (
        None,
        "r06c-companion",
        a_companions_block_is_found_in_the_buffer,
    ),
    (None, "r06d", the_e2e_manifest_is_read_from_the_buffer),
];

/// A stub at `docs/a.md` to `target` and a citation of `FS-a` beside it.
const STUB_USE: &[(&str, &str)] = &[("docs/use.md", "See \u{a7}FS-a.\n")];

/// A declaration of `FS-a` with a one-line body.
const DECLARES: &str = "# FS-a: Alpha\n\nBody.\n";

/// One fixture tree under the worktree's own `target/`, a git repository so that
/// `.git/info/exclude` is one of its ignore files.
struct Tree(PathBuf);

impl Tree {
    fn new(name: &str, config: &str, files: &[(&str, &str)]) -> Self {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/command-agreement")
            .join(format!("{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create fixture root");
        ownership::before_git(&root);
        let git = Command::new("git")
            .args(["init", "-q", "."])
            .current_dir(&root)
            .output()
            .expect("spawn git init");
        assert!(git.status.success(), "git init {}", root.display());
        let tree = Self(root.canonicalize().expect("canonical fixture root"));
        tree.write("grund.toml", config);
        for (path, body) in files {
            tree.write(path, body);
        }
        tree
    }

    fn write(&self, relative: &str, body: &str) {
        let path = self.0.join(relative);
        fs::create_dir_all(path.parent().expect("parent")).expect("create fixture parent");
        fs::write(path, body).expect("write fixture");
    }

    /// The editor's answer and `show`'s for `id`, over `buffers`.
    fn answers(&self, buffers: &BTreeMap<PathBuf, String>, id: &str) -> String {
        let loaded = super::agreement_evidence::editor(
            LspSnapshotOpts {
                path: self.0.clone(),
                path_provided: true,
                open_documents: buffers.clone(),
            },
            lsp_snapshot_with_completion,
        );
        let findings = match &loaded {
            Ok(loaded) => {
                let snapshot = &loaded.metadata.snapshot;
                let line = |finding: &Finding| format!("{}: {}", finding.code, finding.message);
                let errors: Vec<String> = snapshot.report.errors.iter().map(line).collect();
                let warnings: Vec<String> = snapshot.report.warnings.iter().map(line).collect();
                format!("errors {errors:?}, warnings {warnings:?}")
            }
            Err(error) => format!("snapshot refused: {error}"),
        };
        // §FS-lsp.responsiveness.3: `show` as the editor answers it, from the
        // context that snapshot was loaded from.
        let opts = ShowOpts::default();
        let shown = match &loaded {
            Ok(loaded) => show_from_snapshot(&loaded.show, id, opts.mode, opts.format)
                .map(|shown| shown.body)
                .map_err(|error| error.to_string()),
            Err(error) => Err(error.to_string()),
        };
        format!("{findings}, show {id} {shown:?}")
    }

    /// The answers with `text` held as a buffer at `relative`, then with it saved
    /// there and no buffer at all: `Err` names both unless they are one.
    fn buffer_against_saved(&self, relative: &str, text: &str, id: &str) -> Verdict {
        let buffers = BTreeMap::from([(self.0.join(relative), text.to_string())]);
        let buffered = self.answers(&buffers, id);
        self.write(relative, text);
        let saved = self.answers(&BTreeMap::new(), id);
        if buffered == saved {
            Ok(())
        } else {
            Err(format!(
                "buffer at {relative}: {buffered}\n    saved: {saved}"
            ))
        }
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[cfg(unix)]
fn link(tree: &Tree, target: &str, at: &str) {
    std::os::unix::fs::symlink(target, tree.0.join(at)).expect("create fixture symlink");
}

/// `docs/a.md` stubs `../alias/a.md`, `alias -> notes`, and `notes/a.md` holds
/// `saved`; the edit to `edited` is held under `at`, one spelling of that file.
#[cfg(unix)]
fn behind_a_symlink(name: &str, scanned: &[&str], saved: &str, at: &str, edited: &str) -> Verdict {
    let mut files = STUB_USE.to_vec();
    files.push(("docs/a.md", "# FS-a: [a](../alias/a.md)\n"));
    files.push(("notes/a.md", saved));
    let tree = Tree::new(name, &config(scanned), &files);
    link(&tree, "notes", "alias");
    tree.buffer_against_saved(at, edited, "FS-a")
}

#[cfg(not(unix))]
fn behind_a_symlink(_: &str, _: &[&str], _: &str, _: &str, _: &str) -> Verdict {
    Ok(())
}

/// R04: a buffer of the target's physical path that removes the declaration
/// breaks the stub that spells the target through a symlink, as the save does.
fn a_physical_buffer_removes_what_the_stub_reads() -> Verdict {
    behind_a_symlink(
        "r04-physical",
        &[],
        DECLARES,
        "notes/a.md",
        "No declaration.\n",
    )
}

/// R04, under the stub's own spelling: the buffer is opened as `alias/a.md`.
fn a_linked_buffer_removes_what_the_stub_reads() -> Verdict {
    behind_a_symlink("r04-link", &[], DECLARES, "alias/a.md", "No declaration.\n")
}

/// R04, the other direction: a buffer that adds the declaration repairs the stub.
fn a_physical_buffer_adds_what_the_stub_reads() -> Verdict {
    behind_a_symlink(
        "r04b-physical",
        &[],
        "No declaration.\n",
        "notes/a.md",
        DECLARES,
    )
}

/// R04, adding under the stub's own spelling, with `alias/` walked as well.
fn a_linked_buffer_adds_what_the_stub_reads() -> Verdict {
    behind_a_symlink(
        "r04b-link",
        &["alias"],
        "No declaration.\n",
        "alias/a.md",
        DECLARES,
    )
}

/// R05: a stub's target that exists only as a buffer exists, and declares its ID
/// once, as the saved file does.
fn a_buffer_only_target_exists() -> Verdict {
    let mut files = STUB_USE.to_vec();
    files.push(("docs/a.md", "# FS-a: [a](../src/a.rs)\n"));
    let tree = Tree::new("r05", &config(&["src"]), &files);
    let source = "/// FS-a: Alpha\n///\n/// Body.\npub fn a() {}\n";
    tree.buffer_against_saved("src/a.rs", source, "FS-a")
}

/// A buffer-only file in a directory `.git/info/exclude` ignores is not read, as
/// the saved file is not, so its declaration is no duplicate.
fn a_buffer_only_file_the_walk_ignores_is_not_read() -> Verdict {
    let mut files = STUB_USE.to_vec();
    files.push(("docs/fs.md", DECLARES));
    files.push((".git/info/exclude", "gen/\n"));
    let tree = Tree::new("r05b", &config(&[]), &files);
    tree.buffer_against_saved("docs/gen/b.md", "# FS-a: Other\n\nX.\n", "FS-a")
}

/// R06a: an index entry edited to a bare citation.
fn an_index_entry_is_judged_on_the_buffer() -> Verdict {
    let indexed = config(&[]).replace("index = false\n", "");
    let files = [
        ("docs/a.md", DECLARES),
        (
            "docs/README.md",
            "# Index\n\n[\u{a7}FS-a](a.md#fs-a-alpha)\n",
        ),
    ];
    let tree = Tree::new("r06a", &indexed, &files);
    tree.buffer_against_saved("docs/README.md", "# Index\n\n\u{a7}FS-a\n", "FS-a")
}

/// R06b: a dangling citation edited into inline code.
fn the_inline_code_hint_reads_the_buffer() -> Verdict {
    let files = [("docs/a.md", "# FS-a: Alpha\n\nSee \u{a7}FS-z.\n")];
    let tree = Tree::new("r06b", &config(&[]), &files);
    tree.buffer_against_saved("docs/a.md", "# FS-a: Alpha\n\nSee `\u{a7}FS-z`.\n", "FS-a")
}

/// R06c: an `AGENTS.md` managed-block edit.
fn the_agents_block_is_judged_on_the_buffer() -> Verdict {
    let files = [("docs/a.md", DECLARES), ("AGENTS.md", "Hello.\n")];
    let tree = Tree::new("r06c", &config(&[]), &files);
    let edited = "Hello.\n<!-- END GRUND MANAGED BLOCK -->\n";
    tree.buffer_against_saved("AGENTS.md", edited, "FS-a")
}

/// R06c, the companion probe: a `.rules` that no `.zed/` claims is an entrypoint
/// only on the managed block it carries, here held in the buffer alone.
fn a_companions_block_is_found_in_the_buffer() -> Verdict {
    let files = [
        ("docs/a.md", DECLARES),
        ("AGENTS.md", "Hello.\n"),
        (".rules", "Hello.\n"),
    ];
    let tree = Tree::new("r06c-companion", &config(&[]), &files);
    let edited = "Hello.\n<!-- END GRUND MANAGED BLOCK -->\n";
    tree.buffer_against_saved(".rules", edited, "FS-a")
}

/// R06d: an end-to-end case's expected exit code, edited.
fn the_e2e_manifest_is_read_from_the_buffer() -> Verdict {
    let config = "grund_config_version = 1\n[reference]\nstrict = true\n[id]\n\
        format = \"{kind}-{slug}\"\n[[kinds]]\nkind = \"E2E\"\nfolder = \"tests/e2e/cases\"\n\
        index = false\n[scan]\ninclude = [\"tests/e2e\"]\n";
    let files = [
        ("tests/e2e/cases/a/command.args", "check\n"),
        ("tests/e2e/cases/a/expected.exit", "0\n"),
    ];
    let tree = Tree::new("r06d", config, &files);
    tree.buffer_against_saved("tests/e2e/cases/a/expected.exit", "2\n", "E2E-a")
}
