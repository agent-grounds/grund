//! Authored numeric fixtures shared by snapshot and real-server regressions
//! (§FS-config.3.2.8, §FS-lsp.1.3.1).

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);
pub const NOTES: &str = "See §ADR-1 and §ADR-001.terms and §ADR-0001.\n";

pub struct Repo(pub PathBuf);

impl Repo {
    pub fn new(version: u8, per_kind: bool) -> Self {
        let root = std::env::temp_dir().join(format!(
            "grund-819-editor-v{version}-{per_kind}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
        ));
        let repo = Self(root);
        let default = if per_kind {
            "{kind}-{slug}"
        } else {
            "{kind}-{number}"
        };
        let config = if version == 1 {
            let override_format = if per_kind {
                "format = \"{kind}-{number}\"\n"
            } else {
                ""
            };
            format!(
                "grund_config_version = 1\n[reference]\nstrict = true\nrequire_grounding = false\n\
                 [id]\nformat = {default:?}\nnamed_sections = true\n\
                 [[kinds]]\nkind = \"ADR\"\nfolder = \"docs/adr\"\nindex = false\n\
                 {override_format}[scan]\ninclude = [\"docs\"]\nextensions = [\"md\"]\n\
                 respect_gitignore = false\n"
            )
        } else {
            let override_format = if per_kind {
                "id_format = \"{kind}-{number}\"\n"
            } else {
                ""
            };
            format!(
                "grund_config_version = 2\n[schema]\nid_format = {default:?}\n\
                 [schema.sources]\nlanguages = [\"markdown\"]\nrespect_gitignore = false\n\
                 [schema.kinds.ADR]\nfolders = [\"docs/adr\"]\nindex = false\n\
                 {override_format}[schema.kinds.code]\nfolders = [\"docs\"]\n"
            )
        };
        repo.write("grund.toml", &config);
        repo.write(
            "docs/adr/decision.md",
            "# ADR-0001: Decision\n\n## terms: Terms\n\nBody.\n\n\
            ### terms.detail: Detail\n\nDetails.\n\n## other: Other\n\n\
            ### other.terms: Other terms\n\nOther details.\n",
        );
        repo.write("docs/notes.md", NOTES);
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
