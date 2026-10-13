//! The one predicate that says whether a run reads a file, and which rule
//! refuses it where it does not (§AR-scanner.1.13). It is the walk's own
//! filter asked of one path instead of a descent: the name and extension test
//! the walk applies to every file, `WalkDirFilter::refusal` for each directory
//! between the scan root and the file, and the `ignore` crate's matcher for the
//! ignore files, so the rules have one copy rather than one per reader.
//!
//! Four readers ask it. The walk asks the name test of each entry and the
//! filter of each directory. A file that exists only as a buffer, which the
//! walk never meets on disk, is read or skipped by `unread`, exactly as the
//! saved file would be (§AR-system.2.13). A stub's target asks
//! `name_refusal` for its verdict, the two reasons §FS-declarations.stubs.target.3
//! honours, and `excluding_rule` for the warning that the exclude and ignore-file
//! reasons decide it from grund 0.19.0. And a link the walker could not resolve asks `name_refusal` or
//! `directory_refusal`, with `ignored`, whether the walk would have read it
//! (§FS-config.3.5.6), since the walker filters no error entry itself.

use std::path::{Component, Path, PathBuf};

use ignore::{
    Match, WalkBuilder,
    gitignore::{GitignoreBuilder, Glob},
};

use super::walk::WalkDirFilter;
use crate::config::{Frame, Schema, canonical_config_root};
use crate::model::{ExcludingRule, format_path, is_hidden};
use crate::sources::{
    canonicalize_existing_prefix, path_exists, path_is_dir, path_is_entry, path_is_file,
    path_is_symlink, read_git_metadata,
};

/// Why a run does not read a file (§AR-scanner.1.13).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Unread {
    /// A hidden file name (§AR-scanner.1.5).
    HiddenName,
    /// An extension `[scan] extensions` does not list (§AR-scanner.1.12).
    Extension,
    /// A hidden directory, or one `[scan] exclude` names (§AR-scanner.1.2).
    ExcludedDirectory,
    /// An ignore file (§AR-scanner.1.1).
    IgnoreFile,
    /// Outside every scan root (§AR-scanner.1.6).
    OutsideRoots,
    /// Inside a home the config lists without walking (§AR-scanner.1.4).
    UnwalkedHome,
    /// Inside another project's root (§AR-scanner.1.3).
    AnotherProject,
    /// Inside an end-to-end case directory, which the manifest pass owns
    /// (§AR-scanner.6.4).
    E2eCase,
    /// Behind a directory link that loops back over the scan root (§AR-scanner.1.9).
    LoopingLink,
    /// A link the run's input observer refused to cover (§FS-check.6.1.1).
    Unobserved,
}

/// The name and extension reasons, which hold wherever the file lies: all a
/// stub's target honours (§FS-declarations.stubs.target.3), and the test
/// the walk asks of every file it yields (§FS-config.3.5).
pub(crate) fn name_refusal(path: &Path, schema: &Schema) -> Option<Unread> {
    if is_hidden(path) {
        return Some(Unread::HiddenName);
    }
    extension_refusal(path, schema)
}

/// The extension reason alone, for the reader that has to know whether the list
/// would have read a file the hidden rule refused (§FS-check.2.2.2): the one copy of
/// the extension test.
pub(crate) fn extension_refusal(path: &Path, schema: &Schema) -> Option<Unread> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let listed = schema
        .sources
        .extensions
        .iter()
        .any(|allowed| allowed == ext);
    (!listed).then_some(Unread::Extension)
}

/// The name reasons a directory is not descended: a hidden name, or one
/// `[scan] exclude` lists (§AR-scanner.1.2). The walk's filter asks it of every
/// directory, and a looping link the walker reports instead of yielding asks it
/// of the link.
pub(crate) fn directory_refusal(path: &Path, exclude: &[String]) -> Option<Unread> {
    (is_hidden(path) || excluded_entry(path, exclude).is_some())
        .then_some(Unread::ExcludedDirectory)
}

/// The `[scan] exclude` entry that names `path`'s own directory name
/// (§FS-config.3.5.16), as the list writes it.
fn excluded_entry<'a>(path: &Path, exclude: &'a [String]) -> Option<&'a str> {
    let name = path.file_name().and_then(|name| name.to_str())?;
    exclude
        .iter()
        .find(|item| *item == name)
        .map(String::as_str)
}

/// The `[scan] exclude` entry or ignore-file rule that keeps the walk from reading a
/// stub's target, which until grund 0.19.0 is a warning and not a verdict
/// (§FS-declarations.stubs.target.3, §FS-declarations.checks.excluded-stub-target).
/// Asked as the walk asks it, from the config root in place of `include` and from each
/// of `roots`, so a root that reads the target outruns the rule (§AR-scanner.1.7);
/// the config root's answer names the rule. A target outside the config root is no
/// reason, and nor is a hidden directory, another project or an unwalked home
/// (§AR-scanner.1.13).
pub(crate) fn excluding_rule(
    schema: &Schema,
    frame: Frame<'_>,
    roots: &[PathBuf],
    path: &Path,
) -> Option<ExcludingRule> {
    let file = canonicalize_existing_prefix(path);
    let physical_root = canonical_config_root(frame.root());
    if !file.starts_with(&physical_root) {
        return None;
    }
    let mut refusal = None;
    let roots = roots.iter().map(|root| canonicalize_existing_prefix(root));
    for root in std::iter::once(physical_root.clone()).chain(roots) {
        let Ok(relative) = file.strip_prefix(&root) else {
            continue;
        };
        let rule = excluding_rule_under(schema, &physical_root, &root, relative)?;
        refusal.get_or_insert(rule);
    }
    refusal
}

/// `excluding_rule` for one root `relative` lies below: the first rule met on the way
/// down, and at one directory the `exclude` entry before the ignore files, the
/// walk's own order (§FS-declarations.checks.excluded-stub-target).
fn excluding_rule_under(
    schema: &Schema,
    physical_root: &Path,
    root: &Path,
    relative: &Path,
) -> Option<ExcludingRule> {
    let names: Vec<_> = relative
        .components()
        .filter_map(|component| match component {
            Component::Normal(name) => Some(name),
            _ => None,
        })
        .collect();
    let mut path = root.to_path_buf();
    for (depth, name) in names.iter().enumerate() {
        path.push(name);
        let is_dir = depth + 1 < names.len();
        // §FS-declarations.stubs.target.3: a hidden directory is no reason in this release.
        if is_dir
            && !is_hidden(&path)
            && let Some(entry) = excluded_entry(&path, &schema.sources.exclude)
        {
            return Some(ExcludingRule::Exclude(entry.to_string()));
        }
        if schema.sources.respect_gitignore
            && let Some(ignored) = ignoring_rule(&path, is_dir)
        {
            let file = ignored
                .file
                .strip_prefix(physical_root)
                .map_or_else(|_| format_path(&ignored.file), format_path);
            return Some(ExcludingRule::IgnoreFile {
                file,
                pattern: ignored.pattern,
            });
        }
    }
    None
}

/// Whether the walk from `roots` reads `path`, and which rule refuses it where
/// it does not: asked of a file the walk never meets on disk, a buffer with
/// nothing behind it (§AR-system.2.13). A scan root outruns the
/// descent rules here as it does in the walk (§AR-scanner.1.7).
pub(super) fn unread(
    schema: &Schema,
    frame: Frame<'_>,
    roots: &[PathBuf],
    path: &Path,
) -> Option<Unread> {
    if let Some(reason) = name_refusal(path, schema) {
        return Some(reason);
    }
    let file = canonicalize_existing_prefix(path);
    let physical_root = canonical_config_root(frame.root());
    let mut refusal = Some(Unread::OutsideRoots);
    for root in roots {
        let canonical_root = canonicalize_existing_prefix(root);
        let Ok(relative) = file.strip_prefix(&canonical_root) else {
            continue;
        };
        if relative.as_os_str().is_empty() {
            return None;
        }
        refusal = under_root(schema, frame, &canonical_root, &physical_root, relative);
        if refusal.is_none() {
            return None;
        }
    }
    refusal
}

/// `unread` for one root `relative` lies below: the walk's filter for each
/// directory on the way down and for the file, then the ignore files.
fn under_root(
    schema: &Schema,
    frame: Frame<'_>,
    root: &Path,
    physical_root: &Path,
    relative: &Path,
) -> Option<Unread> {
    let link_roots = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let looping_links = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let filter = WalkDirFilter::new(
        schema,
        frame,
        root,
        root,
        physical_root,
        &link_roots,
        &looping_links,
    );
    let names: Vec<_> = relative
        .components()
        .filter_map(|component| match component {
            Component::Normal(name) => Some(name),
            _ => None,
        })
        .collect();
    let mut path = root.to_path_buf();
    for (depth, name) in names.iter().enumerate() {
        path.push(name);
        let is_dir = depth + 1 < names.len();
        let is_link = path_is_symlink(&path);
        if let Some(reason) = filter.refusal(&path, depth + 1, is_dir, is_link) {
            return Some(reason);
        }
        if schema.sources.respect_gitignore && ignored(&path, is_dir) {
            return Some(Unread::IgnoreFile);
        }
    }
    None
}

/// Whether the ignore files the walk honours drop `path` (§AR-scanner.1.1).
/// Where the entry is on disk the walker itself answers, one level deep from its
/// parent, its parents' ignore files and the global ones included, which is
/// also how `walk_errors.rs` asks about a link the walker could not resolve;
/// where it is not, the same files are
/// matched against the name in the walker's order of precedence.
pub(super) fn ignored(path: &Path, is_dir: bool) -> bool {
    walker_ignores(path).unwrap_or_else(|| ignored_by_rules(path, is_dir).is_some())
}

/// The ignore file and the line of it that drop `path`, for a reader that names the
/// rule (§FS-declarations.checks.excluded-stub-target): the rule `ignored` matches,
/// kept only where the walker, asked of an entry on disk, drops it too.
fn ignoring_rule(path: &Path, is_dir: bool) -> Option<IgnoredBy> {
    let rule = ignored_by_rules(path, is_dir)?;
    (walker_ignores(path) != Some(false)).then_some(rule)
}

/// The walker's own answer for an entry on disk, `None` where nothing is there.
fn walker_ignores(path: &Path) -> Option<bool> {
    let Some(parent) = path.parent() else {
        return Some(false);
    };
    if !path_is_entry(path) {
        return None;
    }
    let mut builder = WalkBuilder::new(parent);
    builder.hidden(false).max_depth(Some(1)).follow_links(false);
    Some(
        !builder
            .build()
            .filter_map(Result::ok)
            .any(|entry| entry.path() == path),
    )
}

/// An ignore file's rule that drops a path: the file and the line as written.
struct IgnoredBy {
    file: PathBuf,
    pattern: String,
}

/// What one matcher decides: `None` where it says nothing, `Some(None)` where it
/// whitelists, `Some(Some(rule))` where `rule` ignores.
fn decision(found: Match<&Glob>) -> Option<Option<IgnoredBy>> {
    match found {
        Match::Ignore(glob) => Some(Some(IgnoredBy {
            file: glob.from().map(Path::to_path_buf).unwrap_or_default(),
            pattern: glob.original().to_string(),
        })),
        Match::Whitelist(_) => Some(None),
        Match::None => None,
    }
}

/// The walker's precedence for a name, and the rule that ignores it: `.ignore`
/// files, deepest first; then, inside a git repository, its `.gitignore` files up to
/// the repository root, its `info/exclude`, and the global excludes.
fn ignored_by_rules(path: &Path, is_dir: bool) -> Option<IgnoredBy> {
    let dirs: Vec<&Path> = path
        .ancestors()
        .skip(1)
        .filter(|dir| path_is_dir(dir))
        .collect();
    let repository = dirs.iter().position(|dir| path_exists(&dir.join(".git")));
    let matched = |dir: &Path, file: PathBuf| -> Option<Option<IgnoredBy>> {
        if !path_is_file(&file) {
            return None;
        }
        let mut builder = GitignoreBuilder::new(dir);
        if builder.add(&file).is_some() {
            return None;
        }
        let matcher = builder.build().ok()?;
        decision(matcher.matched(path, is_dir))
    };
    for dir in &dirs {
        if let Some(verdict) = matched(dir, dir.join(".ignore")) {
            return verdict;
        }
    }
    let repository = repository?;
    for dir in &dirs[..=repository] {
        if let Some(verdict) = matched(dir, dir.join(".gitignore")) {
            return verdict;
        }
    }
    let top = dirs[repository];
    if let Some(verdict) = matched(top, git_dir(top).join("info/exclude")) {
        return verdict;
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| top.to_path_buf());
    let (global, _) = GitignoreBuilder::new(cwd).build_global();
    decision(global.matched(path, is_dir)).flatten()
}

/// The repository's common git directory: `.git` itself, or for a worktree the
/// directory its `gitdir:` line and `commondir` name, where `info/exclude` lives.
fn git_dir(top: &Path) -> PathBuf {
    let dot_git = top.join(".git");
    let Ok(text) = read_git_metadata(&dot_git) else {
        return dot_git;
    };
    let Some(gitdir) = text
        .lines()
        .next()
        .and_then(|line| line.strip_prefix("gitdir: "))
    else {
        return dot_git;
    };
    let gitdir = top.join(gitdir);
    match read_git_metadata(&gitdir.join("commondir")) {
        Ok(common) => gitdir.join(common.trim()),
        Err(_) => gitdir,
    }
}
