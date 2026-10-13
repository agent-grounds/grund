# REQ-one-answer: every command gives the same answer about one tree

Every command `grund` offers gives the same answer to the same question about the same
tree, except as [§REQ-one-answer.differences](REQ-one-answer.md#differences-the-differences-known-when-this-requirement-landed) lists. An agent that reads a declaration with
`show`, an editor that underlines a citation and a CI run of `check` are three readers of
one tree; if two of them answer one question differently, the citation an agent trusted
is not the one the gate passed ([§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration), [§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work)), and the
same engine behind three surfaces is not the same engine ([§GOAL-multi-language](../goals.md#goal-multi-language-same-engine-three-platforms)).

Two rules govern the list. A listed difference is outside the contract until its row
closes: this requirement forbids no verdict a listed row describes, and it is never the
prior prohibition a correction of that row rests on ([§REQ-backwards-compatibility.5](REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids)). And
the list only shrinks: a difference that predates this requirement and is found later is
added by an amendment naming the baseline it was found on, while a difference a change
introduces is a defect of that change and cannot be amended in.

## contract: One question, one tree, one answer

The surfaces are `check`, `grund <ID>` in every form, `refs`, `list`, `list --size`,
`cover`, `fmt`, the editor and the bindings, each through its own entrypoint. Two answers
are to the same question when they state the same fact: which declarations an ID has,
which sections a declaration has, whether a stub is healthy, which text a file has, what a
narrowed run reports, which anchor a link takes. They are about the same tree when the
files, the configuration and the effective text are the same, where an editor's open
buffer is its file's text for every entrypoint that accepts buffers. An entrypoint with no
buffer input reads the disk without breaking this contract.

Three pairs of inputs are promised one answer although they are not one input: a stub
target outside `[scan] include` and the same target scanned ([§REQ-one-answer.stubs.outside-include](REQ-one-answer.md#stubsoutside-include-a-target-outside-the-include-answers-as-if-scanned)),
an edit to a stub's target held unsaved and the same edit saved ([§REQ-one-answer.text.targets](REQ-one-answer.md#texttargets-a-stubs-target-answers-as-its-saved-text-would)),
and a `check` narrowed to a path and the whole `check` ([§REQ-one-answer.scope.narrowed](REQ-one-answer.md#scopenarrowed-a-narrowed-run-answers-as-the-whole-run-does)).
Each keeps the domain stated where it is promised.

## identity: Which declarations an ID has

### identity.declarations: Every command reads one declaration set

Configured literals such as `:` remain part of a conforming ID when it is discovered and
read, so the complete ID printed by `list` is the same ID `show` accepts
([§FS-declarations.line.configured-literals](../functional-spec/FS-declarations.md#lineconfigured-literals-configured-literals-belong-to-the-canonical-id), [§FS-show.1.1](../functional-spec/FS-show.md#11-id)).

Where `check` reports an ID a duplicate, `show` and `refs` refuse it as ambiguous at the
same sites ([§FS-show.2.2.1](../functional-spec/FS-show.md#221-ambiguous-id), [§FS-refs.4.1](../functional-spec/FS-refs.md#41-a-stubs-homes-are-found-in-its-target)), `list` and `list --size` mark its rows
duplicate, and `fmt` wraps none of its citations ([§FS-fmt.6.4.2](../functional-spec/FS-fmt.md#642-a-citation-of-a-broken-stub-or-of-an-ambiguous-id-is-not-wrapped)).

### identity.kinds: Every command reads one kind verdict

Every command that takes a kind name — `list --kind`, `list --selector`, a rule's subject
and object, a `[citations]` target, `grund id` — reads its verdict from the one reading of
[§FS-scan.projects.kinds](../functional-spec/FS-scan.md#projectskinds-whose-kinds-a-run-reads), within the scope that command documents. Each command keeps its own
scope, so two commands may accept different names; within one scope a name is citable,
configured but not citable, unknown, or unverifiable here, for every command alike, and a
non-citable kind is refused for that reason everywhere ([§FS-errors.2.5.1](../functional-spec/FS-errors.md#251-a-kind-that-declares-no-ids)).

## sections: Which sections a declaration has

### sections.map: Every surface reads one section map

Every coordinate-bearing surface reads the same body-local map ([§FS-declarations.section.map](../functional-spec/FS-declarations.md#sectionmap-the-section-map-is-the-declarations-body-span)):
direct and batch `show`, exhaustive batch generation, citation and value resolution,
`refs`, completion, list/size output, duplicate detection, and editor navigation, citation
lookups, highlights, and hover counts. A heading the map rejects can therefore neither
resolve nor be suggested, listed, measured, navigated to, validated as an embedded-value
root, or treated as a duplicate claimant.

### sections.ambiguity: `check` and `show` judge a section path from one record

`check` and `show` answer from one recorded section set
([§FS-declarations.checks.duplicate-section](../functional-spec/FS-declarations.md#checksduplicate-section-duplicate-section-path), [§FS-show.2.2.2](../functional-spec/FS-show.md#222-ambiguous-section)), so `grund <ID>.<path>`
refuses exactly when `check` reports `<ID>.<path>` a duplicate section and returns a body
exactly when it does not. Two readers that each decided for themselves would disagree — a
fenced example, a heading past the end of the body — and a coordinate `check` calls clean
but `show` will not resolve is [§REQ-no-wrong-citation](REQ-no-wrong-citation.md#req-no-wrong-citation-a-citation-never-resolves-to-a-guess) failing quietly in the other
direction.

### sections.named: Every command reads a kind's named sections under its own project

Whether a kind's named chapters are addressable is read under the project that configures
the kind ([§FS-scan.projects.kinds](../functional-spec/FS-scan.md#projectskinds-whose-kinds-a-run-reads)), so `list --selector KIND.NAME` at a workspace root
answers whatever `grund <alias>/<ID>.NAME` answers for the same declaration, whatever the
root or another member sets.

## stubs: What a stub answers

### stubs.verdict: Every command reads one verdict per stub

`check`, `show`, `refs`, `list`, `list --size`, `fmt --cross-refs` and the editor all read
each stub's verdict, and the count of the ID's declarations it gives
([§FS-declarations.stubs.verdict](../functional-spec/FS-declarations.md#stubsverdict-one-verdict-per-stub-reached-once-per-scan)); none reaches an answer of its own. So on one tree they
cannot disagree: where `check` reports a stub broken, `show` refuses it as a broken stub
([§FS-show.2.3.4](../functional-spec/FS-show.md#234-broken-stub)), `list --size` labels its row `broken stub` ([§FS-list.3.4.3](../functional-spec/FS-list.md#343-text-rows)), and `fmt`
wraps no citation through it ([§FS-fmt.6.4.2](../functional-spec/FS-fmt.md#642-a-citation-of-a-broken-stub-or-of-an-ambiguous-id-is-not-wrapped)).

### stubs.outside-include: A target outside the include answers as if scanned

A target outside `[scan] include` is counted, measured and linked exactly as it is with
the target scanned, a target that declares the ID twice included ([§FS-list.3.4.6](../functional-spec/FS-list.md#346-a-home-outside-the-scan)).

## text: Which text a file has

### text.targets: A stub's target answers as its saved text would

The verdict on a stub and the body `show` slices out of its target never come from two
versions of the target ([§FS-declarations.stubs.target.1](../functional-spec/FS-declarations.md#stubstarget1-the-target-is-read-as-a-save-would-write-it), [§FS-show.2.3.7](../functional-spec/FS-show.md#237-a-stubs-target-is-found-by-its-id)). An unsaved
edit to the target moves every answer at once, the editor's diagnostics and its navigation
included, exactly as saving it would: every answer is the one the same text gives once it
is saved.

### text.parity: The editor answers as the command line does

Same input and same configuration give the same diagnostics, the same hover body and the
same definition target in the editor as on the command line, byte for byte
([§FS-lsp.4](../functional-spec/FS-lsp.md#4-determinism-and-parity-with-the-cli), [§FS-non-goals.13](../functional-spec/FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)).

## scope: What a narrowed run answers

### scope.narrowed: A narrowed run answers as the whole run does

`grund check <path>` and `grund check .` never disagree about a citation they both read:
an ID declared anywhere in the project resolves under either, and [§FS-check.3.1](../functional-spec/FS-check.md#31-dangling-citation) fires
under neither ([§FS-check.1.3.6.1](../functional-spec/FS-check.md#1361-a-path-scope-narrows-the-report-not-the-resolution)).

## writes: What a write derives

### writes.anchors: `show`'s anchor is the anchor `fmt` writes

The `anchor` `show` reports for a heading ([§FS-show.3.1.3.1](../functional-spec/FS-show.md#3131-the-heading-anchor)) is exactly the anchor
`grund fmt --cross-refs` derives for a citation of the same coordinate ([§FS-fmt.6.2](../functional-spec/FS-fmt.md#62-form)).

## differences: The differences known when this requirement landed

Each row names the two answers that differ, the tree that shows it, and its grade: **R**
reproduced on `a420f558af`, **C** decided by a code path read on it. A defect every
surface shares is not a row; it belongs to the point that defines the right answer.

Each row also cites the decision record that rules on it: which answer is right, and the
step or release that closes the row. A row carries a ruling when it names a step listed
live in `tests/integration/one_answer_steps.toml`, or cites a `DF-` or `DA-` record that
resolves; a row that names neither is unruled, and an unruled row is a defect of the list
itself. So a row whose step closed without removing it cannot stay unnoticed: the list
shrinks because each row says what retires it, not because someone remembers to.

### differences.identity: Identity

G03 and G04 closed with [§REQ-one-answer.identity.kinds](REQ-one-answer.md#identitykinds-every-command-reads-one-kind-verdict), and G06, a
fixed word in the wrong case, with [§FS-rule-refusals.selectors](../functional-spec/FS-rule-refusals.md#selectors-a-refused-selector-is-answered-with-a-selector). G09, a difference that predates
this requirement, was found later, on `e8c40221d7`, and added by amendment: `code`, the
homeless kind's default name, refused as not citable by a `[citations]` target and as
unknown by a rule's subject and object, `list --kind` and `list --selector`. It closed in
0.19.0 with [§DF-homeless-code-declares-no-ids](../decisions/functional/DF-homeless-code-declares-no-ids.md#df-homeless-code-declares-no-ids-code-with-no-row-is-refused-as-declaring-no-ids-alone-from-0190) (#680). No row of identity remains open.

### differences.sections: Sections

P01, P02, P03, P07, P10 and P12 closed with [§DF-one-parse-answers-from-0-19](../decisions/functional/DF-one-parse-answers-from-0-19.md#df-one-parse-answers-from-0-19-one-parses-other-answers-ship-in-0190-after-a-release-that-announces-them-and-warns-where-a-verdict-moves) in 0.19.0, when
every command came to read one parse of each file (#639).

P16, the editor snapshot's span for a C-family section title keeping the `*/` the `--toc`
title leaves out, closed in 0.19.0 with [§DF-fmt-and-editor-answers-from-0-19](../decisions/functional/DF-fmt-and-editor-answers-from-0-19.md#df-fmt-and-editor-answers-from-0-19-fmts-link-wrapper-and-the-editors-title-span-change-in-0190-after-a-release-that-announces-them)
([§FS-lsp.1.3.4](../functional-spec/FS-lsp.md#134-origin-span-and-result-shape)).

### differences.stubs: Stubs

No stub difference is listed. U01 and U05 closed with [§DF-stub-target-section-declared-twice](../decisions/functional/DF-stub-target-section-declared-twice.md#df-stub-target-section-declared-twice-a-section-a-stubs-target-declares-twice-is-reported-where-it-is-written-scanned-or-not) and
[§DF-stubs-by-home-and-at-the-target](../decisions/functional/DF-stubs-by-home-and-at-the-target.md#df-stubs-by-home-and-at-the-target-a-rule-declared-through-a-stub-applies-now-and-a-stub-is-read-by-its-home-and-shown-at-its-target-from-0190), U04, U07 and U08 with
[§DF-stub-target-citations-and-chapters](../decisions/functional/DF-stub-target-citations-and-chapters.md#df-stub-target-citations-and-chapters-a-stubs-target-lends-the-stubbed-declaration-its-citations-and-chapters-and-what-that-newly-raises-warns-until-0190), and Placement, U03a to U03d and U06 in 0.19.0, when
[§DF-stubs-by-home-and-at-the-target](../decisions/functional/DF-stubs-by-home-and-at-the-target.md#df-stubs-by-home-and-at-the-target-a-rule-declared-through-a-stub-applies-now-and-a-stub-is-read-by-its-home-and-shown-at-its-target-from-0190) moved plain `list`, every reader of an outside target's
sections, and every finding about a stubbed declaration itself to the target
([§FS-declarations.stubs.pairing.4](../functional-spec/FS-declarations.md#stubspairing4-every-surface-shows-a-paired-declaration-at-its-target), [§FS-declarations.stubs.target.5](../functional-spec/FS-declarations.md#stubstarget5-what-a-target-outside-the-include-gives-and-what-stays-unjudged)).

### differences.text: Text

No reader differs on text. R06a to R06d, the index check, the inline-code hint on a dangling
citation, the `AGENTS.md` block check and the end-to-end manifest reader, closed in 0.19.0 with
[§DF-four-editor-checks-read-saved-text](../decisions/functional/DF-four-editor-checks-read-saved-text.md#df-four-editor-checks-read-saved-text-four-editor-checks-read-the-saved-file-for-one-more-release), and every reader now reads the text a save
would write ([§FS-scan.text.buffer](../functional-spec/FS-scan.md#textbuffer-a-files-text-is-what-a-save-would-write)).

### differences.scope: Scope

R01 and R02 closed with [§DF-path-narrows-the-report-in-0-19](../decisions/functional/DF-path-narrows-the-report-in-0-19.md#df-path-narrows-the-report-in-0-19-in-0190-a-path-narrows-what-show-list-and-id-report-not-what-they-resolve-and-workspace-check--resolves-every-project-it-sees) in 0.19.0, when `show`, `list`
and `id` came to resolve against the owning project and a workspace `check <dir>` the projects
a run from `<dir>` sees (#621). No row of scope remains open.

## proof: Proved by comparison

The requirement is proved by asking every surface each question on one tree, and each
equivalence on both of its inputs, through each surface's own entrypoint, and by checking
each answer against the point that defines it as well, because two surfaces wrong alike
agree. `crates/grund-cli/tests/command_agreement.rs` is that comparison. The control
baseline entering S4 was seeded thin so that its gate did not grow unpaid: it asked
the command line's surfaces
over the reproduced rows of the ledger behind this list, one tree per row of
[§REQ-one-answer.differences](REQ-one-answer.md#differences-the-differences-known-when-this-requirement-landed) the command line can show, and the first plain project of a fixed
set of end-to-end families. Its retained divergence ledger fails on a divergence it does not
list and on a row that no longer reproduces. A row only the editor, shell completion or
an unsaved buffer shows is deferred there by name, with the step that builds its harness.
The enabled `agreement_acceptance::s4_executes_the_complete_corpus_matrix` now drives
the candidate corpus materializer and CLI/editor/Python/Node entrypoints. Completed
receipts identify original cases, variants, requests, answers, source-byte witnesses,
reviewed manifest goldens and individually validated applicability. Execution and
paid timing must be established before discharging S4; the control baseline above
is not a claim of delivered candidate coverage.

No engine sweep has been removed. `tests/integration/lsp_cli_parity.rs` still compares
the real editor server's diagnostics with `check`'s over
the end-to-end corpus ([§REQ-one-answer.text.parity](REQ-one-answer.md#textparity-the-editor-answers-as-the-command-line-does)) and
`crates/grund-cli/tests/stub_resolution_agreement.rs` the stub answers of every surface
([§REQ-one-answer.stubs](REQ-one-answer.md#stubs-what-a-stub-answers)), and `crates/grund-cli/tests/kind_verdict_agreement.rs` asks every
kind-taking surface about one name on one tree ([§REQ-one-answer.identity.kinds](REQ-one-answer.md#identitykinds-every-command-reads-one-kind-verdict),
[§REQ-one-answer.sections.named](REQ-one-answer.md#sectionsnamed-every-command-reads-a-kinds-named-sections-under-its-own-project)). Real-server URI/range/code/severity/message
delivery, both workspace warnings and plural/singular/zero hover wording remain
transport assertions there. Independent completeness of generic source-site witnesses,
unrepresented equivalences and paid gate timing remain review obligations until the
execution and timing receipts establish them; matching host envelopes alone is no
independent truth about an omitted answer.

Every row of every ledger behind the list carries a ruling in the sense of
[§REQ-one-answer.differences](REQ-one-answer.md#differences-the-differences-known-when-this-requirement-landed): the migration rows of `tests/integration/citation_ledger.toml`, the
rows of `tests/integration/primitive_allowlist.toml`, the comparison's deferred rows and its
widening, and the grammar properties' deferred rows each name a step listed in
`tests/integration/one_answer_steps.toml` or a decision record that resolves. Each listed
step names the decision record it carries out and is named by at least one row, so the
list of live steps cannot keep a step nobody waits on. `tests/integration/test_one_answer_ledgers.py`
refuses a row that names neither, a difference row that cites no record, and a step no row
names, without reaching the forge: a step is live because it is listed, and the change that
closes a step deletes its entry.

### proof.matrix: The complete comparison matrix

S4 completes the comparison over **every case** discovered under `tests/e2e/cases/`,
not just the first plain project of selected families. Each original case maps to
executed evidence or an explicit, justified inapplicability. Discovery includes
`.agents`-only, nested and absent configurations, no-input-repository creation and
refusal cases, command cwd/path/stdin context and symlink manifests. `expected.repo/`
is a result oracle, not an additional input. Deduplication preserves all of those
semantics and maps every original case to its execution; byte equality alone is
insufficient. Mutating `fmt` questions use independent copies.

For each applicable input, execute whole-tree, narrowed, explicit path-argument and
editor-overlay variants. Ask each answering surface its applicable questions:
declaration sites and kind verdicts, sections and ambiguity, citations and resolution,
stub targets/sites, bodies and spans, scope/refusals/warnings and written anchors.
The surfaces include CLI `check`, direct and batch `show`, `refs`, `list` and
`list --size`, `cover` and `cover --lines`, `fmt`, the core editor snapshot and the
actual Python and Node packages built from the same source. Host conversion checked
against a Rust oracle does not replace either host's execution.

Compare facts on their common domain: `cover` bounds its scan
([§FS-scan.path.cover](../functional-spec/FS-scan.md#pathcover-cover-runs-one-narrowed-scan)),
while `check` bounds its report
([§FS-scan.path.check](../functional-spec/FS-scan.md#pathcheck-check-reads-more-than-it-reports)).
CLI and bindings compare isolated saved edits; the editor compares the equivalent
held edit, without adding public buffer parameters
([§REQ-one-answer.contract](REQ-one-answer.md#contract-one-question-one-tree-one-answer)).
Record the reason and defining point for each inapplicable case/variant/question/surface
cell, and assert its applicability accounting. An observed refusal is an answer,
not permission to discard the case. Root-config filters, blanket skips and swallowed
refusals cannot discharge this matrix.

### proof.evidence: Execution and independent truth

The executable acceptance observes completed entrypoint calls and their inputs and
results, not source strings, deleted debt labels or counts reported by the comparison
itself. Evidence identifies the original case, materialized tree, cwd/path/config and
symlink context, variant, question, surface and expected fact. It accounts for every
discovered case as the corpus grows, without freezing today's case count as a cap.

Surface agreement is accompanied by independent, defining-point expectations:
reviewed corpus exit/stdout/stderr/result-tree goldens and explicit question-specific
fixture facts. A second engine call, equality of `list` and `list --size`, an absent
golden treated as an empty answer or regenerated expectations cannot prove truth.
Each compared question needs its own expected fact or a justified applicability
decision; one unrelated golden cannot stand for all answers on a tree.

Keep positive controls, unlisted and stale divergence rejection and requirement-row
accounting. Existing ruled differences remain outside the contract until closed;
newly observed pre-existing differences require an amendment naming the baseline,
and differences introduced by the widening must be fixed
([§REQ-one-answer.differences](REQ-one-answer.md#differences-the-differences-known-when-this-requirement-landed)).

### proof.transport: The editor wire remains a separate proof

The widened core snapshot comparison can replace duplicated engine sweeps only once
equivalent coverage is demonstrated. `tests/integration/lsp_cli_parity.rs` (or an
explicitly documented transport replacement) continues to launch the real server
and assert published diagnostics, URI/path and range/line conversion, diagnostic
code, severity and message, both workspace-warning shapes and actual hover usage
wording for plural, singular and zero counts. Core snapshot equality proves none of
those wire behaviors
([§FS-lsp.1.1.3](../functional-spec/FS-lsp.md#113-workspace-warnings-on-the-runs-warning-channel),
[§FS-lsp.1.2.5](../functional-spec/FS-lsp.md#125-what-a-whole-id-title-counts),
[§FS-lsp.4](../functional-spec/FS-lsp.md#4-determinism-and-parity-with-the-cli)).

### proof.delivery: Paid widening and honest closure

S4 closes only after the complete matrix executes: empty `WIDENING`, remove S4's
live entry and every operational dependent row, and retain offline ledger integrity
checks. Synthetic parser literals are not operational debt. Repointing debt does
not complete it. Until then the thin-seed accounting above remains the baseline;
the ignored acceptance test is forced red before implementation, then enabled by
the implementation. On delivery, replace the baseline description with the executed
coverage and name any remaining obligations held only by review.

The delivery records fixed control/candidate SHAs, repeated comparable runner/cache/
toolchain/build samples, raw CI job timings and URLs, and added/removed seconds and
net delta for each affected gate, including native builds and host startup. Pay for
growth within that gate: savings in another parallel job, one warm local run or merely
staying under fifteen minutes do not establish payment
([AR-ci.requirements.1](../architecture/AR-ci.md#requirements1-every-gate-averages-at-most-fifteen-minutes--directional)).
Distinguish paired experimental evidence from the eight-run rolling main measurement;
unrelated baseline drift does not expand this change's scope.

## why: Why this exists

Each of these promises used to live inside the command or the finding that met it first,
so each command restated it and each fix changed one copy. One requirement gives every
agreement one address, so a citing site names what it must agree with and a test proves
it ([§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work), [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration)).
