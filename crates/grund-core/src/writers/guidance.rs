//! The managed blocks one run renders under scoped guidance (§FS-init.2.3.5.11):
//! the root block's config-derived sections and every destination's scoped
//! block, from one placement. `grund init` writes these bytes and `Expected`
//! carries them to the drift check, so the two ask one component for one answer
//! (§AR-checker.2.7).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::expected::{block_version, chapter_rules_section};
use crate::config::Config;
use crate::model::{Catalog, ExpectedEntrypoint, ExpectedSection, ScopedExpectation};
use crate::sources::path_is_dir;
use crate::templates::{
    ConversationSurface, Layout, citation_directions_section, clickable_citations_section,
    declaration_shapes_section,
};

/// The validated chapter rules a run renders (§FS-init.2.3.5.10): the catalog
/// their citations resolve against, the `(rule ID, sentence)` rows in order,
/// and each rule's subject kind.
#[derive(Clone, Copy)]
pub(crate) struct RuleGuidance<'a> {
    pub(crate) findings: &'a Catalog,
    pub(crate) rows: &'a [(String, String)],
    pub(crate) subjects: &'a BTreeMap<String, String>,
}

/// One run's placement and block version, and the renders that follow from them.
pub(crate) struct Guidance<'a> {
    config: &'a Config,
    pub(crate) layout: Layout,
    /// The version every block of the run carries (§FS-init.2.3.7.1).
    pub(crate) version: u32,
    rules: Option<RuleGuidance<'a>>,
}

impl<'a> Guidance<'a> {
    /// Place every row against the directories that exist under the config root
    /// now (§FS-init.2.3.5.11.4).
    pub(crate) fn new(config: &'a Config, rules: Option<RuleGuidance<'a>>) -> Self {
        let subjects: Vec<String> = rules
            .map(|rules| rules.subjects.values().cloned().collect())
            .unwrap_or_default();
        let root = &config.root;
        let layout = Layout::of(config.project(), config.run(), &subjects, &|dir| {
            path_is_dir(root.join(dir))
        });
        // §FS-init.2.3.7.1: a scoped file or a declaration shape adds two.
        let shapes = declaration_shapes_section(config.project(), &|_| true).is_some();
        let version = block_version(config) + if layout.is_scoped() || shapes { 2 } else { 0 };
        Self {
            config,
            layout,
            version,
            rules,
        }
    }

    /// `dir`'s `### Chapter rules`, its links relative to `path`
    /// (§FS-init.2.3.5.10): the rules whose subject kind lands there. The root
    /// keeps the section, bullets or not, unless every rule landed elsewhere.
    fn chapter_rules(&self, path: &Path, dir: &str) -> Option<String> {
        let rules = self.rules?;
        let lands = |origin: &String| {
            rules
                .subjects
                .get(origin)
                .map_or(dir.is_empty(), |kind| self.layout.lands(kind, dir))
        };
        let rows: Vec<(String, String)> = rules
            .rows
            .iter()
            .filter(|(origin, _)| lands(origin))
            .cloned()
            .collect();
        let renders = if dir.is_empty() {
            !rows.is_empty() || rules.rows.is_empty()
        } else {
            !rows.is_empty()
        };
        renders.then(|| chapter_rules_section(self.config, path, rules.findings, &rows))
    }

    /// The root entrypoint's config-derived sections, rendered for `path`'s
    /// surface (§FS-init.2.3.6.1); a section with no content is expected absent.
    pub(crate) fn root_sections(&self, path: &Path) -> Vec<ExpectedSection> {
        let project = self.config.project();
        let mut sections = vec![
            ExpectedSection {
                heading: "### Citation directions",
                noun: "citation directions",
                bytes: self
                    .layout
                    .root_citation_directions(project, self.config.run()),
            },
            ExpectedSection {
                heading: "### Clickable citations",
                noun: "clickable citations",
                bytes: clickable_citations_section(
                    project,
                    ConversationSurface::for_entrypoint(path),
                ),
            },
        ];
        if self.rules.is_some() {
            sections.push(ExpectedSection {
                heading: "### Chapter rules",
                noun: "chapter rules",
                bytes: self.chapter_rules(path, "").unwrap_or_default(),
            });
        }
        sections.extend([
            ExpectedSection {
                heading: "### Declaration shapes",
                noun: "declaration shapes",
                bytes: self
                    .layout
                    .declaration_shapes(project, "")
                    .unwrap_or_default(),
            },
            ExpectedSection {
                heading: "### Scoped guidance",
                noun: "scoped guidance",
                bytes: self.layout.pointer().unwrap_or_default(),
            },
        ]);
        sections
    }

    /// A root entrypoint as the drift check holds it (§FS-check.3.5).
    pub(crate) fn root_entrypoint(&self, path: PathBuf, require_block: bool) -> ExpectedEntrypoint {
        ExpectedEntrypoint {
            sections: self.root_sections(&path),
            path,
            require_block,
            scoped: None,
        }
    }

    /// Finish a root block rendered from the template: the version this run
    /// carries, the citation directions of the rows that stay at the root, and
    /// the declaration shapes, chapter rules and pointer before `### Clickable
    /// citations` (§FS-init.2.3.5.12.2, §FS-init.2.3.5.13). `path` is the
    /// entrypoint under the config root the chapter-rule links are relative to.
    pub(crate) fn finish_root_block(&self, mut block: String, path: &Path) -> String {
        let (project, run) = (self.config.project(), self.config.run());
        block = block.replacen(
            "Grounding with grund (v27)",
            &format!("Grounding with grund (v{})", self.version),
            1,
        );
        if self.layout.is_scoped() {
            block = block.replacen(
                &citation_directions_section(project, run),
                &self.layout.root_citation_directions(project, run),
                1,
            );
        }
        let inserted: String = [
            self.layout.declaration_shapes(project, ""),
            self.chapter_rules(path, ""),
            self.layout.pointer(),
        ]
        .into_iter()
        .flatten()
        .map(|section| format!("\n{section}"))
        .collect();
        let at = block
            .find("\n### Clickable citations")
            .or_else(|| block.find("\n<!-- END GRUND MANAGED BLOCK -->"));
        match at {
            Some(index) => block.insert_str(index, &inserted),
            None => block.push_str(&inserted),
        }
        block
    }

    /// The managed block of `dir`'s scoped entrypoint at `path`
    /// (§FS-init.2.3.5.12.1).
    pub(crate) fn scoped_block(&self, dir: &str, path: &Path) -> String {
        let chapter_rules = self.chapter_rules(path, dir);
        self.layout.scoped_block(
            self.config.project(),
            self.config.run(),
            dir,
            self.version,
            chapter_rules.as_deref(),
        )
    }

    /// `dir`'s scoped entrypoint at `path` as the drift check holds it: each
    /// section, then the block byte for byte (§FS-init.2.3.5.8).
    pub(crate) fn scoped_entrypoint(
        &self,
        dir: &str,
        path: PathBuf,
        require_block: bool,
    ) -> ExpectedEntrypoint {
        let chapter_rules = self.chapter_rules(&path, dir);
        let nouns = [
            "citation directions",
            "declaration shapes",
            "chapter rules",
            "scoped guidance",
        ];
        let sections = self
            .layout
            .scoped_sections(
                self.config.project(),
                self.config.run(),
                dir,
                chapter_rules.as_deref(),
            )
            .into_iter()
            .zip(nouns)
            .map(|((heading, bytes), noun)| ExpectedSection {
                heading,
                noun,
                bytes: bytes.unwrap_or_default(),
            })
            .collect();
        ExpectedEntrypoint {
            scoped: Some(ScopedExpectation::Block(self.scoped_block(dir, &path))),
            path,
            require_block,
            sections,
        }
    }
}
