//! git's global `core.excludesFile`, anchored where git anchors it: at the top of
//! the repository each path belongs to (§FS-scan.scope.ignore-files.global).
//!
//! The `ignore` crate roots a walker's global matcher at one directory, the process
//! working directory unless told otherwise, and strips that root from each path
//! before it matches. Where the root does not prefix the path in the same spelling,
//! an anchored pattern such as `notes/gen/` has nothing to anchor to and matches
//! nothing: from a subdirectory, from outside the repository, under a config root
//! above it, and on Windows from anywhere, since the walk reads `\\?\` paths that no
//! working directory spells. Nor is any one root right for a walk that enters a
//! nested repository. So the root is derived here, once per path, and every place
//! that consults the global file reads it through [`GlobalExcludes`]: the walk and
//! the one-level probe walker, which leave the global file to their filter
//! ([`GlobalExcludes::take_over`]), and the rule-by-rule predicate that names the
//! ignoring rule, which asks [`GlobalExcludes::matched`] last, in git's precedence
//! (§AR-scanner.1.1).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use anyhow::Result;
use ignore::gitignore::{Gitignore, GitignoreBuilder, Glob};
use ignore::{Match, WalkBuilder};

use crate::config::global_excludes_path;
use crate::sources::{path_exists, path_is_file};

/// What marks a directory as a repository top: the set the `ignore` crate counts,
/// so the global file anchors at the top that bounds the `.gitignore` files
/// (§FS-scan.scope.ignore-files.global).
const REPOSITORY_MARKERS: [&str; 2] = [".git", ".jj"];

/// The global excludes file's patterns, the repository top each path is matched
/// against them from (§FS-scan.scope.ignore-files.global), and the ignore files
/// a global match is weighed against, each read once.
pub(super) struct GlobalExcludes {
    /// Rooted at `.`, so the matcher strips nothing: each path is handed to it
    /// relative to its own repository top.
    patterns: Gitignore,
    /// Every directory asked about, and the top of the repository the paths in it
    /// belong to, so a walk probes each directory for a repository marker once
    /// (§GOAL-fast-feedback).
    tops: Mutex<HashMap<PathBuf, Option<PathBuf>>>,
    /// Every `.ignore`, `.gitignore` and `info/exclude` asked about, parsed, and
    /// `None` where there is none: a walk re-reads the files above the global one
    /// for each entry the global file ignores, a `.DS_Store` in every directory
    /// among them, so each is parsed once per walk rather than once per entry.
    files: Mutex<HashMap<PathBuf, Option<Arc<Gitignore>>>>,
}

impl GlobalExcludes {
    /// The global excludes file git's configuration names, read once; no pattern at
    /// all where none is configured. The shared resolver preserves literal
    /// tildes (§FS-scan.scope.ignore-files.pathname); `add` reads that pathname
    /// directly, without the dependency's global-path expansion.
    pub(super) fn load() -> Result<Self> {
        let mut builder = GitignoreBuilder::new(".");
        if let Some(path) = global_excludes_path()?
            && path_is_file(&path)
        {
            // Preserve the existing policy for partial pattern/read errors;
            // watch-coverage errors above propagate, and an unresolved pathname
            // is never retried through the dependency's global loader.
            let _ = builder.add(path);
        }
        Ok(Self {
            patterns: builder.build()?,
            tops: Mutex::default(),
            files: Mutex::default(),
        })
    }

    /// The global excludes a walker leaves to its filter, with the builder's own
    /// matcher, rooted at one directory, turned off even for an empty or absent
    /// file: otherwise its corrupt pathname expansion can consult a different
    /// file (§FS-scan.scope.ignore-files.pathname). `None` where there are no
    /// patterns, so a machine without one pays nothing per entry (§GOAL-fast-feedback).
    pub(super) fn take_over(builder: &mut WalkBuilder) -> Result<Option<Self>> {
        builder.git_global(false);
        let global = Self::load()?;
        if global.patterns.is_empty() {
            return Ok(None);
        }
        Ok(Some(global))
    }

    /// The global pattern that decides `path`, matched relative to the top of its
    /// repository, and `Match::None` for a path in no repository, as git matches
    /// none there (§FS-scan.scope.ignore-files.global).
    pub(super) fn matched(&self, path: &Path, is_dir: bool) -> Match<&Glob> {
        if self.patterns.is_empty() {
            return Match::None;
        }
        let Some(top) = self.top(path) else {
            return Match::None;
        };
        match path.strip_prefix(&top) {
            Ok(relative) => self.patterns.matched(relative, is_dir),
            Err(_) => Match::None,
        }
    }

    /// The top of the repository `path` belongs to: the innermost directory above it
    /// that holds one of the [`REPOSITORY_MARKERS`], `.git` or a Jujutsu `.jj`, so a
    /// nested repository's paths anchor at its own top
    /// (§FS-scan.scope.ignore-files.global). It is one of `path`'s ancestors, so it
    /// is spelled as `path` is, `\\?\` included, and always prefixes it.
    pub(super) fn top(&self, path: &Path) -> Option<PathBuf> {
        let mut tops = self.tops.lock().unwrap_or_else(PoisonError::into_inner);
        let mut unknown = Vec::new();
        let mut top = None;
        for dir in path.ancestors().skip(1) {
            if dir.as_os_str().is_empty() {
                break;
            }
            if let Some(known) = tops.get(dir) {
                top.clone_from(known);
                break;
            }
            unknown.push(dir);
            if REPOSITORY_MARKERS
                .iter()
                .any(|marker| path_exists(dir.join(marker)))
            {
                top = Some(dir.to_path_buf());
                break;
            }
        }
        for dir in unknown {
            tops.insert(dir.to_path_buf(), top.clone());
        }
        top
    }

    /// The ignore file at `file`, its patterns rooted at `dir`, or `None` where no
    /// such file is there or it does not parse.
    pub(super) fn ignore_file(&self, dir: &Path, file: PathBuf) -> Option<Arc<Gitignore>> {
        let mut files = self.files.lock().unwrap_or_else(PoisonError::into_inner);
        files
            .entry(file)
            .or_insert_with_key(|file| {
                if !path_is_file(file) {
                    return None;
                }
                let mut builder = GitignoreBuilder::new(dir);
                if builder.add(file).is_some() {
                    return None;
                }
                builder.build().ok().map(Arc::new)
            })
            .clone()
    }
}
