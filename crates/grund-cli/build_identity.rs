//! What a build of the CLI reports as its version (§FS-distribution.4.1.1): the
//! Cargo version, followed for a development build by the commit it was built
//! from. One function derives it, and both `build.rs` and the derivation tests
//! include this file, so the label is worked out in one place.

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

/// What `git rev-parse --show-prefix` prints from the crate directory in qualifying
/// git: the crate is the tracked one, not a copy nested elsewhere in a checkout.
const CRATE_PREFIX: &[u8] = b"crates/grund-cli/";

/// The CLI's manifest, which qualifying git tracks.
const MANIFEST: &str = "crates/grund-cli/Cargo.toml";

/// The one path watched where no qualifying git exists. It never exists, so Cargo
/// reruns the script on every build and a tree that gains git is noticed.
const NO_GIT: &str = "no-qualifying-git-rerun-every-build";

/// Git's local-environment variables (`git rev-parse --local-env-vars`). A hook
/// sets them, and they would point git at another repository or index.
const GIT_LOCAL_ENV: &[&str] = &[
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_CONFIG",
    "GIT_CONFIG_PARAMETERS",
    "GIT_CONFIG_COUNT",
    "GIT_OBJECT_DIRECTORY",
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_IMPLICIT_WORK_TREE",
    "GIT_GRAFT_FILE",
    "GIT_INDEX_FILE",
    "GIT_NO_REPLACE_OBJECTS",
    "GIT_REPLACE_REF_BASE",
    "GIT_PREFIX",
    "GIT_SHALLOW_FILE",
    "GIT_COMMON_DIR",
];

/// What a build records about itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildIdentity {
    /// `GRUND_BUILD_VERSION`, the `<semver>` of `grund --version`: the Cargo
    /// version, then `+<commit>` or `+<commit>.dirty` when a development build
    /// has a commit (§FS-distribution.4.1.1).
    pub version: String,
    /// The paths whose change can change `version`, each one a
    /// `cargo:rerun-if-changed` line. Every path exists, except where no
    /// qualifying git exists and the next build has to look for it again.
    pub watched: Vec<PathBuf>,
}

/// A `GRUND_BUILD_COMMIT` a development build refuses, so the build fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MalformedCommit {
    /// The variable's value as given, decoded lossily.
    pub value: String,
}

impl fmt::Display for MalformedCommit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "GRUND_BUILD_COMMIT={:?} is not ten lowercase hexadecimal digits, \
             optionally followed by `.dirty`",
            self.value
        )
    }
}

/// Derive the identity of the build of the crate at `crate_dir`, whose Cargo
/// version is `version`. `commit` is `GRUND_BUILD_COMMIT` as the build's
/// environment gives it, and `git` the program run to read the tree.
///
/// A release version is the label, and nothing is read for it. A development
/// version takes its commit from qualifying git, else from a well-formed
/// variable, else stays bare; a set variable is held to its spelling either way,
/// so no value suppresses the commit (§FS-distribution.4.1.1).
pub fn build_identity(
    version: &str,
    crate_dir: &Path,
    commit: Option<&OsStr>,
    git: &OsStr,
) -> Result<BuildIdentity, MalformedCommit> {
    if !version.ends_with("-dev") {
        return Ok(BuildIdentity {
            version: version.to_string(),
            watched: Vec::new(),
        });
    }
    let given = commit.map(well_formed).transpose()?;
    if let Some(tree) = read_git(crate_dir, git) {
        let dirty = if tree.dirty { ".dirty" } else { "" };
        return Ok(BuildIdentity {
            version: format!("{version}+{}{dirty}", tree.commit),
            watched: tree.watched,
        });
    }
    let version = match given {
        Some(given) => format!("{version}+{given}"),
        None => version.to_string(),
    };
    Ok(BuildIdentity {
        version,
        watched: vec![crate_dir.join(NO_GIT)],
    })
}

/// `GRUND_BUILD_COMMIT` as given, when it is ten lowercase hexadecimal digits
/// optionally followed by `.dirty` (§FS-distribution.4.1.1).
fn well_formed(value: &OsStr) -> Result<&str, MalformedCommit> {
    let refused = || MalformedCommit {
        value: value.to_string_lossy().into_owned(),
    };
    let text = value.to_str().ok_or_else(refused)?;
    let commit = text.strip_suffix(".dirty").unwrap_or(text);
    if is_commit(commit) {
        Ok(text)
    } else {
        Err(refused())
    }
}

fn is_commit(text: &str) -> bool {
    text.len() == 10 && text.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// What qualifying git says about the tree.
struct Tree {
    /// The first ten digits of HEAD's full SHA, never git's own abbreviation.
    commit: String,
    /// Whether a tracked file differs from HEAD in the index or the working tree.
    dirty: bool,
    watched: Vec<PathBuf>,
}

/// Read qualifying git: the crate directory is `crates/grund-cli/` of a repository
/// that tracks the manifest and has a commit. Git missing, refusing the checkout
/// or failing any read counts as none, and `safe.directory` is never set here
/// (§FS-distribution.4.1.1).
fn read_git(crate_dir: &Path, git: &OsStr) -> Option<Tree> {
    // An exact prefix rather than a comparison of paths, which differ in spelling
    // on Windows; it also puts the top level two directories up.
    let prefix = run(git, crate_dir, &["rev-parse", "--show-prefix"])?;
    if line(&prefix) != CRATE_PREFIX {
        return None;
    }
    let root = crate_dir.parent()?.parent()?;
    run(git, root, &["ls-files", "--error-unmatch", "--", MANIFEST])?;
    let head = run(
        git,
        root,
        &["rev-parse", "--verify", "--quiet", "HEAD^{commit}"],
    )?;
    let commit = std::str::from_utf8(head.get(..10)?).ok()?;
    if !is_commit(commit) {
        return None;
    }
    let status = run(
        git,
        root,
        &[
            "status",
            "--porcelain",
            "-z",
            "--no-renames",
            "--untracked-files=no",
        ],
    )?;
    // Each record is `XY <path>`; a deleted file is listed, and with renames
    // off a rename is its two sides.
    let changed: Vec<PathBuf> = status
        .split(|&b| b == 0)
        .filter(|record| record.len() > 3)
        .map(|record| root.join(path_from(&record[3..])))
        .collect();
    let dirty = !changed.is_empty();
    // Watch tracked paths while clean, changed paths while dirty. Cargo observes
    // their mtimes; unchanged-mtime edits can need a forced rebuild (§FS-distribution.4.1.1).
    let tree = if dirty { changed } else { tracked(git, root)? };
    let mut watched = head_paths(git, root)?;
    watched.extend(tree.iter().map(|path| nearest_existing(path)));
    Some(Tree {
        commit: commit.to_string(),
        dirty,
        watched: watched.into_iter().collect(),
    })
}

/// Every tracked file, as a path under `root`.
fn tracked(git: &OsStr, root: &Path) -> Option<Vec<PathBuf>> {
    let listed = run(git, root, &["ls-files", "-z"])?;
    let paths = listed.split(|&b| b == 0).filter(|path| !path.is_empty());
    Some(paths.map(|path| root.join(path_from(path))).collect())
}

/// What HEAD resolves through, the index and a linked worktree's `.git` file, each
/// one present. In the files backend a branch is its loose ref while one exists:
/// that file wins over `packed-refs`, so a packed-refs change cannot move HEAD
/// until the loose ref is written or removed. A packed branch watches
/// `packed-refs` and the directory its loose ref would be written into. In a
/// reftable repository the tables hold every ref (§FS-distribution.4.1.1).
fn head_paths(git: &OsStr, root: &Path) -> Option<BTreeSet<PathBuf>> {
    // A git too old to answer echoes the option back: the files backend, because
    // such a git cannot read a reftable repository at all.
    let format = run(git, root, &["rev-parse", "--show-ref-format"]);
    let reftable = format.is_some_and(|format| line(&format) == b"reftable");
    let mut args: Vec<&OsStr> = [
        "rev-parse",
        "--git-dir",
        "--git-common-dir",
        "--git-path",
        "HEAD",
        "--git-path",
        "index",
        "--git-path",
        "packed-refs",
    ]
    .into_iter()
    .map(OsStr::new)
    .collect();
    let branches = if reftable {
        Vec::new()
    } else {
        symbolic_refs(git, root)?
    };
    for branch in &branches {
        args.extend([OsStr::new("--git-path"), branch.as_os_str()]);
    }
    let resolved = run(git, root, &args)?;
    let paths: Vec<PathBuf> = resolved
        .split(|&b| b == b'\n')
        .filter(|path| !path.is_empty())
        .map(|path| root.join(path_from(line(path))))
        .collect();
    let [git_dir, common_dir, head, index, packed_refs, loose @ ..] = paths.as_slice() else {
        return None;
    };
    let mut watched = BTreeSet::new();
    let dot_git = root.join(".git");
    if dot_git.is_file() {
        watched.insert(dot_git);
    }
    watched.insert(nearest_existing(head));
    watched.insert(nearest_existing(index));
    if reftable {
        watched.insert(nearest_existing(&git_dir.join("reftable")));
        watched.insert(nearest_existing(&common_dir.join("reftable")));
    } else {
        for loose in loose {
            if !loose.exists() {
                watched.insert(nearest_existing(packed_refs));
            }
            watched.insert(ref_watch_path(loose));
        }
    }
    Some(watched)
}

/// Every ref in HEAD's symbolic chain, including the final direct ref. Watching
/// only the terminal ref misses an intermediate ref's retargeting
/// (§FS-distribution.4.1.1). A detached HEAD returns an empty list. A cycle,
/// excessive depth or failed read counts as no qualifying git; the bound also
/// terminates a chain that another process keeps extending while it is read.
fn symbolic_refs(git: &OsStr, root: &Path) -> Option<Vec<PathBuf>> {
    let mut refs = Vec::new();
    let mut seen = BTreeSet::new();
    let mut name = PathBuf::from("HEAD");
    for _ in 0..64 {
        if !seen.insert(name.clone()) {
            return None;
        }
        let output = git_output(
            git,
            root,
            &[
                OsStr::new("symbolic-ref"),
                OsStr::new("--no-recurse"),
                OsStr::new("-q"),
                name.as_os_str(),
            ],
        )?;
        match output.status.code() {
            Some(0) => {
                name = path_from(line(&output.stdout));
                if name.as_os_str().is_empty() {
                    return None;
                }
                refs.push(name.clone());
            }
            Some(1) => return Some(refs),
            _ => return None,
        }
    }
    None
}

/// Git ref paths retain their native bytes (§FS-distribution.4.1.1). Cargo's
/// build-script instructions require UTF-8, so a non-UTF-8 ref watches its nearest
/// existing UTF-8 ancestor recursively rather than emitting a lossy, missing path.
fn ref_watch_path(path: &Path) -> PathBuf {
    path.ancestors()
        .find(|ancestor| ancestor.to_str().is_some() && ancestor.exists())
        .unwrap_or(path)
        .to_path_buf()
}

/// `path`, or its nearest ancestor that exists: Cargo reads a missing watched path
/// as changed and would rerun the script on every build.
fn nearest_existing(path: &Path) -> PathBuf {
    path.ancestors()
        .find(|ancestor| ancestor.exists())
        .unwrap_or(path)
        .to_path_buf()
}

/// Run git in `dir` and return its stdout when it exits 0.
fn run(git: &OsStr, dir: &Path, args: &[impl AsRef<OsStr>]) -> Option<Vec<u8>> {
    let output = git_output(git, dir, args)?;
    output.status.success().then_some(output.stdout)
}

/// Run git in `dir` with its local environment cleared and without the optional
/// locks, so reading the index never rewrites the file the script watches.
fn git_output(git: &OsStr, dir: &Path, args: &[impl AsRef<OsStr>]) -> Option<Output> {
    let mut command = Command::new(git);
    for name in GIT_LOCAL_ENV {
        command.env_remove(name);
    }
    command
        .arg("--no-optional-locks")
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()
}

/// One line of git's output without its line ending.
fn line(bytes: &[u8]) -> &[u8] {
    let bytes = bytes.strip_suffix(b"\n").unwrap_or(bytes);
    bytes.strip_suffix(b"\r").unwrap_or(bytes)
}

#[cfg(unix)]
fn path_from(bytes: &[u8]) -> PathBuf {
    use std::os::unix::ffi::OsStrExt;
    PathBuf::from(OsStr::from_bytes(bytes))
}

#[cfg(not(unix))]
fn path_from(bytes: &[u8]) -> PathBuf {
    PathBuf::from(String::from_utf8_lossy(bytes).into_owned())
}
