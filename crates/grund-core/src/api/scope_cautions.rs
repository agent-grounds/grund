//! The cautions a run earns for what it did *not* find (§AR-system.2.9): a walk
//! that read no files, one that read files and recognized nothing in them, and a
//! `--full` that had nothing left to cancel (§FS-check.2.2, §FS-check.4.5,
//! §FS-check.1.3).
//!
//! Every one is a `Diagnostic` and none of them prints, which is why they left
//! the deprecated path's `output` category with the run that folds them in: the
//! LSP snapshot builds the same two from the same function, so an editor and a
//! terminal over one tree say the same thing (§FS-lsp.4, §AR-system.4).

/// §FS-check.4.5: whether a walk that read files matched nothing in them. It asks
/// *recognized*, not *declared* — a project that only cites another project's
/// specs (§FS-workspace.1) declares nothing and is working as intended, so one
/// citation anywhere answers the question and the caution stays quiet.
use std::fs;
use std::path::Path;

use crate::config::{Config, display_path, kind_prefixes};
use crate::grammar::id_shape;
use crate::model::{Diagnostic, Findings, format_path, normalize_path_lexically};
use crate::workspace::scope_is_config_root;

fn nothing_recognized(findings: &Findings) -> bool {
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
/// workspace loop beside it, and the LSP snapshot (§FS-lsp.4, `check_workspace_context`).
/// Spelled once per surface it was already wrong once — the LSP kept the empty
/// scan and never grew the second caution, so an editor and a terminal disagreed
/// about one tree.
///
/// `report_is_silent` is the caller's own answer to "did this project report
/// anything about its configured scope?" — findings and unreadable files both.
/// The out-of-scope tier (§FS-check.3.14) is deliberately not part of it: those
/// are findings about the tree *outside* the scope, and a run that finds the
/// citations out there is exactly the one where saying the configured scope is
/// empty helps most.
pub(super) fn scan_scope_caution(
    config: &Config,
    findings: &Findings,
    path: &Path,
    path_provided: bool,
    report_is_silent: bool,
) -> Option<Diagnostic> {
    if !report_is_silent {
        return None;
    }
    if findings.scanned_files.is_empty() {
        return Some(empty_scan_warning(config, path, path_provided));
    }
    // §FS-check.4.5: only a run over the whole project makes the claim. A narrowed
    // `grund check <dir>` is a slice the caller chose, and a slice with no
    // declarations and no citations is an answer, not a misconfiguration.
    (nothing_recognized(findings) && scope_is_config_root(config, path, path_provided))
        .then(|| nothing_recognized_warning(config, findings.scanned_files.len()))
}

/// Whether the path `check` was handed is a **file** that the hidden-name rule
/// alone kept out of the walk (§FS-check.2.2): the caller really handed a path,
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
/// handed explicitly is walked (§FS-config.3.5), so an empty one really did match
/// no extensions — and a directory whose only content is hidden keeps the extension
/// message too, a boundary §FS-check.2.2 states rather than leaves to be found.
fn handed_a_hidden_file(config: &Config, path: &Path, path_provided: bool) -> bool {
    path_provided
        && path.is_file()
        && path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with('.'))
        && path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| config.extensions.iter().any(|allowed| allowed == ext))
}

/// The CLI-level warning `check` reports when the tree walk matched no files
/// (§FS-check.2.2): a scan that read nothing is almost always a misconfigured
/// scope, so we say so instead of printing nothing and exiting `0`. This is a
/// warning — it never changes the exit code. Which of the three messages it
/// carries is a question about *why* nothing was read, so the hidden file is
/// asked first: it was skipped before either of the arms below could decide.
fn empty_scan_warning(config: &Config, path: &Path, path_provided: bool) -> Diagnostic {
    // `grund`, `grund check .`, and `grund check <repo-root>` all walk `[scan] include`
    // relative to the config root — so the "looked under include" message is the
    // accurate one whenever the requested path *is* that root, not just when the
    // path was omitted.
    let scoped_to_root = !path_provided
        || path == Path::new(".")
        || fs::canonicalize(path)
            .map(|p| p == config.root)
            .unwrap_or(false);
    let message = match (&config.include, scoped_to_root) {
        // §FS-check.2.2: name the hidden-name rule that actually skipped the file,
        // rather than the `include` list or the extensions that never got to answer.
        _ if handed_a_hidden_file(config, path, path_provided) => format!(
            "nothing to scan — `{}` is a hidden file. grund reads no file whose own name \
             begins with `.`, whatever `[scan] extensions` says. Rename it, or move what \
             needs checking into a file that is not hidden.",
            format_path(path)
        ),
        (Some(dirs), true) => format!(
            "nothing to scan — grund looked under [scan] include = [{}] and found no files. Run \
             `grund init --docs` to scaffold the canonical requirements.md, docs/, and e2e/ \
             trees, point `[scan] include` in `grund.toml` at your sources, or pass a \
             path explicitly (`grund check <dir>`).",
            dirs.iter()
                .map(|dir| format!("\"{dir}\""))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        _ => format!(
            "nothing to scan — no files under `{}` matched grund's extensions ({}).",
            format_path(path),
            config.extensions.join(", ")
        ),
    };
    Diagnostic {
        code: "empty-scan",
        path: None,
        line: None,
        column: None,
        message,
        sites: Vec::new(),
    }
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
/// apart without judging a line, which is §FS-check.4.6's job: a tree
/// written to another format and a `grund init` scaffold nobody has declared in
/// yet produce the identical fact, and naming only the first would send a fresh
/// adopter to look for a bug in a config that is fine.
fn nothing_recognized_warning(config: &Config, scanned_files: usize) -> Diagnostic {
    let shape = id_shape(&config.id_format);
    let files = if scanned_files == 1 { "file" } else { "files" };
    Diagnostic {
        code: "nothing-recognized",
        path: None,
        line: None,
        column: None,
        message: format!(
            "nothing recognized — grund read {scanned_files} {files} and found no declaration \
             and no citation in them. A declaration heading reads `# {shape}: <title>` and a \
             citation `{marker}{shape}`, under [id] format = \"{format}\" with <KIND> one of \
             {{{kinds}}}. Either nothing is declared yet, or the headings are written to a \
             different shape than that.",
            marker = config.marker,
            format = config.id_format,
            kinds = kind_prefixes(&config.kinds).join(", "),
        ),
        sites: Vec::new(),
    }
}

/// §FS-check.1.3: the caution a `--full` run earns when the caller also typed a
/// path that is not the config root. `--full` cancels `[scan] include`, and an
/// explicit path already bypasses that key, so the flag has nothing left to
/// cancel and the run is the ordinary one. A warning rather than a rejection:
/// the invocation is valid, and a script that passes `--full` uniformly must not
/// fail on the one call where it is redundant. Like every warning it leaves the
/// exit code alone and, per §FS-check.2.1, stands in place of the `success`
/// marker on an otherwise clean run.
pub(super) fn full_scope_ignored_warning(
    config: &Config,
    path: &Path,
    path_provided: bool,
    full: bool,
) -> Option<Diagnostic> {
    if !full || scope_is_config_root(config, path, path_provided) {
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
        code: "full-scope-ignored",
        path: None,
        line: None,
        column: None,
        message: format!(
            "--full has no effect with an explicit PATH — it cancels [scan] include, and {} \
             already bypasses it",
            display_lexical_scope(config, &lexical_scope)
        ),
        sites: Vec::new(),
    })
}

/// Render an explicit lexical scope without resolving its final symlink.
/// Canonical ancestors only identify the report base when the OS respells it.
fn display_lexical_scope(config: &Config, path: &Path) -> String {
    let base = if config.relative_paths {
        &config.root
    } else {
        &config.cli_base
    };
    for ancestor in path.ancestors() {
        if fs::canonicalize(ancestor)
            .map(|resolved| resolved == *base)
            .unwrap_or(false)
        {
            return format_path(path.strip_prefix(ancestor).unwrap_or(path));
        }
    }
    display_path(config, path)
}
