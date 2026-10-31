//! What a build of the CLI reports as its version (§FS-distribution.4.1.1): the
//! Cargo version, followed for a development build by the commit it was built
//! from. One function derives it, and both `build.rs` and the derivation tests
//! include this file, so the label is worked out in one place.

use std::ffi::OsStr;
use std::fmt;
use std::path::{Path, PathBuf};

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
/// Scaffold for #1043: it returns the bare version and reads neither git nor
/// the variable, which is what every build reported before §FS-distribution.4.1.1.
pub fn build_identity(
    version: &str,
    crate_dir: &Path,
    commit: Option<&OsStr>,
    git: &OsStr,
) -> Result<BuildIdentity, MalformedCommit> {
    let _ = (crate_dir, commit, git);
    Ok(BuildIdentity {
        version: version.to_string(),
        watched: Vec::new(),
    })
}
