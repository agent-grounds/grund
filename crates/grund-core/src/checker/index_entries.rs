//! The membership half of the kind-index invariant (§AR-checker.2.16), split
//! from `index.rs` so adding external enrollment does not push that rule
//! family past the core-source file budget (§AR-core-module-layout.3).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use super::index::{KindIndexTarget, declarations_under_folder, kind_index_targets};
use super::index_enrollment::enroll_external_inline_declarations;
use super::spellings_report::PlaceNames;
use crate::config::{Frame, Owners, Schema};
use crate::model::{
    Catalog, Citation, HeadingKind, Id, scanned_decl_relative_path, scanned_path_key,
};
use crate::sources::physical_path_key;

/// The IDs each kind index owes an entry for, keyed by the index's
/// config-root-relative path (§FS-check.3.18). `folder_owed` preserves the
/// original folder-membership rule; `owed` additionally holds external inline
/// declarations enrolled by a canonical link. Their exact sites stay separate
/// so another citation of the same external ID on the page remains real use
/// (§DF-index-not-an-inbound-citation.2.2). `binding_sites` holds the citation
/// sites of the recognized value bindings on each index, which are use rather
/// than the entry even for a folder-owned ID (§FS-check.4.1.2.1). `headings`
/// holds each index's declaration heading lines, whose citations are the entry
/// of no ID (§FS-check.4.1.2.1).
pub(crate) struct KindIndexEntries {
    configured_root: PathBuf,
    physical_root: PathBuf,
    owed: BTreeMap<PathBuf, BTreeSet<Id>>,
    folder_owed: BTreeMap<PathBuf, BTreeSet<Id>>,
    external_sites: BTreeMap<PathBuf, BTreeSet<(usize, usize)>>,
    binding_sites: BTreeMap<PathBuf, BTreeSet<(usize, usize)>>,
    headings: IndexHeadings,
    names: IndexNames,
}

/// The other names of each configured index, keyed as a scanned file's relative path is read: a
/// directory alias lists `docslink/fs/README.md`, and that is still `docs/fs/README.md`
/// (§AR-scanner.1.8). Empty on a tree with no second name, so a lookup there is
/// one miss in an empty map (§AR-scanner.1.8).
#[derive(Default)]
pub(super) struct IndexNames(BTreeMap<PathBuf, Vec<PathBuf>>);

impl IndexNames {
    pub(super) fn new(
        targets: &[KindIndexTarget<'_>],
        frame: Frame<'_>,
        configured_root: &Path,
        physical_root: &Path,
    ) -> Self {
        let spellings = &frame.run.spellings;
        let mut names = Self::default();
        if !spellings.has_second_names() {
            return names;
        }
        for target in targets {
            for name in spellings.names_of(&target.index_file) {
                if let Some(relative) =
                    scanned_decl_relative_path(&name, configured_root, physical_root)
                    && relative.as_ref() != target.index_key
                {
                    let keys = names.0.entry(relative.into_owned()).or_default();
                    keys.push(target.index_key.clone());
                }
            }
        }
        names
    }

    /// Every other name of an index, as a scanned file's relative path is read.
    pub(super) fn relative_names(self) -> impl Iterator<Item = PathBuf> {
        self.0.into_keys()
    }

    /// `relative` as written, and every index it is another name of.
    pub(super) fn keys<'a>(&'a self, relative: &'a Path) -> impl Iterator<Item = &'a Path> {
        let others = self.0.get(relative).into_iter().flatten();
        std::iter::once(relative).chain(others.map(PathBuf::as_path))
    }
}

impl KindIndexEntries {
    /// `index_targets` is `Expected`'s canonical link per (index file, ID), the
    /// destination an external enrollment is compared against (§AR-checker.2.16).
    pub(crate) fn new(
        findings: &Catalog,
        schema: &Schema,
        frame: Frame<'_>,
        index_targets: &BTreeMap<(PathBuf, Id), String>,
    ) -> Self {
        Self::build(findings, schema, frame, index_targets, None)
    }

    /// The same classifier, owing entries only for `ids`: `is_index_entry`
    /// answers a citation of one of them exactly as `new`'s would, without
    /// walking every declaration of the kind (§GOAL-fast-feedback).
    pub(crate) fn restricted(
        findings: &Catalog,
        schema: &Schema,
        frame: Frame<'_>,
        index_targets: &BTreeMap<(PathBuf, Id), String>,
        ids: &BTreeSet<Id>,
    ) -> Self {
        Self::build(findings, schema, frame, index_targets, Some(ids))
    }

    fn build(
        findings: &Catalog,
        schema: &Schema,
        frame: Frame<'_>,
        index_targets: &BTreeMap<(PathBuf, Id), String>,
        only: Option<&BTreeSet<Id>>,
    ) -> Self {
        let configured_root = scanned_path_key(frame.root());
        let physical_root = physical_path_key(frame.root());
        let targets = kind_index_targets(schema, frame);
        let owners = Owners::new(schema, frame);
        let spellings = PlaceNames::new(schema, frame);
        let mut folder_owed: BTreeMap<PathBuf, BTreeSet<Id>> = BTreeMap::new();
        for target in &targets {
            for (id, decls) in &findings.declarations {
                if id.kind == target.kind
                    && only.is_none_or(|ids| ids.contains(id))
                    && declarations_under_folder(decls, target.folder, &owners, &spellings)
                {
                    folder_owed
                        .entry(target.index_key.clone())
                        .or_default()
                        .insert(id.clone());
                }
            }
        }
        let mut owed = folder_owed.clone();
        let names = IndexNames::new(&targets, frame, &configured_root, &physical_root);
        let headings =
            IndexHeadings::new(findings, &targets, &names, &configured_root, &physical_root);
        let external_sites = enroll_external_inline_declarations(
            findings,
            &targets,
            &owners,
            &spellings,
            &names,
            &configured_root,
            &physical_root,
            index_targets,
            frame.sources(),
            only,
            &headings,
            &mut owed,
        );
        let binding_sites = index_value_binding_sites(
            findings,
            &folder_owed,
            &names,
            &configured_root,
            &physical_root,
        );
        Self {
            configured_root,
            physical_root,
            owed,
            folder_owed,
            external_sites,
            binding_sites,
            headings,
            names,
        }
    }

    /// The other names of the configured indexes, which the citations of one are recorded under.
    pub(super) fn names(&self) -> &IndexNames {
        &self.names
    }

    /// The IDs this index owes an entry for, or `None` when `path` is not a
    /// configured index (§FS-fmt.6.1.2). What the always-linkify carve-out wraps:
    /// declarations under the folder plus an external inline ID only after its
    /// canonical enrollment link exists (§FS-check.3.18).
    pub(crate) fn entries_in(&self, path: &Path) -> Option<&BTreeSet<Id>> {
        if self.owed.is_empty() {
            return None;
        }
        let relative =
            scanned_decl_relative_path(path, &self.configured_root, &self.physical_root)?;
        self.names
            .keys(relative.as_ref())
            .find_map(|key| self.owed.get(key))
    }

    /// §FS-check.4.1.2 / §DF-index-not-an-inbound-citation.2.2: folder-owned IDs
    /// keep PR #134's accounting. For an external ID only the canonical link site
    /// is navigation; a second same-ID citation in the index is ordinary use.
    /// §FS-check.4.1.2.1: a folder-owned ID's entry is a citation of the
    /// declaration itself, so a section citation, or the citation inside a
    /// recognized value binding (a root-aimed one with no section included), is use.
    /// So is a citation on a declaration heading, for either branch: it is a
    /// citation from the declaration that heading declares, the entry of no
    /// folder-owned ID (§FS-check.4.1.2.1) and the enrollment of no external one
    /// (§FS-check.3.18.3).
    pub(crate) fn is_index_entry(&self, citation: &Citation) -> bool {
        if citation.namespace.is_some() || self.owed.is_empty() {
            return false;
        }
        let Some(relative) =
            scanned_decl_relative_path(&citation.file, &self.configured_root, &self.physical_root)
        else {
            return false;
        };
        let site = (citation.line, citation.column);
        self.names.keys(relative.as_ref()).any(|key| {
            !self.headings.holds(key, citation.line)
                && ((citation.section.is_none()
                    && self
                        .folder_owed
                        .get(key)
                        .is_some_and(|ids| ids.contains(&citation.id))
                    && !self
                        .binding_sites
                        .get(key)
                        .is_some_and(|sites| sites.contains(&site)))
                    || self
                        .external_sites
                        .get(key)
                        .is_some_and(|sites| sites.contains(&site)))
        })
    }

    /// Whether line `line` of the kind index at `path` is a declaration heading:
    /// the one answer the entry rule, its mention look, the unused carve-out, `list
    /// --unused` and the `refs` label all read (§AR-checker.2.16). A citation on one
    /// is the entry of no ID (§FS-check.4.1.2.1), and the line declares an ID rather
    /// than mentioning one (§FS-check.3.18.5.1).
    pub(super) fn on_declaration_heading(&self, path: &Path, line: usize) -> bool {
        scanned_decl_relative_path(path, &self.configured_root, &self.physical_root).is_some_and(
            |relative| {
                self.names
                    .keys(relative.as_ref())
                    .any(|key| self.headings.holds(key, line))
            },
        )
    }
}

/// The lines of each kind index the scan read as a declaration heading, keyed by
/// the index's config-root-relative path (§AR-checker.2.16). They are read off the
/// index's file record, the record of the text every look at the index reads
/// (§AR-scanner.2.8.5), so the line is not parsed again. The scan records no
/// heading in a fence or in a raw-text HTML block (§FS-check.1.1.5.1), so a
/// recorded line needs no second test of its own.
pub(super) struct IndexHeadings(BTreeMap<PathBuf, BTreeSet<usize>>);

impl IndexHeadings {
    fn new(
        findings: &Catalog,
        targets: &[KindIndexTarget<'_>],
        names: &IndexNames,
        configured_root: &Path,
        physical_root: &Path,
    ) -> Self {
        let mut lines: BTreeMap<PathBuf, BTreeSet<usize>> = BTreeMap::new();
        if targets.is_empty() {
            return Self(lines);
        }
        let keys: BTreeSet<&Path> = targets
            .iter()
            .map(|target| target.index_key.as_path())
            .collect();
        for (file, record) in &findings.files {
            let Some(relative) = scanned_decl_relative_path(file, configured_root, physical_root)
            else {
                continue;
            };
            for key in names.keys(relative.as_ref()) {
                if keys.contains(key) {
                    lines.entry(key.to_path_buf()).or_default().extend(
                        record
                            .headings
                            .iter()
                            .filter(|heading| heading.kind == HeadingKind::Declaration)
                            .map(|heading| heading.line),
                    );
                }
            }
        }
        Self(lines)
    }

    /// Whether line `line` of the index at config-root-relative `index` is one.
    pub(super) fn holds(&self, index: &Path, line: usize) -> bool {
        self.0.get(index).is_some_and(|lines| lines.contains(&line))
    }
}

/// The citation sites of the scanner's recognized value bindings on each kind
/// index (§FS-check.4.1.2.1, §FS-values.3.2), read off the catalog's binding
/// records rather than re-parsed.
fn index_value_binding_sites(
    findings: &Catalog,
    folder_owed: &BTreeMap<PathBuf, BTreeSet<Id>>,
    names: &IndexNames,
    configured_root: &Path,
    physical_root: &Path,
) -> BTreeMap<PathBuf, BTreeSet<(usize, usize)>> {
    let mut sites: BTreeMap<PathBuf, BTreeSet<(usize, usize)>> = BTreeMap::new();
    for binding in &findings.value_bindings {
        if binding.namespace.is_some() {
            continue;
        }
        let Some(relative) =
            scanned_decl_relative_path(&binding.file, configured_root, physical_root)
        else {
            continue;
        };
        for key in names.keys(relative.as_ref()) {
            if folder_owed.contains_key(key) {
                sites
                    .entry(key.to_path_buf())
                    .or_default()
                    .insert((binding.line, binding.citation_column));
            }
        }
    }
    sites
}
