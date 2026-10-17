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
//! `name_refusal` then `excluding_rule` for its verdict, retaining the first
//! refusing rule (§FS-declarations.stubs.target.3). And a link the walker could
//! not resolve asks `name_refusal` or
//! `directory_refusal`, with `ignored`, whether the walk would have read it
//! (§FS-config.3.5.6), since the walker filters no error entry itself.

use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use ignore::{Match, WalkBuilder, gitignore::Glob};

use super::exclude_globs::ExcludeGlobs;
use super::global_excludes::GlobalExcludes;
use super::walk::WalkDirFilter;
use crate::config::{Frame, Schema, canonical_config_root};
use crate::model::{ExcludingRule, format_path, is_hidden, scanned_path_key};
use crate::sources::{
    canonicalize_existing_prefix, is_directory, path_is_entry, path_is_symlink, read_git_metadata,
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
    /// A path a v2 `[schema.sources] exclude` glob matches (§AR-scanner.1.12).
    ExcludedPath,
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

/// The name and extension reasons, which hold wherever the file lies: the reasons
/// a stub's target asks first (§FS-declarations.stubs.target.3), and the test
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
/// stub's target, saved in its broken verdict (§FS-declarations.stubs.target.3).
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
    // §FS-declarations.stubs.target.3: filters and roots match the link spelling,
    // as in §FS-scan.scope.links.names, never the physical target's directories.
    let file = scanned_path_key(path);
    let config_root = scanned_path_key(frame.root());
    if !file.starts_with(&config_root) {
        return None;
    }
    let mut refusal = None;
    let globs = ExcludeGlobs::of(schema);
    // §FS-scan.scope.ignore-files.pathname: resolve once for the whole target probe.
    let global = schema
        .sources
        .respect_gitignore
        .then(|| GlobalExcludes::load().map(Arc::new))
        .transpose()
        .ok()?;
    let roots = roots.iter().map(|root| scanned_path_key(root));
    for root in std::iter::once(config_root.clone()).chain(roots) {
        let Ok(relative) = file.strip_prefix(&root) else {
            continue;
        };
        let rule = excluding_rule_under(
            schema,
            globs.as_ref(),
            &config_root,
            &root,
            relative,
            global.as_ref(),
        )?;
        refusal.get_or_insert(rule);
    }
    refusal
}

/// `excluding_rule` for one root `relative` lies below: the first rule met on the way
/// down, and at one directory the `exclude` entry or a v2 `exclude` pattern before
/// the ignore files, the walk's own order (§FS-declarations.stubs.target.3).
fn excluding_rule_under(
    schema: &Schema,
    globs: Option<&ExcludeGlobs>,
    config_root: &Path,
    root: &Path,
    relative: &Path,
    global: Option<&Arc<GlobalExcludes>>,
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
        // §FS-config-v2.schema.sources.exclude.stubs: a pattern, at a directory or the target.
        if let Some(pattern) = globs.and_then(|globs| globs.excluding(config_root, &path, is_dir)) {
            return Some(ExcludingRule::Pattern(pattern.to_string()));
        }
        // §FS-declarations.stubs.target.3: a hidden directory alone is no refusal.
        if is_dir
            && !is_hidden(&path)
            && let Some(entry) = excluded_entry(&path, &schema.sources.exclude)
        {
            return Some(ExcludingRule::Exclude(entry.to_string()));
        }
        if let Some(global) = global
            && let Some(ignored) = ignoring_rule(&path, is_dir, global)
        {
            let file = ignored
                .file
                .strip_prefix(config_root)
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
    // §FS-scan.scope.ignore-files.pathname: every component asks the same resolved file.
    let global = schema
        .sources
        .respect_gitignore
        .then(|| GlobalExcludes::load().map(Arc::new))
        .transpose()
        .ok()?;
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
        if global
            .as_ref()
            .is_some_and(|global| ignored_with(&path, is_dir, global))
        {
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
/// Both use one resolved global file (§FS-scan.scope.ignore-files.pathname).
pub(super) fn ignored(path: &Path, is_dir: bool) -> bool {
    GlobalExcludes::load()
        .map(Arc::new)
        .is_ok_and(|global| ignored_with(path, is_dir, &global))
}

/// The on-disk probe and rule fallback share one resolved global file
/// (§FS-scan.scope.ignore-files.pathname). Resolution errors stop the owning
/// scan through `scannable_walker`; these optional rule queries add no verdict.
fn ignored_with(path: &Path, is_dir: bool, global: &Arc<GlobalExcludes>) -> bool {
    walker_ignores(path, false, global)
        .unwrap_or_else(|| ignored_by_rules(path, is_dir, global).is_some())
}

/// The ignore file and the line of it that drop `path`, for a reader that names the
/// rule (§FS-declarations.stubs.target.3): the rule `ignored` matches,
/// kept only where the walker, asked of an entry on disk, drops it too.
/// Both read the supplied global matcher (§FS-scan.scope.ignore-files.pathname).
fn ignoring_rule(path: &Path, is_dir: bool, global: &Arc<GlobalExcludes>) -> Option<IgnoredBy> {
    let rule = ignored_by_rules(path, is_dir, global)?;
    // §FS-scan.scope.links.names: a directory link faces directory ignore patterns
    // when followed, just as it does in scannable_walker.
    (walker_ignores(path, true, global) != Some(false)).then_some(rule)
}

/// The walker's own answer for an entry on disk, `None` where nothing is there.
/// Stub eligibility follows links like the scan; the unresolved-link error probe
/// instead asks about the link entry without trying to resolve it again.
fn walker_ignores(path: &Path, follow_links: bool, global: &Arc<GlobalExcludes>) -> Option<bool> {
    let Some(parent) = path.parent() else {
        return Some(false);
    };
    if !path_is_entry(path) {
        return None;
    }
    let mut builder = WalkBuilder::new(parent);
    builder
        .hidden(false)
        .max_depth(Some(1))
        .follow_links(follow_links)
        .git_global(false);
    // §FS-scan.scope.ignore-files.pathname: no dependency fallback, even for an empty file.
    let global = Arc::clone(global);
    builder.filter_entry(move |entry| !globally_ignored(&global, entry));
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

/// Whether a walker that leaves the global file to its filter drops `entry` for it
/// (§FS-scan.scope.ignore-files.global): a global pattern anchored at the entry's
/// repository top ignores it, and no file before the global one in git's precedence
/// re-includes it. The walker has applied those files already, but an entry one of
/// them re-includes reaches the filter looking like one nothing matched, so they are
/// asked again here, and only for an entry the global file ignores
/// (§GOAL-fast-feedback). The walk root is never dropped, as the walker never
/// ignores it.
pub(super) fn globally_ignored(global: &GlobalExcludes, entry: &ignore::DirEntry) -> bool {
    let is_dir = entry.file_type().is_some_and(is_directory);
    entry.depth() > 0
        && matches!(global.matched(entry.path(), is_dir), Match::Ignore(_))
        && ignored_by_rules(entry.path(), is_dir, global).is_some()
}

/// The walker's precedence for a name, and the rule that ignores it: `.ignore`
/// files, deepest first; then, inside a repository, its `.gitignore` files up to
/// the repository top, its `info/exclude`, and the global excludes anchored at that
/// same top (§FS-scan.scope.ignore-files.global). A top that holds only `.jj` has
/// no `info/exclude`, so none is read there and none from a repository around it.
fn ignored_by_rules(path: &Path, is_dir: bool, global: &GlobalExcludes) -> Option<IgnoredBy> {
    let dirs: Vec<&Path> = path
        .ancestors()
        .skip(1)
        .take_while(|dir| !dir.as_os_str().is_empty())
        .collect();
    let matched = |dir: &Path, file: PathBuf| -> Option<Option<IgnoredBy>> {
        decision(global.ignore_file(dir, file)?.matched(path, is_dir))
    };
    for dir in &dirs {
        if let Some(verdict) = matched(dir, dir.join(".ignore")) {
            return verdict;
        }
    }
    let top = global.top(path)?;
    for dir in dirs.iter().take_while(|dir| dir.starts_with(&top)) {
        if let Some(verdict) = matched(dir, dir.join(".gitignore")) {
            return verdict;
        }
    }
    if let Some(verdict) = matched(&top, git_dir(&top).join("info/exclude")) {
        return verdict;
    }
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
