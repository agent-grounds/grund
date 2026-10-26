# The structure that gets cited

From grund 0.19.0, exclude and enabled ignore rules also refuse a stub's target
([§FS-declarations.stubs.target.3](../functional-spec/FS-declarations.md#stubstarget3-a-target-the-scan-does-not-read-declares-nothing)). With `include = ["docs"]`, a stub pointing at
`notes/gen/a.md` is broken if `exclude = ["gen"]`; removing that exclusion keeps
the outside-include target readable. A rejected target lends no body, sections,
chapters or citation edges ([§FS-declarations.stubs.target.5](../functional-spec/FS-declarations.md#stubstarget5-what-a-target-outside-the-include-gives-and-what-stays-unjudged)).

Every fact has a stable ID. The default kinds, all configurable — `*` marks a *place* rather than an ID namespace (`citable = false`: a home, a title and citation rules, no declarations), which is what a test is, and what any directory an agent must be told about can be. See [Citation directions](../user-facing/citation-directions.md) for the complete `[citations]` grammar and its rendered examples:

Repositories can also declare controlled-English constraints over declarations,
named chapters, and citations. The [chapter-rules guide](../user-facing/rules.md)
lists every accepted sentence and refusal rewrite; the runnable
[`examples/rules/`](../../examples/rules/) repository demonstrates the findings and
deduplication behavior ([§FS-rules](../functional-spec/FS-rules.md#fs-rules-grounded-declarations-state-and-enforce-chapter-rules)).
A version-2 kind can declare its chapters as fields — their order, titles and
allowed values, and whether anything else may appear — in the [fields guide](../user-facing/fields.md), with the runnable [`examples/fields/`](../../examples/fields/) ([§FS-fields](../functional-spec/FS-fields.md#fs-fields-a-v2-kind-states-the-shape-of-its-declarations-and-check-holds-every-declaration-to-it)).
A version-2 row may list several places — `folders = ["docs/spec/core", "docs/spec/plugins"]` — and
places may nest: a file listed in a row's `files` is that row's, otherwise the deepest folder
holding it decides, and a file in no place is `code`'s. Each folder of a kind keeps its own index.
The runnable [`examples/places/`](../../examples/places/) shows all three
([§FS-config-v2.schema.places.ownership](../functional-spec/FS-config-v2.md#schemaplacesownership-a-listed-file-then-the-deepest-folder)).

| Kind | What it is | Where it lives |
| --- | --- | --- |
| `GRUND` | Why: project motivation | `docs/grund.md` (one declaration, all of it inline) |
| `GOAL` | Where: project direction and outcomes | `docs/goals.md` (one file, all goals inline) |
| `FS` | What: behavior, requirements, and constraints | `docs/functional-spec/` ([§DF-fs-home-lands-in-0-19](../decisions/functional/DF-fs-home-lands-in-0-19.md#df-fs-home-lands-in-0-19-the-built-in-fs-home-is-docsfunctional-spec-from-0190)) |
| `REQ` † | Hard requirements: what the project must never break | `docs/requirements/` |
| `AR` | How: high-level implementation, structure, and design | `docs/architecture/` — **or inline in a class / module doc-comment** |
| `DF` | product behavior decisions and tradeoffs | `docs/decisions/functional/` (append-only) |
| `DA` | architecture decisions and tradeoffs | `docs/decisions/architectural/` (append-only) |
| `RM`   | planned milestones and sequencing           | `docs/roadmap.md`                              |
| `e2e` * / `integration` * | proof: the spec as a user sees it, and the parts fitting as designed | `tests/e2e/` (must cite `FS`), `tests/integration/` (should cite `AR`) |

† `REQ` is declared by the config `grund init` writes ([§FS-init.2.4.10](../functional-spec/FS-init.md#2410-req-is-declared-in-docsrequirements)) and has no built-in default: a project with no config, or a config that leaves `[[kinds]]` out, has no `REQ`. Before grund 0.19.0 a project with no config homed `FS` in `requirements.md`; such a project now runs `grund init` to write a config that keeps it there, or moves its declarations into `docs/functional-spec/` ([§FS-config.3.4.4](../functional-spec/FS-config.md#344-the-default-kinds)).


**ID format:**

```plaintext
     ┌─────────────────── citation ───────────────────┐
            ┌───────────── ID ───────────────┐
  [§] [alias /] KIND - [number -] slug [.section]
   │     │        │       │         │       │
   │     │        │       │         │       └─ section: .3, .3.1, .format
   │     │        │       │         └───────── slug: [a-z0-9][a-z0-9-]*
   │     │        │       └─────────────────── optional ordinal (e.g., 001)
   │     │        └─────────────────────────── kind: FS, AR, … or your own
   │     └──────────────────────────────────── alias: root, api, …
   └────────────────────────────────────────── citation marker
```

Three schemes are supported. In version 2, `[schema] id_format` selects the
repository default and `[schema.kinds.<KIND>] id_format` can override one kind.
Version 1 uses `[id] format` and `[[kinds]] format` respectively, so
configured per-kind mixing is supported
([§FS-config.3.2](../functional-spec/FS-config.md#32-id--id-grammar)). Each
scheme has a runnable tiny repo under [`examples/`](../../examples/), maintained as a
detailed walkthrough for canonical user workflows
([§FS-examples](../functional-spec/FS-examples.md#fs-examples-examples-teach-canonical-user-workflows)).

| Scheme                                     | Example             | Benefit                                                                                                          | Trade-off                                                                |
|--------------------------------------------|---------------------|------------------------------------------------------------------------------------------------------------------|--------------------------------------------------------------------------|
| `{kind}-{number}-{slug}` *(default)*       | `FS-014-user-login` | Number is stable; a number-only shorthand survives a slug change, while full-ID citations require deliberate updates (and canonical shorthand is reported for rewriting). | Two tokens to type; needs `grund id` to allocate the next number.        |
| `{kind}-{number}` (RFC-style)              | `FS-014`            | Maximally stable — no slug to drift. Familiar from RFCs/PEPs/JEPs/ADRs.                                          | Opaque at the call site: `§FS-014` tells you nothing without resolving it. |
| `{kind}-{slug}` *(`grund` itself uses this)* | `FS-user-login`     | Self-describing — reads like English in prose and code. No number to allocate.                                   | Renaming a slug rewrites every citation. Slug must be unique per kind.   |

Rule of thumb: pick `{kind}-{slug}` until rename churn or ID count starts to hurt; switch to `{kind}-{number}-{slug}` when it does.

Changing that setting does not strand declarations already committed under an
older shape: their exact written IDs and exact marked citations remain readable
across the CLI and editor, while `grund check` points out each mismatch so you
can rename it or restore the matching format. The mismatch is a `check`
error; read compatibility remains
([§FS-config.3.2](../functional-spec/FS-config.md#32-id--id-grammar)).

A citation is the marker `§`, the ID, and an optional `.<section>` — with the target project's alias in front when the repo is a workspace:

```
§FS-user-login.3.1        # section 3.1 of FS-user-login
§api/FS-user-login.3.1    # the same section, in the `api` project of a workspace
```

Type `$$` in a `grund`-aware editor and it's rewritten to `§` automatically. Both marker and trigger are configurable in `grund.toml`.

With the default `{kind}-{number}-{slug}` scheme, a persisted shorthand such as
`§FS-042` is an error and `grund fmt --write` expands it to the descriptive full
ID. A project that deliberately wants both spellings may opt in
([§FS-config.3.1](../functional-spec/FS-config.md#31-reference--citation-form)):

```toml
[reference]
shorthand = "accepted" # default: "canonical"
```

Then `§FS-042` and `§FS-042-user-login` resolve as the same citation, and
formatting preserves whichever marker form the author wrote. Typed trigger input
remains canonicalizing: `$$FS-042` still becomes `§FS-042-user-login`
([§FS-fmt.2.4](../functional-spec/FS-fmt.md#24-shorthand-to-canonical)). The
tradeoff is permanent mixed-form drift while the policy is enabled: searching by
the number finds both forms, but searching by the slug misses shorthand sites,
and the short form is opaque until resolved.

The marker is the whole signal: a `§`-prefixed token is a live, checked citation wherever it appears — including inside Markdown backticks — except in a simple top-level Python assignment whose value is triple-quoted runtime data ([§FS-check.1.1.3.1](../functional-spec/FS-check.md#1131-assigned-python-triple-quoted-data)). To show an *example* ID that shouldn't resolve, write it without the marker (`FS-user-login`), inside a fenced code block (which is how the two citations above are written), or with the marker bracketed (`<§>FS-user-login`) — the escape `grund check` names in its own hint when a citation resolves to nothing, and the one form that is inert under both strict modes ([§FS-check.1.1.9](../functional-spec/FS-check.md#119-an-id-in-an-escape-position)). Put an intentional citation near assigned Python data in a `#` comment or a real docstring.

**Write section citations with their full ID.** This input inside an
`FS-check` declaration body contains a live local section citation:

```text
See §2.1.
```

When section 2.1 exists, the citation remains navigable, but `grund check` reports:

```text
local section citation §2.1; write §FS-check.2.1 — unchecked in grund 0.13.1, an error in 0.14.0; run `grund fmt --write`
```

Every form of the finding ends by naming the two releases its verdict moved between, and
an owned site the formatter would actually write also names the command that clears it
([§FS-check.3.24.1](../functional-spec/FS-check.md#3241-the-release-attribution-and-where-the-command-clause-is-withheld)) — so one line separates *this tree predates the binary running over
it* from *this citation is wrong*. `grund fmt --write` expands safe owned sites;
protected sites need manual replacement and are not offered the command.
Where the owner lacks the section, the finding names the absence and offers a full citation or the escape instead ([§FS-check.3.24.3](../functional-spec/FS-check.md#3243-an-absent-target-section-is-answered-with-the-escape)), the ordinary missing-section error
fires beside it, and the formatter leaves that site alone and does not offer the command ([§FS-fmt.2.4.6](../functional-spec/FS-fmt.md#246-an-absent-target-section-withholds-this-rewrite-and-only-this-one)). A site outside
a declaration is diagnosed without a guessed target and needs a full citation or an
escape: `<§>2.1` is an inert illustration, with no citation diagnostic or navigation.
This is intentionally newly loud compatibility behavior for a
form that older releases silently skipped ([§FS-check.3.24](../functional-spec/FS-check.md#324-declaration-local-section-citation), [§FS-fmt.2.4](../functional-spec/FS-fmt.md#24-shorthand-to-canonical)).

**Specs can live inline in source.** Declare the spec in a class or module doc-comment, then enroll it from the configured kind index with the canonical bare-ID link `grund fmt --cross-refs` writes — no stub file is required:

```rust
/// AR-event-bus: In-process event broadcaster
///
/// ## 1. Topology
pub struct EventBus { /* … */ }
// Kind index: - [§AR-event-bus](../../src/bus.rs)
```
`grund AR-event-bus` reads the source declaration directly, strips the `///` markers, and prints the Rustdoc prose. The same goes for Javadoc, JSDoc, Python docstrings, Go doc blocks, KDoc, Doxygen — every comment form enumerated in `grund`'s scanner spec. A block comment declares with or without the ` * ` gutter, behind `/*`, `/**` or `/*!`, the one-line `/** AR-event-bus: Event bus */` included, once its `*/` is found ([§FS-declarations.line.block-comment](../functional-spec/FS-declarations.md#lineblock-comment-a-line-inside-a-block-comment)). A one-line Markdown stub remains supported when a separate pointer file is useful.

Only that canonical link enrolls a declaration that lives in source. A link you wrote around the ID and a title is an entry for a declaration in the kind's folder, but it enrolls nothing, and the index's citation of a source declaration through it is an ordinary citation ([§FS-check.3.18.3](../functional-spec/FS-check.md#3183-an-external-source-declaration-enrolls-by-its-canonical-link)).

A stub is a heading whose title is one link, `# AR-event-bus: [bus](../../src/bus.rs)`, at any heading level ([§FS-declarations.stubs.shape](../functional-spec/FS-declarations.md#stubsshape-what-a-stub-looks-like)). It is a stub only in a Markdown file of some kind's home, wherever the checkout sits; a heading of that shape anywhere else is a declaration titled with a link. Before grund 0.19.0 any Markdown file with a directory named `docs` on its path held stubs, and 0.18.0 warned as `stub-shape` at every heading the two readings tell apart ([§FS-declarations.checks.stub-shape](../functional-spec/FS-declarations.md#checksstub-shape-stub-shape-read-differently-from-grund-0190)). `grund list` shows a stubbed declaration at its target, with the target's title. The target need not be scanned: one outside `[scan] include` still gives the stub its declaration, its sections, its citations and its chapters, so rules, `refs` and `unused` read it as if it were scanned, and a section it declares twice is reported where it is written; nothing else about the target file is judged ([§FS-declarations.stubs.target.5](../functional-spec/FS-declarations.md#stubstarget5-what-a-target-outside-the-include-gives-and-what-stays-unjudged)). The target must declare the ID itself: a stub there declares nothing, so a chain of stubs is broken ([§FS-declarations.stubs.target.4](../functional-spec/FS-declarations.md#stubstarget4-a-stub-in-the-target-declares-nothing)).

If `src/bus.rs` moves to `src/events/bus.rs`, update the stub in
`docs/architecture/AR-event-bus.md` from:

```markdown
# AR-event-bus: [bus](../../src/bus.rs)
```

to:

```markdown
# AR-event-bus: [bus](../../src/events/bus.rs)
```

While the old target is missing, `check` still reports both `duplicate` and
`broken-stub`. The duplicate finding identifies the broken stub, its old link,
and the surviving declaration at `src/events/bus.rs:1` (if declared on line 1),
then says to point the stub link there. The broken-stub finding gives the same
action and refers to the duplicate declaration finding for the location; that
reference remains when `--only broken-stub` hides the companion. Ambiguous ID
queries also explain the located repair. If several real homes exist, the
guidance lists them without choosing one.
`grund refs AR-event-bus` does not refuse while you plan the repair: the broken stub
is no home for it, so it lists every citation — the sites that follow the move — and
names the stub in one `warning:` line carrying the same repair
([§FS-refs.4.1.1](../functional-spec/FS-refs.md#411-a-broken-stub-beside-a-real-home-is-set-aside-with-a-warning)).
If the sole real declaration is in a missing stub's own file, the guidance instead
explains that linking to that same file will not pair the stub: it remains a duplicate
([§FS-declarations.stubs.repair](../functional-spec/FS-declarations.md#stubsrepair-a-missing-target-beside-another-declaration-explains-the-repair)).

**Where a body, a section and a comment block end.** In Markdown a declaration's body runs to the next heading as high as its own or higher, whether or not that heading declares an ID. In source it is the comment block the declaration line opens, which ends where the comment does, or early at the next declaration line ([§FS-declarations.body](../functional-spec/FS-declarations.md#body-where-a-declarations-body-ends), [§FS-languages.comment-block](../functional-spec/FS-languages.md#comment-block-the-comment-block)). A section runs from its heading down to the next heading as deep or shallower, numbered or plain ([§FS-declarations.section](../functional-spec/FS-declarations.md#section-what-a-section-is)). Inside a doc-comment the depth counts from the declaration, as it does in Markdown: under `/// AR-event-bus:` above, `/// ## 1. Topology` is section 1, and `/// # 1. Topology` is a heading as high as the declaration, so it is no section ([§FS-show.2.3.3](../functional-spec/FS-show.md#233-section-selection-inside-a-doc-comment)). In a `/** … */` block, write every line behind ` * `, a heading's too — ` * ## 1. Topology`. A line without it is read as written, so its first `#` is no comment marker ([§FS-languages.comment-wrapper.headings](../functional-spec/FS-languages.md#comment-wrapperheadings-the-wrapper-of-a-heading-line)).

Every command reads these rules from one parse of each file since grund 0.19.0 ([§DF-one-parse-answers-from-0-19](../decisions/functional/DF-one-parse-answers-from-0-19.md#df-one-parse-answers-from-0-19-one-parses-other-answers-ship-in-0190-after-a-release-that-announces-them-and-warns-where-a-verdict-moves)). grund 0.18.0 still gave 0.17's answer where they moved one and warned `parse-moves` at each line where that moved a `check` verdict; the warning went with the old answers.

`grund` does this itself: [§AR-checker](../../crates/grund-core/src/checker/report.rs) lives only in the doc-comment of `fn check` in [`crates/grund-core/src/checker/report.rs`](../../crates/grund-core/src/checker/report.rs), and its canonical row in [`docs/architecture/README.md`](../architecture/README.md) enrolls it without a stub — `grund AR-checker` prints the source prose.

## Section headings

A heading inside a declaration body is a citable section. Its path is written in the heading, numbered (`## 1. Overview`, `### 1.1 Scope`) or, with `[id] named_sections = true`, named (`## format: Formatting`, `### format.1: Username`). Its depth must match the path, one level per component ([§FS-config.3.3](../functional-spec/FS-config.md#33-section-paths--arbitrary-nesting-depth)).

With named sections on, a name may also follow a number in a Markdown declaration body: `### 1.goals: Goals` under `## 1. Overview` declares `FS-login.1.goals` ([§FS-config.3.3.1](../functional-spec/FS-config.md#331-named-components)). Its address holds the number of the chapter above it, so renumbering that chapter moves it. A cited one is therefore recorded in the section lock described below, as a relative one is ([§FS-section-lock.holds.entry](../functional-spec/FS-section-lock.md#holdsentry-one-line-per-locked-address)).

From grund 0.19.0, a marked citation of that address fails the check if its
section or declaration is absent, just like any other broken reference. The
0.18.0 warning window has ended ([§FS-citations.checks.mixed-path](../functional-spec/FS-citations.md#checksmixed-path-a-citation-of-a-name-under-a-number-is-checked-and-warns-until-0190)).
After adding a resolving citation, run `grund fmt --write` with no path at the
workspace root, or at the project root outside a workspace, and commit the lock
([§FS-section-lock.checks.remedy](../functional-spec/FS-section-lock.md#checksremedy-the-run-that-writes-the-lock));
an unrecorded mixed address still produces an immediate
`unrecorded-section-citation` error ([§FS-section-lock.checks.unrecorded-section-citation](../functional-spec/FS-section-lock.md#checksunrecorded-section-citation-a-locked-address-the-lock-does-not-hold)).

A deep path repeats its parent in every heading below it. A **relative heading** writes only the component it adds: a leading `.`, the component, and the same title syntax as the complete form ([§FS-declarations.section.relative](../functional-spec/FS-declarations.md#sectionrelative-a-heading-may-write-only-the-component-it-adds)):

```markdown
# FS-login: Login

## format: Formatting

### .1: Username
```

`### .1:` takes its parent's path and declares `FS-login.format.1`. The parent is the nearest shallower heading, and it must sit exactly one level up. It is the declaration heading or a section that is neither an orphan nor a duplicate. Relative headings chain, so `#### .1:` under `### .user:` under `## format:` is `format.user.1`. A relative heading with no such parent declares nothing, and `check` reports `relative-section-parent` with the parent it needed. Every place that prints a coordinate uses the full path: citations, `refs`, `list`, completion, JSON, diagnostics, `--toc` and the editor hover. A read of the section itself opens with its own heading at the full path, so `grund FS-login.format.1` prints `### format.1: Username` in every mode, `md` and the JSON `body` ([§FS-declarations.section.relative.shown.selected](../functional-spec/FS-declarations.md#sectionrelativeshownselected-a-selected-sections-own-heading-names-its-path)). Every other heading of the read, those inside the slice and those of a whole-declaration read, prints as written, and the anchor stays the written heading's. The form works in Markdown declaration bodies and in source doc-comment declaration bodies alike: `/// ### .1: Username` under `/// ## format: Formatting` declares `FS-login.format.1`, and so does a name under a number, complete like `/// ### 1.goals: Goals` or relative like `/// ### .goals: Goals` under `/// ## 1. Overview` ([§FS-declarations.section.relative.where](../functional-spec/FS-declarations.md#sectionrelativewhere-markdown-declaration-bodies-and-doc-comments-from-0190)). grund 0.18.0 read these doc-comment lines as prose and warned `parse-moves`; from 0.19.0 they are sections.

A relative address depends on where its block sits. Move `### .1:` from `## format:` to `## parse:` and the same words answer to `FS-login.parse.1`, while a citation of `FS-login.format.1` now reads other text. So each cited relative section is recorded in a committed **section lock**, `.grund/sections/<ID>.lock`. A lock line holds the address and a fingerprint of the section's text ([§FS-section-lock](../functional-spec/FS-section-lock.md#fs-section-lock-a-cited-relative-section-keeps-its-text-or-check-says-where-the-text-went)):

```text
FS-login.format.1 8f332b6fe87bb346
```

A citation into another workspace member, of `FS-login.format.1` in the member `api`, is recorded by the project that holds it, in `.grund/sections/api/FS-login.lock` ([§FS-section-lock.file.members](../functional-spec/FS-section-lock.md#filemembers-citations-into-another-project)). After citing a relative section, run `grund fmt --write` with no path at the workspace root, or at the project root outside a workspace, and commit what it writes. Nothing else writes the lock, and `fmt` never rewrites a heading. `check` reports three things. `unrecorded-section-citation` is a cited address with no entry. `section-address-changed` is a cited text that now sits at another address, with both addresses named and every line that still cites the old one. `invalid-section-lock` is a lock line `check` cannot read. To accept a move, leave a pointer in the old section that cites the new address, or rewrite the citations. The lock has one false positive. If you copy a cited section to a new address and then edit the original, the recorded text is found only at the copy, so `check` reports a move. The way out is to edit the copy as well, or to cite the copy from the original ([§FS-section-lock.move.copy](../functional-spec/FS-section-lock.md#movecopy-copy-then-edit-the-original)).

The lock has one gap. A section moved and edited in the same change matches no recorded fingerprint, so the lock reads it as an edit in place. `check` passes, and the only trace is the refreshed line in the lock diff ([§FS-section-lock.limits.edit-and-move](../functional-spec/FS-section-lock.md#limitsedit-and-move-a-section-moved-and-edited-in-one-change)). Where an address must never move, write its complete path: complete-path headings need no lock, unless the path puts a name under a number. The runnable [`examples/relative-sections/`](../../examples/relative-sections/) prints the full path each relative heading resolves to, and [`examples/relative-sections-lock/`](../../examples/relative-sections-lock/) shows the lock catching an unrecorded citation.

## Maintaining declarations

Before renaming or moving a declaration, run `grund refs <ID>` to find affected
citations. Edit the declaration and affected authored citation IDs yourself, then
run these commands from the config/project root or workspace root
([§FS-init.2.3.4.9](../functional-spec/FS-init.md#2349-declaration-blast-radius)):

```bash
grund fmt --write    # after the authored rename or move
grund check
grund fmt --check
```

When cross-reference emission applies, fmt re-derives generated Markdown link
destinations and anchors ([§FS-fmt.6.3](../functional-spec/FS-fmt.md#63-idempotency-and-re-derive)).
The root-scoped write also updates eligible section locks; commit those changes
with the declaration and citations. Locks are keyed by declaration ID, so moving
only the file leaves its lock name alone. Do not hand-edit locks: fmt does not
silently approve a protected section move. Fix the citations or leave a pointer
at the old address first ([§FS-section-lock.write](../functional-spec/FS-section-lock.md#write-who-writes-the-lock)).
When `check` reports a moved section, `grund config explain section-address-changed`
prints the three ways out and why a lock edit is not one
([§FS-config-explain.4](../functional-spec/FS-config-explain.md#4-a-codes-answer)).
fmt does not replace authored IDs or repair arbitrary URLs. Try the rename on a
copy with [`examples/relative-sections-lock/`](../../examples/relative-sections-lock/).

### Reading fmt's report

Read the preview before you write, and describe the change from the write's
report. A preview row names every kind of rewrite its line gets: the first kind
leads, with the text an expansion writes, and the others follow
([§FS-fmt.3.5.1](../functional-spec/FS-fmt.md#351-a-line-with-several-rewrites-carries-the-first-label)). Under `--marker` a Markdown line is marked and then linked:

```text
$ grund fmt --marker
docs/runbook.md:3: bare → marker, markdown link
src/inventory/reserve.py:3: bare → marker
$ grund fmt --marker --write
rewrote 2 lines: 2 citations marked, 1 link written
  docs/runbook.md (1)
  src/inventory/reserve.py (1)
```

The totals count every occurrence, so a citation marked and then linked counts
once in each, and a link counts only when it is new or points somewhere new
([§FS-fmt.3.7](../functional-spec/FS-fmt.md#37-the---write-report)). After a rename, the lock file of the old ID is deleted and
the lock file of the new one is created, and the report says so instead of
counting lines in each
([§FS-section-lock.write.report](../functional-spec/FS-section-lock.md#writereport-what-fmt-reports)):

```text
$ grund fmt
docs/notes.md:1: markdown link
.grund/sections/FS-login.lock:1: section lock: drop FS-login.code.1; deletes the file
.grund/sections/FS-sign-in.lock:1: section lock: add FS-sign-in.code.1; creates the file
$ grund fmt --write
rewrote 3 lines: 1 link written, 2 section-lock entries
  .grund/sections/FS-login.lock (1) deleted
  .grund/sections/FS-sign-in.lock (1) created
  docs/notes.md (1)
```

Run both on a copy with [`examples/fmt-reporting/`](../../examples/fmt-reporting/).

## Schema and links: the rules a repository is held to

`grund` reads a repository as a graph. Every scan turns the tree into one
snapshot of facts, and every rule, whether written in `grund.toml` or as a
sentence, is a question asked of that snapshot. There are two kinds of rule.
**Schema rules** say what a fact may look like and where it may live, which
decides whether it enters the graph at all. **Link rules** say which citations
must, should, or must not exist between the facts once they are there.
`grund schema` prints both kinds for one repository, read from its configuration
alone ([seeing a repository's schema](schema.md)).

### What a scan builds

Underneath, the snapshot is five relations ([§FS-rules.5.1](../functional-spec/FS-rules.md#51-facts-and-identity)):

| Relation | One row per | Example |
| --- | --- | --- |
| `decl(node, kind)` | declaration | `FS-login` is an `FS` |
| `chapter(node, path, display_name)` | named chapter in a declaration's body | `FS-login.terms`, labelled `Terms` |
| `contains(parent, child)` | chapter inside its declaration or parent chapter | `FS-login` contains `FS-login.terms` |
| `cites(site, from, target)` | citation that resolves | a citation of `FS-login.3` at `auth.rs:42`, from `AR-auth` to `FS-login` |
| `site_in(site, unit)` | unit that contains a citation site | that site is in `AR-auth` |

A citation of a numbered section counts as a citation of its nearest named
chapter, or else of the declaration. A citation that does not resolve adds no
`cites` row; it gets its own finding instead. Every declaration and chapter has
its row whether or not it cites anything, so "each FS must …" means every FS.
The meaning is specified as Datalog over that one complete snapshot. The
implementation is a hand-written evaluator, not a Datalog engine.

Citation directions and chapter rules are also rendered into the agent
instructions `grund init` writes. An agent therefore reads a rule before it
writes, and `grund check` holds it to the rule after.

### Schema rules: what a fact may look like

| Rule | You write | What it means in the graph | `grund check` reports |
| --- | --- | --- | --- |
| Kinds and homes | a `[[kinds]]` row: `kind`, and `file` or `folder` ([§FS-config.3.4](../functional-spec/FS-config.md#34-kinds--recognized-kinds)) | the node types, and where a node of each type may be declared | `misplaced-declaration`, `duplicate`, `declaration-near-miss` |
| ID grammar | `[id] format` and its patterns, optionally per kind ([§FS-config.3.2](../functional-spec/FS-config.md#32-id--id-grammar)) | the address space: one spelling per node | a format mismatch, as an error |
| Sections | numbered `## 1. …` or named `## terms: Terms` headings ([§FS-declarations.section](../functional-spec/FS-declarations.md#section-what-a-section-is)) | child coordinates `<ID>.<path>`, and their `contains` rows | `section-heading-level`, `duplicate-section`, `section-outside-declaration`, `orphan-section`, `unmarked-heading` |
| Inline declarations and stubs | a declaration line in a doc-comment, or a stub heading whose title is one link | a node whose body lives in source | `broken-stub`, `stub-shape` |
| Declaration bodies | a body with no merge conflict left in it ([§FS-declarations.body.conflict](../functional-spec/FS-declarations.md#bodyconflict-a-body-still-in-the-middle-of-a-merge)) | each node has one text, so a citation and a section-lock fingerprint each read one | `conflict-marker` |
| Kind index | a folder kind's `README.md` ([§FS-check.3.18](../functional-spec/FS-check.md#318-declaration-missing-from-its-kinds-index)) | every node of the kind is listed there, as a link: the one `fmt` writes, or one you wrote whose text holds the citation ([§FS-check.3.18.5](../functional-spec/FS-check.md#3185-what-an-entry-is)) | `missing-index-entry`, `unlinked-index-entry` |
| Fields *(config v2)* | `fields` on a kind ([guide](fields.md), [§FS-fields](../functional-spec/FS-fields.md#fs-fields-a-v2-kind-states-the-shape-of-its-declarations-and-check-holds-every-declaration-to-it)) | which chapters each node carries, in what order, with what values | `missing-field`, `mistitled-field`, `misplaced-field`, `invalid-field-value`, `undeclared-chapter` |
| Values | a numbered component in an opted-in kind ([guide](values.md), [§FS-values](../functional-spec/FS-values.md#fs-values-opted-in-kinds-bind-authored-components-to-one-declared-value)) | one node is the source of a value, and every use is compared with it | `invalid-value-declaration`, `invalid-value-binding`, `value-mismatch` |
| Lead size *(opt-in)* | `[reference] lead_size_warning` ([guide](coordinate-sizes.md)) | what a default `grund <ID>` read costs | `oversized-lead` |

A marked kind-index entry also follows citation directions. If it is forbidden,
keep the entry and use the finding's rule repair rather than removing the
marker ([§FS-check.3.12.2](../functional-spec/FS-check.md#3122-repair-advice-preserves-a-required-index-entry),
[guide](citation-directions.md)).

### Link rules: which citations must exist

| Rule | You write | What it means in the graph | `grund check` reports |
| --- | --- | --- | --- |
| Every citation resolves | `§<ID>[.<section>]`, or `§<alias>/<ID>` in a workspace ([§FS-check.3.1](../functional-spec/FS-check.md#31-dangling-citation)) | an edge exists only if its target node does | `dangling`, `missing-section`, `unknown-project`, `glob-citation`, and with `--full` the `out-of-scope-*` codes |
| One spelling per edge | `[reference] shorthand` and the inline-note limits ([§FS-config.3.1](../functional-spec/FS-config.md#31-reference--citation-form)) | the same edge is written the same way everywhere | `shorthand-citation`, `shorthand-numeric-run`, `local-section-citation`, `inline-citation-style` |
| Citation directions | `[citations.<KIND>]` with `must`, `should`, `should-not`, `must-not` ([guide](citation-directions.md), [§FS-config.3.9](../functional-spec/FS-config.md#39-citations--citation-direction-rules)) | per declaration, at least one `cites` row into the named kinds; per site, no `cites` row into a forbidden one | `missing-citation`, `forbidden-citation`, and with `--suggestions` `suggested-citation`, `discouraged-citation` |
| Grounding *(opt-in)* | `require_grounding` and `grounding_level`, in `[reference]` or on a kind ([§FS-check.3.6](../functional-spec/FS-check.md#36-ungrounded-unit-opt-in)) | every file, or every unit in it, has at least one outgoing `cites` row | `ungrounded` |
| Chapter rules | a declaration of a kind with `rules = true`, whose title is one sentence ([guide](rules.md), [`examples/rules/`](../../examples/rules/), [§FS-rules.1](../functional-spec/FS-rules.md#1-rule-declarations-and-opt-in)) | a count over `chapter` or `cites` rows, for every unit the sentence's subject selects | `chapter-cardinality`, `citation-cardinality`, `uncited-unit`, `unreached-declaration`, `invalid-rule` |

A chapter rule is a sentence from one of five families ([§FS-rules.3](../functional-spec/FS-rules.md#3-the-five-sentence-families)):

- **Chapter presence:** `Each FS must have exactly one Terms chapter.`
- **Outbound count:** `The requirements chapter of each FS must cite at least one REQ.`
- **Per-target coverage:** `Each RM must cite each GOAL at least once.`
- **Inbound count:** `Each GOAL should be cited by at least one FS.`
- **Prohibition:** `Each FS must not cite any AR.`

A citation direction and a rule that say the same thing are one constraint and
produce one finding ([§FS-rules.6](../functional-spec/FS-rules.md#6-semantic-deduplication)). A `must` is an error and fails the run with
exit `1`. A `should` or `should-not` goes to the suggestions channel, which
`--suggestions` shows ([§FS-check.2.3](../functional-spec/FS-check.md#23-suggestions-channel-opt-in)). Warnings never fail a run. The code
catalog lists every finding code, its severity, and the setting that turns it on
([§FS-errors.5.5](../functional-spec/FS-errors.md#55-the-check-code-catalog)).

### Linking: what grund does

- **One citation, every language.** A citation is read by the same resolver in
  Markdown and in every doc-comment form: Rustdoc, Javadoc, Python docstrings,
  Go, JSDoc and the rest ([§GOAL-polyglot-citation](../goals.md#goal-polyglot-citation-ids-cite-cleanly-from-anywhere-they-are-useful)). It names an ID, not a path,
  so it survives a moved file and a reworded heading.
- **Clickable in Markdown.** In a `.md` file, `grund fmt --write` wraps each
  citation in a link to its declaration. It computes the anchor exactly as
  GitHub does ([§DF-github-anchor-fidelity](../decisions/functional/DF-github-anchor-fidelity.md#df-github-anchor-fidelity-the-github-anchor-profile-reproduces-github-slugger-exactly)) and rewrites the link when the target
  moves or its heading changes. Generated configs turn this on with
  `[fmt.cross_refs]`, and a kind's index is always linked ([§FS-fmt.6](../functional-spec/FS-fmt.md#6-cross-reference-emission)). Source
  comments keep the bare citation, because their languages do not render
  Markdown. `grund fmt --check` reports a stale link as a pending rewrite.

  ```markdown
  [§FS-login.3](../functional-spec/FS-login.md#3-session-timeout)
  ```

  That link, whose text is the citation alone, is `fmt`'s to write and keep
  current. A link you wrote around a citation and more text is yours, and `fmt`
  leaves it byte for byte, in a kind's index as anywhere: it does not wrap the
  citation inside it, which would put a link inside your link, and it does not
  touch your destination ([§FS-citations.wrap.enclosing](../functional-spec/FS-citations.md#wrapenclosing-a-citation-inside-a-link-its-author-wrote-is-left-to-the-author)). The citation
  in it is still resolved and checked like any other.

  ```markdown
  See [the §FS-login.3 timeout rule](../functional-spec/FS-login.md).
  ```

  Every form a link is written in counts: an inline link whose text runs onto
  the next line of its paragraph, a reference link `[text][key]`, a collapsed
  `[text][]` or shortcut `[text]` reference, a definition `[key]: url`, an
  autolink `<https://…>` and a raw HTML `<a>` element ([§FS-citations.wrap.enclosing.scope](../functional-spec/FS-citations.md#wrapenclosingscope-every-form-a-link-is-written-in)).
  A reference key, and every byte of a raw HTML tag, is yours too: no pass writes
  into one, so a collapsed reference and its definition keep spelling the same
  key ([§FS-citations.zones.authored-markup](../functional-spec/FS-citations.md#zonesauthored-markup-a-reference-key-and-an-html-tag-keep-their-bytes)). A citation that is an image's whole
  description, `![<§>FS-login](diagram.png)`, keeps its destination; one in a
  longer description is wrapped as prose is ([§FS-citations.wrap.enclosing.images](../functional-spec/FS-citations.md#wrapenclosingimages-an-image-is-not-a-link)).
  The [authored-citations example](../../examples/authored-citations/) shows each
  form. A line an older `grund` already nested, `[[§…](…): title](…)`, keeps
  its inner link and is left as it stands: search for `[[§` to find one, and
  take the outer brackets out by hand ([§DF-fmt-preserves-authored-markup](../decisions/functional/DF-fmt-preserves-authored-markup.md#df-fmt-preserves-authored-markup-fmt-leaves-a-citation-in-every-link-form-its-author-wrote-and-in-an-image-and-check-credits-only-the-ones-that-go-somewhere)).

- **Typing and expanding.** `$$` becomes `§`, and a persisted number-only
  shorthand is expanded to the full ID.
- **Following an edge backwards.** `grund refs <ID>` lists every citation of an
  ID. Run it before renaming or moving a declaration, then follow the
  [post-rename maintenance workflow](#maintaining-declarations).
- **In the editor and the terminal.** The language server jumps from a citation
  to its declaration, shows the section map on hover, lists references, and
  turns citations into document links ([LSP guide](lsp.md), [§FS-lsp](../functional-spec/FS-lsp.md#fs-lsp-grund-ships-an-optional-lsp-server)).
  `grund integrations <client> --write` makes a citation clickable in your
  terminal ([guide](clickable-citations.md), [§FS-integrations](../functional-spec/FS-integrations.md#fs-integrations-grund-prints-and-installs-its-rendering-layer-integrations)).
- **In agents' output.** The agent instructions say how to link a citation in
  pull request bodies, reviews and conversations.

Ordinary links, such as URLs and relative paths without a `§`, are not
`grund`'s to check: see [what it does not check](checking.md#what-it-does-not-check).
