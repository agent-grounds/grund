//! The two scopes a repository takes out of `fmt`'s reach (§FS-fmt.2.5): the
//! `[fmt] exclude` list, and the `grund:fmt` regions written in the files.
//! Beside the walk rather than in it, per §AR-core-module-layout.3's file
//! budget.
//!
//! The glob compiler these read and the validator the config reader refused a
//! malformed pattern with went down into `config/fmt_block.rs` when
//! §AR-system.2.8 became a module: the pattern grammar of §FS-config.3.10 is the
//! `[fmt]` section's, and the matcher below reads it downward (§AR-system.4).

use anyhow::{Result, anyhow};
use ignore::gitignore::Gitignore;
use std::path::{Path, PathBuf};

use crate::config::{Config, build_fmt_exclude_matcher};
use crate::grammar::{DocstringContent, comment_strip_prefixes, strip_comment_tokens};
use crate::model::canonical_snapshot_path;

/// The fixed text of a suppression directive (§FS-fmt.2.5.2). Not configurable,
/// for the same reason the fence syntax is not: a marker that reads differently
/// per repository is one nobody can recognize on sight
/// (§DF-fmt-suppression.2.2).
pub(super) const FMT_DIRECTIVE: &str = "grund:fmt";

/// The files this project's `[fmt] exclude` takes out of every rewrite
/// (§FS-fmt.2.5.1). Empty — and free — for the repositories that set no key,
/// which is why the matcher is an `Option` rather than an empty `Gitignore`:
/// `matched_path_or_any_parents` walks a path's ancestors, and a run with no
/// patterns should not pay for that on every file (§GOAL-fast-feedback).
pub(crate) struct FmtExcluded {
    root: PathBuf,
    matcher: Option<Gitignore>,
}

impl FmtExcluded {
    /// The matcher for one project's config. Patterns were already validated at
    /// load (§FS-config.3.10), so a failure here is a grund bug rather than a
    /// user error — it is still reported rather than swallowed, because the
    /// alternative is a `--write` that silently rewrites a protected file.
    pub(crate) fn new(config: &Config) -> Result<Self> {
        let matcher = if config.fmt_exclude.is_empty() {
            None
        } else {
            Some(
                build_fmt_exclude_matcher(&config.fmt_exclude)
                    .map_err(|message| anyhow!("[fmt] exclude: {message}"))?,
            )
        };
        Ok(Self {
            root: config.root.clone(),
            matcher,
        })
    }

    /// Whether `path` is excluded. The patterns are config-root-relative
    /// (§FS-config.3.10), so the walk's path is rebased before matching and a
    /// path that is not under the root — nothing the walk produces today — is
    /// simply not excluded rather than guessed about.
    pub(crate) fn contains(&self, path: &Path) -> bool {
        let Some(matcher) = &self.matcher else {
            return false;
        };
        let Some(relative) = self.rebase(path) else {
            return false;
        };
        matcher
            .matched_path_or_any_parents(&relative, false)
            .is_ignore()
    }

    /// `path` as the patterns see it: relative to the config root. The walk builds
    /// every path out of the root itself, so the prefix strips outright and
    /// nothing further is asked of it (§GOAL-fast-feedback).
    ///
    /// An editor does not. The LSP hands in the path its client opened, and the
    /// config root was canonicalized when it was loaded (§FS-config.1), so the two
    /// name one file in two spellings wherever a symlink stands between them — a
    /// `/var` `$TMPDIR` on macOS, a short filename on Windows, any repository
    /// reached through a link — and a strip of the literal prefix would let the
    /// rewrite through in exactly the file the config took out of its reach. So
    /// the strip is retried with both sides resolved the way the LSP resolves a
    /// request URI (§AR-lsp.5), which does not require the file to exist yet. A
    /// path genuinely outside the root is still not excluded rather than guessed
    /// about.
    fn rebase(&self, path: &Path) -> Option<PathBuf> {
        if let Ok(relative) = path.strip_prefix(&self.root) {
            return Some(relative.to_path_buf());
        }
        canonical_snapshot_path(path)
            .strip_prefix(canonical_snapshot_path(&self.root))
            .ok()
            .map(Path::to_path_buf)
    }
}

/// The `grund:fmt off` / `grund:fmt on` region state for one file
/// (§FS-fmt.2.5.2): whether the rewrite is on at the line about to be read, and
/// how a directive is spelled in this file's syntax. Every file starts with the
/// rewrite on — nothing carries across files.
pub(crate) struct FmtDirectives<'a> {
    rewriting: bool,
    /// `None` in Markdown, where the directive is an HTML comment. In a source
    /// file, the comment prefixes to strip — built once per file rather than
    /// once per line, because `comment_strip_prefixes` allocates and sorts
    /// (§GOAL-fast-feedback).
    prefixes: Option<Vec<&'a str>>,
}

impl<'a> FmtDirectives<'a> {
    pub(crate) fn new(config: &'a Config, is_md: bool) -> Self {
        Self {
            rewriting: true,
            prefixes: (!is_md).then(|| comment_strip_prefixes(config)),
        }
    }

    /// Take `line` when it is a directive, returning whether it was one. A
    /// directive line is never rewritten, whichever state it leaves behind
    /// (§FS-fmt.2.5.2) — so the caller passes it through on `true`.
    pub(crate) fn consume(&mut self, line: &str, docstring: DocstringContent<'_>) -> bool {
        match self.directive(line, docstring) {
            Some(rewriting) => {
                // A redundant directive is a no-op: assigning the state it
                // already holds is exactly that (§FS-fmt.2.5.2).
                self.rewriting = rewriting;
                true
            }
            None => false,
        }
    }

    /// Whether the rewrite is on for the line about to be read.
    pub(crate) fn rewriting(&self) -> bool {
        self.rewriting
    }

    /// The state this line asks for, if it is a directive at all. Only an exact
    /// content match counts (§FS-fmt.2.5.2): `grund:fmt-off` and
    /// `grund:fmt off please` are ordinary comments.
    pub(crate) fn directive(&self, line: &str, docstring: DocstringContent<'_>) -> Option<bool> {
        // The cheap gate first — `fmt` asks this of every line of every scanned
        // file, and almost none of them carry the text (§GOAL-fast-feedback).
        if !line.contains(FMT_DIRECTIVE) {
            return None;
        }
        let content = match &self.prefixes {
            None => line
                .trim()
                .strip_prefix("<!--")?
                .strip_suffix("-->")?
                .trim(),
            Some(prefixes) => {
                let text = docstring.text_of(line);
                // A docstring line is documentation and carries no prefix of its
                // own (§FS-fmt.2.3.1); every other source line must actually be
                // a comment, or a string holding this text would toggle a region.
                if !docstring.is_docstring()
                    && !prefixes
                        .iter()
                        .any(|prefix| text.trim_start().starts_with(prefix))
                {
                    return None;
                }
                strip_comment_tokens(text, prefixes)
            }
        };
        match content.strip_prefix(FMT_DIRECTIVE)?.trim() {
            "off" => Some(false),
            "on" => Some(true),
            _ => None,
        }
    }
}
