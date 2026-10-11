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

## sections: Which sections a declaration has

### sections.map: Every surface reads one section map

Every coordinate-bearing surface reads the same body-local map ([§FS-show.2.1.2.1](../functional-spec/FS-show.md#2121-the-map-is-the-declarations-body-span)):
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
versions of the target ([§FS-declarations.checks.broken-stub.1](../functional-spec/FS-declarations.md#checksbroken-stub1-the-target-is-read-as-a-save-would-write-it), [§FS-show.2.3.7](../functional-spec/FS-show.md#237-a-stubs-target-is-found-by-its-id)). An unsaved
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

### differences.identity: Identity

- **G03.** A `[citations]` target `api/API` is refused on load as an unknown kind, while
  the rule `Each FS must cite at least one api/API.` parses in the same workspace. R.
- **G04.** One non-citable kind gets three different refusal reasons from `list --kind`,
  `list --selector` and a rule subject. R.

### differences.sections: Sections

- **P01.** In a file with no sections, `show --full` runs a body past a plain `# Appendix`,
  while `list --size` and `refs` end it before. R.
- **P02.** A `/** … */` section without ` * ` is listed by `--toc` and refused by a direct
  query. R.
- **P03.** `/// # 1. Sub` is listed by `--toc` and refused by `grund FS-b.1`. R.
- **P07.** `## 1. One` then a plain `## Notes`: `--full` includes Notes, `refs` gives the
  citations under it no section. R.
- **P10.** `show` includes an adjacent `//` note in a `///` body, `refs` gives it no
  enclosing declaration. R.
- **P12.** In a doc-comment, ownership continues a section past a plain or duplicate
  heading that ends it for `show`. R.
- **P16.** The editor snapshot's span for a C-family section keeps `*/`, the `--toc` title
  does not; the core fields only. R.
- **G05.** With named sections off at the root and on in member `api`, `grund
  api/API-a.terms` answers and an unfiltered `list --selector API.terms` refuses. R.

### differences.stubs: Stubs

- **Placement.** For a healthy stub to a target outside the include, `list` shows the stub
  and `list --size` the target. R.
- **U01.** A citation into a section the outside target declares twice passes `check` and
  is refused by `show`. R.
- **U03a.** Exhaustive `show --batch` omits the outside target's section direct `show`
  returns. R.
- **U03b.** Shell completion offers none of the outside target's sections. C.
- **U03c.** The editor snapshot enumerates none of the outside target's sections. R.
- **U03d.** `list --selector FS.terms` omits the outside target's section `show` returns. R.
- **U04.** Citation obligations are not checked on an outside target, and are on the same
  target scanned. R.
- **U05.** `check` reports an outside RULE target's title missing, which `show --brief`
  reads. R.
- **U06.** `check --only unused` places the warning at the stub, `show` places the
  declaration at the target. R.

### differences.text: Text

- **R04.** A buffer opened under the target's physical path does not move a stub that
  spells it through a symlink; the same edit saved does. R.
- **R05.** A stub to a target that exists only as a buffer is broken, or its ID a
  duplicate, until the same bytes are saved. R.
- **R06a.** An index link edited to a bare citation is clean in the editor and
  `unlinked-index-entry` once saved. R.
- **R06b.** The inline-code hint on a dangling citation differs between buffer and save. R.
- **R06c.** An `AGENTS.md` block edit is judged on the old disk text until saved. R.
- **R06d.** An end-to-end manifest edit is read from disk until saved. R.

### differences.scope: Scope

- **R01.** `check sub` resolves `FS-001-a`, which `grund FS-001-a sub` reports not found.
  R.
- **R02.** In a workspace, `check docs` reports `unknown project alias` for a citation the
  root `check` and `list docs` resolve. R.

## proof: Proved by comparison

The requirement is proved by asking every surface each question on one tree, and each
equivalence on both of its inputs, through each surface's own entrypoint, and by checking
each answer against the point that defines it as well, because two surfaces wrong alike
agree. Today two tests compare part of it: `crates/grund-cli/tests/stub_resolution_agreement.rs`
compares the stub answers of [§REQ-one-answer.stubs](REQ-one-answer.md#stubs-what-a-stub-answers) and [§REQ-one-answer.identity](REQ-one-answer.md#identity-which-declarations-an-id-has), and
`tests/integration/lsp_cli_parity.rs` compares the editor's diagnostics with `check`'s
over the end-to-end corpus ([§REQ-one-answer.text.parity](REQ-one-answer.md#textparity-the-editor-answers-as-the-command-line-does)). The comparison over every
surface and every tree is not built yet; until it is, a section no test compares is held
by review.

## why: Why this exists

Each of these promises used to live inside the command or the finding that met it first,
so each command restated it and each fix changed one copy. One requirement gives every
agreement one address, so a citing site names what it must agree with and a test proves
it ([§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work), [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration)).
