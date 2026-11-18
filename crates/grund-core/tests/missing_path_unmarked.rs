//! §FS-scan.scope.homes.lookup.path.library: a library caller's path whose lookup says nothing
//! stands there is refused with `path does not exist: <path>`, whatever `path_provided` says.
//! Ported from tool report grund.308's reproducer (agent-grounds/grund#1443): an embedder that
//! sets only `path` got `Ok`, the check of the project the missing path's parent sits in.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use grund_core::{
    CheckOpts, CompleteIdsOpts, CoverOpts, FmtOpts, IdOpts, ListOpts, ListSizeOpts,
    LspSnapshotOpts, RefsOpts, check_with_opts, complete_ids, cover, cover_text, format_references,
    list, list_sizes, lsp_snapshot, propose_id, refs, refs_outcome,
};

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Repo(PathBuf);

impl Repo {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "grund-1443-missing-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let repo = Self(root);
        repo.write(
            "grund.toml",
            "grund_config_version = 1\n\
             [id]\nformat = \"{kind}-{number}-{slug}\"\n\
             [scan]\ninclude = [\"docs\"]\nrespect_gitignore = false\n\
             [[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n",
        );
        repo.write("docs/FS-001-alpha.md", "# FS-001-alpha: Alpha\n\nBody.\n");
        repo
    }

    fn write(&self, path: &str, text: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    /// A name inside the project that nothing stands at: its parent is the project root, so
    /// configuration discovery from it finds the project, as `nope` did from the reporter's cwd.
    fn missing(&self) -> PathBuf {
        self.0.join("nope")
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// The refusal the CLI prints for the same path (§FS-scan.scope.homes.lookup.path).
fn assert_refused<T: std::fmt::Debug>(entry: &str, path: &Path, result: anyhow::Result<T>) {
    let expected = format!("path does not exist: {}", path.display());
    match result {
        Err(error) => assert_eq!(error.to_string(), expected, "{entry}: wrong refusal"),
        Ok(answer) => panic!(
            "{entry}: returned Ok for a path that does not exist, unmarked \
             (path_provided = false); expected Err({expected:?}); got {answer:?}"
        ),
    }
}

#[test]
fn check_with_opts_refuses_missing_unmarked_path() {
    let repo = Repo::new();
    let mut opts = CheckOpts::default();
    opts.path = repo.missing();
    let result = check_with_opts(opts).map(|out| (out.report.errors.len(), out.had_scan_errors));
    assert_refused("check_with_opts", &repo.missing(), result);
}

#[test]
fn list_refuses_missing_unmarked_path() {
    let repo = Repo::new();
    let mut opts = ListOpts::default();
    opts.path = repo.missing();
    let result = list(opts).map(|out| out.entries.len());
    assert_refused("list", &repo.missing(), result);
}

#[test]
fn list_sizes_refuses_missing_unmarked_path() {
    let repo = Repo::new();
    let mut opts = ListSizeOpts::default();
    opts.path = repo.missing();
    let result = list_sizes(opts).map(|out| out.entries.len());
    assert_refused("list_sizes", &repo.missing(), result);
}

#[test]
fn refs_refuses_missing_unmarked_path() {
    let repo = Repo::new();
    let mut opts = RefsOpts::default();
    opts.path = repo.missing();
    opts.id = "FS-001-alpha".into();
    let result = refs(opts.clone()).map(|out| out.hits.len());
    assert_refused("refs", &repo.missing(), result);
    let result = refs_outcome(opts).map(|out| out.output.hits.len());
    assert_refused("refs_outcome", &repo.missing(), result);
}

#[test]
fn format_references_refuses_missing_unmarked_path() {
    let repo = Repo::new();
    let mut opts = FmtOpts::default();
    opts.path = repo.missing();
    let result = format_references(opts).map(|out| out.changes.len());
    assert_refused("format_references", &repo.missing(), result);
}

#[test]
fn cover_refuses_missing_unmarked_path() {
    let repo = Repo::new();
    let mut opts = CoverOpts::default();
    opts.path = repo.missing();
    let result = cover(opts.clone()).map(|out| out.entries.len());
    assert_refused("cover", &repo.missing(), result);
    let result = cover_text(opts).map(|out| out.entries.len());
    assert_refused("cover_text", &repo.missing(), result);
}

#[test]
fn complete_ids_refuses_missing_unmarked_path() {
    let repo = Repo::new();
    let mut opts = CompleteIdsOpts::default();
    opts.path = repo.missing();
    opts.prefix = "FS-".into();
    assert_refused("complete_ids", &repo.missing(), complete_ids(opts));
}

#[test]
fn propose_id_refuses_missing_unmarked_path() {
    let repo = Repo::new();
    let mut opts = IdOpts::default();
    opts.path = repo.missing();
    let result = propose_id("FS", "Beta", opts).map(|_| ());
    assert_refused("propose_id", &repo.missing(), result);
}

#[test]
fn lsp_snapshot_refuses_missing_unmarked_path() {
    let repo = Repo::new();
    let mut opts = LspSnapshotOpts::default();
    opts.path = repo.missing();
    let result = lsp_snapshot(opts).map(|_| ());
    assert_refused("lsp_snapshot", &repo.missing(), result);
}

/// The guard: a path that exists keeps its meaning unmarked — the anchor the project is found
/// from, the whole project checked and listed — and the default `.` is never refused.
#[test]
fn existing_unmarked_path_keeps_its_meaning() {
    let repo = Repo::new();
    let docs = repo.0.join("docs");

    let mut opts = CheckOpts::default();
    opts.path = docs.clone();
    let out = check_with_opts(opts).expect("an existing unmarked path is checked");
    assert!(!out.had_scan_errors, "{:?}", out.report);
    assert!(out.report.errors.is_empty(), "{:?}", out.report.errors);

    let mut opts = ListOpts::default();
    opts.path = docs;
    let ids: Vec<String> = list(opts)
        .expect("an existing unmarked path is listed")
        .entries
        .into_iter()
        .map(|entry| entry.id)
        .collect();
    assert_eq!(ids, ["FS-001-alpha"]);
}
