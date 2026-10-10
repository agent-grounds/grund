//! Snapshot-owned citation authoring semantics (§FS-lsp.1.6). No request scans
//! declarations or reloads configuration; the snapshot builder supplies both.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::editor_on_type::{docstring_content_at, line_is_rewritable};
use crate::config::{Config, fmt_excluded};
use crate::grammar::{in_escape_position, never_rewrite_context_in, parse_id_arg, render_id};
use crate::model::{canonical_snapshot_path, format_path, id_homes, relative_from_base};
use crate::resolver::WorkspaceContext;

#[path = "editor_completion_token.rs"]
mod token;

/// An ordinary single-line byte edit plus candidate metadata (§FS-lsp.1.6.2,
/// §FS-lsp.1.6.3). Protocol offsets and completion item objects belong to LSP.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CitationCompletion {
    pub id: String,
    pub title: String,
    pub source_path: String,
    pub sort_text: String,
    pub filter_text: String,
    pub start: usize,
    pub end: usize,
    pub text: String,
}

/// Opaque additive context; existing snapshot record constructors are unchanged
/// (§AR-lsp.2). Rebuilt together with notified overlays (§FS-lsp.1.6.4).
pub struct LspCompletionContext {
    projects: Vec<CompletionProject>,
    qualified: bool,
    complete: bool,
}

struct CompletionProject {
    alias: String,
    config: Config,
    files: BTreeSet<PathBuf>,
    editable: BTreeSet<PathBuf>,
    candidates: Vec<Candidate>,
}

struct Candidate {
    id: String,
    title: String,
    path: String,
    kind_order: usize,
}

impl LspCompletionContext {
    /// Cache uniquely resolving declared spellings and formatter file scopes in
    /// the snapshot's existing scan (§FS-lsp.1.6.2, §FS-lsp.1.6.4).
    pub(crate) fn from_workspace(context: &WorkspaceContext) -> Self {
        let projects = context
            .projects
            .iter()
            .map(|project| {
                let config = &project.config;
                let files: BTreeSet<_> = project
                    .findings
                    .scanned_files
                    .iter()
                    .map(|path| canonical_snapshot_path(path))
                    .collect();
                let excluded = fmt_excluded(config).ok();
                let editable = files
                    .iter()
                    .filter(|path| excluded.as_ref().is_some_and(|scope| !scope.contains(path)))
                    .cloned()
                    .collect();
                let mut candidates = Vec::new();
                for (id, decls) in &project.findings.declarations {
                    let Some(home) = id_homes(decls).sole() else {
                        continue;
                    };
                    let home = home.stand_in;
                    // §FS-lsp.1.6.2: a stub without its scanned inline home
                    // cannot supply a resolving authoring candidate.
                    if home.is_stub {
                        continue;
                    }
                    let spelling = render_id(&config.grammar, id);
                    // §FS-lsp.1.6.2: a displayed spelling must resolve through the
                    // existing parser, including its retained legacy grammar.
                    if id.legacy_spelling() != Some(spelling.as_str())
                        && !parse_id_arg(&spelling, &config.grammar)
                            .is_ok_and(|(parsed, section)| parsed == *id && section.is_none())
                    {
                        continue;
                    }
                    candidates.push(Candidate {
                        id: spelling,
                        title: home.title.clone().unwrap_or_default(),
                        path: format_path(&relative_from_base(&config.root, &home.file)),
                        kind_order: config
                            .kinds
                            .iter()
                            .position(|kind| kind.kind == id.kind)
                            .unwrap_or(config.kinds.len()),
                    });
                }
                CompletionProject {
                    alias: project.alias.clone(),
                    config: (**config).clone(),
                    files,
                    editable,
                    candidates,
                }
            })
            .collect();
        Self {
            projects,
            qualified: context.workspace_loaded,
            complete: context
                .projects
                .iter()
                .all(|project| project.scan_errors.is_empty()),
        }
    }

    /// Initial, deduplicated final introducer characters (§FS-lsp.1.6.1).
    pub fn trigger_characters(&self) -> Vec<String> {
        self.projects
            .iter()
            .flat_map(|project| {
                [&project.config.marker, &project.config.trigger]
                    .into_iter()
                    .filter_map(|prefix| prefix.chars().last().map(|ch| ch.to_string()))
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    /// Complete the owner-effective active token using this cached catalog.
    /// Invalid cursor boundaries and protected files produce no edits
    /// (§FS-lsp.1.6.1); right-hand suffixes are consumed (§FS-lsp.1.6.3).
    pub fn complete(
        &self,
        path: &Path,
        text: &str,
        line_index: usize,
        cursor: usize,
    ) -> Vec<CitationCompletion> {
        if !self.complete {
            return Vec::new();
        }
        let canonical = canonical_snapshot_path(path);
        let scanned: Vec<_> = self
            .projects
            .iter()
            .filter(|project| project.files.contains(&canonical))
            .collect();
        let owner = scanned
            .iter()
            .copied()
            .filter(|project| canonical.starts_with(&project.config.root))
            .max_by_key(|project| project.config.root.components().count())
            .or_else(|| match scanned.as_slice() {
                [owner] => Some(*owner),
                _ => None,
            });
        let Some(owner) = owner else {
            return Vec::new();
        };
        // §FS-lsp.1.6.1: a file symlink escaping its owner is a read target,
        // never an authoring edit target, even if the scanner followed it.
        if !owner.editable.contains(&canonical)
            || (path.starts_with(&owner.config.root) && !canonical.starts_with(&owner.config.root))
        {
            return Vec::new();
        }
        let Some(line) = text.lines().nth(line_index) else {
            return Vec::new();
        };
        if cursor > line.len() || !line.is_char_boundary(cursor) {
            return Vec::new();
        }
        let is_md = path.extension().and_then(|s| s.to_str()) == Some("md");
        let is_py = path.extension().and_then(|s| s.to_str()) == Some("py");
        if !line_is_rewritable(&owner.config, text, line_index, is_md, is_py) {
            return Vec::new();
        }
        let Some(token) = token::active_token(self, owner, line, cursor) else {
            return Vec::new();
        };
        let docstring = docstring_content_at(&owner.config, text, line_index, is_py);
        if in_escape_position(line, token.start, &owner.config.marker)
            || never_rewrite_context_in(docstring, line, is_md, token.start)
        {
            return Vec::new();
        }
        let mut candidates: Vec<_> = token
            .target
            .candidates
            .iter()
            .filter(|candidate| candidate.id.starts_with(token.prefix))
            .collect();
        candidates.sort_by(|a, b| {
            (a.id != token.prefix, a.kind_order, &a.id).cmp(&(
                b.id != token.prefix,
                b.kind_order,
                &b.id,
            ))
        });
        candidates
            .into_iter()
            .map(|candidate| {
                let id = format!("{}{}", token.qualification, candidate.id);
                CitationCompletion {
                    title: candidate.title.clone(),
                    source_path: candidate.path.clone(),
                    sort_text: format!(
                        "{}:{:08}:{}",
                        u8::from(candidate.id != token.prefix),
                        candidate.kind_order,
                        candidate.id
                    ),
                    filter_text: format!("{}{id}", token.introducer),
                    start: token.start,
                    end: token.end,
                    text: format!("{}{id}", owner.config.marker),
                    id,
                }
            })
            .collect()
    }
}
