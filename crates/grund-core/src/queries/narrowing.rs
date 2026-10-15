//! §FS-scan.path.warning: where a path narrower than its config root changes what
//! `show`, `list` or `id` answers, the line that says so — release N of ruling D1
//! of §DF-one-answer-rulings-run, decided in §DF-path-narrowing-warns. Each answer
//! keeps its bytes; the line is computed beside it and the frontend writes it last.
//!
//! One gate for all three: the report scope `check` narrows to (§FS-scan.path.check),
//! which is `None` for a run with no path or one at its config root, so such a run
//! reads nothing more (§FS-scan.path.warning.cost). The owning project is read the
//! way `check <path>` reads it, through the `Check` selection, so the scope fact has
//! one implementation (§AR-system.principle).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::citation_counts::{ListCitationCounts, kind_index_entries};
use crate::checker::{ScanScope, path_report_scope};
use crate::config::{Config, Frame, Invocation, Schema, Selection, display_path};
use crate::grammar::render_id;
use crate::model::{Catalog, Citation, Diagnostic, Id, paired_declarations, sort_path_key};
use crate::resolver::{WorkspaceContext, WorkspaceProject, load_invoked_workspace_context};
use crate::scanner::{id_arg_targets, scan_tree};
use crate::workspace::{open, split_qualified_id_arg};

/// §FS-scan.path.warning.release: the release from which a path narrows what is
/// reported and not what resolves. Every narrowing line names it, and so does the
/// alias clause of §FS-scan.projects.narrowed.alias.
pub(crate) const PATH_RESOLVES_PROJECT_WIDE: &str = "0.19.0";

/// §FS-scan.path.warning: the clause every narrowing line ends in.
fn clause() -> String {
    format!(
        "; from grund {PATH_RESOLVES_PROJECT_WIDE} a path narrows what is reported, not what resolves"
    )
}

/// A site outside the path, keyed for §FS-scan.path.warning.shape's path-then-line
/// order and rendered against the output's base.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct Site {
    key: (String, usize),
    text: String,
}

/// §FS-scan.path.warning.cost: a path narrower than its config root, as the caller
/// wrote it, and the files it bounds.
pub(crate) struct NarrowedPath {
    operand: String,
    scope: ScanScope,
}

impl NarrowedPath {
    /// `None` for no path, or one at or above the config root: nothing narrows.
    pub(crate) fn of(
        schema: &Schema,
        frame: Frame<'_>,
        path: &Path,
        path_provided: bool,
    ) -> Option<Self> {
        let scope = path_report_scope(schema, frame, path, path_provided).ok()??;
        Some(Self {
            operand: path.display().to_string(),
            scope,
        })
    }

    /// §FS-scan.path.warning.shape: the first of `sites` outside the path, with
    /// what it was found on.
    fn first_outside<'a, T>(
        &self,
        config: &Config,
        sites: impl IntoIterator<Item = (T, &'a PathBuf, usize)>,
    ) -> Option<(T, Site)> {
        sites
            .into_iter()
            .filter(|(_, file, _)| !self.scope.contains(file))
            .map(|(on, file, line)| {
                let site = Site {
                    key: (sort_path_key(file), line),
                    text: format!("{}:{line}", display_path(config, file)),
                };
                (on, site)
            })
            .min_by(|a, b| a.1.cmp(&b.1))
    }
}

/// The owning project a narrowed `show` or `list` run resolves against from
/// grund 0.19.0, read once (§FS-scan.path.warning.cost).
pub(crate) struct Owner {
    path: NarrowedPath,
    context: WorkspaceContext,
}

impl Owner {
    /// `narrow` is the run's own context. A workspace run already reads each
    /// project's whole root, so only a one-project run can have narrowed.
    pub(crate) fn of(narrow: &WorkspaceContext, path: &Path, path_provided: bool) -> Option<Self> {
        if narrow.workspace_loaded {
            return None;
        }
        let config = &narrow.current_project()?.config;
        let narrowed = NarrowedPath::of(config.schema(), config.frame(), path, path_provided)?;
        // §FS-scan.path.warning: read as `check`'s resolution scope is (§FS-scan.path.check).
        let invocation = Invocation::new(path, true, Selection::Check).classify(false);
        let context = load_invoked_workspace_context(&invocation).ok()?;
        (!context.workspace_loaded).then_some(Self {
            path: narrowed,
            context,
        })
    }

    fn project(&self) -> &WorkspaceProject {
        &self.context.projects[0]
    }

    /// §FS-scan.path.show: the line for one query, if the owning project moves it —
    /// a miss it declares outside the path, or a hit it also declares there, or a
    /// short form it resolves to more IDs than the path holds.
    pub(crate) fn show_line(&self, narrow: &WorkspaceContext, id_arg: &str) -> Option<String> {
        let (alias, raw_id) = split_qualified_id_arg(id_arg).ok()?;
        if alias.is_some() {
            return None;
        }
        let owner = self.project();
        let (config, findings) = (&owner.config, &owner.findings);
        let targets = id_arg_targets(raw_id, config.schema(), config.frame(), findings).ok()?;
        // A paired declaration is named at its record, as plain `list` names it
        // (§FS-declarations.stubs.pairing.4).
        let homes = |keep: &dyn Fn(&Id) -> bool| {
            targets
                .iter()
                .filter(|(id, _)| keep(id))
                .filter_map(|(id, _)| findings.declarations.get(id))
                .flat_map(|decls| paired_declarations(decls).records().collect::<Vec<_>>())
                .collect::<Vec<_>>()
        };
        // Each ID the query names within the path, with its count of homes there.
        let inside = narrow.current_project().map_or_else(Vec::new, |project| {
            let (schema, frame) = (project.config.schema(), project.config.frame());
            id_arg_targets(raw_id, schema, frame, &project.findings)
                .unwrap_or_default()
                .into_iter()
                .filter_map(|(id, _)| {
                    let decls = project.findings.declarations.get(&id)?;
                    Some((id, paired_declarations(decls).len()))
                })
                .collect::<Vec<_>>()
        });
        let (verb, homes) = match inside.as_slice() {
            [] => ("is declared", homes(&|_| true)),
            [(_, 1)] if homes(&|_| true).len() > 1 => ("is also declared", homes(&|_| true)),
            // Already ambiguous within the path: only a match it does not hold moves it.
            [_, _, ..] => (
                "is also declared",
                homes(&|id| inside.iter().all(|(held, _)| held != id)),
            ),
            _ => return None,
        };
        let (_, outside) = self
            .path
            .first_outside(config, homes.iter().map(|home| ((), &home.file, home.line)))?;
        // `<ID>` as the query wrote it, without its section.
        let section = targets.first().and_then(|(_, section)| section.as_deref());
        let written = section
            .and_then(|section| {
                raw_id
                    .strip_suffix(section)?
                    .strip_suffix(&*config.section_separator)
            })
            .unwrap_or(raw_id);
        Some(format!(
            "{written} {verb} at {}, outside {}{}",
            outside.text,
            self.path.operand,
            clause()
        ))
    }

    /// §FS-scan.path.list: what the owning project moves about each catalog entry
    /// the narrowed run lists, keyed by the entry's rendered ID.
    pub(crate) fn list_moves(&self, narrow: &WorkspaceContext, unused_only: bool) -> ListMoves {
        let Some(listed) = narrow.current_project() else {
            return ListMoves::default();
        };
        let owner = self.project();
        let config = &owner.config;
        let (narrow_counts, owner_counts) = (
            ListCitationCounts::new(narrow),
            ListCitationCounts::new(&self.context),
        );
        let count = |counts: &BTreeMap<&Id, usize>, id: &Id| counts.get(id).copied().unwrap_or(0);
        // §FS-list.1.3: an index entry is not use, so it never drops an entry.
        let index_entries = kind_index_entries(owner);
        let mut moves = BTreeMap::new();
        for (id, decls) in &listed.findings.declarations {
            let citations = |use_only: bool| {
                self.path
                    .first_outside(
                        config,
                        owner
                            .findings
                            .edges()
                            .filter(|cite| cite.namespace.is_none() && &cite.id == id)
                            .filter(|cite| !use_only || !index_entries.is_index_entry(cite))
                            .map(|cite| ((), &cite.file, cite.line)),
                    )
                    .map(|(_, site)| site)
            };
            let refs = (count(narrow_counts.refs_for(""), id)
                != count(owner_counts.refs_for(""), id))
            .then(|| citations(false))
            .flatten();
            let vanishes = (unused_only
                && count(narrow_counts.used_for(""), id) == 0
                && count(owner_counts.used_for(""), id) > 0)
                .then(|| citations(true))
                .flatten();
            let duplicate = owner
                .findings
                .declarations
                .get(id)
                .map(|homes| paired_declarations(homes))
                .filter(|homes| paired_declarations(decls).len() <= 1 && homes.len() > 1)
                .and_then(|homes| {
                    let homes = homes.records().map(|home| ((), &home.file, home.line));
                    self.path.first_outside(config, homes).map(|(_, site)| site)
                });
            if refs.is_some() || vanishes.is_some() || duplicate.is_some() {
                moves.insert(
                    render_id(&listed.config.grammar, id),
                    Moved {
                        refs,
                        duplicate,
                        vanishes,
                    },
                );
            }
        }
        ListMoves {
            operand: self.path.operand.clone(),
            moves,
        }
    }
}

/// What the owning project moves about one listed ID (§FS-scan.path.list).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct Moved {
    /// A citation outside the path, which changes the JSON `refs`.
    refs: Option<Site>,
    /// Another home outside the path, which adds the duplicate note.
    duplicate: Option<Site>,
    /// A citation outside the path, which drops the entry from `--unused`.
    vanishes: Option<Site>,
}

/// §FS-scan.path.list, by rendered ID; empty where the path narrows nothing.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ListMoves {
    operand: String,
    moves: BTreeMap<String, Moved>,
}

/// Which of a catalog entry's fields the frontend prints (§FS-scan.path.list):
/// the JSON row shows `refs` and `duplicate`, the text row the duplicate note,
/// and `--summary` neither.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ListShown {
    pub refs: bool,
    pub duplicate: bool,
}

/// One printed catalog entry the owning project moves (§FS-scan.path.list).
#[derive(Clone, Debug, Eq, PartialEq)]
struct NarrowedEntry {
    id: String,
    /// A chapter row prints neither `refs` nor the duplicate note (§FS-rules.8).
    chapter: bool,
    moved: Moved,
}

/// The `list` lines a narrowed path owes, held until the frontend knows which
/// fields it prints (§FS-scan.path.list, §FS-scan.path.warning.shape).
#[doc(hidden)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ListNarrowing {
    operand: String,
    entries: Vec<NarrowedEntry>,
}

impl ListNarrowing {
    /// One per printed entry whose output the owning project moves, in output order.
    pub(crate) fn of<'a>(
        moves: ListMoves,
        printed: impl IntoIterator<Item = (&'a str, bool)>,
    ) -> Self {
        let entries = printed
            .into_iter()
            .filter_map(|(id, chapter)| {
                moves.moves.get(id).map(|moved| NarrowedEntry {
                    id: id.to_string(),
                    chapter,
                    moved: moved.clone(),
                })
            })
            .collect();
        Self {
            operand: moves.operand,
            entries,
        }
    }

    /// The lines, without the `warning: ` prefix, for a frontend printing `shown`.
    pub fn lines(&self, shown: ListShown) -> Vec<String> {
        self.entries
            .iter()
            .filter_map(|entry| {
                let moved = &entry.moved;
                let fields = !entry.chapter;
                [
                    moved.vanishes.as_ref(),
                    moved.refs.as_ref().filter(|_| fields && shown.refs),
                    moved
                        .duplicate
                        .as_ref()
                        .filter(|_| fields && shown.duplicate),
                ]
                .into_iter()
                .flatten()
                .min()
                .map(|site| {
                    format!(
                        "{} entry depends on {}, outside {}{}",
                        entry.id,
                        site.text,
                        self.operand,
                        clause()
                    )
                })
            })
            .collect()
    }
}

/// §FS-scan.path.id: the line for a proposal the owning project moves — a higher
/// number of the kind held outside the path, or, under a number-less format, the
/// proposal itself declared there. `narrow` is the run's own config; the owning
/// project is read only where the path narrows (§FS-scan.path.warning.cost).
/// `render` writes an ID against the owning project's config, as `id` writes its own.
pub(crate) fn id_line(
    narrow: &Config,
    path: &Path,
    path_provided: bool,
    proposal: &Id,
    render: impl Fn(&Id, &Config) -> String,
) -> Option<String> {
    let narrowed = NarrowedPath::of(narrow.schema(), narrow.frame(), path, path_provided)?;
    // §FS-scan.path.warning: read as `check`'s resolution scope is (§FS-scan.path.check).
    let config = open(&Invocation::new(path, true, Selection::Check).classify(false)).ok()?;
    let (findings, _) = scan_tree(config.schema(), config.frame(), Some(path), true).ok()?;
    id_line_in(&narrowed, &config, &findings, proposal, render)
}

fn id_line_in(
    path: &NarrowedPath,
    config: &Config,
    findings: &Catalog,
    proposal: &Id,
    render: impl Fn(&Id, &Config) -> String,
) -> Option<String> {
    if let Some(number) = proposal.num {
        let of_kind = || {
            findings
                .declarations
                .iter()
                .filter(|(id, _)| id.kind == proposal.kind)
        };
        let highest = of_kind().filter_map(|(id, _)| id.num).max()?;
        if highest >= number {
            let holders = of_kind()
                .filter(|(id, _)| id.num == Some(highest))
                .flat_map(|(id, decls)| decls.iter().map(move |decl| (id, &decl.file, decl.line)));
            let (holder, site) = path.first_outside(config, holders)?;
            let moved = Id {
                num: Some(highest + 1),
                ..proposal.clone()
            };
            return Some(format!(
                "{} at {}, outside {}, holds the highest {} number{}, and id proposes {}",
                render_id(&config.grammar, holder),
                site.text,
                path.operand,
                proposal.kind,
                clause(),
                render(&moved, config)
            ));
        }
    }
    let homes = findings.declarations.get(proposal)?;
    let homes = homes.iter().map(|decl| ((), &decl.file, decl.line));
    let (_, site) = path.first_outside(config, homes)?;
    Some(format!(
        "{} is already declared at {}, outside {}{}, and id refuses it",
        render(proposal, config),
        site.text,
        path.operand,
        clause()
    ))
}

/// §FS-scan.projects.narrowed.alias: a narrowed `check <path>` appends one clause to
/// each `unknown project alias <alias>` the workspace its config root declares
/// resolves, keeping the message a verbatim prefix (§FS-errors.3). The kind table a
/// narrowed run installs is that workspace's projects by alias (§AR-config.1.5); a
/// member run, or a run with no workspace, holds none and gets nothing.
pub(crate) fn name_the_workspace_root(
    errors: &mut [Diagnostic],
    citations: &[Citation],
    config: &Config,
    path: &Path,
) {
    let table = config.kind_table();
    for error in errors
        .iter_mut()
        .filter(|error| error.code == "unknown-project")
    {
        let alias = citations
            .iter()
            .find(|cite| {
                Some(&cite.file) == error.path.as_ref()
                    && Some(cite.line) == error.line
                    && Some(cite.column) == error.column
            })
            .and_then(|cite| cite.namespace.as_deref());
        if alias.is_some_and(|alias| table.holds_alias(alias)) {
            error.message.push_str(&format!(
                "; grund check from the workspace root resolves this alias, and from grund \
                 {PATH_RESOLVES_PROJECT_WIDE} grund check {} will too",
                path.display()
            ));
        }
    }
}
