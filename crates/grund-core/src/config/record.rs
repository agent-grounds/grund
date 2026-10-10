//! The effective configuration record (§AR-system.2.3): one validated `Config`
//! per project — every `grund.toml` key (§FS-config.3) merged over the built-in
//! defaults (§FS-config.principle.unit) — with the `[[kinds]]` lookups that
//! answer questions about the finalized kind set.
//!
//! The façade rather than the reader: `v1/` lowers a file into the records of
//! §AR-config.1 and `facade.rs` builds one of these from them (§AR-config.5);
//! this file says what the façade shows and what a built one means. It lived in
//! `model/records.rs` while config was a file-name category, which is what made
//! `model` read the `[[kinds]]` defaults and the point-size policy upward
//! (§AR-system.4); it is config's own now.

#[cfg(test)]
use anyhow::Result;
use std::path::PathBuf;

use super::call_scope::PathBase;
use super::citations::CitationRules;
use super::facade::Records;
use super::kind::KindConfig;
use super::point_sizes::LeadSizeWarning;
use super::project::Schema;
use super::rows::Form;
use super::run_warnings::RunWarning;
use super::slots::{Content, Handle, Slot};
use super::v1;
#[cfg(test)]
use crate::grammar::GrammarKind;
use crate::grammar::{Grammar, LexicalSettings};

#[derive(Clone, Debug, PartialEq)]
pub struct ConfigLocation {
    pub path: PathBuf,
    pub line: usize,
}

/// One `[workspace] optional_members` entry whose directory this checkout does
/// not have, and therefore one namespace this run did not read
/// (§FS-workspace.2.2, §FS-workspace.2.2.1).
///
/// A record rather than a rule: it sits beside the `workspace_absent_optional`
/// field that carries it, because a `Config` the run loaded is what every reader
/// of the fact holds (§FS-workspace.4). It was the workspace component's while
/// `Config` was `model`'s, which made this file read it upward (§AR-system.4);
/// what workspace owns is the walk that *fills* it (§FS-check.4.9).
#[derive(Clone, Debug, PartialEq)]
pub struct AbsentOptionalNamespace {
    /// The entry **as the config wrote it** — the string an author can edit
    /// (§FS-errors.4).
    pub written: String,
    /// The whole alias path this run spells the namespace with: one segment per
    /// workspace level, so an entry one `[workspace]` block down is `sub/vendored`
    /// while the entry itself stays `vendored` (§FS-check.4.9.2). Expansion sets it
    /// to the bare segment; the walk that knows the enclosing path composes the
    /// rest.
    pub alias_path: String,
    /// The `optional_members` line of the block that holds the entry.
    pub source: ConfigLocation,
}

/// The persisted number-only citation policy from `[reference] shorthand`
/// (§FS-config.3.1.1). Trigger input remains authoring sugar under both values;
/// this enum governs only marker-origin shorthand already present in a file.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShorthandPolicy {
    Canonical,
    Accepted,
}

impl ShorthandPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Canonical => "canonical",
            Self::Accepted => "accepted",
        }
    }
}

/// The effective configuration: every `grund.toml` key (§FS-config.3) merged
/// over the built-in defaults (§FS-config.principle.unit), plus the compiled
/// `Grammar` and the `root` / `cli_base` paths the walk and the report use.
#[derive(Clone)]
pub struct Config {
    pub root: PathBuf,
    /// The resolved path argument (or cwd) — the base for reports when
    /// `[output] relative_paths = false`, i.e. the base `grund` would use if no
    /// config were discovered (§FS-config.3.6.1).
    pub cli_base: PathBuf,
    /// The config file that was actually read — either `.agents/grund.toml` or
    /// the bare `grund.toml` (§FS-config.1). `None` in a zero-config tree, where
    /// the defaults come from no file at all.
    ///
    /// A **report path**, not a filesystem handle: relative to `root` for a
    /// standalone project, but to the *workspace* root for a member loaded as
    /// part of one (§FS-errors.4 — a workspace report names members from the
    /// root the run was launched at, so `packages/beta/grund.toml` reads the
    /// same in the diagnostic as it does in `[workspace] members`). Join it
    /// against the base the report uses, never unconditionally against `root`.
    pub config_file: Option<PathBuf>,
    /// The config file at `root` that the discovered one outranks (§FS-config.1.1).
    /// `Some` only for the redundant pair `check` and `config` warn about
    /// (§FS-check.4.3); read by nothing else. Same report-path base as
    /// [`Config::config_file`].
    pub redundant_config_file: Option<PathBuf>,
    pub project_name: Option<String>,
    pub project_name_source: Option<ConfigLocation>,
    /// Optional one-line description rendered beside the project's alias in
    /// generated workspace member lists (§FS-config.3, §FS-workspace.3.1,
    /// §DF-workspace-member-descriptions). Presentation metadata only.
    pub project_description: Option<String>,
    pub marker: String,
    pub trigger: String,
    pub strict: bool,
    /// `[reference] shorthand` (§FS-config.3.1.1): whether a uniquely resolving
    /// marker-origin shorthand must be canonicalized or may persist unchanged.
    pub shorthand: ShorthandPolicy,
    /// `[reference] require_grounding` (§FS-config.3.1.7, §FS-check.3.6,
    /// §DF-require-grounding) — when true, `check` also reports every scanned
    /// source file that carries no resolving citation (and declares no ID inline).
    /// `--require-grounding` on `grund check` forces it on for one run.
    pub require_grounding: bool,
    /// `[reference] grounding_level` (§FS-config.3.4.8.2, §FS-check.3.6.2) — the
    /// default unit inside each governed file, in Markdown heading levels, for
    /// every `[[kinds]]` row that does not set its own. `1` is the file.
    pub grounding_level: usize,
    /// Whether any row's effective `grounding_level` is finer than the file:
    /// the façade's view of a non-empty `ScanDemand` (§AR-config.6.1), which the
    /// scanner reads instead, so it records per-file structure only where a row
    /// asks for it and a level-1 tree pays nothing (§GOAL-fast-feedback).
    pub grounding_units: bool,
    /// `[reference] conversation` (§FS-config.3.1.3, §DF-repo-conversation-opinion) —
    /// the repository's committed conversation-rendering opinion. `None` means no
    /// opinion; the only accepted value is `"link"` (closed enum, widenable later).
    /// Read solely by the agent-entrypoint renderer (§FS-init.2.3.4.17).
    pub conversation: Option<String>,
    /// `[reference] lead_size_warning` (§FS-config.3.1.2, §FS-declarations.checks.oversized-lead.3).
    /// `None` preserves the pre-feature checker byte-for-byte and avoids all
    /// point-body measurement work.
    pub lead_size_warning: Option<LeadSizeWarning>,
    pub inline_style: String,
    pub inline_note_suggested_lines: usize,
    pub inline_note_max_lines: usize,
    pub inline_note_max_columns: usize,
    /// `[reference] inline_note_layout` (§FS-config.3.1.9,
    /// §FS-inline-citation-style.3.3, §DF-inline-note-layout) — the project's
    /// house style for where citations sit inside an inline note. Closed enum:
    /// `any` (default, no constraint) or `citation-first-colon`.
    pub inline_note_layout: String,
    /// `[reference] inline_note_layout_check` (§FS-inline-citation-style.4.4) —
    /// whether `check` reports a layout deviation, and through which channel.
    /// Closed enum: `off` (default), `warn`, or `error`. Inert under
    /// `inline_note_layout = "any"`, which is why it is a second key
    /// (§DF-inline-note-layout.2.1).
    pub inline_note_layout_check: String,
    pub warn_on_suggested: bool,
    pub include: Option<Vec<String>>,
    /// §FS-check.1.3: `grund check --full` for this run — walk the whole config
    /// root and ignore `include`. Not a `grund.toml` key and never read from one
    /// (§DF-check-full-scope.2.5): a project that wants its whole tree governed
    /// widens `include`, and a second knob describing one scope is how two
    /// installs come to disagree (§FS-non-goals.13).
    pub scan_full: bool,
    /// §FS-check.1.3.6.1: this run was given an explicit path below the config
    /// root, so the walk reads the project's ordinary roots *as well as* that
    /// path — the path sets the report scope and not the resolution one. Like
    /// `scan_full` it is a per-run fact rather than a `grund.toml` key and is
    /// never read from one: it describes one invocation's scope, and the
    /// scanner asks for it four frames below the run that decided it.
    pub scan_resolution_wide: bool,
    pub exclude: Vec<String>,
    pub extensions: Vec<String>,
    pub comment_prefixes: Vec<String>,
    pub docstring_python: bool,
    pub respect_gitignore: bool,
    pub output_format: String,
    pub relative_paths: bool,
    pub id_format: String,
    pub section_separator: String,
    pub number_pattern: String,
    pub slug_pattern: String,
    /// The absent-by-default named-section grammar gate (§FS-config.3.2.7).
    pub named_sections: bool,
    pub section_heading_levels: String,
    pub kinds: Vec<KindConfig>,
    /// `[fmt] exclude` (§FS-config.3.10) — the files `grund fmt` performs no
    /// rewrite in, as gitignore-style globs against the config root. Read by
    /// `fmt` alone: the walk, the scan, and every check are untouched by it
    /// (§FS-fmt.2.5.1). Empty is what every config written before the key
    /// existed means.
    pub fmt_exclude: Vec<String>,
    pub fmt_cross_refs_enabled: bool,
    pub cross_ref_anchor_format: String,
    pub workspace_declared: bool,
    pub workspace_members: Vec<String>,
    pub workspace_members_source: Option<ConfigLocation>,
    /// `[workspace] optional_members` (§FS-workspace.2.2) — the members this
    /// repository has declared **may be legitimately absent** from a checkout. A
    /// sibling of `members` with the same grammar less the glob: present, an entry
    /// here is an ordinary member; absent, the block loads without it and the
    /// namespace it would have contributed goes unverified (§FS-workspace.4).
    pub workspace_optional_members: Vec<String>,
    pub workspace_optional_members_source: Option<ConfigLocation>,
    /// §FS-check.4.9: the optional members this run found absent, named by the
    /// whole alias path it spells their namespaces with. Not a `grund.toml` key and
    /// never read from one — like `workspace_boundary_roots`, it is what expansion
    /// learned about this checkout — and stamped onto every project the run loaded,
    /// because both readers need it: the report announces each one once, and
    /// resolution asks whether a citation lands in one (§FS-workspace.4).
    pub workspace_absent_optional: Vec<AbsentOptionalNamespace>,
    /// Where the `[workspace]` table header itself was written (§FS-config.4.3).
    /// The anchor for an error about the *block* rather than about one key — a
    /// block with no `members` key at all still has to say which of a tree's many
    /// blocks it is (§FS-workspace.6.1.3).
    pub workspace_section_source: Option<ConfigLocation>,
    pub workspace_include_root: bool,
    /// Where `include_root` was written (§FS-config.4.3). The breadcrumb
    /// §FS-check.4.10.5 wears: the key that took the block's files out of every
    /// scan is the line the reader should open, which neither the `members` line
    /// nor the `[workspace]` header is. `None` where the key is absent, and the
    /// default `true` makes that unreachable for the one finding that reads it.
    pub workspace_include_root_source: Option<ConfigLocation>,
    pub workspace_boundary_roots: Vec<PathBuf>,
    /// The run's warning channel (§FS-distribution.3.1): the `[workspace]`
    /// cautions of §FS-check.4.10 and §FS-workspace.6.1.7, in the
    /// order the run settled them. Accumulated on the config the run was
    /// launched with, by the points that populate a block's member boundary and
    /// climb the claimed chain, and handed to whichever frontend asked — which
    /// is what keeps the engine from writing one of them to a stream
    /// (§AR-bindings.2). Each stands in place of the `success` marker on an
    /// otherwise clean run (§FS-check.2.1.3). Not a `grund.toml` key.
    pub(crate) run_warnings: Vec<RunWarning>,
    /// The run's `--path-base`, when the caller passed one: it outranks
    /// `relative_paths` for every project the run loads (§FS-cli.3.4). Taken
    /// from the call scope when the config is built, so members carry it too.
    /// Not a `grund.toml` key, so `config show` never prints it.
    pub(crate) path_base: Option<PathBase>,
    /// §AR-workspace.6: the canonical root of **every** project this run loaded.
    /// `workspace_boundary_roots` above says what lies *below* this project, so a
    /// leaf member has none; this says where the *others* are, which is how a
    /// member's walk tells a link into a sibling — or back up into the root
    /// project — from a link into ordinary outside content. Empty for a run that
    /// loaded no workspace, a member checked on its own included (§FS-workspace.6.3).
    pub workspace_project_roots: Vec<PathBuf>,
    /// §FS-workspace.6.1.5: the alias path of the *run's* own workspace root, read
    /// from the outermost workspace and stamped onto every project the run loaded.
    /// Empty at the outermost root and for a single-project run; non-empty exactly
    /// when the run is narrowed to a subtree. Not a `grund.toml` key and never read
    /// from one (like `workspace_boundary_roots`, it is what expansion learned
    /// about this run): §FS-check.3.8.3 reads it to know that a path it cannot
    /// resolve may still be correct at the workspace root.
    pub workspace_scope_path: String,
    /// Parsed `[citations]` direction rules (§FS-config.3.9). Empty/absent unless
    /// the config declares the section.
    pub citations: CitationRules,
    /// Whether the scan computes citing-side classification — declaration body
    /// ranges and each citation's `source_kind` (§AR-scanner.2.4). Only the
    /// citation-direction checks read it, so `grund check` leaves it on while the
    /// read-only commands (`list`, `show`, `refs`, `cover`, `fmt`) turn it off to
    /// skip the post-pass entirely (§AR-benchmarks). Combined with
    /// `citations.declared`, a project without direction rules pays nothing.
    pub classify_citation_sources: bool,
    /// The line ranges a `cover --lines` run asks the owners of, inclusive and
    /// 1-based (§FS-cover.6.1). Empty on every other run, so no scan pays for the
    /// lookup it does not need; non-empty, each scanned file's pass resolves them
    /// while its heading stack is held (§AR-scanner.2.4.4).
    pub owner_lines: Vec<(usize, usize)>,
    pub grammar: Grammar,
    /// The records this façade shows (§AR-config.5), read through
    /// [`Config::project`], [`Config::run`] and [`Config::compiled`].
    pub(super) records: Records,
}

/// The **default** name of the homeless kind — the citing kind of every site
/// outside every configured home (§AR-scanner.2.4.2, §FS-config.3.9.2.2). It is a
/// default and not a fixed name: `code` is the right word for most
/// repositories and the wrong one for a Terraform, SQL, or prose tree, so a
/// project may declare the homeless kind itself and name it (`src`,
/// `modules`, …). See [`Config::homeless_kind`].
pub(super) const CODE_SOURCE_KIND: &str = "code";
/// The default `grounding_level` (§FS-config.3.4.8.2): the file — the H1's own
/// subtree, so one citation anywhere under it. It is the unit every config had
/// before the key existed, which is what keeps the key additive.
pub(crate) const DEFAULT_GROUNDING_LEVEL: usize = 1;
/// The heading levels a `grounding_level` may name (§FS-config.3.4.8.2). Markdown
/// has six, and a value outside them names no heading.
pub(super) const GROUNDING_LEVELS: std::ops::RangeInclusive<usize> = 1..=6;

impl Config {
    /// The built-in defaults — the canonical grammar a conformant tree gets with
    /// no config at all (§FS-config.3, §GOAL-zero-config): the v1 default project
    /// (§AR-config.3.2) under a run rooted at `root`. `grund init` writes these
    /// same values out verbatim as a teaching surface (§FS-init.2.4.3).
    pub(crate) fn default_for(root: PathBuf) -> Self {
        Self::from_project(&v1::default_project(false), root).expect("default grammar must compile")
    }

    /// The homeless kind for this config (§FS-config.3.9.2) — the citing kind
    /// every site outside every configured home resolves to. The declared entry
    /// when the table has one, else the reserved `code`.
    #[cfg(test)]
    pub(crate) fn homeless_kind(&self) -> &str {
        declared_homeless_kind(&self.kinds).map_or(CODE_SOURCE_KIND, |kind| kind.kind.as_str())
    }

    /// Recompile the `Grammar` after a test set the façade's `[id]` /
    /// `[[kinds]]` / `[scan].comment_prefixes` fields directly (§FS-config.3) —
    /// keeps the regexes and the scalar fields in lockstep. A loaded config is
    /// compiled once, from its `Project` (§AR-config.1.5).
    #[cfg(test)]
    pub(crate) fn rebuild_grammar(&mut self) -> Result<()> {
        self.grammar = Grammar::build(
            &self.id_format,
            &grammar_kinds(&self.kinds),
            &self.number_pattern,
            &self.slug_pattern,
            &self.section_separator,
            self.named_sections,
            &self.comment_prefixes,
        )?;
        std::sync::Arc::make_mut(&mut self.records.compiled).grammar = self.grammar.clone();
        Ok(())
    }

    /// §AR-system.2.1: what the grammar component reads of this record — the
    /// compiled patterns plus the scalar keys a lexical reader consults beside
    /// them (§FS-config.3.1, §FS-inline-citation-style.3.3).
    ///
    /// A borrowed view rather than a copy, built wherever a reader asks, so the
    /// grammar always sees the value this record holds *now*: the component
    /// below must not be able to disagree with the config above it about what a
    /// citation or a comment looks like, and a snapshot taken when the grammar
    /// was compiled could (§AR-system.4).
    pub(crate) fn lexical(&self) -> LexicalSettings<'_> {
        LexicalSettings {
            grammar: &self.grammar,
            marker: &self.marker,
            strict: self.strict,
            comment_prefixes: &self.comment_prefixes,
            docstring_python: self.docstring_python,
            inline_style: &self.inline_style,
            inline_note_layout: &self.inline_note_layout,
            inline_note_layout_check: &self.inline_note_layout_check,
        }
    }
}

/// The citable `[[kinds]]` rows as the ID grammar reads them (§FS-config.3.4):
/// the name that prefixes every ID of the kind, and the row's `format` override
/// where it carries one (§FS-config.3.2).
///
/// The `citable` filter is applied here, once, rather than at each of the five
/// places inside `Grammar::build` that used to repeat it: a non-citable kind
/// declares no IDs, so it enters no pattern, and deciding that is config's
/// (§AR-system.2.1).
#[cfg(test)]
fn grammar_kinds(kinds: &[KindConfig]) -> Vec<GrammarKind> {
    kinds
        .iter()
        .filter(|kind| kind.citable)
        .map(|kind| GrammarKind {
            name: kind.kind.clone(),
            format: kind.format.clone(),
        })
        .collect()
}

/// The ID prefixes a config recognizes (§FS-config.3.4): the name of every
/// *citable* kind, in `[[kinds]]` order. A non-citable kind declares no IDs, so
/// its name never tokenizes and never enters the grammar, the `KIND ∈ {…}`
/// vocabulary, or the kind lists `grund id` and `grund list --kind` accept.
pub(crate) fn kind_prefixes(kinds: &[KindConfig]) -> Vec<String> {
    kinds
        .iter()
        .filter(|kind| kind.citable)
        .map(|kind| kind.kind.clone())
        .collect()
}

/// The `known kinds:` line that ends an unknown `--kind` (§FS-list.1.1), a
/// `--selector` refusal that recovers no kind (§FS-rules.8.1), and a
/// `check --rule` refusal that offers no form where a kind is missing
/// (§FS-rules.3.5.4.4): every citable kind named in `citable`, once, in
/// configuration order.
pub(crate) fn known_kinds_line<'a>(citable: impl IntoIterator<Item = &'a str>) -> String {
    let mut seen = std::collections::BTreeSet::new();
    let known = citable
        .into_iter()
        .filter(|kind| seen.insert(*kind))
        .collect::<Vec<_>>();
    format!("known kinds: {}", known.join(", "))
}

/// Why a configured kind cannot be selected with `--kind` or minted from
/// (§FS-list.1.1, §FS-id.1.1). A non-citable kind is not a typo — it is a real row
/// in `[[kinds]]` that will never have a declaration — so the message says that
/// rather than calling it unknown, and names the home, which is the thing the
/// caller can actually go and open.
pub(crate) fn non_citable_kind_error(kind: &KindConfig) -> String {
    match kind.place_label() {
        Some(place) => format!(
            "kind `{}` declares no IDs — {place} is not a citable home",
            kind.kind
        ),
        None => format!("kind `{}` declares no IDs", kind.kind),
    }
}

/// Every citing kind `[citations.<kind>]` may name (§FS-config.3.9): each
/// configured kind, citable or not, plus `code` — but only where the table did
/// not declare the homeless kind itself. A config that names its complement
/// `src` has no `code`, and `[citations.code]` in it is a rule about nothing
/// (§FS-config.3.9.2.1).
pub(super) fn citing_kind_names(kinds: &[KindConfig]) -> Vec<&str> {
    let named = declared_homeless_kind(kinds).is_some();
    kinds
        .iter()
        .map(|kind| kind.kind.as_str())
        .chain((!named).then_some(CODE_SOURCE_KIND))
        .collect()
}

/// The `[[kinds]]` entry that *is* the homeless kind, if the table declares one
/// (§FS-config.3.9.2.1): non-citable, and with no `folder` or `file`, because it
/// is the complement of every home rather than one of them. At most one entry
/// can be this, which the config validator holds.
pub(super) fn declared_homeless_kind(kinds: &[KindConfig]) -> Option<&KindConfig> {
    kinds
        .iter()
        .find(|kind| !kind.citable && kind.folder.is_none() && kind.file.is_none())
}

/// Whether a `[[kinds]]` row opted its home into first-class values
/// (§FS-values.2, §FS-config.3.4): its form is a value form (§AR-config.1.3).
pub(crate) fn kind_uses_values(schema: &Schema, kind: &str) -> bool {
    // §AR-config.6.2: the value shape is read through the kind's slots.
    schema.kinds().any(|(name, configured)| {
        name == kind
            && configured
                .slots()
                .any(|slot| slot.handle == Handle::Numbered && slot.content == Content::Values)
    })
}

/// The chapter a `[[kinds]]` row declared its values under, if any
/// (§FS-config.3.4.13, §FS-values.2.5). The same row lookup as
/// `kind_uses_values` above, and the only thing that turns a named section into
/// a value root.
pub(crate) fn kind_value_chapter<'a>(schema: &'a Schema, kind: &str) -> Option<&'a str> {
    schema
        .kinds()
        .find(|(name, _)| *name == kind)
        .and_then(|(_, configured)| match &configured.form {
            // §AR-config.6.2: a value kind's chapter is its one named values slot.
            Form::Value { .. } => configured.slots().find_map(|slot| match slot {
                Slot {
                    handle: Handle::Named(chapter),
                    content: Content::Values,
                    ..
                } => Some(chapter),
                _ => None,
            }),
            // A rule kind yields no slot, so its v1 value chapter is read off the form.
            Form::Rule { value_chapter } => value_chapter.as_deref(),
            Form::Prose => None,
        })
}
