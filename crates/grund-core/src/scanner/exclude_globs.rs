//! A v2 project's `[schema.sources] exclude` globs (§FS-config-v2.schema.sources.exclude),
//! compiled once and asked by every reader of what the walk reads: the walk's own
//! filter, the rule a stub's target is refused by, and the empty-scope caution
//! (§AR-scanner.1.13). One matcher, so the three cannot disagree about a path, and
//! each answer is the pattern as written, so a message can name it.

use std::path::Path;
use std::sync::Arc;

use ignore::Match;
use ignore::gitignore::{Gitignore, GitignoreBuilder};

use crate::config::Schema;

/// The compiled `exclude` list of a v2 project, matched over config-root paths.
#[derive(Clone)]
pub(super) struct ExcludeGlobs(Arc<Gitignore>);

impl ExcludeGlobs {
    /// The project's globs; `None` for v1 and for an empty list. The reader refused
    /// any pattern the builder refuses, so none fails here.
    pub(super) fn of(schema: &Schema) -> Option<Self> {
        let model = schema.sources.model.as_ref()?;
        if model.exclude.is_empty() {
            return None;
        }
        let mut builder = GitignoreBuilder::new("");
        for pattern in &model.exclude {
            builder.add_line(None, pattern).ok()?;
        }
        builder.build().ok().map(|globs| Self(Arc::new(globs)))
    }

    /// The pattern that excludes `path`, as the list writes it. A path under
    /// `config_root` is matched by its config-relative spelling; one outside it by
    /// its own name alone, so an anchored pattern never reaches outside the root
    /// while an unanchored one still applies below it
    /// (§FS-config-v2.schema.sources.exclude.roots).
    pub(super) fn excluding(&self, config_root: &Path, path: &Path, is_dir: bool) -> Option<&str> {
        let relative = match path.strip_prefix(config_root) {
            Ok(relative) => relative,
            Err(_) => Path::new(path.file_name()?),
        };
        match self.0.matched(relative, is_dir) {
            Match::Ignore(glob) => Some(glob.original()),
            Match::None | Match::Whitelist(_) => None,
        }
    }

    /// The first pattern met on the way down from `root` to `file`, a directory
    /// before what lies under it and the file last, as the walk meets them; `root`
    /// itself is read whatever names it (§FS-config-v2.schema.sources.exclude.roots).
    pub(super) fn excluding_below(
        &self,
        config_root: &Path,
        root: &Path,
        file: &Path,
    ) -> Option<&str> {
        let relative = file.strip_prefix(root).ok()?;
        let mut path = root.to_path_buf();
        let mut names = relative.iter().peekable();
        while let Some(name) = names.next() {
            path.push(name);
            if let Some(pattern) = self.excluding(config_root, &path, names.peek().is_some()) {
                return Some(pattern);
            }
        }
        None
    }
}
