//! What a snapshot's judge keeps so that a keystroke is judged from the edited document alone
//! (§FS-lsp.responsiveness.4, §AR-lsp.3.2): every file each project walked, under the path the
//! editor names it by, and every file the one-file reading cannot judge exactly. Both are settled
//! where the snapshot is built, beside its scan, so the judgement resolves and reads nothing of the
//! tree.

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::resolver::WorkspaceContext;
use crate::scanner::file_again_keeps_declarations;
use crate::sources::{Buffers, canonical_snapshot_path, stub_target_candidates};
use crate::workspace::WorkspaceCitationTarget;

use super::LspShowContext;

/// The names a reading of one file cannot judge whatever they hold: a config, which every file's
/// reading follows, and an ignore file, which decides what the walk reads (§FS-lsp.responsiveness.4).
const CONFIG_NAMES: [&str; 3] = ["grund.toml", ".gitignore", ".ignore"];

/// A snapshot's judge (§AR-lsp.3.2), built from its show context where the snapshot is built,
/// and asked only about that same context.
pub struct TypingJudge {
    /// Per project of the context, in its order: each file its walk read, keyed by its canonical
    /// path, with the spelling the walk read it by.
    walked: Vec<BTreeMap<PathBuf, PathBuf>>,
    /// Canonical paths whose declarations a one-file reading cannot judge: a stub's file, both
    /// files its link can name, and a file the scan failed on.
    unjudged: BTreeSet<PathBuf>,
    /// Canonical case directories, whose every file a case's declaration reads.
    cases: Vec<PathBuf>,
    targets: Vec<WorkspaceCitationTarget>,
}

impl TypingJudge {
    fn of(context: &WorkspaceContext) -> Self {
        let mut unjudged = BTreeSet::new();
        let mut cases = Vec::new();
        let walked = context
            .projects
            .iter()
            .map(|project| {
                let root = &project.config.root;
                for decl in project.findings.declarations.values().flatten() {
                    if decl.is_stub {
                        unjudged.insert(canonical_snapshot_path(&root.join(&decl.file)));
                        if let Some(target) = &decl.defined_in {
                            unjudged.extend(
                                stub_target_candidates(root, &decl.file, target)
                                    .iter()
                                    .map(|path| canonical_snapshot_path(path)),
                            );
                        }
                    }
                    if let Some(case) = &decl.e2e_case {
                        cases.push(canonical_snapshot_path(&root.join(&case.dir)));
                    }
                }
                unjudged.extend(
                    project
                        .scan_errors
                        .iter()
                        .map(|failure| canonical_snapshot_path(&root.join(&failure.path))),
                );
                project
                    .findings
                    .scanned_files
                    .iter()
                    .map(|path| (canonical_snapshot_path(&root.join(path)), path.clone()))
                    .collect()
            })
            .collect();
        Self {
            walked,
            unjudged,
            cases,
            targets: context.citation_targets(),
        }
    }
}

impl LspShowContext {
    /// This snapshot's judge, built where the snapshot is, never on a keystroke: it resolves
    /// every walked path once (§AR-lsp.3.2).
    pub fn typing_judge(&self) -> TypingJudge {
        TypingJudge::of(&self.context)
    }
}

impl TypingJudge {
    /// Whether `text` in place of `path` leaves the declarations `show` holds for it standing
    /// (§FS-lsp.responsiveness.4): `Some(true)` where every project that walked `path` reads the
    /// same declarations from `text` as its catalog holds, `Some(false)` where one does not or the
    /// one-file reading cannot judge the file exactly, and `None` where no project of `show`
    /// walked it or names it. `show` is the context this judge was built from. Reads `text` and
    /// nothing else (§AR-lsp.3.2).
    pub fn declarations_stand(
        &self,
        show: &LspShowContext,
        path: &Path,
        text: &str,
    ) -> Option<bool> {
        let judge = self;
        let key = canonical_snapshot_path(path);
        let name = key.file_name().and_then(OsStr::to_str).unwrap_or_default();
        let unjudged = CONFIG_NAMES.contains(&name)
            || judge.unjudged.contains(&key)
            || judge.cases.iter().any(|dir| key.starts_with(dir));
        let mut read = None;
        for (project, walked) in show.context.projects.iter().zip(&judge.walked) {
            let Some(spelling) = walked.get(&key) else {
                continue;
            };
            if unjudged {
                return Some(false);
            }
            let view = project
                .config
                .with_buffers(Buffers::new([(spelling.clone(), text.to_string())]));
            let stands = file_again_keeps_declarations(
                &project.findings,
                spelling,
                view.schema(),
                view.frame(),
                &judge.targets,
            );
            if !stands {
                return Some(false);
            }
            read = Some(true);
        }
        // §FS-lsp.responsiveness.4: a stub's target this snapshot never walked is still its input.
        read.or_else(|| judge.unjudged.contains(&key).then_some(false))
    }
}
