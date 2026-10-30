//! Snapshot-owned citation authoring semantics (§FS-lsp.1.6). No request scans
//! declarations or reloads configuration; the snapshot builder supplies both.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::editor_on_type::{docstring_content_at, line_is_rewritable};
use crate::config::{Config, fmt_excluded};
use crate::grammar::{
    MarkdownLinks, in_escape_position, never_rewrite_context_in, parse_id_arg,
    render_declaration_id,
};
use crate::model::{format_path, paired_declarations, relative_from_base};
use crate::resolver::WorkspaceContext;
use crate::sources::canonical_snapshot_path;

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
    /// Each import's alias in the run's alias list, with its root as the run
    /// spells it (§FS-imports.editor).
    imports: Vec<(String, String)>,
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
    /// the snapshot's existing scan (§FS-lsp.1.6.2, §FS-lsp.1.6.4). An ID resolves
    /// uniquely where it has one declaration once its stubs are paired by their
    /// verdicts (§AR-scanner.4.6).
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
                    let Some(paired) = paired_declarations(decls).sole() else {
                        continue;
                    };
                    let home = paired.stand_in;
                    // §FS-lsp.1.6.2: a stub without its scanned inline home
                    // cannot supply a resolving authoring candidate.
                    if home.is_stub {
                        continue;
                    }
                    let spelling = render_declaration_id(&config.grammar, paired.record);
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
        let imports = context
            .projects
            .iter()
            .filter(|project| context.workspace_loaded && project.mount.is_some())
            .map(|project| {
                let root = relative_from_base(&context.render_root, &project.config.root);
                (project.alias.clone(), format_path(&root))
            })
            .collect();
        Self {
            projects,
            imports,
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
        // §FS-config-v2.schema.sources: the file is read as `grund fmt` reads it.
        let is_md = owner.config.reading(path).markdown;
        if !line_is_rewritable(&owner.config, path, text, line_index) {
            return Vec::new();
        }
        let Some(token) = token::active_token(self, owner, line, cursor) else {
            return Vec::new();
        };
        let docstring = docstring_content_at(&owner.config, path, text, line_index);
        let links = MarkdownLinks::new(text);
        if in_escape_position(line, token.start, &owner.config.marker)
            || never_rewrite_context_in(docstring, links.line(line_index), line, is_md, token.start)
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
        let completion =
            |id: String, title: &str, source_path: &str, sort_text: String| CitationCompletion {
                title: title.to_string(),
                source_path: source_path.to_string(),
                sort_text,
                filter_text: format!("{}{id}", token.introducer),
                start: token.start,
                end: token.end,
                text: format!("{}{id}", owner.config.marker),
                id,
            };
        let declared = candidates.into_iter().map(|candidate| {
            let sort_text = format!(
                "{}:{:08}:{}",
                u8::from(candidate.id != token.prefix),
                candidate.kind_order,
                candidate.id
            );
            let id = format!("{}{}", token.qualification, candidate.id);
            completion(id, &candidate.title, &candidate.path, sort_text)
        });
        // §FS-imports.editor: each import's alias, after the declared IDs, before a `/`.
        let imports = self
            .imports
            .iter()
            .filter(|(alias, _)| token.qualification.is_empty() && alias.starts_with(token.prefix))
            .map(|(alias, root)| completion(format!("{alias}/"), "", root, format!("2:{alias}/")));
        declared.chain(imports).collect()
    }
}
