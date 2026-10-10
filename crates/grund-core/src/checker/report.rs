use super::conform::conform;
use super::judge::{CheckWorkspace, judge};
use super::support::{diagnostic_cmp, sort_diagnostics};
use crate::config::{Frame, Rules, Schema};
use crate::model::{Catalog, CheckReport, Diagnostic, Expected, TextOverlays};

/// AR-checker: how grund validates the scanner's findings
///
/// The checker takes the resolved `Catalog` produced from §AR-scanner, asks
/// §AR-rules for chapter-rule diagnostics, and produces one `CheckReport`. It
/// implements the checks in §FS-check and orchestrates, but does not implement,
/// the parser/facts/engine split of §FS-rules.11.
///
/// ## placement: Where the checker sits
///
/// ```text
/// resolver ─► loaded Catalog ──┐
/// rules ─► Diagnostic ──────────┤
/// config ─► Schema, Rules ──────┼─► [ conform + judge ] ─► Report ─► api ─► cli, lsp
/// writers ─► Expected ──────────┘
/// ```
///
/// The sixth box of the pipeline (§AR-system.2.6). It takes `Catalog` from the
/// resolver (§AR-system.2.10), rule diagnostics from §AR-system.2.12, the
/// schema and rules it judges by, and the expected bytes the writers rendered
/// (§AR-checker.1.3), and gives one `Report` to the api
/// (§AR-system.2.9), which every frontend renders unchanged. It knows no
/// frontend. Chapter-rule sentence parsing, fact adaptation, evaluation, and
/// semantic deduplication stay in §AR-rules; this component only sequences them
/// and merges their diagnostics.
///
/// A rule judges what the scan recorded in `Findings`. These read a file's text
/// after the scan as well — the text an editor's overlay can stand in for, which
/// a probe for whether a path exists or a scope path made canonical never reads —
/// and `tests_post_scan_readers.rs` holds the list to the code. A stub's target is
/// not among them: the scan reads it once and records its verdict on the stub
/// (§AR-scanner.4.6), so the broken-stub rule, a cited section, a value binding's
/// target and a rule's `cites` fact read that record instead of the file
/// (§AR-resolver.5):
///
/// - §AR-checker.2.3's hint: the line of a dangling Markdown citation, from disk.
/// - §AR-checker.2.6: every configured kind index, from disk, through
///   §AR-checker.2.16's membership derivation, so that an index entry is not
///   counted as an inbound citation (§DF-index-not-an-inbound-citation).
/// - §AR-checker.2.7: each agent entrypoint, from disk. Its companions are listed,
///   and read, by the writers before the check, in `Expected` (§AR-checker.1.3).
/// - §AR-checker.2.16: every configured kind index, from disk.
/// - The opt-in lead budget (§FS-declarations.checks.oversized-lead): every
///   Markdown or doc-comment home, through the resolver's point-body slicer
///   (§AR-system.2.10), overlay first and disk second.
///
/// ## terms: Terms
///
/// Leans on §FS-terms.terms.1 (declaration, ID, kind, home, citable, body,
/// section, coordinate, index), §FS-terms.terms.2 (marker, citation, shorthand,
/// canonical form, citation site), §FS-terms.terms.3 (stub, doc-comment, note),
/// §FS-terms.terms.4 (scan, scope, config root, workspace, member),
/// §FS-terms.terms.5 (finding, suggestion, verdict), §FS-terms.terms.6 (direction,
/// rule), §FS-terms.terms.7 (value, component, binding), and §FS-terms.terms.8
/// (box).
///
/// ## 1. Inputs and outputs
///
/// The checker is two functions and a merge, and what each is handed is the cut
/// §DA-config-concern-records.2.3 makes (§FS-config.concerns): a stage that is
/// not handed a concern cannot read it, so the compiler holds the line.
///
/// - Input: the loaded `Catalog` from the resolver, the `Schema` and `Rules`
///   records of §AR-config.1.2, the `Expected` bytes of §AR-checker.1.3, and a
///   `Frame { run, compiled, name, alias, version, display }` carrying the `Run` and `Compiled`
///   of §AR-config.1.5, which are not concerns. Chapter-rule `Diagnostic`s
///   arrive from §AR-rules after the driver. No input is the `Config` façade
///   (§AR-config.5) and none is `Presentation`.
/// - Output: a `CheckReport` containing three channel partitions: `errors`,
///   `warnings`, and opt-in `suggestions`. Each partition is deterministic; the
///   CLI renderer groups text by channel while preserving JSON's global order
///   (§FS-errors.4.1, §FS-non-goals.9) for §GOAL-friendliness-first.
///
/// ### 1.1 `conform`: the node-local half
///
/// `conform(schema, catalog, frame, overlays)` judges each declaration and
/// heading against its own kind's shape and nothing else: duplicate,
/// misplaced-declaration, broken-stub and declaration-near-miss
/// (§FS-declarations.checks), the section family of `sections.rs` (heading
/// level, duplicate and orphan section, section outside a declaration, unmarked
/// heading), inline citation style (§AR-checker.2.14) and the opt-in lead budget
/// (§FS-declarations.checks.oversized-lead). It is handed no `Rules` and no
/// `Presentation`. Where it reads a kind's value shape it reads it through the
/// kind's slots (§AR-config.6.2), so a field model added to the kind reaches it
/// without a second reader.
///
/// ### 1.2 `judge`: the relational half
///
/// `judge(rules, schema, catalog, expected, frame, workspace)` judges what one
/// fact says about another: resolution (§AR-checker.2.3, §AR-checker.2.12),
/// explicit values (§AR-checker.2.18, after resolution), escaped citations that
/// resolve, kind indexes, unused declarations, grounding, citation directions,
/// and the managed-block comparison (§AR-checker.2.7). The chapter-rule pass is
/// on this side too, called where the api calls it after the driver, because the
/// ad-hoc sentence and the resolution-completeness flag are run facts that
/// arrive there. `judge` reads `Schema` read-only because a rule is written in
/// its vocabulary — it names kinds and places, and an index is spelled on its
/// kind's row (§AR-config.1.4). It is handed no `Presentation`: where a verdict
/// depends on bytes presentation decides, it compares `Expected` and renders
/// nothing.
///
/// ### 1.3 `Expected`: what presentation decided, as bytes
///
/// `Expected` is plain data in `model/`, built by the writers before `judge`
/// runs (§AR-system.2.11): the managed-block version, every agent entrypoint
/// with the config-derived sections it should carry, each rendered for its own
/// conversation surface (§FS-init.2.3.6.1) in today's comparison order, the
/// entrypoint probe's io error kept as data, and the canonical link target of
/// each index-file citation that external enrollment inspects
/// (§FS-check.3.18.3). The chapter-rule section is rendered from the check run's
/// own catalog, so building it adds no walk (§FS-init.2.3.4.15). A
/// presentation-only change can therefore move a drift finding through these
/// bytes, and cannot move any other verdict.
///
/// ### 1.4 The merge and its order
///
/// Per channel, `conform`'s findings are appended, then `judge`'s, then the one
/// stable `sort_diagnostics` (path, line, message) runs, as it did over the
/// single driver. Inside each half the passes keep their relative order, so
/// ties inside a pass come out as before; a tie across the halves would need one
/// path, line and message from two codes, and a debug assertion in the merge
/// holds that it never happens (§REQ-deterministic-output).
///
/// ## 2. Rules
///
/// Each rule is a single pass over part of the findings. Rules are independent —
/// adding a rule does not force re-scanning.
///
/// ### 2.1 Duplicate declarations (§FS-declarations.checks.duplicate)
///
/// For each ID with more than one declaration, emit one error anchored at the
/// lexicographically-first site (sort by `path`, then `line`); list every other
/// site parenthetically in the message. This keeps the report's `path:line:`
/// prefix invariant (§AR-checker.3, §FS-check.2.1) while still naming all sites. The
/// declarations are the ones `paired_declarations` derives from each stub's verdict
/// (§FS-declarations.stubs.verdict, §AR-scanner.4.6): a stub and the inline declaration it
/// points at count as one, not two — whether or not the walk reached that declaration, and
/// however many stubs point at it — and a declaration a stub stands for is named at its
/// target (§FS-declarations.checks.duplicate.1, §FS-declarations.checks.duplicate.2,
/// §FS-declarations.checks.duplicate.3).
///
/// ### 2.2 Misplaced declarations (§FS-declarations.checks.misplaced-declaration)
///
/// For each declaration, validate placement from the scanner-recorded `file` and
/// `id.kind`. A single-file kind (`[[kinds]].file`) must live in that exact file.
/// Separately, when the declaration's file is contained by exactly one configured
/// kind home, the declaration kind must match that home kind. The checker builds
/// one kind-home index per check run: `file` homes are exact matches and `folder`
/// homes are path-prefix matches under the config root; if zero or multiple homes
/// match, there is no unique expected home kind and the checker emits no
/// home-kind diagnostic.
///
/// This rule uses only `Declaration` records; it does not rescan files. Stubs are
/// checked by their stub file path for home-kind placement, while the existing
/// broken-stub rule still verifies that the linked source file contains the inline
/// declaration it claims.
///
/// ### 2.3 Dangling citations (§FS-check.3.1)
///
/// For each citation whose ID has no declaration, emit one error at the citation
/// site.
///
/// ### 2.4 Missing sections (§FS-check.3.2)
///
/// For each citation with a section path, look up the section in the matching
/// declaration's recorded sections. Where the walk recorded only a stub of the ID,
/// the lookup goes on to the stub's target, scanned or not (§FS-check.3.2.1),
/// through the target's record the stub's verdict holds
/// (§FS-declarations.stubs.verdict, §AR-resolver.5). Missing → one error at the
/// citation site.
///
/// ### 2.5 Broken inline-spec stubs (§FS-declarations.checks.broken-stub)
///
/// For each declaration whose H1 has the stub shape `# <ID>: [<text>](<path>)`
/// (description after the colon is a single bare markdown link), report the verdict
/// the scan recorded on the stub (§FS-declarations.stubs.verdict, §AR-scanner.4.6): a
/// target that is missing draws `stub link target missing`, and one the scan does not
/// read, or that does not declare the ID, draws `stub link target lacks <ID>` — one
/// error at the stub site.
/// The rule reads no file and decides nothing of its own, so the stub it reports is
/// exactly the stub `show` refuses, `list --size` labels and `fmt` will not link
/// through (§FS-declarations.checks.broken-stub.4), and the verdict is the one a save
/// would give, a hidden or unlisted name judged as the stub wrote it and a heading
/// inside a fence or a raw-text HTML block declaring nothing
/// (§FS-declarations.checks.broken-stub.1, §FS-declarations.checks.broken-stub.2,
/// §FS-check.1.1.5.1, §FS-declarations.checks.broken-stub.3).
///
/// ### 2.6 Unused declarations (§FS-check.4.1)
///
/// For each declared ID never cited, emit one warning. Warnings do not cause a
/// non-zero exit. `E2E` declarations are exempt — a case is exercised by being
/// run, not by being cited (§FS-check.4.1.3).
///
/// ### 2.7 Invalid agent-entrypoint init block (§FS-check.3.5)
///
/// When `<root>/AGENTS.md` exists, verify its versioned `grund init` block (and the
/// matching block in any non-symlink companion entrypoint that is present): a
/// missing block, an older version, or a newer unsupported version is one error at
/// the entrypoint's line. When only companion entrypoints exist, validate the ones
/// that already contain a managed block and leave project-owned unmanaged files
/// alone.
///
/// The text a current block is compared against is not this rule's to know, and
/// this rule renders nothing. What a managed block should say is a function of
/// config and of the run's own catalog, so the writers render each entrypoint's
/// config-derived sections through the same renderers `grund init` writes
/// through (§AR-system.2.11) and hand them over as `Expected`
/// (§AR-checker.1.3); this rule compares those bytes with disk, byte for byte,
/// and never reads a presentation key. Which companion files count as
/// entrypoints is the one entrypoint walk in `scanner/agent_entrypoints.rs`,
/// which `init`'s own selection is derived from, and whose surface reach takes
/// `&Presentation` explicitly (§AR-scanner.7). So this rule cannot disagree
/// with the command that is supposed to clear it (§FS-init.2.1, §FS-init.2.3).
///
/// ### 2.8 Ungrounded units — opt-in (§FS-check.3.6, §DF-require-grounding)
///
/// Off by default, and asked per `[[kinds]]` row: each scanned file resolves to
/// the one row that governs it (§FS-check.3.6.1), and that row's effective
/// `require_grounding` / `grounding_level` decide whether the file is checked and
/// what the unit inside it is (§FS-config.3.4.8). At level 1 the unit is the file
/// and the rule is what it always was — one resolving citation anywhere, or an
/// inline declaration, anchored at line 1. Above it the units are the heading
/// subtrees the scanner recorded for a Markdown file, or its doc-comment blocks
/// for a source one (§AR-scanner.2.7), each finding anchored at its own unit. The
/// pass is `grounding.rs`; `[citations]` obligations read the same cut
/// (§AR-checker.2.9), so *whether* and *what* are asked of one thing.
/// Whether the scanner recorded the structure a row's units are cut from is
/// the `ScanDemand` config computed with the same level rule (§AR-config.6.1),
/// so what was recorded and what is cut stay one rule.
///
/// ### 2.9 Citation-direction obligations (§FS-check.3.11, §FS-config.3.9, §DF-citation-directions)
///
/// When `[citations]` sets `must` / `should` obligations for a citing kind, every
/// top-level declaration of that kind must carry, in its body, at least one citation
/// satisfying each obligation entry (entries are conjunctive, `|` inside an entry is
/// a disjunction). The body extent and the per-citation `enclosing_declaration` come
/// from the scanner (§AR-scanner.2.4), so this pass is a lookup, not a re-scan. The
/// homeless-kind obligation is per source file (§FS-config.3.9.2.4) rather than
/// per declaration, and both per-file units are cut by the row's
/// `grounding_level` like the grounding pass above (§FS-check.3.11.3). A `must` miss is a `missing-citation` error; a `should` miss is a
/// `suggested-citation` suggestion, emitted only under `--suggestions` (§FS-check.2.3).
///
/// ### 2.10 Citation-direction prohibitions (§FS-check.3.12, §FS-config.3.9, §DF-citation-directions)
///
/// When `[citations]` sets `must-not` / `should-not` prohibitions for a citing kind,
/// every citation site of that kind (its resolved `source_kind`) to a prohibited
/// target — matched on cited kind and namespace per the rule grammar
/// (§FS-config.3.9.3) — is reported at the site. A `must-not` hit is a
/// `forbidden-citation` error; a `should-not` hit is a `discouraged-citation`
/// suggestion, emitted only under `--suggestions` (§FS-check.2.3).
///
/// ### 2.11 Escaped citations that resolve (§FS-check.2.3.1)
///
/// The scanner records every `<§>`-escaped illustration (§AR-scanner.2.5) into
/// `findings.escaped_citations`, a list inert to every rule above. This pass is
/// its only reader: for each escape it runs the same resolver as the dangling
/// check (§AR-checker.2.3) and, when the ID resolves to a real declaration, emits an
/// `escaped-citation-resolves` suggestion — the mirror of dangling, which fires
/// when a *live* citation does not resolve. It is a suggestion, never a warning
/// or error, so it is withheld unless `--suggestions` is passed and never moves
/// the exit code; illustrating a real ID is legitimate.
///
/// ### 2.12 Number-only shorthand citations (§FS-check.3.13, §DF-number-only-citation-shorthand)
///
/// The scanner flags every citation written in the number-only shorthand and, in
/// the same walk, rewrites the uniquely-resolving ones to their canonical `Id`
/// (§AR-scanner.2.6.6). So by the time the checker runs, a resolved shorthand is
/// indistinguishable from a full citation to every rule above — which is the
/// point: `refs`, `cover`, the unused warning (§AR-checker.2.6), and the direction passes
/// (§AR-checker.2.9, §AR-checker.2.10) all count it without knowing it exists.
///
/// This pass adds the one thing that does differ: under the target project's
/// `canonical` policy, a finding naming the canonical form to write; under
/// `accepted`, a unique marker-origin shorthand adds no form finding
/// (§FS-config.3.1.1). Unknown and ambiguous candidates remain findings under both
/// policies. It looks the candidate set up in a per-namespace `(kind,
/// number)` index — built on first use, because deriving it per site is quadratic
/// on the tree this rule asks people to migrate — so the three outcomes (unique,
/// ambiguous, unknown) pick the message. The dangling check (§AR-checker.2.3) skips shorthand
/// sites, so one *cause* never yields two findings; rules judging a different fact
/// about the same site, such as a missing section (§AR-checker.2.4) or a forbidden direction
/// (§AR-checker.2.10), are untouched and report alongside it.
///
/// A resolving shorthand at a site `grund fmt` may not rewrite (§FS-fmt.2.3 — inline
/// code, a link destination, a runtime string) is not reported at all. The citation
/// still resolves and still counts everywhere above; withholding the finding is what
/// keeps `check` from naming `grund fmt --write` as the fix for a site the formatter
/// declines to touch, which would leave the repository permanently red.
///
/// ### 2.13 Scope tiering — `--full` (§FS-check.1.3, §FS-check.3.14, §DF-check-full-scope)
///
/// `[scan] include` decides which roots the walk starts from, so a citation
/// outside it is invisible rather than merely unchecked. `grund check --full`
/// widens the walk to the whole config root and the run then has two scopes.
/// `references.rs` owns both halves of that: the tier is read off the
/// *whole* walk first — only the tier §FS-check.3.14.2 admits, so a directory
/// nobody configured is never judged against conventions it never adopted — and the
/// findings are then narrowed in place to the configured scope, so every rule
/// above sees exactly the tree a run without the flag sees. That ordering is
/// what makes `--full` purely additive: it can only add findings, never withdraw
/// one the ordinary run would have made. The narrowing also undoes the shorthand
/// resolution the wider walk enabled (§AR-scanner.2.6.6) where the declaration it
/// resolved against has just been dropped, so such a site is the unresolved
/// shorthand a plain run reports — one cause, one finding (§FS-check.3.13.3).
///
/// ### 2.14 Inline citation style (§FS-check.3.10, §FS-check.4.4, §FS-inline-citation-style.4)
///
/// One pass over `findings.citations`, deduplicated by enclosing comment block,
/// judging each block against `[reference] inline_style`, the `inline_note_*`
/// budgets, and `inline_note_layout`. Everything it compares — the block's span,
/// its widest column, whether it carries a note, and which of its lines deviate
/// from the configured layout — was recorded by the scanner (§AR-scanner.3), so
/// this rule reads no file. A site that misses several caps yields one finding
/// per cap; a block whose layout deviates yields one per offending *line*,
/// anchored there rather than at the block's opener, because that is the line an
/// author edits (§FS-inline-citation-style.4.4.1). Two of the three tiers are
/// opt-in and silent by default: the soft cap under `warn_on_suggested`, the
/// layout under `inline_note_layout_check`.
///
/// The rule is `inline_style.rs` rather than here — one file per invariant, the
/// arrangement §AR-checker.2.12's shorthand rule already uses for the same reason. The
/// classifier it judges by stays in `grammar/inline_note_layout.rs`, which is
/// what the scanner annotates a site from: the two stages read one answer about
/// what a well-laid-out note is, and only this component turns it into a
/// finding (§AR-system.2.1).
///
/// ### 2.15 Duplicate section paths (§FS-declarations.checks.duplicate-section, §DF-duplicate-section-path)
///
/// One pass over the declarations. The scanner records a section path once, by
/// the first heading that claims it, and appends every later claimant *inside the
/// declaration's own body* to `duplicate_sections` (§AR-scanner.2.2.3); this rule
/// groups that list by path and emits one error per collided path, anchored at
/// the first heading with the rest named in the message — §AR-checker.2.1's shape for
/// declarations, one level down. The heading-level rule above reads only the map,
/// so a duplicate heading is not additionally judged for depth: nothing resolves
/// to it, and the run has already said it should not exist
/// (§DF-duplicate-section-path.2.4).
///
/// Nothing here re-derives *which* headings a declaration owns — the scan
/// answered that once, which is what makes `show`'s refusal (§FS-show.2.2.2.2) name
/// exactly the coordinates this rule reports. A heading in the next item's
/// doc-comment and a stub's prose are outside the body and never reach the list,
/// so neither is filtered here (§FS-declarations.checks.duplicate-section.2).
///
/// ### 2.16 Kind indexes (§FS-check.3.18, §FS-check.3.17, §DF-index-entry-form)
///
/// One pass per `[[kinds]]` entry that has a `folder` and an enabled `index`
/// (§FS-config.3.4). For each, membership is the declarations under that
/// folder's whole subtree — a stub and the inline body it points at collapsing
/// to one ID, as in §AR-checker.2.1 — plus an external inline declaration whose canonical
/// bare-ID source link enrolls it directly (§FS-check.3.18.3). The citations already
/// recorded in the index file say which members it names. The index file itself
/// is re-read (§AR-checker.placement), because wrapper form and an external
/// enrollment's exact destination are facts about the line, not the citation
/// record. Ordinary in-folder entries still require only the wrapper shape; only
/// external enrollment compares the destination to the one `fmt` derives
/// (§FS-fmt.6.2, §DF-index-entry-form.2.7), and that destination arrives as
/// `Expected`'s index target for the (index file, ID) pair (§AR-checker.1.3):
/// the anchor profile is presentation's, and this pass only looks it up.
///
/// Both halves of the entry contract are errors, each anchored where its own fix
/// is: a missing entry at the declaration, a bare one at its line in the index.
/// They arrived at that verdict by different routes — the bare entry on arrival,
/// the missing one at the end of a ramp (§DF-index-compatibility-ramp.3) — and
/// the anchors are what still tells them apart. The same pass owns the carve-out
/// that keeps §AR-checker.2.6 honest: an index entry is not an inbound citation, so the
/// unused warning still fires for a declaration only its own index names
/// (§DF-index-not-an-inbound-citation). The finding pass lives in
/// `index.rs` and its shared membership derivation in
/// `index_entries.rs`, one file per invariant family and bounded helper
/// (§AR-core-module-layout.1, §AR-core-module-layout.3).
///
/// ### 2.17 Named section prefixes (§FS-declarations.checks.orphan-section)
///
/// One pass over each declaration's scanner-recorded section set. For every
/// name-bearing path, walk its proper prefixes and emit one `orphan-section`
/// error at the descendant heading for the first prefix absent from the same
/// declaration. The pass does not parse headings or infer hierarchy from their
/// Markdown placement; it consumes the shared path set. It is independent of
/// heading-depth and duplicate checks, so those findings compose rather than
/// suppress one another. Purely numeric paths bypass the pass.
///
/// ### 2.18 Explicit values (§FS-values.5, §DA-explicit-value-bindings)
///
/// A focused `checker_values` pass consumes the scanner's declarations,
/// components, bindings, and exact spans, and reads no file: where a binding's
/// home is a stub whose target the walk did not reach, it compares against the
/// target's record the scan kept on the stub (§AR-resolver.5). It routes the
/// binding citation through the same local/workspace resolver as every citation.
/// For an embedded binding, the longest marked parent path owns the site; an
/// invalid immediate parent suppresses comparison and secondary binding errors,
/// while a binding to the root or below a component remains invalid. Config,
/// declaration, and resolution failures suppress comparison. Only one valid
/// unique numbered target reaches exact decimal-or-decoded-string equality,
/// producing the fixed value errors and declaration site required by §FS-values.5.
///
/// ### 2.19 Sections outside declarations (§FS-declarations.checks.section-outside-declaration)
///
/// The scanner narrows both section maps against the declaration body span and
/// retains each rejected numeric or enabled named heading as one located site.
/// This pass translates those sites into hard findings; no consumer can resolve
/// them because checking happens after the shared maps have already been pruned.
///
/// ### 2.20 Unmarked Markdown headings (§FS-declarations.checks.unmarked-heading)
///
/// One pass translates the scanner's body-owned Markdown-only candidates into
/// fixed warning findings. The scanner has already selected the nearest enclosing
/// declaration, skipped fences and source doc-comments, and assigned each heading
/// its deterministic unused coordinate suggestion; the checker formats that
/// record and applies no config or heading-level-mode gate. Full-scope narrowing
/// drops candidates outside configured scan scope before this pass. The ordinary
/// report path supplies text, JSON, exact-code selection, and LSP parity.
///
/// ## 3. Diagnostic data and rendering
///
/// The checker retains each finding's channel, path, line, code, and message.
/// The CLI turns that data into the channel-bearing located text shape while
/// keeping `<path>:<line>:` first for editor jumps; JSON and LSP render the same
/// data in their own unchanged shapes (§FS-check.2.1, §FS-errors.2.1).
///
/// Findings without a single source location (CLI launch errors, malformed
/// configuration that prevents a scan from starting, a per-file read failure
/// mid-walk) are emitted on stderr as `error: <message>` per §FS-check.2.1.1,
/// distinguishable from per-finding lines by the leading `error:`.
///
/// ## 4. Why a separate stage from the scanner
///
/// The scanner produces a complete view of the world; the checker enforces rules
/// on that view. Keeping them separate means:
///
/// - New rules can be added without touching the scanner.
/// - The optional LSP server (§AR-lsp) can run a subset of checks (e.g., only
///   dangling references on the active file's citations) against a cached scan.
/// - Tests can feed synthetic `Catalog` directly to the checker without disk I/O.
pub(crate) fn check_on_disk(
    rules: &Rules,
    schema: &Schema,
    catalog: &Catalog,
    expected: &Expected,
    frame: Frame<'_>,
    workspace: &CheckWorkspace<'_>,
) -> CheckReport {
    check_with_workspace_and_overlays(
        rules,
        schema,
        catalog,
        expected,
        frame,
        workspace,
        &TextOverlays::new(),
    )
}

/// The driver over editor overlays too, so §FS-declarations.checks.oversized-lead
/// measures the live text; `check_on_disk` is it with none.
///
/// `frame` is displayed by the report's root (§FS-workspace.8.1) — the workspace
/// root's in workspace mode, the project's own otherwise — and checked as the
/// member's alias. A path baked *into* a message is spelled through it, or in a
/// workspace it would be spelled from the member's root while the finding's own
/// anchor is spelled from the workspace's, and neither the reader nor an editor
/// could follow it (§FS-config.3.6).
///
/// Why the escaped-citation finding is only a suggestion: illustrating a real ID
/// in prose is legitimate, so a resolving escape is never an error — it is
/// withheld unless the caller passes `--suggestions`.
///
/// Why an index entry is not counted as an inbound citation: an index names every
/// declaration in its folder by construction, so counting its entries would leave
/// every ID in an indexed folder permanently "cited" and empty the `unused`
/// warning of everything it exists to find.
///
/// Why grounding needs no tooling: it is a pure function of (tree, config) — no
/// git, no AST. Markdown is exempt from it because a document is not
/// implementation; a non-citable home is not, because it is a directory the
/// maintainer declared matters and is usually all Markdown, so the exemption
/// would make the rule inert exactly where it was asked for. Which place is
/// asked, and how finely, is a `[[kinds]]` row's to say (§FS-config.3.4.8).
pub(crate) fn check_with_workspace_and_overlays(
    rules: &Rules,
    schema: &Schema,
    catalog: &Catalog,
    expected: &Expected,
    frame: Frame<'_>,
    workspace: &CheckWorkspace<'_>,
    overlays: &TextOverlays,
) -> CheckReport {
    let conformed = conform(schema, catalog, frame, overlays);
    let judged = judge(rules, schema, catalog, expected, frame, workspace);
    merge(conformed, judged)
}

/// §AR-checker.1.4: per channel, `conform`'s findings then `judge`'s, then the one
/// stable sort the single driver ran. A tie across the halves would make the
/// order depend on which half ran first, so none may exist (§REQ-deterministic-output).
fn merge(conformed: CheckReport, judged: CheckReport) -> CheckReport {
    CheckReport {
        errors: merge_channel(conformed.errors, judged.errors),
        warnings: merge_channel(conformed.warnings, judged.warnings),
        suggestions: merge_channel(conformed.suggestions, judged.suggestions),
    }
}

fn merge_channel(mut conformed: Vec<Diagnostic>, judged: Vec<Diagnostic>) -> Vec<Diagnostic> {
    debug_assert!(
        no_cross_half_tie(&conformed, &judged),
        "a conform finding and a judge finding share path, line and message"
    );
    conformed.extend(judged);
    sort_diagnostics(&mut conformed);
    conformed
}

/// Whether no finding of one half sorts equal to one of the other (§AR-checker.1.4).
fn no_cross_half_tie(conformed: &[Diagnostic], judged: &[Diagnostic]) -> bool {
    let mut keys: Vec<&Diagnostic> = conformed.iter().collect();
    keys.sort_by(|a, b| diagnostic_cmp(a, b));
    judged
        .iter()
        .all(|j| keys.binary_search_by(|c| diagnostic_cmp(c, j)).is_err())
}
