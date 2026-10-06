//! Point-size catalog options and records (§FS-list.3.4).
use crate::config::PointSizeUnit;
use crate::model::Finding;
use crate::scanner::ApiScanError;
use std::collections::BTreeSet;
use std::path::PathBuf;

/// Options for the additive per-point size catalog (§FS-list.1, §FS-list.3.4).
#[derive(Clone)]
pub struct ListSizeOpts {
    pub path: PathBuf,
    pub path_provided: bool,
    pub kind_filter: BTreeSet<String>,
    pub project_filter: BTreeSet<String>,
    pub unused_only: bool,
    /// Optional declaration/chapter selector (§FS-rules.8).
    pub selector: Option<String>,
    pub units: Vec<PointSizeUnit>,
    pub top: Option<usize>,
}

impl Default for ListSizeOpts {
    fn default() -> Self {
        Self {
            path: PathBuf::from("."),
            path_provided: false,
            kind_filter: BTreeSet::new(),
            project_filter: BTreeSet::new(),
            unused_only: false,
            selector: None,
            units: vec![
                PointSizeUnit::Lines,
                PointSizeUnit::Words,
                PointSizeUnit::Bytes,
            ],
            top: None,
        }
    }
}

/// One selected unit's lead/full pair, kept in caller order (§FS-list.3.4).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListSizeMeasurement {
    pub unit: PointSizeUnit,
    pub lead: Option<usize>,
    pub full: Option<usize>,
}

/// One declaration or section site in the size catalog (§FS-list.2,
/// §FS-list.3.4). Unlike [`ListEntry`](crate::ListEntry), it deliberately carries no title/ref
/// fields and never collapses ambiguous sites.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListSizeEntry {
    pub project: Option<String>,
    pub id: String,
    pub section: Option<String>,
    /// The owning project's configured separator, used to render the text
    /// coordinate without adding a wire field (§FS-list.3.4.3).
    pub section_separator: String,
    pub kind: String,
    pub path: String,
    pub line: usize,
    pub stub: bool,
    pub defines: Option<String>,
    pub duplicate: bool,
    pub measurements: Vec<ListSizeMeasurement>,
}

/// Structured result for one deterministic size-catalog scan (§FS-list.3.4).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ListSizeOutput {
    pub output_format: String,
    pub workspace: bool,
    pub entries: Vec<ListSizeEntry>,
    pub scan_errors: Vec<ApiScanError>,
    /// The run's warning channel (§FS-distribution.3.1): the three `[workspace]`
    /// cautions of §FS-check.3.29.15, §FS-check.4.10.11 and
    /// §FS-workspace.6.1.7. A frontend renders each as one CLI-level `warning:`
    /// on stderr (§FS-check.2.1.1).
    ///
    /// Three keep the anchor the engine gave them — the `grund.toml` line their
    /// own message already names — and an editor publishes those on that line
    /// (§FS-lsp.1.1.3). §FS-check.3.29.15's is the exception: it carries no
    /// location field at all, states its location inside its own text
    /// (§FS-check.3.29.7), and reaches an editor off `check`'s report instead,
    /// located and as an error (§FS-check.3.29.13).
    pub warnings: Vec<Finding>,
}
