//! The cautions a run earns for what it did *not* find (§AR-system.2.9): a walk
//! that read no files, one that read files and recognized nothing in them, and a
//! `--full` that had nothing left to cancel (§FS-check.2.2, §FS-check.4.5,
//! §FS-check.1.3.7).
//!
//! Every one is a `Diagnostic` and none of them prints, which is why they left
//! the deprecated path's `output` category with the run that folds them in: the
//! LSP snapshot builds the same two from the same function, so an editor and a
//! terminal over one tree say the same thing (§FS-lsp.4.1, §AR-system.4).

/// §FS-check.4.5.3: whether a walk that read files matched nothing in them. It asks
/// *recognized*, not *declared* — a project that only cites another project's
/// specs (§FS-workspace.1) declares nothing and is working as intended, so one
/// citation anywhere answers the question and the caution stays quiet.
use crate::model::remedies;
use crate::sources::{canonical_path, path_is_file};
use std::path::Path;

use super::unread_configs::UnreadConfigs;
use crate::checker::{ScanScope, scope_read_any_file};
use crate::config::{Config, Display, format_guidance, kind_prefixes, source_scope_name};
use crate::grammar::id_shape;
use crate::model::{Catalog, Code, Diagnostic, format_path, normalize_path_lexically};
use crate::scanner::{Unread, extension_refusal, name_refusal, pruning_pattern};
use crate::workspace::scope_is_config_root;

fn nothing_recognized(findings: &Catalog) -> bool {
    !findings.scanned_files.is_empty()
        && findings.declarations.is_empty()
        && findings.citations.is_empty()
}

/// The one caution a walk earns for what it did *not* find: §FS-check.2.2 when it
/// read no files, §FS-check.4.5 when it read files and matched nothing in them.
/// At most one — the empty scan is asked first, because a walk that read nothing
/// had nothing to recognize.
///
/// One decision for every surface that runs the engine: `grund check`, the
/// workspace loop beside it, and the LSP snapshot (§FS-lsp.4.1, `check_workspace_context`).
/// Spelled once per surface it was already wrong once — the LSP kept the empty
/// scan and never grew the second caution, so an editor and a terminal disagreed
/// about one tree.
///
/// `report_is_silent` is the caller's own answer to "did this project report
/// anything about its configured scope?" — findings and unreadable files both.
/// The out-of-scope tier (§FS-check.3.14) is deliberately not part of it: those
/// are findings about the tree *outside* the scope, and a run that finds the
/// citations out there is exactly the one where saying the configured scope is
/// empty helps most. The §FS-check.1.3.6.3 unread-source caution is out for the
/// same reason and is appended after this call (§FS-check.2.2.3.1): it names a
/// file the *resolution* scope could not read, so letting it answer the question
/// would delete the caution of the run that needs it most — a mistyped path in a
/// tree that happens to hold one unreadable file elsewhere.
///
/// §FS-check.1.3.6.1: `report_scope` is why the emptiness question is asked of it
/// rather than of the walk. A path-scoped run reads the whole project, so a path
/// holding no scannable file would otherwise stop earning its caution because the
/// wider walk read plenty — the caution is about what the caller asked about.
///
/// `display` is how the run's report spells a path, which in a workspace is the
/// root's and not this project's (§FS-workspace.8.1, §FS-workspace.5.3).
pub(super) fn scan_scope_caution(
    config: &Config,
    findings: &Catalog,
    path: &Path,
    path_provided: bool,
    report_is_silent: bool,
    report_scope: Option<&ScanScope>,
    display: Display<'_>,
) -> Option<Diagnostic> {
    if !report_is_silent {
        return None;
    }
    if !scope_read_any_file(findings, report_scope) {
        return Some(empty_scan_warning(config, path, path_provided, display));
    }
    // §FS-check.4.5.4: only a run over the whole project makes the claim. A narrowed
    // `grund check <dir>` is a slice the caller chose, and a slice with no
    // declarations and no citations is an answer, not a misconfiguration.
    (nothing_recognized(findings) && scope_is_config_root(&config.root, path, path_provided))
        .then(|| nothing_recognized_warning(config, findings.scanned_files.len()))
}

/// §FS-check.2.2, §FS-check.1.3.6.1: the scope caution of a workspace run narrowed to
/// a path below its root, asked once of the path rather than per project, since one
/// member reading nothing under it says nothing about another. `grund check`'s
/// workspace arm and the editor's (§AR-lsp.5.1.1) both ask it here, with `root` the
/// workspace root's config and `report_is_silent` the narrowed report's answer.
pub(super) fn workspace_path_caution<'a>(
    root: &Config,
    catalogs: impl IntoIterator<Item = &'a Catalog>,
    path: &Path,
    path_provided: bool,
    report_is_silent: bool,
    report_scope: Option<&ScanScope>,
) -> Option<Diagnostic> {
    let read_any = catalogs
        .into_iter()
        .any(|findings| scope_read_any_file(findings, report_scope));
    (report_scope.is_some() && report_is_silent && !read_any)
        .then(|| empty_scan_warning(root, path, path_provided, root.display()))
}

/// Whether the path `check` was handed is a **file** that the hidden-name rule
/// alone kept out of the walk (§FS-check.2.2.2): the caller really handed a path,
/// its own name begins with `.`, and its extension is one `[scan] extensions`
/// lists — so the extension list is the one rule that did *not* decide, and the
/// message must not send the reader there.
///
/// Every term is load-bearing. Without `path_provided` a library caller pairing a
/// hidden `path` with `path_provided: false` would be told a file the run never
/// looked at was skipped, since that run walks `[scan] include` and ignores `path`
/// entirely. A hidden file whose extension is *also* unlisted has two reasons and
/// keeps the extension message, because naming one of two causes is its own
/// misdirection. And the `is_file` test keeps this to files: a hidden **directory**
/// handed explicitly is walked (§FS-config.3.5.12), so an empty one really did match
/// no extensions — and a directory whose only content is hidden keeps the extension
/// message too, a boundary §FS-check.2.2.2 states rather than leaves to be found.
///
/// Both name rules are the read predicate's answers (§AR-scanner.1.13), so this
/// caution and the walk cannot disagree about which rule refused the file.
fn handed_a_hidden_file(config: &Config, path: &Path, path_provided: bool) -> bool {
    path_provided
        && path_is_file(&path)
        && name_refusal(path, config.schema()) == Some(Unread::HiddenName)
        && extension_refusal(path, config.schema()).is_none()
}

/// The CLI-level warning `check` reports when the tree walk matched no files
/// (§FS-check.2.2): a scan that read nothing is almost always a misconfigured
/// scope, so we say so instead of printing nothing and exiting `0`. This is a
/// warning — it never changes the exit code. Which of the three messages it
/// carries is a question about *why* nothing was read, so the hidden file is
/// asked first: it was skipped before either of the arms below could decide.
/// A root run no config applies to is asked next, before the root arms: when it
/// holds a project config below, the include list those arms name is the
/// defaults' and the scaffold they offer would compete with that project
/// (§FS-check.2.2.2.5).
///
/// Every arm names `path` as `display` spells it (§FS-workspace.5.3), while
/// which arm speaks is still asked of the real path.
pub(super) fn empty_scan_warning(
    config: &Config,
    path: &Path,
    path_provided: bool,
    display: Display<'_>,
) -> Diagnostic {
    let shown = display_scope(display, path);
    let version = config.frame().version;
    // `grund`, `grund check .`, and `grund check <repo-root>` all walk `[scan] include`
    // relative to the config root — so the "looked under include" message is the
    // accurate one whenever the requested path *is* that root, not just when the
    // path was omitted.
    let scoped_to_root = !path_provided
        || path == Path::new(".")
        || canonical_path(path)
            .map(|p| p == config.root)
            .unwrap_or(false);
    // §FS-check.2.2.2.5: asked of the root run alone, which a handed hidden file
    // never is; `below` walks nothing for a run that loaded a config.
    let unread = scoped_to_root
        .then(|| UnreadConfigs::below(config, display))
        .flatten();
    let message = match (&config.include, scoped_to_root, unread) {
        // §FS-check.2.2.2: name the hidden-name rule that actually skipped the file,
        // rather than the `include` list or the extensions that never got to answer.
        _ if handed_a_hidden_file(config, path, path_provided) => format!(
            "nothing to scan — `{}` is a hidden file. grund reads no file whose own name \
             begins with `.`, whatever `[scan] extensions` says. {}.",
            shown,
            remedies::empty_scan_hidden(version)
        ),
        // §FS-check.2.2.2.5, §FS-check.2.2.2.5.1: the defaults' include list and the
        // scaffold would both be false here, so name the configs the run did not read.
        (_, _, Some(unread)) => format!(
            "nothing to scan — no grund.toml applies here, so grund used its defaults and \
             found no files. {}: {}.",
            unread.names(),
            unread.check_routes()
        ),
        // §FS-check.2.2.2.1: a project with no kinds has no starter file to
        // scaffold, so the first remedy it is offered is its first kind.
        (Some(dirs), true, None) if kind_prefixes(&config.kinds).is_empty() => format!(
            "nothing to scan — grund looked under [scan] include = [{}] and found no files. This \
             project declares no kinds yet: {}.",
            quoted_list(dirs),
            remedies::empty_scan_no_kinds(version)
        ),
        (Some(dirs), true, None) => format!(
            "nothing to scan — grund looked under [scan] include = [{}] and found no files. {}.",
            quoted_list(dirs),
            remedies::empty_scan_root(version)
        ),
        // §FS-check.2.2.2.2: under v2 a pattern can prune what the extensions admit.
        _ => match pruning_pattern(config.schema(), config.frame(), path) {
            Some(pattern) => format!(
                "nothing to scan — every file under `{}` that grund would read is excluded by \
                 the [schema.sources] exclude pattern `{pattern}`.",
                shown
            ),
            None => format!(
                "nothing to scan — no files under `{}` matched grund's extensions ({}).",
                shown,
                config.extensions.join(", ")
            ),
        },
    };
    Diagnostic {
        code: Code::EMPTY_SCAN,
        path: None,
        line: None,
        column: None,
        message,
        sites: Vec::new(),
        authority: Vec::new(),
    }
}

/// `["a", "b"]`'s inside, as the empty-scan warning names `[scan] include` and
/// the `members` of §FS-check.2.2.2.5.1.
pub(super) fn quoted_list(dirs: &[String]) -> String {
    dirs.iter()
        .map(|dir| format!("\"{dir}\""))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The CLI-level warning `check` reports when the walk read files and recognized
/// nothing in them — no declaration and no citation (§FS-check.4.5). The empty
/// scan above says the scope found no files; this says the scope was right and
/// the grammar matched none of their content, which is what a docs tree written
/// for a different `[id] format` looks like from the inside. A warning, so the
/// exit code is untouched — what it takes away is the `success` marker
/// (§DF-nothing-recognized.2.2).
///
/// The shapes come from the configured template (`id_shape`) and the marker, and
/// the kinds are named in config order; nothing here is derived from the tree, so
/// two runs over one config print one string (§FS-errors.4).
///
/// The closing sentence offers both readings because the run cannot tell them
/// apart without judging a line, which is §FS-declarations.checks.declaration-near-miss's job: a tree
/// written to another format and a `grund init` scaffold nobody has declared in
/// yet produce the identical fact, and naming only the first would send a fresh
/// adopter to look for a bug in a config that is fine.
fn nothing_recognized_warning(config: &Config, scanned_files: usize) -> Diagnostic {
    // §FS-check.4.5.2: no selected kind, so describe the loaded global grammar.
    let guidance = format_guidance(config.schema(), config.frame(), None);
    let shape = id_shape(guidance.template);
    let files = if scanned_files == 1 { "file" } else { "files" };
    let kinds = kind_prefixes(&config.kinds);
    // §FS-check.4.5.2: with no kind, no token is a citation; say so, not `{}`.
    let message = if kinds.is_empty() {
        let target = if config.frame().version == 2 {
            "[schema.kinds.<NAME>]"
        } else {
            "[[kinds]]"
        };
        format!(
            "nothing recognized — grund read {scanned_files} {files}, and this project declares \
             no kinds, so no heading in them is a declaration and no token a citation. {}. See \
             `grund config explain '{target}'` for every key a kind takes.",
            remedies::nothing_recognized()
        )
    } else {
        format!(
            "nothing recognized — grund read {scanned_files} {files} and found no declaration \
             and no citation in them. A declaration heading reads `# {shape}: <title>` and a \
             citation `{marker}{shape}`, under {key} = \"{format}\" with <KIND> one of \
             {{{kinds}}}. Either nothing is declared yet, or the headings are written to a \
             different shape than that.",
            marker = config.marker,
            key = guidance.key,
            format = guidance.template,
            kinds = kinds.join(", "),
        )
    };
    Diagnostic {
        code: Code::NOTHING_RECOGNIZED,
        path: None,
        line: None,
        column: None,
        message,
        sites: Vec::new(),
        authority: Vec::new(),
    }
}

/// §FS-check.1.3.7: the caution a `--full` run earns when the caller also typed a
/// path that is not the config root. The flag has nothing left to cancel because
/// of the two-scope rule of §FS-check.1.3.6.1 — the path already resolves against
/// the configured scope, so widening it subtracts nothing from what the run
/// reads, and the report is the path with the flag or without it. A warning rather
/// than a rejection:
/// the invocation is valid, and a script that passes `--full` uniformly must not
/// fail on the one call where it is redundant. Like every warning it leaves the
/// exit code alone and, per §FS-check.2.1.3, stands in place of the `success`
/// marker on an otherwise clean run.
pub(super) fn full_scope_ignored_warning(
    config: &Config,
    path: &Path,
    path_provided: bool,
    full: bool,
) -> Option<Diagnostic> {
    if !full || scope_is_config_root(&config.root, path, path_provided) {
        return None;
    }
    // §FS-config.3.5.2 / §FS-config.3.6: the warning reports the explicit
    // spelling the scanner keeps, not the physical target an in-tree symlink
    // resolves to. Identity checks above may canonicalize; report text may not.
    let lexical_scope = if path.is_absolute() {
        normalize_path_lexically(path)
    } else {
        std::env::current_dir()
            .map(|cwd| normalize_path_lexically(&cwd.join(path)))
            .unwrap_or_else(|_| normalize_path_lexically(path))
    };
    Some(Diagnostic {
        code: Code::FULL_SCOPE_IGNORED,
        path: None,
        line: None,
        column: None,
        message: format!(
            "--full has no effect with an explicit PATH — {} already resolves against \
             {}, and the report is the path either way; {}",
            display_scope(config.display(), &lexical_scope),
            source_scope_name(config.frame().version),
            remedies::full_scope_ignored()
        ),
        sites: Vec::new(),
        authority: Vec::new(),
    })
}

/// §FS-workspace.5.3, §FS-config.3.6: a scope as a report names it, never absolute.
/// A path the caller typed relative is kept as typed. An absolute one — a typed
/// path, a workspace project's root, the LSP's anchor — is spelled from the
/// report base without resolving its final symlink: canonical ancestors only
/// identify the base when the OS respells it. The base itself reads `.`.
pub(super) fn display_scope(display: Display<'_>, path: &Path) -> String {
    if !path.is_absolute() {
        return format_path(path);
    }
    let base = display.base();
    let shown = path
        .ancestors()
        .find(|ancestor| canonical_path(ancestor).is_ok_and(|resolved| resolved == base))
        .map(|ancestor| format_path(path.strip_prefix(ancestor).unwrap_or(path)))
        .unwrap_or_else(|| display.path(path));
    if shown.is_empty() {
        ".".to_owned()
    } else {
        shown
    }
}
