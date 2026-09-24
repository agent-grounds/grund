//! The built-in kind table (§FS-config.3.4): the canonical kind set a
//! zero-config tree gets (§FS-config.3, §GOAL-zero-config) and one function per
//! key whose default is keyed on the kind's name.
//!
//! Separate from `kind_table.rs` because the two answer different questions
//! (§AR-core-module-layout.1): this file is what a repository with no
//! `[[kinds]]` block is governed by, while that one reads and validates a block
//! a repository declared. They meet where a declared row picks up a per-name
//! default the key left unset.

use super::kind::KindIndex;

/// The canonical kind set (§FS-config.3.4). `e2e` and `integration` are
/// *non-citable*: a test proves a claim someone else wrote, so it cites and is
/// never cited, and lowercase names them as places rather than ID prefixes.
pub(super) const DEFAULT_KINDS: &[&str] = &[
    "GRUND",
    "GOAL",
    "FS",
    "AR",
    "DF",
    "DA",
    "e2e",
    "integration",
    "RM",
];

/// §FS-config.3.4: `E2E` is the canonical `index = false` kind — its home holds
/// case directories rather than a navigable document set, and the `e2e/README.md`
/// one level up describes the case layout in English instead of naming `E2E-` IDs.
/// Every other default folder kind takes the `README.md` default.
///
/// `E2E` is no longer one of the [`DEFAULT_KINDS`], but the default stays keyed
/// on the name: it exists for the configs that declare `E2E` themselves, which
/// is every config `grund init` wrote before the kind left the default set.
pub(super) fn default_kind_index(kind: &str) -> KindIndex {
    match kind {
        "E2E" => KindIndex::Disabled,
        _ => KindIndex::Default,
    }
}

/// Whether a built-in kind declares IDs (§FS-config.3.4). The two test kinds do
/// not: a test is evidence for a claim declared elsewhere, so it has a home and
/// citation directions but no ID namespace.
pub(super) fn default_kind_citable(kind: &str) -> bool {
    !matches!(kind, "e2e" | "integration")
}

/// Default home folder for each built-in kind — the directory `grund id` proposes
/// a path under and `grund check` expects the declaration to live in (§FS-config.3.4.11).
pub(super) fn default_kind_folder(kind: &str) -> Option<&'static str> {
    match kind {
        "AR" => Some("docs/architecture"),
        "DA" => Some("docs/decisions/architectural"),
        "DF" => Some("docs/decisions/functional"),
        "E2E" => Some("e2e/cases"),
        "e2e" => Some("tests/e2e"),
        "integration" => Some("tests/integration"),
        // GRUND, GOAL, RM are single-file kinds — see `default_kind_file`. A
        // kind can always be broken up later by swapping `file = "…"` for
        // `folder = "…"` and moving the document into the folder.
        _ => None,
    }
}

/// Default single-file home for kinds whose declarations all live in one
/// document — `GRUND` in `docs/grund.md`, `GOAL` in `docs/goals.md`, `FS` in
/// `requirements.md`, and `RM` in `docs/roadmap.md` (§FS-config.3.4). Other
/// built-in kinds have no `file` (each declaration is its own file).
pub(super) fn default_kind_file(kind: &str) -> Option<&'static str> {
    match kind {
        "GRUND" => Some("docs/grund.md"),
        "GOAL" => Some("docs/goals.md"),
        "FS" => Some("requirements.md"),
        "RM" => Some("docs/roadmap.md"),
        _ => None,
    }
}

/// Default human title for each built-in kind, printed by `grund id` (§FS-config.3.4,
/// §FS-id.2).
pub(super) fn default_kind_title(kind: &str) -> Option<&'static str> {
    match kind {
        "GRUND" => Some("Why: project motivation"),
        "GOAL" => Some("Where: project direction and outcomes"),
        "FS" => Some("What: behavior, requirements, and constraints"),
        "AR" => Some("How: high-level implementation, structure, and design"),
        "DA" => Some("Architecture decisions and tradeoffs"),
        "DF" => Some("Product behavior decisions and tradeoffs"),
        "E2E" => Some("Executable user scenarios"),
        "e2e" => Some("User scenarios: black-box proof of the spec"),
        "integration" => Some("Integration tests: proof that the parts fit as designed"),
        "RM" => Some("Planned milestones and sequencing"),
        _ => None,
    }
}
