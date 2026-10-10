//! §AR-config.5: the `Config` façade, built from the three records and written
//! nowhere outside this component. `from_records` is the one place a façade
//! field gets its value; every per-run change goes through a setter here that
//! records the fact on the `Run` first and then refreshes the field it shows,
//! so the façade a component holds and the records behind it never disagree
//! about what the invocation decided.
//!
//! A setter refreshes its own field rather than rebuilding the whole façade:
//! tests still set façade fields directly, and a whole rebuild would undo them.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;

use super::compiled::{Compiled, compile};
use super::frame::{Display, Frame};
use super::project::{Project, Rules, Schema};
use super::record::{AbsentOptionalNamespace, Config};
use super::run::Run;
use super::run_warnings::RunWarning;

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
        }
    }

    /// The records behind the façade. A test may still set a façade field
    /// directly, so under test they are read back from the fields
    /// (`facade_sync.rs`); nothing else writes a field outside a setter.
    #[cfg(not(test))]
    fn records(&self) -> &Records {
        &self.records
    }

    /// Re-roots the run at `root`, the canonical config root a workspace walk
    /// resolved (§AR-workspace.6).
    pub(crate) fn set_root(&mut self, root: PathBuf) {
        self.records.run.root = root;
        self.root = self.records.run.root.clone();
    }

    /// `grund check --full` for this run (§FS-check.1.3).
    pub(crate) fn set_scan_full(&mut self, full: bool) {
        self.records.run.scope.full = full;
        self.scan_full = full;
    }

    /// An explicit path below the config root (§FS-check.1.3.6.1).
    pub(crate) fn set_scan_resolution_wide(&mut self, wide: bool) {
        self.records.run.scope.resolution_wide = wide;
        self.scan_resolution_wide = wide;
    }

    /// Whether this run's scan classifies citing sides (§AR-scanner.2.4).
    pub(crate) fn set_classify_citation_sources(&mut self, classify: bool) {
        self.records.run.scope.classify_citation_sources = classify;
        self.classify_citation_sources = classify;
    }

    /// The `cover --lines` ranges this run asks the owners of (§FS-cover.6.1).
    pub(crate) fn set_owner_lines(&mut self, lines: Vec<(usize, usize)>) {
        self.records.run.scope.owner_lines = lines;
        self.owner_lines = self.records.run.scope.owner_lines.clone();
    }

    /// `grund check --require-grounding`: on for this run, never off
    /// (§FS-check.3.6).
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

    /// This project under another identity line — the name and description a
    /// scaffold renders (§FS-init.2.4).
    pub(crate) fn with_identity(&self, name: Option<String>, description: Option<String>) -> Self {
        let mut project = self.project().clone();
        project.name = name;
        project.presentation.description = description;
        self.with_project(project)
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
    pub(super) fn from_project(project: &Project, root: PathBuf) -> Result<Self> {
        let compiled = compile(project)?;
        Ok(Self::from_records(project, &Run::at(root), &compiled))
    }
}
