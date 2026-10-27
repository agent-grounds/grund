# Architectural decisions

Why `grund` is *built* the way it is. Each file here is one decision about structure, packaging, or measurement: the context, the call, the alternatives, and the cost. The H1 declares a `DA-<slug>` ID, and the architecture point it settles cites it. Each record's first chapter is `## status:`, holding `Accepted` or `Superseded`; its `Date`, `Superseded by` and other prose lines stay above it, in the lead ([§DISC-core-concerns.10.2](../../discussions/proposals/2026-09-30-core-concerns.md#102-the-five-decisions-the-plan-makes-one-way)).

Half of this folder is the record of one rename. That is deliberate: the superseded entries are kept for the evaluation they contain, not for their verdicts, and deleting them would leave the accepted decisions arguing with nobody.

## Shape of the shipped tool

- [§DA-explicit-value-bindings](DA-explicit-value-bindings.md#da-explicit-value-bindings-compare-only-authored-delimited-value-bindings) — compare only authored delimited value bindings
- [§DA-root-aimed-value-bindings](DA-root-aimed-value-bindings.md#da-root-aimed-value-bindings-a-value-binding-may-aim-at-the-whole-root-its-components-joined-by-one-ascii-space) — a value binding may aim at the whole root, its components joined by one ASCII space
- [§DA-yaml-value-reader](DA-yaml-value-reader.md#da-yaml-value-reader-one-yaml-event-parser-builds-one-span-tree-and-value-rules-are-a-pass-over-it) — one YAML event parser builds one span tree, and value rules are a pass over it
- [§DA-toml-front-matter-reader](DA-toml-front-matter-reader.md#da-toml-front-matter-reader-one-toml-event-parser-builds-the-span-tree-a-toml-front-matter-block-is-read-through) — one TOML event parser builds the span tree a TOML front-matter block is read through
- [§DA-lsp-optional](DA-lsp-optional.md#da-lsp-optional-lsp-server-ships-as-a-separate-optional-binary) — LSP server ships as a separate, optional binary
- [§DA-pgo-release](DA-pgo-release.md#da-pgo-release-distributed-binaries-are-pgo-built-trained-on-the-benchmark-workload) — distributed binaries are PGO-built, trained on the benchmark workload
- [§DA-benchmark-instruction-counting](DA-benchmark-instruction-counting.md#da-benchmark-instruction-counting-the-performance-harness-counts-instructions-not-wall-clock-seconds) — the performance harness counts instructions, not wall-clock seconds
- [§DA-benchmark-frontends](DA-benchmark-frontends.md#da-benchmark-frontends-the-benchmarks-measure-the-language-server-and-the-python-extension-not-the-core-library-or-the-node-addon) — the benchmarks measure the language server and the Python extension, not the core library or the Node addon
- [§DA-engine-renders-nothing](DA-engine-renders-nothing.md#da-engine-renders-nothing-the-engine-renders-nothing-so-the-deprecated-compat-frontend-retires) — the engine renders nothing, so the deprecated compat frontend retires
- [§DA-config-concern-records](DA-config-concern-records.md#da-config-concern-records-the-configuration-becomes-three-concern-records-inside-an-envelope-and-the-checker-splits-in-two) — the configuration becomes three concern records inside an envelope, and the checker splits in two
- [§DA-one-fact-one-point-one-implementation](DA-one-fact-one-point-one-implementation.md#da-one-fact-one-point-one-implementation-every-fact-grund-decides-has-one-defining-point-one-implementation-and-reads-of-it-everywhere-else) — every fact grund decides has one defining point, one implementation, and reads of it everywhere else

## Name and packaging

- [§DA-rename-to-grund](DA-rename-to-grund.md#da-rename-to-grund-rename-gnd-to-grund-before-first-publish) — Rename gnd to grund before first publish
- [§DA-pypi-uses-grund-as-the-package-name](DA-pypi-uses-grund-as-the-package-name.md#da-pypi-uses-grund-as-the-package-name-pypi-uses-grund-as-the-package-name) — PyPI uses grund as the package name
- [§DA-reference-checker-name](DA-reference-checker-name.md#da-reference-checker-name-name-for-the-spec-reference-checker-tool) — *(superseded)* Name for the spec-reference checker tool
- [§DA-pypi-package-name](DA-pypi-package-name.md#da-pypi-package-name-pypi-uses-gnd-cli-as-the-package-name) — *(superseded)* PyPI uses gnd-cli as the package name

This index is navigational — citations should target the decision ID directly, never this file.
