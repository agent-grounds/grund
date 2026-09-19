//! The editor's snapshot vocabulary (§AR-system.2.7): what one request asks for
//! and the declaration, section, stub and citation ranges the answer carries,
//! with the navigation target each resolves to (§FS-lsp.1.1, §FS-lsp.1.3,
//! §AR-lsp.5).
//!
//! The records are here and the walk that fills them is `api/lsp_snapshot.rs`,
//! because building one needs the whole pipeline and only the api sits above all
//! of it. They sat up there too while `editor_hover.rs` read the snapshot and its
//! citation record upward to answer the title hover of §FS-lsp.1.2
//! (§AR-system.4); what an editor is told about a title is this component's
//! answer, so its vocabulary belongs beside the two answers that read it. They
//! stay `pub` — `grund-lsp` marshals exactly these fields (§AR-lsp).

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use crate::model::Report;
use crate::scanner::ApiScanError;

/// One snapshot request (§AR-lsp.5): the anchor folder, whether the editor named
/// it explicitly (§FS-lsp.2.2), and the buffers it holds unsaved.
#[derive(Clone)]
pub struct LspSnapshotOpts {
    pub path: PathBuf,
    pub path_provided: bool,
    pub open_documents: BTreeMap<PathBuf, String>,
}

impl Default for LspSnapshotOpts {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            path_provided: false,
            open_documents: BTreeMap::new(),
        }
    }
}

/// Everything an editor is told about one tree at one moment (§FS-lsp.1,
/// §AR-lsp.5): the diagnostics `grund check` reports over it, and every
/// scanner-derived range with its resolved target.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LspSnapshot {
    pub root: PathBuf,
    pub marker: String,
    pub trigger: String,
    pub workspace: bool,
    pub report: Report,
    pub declarations: Vec<LspDeclaration>,
    /// Citable section headings (`<ID>.<section>`) inside declaration bodies,
    /// each a declaration-side title editors can navigate to its section
    /// citations (§FS-lsp.1.3.1). Kept separate from `declarations` so the
    /// whole-ID home set stays the bare-ID declarations.
    pub sections: Vec<LspDeclaration>,
    /// Exact title spans for located findings that must not become editor
    /// navigation targets (§FS-lsp.1.1, §FS-check.3.23).
    pub finding_ranges: Vec<LspFindingRange>,
    pub stubs: Vec<LspStub>,
    pub citations: Vec<LspCitation>,
    /// Every file this snapshot's scan read, absolutized the same way token
    /// paths are. A project's scan is not bounded by its root — a symlinked
    /// `[scan] include` resolves outside it, and an include path may be
    /// parent-relative — so a root prefix cannot answer "does this project
    /// cover this document?" on its own, and an LSP that asked only that would
    /// go silent on files the CLI checks (§FS-lsp.2.2, §AR-lsp.2).
    pub scanned_files: BTreeSet<PathBuf>,
    pub scan_errors: Vec<ApiScanError>,
}

/// One declaration-side title an editor can navigate from (§FS-lsp.1.3).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LspDeclaration {
    pub project: Option<String>,
    pub path: PathBuf,
    pub display_path: String,
    pub line: usize,
    pub column: usize,
    pub text: String,
    pub query_id: String,
    pub section_separator: String,
}

/// The exact span of a title a finding is anchored on (§FS-lsp.1.1).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LspFindingRange {
    pub code: &'static str,
    pub path: PathBuf,
    pub line: usize,
    pub column: usize,
    pub text: String,
}

/// LSP token for an inline-spec stub title whose definition follows to the
/// source doc-comment declaration. §FS-lsp.1.3
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LspStub {
    pub project: Option<String>,
    pub path: PathBuf,
    pub display_path: String,
    pub line: usize,
    pub column: usize,
    pub text: String,
    pub query_id: String,
    pub section_separator: String,
    pub target_path: PathBuf,
    pub target_line: usize,
}

/// One citation site with the declaration it resolves to (§FS-lsp.1.1).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LspCitation {
    pub project: Option<String>,
    pub path: PathBuf,
    pub display_path: String,
    pub line: usize,
    pub column: usize,
    pub text: String,
    pub query_id: String,
    pub declaration_query_id: String,
    pub section_separator: String,
    pub target_path: Option<PathBuf>,
    pub target_line: Option<usize>,
}

/// Additive snapshot metadata for §FS-config.3.4.3 and §FS-lsp.1.2, keyed by
/// the snapshot's resolved query IDs (including section IDs). Keeping this
/// outside `LspSnapshot` preserves existing public struct construction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LspSnapshotWithMetadata {
    pub snapshot: LspSnapshot,
    pub kind_titles: BTreeMap<String, String>,
}
