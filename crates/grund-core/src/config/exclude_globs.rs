//! The `[schema.sources] exclude` glob grammar (§FS-config-v2.schema.sources.exclude),
//! compiled once and asked by every reader of what the walk reads: the walk's own
//! filter, the rule a stub's target is refused by, and the empty-scope caution
//! (§AR-scanner.1.13). One matcher, so the three cannot disagree about a path, and
//! each answer is the pattern as written, so a message can name it. It lives in
//! config because migration asks it of a version 1 entry and the reader refuses with
//! its check, and config may not read the scanner (§AR-config.8.1, §AR-system.4).

use std::path::Path;
use std::sync::Arc;

use ignore::Match;
use ignore::gitignore::{Gitignore, GitignoreBuilder};

use crate::config::Schema;

/// Add `pattern` to `builder`, or say why the reader refuses it
/// (§FS-config-v2.schema.sources.exclude).
pub(super) fn add_pattern(builder: &mut GitignoreBuilder, pattern: &str) -> Result<(), String> {
    if unclosed_class(pattern) {
        return Err("unclosed character class".to_string());
    }
    builder
        .add_line(None, pattern)
        .map(|_| ())
        .map_err(|err| err.to_string())
}

/// Whether `pattern` opens a `[` class it never closes. The glob builder reads
/// such a pattern as a literal, where git matches nothing with it, so the reader
/// refuses it as the malformed pattern it is (§FS-config-v2.schema.sources.exclude).
fn unclosed_class(pattern: &str) -> bool {
    let chars: Vec<char> = pattern.chars().collect();
    let mut at = 0;
    while at < chars.len() {
        match chars[at] {
            '\\' => at += 1,
            '[' => {
                let mut end = at + 1;
                if matches!(chars.get(end), Some('!' | '^')) {
                    end += 1;
                }
                // A `]` first in the class is a member, not its close.
                if chars.get(end) == Some(&']') {
                    end += 1;
                }
                match chars[end.min(chars.len())..].iter().position(|c| *c == ']') {
                    Some(close) => at = end + close,
                    None => return true,
                }
            }
            _ => {}
        }
        at += 1;
    }
    false
}

/// The path `pattern` spells: no trailing spaces, and no `/` at either end
/// (§FS-config-migrate.3.1.4.1).
pub(super) fn spelled_path(pattern: &str) -> &str {
    pattern.trim_end().trim_matches('/')
}

/// Whether `pattern` can match only the path it spells: a leading or inner `/` anchors
/// it, and it holds no glob syntax (§FS-config-migrate.3.1.4.1).
pub(super) fn matches_only_its_path(pattern: &str) -> bool {
    let line = pattern.trim_end().trim_end_matches('/');
    line.contains('/') && !line.contains(['*', '?', '[', '{', '\\'])
}

/// The compiled `exclude` list of a v2 project, matched over config-root paths.
#[derive(Clone)]
pub(crate) struct ExcludeGlobs(Arc<Gitignore>);

impl ExcludeGlobs {
    /// The project's globs; `None` for v1 and for an empty list. The reader refused
    /// any pattern the builder refuses, so none fails here.
    pub(crate) fn of(schema: &Schema) -> Option<Self> {
        let model = schema.sources.model.as_ref()?;
        if model.exclude.is_empty() {
            return None;
        }
        Self::compile(&model.exclude).map(Self::from)
    }

    /// `pattern` read alone (§AR-config.8.1); `None` for one that so stops nothing: a
    /// comment, a negation with no earlier line to rescue from, and a pattern the reader
    /// refuses, which migration's tolerant read let through (§FS-config-migrate.3.1.4.1).
    pub(super) fn alone(pattern: &str) -> Option<Self> {
        Self::compile(std::slice::from_ref(&pattern))
            .filter(|globs| globs.num_ignores() > 0)
            .map(Self::from)
    }

    fn compile(patterns: &[impl AsRef<str>]) -> Option<Gitignore> {
        let mut builder = GitignoreBuilder::new("");
        for pattern in patterns {
            add_pattern(&mut builder, pattern.as_ref()).ok()?;
        }
        builder.build().ok()
    }

    /// The pattern that excludes `path`, as the list writes it. A path under
    /// `config_root` is matched by its config-relative spelling; one outside it by
    /// its own name alone, so an anchored pattern never reaches outside the root
    /// while an unanchored one still applies below it
    /// (§FS-config-v2.schema.sources.exclude.roots).
    pub(crate) fn excluding(&self, config_root: &Path, path: &Path, is_dir: bool) -> Option<&str> {
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
    pub(crate) fn excluding_below(
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

impl From<Gitignore> for ExcludeGlobs {
    fn from(globs: Gitignore) -> Self {
        Self(Arc::new(globs))
    }
}
