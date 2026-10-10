//! A loaded project as the resolver holds it (§AR-config.5, §AR-resolver.3):
//! the records a stage below the writers reads — the schema, the rules, the
//! presentation, and the frame the run gives them — over the one façade the
//! components still on §AR-config.5's list read through.
//!
//! The resolver loads every project a command sees and scans it, and the
//! queries, writers and api above it read what it loaded. It reads the records
//! here and names no façade; the façade rides along, reached by `Deref`, until
//! the components above it leave the list too.

use std::ops::Deref;

use super::compiled::Compiled;
use super::frame::Frame;
use super::project::{Project, Rules, Schema};
use super::record::Config;
use super::run::Run;

/// One loaded project's records, and the façade over them (§AR-config.5).
#[derive(Clone)]
pub(crate) struct ProjectRecords {
    facade: Config,
}

impl ProjectRecords {
    /// The records `config` shows — under test, what a case that edited a
    /// field set them to (§AR-config.5).
    pub(crate) fn of(config: Config) -> Self {
        Self { facade: config }
    }

    /// The project (§AR-config.1.1).
    pub(crate) fn project(&self) -> &Project {
        self.facade.project()
    }

    /// The schema (§AR-config.1.2).
    pub(crate) fn schema(&self) -> &Schema {
        self.facade.schema()
    }

    /// The rules (§AR-config.1.2).
    pub(crate) fn rules(&self) -> &Rules {
        self.facade.rules()
    }

    /// The invocation's facts (§AR-config.1.5).
    pub(crate) fn run(&self) -> &Run {
        self.facade.run()
    }

    /// What was compiled from the project once (§AR-config.1.5).
    pub(crate) fn compiled(&self) -> &Compiled {
        self.facade.compiled()
    }

    /// The frame a stage runs this project in (§AR-checker.1).
    pub(crate) fn frame(&self) -> Frame<'_> {
        self.facade.frame()
    }

    /// The façade, for a component still on §AR-config.5's list that needs it
    /// by value or mutably — the workspace pass expands a root from it.
    pub(crate) fn facade_mut(&mut self) -> &mut Config {
        &mut self.facade
    }

    /// The façade, handed to a component still on §AR-config.5's list.
    pub(crate) fn into_facade(self) -> Config {
        self.facade
    }

    /// Whether this run classifies citing sides (§AR-scanner.2.4).
    pub(crate) fn set_classify_citation_sources(&mut self, classify: bool) {
        self.facade.set_classify_citation_sources(classify);
    }

    /// `grund check --full` for this run (§FS-check.1.3).
    pub(crate) fn set_scan_full(&mut self, full: bool) {
        self.facade.set_scan_full(full);
    }

    /// `--require-grounding` for this run (§FS-check.3.5).
    pub(crate) fn force_require_grounding(&mut self) {
        self.facade.force_require_grounding();
    }

    /// The `cover --lines` ranges this run asks the owners of (§FS-cover.6.1).
    pub(crate) fn set_owner_lines(&mut self, lines: Vec<(usize, usize)>) {
        self.facade.set_owner_lines(lines);
    }
}

/// The components still on §AR-config.5's list read the façade straight
/// through a loaded project; the resolver reads the records above.
impl Deref for ProjectRecords {
    type Target = Config;

    fn deref(&self) -> &Config {
        &self.facade
    }
}
