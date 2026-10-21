//! Scoped guidance (§FS-init.2.3.5.11): where each row's guidance lands, and the
//! sections a destination's managed block renders. One placement feeds the root
//! block, every scoped block and the `### Scoped guidance` pointer, so `init`
//! writes and `check` compares one answer (§AR-checker.2.7).

use std::collections::BTreeMap;
use std::path::{Component, Path};

use crate::config::{Extent, Project, Row, Run, SchemaView};
use crate::grammar::agents_block_heading;

use super::citation_directions::{
    bullet_row_order, root_citation_directions, row_has_bullet, scoped_citation_directions,
};
use super::citation_grounding::row_has_grounding_clause;

/// The root destination: the root entrypoint's own block (§FS-init.2.3.5.11.1).
const ROOT: &str = "";

/// Where every row that renders something lands (§FS-init.2.3.5.11).
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Layout {
    /// Row name to destination directory, `""` for the root; a row that renders
    /// nothing is absent (§FS-init.2.3.5.11.3).
    destinations: BTreeMap<String, String>,
    /// The non-root destinations in pointer order, each with the units its line
    /// names (§FS-init.2.3.5.13).
    scoped: Vec<(String, Vec<String>)>,
    /// The complement's destination when it is not the root, which carries the
    /// pointer too (§FS-init.2.3.5.12.1).
    complement: Option<String>,
}

impl Layout {
    /// Place every row. `subjects` names the kinds a chapter rule is about
    /// (§FS-init.2.3.5.10); `exists` says whether a destination directory exists
    /// at this run, and one that does not renders at the root
    /// (§FS-init.2.3.5.11.4).
    pub(crate) fn of(
        project: &Project,
        run: &Run,
        subjects: &[String],
        exists: &dyn Fn(&str) -> bool,
    ) -> Self {
        if project.presentation.rules_at_root {
            // §FS-init.2.3.5.11.5: every row renders at the root.
            return Self::default();
        }
        let shaped = shaped_kinds(project);
        let homeless = project.schema.complement_name().to_string();
        let mut layout = Self::default();
        for name in bullet_row_order(project) {
            let renders = row_has_bullet(project, name)
                || row_has_grounding_clause(project, run, name)
                || shaped.iter().any(|kind| kind == name)
                || subjects.iter().any(|kind| kind == name);
            if !renders {
                continue;
            }
            let row = project.schema.rows.iter().find(|row| row.name == name);
            let dir = row
                .map(destination)
                .filter(|dir| dir.is_empty() || exists(dir))
                .unwrap_or_default();
            if !dir.is_empty() {
                let unit = row.map_or_else(|| name.to_string(), |row| unit_label(row, &homeless));
                match layout.scoped.iter_mut().find(|(at, _)| *at == dir) {
                    Some((_, units)) => units.push(unit),
                    None => layout.scoped.push((dir.clone(), vec![unit])),
                }
                if name == homeless {
                    layout.complement = Some(dir.clone());
                }
            }
            layout.destinations.insert(name.to_string(), dir);
        }
        layout
    }

    /// The non-root destination directories, in pointer order.
    pub(crate) fn scoped_dirs(&self) -> impl Iterator<Item = &str> {
        self.scoped.iter().map(|(dir, _)| dir.as_str())
    }

    /// Whether any guidance lands outside the root.
    pub(crate) fn is_scoped(&self) -> bool {
        !self.scoped.is_empty()
    }

    /// Whether `row` renders in the block of `dir` (`""` for the root): a row
    /// with no destination renders nothing, so the root may claim it.
    pub(crate) fn lands(&self, row: &str, dir: &str) -> bool {
        self.destinations.get(row).map_or(ROOT, String::as_str) == dir
    }

    /// The root block's `### Citation directions` (§FS-init.2.3.5.1).
    pub(crate) fn root_citation_directions(&self, project: &Project, run: &Run) -> String {
        root_citation_directions(project, run, &|row| self.lands(row, ROOT))
    }

    /// The `### Declaration shapes` of `dir`'s block, `None` when no kind with a
    /// shape lands there (§FS-init.2.3.5.12.2).
    pub(crate) fn declaration_shapes(&self, project: &Project, dir: &str) -> Option<String> {
        declaration_shapes_section(project, &|row| self.lands(row, dir))
    }

    /// The `### Scoped guidance` pointer (§FS-init.2.3.5.13), `None` when no
    /// row lands outside the root.
    pub(crate) fn pointer(&self) -> Option<String> {
        if self.scoped.is_empty() {
            return None;
        }
        let mut section = String::from(
            "### Scoped guidance\n\nSome places' citation directions and declaration shapes are in the `AGENTS.md` of the directory above them. If your agent has not already loaded that file, read it before writing there:\n\n",
        );
        for (dir, units) in &self.scoped {
            section.push_str(&format!("- `{dir}/AGENTS.md`: {}\n", units.join(", ")));
        }
        Some(section)
    }

    /// The managed block of the scoped `AGENTS.md` in `dir`, at `version`
    /// (§FS-init.2.3.5.12.1). `chapter_rules` is that destination's
    /// `### Chapter rules` section, rendered by the caller for this file.
    pub(crate) fn scoped_block(
        &self,
        project: &Project,
        run: &Run,
        dir: &str,
        version: u32,
        chapter_rules: Option<&str>,
    ) -> String {
        let sections = self.scoped_sections(project, run, dir, chapter_rules);
        // §FS-init.2.3.9.3: the same labelled version heading as the root block.
        let mut block = format!(
            "<!-- BEGIN GRUND MANAGED BLOCK -->\n{}\n\nThese rules cover the files under `{dir}/`; the rest of this project's grund guidance is in the root `AGENTS.md`.\n",
            agents_block_heading(version)
        );
        for (_, bytes) in sections {
            if let Some(bytes) = bytes {
                block.push('\n');
                block.push_str(bytes.trim_end());
                block.push('\n');
            }
        }
        block.push_str("<!-- END GRUND MANAGED BLOCK -->\n");
        block
    }

    /// The four sections of `dir`'s scoped block by heading, in block order,
    /// each `None` where it has no content (§FS-init.2.3.5.12.1).
    pub(crate) fn scoped_sections(
        &self,
        project: &Project,
        run: &Run,
        dir: &str,
        chapter_rules: Option<&str>,
    ) -> [(&'static str, Option<String>); 4] {
        let pointer = (self.complement.as_deref() == Some(dir))
            .then(|| self.pointer())
            .flatten();
        [
            (
                "### Citation directions",
                scoped_citation_directions(project, run, &|row| self.lands(row, dir)),
            ),
            (
                "### Declaration shapes",
                self.declaration_shapes(project, dir),
            ),
            ("### Chapter rules", chapter_rules.map(str::to_string)),
            ("### Scoped guidance", pointer),
        ]
    }
}

/// The deepest directory holding every place of `row`: a folder counts as
/// itself, a file as its parent, and no common leading component — or a place
/// that climbs out with `..` — is the root (§FS-init.2.3.5.11.1). A row with no
/// place, the homeless kind or a complement over the config root, is the root
/// too (§FS-init.2.3.5.11.2).
pub(crate) fn destination(row: &Row) -> String {
    let mut common: Option<Vec<String>> = None;
    for place in &row.places {
        let dir = match &place.extent {
            Extent::Folder(path) => Path::new(path),
            Extent::File(path) => Path::new(path).parent().unwrap_or(Path::new("")),
            Extent::Complement => continue,
        };
        let mut parts = Vec::new();
        for component in dir.components() {
            match component {
                Component::Normal(part) => parts.push(part.to_string_lossy().into_owned()),
                Component::CurDir => {}
                _ => return ROOT.to_string(),
            }
        }
        common = Some(match common {
            None => parts,
            Some(mut shared) => {
                let same = shared
                    .iter()
                    .zip(&parts)
                    .take_while(|(a, b)| a == b)
                    .count();
                shared.truncate(same);
                shared
            }
        });
    }
    common.unwrap_or_default().join("/")
}

/// How the pointer names a row: a citable kind by its name, the complement as
/// its name, a non-citable row by its places (§FS-init.2.3.5.13).
fn unit_label(row: &Row, homeless: &str) -> String {
    if row.name == homeless || row.kind.is_some() {
        return row.name.clone();
    }
    row.place_labels().unwrap_or_else(|| row.name.clone())
}

/// The kinds whose declaration shape renders: those `grund schema` prints a
/// field or a `closed` line under, in a version-2 config (§FS-init.2.3.5.12.2).
fn shaped_kinds(project: &Project) -> Vec<String> {
    if project.version != 2 {
        return Vec::new();
    }
    SchemaView::of(project)
        .nodes
        .into_iter()
        .filter(|node| !node.shape_lines().is_empty())
        .map(|node| node.name)
        .collect()
}

/// The `### Declaration shapes` section over the kinds `here` admits, in row
/// order, worded as `grund schema` words a field line with single spaces
/// (§FS-init.2.3.5.12.2); `None` when none of them has a shape.
pub(crate) fn declaration_shapes_section(
    project: &Project,
    here: &dyn Fn(&str) -> bool,
) -> Option<String> {
    if project.version != 2 {
        return None;
    }
    let mut bullets = String::new();
    for node in SchemaView::of(project).nodes {
        let lines = node.shape_lines();
        if lines.is_empty() || !here(&node.name) {
            continue;
        }
        bullets.push_str(&format!("- **{}**\n", node.name));
        for line in lines {
            bullets.push_str(&format!("  - {line}\n"));
        }
    }
    (!bullets.is_empty()).then(|| {
        format!(
            "### Declaration shapes\n\n`grund check` holds each declaration of these kinds to these fields, as `grund schema` lists them:\n\n{bullets}"
        )
    })
}
