//! §AR-config.5: the `Config` façade, built from the three records and written
//! nowhere outside this component. `from_records` is the one place a façade
//! field gets its value. What the invocation decided is settled before its `Run`
//! is built (§AR-config.1.5), so the setters left here are what workspace
//! expansion learns about the checkout, and the ones a unit test states a scope
//! with. Each records the fact on the `Run` first and then refreshes the field it
//! shows, so the façade a component holds and the records behind it never
//! disagree.
//!
//! A setter refreshes its own field rather than rebuilding the whole façade:
//! tests still set façade fields directly, and a whole rebuild would undo them.

use std::path::{Path, PathBuf};
use std::sync::Arc;

#[cfg(test)]
use anyhow::Result;

use super::compiled::Compiled;
#[cfg(test)]
use super::compiled::compile;
use super::file_reading::FileReading;
use super::frame::{Display, Frame};
use super::kinds::KindTable;
#[cfg(test)]
use super::mount_edge::MountEdge;
use super::project::{Project, Rules, Schema};
use super::record::{AbsentOptionalNamespace, Config};
use super::run::Run;
use super::run_warnings::RunWarning;
use crate::sources::Buffers;

/// The records a façade was built from (§AR-config.1). Private, so no
/// component reads past the façade into them except through the accessors.
#[derive(Clone)]
pub(super) struct Records {
    pub(super) project: Arc<Project>,
    pub(super) run: Run,
    pub(super) compiled: Arc<Compiled>,
}

impl Config {
    /// §AR-config.5: the façade over `project`, `run` and `compiled`. A
    /// command-line override is a `Run` fact applied over the project's value:
    /// `--require-grounding` turns the `[reference]` default on, never off
    /// (§FS-check.3.6).
    pub(crate) fn from_records(project: &Project, run: &Run, compiled: &Compiled) -> Self {
        let schema = &project.schema;
        let notes = &schema.notes;
        let sources = &schema.sources;
        let ids = &schema.ids;
        let presentation = &project.presentation;
        let members = &project.workspace;
        Self {
            root: run.root.clone(),
            cli_base: run.cli_base.clone(),
            config_file: run.config_file.clone(),
            redundant_config_file: run.redundant_config_file.clone(),
            project_name: project.name.clone(),
            project_name_source: project.name_source.clone(),
            project_description: presentation.description.clone(),
            marker: schema.citation.marker.clone(),
            trigger: presentation.trigger.clone(),
            strict: schema.citation.strict,
            shorthand: schema.citation.shorthand,
            require_grounding: project.rules.grounding.require || run.scope.require_grounding,
            grounding_level: project.rules.grounding.level,
            grounding_units: !compiled.demand.is_empty(),
            conversation: presentation.conversation.clone(),
            lead_size_warning: schema.leads.clone(),
            inline_style: notes.inline_style.clone(),
            inline_note_suggested_lines: notes.suggested_lines,
            inline_note_max_lines: notes.max_lines,
            inline_note_max_columns: notes.max_columns,
            inline_note_layout: notes.layout.clone(),
            inline_note_layout_check: notes.layout_check.clone(),
            warn_on_suggested: notes.warn_on_suggested,
            include: sources.include.clone(),
            scan_full: run.scope.full,
            scan_resolution_wide: run.scope.resolution_wide,
            exclude: sources.exclude.clone(),
            extensions: sources.extensions.clone(),
            comment_prefixes: sources.comment_prefixes.clone(),
            docstring_python: sources.docstring_python,
            respect_gitignore: sources.respect_gitignore,
            output_format: presentation.output.format.clone(),
            relative_paths: presentation.output.relative_paths,
            id_format: ids.format.clone(),
            section_separator: ids.section_separator.clone(),
            number_pattern: ids.number_pattern.clone(),
            slug_pattern: ids.slug_pattern.clone(),
            named_sections: ids.named_sections,
            section_heading_levels: ids.section_heading_levels.clone(),
            kinds: project.kind_configs(),
            fmt_exclude: presentation.fmt.exclude.clone(),
            fmt_cross_refs_enabled: presentation.fmt.cross_refs_enabled,
            cross_ref_anchor_format: presentation.fmt.anchor_format.clone(),
            workspace_declared: members.declared,
            workspace_members: members.members.clone(),
            workspace_members_source: members.members_source.clone(),
            workspace_optional_members: members.optional_members.clone(),
            workspace_optional_members_source: members.optional_members_source.clone(),
            workspace_absent_optional: run.workspace.absent_optional.clone(),
            workspace_section_source: members.section_source.clone(),
            workspace_include_root: members.include_root,
            workspace_include_root_source: members.include_root_source.clone(),
            workspace_boundary_roots: run.workspace.boundary_roots.clone(),
            run_warnings: run.warnings.clone(),
            workspace_project_roots: run.workspace.project_roots.clone(),
            workspace_scope_path: run.workspace.scope_path.clone(),
            citations: project.rules.citations.clone(),
            classify_citation_sources: run.scope.classify_citation_sources,
            owner_lines: run.scope.owner_lines.clone(),
            path_base: run.path_base,
            grammar: compiled.grammar.clone(),
            records: Records {
                project: Arc::new(project.clone()),
                run: run.clone(),
                compiled: Arc::new(compiled.clone()),
            },
        }
    }

    /// The project this façade shows (§AR-config.1.1).
    pub(crate) fn project(&self) -> &Project {
        &self.records().project
    }

    /// The invocation's facts this façade shows (§AR-config.1.5).
    pub(crate) fn run(&self) -> &Run {
        &self.records().run
    }

    /// What was derived from the project once (§AR-config.1.5).
    pub(crate) fn compiled(&self) -> &Compiled {
        &self.records().compiled
    }

    /// How `path` is read under this project (§FS-config-v2.schema.sources): what a
    /// writer or a query asks of a file, so it reads the file as the scan does.
    pub(crate) fn reading<'a>(&'a self, path: &'a Path) -> FileReading<'a> {
        self.compiled().reading(self.schema(), path)
    }

    /// The schema a stage below the writers is handed (§AR-config.1.2,
    /// §AR-checker.1).
    pub(crate) fn schema(&self) -> &Schema {
        &self.project().schema
    }

    /// The rules the checker's relational half is handed (§AR-checker.1.2).
    pub(crate) fn rules(&self) -> &Rules {
        &self.project().rules
    }

    /// The frame a stage runs in, its report spelled from this project
    /// (§AR-checker.1, §FS-config.3.6).
    pub(crate) fn frame(&self) -> Frame<'_> {
        let records = self.records();
        Frame {
            run: &records.run,
            compiled: &records.compiled,
            name: records.project.name.as_deref(),
            alias: None,
            version: records.project.version,
            display: self.display(),
        }
    }

    /// How this run's report spells a path from this project: the run's
    /// `--path-base` over `[output] relative_paths` (§FS-cli.3.4), settled here so
    /// a stage reads no presentation key (§AR-config.5).
    pub(crate) fn display(&self) -> Display<'_> {
        Display {
            root: &self.root,
            cli_base: &self.cli_base,
            from_root: self.reports_from_root(),
            mounted: &self.records.run.workspace.mounted_roots,
        }
    }

    /// The records behind the façade. A test may still set a façade field
    /// directly, so under test they are read back from the fields
    /// (`facade_sync.rs`); nothing else writes a field outside a setter.
    #[cfg(not(test))]
    fn records(&self) -> &Records {
        &self.records
    }

    /// `grund check --full`, stated on a test's façade; a command hands it to
    /// `open()` (§AR-config.1.5, §FS-check.1.3).
    #[cfg(test)]
    pub(crate) fn set_scan_full(&mut self, full: bool) {
        self.records.run.scope.full = full;
        self.scan_full = full;
    }

    /// `cover --lines` ranges, stated on a test's façade (§FS-cover.6.1).
    #[cfg(test)]
    pub(crate) fn set_owner_lines(&mut self, lines: Vec<(usize, usize)>) {
        self.records.run.scope.owner_lines = lines;
        self.owner_lines = self.records.run.scope.owner_lines.clone();
    }

    /// `--require-grounding`, stated on a test's façade (§FS-check.3.6).
    #[cfg(test)]
    pub(crate) fn force_require_grounding(&mut self) {
        self.records.run.scope.require_grounding = true;
        self.require_grounding = true;
    }

    /// What lies below this project in the workspace (§AR-workspace.6).
    pub(crate) fn set_workspace_boundary_roots(&mut self, roots: Vec<PathBuf>) {
        self.records.run.workspace.boundary_roots = roots;
        self.workspace_boundary_roots = self.records.run.workspace.boundary_roots.clone();
    }

    /// Where every project this run loaded lives (§AR-workspace.6).
    pub(crate) fn set_workspace_project_roots(&mut self, roots: Vec<PathBuf>) {
        self.records.run.workspace.project_roots = roots;
        self.workspace_project_roots = self.records.run.workspace.project_roots.clone();
    }

    /// The alias path of the run's own workspace root, stamped onto every
    /// project it loaded (§FS-workspace.6.1.5).
    pub(crate) fn set_workspace_scope_path(&mut self, scope_path: String) {
        self.records.run.workspace.scope_path = scope_path;
        self.workspace_scope_path = self.records.run.workspace.scope_path.clone();
    }

    /// The optional members this run found absent (§FS-check.4.9).
    pub(crate) fn set_workspace_absent_optional(&mut self, absent: Vec<AbsentOptionalNamespace>) {
        self.records.run.workspace.absent_optional = absent;
        self.workspace_absent_optional = self.records.run.workspace.absent_optional.clone();
    }

    /// Whether any block expansion walked lists an optional member
    /// (§FS-check.1.4.1). Not a façade field, like the kind table below.
    pub(crate) fn set_workspace_lists_optional(&mut self, lists: bool) {
        self.records.run.workspace.lists_optional = lists;
    }

    /// Mark this project as loaded under the mounted root at `root`
    /// (§FS-remote-projects.mount.rule): its walk reads ignore files at and below
    /// it only (§FS-remote-projects.ignore), and no claim above it is climbed.
    pub(crate) fn set_mounted_root(&mut self, root: PathBuf) {
        self.records.run.workspace.mounted_root = Some(root);
    }

    /// The mounted root this project was loaded under, if any.
    pub(crate) fn mounted_root(&self) -> Option<&Path> {
        self.records.run.workspace.mounted_root.as_deref()
    }

    /// The canonical roots of the mounts the run loaded, whose files are reported
    /// relative to the report base (§FS-imports.paths).
    pub(crate) fn set_mounted_roots(&mut self, roots: Vec<PathBuf>) {
        self.records.run.workspace.mounted_roots = roots;
    }

    /// The mount edges this run's blocks declare (§AR-resolver.mount). No config key
    /// declares one yet, so only the mount layer's own proof sets them
    /// (§REQ-remote-equivalence.4).
    #[cfg(test)]
    pub(crate) fn set_mount_edges(&mut self, mounts: Vec<MountEdge>) {
        self.records.run.workspace.mounts = mounts;
    }

    /// What [`Self::set_workspace_lists_optional`] recorded; `false` in a run
    /// that expanded no workspace.
    pub(crate) fn workspace_lists_optional(&self) -> bool {
        self.records.run.workspace.lists_optional
    }

    /// The run's kind table, which expansion built (§AR-config.1.5,
    /// §AR-config.1.5). Not a façade field: nothing but the setter
    /// writes it, so the records hold it alone.
    pub(crate) fn set_kind_table(&mut self, table: Arc<KindTable>) {
        self.records.run.workspace.kinds = Some(table);
    }

    /// The kind table every kind-name question of this run is answered from
    /// (§AR-config.1.5): the one expansion built, or this project's
    /// own where the run expanded no workspace.
    pub(crate) fn kind_table(&self) -> Arc<KindTable> {
        match &self.records.run.workspace.kinds {
            Some(table) => Arc::clone(table),
            None => Arc::new(KindTable::standalone(
                self.schema(),
                self.records().project.version,
            )),
        }
    }

    /// Adds to the run's warning channel (§FS-distribution.3.1).
    pub(crate) fn extend_run_warnings(&mut self, warnings: impl IntoIterator<Item = RunWarning>) {
        self.records.run.warnings.extend(warnings);
        self.run_warnings = self.records.run.warnings.clone();
    }

    /// Empties the run's warning channel on a project that is not the run's:
    /// the run's cautions are announced once, by the config that owns them
    /// (§FS-distribution.3.1).
    pub(crate) fn clear_run_warnings(&mut self) {
        self.records.run.warnings.clear();
        self.run_warnings.clear();
    }

    /// This project as one standalone namespace: the `[workspace]` member lists
    /// gone, so a rule that reads the project alone sees no members
    /// (§FS-config.3.9).
    pub(crate) fn without_members(&self) -> Self {
        let mut project = self.project().clone();
        project.workspace.members.clear();
        project.workspace.optional_members.clear();
        self.with_project(project)
    }

    /// This façade under a run whose readers see `buffers` in front of the disk
    /// (§AR-system.2.13): how `fmt`'s lock pass reads the tree as its walk
    /// leaves it, before a byte of it is written (§FS-section-lock.write.same-run).
    pub(crate) fn with_buffers(&self, buffers: Buffers) -> Self {
        let mut run = self.run().clone();
        run.buffers = buffers;
        Self::from_records(self.project(), &run, self.compiled())
    }

    /// The façade over `project` with this run and this compiled grammar, which
    /// an edit that touches neither the ID grammar nor the scan demand keeps.
    fn with_project(&self, project: Project) -> Self {
        Self::from_records(&project, self.run(), self.compiled())
    }

    /// Edits the project and rebuilds the whole façade from it, recompiling —
    /// how a test states a configuration without writing a file, and keeps the
    /// records and the façade in agreement while doing it.
    #[cfg(test)]
    pub(crate) fn edit_project(&mut self, edit: impl FnOnce(&mut Project)) -> Result<()> {
        let mut project = self.project().clone();
        edit(&mut project);
        let compiled = compile(&project)?;
        *self = Self::from_records(&project, self.run(), &compiled);
        Ok(())
    }

    /// The façade a loaded project starts from: `project` compiled, under a run
    /// that has decided nothing but its root (§AR-config.5).
    #[cfg(test)]
    pub(super) fn from_project(project: &Project, root: PathBuf) -> Result<Self> {
        let compiled = compile(project)?;
        Ok(Self::from_records(project, &Run::at(root), &compiled))
    }
}
