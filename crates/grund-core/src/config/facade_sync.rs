//! The records a test's façade shows (§AR-config.5). A test may still set a
//! façade field directly, and the stages below the writers read the records
//! rather than the façade (§AR-checker.1), so under test the records are read
//! back from the fields: a façade that agrees with its records hands them out
//! as they are, and one a test edited hands out records rebuilt from the edit.
//! The rebuilt records live as long as the test process, which is the price of
//! handing out a borrow of something built on demand, and it is paid only by a
//! test that edited a field.

use std::sync::Arc;

use super::compiled::{Compiled, scan_demand};
use super::facade::Records;
use super::kind::KindConfig;
use super::project::{KindGrounding, Project};
use super::record::Config;
use super::rows::{Extent, Form, Kind, Origin, Place, Row};
use super::run::Run;

impl Config {
    /// The records this façade shows, rebuilt from the fields when a test
    /// edited one (§AR-config.5).
    pub(super) fn records(&self) -> &Records {
        let project = self.project_from_fields();
        if project == *self.records.project && self.run_agrees_with_fields() {
            return &self.records;
        }
        let mut run = self.records.run.clone();
        self.write_run_fields(&mut run);
        let compiled = Compiled {
            grammar: self.grammar.clone(),
            demand: scan_demand(&project),
        };
        Box::leak(Box::new(Records {
            project: Arc::new(project),
            run,
            compiled: Arc::new(compiled),
        }))
    }

    /// The project the fields describe: the records' own, every field the
    /// façade shows written back over it.
    fn project_from_fields(&self) -> Project {
        let mut project = (*self.records.project).clone();
        project.name = self.project_name.clone();
        project.name_source = self.project_name_source.clone();
        let schema = &mut project.schema;
        schema.citation.marker = self.marker.clone();
        schema.citation.strict = self.strict;
        schema.citation.shorthand = self.shorthand;
        schema.ids.format = self.id_format.clone();
        schema.ids.section_separator = self.section_separator.clone();
        schema.ids.number_pattern = self.number_pattern.clone();
        schema.ids.slug_pattern = self.slug_pattern.clone();
        schema.ids.named_sections = self.named_sections;
        schema.ids.section_heading_levels = self.section_heading_levels.clone();
        schema.sources.include = self.include.clone();
        schema.sources.exclude = self.exclude.clone();
        schema.sources.extensions = self.extensions.clone();
        schema.sources.comment_prefixes = self.comment_prefixes.clone();
        schema.sources.docstring_python = self.docstring_python;
        schema.sources.respect_gitignore = self.respect_gitignore;
        schema.notes.inline_style = self.inline_style.clone();
        schema.notes.suggested_lines = self.inline_note_suggested_lines;
        schema.notes.max_lines = self.inline_note_max_lines;
        schema.notes.max_columns = self.inline_note_max_columns;
        schema.notes.layout = self.inline_note_layout.clone();
        schema.notes.layout_check = self.inline_note_layout_check.clone();
        schema.notes.warn_on_suggested = self.warn_on_suggested;
        schema.leads = self.lead_size_warning.clone();
        let rules = &mut project.rules;
        rules.citations = self.citations.clone();
        // The façade shows `require || --require-grounding`; only an edit that
        // disagrees with that is written back (§FS-check.3.6).
        let shown = rules.grounding.require || self.records.run.scope.require_grounding;
        if self.require_grounding != shown {
            rules.grounding.require = self.require_grounding;
        }
        rules.grounding.level = self.grounding_level;
        let presentation = &mut project.presentation;
        presentation.description = self.project_description.clone();
        presentation.trigger = self.trigger.clone();
        presentation.conversation = self.conversation.clone();
        presentation.output.format = self.output_format.clone();
        presentation.output.relative_paths = self.relative_paths;
        presentation.fmt.exclude = self.fmt_exclude.clone();
        presentation.fmt.cross_refs_enabled = self.fmt_cross_refs_enabled;
        presentation.fmt.anchor_format = self.cross_ref_anchor_format.clone();
        let members = &mut project.workspace;
        members.declared = self.workspace_declared;
        members.members = self.workspace_members.clone();
        members.members_source = self.workspace_members_source.clone();
        members.optional_members = self.workspace_optional_members.clone();
        members.optional_members_source = self.workspace_optional_members_source.clone();
        members.section_source = self.workspace_section_source.clone();
        members.include_root = self.workspace_include_root;
        members.include_root_source = self.workspace_include_root_source.clone();
        if self.kinds != project.kind_configs() {
            write_kinds(&mut project, &self.kinds);
        }
        project
    }

    /// Whether the run facts the façade shows are the records' own.
    fn run_agrees_with_fields(&self) -> bool {
        let mut run = self.records.run.clone();
        self.write_run_fields(&mut run);
        let (ours, theirs) = (&run, &self.records.run);
        ours.root == theirs.root
            && ours.cli_base == theirs.cli_base
            && ours.config_file == theirs.config_file
            && ours.redundant_config_file == theirs.redundant_config_file
            && ours.path_base == theirs.path_base
            && ours.scope.full == theirs.scope.full
            && ours.scope.resolution_wide == theirs.scope.resolution_wide
            && ours.scope.classify_citation_sources == theirs.scope.classify_citation_sources
            && ours.scope.owner_lines == theirs.scope.owner_lines
            && ours.workspace.boundary_roots == theirs.workspace.boundary_roots
            && ours.workspace.project_roots == theirs.workspace.project_roots
            && ours.workspace.scope_path == theirs.workspace.scope_path
            && ours.workspace.absent_optional == theirs.workspace.absent_optional
    }

    /// The run facts the façade shows, written over `run`. The warning channel
    /// is the setters' alone, so it is the records'.
    fn write_run_fields(&self, run: &mut Run) {
        run.root = self.root.clone();
        run.cli_base = self.cli_base.clone();
        run.config_file = self.config_file.clone();
        run.redundant_config_file = self.redundant_config_file.clone();
        run.path_base = self.path_base;
        run.scope.full = self.scan_full;
        run.scope.resolution_wide = self.scan_resolution_wide;
        run.scope.classify_citation_sources = self.classify_citation_sources;
        run.scope.owner_lines = self.owner_lines.clone();
        run.workspace.boundary_roots = self.workspace_boundary_roots.clone();
        run.workspace.project_roots = self.workspace_project_roots.clone();
        run.workspace.scope_path = self.workspace_scope_path.clone();
        run.workspace.absent_optional = self.workspace_absent_optional.clone();
    }
}

/// The v1 rows written back as rows and the per-kind facts the other two
/// concerns hold — `kind_config`'s inverse (§AR-config.3.1).
fn write_kinds(project: &mut Project, kinds: &[KindConfig]) {
    project.schema.rows = kinds.iter().map(row_of).collect();
    for kind in kinds {
        let name = &kind.kind;
        let grounding = project.rules.grounding.kinds.entry(name.clone());
        let grounding = grounding.or_insert_with(KindGrounding::default);
        grounding.require = kind.require_grounding;
        grounding.level = kind.grounding_level;
        match kind.resolve {
            Some(resolve) => project.rules.resolution.insert(name.clone(), resolve),
            None => project.rules.resolution.remove(name),
        };
        match &kind.title {
            Some(title) => project
                .presentation
                .kinds
                .insert(name.clone(), title.clone()),
            None => project.presentation.kinds.remove(name),
        };
    }
}

/// One v1 row as a schema row: its home, or the complement for the declared
/// homeless kind (§FS-config.3.9.2.1), and its kind when it is citable.
fn row_of(kind: &KindConfig) -> Row {
    let extent = match (&kind.folder, &kind.file) {
        (Some(folder), _) => Some(Extent::Folder(folder.clone())),
        (None, Some(file)) => Some(Extent::File(file.clone())),
        (None, None) if !kind.citable => Some(Extent::Complement),
        (None, None) => None,
    };
    let form = match (&kind.value_chapter, kind.values, kind.rules) {
        (value_chapter, _, true) => Form::Rule {
            value_chapter: value_chapter.clone(),
        },
        (Some(chapter), _, false) => Form::Value {
            chapter: Some(chapter.clone()),
        },
        (None, true, false) => Form::Value { chapter: None },
        (None, false, false) => Form::Prose,
    };
    Row {
        name: kind.kind.clone(),
        places: extent
            .into_iter()
            .map(|extent| Place {
                extent,
                scanned: kind.scan,
            })
            .collect(),
        kind: kind.citable.then(|| Kind {
            id_format: kind.format.clone(),
            form,
            index: kind.index.clone(),
            origin: match &kind.fetch {
                Some(fetch) => Origin::External {
                    fetch: fetch.clone(),
                },
                None => Origin::Local,
            },
        }),
    }
}
