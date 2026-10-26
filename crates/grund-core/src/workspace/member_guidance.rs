//! Config-only context for a member's unknown-alias finding (§FS-workspace.5.1.1).

use super::expand::enclosing_workspace_of;
use super::members::AncestorWorkspaces;
use super::scope::{RootMode, derive_alias};
use crate::config::Run;
use crate::model::{format_path, relative_from_base};

/// §FS-workspace.5.1.1: verified root-project identity and a location usable
/// from the invocation CWD, independent of the report's path-base setting.
pub(crate) struct MemberRootGuidance {
    alias: String,
    location: String,
}

impl MemberRootGuidance {
    /// §FS-workspace.5.1.1: explain only the actual root alias; the command
    /// validates the citation without promising that its target resolves.
    pub(crate) fn message(&self, namespace: &str) -> Option<String> {
        (namespace == self.alias).then(|| {
            format!(
                "unknown project alias {namespace}; {namespace} names the enclosing workspace's root project, and this project is a member of that workspace — {}",
                crate::model::remedies::unknown_project_root(&self.location)
            )
        })
    }
}

/// §FS-workspace.5.1.1: reuse the canonical membership verifier and alias
/// derivation without expanding or scanning the enclosing project's declarations.
/// An undecidable claim withholds the hint rather than changing the check's verdict:
/// one named and not answered, and one above the root whose `members` text cannot
/// be obtained (§FS-workspace.6.1.7.5), which may list that root under another.
pub(crate) fn member_root_guidance(run: &Run) -> Option<MemberRootGuidance> {
    let mut ancestors = AncestorWorkspaces::quiet_for_run_at(&run.root);
    let mut child = run.root.clone();
    let mut enclosing = None;
    while let Some(parent) = enclosing_workspace_of(&child, &run.cli_base, &mut ancestors).ok()? {
        child = parent.root.clone();
        enclosing = Some(parent);
    }
    // §FS-workspace.5.1.1: the top of the chain is the root only when nothing above
    // it could still claim it.
    if ancestors.has_undecidable_ancestor(&child) {
        return None;
    }
    let root = enclosing?;
    if !root.workspace_include_root {
        return None;
    }
    let alias = derive_alias(&root, None, RootMode::Root).ok()?;
    let cwd = crate::sources::canonical_path(&std::env::current_dir().ok()?).ok()?;
    let relative = relative_from_base(&cwd, &root.root);
    let location = if relative.as_os_str().is_empty() {
        ".".to_string()
    } else {
        format_path(&relative)
    };
    Some(MemberRootGuidance { alias, location })
}
