//! Shared regression fixture for §FS-scan.path.refs and §FS-config.3.2.6.
//! Ported from grund.166's Haskell reproducer, with a separate citing subtree.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

pub struct Repo(pub PathBuf);

impl Repo {
    pub fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "grund-1126-refs-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let repo = Self(root);
        repo.write("grund.toml", CONFIG);
        repo.write(
            "src/A.hs",
            "-- | FS-BadHaskell: per \u{a7}FS-001-alpha\n-- | ## 1. Detail\n-- | Body.\n",
        );
        repo.write("docs/FS-001-alpha.md", "# FS-001-alpha: Alpha\n");
        repo.write(
            "hits/citer.md",
            "See \u{a7}FS-BadHaskell and \u{a7}FS-BadHaskell.1 and \u{a7}FS-001-alpha.\n",
        );
        repo.write("empty/nothing.md", "No citations here.\n");
        repo
    }

    pub fn write(&self, path: &str, text: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub const CONFIG: &str = "grund_config_version = 1\n\
    [id]\nformat = \"{kind}-{number}-{slug}\"\n\
    [scan]\ninclude = [\"docs\", \"src\", \"hits\", \"empty\"]\n\
    respect_gitignore = false\n\
    [[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n";
