# DA-config-concern-records: the configuration becomes three concern records inside an envelope, and the checker splits in two

**Status:** Accepted
**Date:** 2026-09-30

## 1. Context

`Config` is one flat record of about sixty fields, and every component receives all of it. [§AR-system.2.3](../../architecture/README.md#23-config) gives config one job — load, validate, compile — and [§AR-system.4](../../architecture/README.md#4-dependency-direction) already holds the dependency direction between components, but nothing holds *which settings a component may read*. So the scanner can read a citation rule, the checker can read a presentation setting, and neither is visible in a signature.

[§FS-config.concerns](../../functional-spec/FS-config.md#concerns-every-key-belongs-to-exactly-one-concern) now says the keys are three kinds of thing: schema is what exists and what a well-formed one looks like, rules are how nodes relate, presentation is the bytes `grund` writes. [§DF-config-concerns](../functional/DF-config-concerns.md#df-config-concerns-a-keys-concern-is-derived-from-what-a-finding-from-it-can-be-about) derives which key is which, and [§DISC-core-concerns.2.7](../../discussions/proposals/2026-09-30-core-concerns.md#27-the-engine-already-reads-by-concern) is the survey behind this record: every non-test read of a `Config` field in `grund-core` already falls in one of the three, with exactly three reads crossing a concern.

That survey is what makes the split worth recording as a decision rather than proposing as a rewrite. The concerns are not a new structure to impose; they are the structure the engine already has, spelled nowhere.

## 2. Decision

### 2.1 One `Project`, three concern records, and an envelope

The loaded configuration becomes `Project { name, version, workspace, schema, rules, presentation }`. The first three fields are the **envelope** — identity, the version that spelled the file, and the members — which is read before any concern, decides which file governs and which projects there are, and constrains no node ([§FS-config.concerns](../../functional-spec/FS-config.md#concerns-every-key-belongs-to-exactly-one-concern)). It sits on `Project` rather than inside a concern because it is what the concerns are read inside.

`Schema` carries the citation syntax, the ID grammar, how a file is read, the note and lead measures, and one `Row` per `[[kinds]]` row. A row is `{ name, place: Option<Place>, kind: Option<Kind> }` — [§DF-non-citable-kinds.2.1](../functional/DF-non-citable-kinds.md#21-citable--false-on-the-existing-kinds-table)'s two-by-two read as types — with `places()` and `kinds()` as two iterators over one vector, so a feature that asks *every place* or *every kind* has one thing to ask and nothing to forget. `Rules` carries the citation rules, grounding and resolution. `Presentation` carries `fmt`, the per-kind titles, the description, the conversation opinion and the trigger.

**Amended 2026-10-07.** A row is `{ name, places: Vec<Place>, kind: Option<Kind> }`, not the single `place: Option<Place>` above. The owner's ruling of 2026-09-30 lets any row, citable or not, name several places ([§DISC-core-concerns.6.4](../../discussions/proposals/2026-09-30-core-concerns.md#64-amended-after-the-verdict-v-a08-and-v-a14)), which one optional place cannot hold. v1 never lowers more than one. The three states the record keeps apart stay distinct: a non-citable place has no kind, a homeless citable kind has no places, and the complement is a place whose extent is `Complement`. The shape as built is [§AR-config.1.3](../../architecture/AR-config.md#13-rows-places-and-kinds-over-one-vector).

### 2.2 `Run` and `Compiled` leave the record

Two groups of `Config` fields are not configuration at all. Per-run state — the root, the CLI base, the config file, the scope, the warnings raised while loading — becomes `Run`, which no file ever spells. Derived state — the compiled `Grammar`, and the scan demand — becomes `Compiled`, produced once from a `Project` by the config component.

`Compiled` is what closes the first of [§DISC-core-concerns.2.7](../../discussions/proposals/2026-09-30-core-concerns.md#27-the-engine-already-reads-by-concern)'s three crossings without moving work: config lowers the grounding rules into *record heading structure to depth d in place p* ([§AR-scanner.2.7](../../architecture/AR-scanner.md#27-grounding-units-per-file)), so the scanner keeps its speed and reads no rule.

### 2.3 The checker splits into `conform` and `judge`

```rust
fn compile(project: &Project) -> Result<Compiled, ConfigError>;
fn scan(schema: &Schema, compiled: &Compiled, run: &Run) -> Catalog;
fn conform(schema: &Schema, catalog: &Catalog) -> Report;
fn judge(rules: &Rules, catalog: &Catalog, expected: &Expected) -> Report;
fn present(p: &Presentation, schema: &Schema, rules: &Rules, catalog: &Catalog) -> Output;
```

`conform` is the checker's node-local half and `judge` its relational half, which is the same line [§FS-config.concerns](../../functional-spec/FS-config.md#concerns-every-key-belongs-to-exactly-one-concern) draws between schema and rules. A stage that is not handed a concern cannot read it, so the cut becomes a property of signatures that the compiler holds — the way [§AR-core-module-layout.1.1](../../architecture/AR-core-module-layout.md#11-modrs-is-the-components-whole-boundary) already has the compiler hold a component's boundary, rather than a convention a review has to notice.

`expected` is what keeps `judge` free of presentation. A drift check compares what a renderer produced with what is on disk, so the renderer computes the managed regions and the canonical index-entry link targets and hands them over; `judge` never interprets a presentation key. This is the agora's correction to the draft ([§DISC-core-concerns.6.2](../../discussions/proposals/2026-09-30-core-concerns.md#62-corrected)): a presentation setting does reach a finding today, and the way to keep the concerns apart is to pass the expectation rather than the setting.

**Amended 2026-10-10.** The signatures as built differ from the sketch above in two ways, and the cut they hold does not. `judge` also reads `&Schema`, read-only: a rule is written in schema's vocabulary — it names kinds and places, its finding spells an ID with the schema's marker and grammar, and an index is spelled on its kind's row ([§AR-config.1.4](../../architecture/AR-config.md#14-index-is-spelled-on-the-kind-and-judged-by-rules)) — so the alternative, copying those facts into `Rules` or `Catalog`, would be a second model of places. And `Run` and `Compiled`, which [§DA-config-concern-records.2.2](DA-config-concern-records.md#22-run-and-compiled-leave-the-record) takes out of the record because they are not concerns, reach every stage as one frame rather than as concern arguments. What the signatures hold is three prohibitions: `conform` is handed no `Rules` and no `Presentation`, `judge` is handed no `Presentation`, and the scanner is handed no `Rules`. Below the writers, `Presentation` is named only by the link target and the entrypoint probe's surface reach, each handed it explicitly. The shape as built is [§AR-checker.1](../../../crates/grund-core/src/checker/report.rs), and the ledger that holds it is [§AR-system.4](../../architecture/README.md#4-dependency-direction).

### 2.4 A version is a reader, and `Findings` becomes `Catalog`

[§FS-config.5.2](../../functional-spec/FS-config.md#52-every-older-version-keeps-its-meaning) promises that a binary reads every version it ever supported under that version's meaning. With one `Project` that promise costs one reader per version: the v1 reader parses today's file with today's defaults and carve-outs and lowers it into a `Project`, and no component above config learns that a second version exists. Validation about meaning rather than spelling — prefix-freedom, home uniqueness, value prerequisites — runs once, on the `Project`, for every reader. The reader is the directory `config/v1/` rather than one file, because together its parts exceed the size budget ([§AR-config.2](../../architecture/AR-config.md#2-the-v1-reader-and-its-isolation)).

`Findings` is renamed `Catalog`, the name [§FS-terms.terms.1](../../functional-spec/FS-terms.md#terms1-declarations-and-coordinates) already gives the scan's shared set. A *finding* is a report item; the central type should not wear another word's name.

## 3. Alternatives rejected

### 3.1 Keep the flat `Config` and document which component may read what

Cheapest, and it holds for exactly as long as everyone remembers. The three crossings of [§DISC-core-concerns.2.7](../../discussions/proposals/2026-09-30-core-concerns.md#27-the-engine-already-reads-by-concern) were each introduced deliberately and each is defensible, which is the point: nothing said *no*, so nothing said anything. A convention that a signature could hold and does not is a convention with no gate.

### 3.2 One record per component

More precise than three concerns — the scanner's view, the checker's view, the writers' view — and it fixes nothing. Two components in the same concern would carry two records that drift, the concern words would have no home in the code, and the shape would follow today's module list rather than the question [§FS-config.concerns](../../functional-spec/FS-config.md#concerns-every-key-belongs-to-exactly-one-concern) asks. Three records with two iterators is the smaller surface.

### 3.3 Split the checker by finding code rather than in two

The checker has a dozen rule families and a record could give each its own input. But the line that matters is the one the specification draws — one node against at least two — and it is the line that decides what each half is allowed to read. A dozen inputs would be a dozen chances to hand a rules half the schema it did not need.

## 4. What it costs

`Config` stays as a façade while components move off it one at a time, so the list of components that still name it only shrinks — held the way [§AR-system.4](../../architecture/README.md#4-dependency-direction)'s upward reads are held, by a list a reviewer can read rather than by a flag day. The public root names `Config`, `KindConfig`, `CitationRules` and `Findings` take the deprecation path of [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): the new records ship beside them, the release notes name the removal, and the note names **`0.17.0`** ([§DISC-core-concerns.9.7](../../discussions/proposals/2026-09-30-core-concerns.md#97-what-a-deprecation-note-may-name)). The path's floor is the following minor, but a note naming `0.16.0` while the old names still stand makes `0.16.0` unpublishable, so the release named here is longer than the floor and never shorter.

**Amended 2026-10-07.** The `0.17.0` above was written while `0.15.0` was expected to ship the records, and it did not: neither `0.15.0` nor `0.16.0` carried them or any note ([§DISC-core-concerns.9.8](../../discussions/proposals/2026-09-30-core-concerns.md#98-what-shipped)). The removal release is therefore not a number this record fixes. A note is written in the release that actually ships the records and names the minor two after it ([§DISC-core-concerns.9.7](../../discussions/proposals/2026-09-30-core-concerns.md#97-what-a-deprecation-note-may-name)), and the ledger of [§DISC-core-concerns.9.9](../../discussions/proposals/2026-09-30-core-concerns.md#99-the-deprecation-ledger) holds the current plan: notice in `0.17.0`, removal in `0.19.0`. Where this paragraph and that ledger disagree, the ledger is right and this record is stale.

The gate for the change is that nothing user-visible moves: every existing test and e2e case byte-identical, no new upward read, and the derived concern inventory of [§DF-config-concerns](../functional/DF-config-concerns.md#df-config-concerns-a-keys-concern-is-derived-from-what-a-finding-from-it-can-be-about) regenerated from the new signatures and unchanged.

## release-note: Release note

- [§DA-config-concern-records](DA-config-concern-records.md#da-config-concern-records-the-configuration-becomes-three-concern-records-inside-an-envelope-and-the-checker-splits-in-two), [§AR-config](../../architecture/AR-config.md#ar-config-one-project-per-project-read-by-one-reader-per-version-and-lowered-losslessly), [§FS-distribution.3.1](../../functional-spec/FS-distribution.md#31-rust-grund-core-crate), [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): **`grund-core` deprecates `Findings`, `Config`, `KindConfig` and `CitationRules` at its root, to be removed in `0.19.0`.** The engine now reads a `grund.toml` into `Project` (its `schema`, `rules` and `presentation`, inside an envelope of name, version and workspace), `Run` and `Compiled`, and the scan returns a `Catalog`. All four are public beside the old names. `Findings` is now an alias of `Catalog`, and `Config` is a façade built from the records. Code that names an old type still compiles, with a deprecation warning that names the replacement. Only a build with `-D warnings` fails. No `grund` command, LSP response, JSON report or written file changes, for any v1 config. Closes [issue #453](https://github.com/agent-grounds/grund/issues/453). (PR #501)
