//! Test module: the one-file judgement a keystroke's completion is answered by is never wrong
//! (§FS-lsp.responsiveness.4, §AR-lsp.3.2). Over every end-to-end case's tree whose config loads,
//! each scanned file takes each of eight edits as an open buffer; wherever the judgement says the
//! declarations stand, a full scan with that buffer gives the catalog it was judged against. It
//! may say "changed" where nothing did, but not for a line of prose or a citation in a plain
//! Markdown file, so it cannot pass by never saying "unchanged". Reads the cases in place and
//! writes nothing.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use rayon::prelude::*;

use crate::api::lsp_snapshot_with_completion;
use crate::queries::{LspDeclaration, LspSnapshotOpts, LspSnapshotWithCompletion};
use crate::sources::{canonical_snapshot_path, stub_target_candidates};

/// The eight edits, each made to one file's text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Edit {
    /// A citation of a declared ID appended to a body line, or to a comment in a source.
    Citation,
    /// The editor's `$$` shorthand before an ID, where the citation goes.
    Shorthand,
    /// A blank line above the first declaration heading.
    BlankAbove,
    /// A line of body prose, or of comment in a source, at the end.
    Prose,
    /// The title after `: ` on the first declaration heading changed.
    Retitle,
    /// A new declaration heading at the end.
    NewHeading,
    /// The first declaration heading deleted.
    DeleteHeading,
    /// A stub line at the end.
    Stub,
}

const EDITS: [Edit; 8] = [
    Edit::Citation,
    Edit::Shorthand,
    Edit::BlankAbove,
    Edit::Prose,
    Edit::Retitle,
    Edit::NewHeading,
    Edit::DeleteHeading,
    Edit::Stub,
];

/// The names a one-file reading never judges, which no "must stand" holds to.
const CONFIG_NAMES: [&str; 3] = ["grund.toml", ".gitignore", ".ignore"];

/// The fewest plain Markdown files the sweep must hold to "unchanged", so that it cannot pass on
/// a tree list that found none.
const PLAIN_MARKDOWN_FLOOR: usize = 1000;

/// One scanned file as the edits see it.
struct Subject<'a> {
    text: &'a str,
    markdown: bool,
    /// A source's line comment; unused in Markdown.
    comment: &'static str,
    /// A source's declaring comment; unused in Markdown.
    doc: &'static str,
    marker: &'a str,
    /// An ID the file's project declares, as it is spelled.
    id: &'a str,
    /// The first declaration heading's line, from zero.
    heading: Option<usize>,
}

impl Subject<'_> {
    fn edited(&self, edit: Edit) -> String {
        let mut lines: Vec<String> = self.text.lines().map(str::to_string).collect();
        let (id, comment) = (self.id, self.comment);
        let line = |body: String| match self.markdown {
            true => body,
            false => format!("{comment} {body}"),
        };
        let heading = self.heading.filter(|at| *at < lines.len());
        match edit {
            Edit::Citation | Edit::Shorthand => {
                let token = match edit {
                    Edit::Citation => format!("{}{id}", self.marker),
                    _ => format!("$${id}"),
                };
                match self.citing_line(&lines) {
                    Some(at) => lines[at] = format!("{} {token}", lines[at]),
                    None => lines.push(line(token)),
                }
            }
            Edit::BlankAbove => lines.insert(heading.unwrap_or(0), String::new()),
            Edit::Prose => lines.extend([String::new(), line("A line of body prose.".into())]),
            Edit::Retitle => {
                if let Some(at) = heading {
                    let kept = lines[at].find(": ").map_or(lines[at].len(), |colon| colon);
                    lines[at] = format!("{}: Retitled", &lines[at][..kept]);
                }
            }
            Edit::NewHeading => {
                let doc = if self.markdown { "#" } else { self.doc };
                lines.extend([String::new(), format!("{doc} {id}9: New")]);
            }
            Edit::DeleteHeading => {
                if let Some(at) = heading {
                    lines.remove(at);
                }
            }
            Edit::Stub => lines.extend([
                String::new(),
                line(format!("# {id}9: [sweep.rs](sweep.rs)")),
            ]),
        }
        let mut text = lines.join("\n");
        if self.text.ends_with('\n') {
            text.push('\n');
        }
        text
    }

    /// Where a citation goes: the last body line of Markdown, the last comment of a source.
    fn citing_line(&self, lines: &[String]) -> Option<usize> {
        if !self.markdown {
            let comment = self.comment;
            return lines
                .iter()
                .rposition(|line| line.trim_start().starts_with(comment));
        }
        let mut fence: Option<&str> = None;
        let mut front = lines.first().is_some_and(|line| line.trim_end() == "---");
        let mut body = None;
        for (at, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if front {
                front = at == 0 || !matches!(trimmed, "---" | "...");
                continue;
            }
            if let Some(open) = fence {
                fence = (!trimmed.starts_with(open)).then_some(open);
                continue;
            }
            if let Some(open) = ["```", "~~~"]
                .into_iter()
                .find(|open| trimmed.starts_with(open))
            {
                fence = Some(open);
                continue;
            }
            let underline = |text: &str| {
                !text.is_empty() && text.chars().all(|ch| matches!(ch, '=' | '-' | '*' | '_'))
            };
            let plain = !trimmed.is_empty()
                && !line.starts_with("    ")
                && !trimmed.starts_with(['#', '<', '|', '[', '>'])
                && !underline(trimmed)
                && !lines.get(at + 1).is_some_and(|next| underline(next.trim()));
            if plain {
                body = Some(at);
            }
        }
        body
    }
}

fn comment_of(path: &Path) -> (&'static str, &'static str) {
    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    match extension {
        "rs" => ("//", "///"),
        "py" | "sh" | "bash" | "zsh" | "rb" | "pl" | "toml" | "yaml" | "yml" | "r" | "nix" => {
            ("#", "#")
        }
        "sql" | "lua" | "hs" => ("--", "--"),
        "lisp" | "el" | "clj" | "scm" => (";", ";"),
        _ => ("//", "//"),
    }
}

/// What the judgement promises stands: every declaration, section and stub by ID, file and
/// title, and every completion candidate.
#[derive(Debug, PartialEq)]
struct Catalog {
    declarations: Vec<(String, PathBuf, String, Option<String>)>,
    sections: Vec<(String, PathBuf, String, Option<String>)>,
    stubs: Vec<(String, PathBuf, PathBuf)>,
    candidates: Vec<(String, String, String, String, usize)>,
}

fn catalog(snapshot: &LspSnapshotWithCompletion) -> Catalog {
    let titles = |records: &[LspDeclaration]| {
        let mut rows: Vec<_> = records
            .iter()
            .map(|decl| {
                let path = decl.path.clone();
                (
                    decl.query_id.clone(),
                    path,
                    decl.text.clone(),
                    decl.hover_text.clone(),
                )
            })
            .collect();
        rows.sort();
        rows
    };
    let lsp = &snapshot.metadata.snapshot;
    let mut stubs: Vec<_> = lsp
        .stubs
        .iter()
        .map(|stub| {
            (
                stub.query_id.clone(),
                stub.path.clone(),
                stub.target_path.clone(),
            )
        })
        .collect();
    stubs.sort();
    Catalog {
        declarations: titles(&lsp.declarations),
        sections: titles(&lsp.sections),
        stubs,
        candidates: snapshot.completion.candidate_rows(),
    }
}

fn snapshot(
    repo: &Path,
    open: BTreeMap<PathBuf, String>,
) -> anyhow::Result<LspSnapshotWithCompletion> {
    lsp_snapshot_with_completion(LspSnapshotOpts {
        path: repo.to_path_buf(),
        path_provided: true,
        open_documents: open,
    })
}

/// Every file a project of `base` walked, with the files no reading of one file judges by what
/// the catalog records: a stub's file and both files its link can name, a file holding a case or
/// a value, a file the scan failed on, and any file under a case directory.
struct Walked {
    files: BTreeSet<PathBuf>,
    declined: BTreeSet<PathBuf>,
    cases: Vec<PathBuf>,
}

fn walked(base: &LspSnapshotWithCompletion) -> Walked {
    let mut walked = Walked {
        files: BTreeSet::new(),
        declined: BTreeSet::new(),
        cases: Vec::new(),
    };
    for project in &base.show.context.projects {
        let root = &project.config.root;
        let findings = &project.findings;
        walked
            .files
            .extend(findings.scanned_files.iter().map(|path| root.join(path)));
        for decl in findings.declarations.values().flatten() {
            let file = canonical_snapshot_path(&root.join(&decl.file));
            if decl.is_stub || decl.e2e_case.is_some() || decl.value_valid.is_some() {
                walked.declined.insert(file);
            }
            if let (true, Some(target)) = (decl.is_stub, &decl.defined_in) {
                let named = stub_target_candidates(root, &decl.file, target);
                walked
                    .declined
                    .extend(named.iter().map(|path| canonical_snapshot_path(path)));
            }
            if let Some(case) = &decl.e2e_case {
                walked
                    .cases
                    .push(canonical_snapshot_path(&root.join(&case.dir)));
            }
        }
        walked.declined.extend(
            project
                .scan_errors
                .iter()
                .map(|failure| canonical_snapshot_path(&root.join(&failure.path))),
        );
    }
    walked
}

impl Walked {
    /// Whether the prose and citation edits to `file` must be judged to leave it standing.
    fn plain_markdown(&self, file: &Path) -> bool {
        let key = canonical_snapshot_path(file);
        let name = file
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        file.extension().is_some_and(|ext| ext == "md")
            && !CONFIG_NAMES.contains(&name)
            && !self.declined.contains(&key)
            && !self.cases.iter().any(|dir| key.starts_with(dir))
    }
}

/// The sweep over one case's tree: every wrong "unchanged", every plain Markdown edit judged
/// "changed", and how many plain Markdown files were held to it. `None` where the tree does not
/// load, or where what loads is a config above it: a case tree with none of its own climbs to
/// this repository's, which is not that tree's config and would sweep the whole repository.
fn sweep(repo: &Path) -> Option<(Vec<String>, usize)> {
    let homes = [
        "grund.toml",
        ".agents/grund.toml",
        ".agent-grounds/grund.toml",
    ];
    if !homes.iter().any(|home| repo.join(home).is_file()) {
        return None;
    }
    let base = snapshot(repo, BTreeMap::new()).ok()?;
    let projects = &base.show.context.projects;
    if projects
        .iter()
        .any(|project| !canonical_snapshot_path(&project.config.root).starts_with(repo))
    {
        return None;
    }
    let before = catalog(&base);
    let walked = walked(&base);
    let lsp = &base.metadata.snapshot;
    let judge = base.show.typing_judge();
    let (mut wrong, mut plain) = (Vec::new(), 0);
    let fallback_id = lsp
        .declarations
        .first()
        .map_or("FS-sweep", |decl| decl.text.as_str());
    for file in &walked.files {
        let Ok(text) = std::fs::read_to_string(file) else {
            continue;
        };
        let key = canonical_snapshot_path(file);
        let heading = lsp
            .declarations
            .iter()
            .filter(|decl| canonical_snapshot_path(&decl.path) == key)
            .map(|decl| decl.line.saturating_sub(1))
            .min();
        let (comment, doc) = comment_of(file);
        let subject = Subject {
            text: &text,
            markdown: file.extension().is_some_and(|ext| ext == "md"),
            comment,
            doc,
            marker: &lsp.marker,
            id: fallback_id,
            heading,
        };
        let must_stand = walked.plain_markdown(file);
        plain += usize::from(must_stand);
        for edit in EDITS {
            let edited = subject.edited(edit);
            if edited == text {
                continue;
            }
            let judged = judge.declarations_stand(&base.show, file, &edited);
            let at = format!("{}: {edit:?}", file.display());
            if must_stand && matches!(edit, Edit::Citation | Edit::Prose) && judged != Some(true) {
                wrong.push(format!(
                    "{at}: judged {judged:?}, a plain Markdown edit stands"
                ));
            }
            if judged != Some(true) {
                continue;
            }
            match snapshot(repo, BTreeMap::from([(file.clone(), edited)])) {
                Ok(after) if catalog(&after) == before => {}
                Ok(after) => wrong.push(format!(
                    "{at}: judged unchanged, scanned to {:#?}, was {before:#?}",
                    catalog(&after)
                )),
                Err(err) => wrong.push(format!("{at}: judged unchanged, the scan failed: {err:#}")),
            }
        }
    }
    Some((wrong, plain))
}

/// §FS-lsp.responsiveness.4, §AR-lsp.3.2: conservative, never wrong, over every case's tree.
#[test]
fn the_file_again_judgement_is_never_wrong_across_every_e2e_case() {
    let cases = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/e2e/cases");
    // Absent from a packaged crate, where nothing here can break the promise.
    let Ok(cases) = std::fs::canonicalize(cases) else {
        return;
    };
    let mut repos: Vec<PathBuf> = std::fs::read_dir(&cases)
        .unwrap()
        .filter_map(|entry| Some(entry.ok()?.path().join("repo")))
        .filter(|repo| repo.is_dir())
        .collect();
    repos.sort();
    let swept: Vec<(Vec<String>, usize)> =
        repos.par_iter().filter_map(|repo| sweep(repo)).collect();
    let plain: usize = swept.iter().map(|(_, plain)| plain).sum();
    let wrong: Vec<&String> = swept.iter().flat_map(|(wrong, _)| wrong).collect();
    assert!(
        wrong.is_empty(),
        "{} judgements were wrong across {} trees; the first:\n{}",
        wrong.len(),
        swept.len(),
        wrong
            .iter()
            .take(10)
            .map(|line| line.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(
        plain >= PLAIN_MARKDOWN_FLOOR,
        "only {plain} plain Markdown files across {} trees were held to \"unchanged\"",
        swept.len()
    );
}
