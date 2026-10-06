# The structure that gets cited

Every fact has a stable ID. The default kinds, all configurable — `*` marks a *place* rather than an ID namespace (`citable = false`: a home, a title and citation rules, no declarations), which is what a test is, and what any directory an agent must be told about can be. See [Citation directions](../user-facing/citation-directions.md) for the complete `[citations]` grammar and its rendered examples:

Repositories can also declare controlled-English constraints over declarations,
named chapters, and citations. The [chapter-rules guide](../user-facing/rules.md)
lists every accepted sentence and refusal rewrite; the runnable
[`examples/rules/`](../../examples/rules/) repository demonstrates the findings and
deduplication behavior ([§FS-rules](../functional-spec/FS-rules.md#fs-rules-grounded-declarations-state-and-enforce-chapter-rules)).

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

**Specs can live inline in source.** Declare the spec in a class or module doc-comment, then enroll it from the configured kind index with the canonical bare-ID link `grund fmt --cross-refs` writes — no stub file is required:

```rust
/// AR-event-bus: In-process event broadcaster
///
/// ## 1. Topology
pub struct EventBus { /* … */ }
// Kind index: - [§AR-event-bus](../../src/bus.rs)
```
`grund AR-event-bus` reads the source declaration directly, strips the `///` markers, and prints the Rustdoc prose. The same goes for Javadoc, JSDoc, Python docstrings, Go doc blocks, KDoc, Doxygen — every comment form enumerated in `grund`'s scanner spec. A one-line Markdown stub remains supported when a separate pointer file is useful.

`grund` does this itself: [§AR-checker](../../crates/grund-core/src/checker/report.rs) lives only in the doc-comment of `fn check` in [`crates/grund-core/src/checker/report.rs`](../../crates/grund-core/src/checker/report.rs), and its canonical row in [`docs/architecture/README.md`](../architecture/README.md) enrolls it without a stub — `grund AR-checker` prints the source prose.
