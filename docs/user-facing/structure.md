# The structure that gets cited

Every fact has a stable ID. The default kinds, all configurable — `*` marks a *place* rather than an ID namespace (`citable = false`: a home, a title and citation rules, no declarations), which is what a test is, and what any directory an agent must be told about can be. See [Citation directions](../user-facing/citation-directions.md) for the complete `[citations]` grammar and its rendered examples:

Repositories can also declare controlled-English constraints over declarations,
named chapters, and citations. The [chapter-rules guide](../user-facing/rules.md)
lists every accepted sentence and refusal rewrite; the runnable
[`examples/rules/`](../../examples/rules/) repository demonstrates the findings and
deduplication behavior ([§FS-rules](../functional-spec/FS-rules.md#fs-rules-grounded-declarations-state-and-enforce-chapter-rules)).
A version-2 kind can declare its chapters as fields — their order, titles and
allowed values, and whether anything else may appear — in the [fields guide](../user-facing/fields.md), with the runnable [`examples/fields/`](../../examples/fields/) ([§FS-fields](../functional-spec/FS-fields.md#fs-fields-a-v2-kind-states-the-shape-of-its-declarations-and-check-holds-every-declaration-to-it)).

| Kind | What it is | Where it lives |
| --- | --- | --- |
| `GRUND` | Why: project motivation | `docs/grund.md` (one declaration, all of it inline) |
| `GOAL` | Where: project direction and outcomes | `docs/goals.md` (one file, all goals inline) |
| `FS` | What: behavior, requirements, and constraints | `requirements.md` |
| `AR` | How: high-level implementation, structure, and design | `docs/architecture/` — **or inline in a class / module doc-comment** |
| `DF` | product behavior decisions and tradeoffs | `docs/decisions/functional/` (append-only) |
| `DA` | architecture decisions and tradeoffs | `docs/decisions/architectural/` (append-only) |
| `RM`   | planned milestones and sequencing           | `docs/roadmap.md`                              |
| `e2e` * / `integration` * | proof: the spec as a user sees it, and the parts fitting as designed | `tests/e2e/` (must cite `FS`), `tests/integration/` (should cite `AR`) |


**ID format:**

```plaintext
     ┌─────────────────── citation ───────────────────┐
            ┌───────────── ID ───────────────┐
  [§] [alias /] KIND - [number -] slug [.section]
   │     │        │       │         │       │
   │     │        │       │         │       └─ dotted path of arbitrary depth (.3, .3.1, …)
   │     │        │       │         └───────── [a-z0-9][a-z0-9-]*  (default slug_pattern)
   │     │        │       └─────────────────── optional ordinal (e.g., 001)
   │     │        └─────────────────────────── GRUND│GOAL|FS│AR│DF│DA│RM│[custom]
   │     └──────────────────────────────────── project alias for subprojects or monorepo
   └────────────────────────────────────────── citation marker (writing only)
```

Three schemes are supported. `[id].format` selects the repository default; an
explicit `[[kinds]].format` may give one kind a different stable scheme, so
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
`grund AR-event-bus` reads the source declaration directly, strips the `///` markers, and prints the Rustdoc prose. The same goes for Javadoc, JSDoc, Python docstrings, Go doc blocks, KDoc, Doxygen — every comment form enumerated in `grund`'s scanner spec. A one-line Markdown stub remains supported when a separate pointer file is useful.

A stub is a heading whose title is one link, `# AR-event-bus: [bus](../../src/bus.rs)`, at any heading level ([§FS-declarations.stubs.shape](../functional-spec/FS-declarations.md#stubsshape-what-a-stub-looks-like)). From grund 0.19.0 it is read as a stub only in a Markdown file of some kind's home; until then it is read as one in any Markdown file with a directory named `docs` on its path, and `grund check` warns as `stub-shape` at every heading the two readings tell apart ([§FS-declarations.checks.stub-shape](../functional-spec/FS-declarations.md#checksstub-shape-stub-shape-read-differently-from-grund-0190)). The target need not be scanned: one outside `[scan] include` still gives the stub its declaration and its sections, and a section it declares twice is reported where it is written ([§FS-declarations.stubs.target.5](../functional-spec/FS-declarations.md#stubstarget5-what-a-target-outside-the-include-gives-and-what-stays-unjudged)). The target must declare the ID itself: a stub there declares nothing, so a chain of stubs is broken ([§FS-declarations.stubs.target.4](../functional-spec/FS-declarations.md#stubstarget4-a-stub-in-the-target-declares-nothing)).

**Where a body, a section and a comment block end.** In Markdown a declaration's body runs to the next heading as high as its own or higher, whether or not that heading declares an ID. In source it is the comment block the declaration line opens, which ends where the comment does, or early at the next declaration line ([§FS-declarations.body](../functional-spec/FS-declarations.md#body-where-a-declarations-body-ends), [§FS-languages.comment-block](../functional-spec/FS-languages.md#comment-block-the-comment-block)). A section runs from its heading down to the next heading as deep or shallower, numbered or plain ([§FS-declarations.section](../functional-spec/FS-declarations.md#section-what-a-section-is)). Inside a doc-comment the depth counts from the declaration, as it does in Markdown: under `/// AR-event-bus:` above, `/// ## 1. Topology` is section 1, and `/// # 1. Topology` is a heading as high as the declaration, so it is no section ([§FS-show.2.3.3](../functional-spec/FS-show.md#233-section-selection-inside-a-doc-comment)). In a `/** … */` block, write every line behind ` * `, a heading's too — ` * ## 1. Topology`. A line without it is read as written, so its first `#` is no comment marker ([§FS-languages.comment-wrapper.headings](../functional-spec/FS-languages.md#comment-wrapperheadings-the-wrapper-of-a-heading-line)).

grund 0.18.0 still gives 0.17's answer where these rules move one, and from 0.19.0 every command reads them ([§DF-one-parse-answers-from-0-19](../decisions/functional/DF-one-parse-answers-from-0-19.md#df-one-parse-answers-from-0-19-one-parses-other-answers-ship-in-0190-after-a-release-that-announces-them-and-warns-where-a-verdict-moves)). Where that would move a `check` verdict, 0.18.0 warns `parse-moves` at the line that moves it and names the edit that means the same in both releases ([§FS-declarations.checks.parse-moves](../functional-spec/FS-declarations.md#checksparse-moves-a-verdict-that-moves-in-0190)): a doc-comment heading that will end a section above a citation, a `/** … */` line without ` * ` that will become a section, and a heading that will stop cutting a lead past its budget.

`grund` does this itself: [§AR-checker](../../crates/grund-core/src/checker/report.rs) lives only in the doc-comment of `fn check` in [`crates/grund-core/src/checker/report.rs`](../../crates/grund-core/src/checker/report.rs), and its canonical row in [`docs/architecture/README.md`](../architecture/README.md) enrolls it without a stub — `grund AR-checker` prints the source prose.
