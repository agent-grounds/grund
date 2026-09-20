use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::e2e::E2eCase;
use super::headings::{NearMissHeading, SectionHeadingOutsideDeclaration, UnmarkedHeading};
use super::paths::{normalize_path_lexically, paths_same_location};
use super::values::{
    DeclarationSource, EmbeddedValueRoot, InvalidValueSite, ValueBinding, ValueComponent,
};

/// A parsed ID: its kind plus whichever of `{number}` / `{slug}` the configured
/// `[id] format` carries (§FS-config.3.2).
///
/// `Id` is rendered for output via `render_id` / `format_id`, which honour the
/// repo's `[id] format` and `--width` (§FS-config.3.2). There is deliberately no
/// `Display` impl — a bare `{}` would have to guess the format and would be wrong
/// on any repo that configured a non-default one.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Id {
    pub kind: String,
    pub num: Option<u32>,
    pub slug: Option<String>,
}

const LEGACY_ID_SENTINEL: char = '\0';

impl Id {
    /// Preserve an off-grammar declaration's exact spelling without widening
    /// the configured authoring grammar (§FS-config.3.2.5). The spelling lives in
    /// the otherwise grammar-owned slug slot behind an impossible sentinel, so
    /// existing `Id` construction and kind-based graph rules stay unchanged.
    pub(crate) fn legacy(kind: String, spelling: &str) -> Self {
        Self {
            kind,
            num: None,
            slug: Some(format!("{LEGACY_ID_SENTINEL}{spelling}")),
        }
    }

    pub(crate) fn legacy_spelling(&self) -> Option<&str> {
        self.slug
            .as_deref()
            .and_then(|slug| slug.strip_prefix(LEGACY_ID_SENTINEL))
    }
}

/// One declaration site discovered by the scanner: a `# <ID>: …` heading in a
/// Markdown file or an inline declaration in a code doc-comment
/// (§AR-scanner.2.1, §AR-scanner.4), with its section body map
/// (§AR-scanner.2.2) and, for stub headings, the inline-home path it points at
/// (§FS-show.2.3, §FS-check.3.4).
#[derive(Debug, Clone)]
pub struct Declaration {
    pub id: Id,
    pub file: PathBuf,
    pub line: usize,
    pub heading_level: usize,
    pub sections: BTreeMap<String, SectionInfo>,
    /// Every later heading that claimed a section path `sections` already holds,
    /// in file order, narrowed to the ones inside this declaration's own body
    /// span (§AR-scanner.2.2.3).
    ///
    /// Nothing *resolves* through it: a `§<ID>.<path>` citation, the completion
    /// candidates, and §FS-check.3.9 all read the map. It exists for the two
    /// commands that have to say the coordinate is ambiguous — §FS-check.3.16
    /// names each colliding line, and §FS-show.2.2.2 refuses a query for a path
    /// it holds. `--toc` reads neither: it re-scans the source, which is what
    /// §FS-show.2.2.2.3 exempts it for.
    pub duplicate_sections: Vec<(String, SectionInfo)>,
    pub is_stub: bool,
    pub defined_in: Option<PathBuf>,
    pub e2e_case: Option<E2eCase>,
    /// Heading text after `<ID>:` — the one-line title an author wrote
    /// (§AR-scanner.2.1). `None` when the heading carries no `: <text>` tail, or
    /// when the heading is a stub link (`# <ID>: [<text>](<path>)`), whose tail
    /// is a path, not a title.
    pub title: Option<String>,
    /// The declaration's body line span (1-indexed, inclusive), §AR-scanner.2.4.1:
    /// in Markdown it runs from the declaration heading to the line before the
    /// next same-or-higher heading (or end of file); in a source file it is
    /// bounded by the comment/docstring block, capped before the next
    /// declaration in a multi-ID block. A stub heading and an `E2E` case span
    /// their single declaration line only. Used to classify a citation's citing
    /// side (its `enclosing_declaration`) and shared with `grund cover` /
    /// §RM-gap-report.
    pub body_start: usize,
    pub body_end: usize,
    /// Exact source metadata for catalog consumers. Ordinary Markdown and
    /// source declarations carry `Text`; home JSON members retain both their
    /// member slice and key span (§FS-values.2.2.2, §FS-values.6.2).
    pub source: DeclarationSource,
    /// `Some(true)` for a valid opted-in value declaration, `Some(false)` for
    /// a readable declaration with invalid value grammar, and `None` for an
    /// ordinary declaration (§FS-values.2, §FS-values.5.1).
    pub value_valid: Option<bool>,
}

/// One numeric or explicitly named subsection heading recorded inside a
/// declaration (§AR-scanner.2.2): the heading text used for anchors, plus the
/// source line and Markdown heading level used by the section-depth checker
/// (§FS-check.3.9).
#[derive(Debug, Clone)]
pub struct SectionInfo {
    pub title: String,
    pub line: usize,
    pub heading_level: usize,
    pub value: Option<ValueComponent>,
    /// Present only when this existing numeric section carries the exact
    /// embedded-value suffix. The section remains the catalog identity; this
    /// metadata records authority without synthesizing a declaration
    /// (§FS-values.2.4, §FS-values.6.1).
    pub value_root: Option<EmbeddedValueRoot>,
}

/// One citation site: an `<ID>[.<section>]` token, optionally `§`-prefixed, or
/// a uniquely owned local section spelling promoted to the same graph edge
/// (§AR-scanner.2.3, §AR-scanner.2.4). `has_marker` drives strict-mode
/// filtering (§FS-config.3.1) and is what `grund fmt` upgrades a bare token
/// from (§FS-fmt.2.2).
#[derive(Debug)]
pub struct Citation {
    pub namespace: Option<String>,
    pub id: Id,
    pub section: Option<String>,
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub has_marker: bool,
    /// Written in the number-only shorthand (§FS-check.1.2). The scanner's
    /// resolution pass has already rewritten `id` to the canonical declaration
    /// when exactly one matched (§AR-scanner.2.6.6), so this flag is what
    /// distinguishes a resolved shorthand from a full citation — and the only
    /// thing that has to: every graph consumer deliberately ignores it and reads
    /// `id`. When `id.slug` is still `None` the shorthand resolved to zero or
    /// several declarations.
    pub shorthand: bool,
    /// Written as a declaration-local numeric section path such as `<§>2.1`.
    /// The scanner has already resolved `id` to the uniquely enclosing
    /// declaration, so graph consumers read this as an ordinary edge; only the
    /// checker and formatter inspect the flag to require canonical storage
    /// (§FS-check.3.24, §AR-scanner.2.4).
    pub local_section: bool,
    /// Whether `grund fmt` may canonicalize this noncanonical citation in place:
    /// a number-only shorthand or a local section spelling. It is `false` inside
    /// inline code, a Markdown link destination, or a runtime string literal,
    /// the contexts §FS-fmt.2.3 forbids every rewrite from touching. The site is
    /// still a citation in every other sense. For number-only shorthand, the
    /// flag also withholds the §FS-check.3.13.1 error that names
    /// `grund fmt --write`, so `check` never demands an edit the formatter
    /// refuses to make. It is always `true` for canonical citations.
    pub shorthand_rewritable: bool,
    /// Whether this shorthand is glued to a second number — `§SPEC-001→SPEC-003`
    /// — which makes it a numeral in a run rather than a citation, and forbids
    /// `grund fmt` from expanding it (§FS-fmt.2.4.1). Distinct from
    /// `shorthand_rewritable` because the two reach different verdicts: an
    /// illustration in inline code wants no edit and earns no finding, while a
    /// run needs one and earns §FS-check.3.15. Always `false` when `shorthand`
    /// is `false`.
    pub numeric_run: bool,
    pub text: String,
    /// The comment block this citation sits in, when that block is an inline
    /// citation site (§FS-inline-citation-style.1). `None` outside a comment
    /// block, in Markdown, in a block that declares an ID — and in a **doc
    /// comment**: `///`, `//!`, `/** … */`, a docstring, or a comment a position
    /// language puts right above a definition is documentation, not a note, so
    /// it is no site and carries no budget, style, or layout
    /// (§FS-inline-citation-style.1.1).
    pub inline_site: Option<InlineCitationSite>,
    /// The resolved *citing* kind for this site (§AR-scanner.2.4.2): the kind of
    /// the enclosing declaration, else the file's unique kind home, else the
    /// homeless kind (`code` by default, §FS-config.3.9.2.2). Drives the citation-direction
    /// checks (§FS-config.3.9, §AR-checker.2.9, §AR-checker.2.10).
    pub source_kind: String,
    /// The nearest preceding declaration whose body range contains this site
    /// (§AR-scanner.2.4.3), or `None` when the site sits in no declaration body.
    /// Lets the obligation pass ask "does this declaration cite the target?" as
    /// a lookup rather than a re-scan.
    pub enclosing_declaration: Option<Id>,
}

/// A configured-marker-plus-digit token deferred until declaration body spans
/// are known (§FS-check.1.1.8, §AR-scanner.2.3). Supported numeric paths
/// with one enclosing declaration are promoted to [`Citation`]; unsupported or
/// ownerless forms stay here so the checker can diagnose them without inventing
/// a graph target (§AR-scanner.2.4).
#[derive(Debug)]
pub(crate) struct LocalSectionCitationCandidate {
    pub(crate) text: String,
    pub(crate) section: Option<String>,
    pub(crate) file: PathBuf,
    pub(crate) line: usize,
    pub(crate) column: usize,
    pub(crate) rewritable: bool,
    pub(crate) inline_site: Option<InlineCitationSite>,
    pub(crate) source_kind: String,
    pub(crate) enclosing_declaration: Option<Id>,
}

/// A marker-prefixed token the configured grammar rejected, retained during
/// the same file scan until the project catalog can prove it names an exact
/// persisted declaration (§FS-check.1.1.1, §FS-config.3.2.6).
#[derive(Debug)]
pub(crate) struct LegacyCitationCandidate {
    pub(crate) namespace: Option<String>,
    pub(crate) tail: String,
    pub(crate) file: PathBuf,
    pub(crate) line: usize,
    pub(crate) column: usize,
    pub(crate) inline_site: Option<InlineCitationSite>,
    pub(crate) inline_block_lines: Option<std::sync::Arc<[String]>>,
    pub(crate) source_kind: String,
    pub(crate) enclosing_declaration: Option<Id>,
}

/// The enclosing source-comment citation site for one citation
/// (§FS-inline-citation-style.1, §FS-inline-citation-style.2.3). Markdown
/// citations and citations outside recognized comment blocks carry `None`, and
/// so does a citation in a **doc comment**: what a language calls documentation
/// is not an inline note, so its block is not a site
/// (§FS-inline-citation-style.1.1). Such a citation still resolves and is still
/// checked for everything else — dangling, direction, grounding, shorthand.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct InlineCitationSite {
    pub first_line: usize,
    pub last_line: usize,
    /// Width of the site's longest line in **characters** — Unicode scalar
    /// values, one column each (§FS-inline-citation-style.2.3.2). Not the byte
    /// length, and not the display width: `é`, `—`, and the `§` marker itself
    /// cost one column apiece, and so does a tab
    /// (§DF-note-columns-are-characters). This is a different measure from the
    /// byte-addressed start column a `Citation` records (§AR-scanner.3); the
    /// two agree only on a line of pure ASCII.
    pub max_columns: usize,
    pub has_note: bool,
    /// The site's judged lines that deviate from
    /// `[reference] inline_note_layout` (§FS-inline-citation-style.3.3.1), 1-based
    /// and ascending. Judged is rule 1's set, not every line carrying a citation:
    /// the line that opens the note, and any later line that opens with a
    /// citation of its own.
    ///
    /// This is **what `check` will report**, never an independent survey of the
    /// tree. It is empty — and no line is classified — under the default
    /// `inline_note_layout = "any"`, under `inline_style = "citation-only"`,
    /// where no note is permitted and so none has a layout, and at
    /// `inline_note_layout_check = "off"`, where the verdicts would reach no
    /// channel (§FS-inline-citation-style.4.4). So the field costs a project only
    /// what it asked for: until it configures a layout and gates it, no line is
    /// tokenized or classified on its account (§AR-scanner.3.2). A consumer that
    /// wants the deviations of a tree whose gate is `off` is asking a different
    /// question and has to gate it, or classify the lines itself — reading an
    /// empty list here is not evidence that the tree conforms.
    pub layout_violations: Vec<usize>,
}

pub(crate) type TextOverlays = BTreeMap<PathBuf, String>;

/// Everything the scanner found in one tree walk — declarations grouped by ID
/// (so duplicates surface, §FS-check.3.3) and citations in encounter order. This
/// is the scanner's whole output; the checker (§AR-checker) consumes it without
/// re-reading files.
#[derive(Default)]
pub struct Findings {
    pub declarations: BTreeMap<Id, Vec<Declaration>>,
    pub citations: Vec<Citation>,
    /// Numeric and enabled named section headings that the line scan attached
    /// to a stale declaration context, then the body-span post-pass proved the
    /// declaration does not own (§FS-check.3.23).
    pub section_headings_outside_declarations: Vec<SectionHeadingOutsideDeclaration>,
    /// Markdown ATX headings owned by declaration bodies but carrying neither a
    /// declaration ID nor a section coordinate (§FS-check.4.14,
    /// §AR-scanner.2.2.7). The scanner assigns the owner and a collision-free
    /// suggested coordinate before any checker consumes this list.
    pub unmarked_headings: Vec<UnmarkedHeading>,
    pub(crate) legacy_citation_candidates: Vec<LegacyCitationCandidate>,
    pub(crate) local_section_citation_candidates: Vec<LocalSectionCitationCandidate>,
    pub value_bindings: Vec<ValueBinding>,
    pub invalid_value_declarations: Vec<InvalidValueSite>,
    pub invalid_value_bindings: Vec<InvalidValueSite>,
    /// Every file the walk read successfully (§AR-scanner.1) — the universe the
    /// `[reference] require_grounding` check iterates over (§FS-check.3.6,
    /// §DF-require-grounding). Files that failed to read are not here; they are in
    /// the walk's `ScanError` list instead.
    pub scanned_files: Vec<PathBuf>,
    /// Every directory the walk descended into (§AR-scanner.1.10), scan roots
    /// included — the candidate set the unlisted-`[workspace]` rule of
    /// §FS-check.4.8 probes. Carried rather than judged here: the walk knows what
    /// it reached, and nothing about workspaces (§AR-workspace.1).
    pub walked_dirs: Vec<PathBuf>,
    /// Per-file heading and doc-comment structure, for the files a grounding
    /// unit finer than the file is asked of (§AR-scanner.2.7, §FS-check.3.6.2).
    /// Empty — and never collected — unless `Config::grounding_units` is set.
    pub file_structure: BTreeMap<PathBuf, FileStructure>,
    /// `<§>`-escaped citation illustrations (§AR-scanner.2.5): the schematic
    /// `<§>[alias/]ID[.section]` shape the detection passes deliberately skip
    /// because the literal `<§>` does not end with the marker. Inert to every
    /// existing check; recorded only so the checker can flag one whose ID
    /// resolves to a real declaration — a likely bracketed live citation rather
    /// than an intended illustration (§FS-check.2.3.1, §AR-checker.2.11).
    pub escaped_citations: Vec<Citation>,
    /// Headings that open like a declaration and do not parse as one
    /// (§FS-check.4.6) — recorded where the scan already decided the line was
    /// not a declaration, so the rule costs one regex on heading-shaped lines
    /// rather than a second read of the tree.
    pub near_miss_headings: Vec<NearMissHeading>,
}

/// ID-query slice mode (§FS-show.1.6): each rung adds to the previous one —
/// `--brief` is heading + first paragraph; `Default` adds the rest of the lead
/// (cut at the first child section); `Toc` adds the nested section map; `Full`
/// adds every subsection body. `Outline` is an internal-only mode used by `Toc`
/// to collect the section map; the CLI does not expose it.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum ShowRenderMode {
    Brief,
    Default,
    Toc,
    Full,
    Outline,
}

pub struct ShowSection {
    pub path: String,
    pub title: String,
    pub depth: usize,
}

/// What an ID query resolved to: the body text to print, the `path:line` it
/// came from, the section map (`--toc` only), and the pre-rendered JSON when
/// `--format json` was asked for (§FS-show.3, §FS-errors.5).
pub struct ShowOutput {
    pub body: String,
    pub path: PathBuf,
    pub line: usize,
    pub json: Option<String>,
    pub sections: Vec<ShowSection>,
}

/// Whether this stub heading is the one-line pointer to an inline declaration in
/// code (`# <ID>: [text](src/foo.rs)` whose target also declares `<ID>`) — such a
/// stub does not count as a second home, so it is not a duplicate (§AR-scanner.4,
/// §FS-show.2.3).
pub(crate) fn is_stub_for_inline_decl(
    root: &Path,
    decl: &Declaration,
    decls: &[Declaration],
) -> bool {
    if !decl.is_stub {
        return false;
    }
    let Some(target) = &decl.defined_in else {
        return false;
    };
    let resolved = resolve_stub_target(root, &decl.file, target);
    decls
        .iter()
        .any(|other| paths_same_location(&other.file, &resolved) && other.file != decl.file)
}

pub(crate) fn resolve_stub_target(root: &Path, stub_file: &Path, target: &Path) -> PathBuf {
    if target.is_absolute() {
        return target.to_path_buf();
    }
    let stub_file = if stub_file.is_absolute() {
        stub_file.to_path_buf()
    } else {
        root.join(stub_file)
    };
    let markdown_relative =
        normalize_path_lexically(&stub_file.parent().unwrap_or(root).join(target));
    if markdown_relative.exists() {
        markdown_relative
    } else {
        normalize_path_lexically(&root.join(target))
    }
}

/// One Markdown heading outside a fence (§AR-scanner.2.7): its line, its level,
/// and its text without the leading `#`s, which is what a section finding quotes
/// back (§FS-check.3.6.3).
pub struct FileHeading {
    pub line: usize,
    pub level: usize,
    pub text: String,
}

/// One doc-comment block in a source file (§AR-scanner.2.7): its 1-indexed
/// inclusive line span, and whether its first line starts at column 0.
/// Indentation is the parse-free stand-in for "top-level item" that
/// §FS-check.3.6.2.2 reads at level 2 — it holds across Rust, Python, Java, Go, and
/// Kotlin without knowing any of them (§FS-non-goals.3).
pub struct DocCommentBlock {
    pub start: usize,
    pub end: usize,
    pub indented: bool,
}

/// One file's grounding structure (§AR-scanner.2.7). A Markdown file fills
/// `headings` and a source file `doc_comments`; `total_lines` closes the last
/// subtree or block, which would otherwise have no end.
#[derive(Default)]
pub struct FileStructure {
    pub headings: Vec<FileHeading>,
    pub doc_comments: Vec<DocCommentBlock>,
    pub total_lines: usize,
}
