//! The three cautions a `[workspace]` block earns from its own member list
//! (§AR-system.2.4): an absorbed scan, an opted-out block nobody reads, and an
//! ancestor claim nothing could answer (§FS-check.4.7, §FS-check.4.10,
//! §FS-workspace.6.1).
//!
//! Split from `members.rs`, which expands a `members` list and enforces the
//! invariants that list has to satisfy (§AR-core-module-layout.1). These are
//! what a reader is told *about* an expansion that is itself legal — a different
//! question, with a different shape: every one is a sentence built apart from the
//! `Diagnostic` that carries it, so a test can read the sentence, and every one
//! is **returned** rather than printed, because rendering is a frontend's
//! (§DA-engine-renders-nothing, §FS-distribution.3.1).
//!
//! The anchor beside each sentence is the other half of that: the `grund.toml`
//! line the message already names, carried as the finding's own location so an
//! editor places it without parsing the text (§FS-lsp.1.1, §AR-bindings.2).

use std::path::{Path, PathBuf};

use super::members::{WorkspaceMember, canonical_workspace_path};
use super::scope::config_location_message;
use crate::config::{Config, ConfigLocation, config_file_in, root_scope_roots};
use crate::model::{Diagnostic, format_path, relative_from_base};

/// The release the finding this file builds the sentence for stops being a
/// warning and becomes an error in
/// (§FS-check.4.7, §RM-workspace-absorbed-scan-error). The deprecation path
/// §REQ-backwards-compatibility.2 requires puts it one minor past the release the
/// warning ships in; the message names it, because a warning that does not say
/// when it bites tells a maintainer they have a problem and not that they have a
/// deadline, and a unit test holds it ahead of the running version so the window
/// cannot expire unnoticed.
pub(crate) const ABSORBED_SCAN_ERROR_RELEASE: &str = "0.14.0";

/// §FS-workspace.2.1: each of the block's own walk roots that a member root
/// covers, rendered `` `<root>` in `<member>` `` — empty unless **every** root
/// that exists is covered, because a partly covered scope is §FS-workspace.6's
/// boundary working as designed and has to stay silent.
///
/// The roots are [`root_scope_roots`] at `full = false` rather than a second
/// reading of `[scan] include`: the question is about the set that boundary
/// prunes, and one definition of it is what keeps this true when a `[[kinds]]`
/// home or an unwalked home changes the set (§FS-config.3.5). `--full` is never
/// asked, because the flag adds the config root as a root while the boundary
/// still prunes — the finding is a property of the configuration, not of one
/// walk (§FS-check.1.3).
///
/// Why a root that is not on disk is skipped: the walk skips it too, before it
/// prunes, so it is read by nobody and rescues nothing. Without that filter a
/// default kind home nobody scaffolded would answer "not every root is covered"
/// for a project that plainly reads nothing.
///
/// Comparison is canonical and by prefix, exactly as the walk's own prune
/// compares, so a member reached through a symlink or a glob covers what it
/// actually lands on. Each root is named by its path under the block root, and
/// the member beside it is the entry as written, which is what it was resolved
/// from.
pub(crate) fn absorbed_scan_roots(config: &Config, members: &[WorkspaceMember]) -> Vec<String> {
    // §FS-workspace.2.1: a block with `include_root = false` is not a project, so
    // it has no scan of its own to lose — what its own files cost is the same
    // question asked from the other side, and §FS-check.4.10 is where it is asked.
    if !config.workspace_include_root || members.is_empty() {
        return Vec::new();
    }
    let mut covered = Vec::new();
    for (root, member) in block_scope_roots(config, members) {
        let Some(member) = member else {
            return Vec::new();
        };
        covered.push(format!(
            "`{}` in `{}`",
            format_path(block_relative_root(config, &root)),
            member.written
        ));
    }
    covered
}

/// Each root of the block's **own default scope** that is on disk, paired with
/// the member root that covers it — `None` where the members leave the root to
/// the block itself.
///
/// One list, read from opposite ends by the two findings that ask what a block's
/// own scan comes to: §FS-workspace.2.1 fires when every entry is covered, and
/// §FS-check.4.10 when an uncovered entry holds a file. Sharing it is what keeps
/// a `[[kinds]]` home, or a home the config lists without walking, moving both
/// rules together instead of one of them (§FS-config.3.5) — which is why this
/// and [`block_relative_root`] are `pub(crate)`: the second of those findings
/// has to run the walker to answer, so it is `resolver/unread_block.rs` and
/// reads the one list downward (§AR-resolver.placement).
///
/// The roots are [`root_scope_roots`] at `full = false` rather than a second
/// reading of `[scan] include`: the question is about the set §FS-workspace.6's
/// boundary prunes. `--full` is never asked, because the flag adds the config
/// root as a root while the boundary still prunes — both findings are properties
/// of the configuration, not of one walk (§FS-check.1.3).
///
/// Why a root that is not on disk is dropped: the walk skips it before it prunes,
/// so it is read by nobody and rescues nobody. Comparison is canonical and by
/// prefix, exactly as the walk's own prune compares.
pub(crate) fn block_scope_roots<'a>(
    config: &Config,
    members: &'a [WorkspaceMember],
) -> Vec<(PathBuf, Option<&'a WorkspaceMember>)> {
    root_scope_roots(config, false)
        .into_iter()
        .filter(|root| root.exists())
        .map(|root| {
            let canonical = canonical_workspace_path(&root);
            let member = members
                .iter()
                .find(|member| canonical.starts_with(&member.root));
            (root, member)
        })
        .collect()
}

/// A scope root named by its path **under the block root** — the config's
/// spelling normalized rather than the spelling itself, so `./docs/` is named
/// `docs` — because a canonical root renders as nothing when it equals the render
/// base and as an absolute path when it does not (§FS-errors.4).
pub(crate) fn block_relative_root<'a>(config: &Config, root: &'a Path) -> &'a Path {
    root.strip_prefix(&config.root).unwrap_or(root)
}

/// The engine's own handle on the three `[workspace]` cautions this rule set
/// produces. None of them ever renders as one of `check`'s JSON objects — each
/// keeps its text on every surface (§FS-check.4.7, §FS-check.4.10,
/// §FS-workspace.6.1) — so none enters the §FS-errors.5 selector vocabulary, and
/// these are what an editor tags a published diagnostic with and what a test
/// selects one by.
pub(crate) const ABSORBED_WORKSPACE_SCAN: &str = "absorbed-workspace-scan";
pub(crate) const UNREAD_WORKSPACE_BLOCK: &str = "unread-workspace-block";
pub(crate) const UNDECIDABLE_WORKSPACE_CLAIM: &str = "undecidable-workspace-claim";

/// The sentence §FS-check.4.7 carries, built apart from the diagnostic that
/// carries it so a test can read it: what was swallowed by what, what that costs
/// the project, the two ways out, and the release the finding stops being a
/// warning in.
pub(crate) fn absorbed_scan_warning(covered: &[String]) -> String {
    format!(
        "[workspace] members swallows this project's whole scan — every scan root \
         is inside a member: {} — so its declarations are unreachable and its \
         citations are never checked. Point [scan] include at a directory that is \
         not a member, or set include_root = false. This becomes an error in grund \
         {ABSORBED_SCAN_ERROR_RELEASE}.",
        covered.join(", ")
    )
}

/// §FS-check.4.7: the block's absorbed scan as one of the run's warnings, or
/// `None` where its members swallow nothing.
///
/// It **anchors at the block's `members` line** — the `grund.toml:<line>`
/// breadcrumb the message text already carries is the finding's own location
/// too, so a frontend places it without parsing the message (§FS-lsp.1.1). The
/// CLI-level shape §FS-check.2.1.1 fixes stays off the anchor: a fact about the
/// run's configuration is not a finding at a site in the citation graph, so no
/// frontend wears it as a `<path>:<line>:` prefix.
pub(crate) fn absorbed_scan_diagnostic(
    config: &Config,
    members: &[WorkspaceMember],
) -> Option<Diagnostic> {
    let covered = absorbed_scan_roots(config, members);
    if covered.is_empty() {
        return None;
    }
    Some(config_location_diagnostic(
        ABSORBED_WORKSPACE_SCAN,
        config.workspace_members_source.as_ref(),
        &config.root,
        absorbed_scan_warning(&covered),
    ))
}

/// §FS-check.4.10: the roots of a block's own scope that its members leave to the
/// block itself — [`block_scope_roots`] read from the uncovered end, and from the
/// boundary roots a populated block already carries rather than from an expanded
/// member list.
///
/// It is read from the config because the finding travels as the block it was
/// asked of: the walk that answers it sits above this component
/// (§AR-resolver.placement), so what the run carries between the question and the
/// answer is a `Config` with its boundary set and nothing else
/// (§DA-engine-renders-nothing).
pub(crate) fn uncovered_block_scope_roots(config: &Config) -> Vec<PathBuf> {
    root_scope_roots(config, false)
        .into_iter()
        .filter(|root| root.exists())
        .filter(|root| {
            let canonical = canonical_workspace_path(root);
            !config
                .workspace_boundary_roots
                .iter()
                .any(|member| canonical.starts_with(member))
        })
        .collect()
}

/// The sentence §FS-check.4.10 carries, built apart from the diagnostic that
/// carries it so a test can read it: the tree no scan reaches,
/// what that costs, and the two remedies the ticket itself named.
///
/// Shorter than [`absorbed_scan_warning`] above, and without its "declarations are
/// unreachable" half. That line is only ever printed on a green run, while this
/// one stands beside an error in three cases of the corpus, so it keeps to the
/// 180-byte cap a non-zero case's stderr is held to (§DF-unread-opted-out-block.2.4).
/// It names no release: there is none.
pub(crate) fn unread_block_warning(root: &str) -> String {
    format!(
        "no project scans `{root}`, so its citations are never checked. \
         Set include_root = true, or point another project's [scan] include at it."
    )
}

/// §FS-check.4.10: the unread block as one of the run's warnings, once a walker
/// has named the first root of its own scope that holds a file it would have
/// read.
///
/// It **anchors at the block's `include_root` line**, falling back to its
/// `[workspace]` line exactly as the breadcrumb does — the key that took the
/// block's files out of every scan is the line the reader should open
/// (§FS-lsp.1.1).
pub(crate) fn unread_block_diagnostic(config: &Config, root: &str) -> Diagnostic {
    config_location_diagnostic(
        UNREAD_WORKSPACE_BLOCK,
        config
            .workspace_include_root_source
            .as_ref()
            .or(config.workspace_section_source.as_ref()),
        &config.root,
        unread_block_warning(root),
    )
}

/// The sentence §FS-workspace.6.1 carries, built apart from the diagnostic that
/// carries it so a test can read it: what could not be answered, and what that
/// costs the reader — the alias paths below this directory, which is the
/// difference between a citation that passes here and one that passes at the root.
pub(crate) fn undecidable_ancestor_claim_warning(
    config_path: &Path,
    report_base: &Path,
    reason: &str,
) -> String {
    format!(
        "{}: cannot read [workspace] members ({reason}); \
         alias paths below it may be missing a segment",
        format_path(&relative_from_base(report_base, config_path))
    )
}

/// §FS-workspace.6.1: the undecidable claim as one of the run's warnings.
///
/// It **anchors at the config it could not read — that file and no line**,
/// because the `members` value whose line would be the anchor is exactly what
/// could not be obtained (§FS-lsp.1.1). The bytes the CLI prints are the
/// sentence above, unchanged.
pub(crate) fn undecidable_ancestor_claim_diagnostic(
    config_path: &Path,
    report_base: &Path,
    reason: &str,
) -> Diagnostic {
    Diagnostic {
        code: UNDECIDABLE_WORKSPACE_CLAIM,
        path: Some(config_path.to_path_buf()),
        line: None,
        column: None,
        message: undecidable_ancestor_claim_warning(config_path, report_base, reason),
        sites: Vec::new(),
    }
}

/// One `[workspace]` caution anchored at the config key its own breadcrumb names
/// (§FS-check.4.7, §FS-check.4.10).
///
/// The message keeps the `<config>:<line>:` breadcrumb §FS-config.4.3 gives a
/// diagnostic about a config key, byte for byte; the anchor beside it is the
/// same place spelled absolutely, so an editor opens the file this run read
/// rather than re-deriving one from the text (§AR-bindings.2).
fn config_location_diagnostic(
    code: &'static str,
    source: Option<&ConfigLocation>,
    block_root: &Path,
    message: String,
) -> Diagnostic {
    Diagnostic {
        code,
        path: config_file_in(block_root),
        line: source.map(|source| source.line),
        column: None,
        message: config_location_message(source, message),
        sites: Vec::new(),
    }
}
