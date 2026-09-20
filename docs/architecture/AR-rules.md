# AR-rules: sentences and facts meet only in the rule engine

Chapter rules need a sentence front end and a logic engine that can change
independently. The boundary is the two representations fixed by
[§FS-rules.11](../functional-spec/FS-rules.md#11-functional-architecture-constraint):
the front end produces `ParsedRule`, fact producers produce `RuleFacts`, and
the engine evaluates those values without recovering either producer's input.

## placement: Where chapter rules sit

```text
rule title + config vocabulary ─► [ rules ] ─► Diagnostic ─► checker
resolved structural model ──────► [       ]
```

The twelfth component of the pipeline ([§AR-system.2.12](README.md#212-rules)).
It takes authored rule titles and vocabulary from the checker, and the complete
resolved structural model from the resolver; it gives the checker located
diagnostics to merge into the shared report. Internally, the sentence front end
gives only `ParsedRule` to the engine and the Markdown adapter gives only
`RuleFacts`. It must not know a frontend or renderer, and the scanner must not
know this component at all ([§AR-system.4](README.md#4-dependency-direction)).

## 1. Owners and one-way dependencies

The component has three internal owners and one external orchestrator. Files
for each owner stay under its named module so the import guard can judge the
boundary rather than infer ownership from a function name:

1. `sentence/` owns the five productions, project-vocabulary and selector
   validation, canonical refusals, normalization, and `ParsedRule`.
2. `facts/` owns the producer-neutral `RuleFacts` schema; `markdown/` is the
   phase-1 adapter from the resolver's structural model into that schema.
3. `engine/` owns relational evaluation and semantic deduplication. It returns
   the model's internal `Diagnostic` data and renders nothing.
4. The checker owns sequencing: extract title, identity, anchor, and vocabulary;
   ask the adapter for facts; call the engine; merge the diagnostics. It owns no
   grammar production, fact clause, or deduplication rule.

The source dependency is one way even though data returns to the caller:

```text
checker ─┬─► sentence ─► ParsedRule ─┐
         └─► markdown ─► RuleFacts ──┴─► engine ─► Diagnostic ─► checker report
resolver ──────────────► markdown
scanner ─► resolver                         scanner ─X─► rules
```

The `rules` module may read resolver, config, grammar, and model boundaries
below it. The sentence front end may read only grammar/config vocabulary and
its scalar inputs; it may not import facts, the adapter, the engine,
deduplication, scanner records, or diagnostic/finding types. The engine may
read `ParsedRule`, `RuleFacts`, and `Diagnostic`; it may not import the sentence
module, Markdown adapter, grammar, resolver, scanner, filesystem APIs, public
`Finding`/`Report` records, or any tree-walking type. The adapter may read the
resolver's complete model and model records but never calls the parser or
engine. No callback crosses any of these boundaries.

## 2. `ParsedRule`

`ParsedRule` is immutable normalized data, not a syntax tree and not a wrapper
around the authored sentence ([§FS-rules.3](../functional-spec/FS-rules.md#3-the-five-sentence-families),
[§FS-rules.11](../functional-spec/FS-rules.md#11-functional-architecture-constraint)).
It has these fields:

| Field | Contract |
|---|---|
| `origin` | configured rule identity, including its project, or the stable `--rule` origin |
| `anchor` | repository-relative path, line and optional column for a declaration; the stable `--rule` pseudo-anchor otherwise |
| `subject` | one normalized local kind, declaration, quantified named chapter, or exact named-chapter selector |
| `modality` | required or recommended, paired with positive or prohibiting polarity |
| `relation` | `have chapter`, `cite`, or `be cited by`; per-target coverage is `cite` with per-target cardinality |
| `targets` | a chapter name or a bytewise-sorted, duplicate-free set of qualified kinds, with aggregate or per-target mode |
| `cardinality` | inclusive normalized lower and upper bounds; unbounded sides are absent and a prohibition is an upper bound of zero |

Normalization erases spelling alternatives such as `one` versus decimal `1`
and preserves no title text. Equality over every field except `origin` and
`anchor` is semantic equality for deduplication; contributing origins are then
sorted separately. A subject or target never contains a path, file, folder,
wildcard, symbol, definition, or derived term in phase 1
([§FS-rules.12](../functional-spec/FS-rules.md#12-deliberate-phase-1-absences)).

The sentence front end stands alone when a test supplies title text, scalar
origin/anchor values, and a hand-written config vocabulary, then compares the
whole `ParsedRule`. Such a test constructs no `RuleFacts`, scanner record,
diagnostic, or report.

## 3. `RuleFacts`

`RuleFacts` is one immutable snapshot. It is an in-memory boundary only; phase
1 defines no serialized form ([§FS-rules.5.1](../functional-spec/FS-rules.md#51-facts-and-identity),
[§FS-rules.12](../functional-spec/FS-rules.md#12-deliberate-phase-1-absences)).
It contains:

| Field | Contract |
|---|---|
| `header.schema` | fact-schema version, independent of config schema |
| `header.project` | stable identity of the selected project/workspace scope |
| `header.producer` | stable identity of the producer that minted the opaque keys |
| `header.completeness` | explicit complete/incomplete state, never inferred from an empty relation |
| `decl` | set of `(node, kind)` tuples |
| `chapter` | set of `(node, section_path, display_name)` tuples |
| `contains` | set of `(parent_node, child_node)` tuples |
| `cites` | set of `(site, immediate_from_node, target_node)` tuples |
| `site_in` | set of `(site, unit_node)` tuples, one for every containing rule unit |
| `nodes` | side map from opaque node key to stable authored label and repository-relative anchor |
| `sites` | side map from opaque site key to stable authored label and repository-relative anchor |

Node and site keys are opaque values scoped by both project and producer; an
engine may compare them inside the snapshot but may not derive paths, IDs, or
scanner indexes from them. Side metadata is for located diagnostics and never
participates in logical equality. Every relation key has metadata and refers to
declared nodes/sites in the same snapshot.

Every declaration and accepted chapter gets its own node facts even when no
citation touches it, so citation edges never define a quantified universe.
Every physical citation gets a distinct site key, so equal written targets at
two sites still count twice. `cites` names the immediate enclosing declaration
or chapter, while `site_in` records every larger unit that contains the site.
Unresolved or ambiguous citations keep their ordinary scan diagnostics and add
no `cites` tuple.

The Markdown adapter stands alone when its output is compared with hand-built
expected relations. The engine stands alone when a test constructs both
representations directly. A second test producer can therefore build an
equivalent `RuleFacts` without a Markdown tree; equal facts and equal rules must
produce equal diagnostics.

## 4. Completeness is an engine gate

`header.completeness` is checked by the consumer, not trusted as a convention
of the Markdown adapter. An incomplete snapshot disables every conclusion that
depends on a closed world: absence, lower-bound failure, upper/exact count, and
per-target coverage. A positive physical-site prohibition may still be emitted
from a known `cites` tuple, because it proves presence rather than absence. Scan
diagnostics and exit 2 remain the checker's responsibility
([§FS-rules.4](../functional-spec/FS-rules.md#4-validation-lifecycle)).

The engine indexes the five relations and evaluates them; it never walks a
tree, opens a path, or asks a producer for more data. The first release is a
hand-written evaluator. Settings, exceptions and `grund:allow`, definitions,
program facts, a SCIP/LSIF producer, symbol vocabulary, on-disk exchange, and a
Datalog runtime remain deferred. The future program graph constrains producer
replaceability here; it is not an implementation target
([§FS-rules.12](../functional-spec/FS-rules.md#12-deliberate-phase-1-absences)).

## 5. Report boundary and consumers

The engine returns internal `Diagnostic` values with code, channel, message,
stable fact anchor, and secondary anchors. It does not construct the published
`Finding` or `Report`, render text/JSON, select suggestions, or map an exit code.
The checker merges and orders diagnostics through its existing report boundary;
the api then exposes one report to both CLI and LSP. Neither frontend may parse
a sentence, adapt facts, evaluate a clause, or deduplicate a rule
([§FS-rules.7.6](../functional-spec/FS-rules.md#76-selection-json-ordering-and-exits),
[§FS-rules.9](../functional-spec/FS-rules.md#9-managed-guidance-and-editor-parity)).

## 6. Boundary tests

`tests/integration/test_dependency_direction.py` places `rules` between resolver
and checker. `tests/integration/test_module_layout.py` reserves the thirteenth
module directory and refuses to leave it marked pending once `mod rules;`
lands. `tests/integration/test_rules_architecture.py` rejects scanner imports of
rules, sentence imports of facts/engine/diagnostics, engine imports of sentence,
Markdown/scanner/filesystem internals, unowned files, and a landed component
whose four behavioral drivers are still pending.

`crates/grund-core/src/rules/tests_boundaries.rs` carries those four drivers,
ignored only while the component is test-only:

- `sentence_front_end_returns_complete_parsed_rule_without_facts_or_diagnostics`
- `logic_engine_evaluates_hand_built_rule_and_facts_without_parser_or_scanner`
- `markdown_adapter_and_second_producer_drive_the_same_engine_result`
- `incomplete_fact_snapshot_suppresses_absence_and_count_conclusions`

Forcing ignored tests to run is deliberately red today. The implementation
replaces each sentinel with its named arrangement/assertions and removes its
ignore before declaring `mod rules;`; the Python guard makes forgetting that
step fail. These tests are the replacement proof required by
[§FS-rules.11](../functional-spec/FS-rules.md#11-functional-architecture-constraint),
not a promise left only in this page.
