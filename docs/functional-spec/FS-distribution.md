# FS-distribution: grund distribution targets

`grund` is one Rust engine that targets the cargo, npm and PyPI ecosystems, each with the CLI, the optional LSP server and an idiomatic API; the check engine is one shared library and only the surfaces differ. This declaration says which packages carry it, what each binding offers, and how a release is cut and published. The release publishes the Rust crates `grund-core`, `grund` and `grund-lsp`; the npm and PyPI packages, `grund-lsp` among them, are assembled and rehearsed as a candidate and published by no path ([§FS-distribution.4.13](FS-distribution.md#413-a-candidate-is-rehearsed-before-anything-is-published)). Serves [§GOAL-multi-language](../goals.md#goal-multi-language-same-engine-three-platforms) and [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, body, section, lead), [§FS-terms.terms.2](FS-terms.md#terms2-citations)
(shorthand), [§FS-terms.terms.3](FS-terms.md#terms3-source-forms) (stub), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, scope, config root, workspace,
alias), and [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding, severity, suggestion, caution, verdict).

- **compatibility notice** — The one line a release publishes about a change that a pull
  request's title cannot carry: a verdict the release moves, in the words of the decision
  record that moves it, written as that record's `release-note` section.

## requirements: What every distribution of grund holds to

These bind the distributions as a whole, beside what every install, operating system and binding already holds to: the same bytes and the same answer ([§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes), [§REQ-one-answer](../requirements/REQ-one-answer.md#req-one-answer-every-command-gives-the-same-answer-about-one-tree)), and an API name removed only along [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path).

### requirements.ecosystems: Every ecosystem the targets name installs `grund`

The three crates publish ([§FS-distribution.1](FS-distribution.md#1-targets), [§FS-distribution.4.10](FS-distribution.md#410-cratesio-publishes-in-dependency-order-and-artifacts-upload-last)), so it holds on cargo; on npm and PyPI it is deferred to [§RM-distribution](../roadmap.md#rm-distribution-cargo--npm--pypi-from-one-engine): the packages are assembled and rehearsed as a candidate ([§FS-distribution.4.13](FS-distribution.md#413-a-candidate-is-rehearsed-before-anything-is-published)), and none is published, because the publisher is disabled ([§FS-distribution-candidate.8.1](FS-distribution-candidate.md#81-publication-is-disabled-and-gated)) and the authority to publish is not yet held ([§FS-distribution-candidate.8.2](FS-distribution-candidate.md#82-identity-is-not-authority)). Serves [§GOAL-multi-language](../goals.md#goal-multi-language-same-engine-three-platforms).

### requirements.library: A binding answers as a library, never as a process

Every binding returns data and leaves its host as it found it: it prints nothing, reads no argv, never exits and never changes the working directory ([§FS-distribution.3.1](FS-distribution.md#31-rust-grund-core-crate), [§FS-distribution.3.1.2](FS-distribution.md#312-what-10-freezes), [§FS-distribution.3.2.3](FS-distribution.md#323-workers-loading-and-source-builds), [§FS-distribution.3.3.4](FS-distribution.md#334-silent-synchronous-calls)). A failure is data too, carrying the class and location the engine gave it rather than ones parsed from rendered text ([§FS-distribution.3.2.2](FS-distribution.md#322-typed-readonly-host-records), [§FS-distribution.3.3.2](FS-distribution.md#332-operational-failures-and-invalid-arguments)), and the CLI stays the only renderer ([§FS-distribution.3.0.3](FS-distribution.md#303-complete-data-and-canonical-parity)). Serves [§GOAL-multi-language.2](../goals.md#2-idiomatic-surfaces).

### requirements.own-payload: An install runs the payload built for its host at its own version, or refuses saying how to get one

It holds in the candidate. npm installs the one platform package matching the host's OS, CPU and libc at exactly the umbrella's version, and the command and the addon share it ([§FS-distribution-candidate.2.2](FS-distribution-candidate.md#22-npm-selects-one-platform-package-by-os-cpu-and-libc)); a wheel carries the executable and the extension together ([§FS-distribution-candidate.3.1](FS-distribution-candidate.md#31-payloads-sit-where-each-registry-runs-them)). A platform package that is missing, of another version or for an unlisted row is an error naming the fix, never another row's payload, a download or a compile ([§FS-distribution-candidate.3.3](FS-distribution-candidate.md#33-a-missing-payload-is-an-error-never-a-substitute), [§FS-distribution.3.2.3](FS-distribution.md#323-workers-loading-and-source-builds)), and an npm install never compiles ([§FS-distribution-candidate.4.1](FS-distribution-candidate.md#41-npm-builds-from-source-only-when-asked)). A target the matrix defers may build from source and gains no support by succeeding ([§FS-distribution-candidate.1.4](FS-distribution-candidate.md#14-deferred-targets-gain-no-support)). Serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible).

### requirements.versions: What a distributed binary says about versions is true of it

A build from `main` reports the next version with its `-dev` suffix rather than the release it is ahead of, and `grund` names its commit too ([§FS-distribution.4.1](FS-distribution.md#41-between-releases-main-carries-a-dev-version), [§FS-distribution.4.1.1](FS-distribution.md#411-a-development-build-names-its-commit)); a released binary carries its tag's version, which every package of the release shares ([§FS-distribution.4.3](FS-distribution.md#43-releaseyml-publishes-a-commit-that-already-carries-its-version), [§FS-distribution-candidate.6.2](FS-distribution-candidate.md#62-one-version-across-every-package)); and no release is cut at a version that contradicts a release its own messages name ([§FS-distribution.4.2](FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name)). The release named in a warning is what buys a repository its time, and it buys nothing if it is false. Serves [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path).

### requirements.next-operation: The next operation reaches every binding, or is excluded there by name

Each binding offers the CLI's conceptual operations, and one that is absent is never advertised ([§FS-distribution.3](FS-distribution.md#3-api-surfaces)), so Node and Python each list their inventory and their exclusions ([§FS-distribution.3.2.1](FS-distribution.md#321-operations-options-and-refusals), [§FS-distribution.3.3.5](FS-distribution.md#335-complete-inventory)). A subcommand added from here on joins each binding's inventory or, with its reason, that binding's exclusions, never neither; a field the engine adds to a result reaches each binding's record in that binding's spelling ([§FS-distribution.3.0.1](FS-distribution.md#301-report-and-finding), [§FS-distribution.3.3.1](FS-distribution.md#331-typed-immutable-results)). This binds what is added next, not the inventories already written. Serves [§GOAL-multi-language](../goals.md#goal-multi-language-same-engine-three-platforms).

## 1. Targets

An `implemented` package is one the release publishes ([§FS-distribution.4.10](FS-distribution.md#410-cratesio-publishes-in-dependency-order-and-artifacts-upload-last)). A `candidate` package is assembled from one commit for the support matrix of [§FS-distribution-candidate.1.1](FS-distribution-candidate.md#11-six-rows-five-of-them-registry-rows), in the inventory of [§FS-distribution-candidate.2.1](FS-distribution-candidate.md#21-thirty-nine-artifacts-from-one-version), and installed and proved on every registry row by a rehearsal that holds no credential ([§FS-distribution.4.13](FS-distribution.md#413-a-candidate-is-rehearsed-before-anything-is-published)); it is not published, and only the disabled publisher of [§FS-distribution-candidate.8.1](FS-distribution-candidate.md#81-publication-is-disabled-and-gated), once separately authorized, can make one `published`.

| Registry | Package name        | Status      | Contents                                                                  |
|----------|---------------------|-------------|---------------------------------------------------------------------------|
| cargo    | `grund-core`          | implemented | Shared engine library used by the CLI, the LSP and the bindings.            |
| cargo    | `grund`               | implemented | CLI crate depending on `grund-core`; installs the `grund` binary.              |
| cargo    | `grund-lsp`           | implemented | Optional LSP server crate ([§FS-lsp](FS-lsp.md#fs-lsp-grund-ships-an-optional-lsp-server)) depending on `grund-core`; installs the `grund-lsp` binary. |
| npm      | `grund-cli`           | candidate   | `grund` launcher + Node API (via `napi-rs`); five `@grund-cli/<suffix>` platform packages carry the CLI and addon. |
| npm      | `grund-lsp`           | candidate   | `grund-lsp` launcher; five `@grund-lsp/<suffix>` platform packages carry the server. |
| PyPI     | `grund`               | candidate   | Five `cp310-abi3` wheels carrying the `grund` executable + Python API (via `PyO3` / `maturin`), and an sdist. |
| PyPI     | `grund-lsp`           | candidate   | Five `py3-none` wheels carrying the `grund-lsp` executable (`pipx install grund-lsp`), and an sdist. |

### 1.1 Package names

The CLI package is `grund` on cargo and PyPI, where the wheel installs the `grund` command and the import module `grund`, and `grund-cli` on npm, because the unscoped `grund` package is externally occupied; it still installs the `grund` command ([§DA-pypi-uses-grund-as-the-package-name](../decisions/architectural/DA-pypi-uses-grund-as-the-package-name.md#da-pypi-uses-grund-as-the-package-name-pypi-uses-grund-as-the-package-name)). The npm platform packages are the five `@grund-cli/<suffix>` and five `@grund-lsp/<suffix>` names of [§FS-distribution-candidate.2.2](FS-distribution-candidate.md#22-npm-selects-one-platform-package-by-os-cpu-and-libc), one per registry row, whose scopes and names are provisional until held ([§FS-distribution-candidate.8.2](FS-distribution-candidate.md#82-identity-is-not-authority)).

Before it publishes, the release's registry-name check, `scripts/check-registry-names.sh`, re-verifies every one of these names, the npm and PyPI LSP names and the platform packages included, against the live registries ([§FS-distribution.4.3](FS-distribution.md#43-releaseyml-publishes-a-commit-that-already-carries-its-version)). A name is free only when the package's own endpoint answers `404`: a failure to query that endpoint is a query failure and never an availability, and once a registry has answered that the package exists, nothing later turns the name back into a free one. Whether an existing package is this project's is registry-specific:

- On crates.io, for `grund-core`, `grund` and `grund-lsp`, only the registry's owner record decides: the owner endpoint `/api/v1/crates/<name>/owners` must return a `users` entry whose `kind` is exactly `user` and whose `login` is exactly `vjovanov`, this project's crates.io identity. Nothing else in the response establishes ownership: not the display `name`, the URL text, the numeric id or the `github_username_matches` flag, not a bare organization string such as `agent-grounds`, and not a team record, since no team is part of the identity this project declares. The package's own `repository` metadata never participates: it is publisher-supplied, so any package may copy this repository's URL into it, and after a repository move only the publish the registry-name check stands in front of could correct it.
- On npm and PyPI, a package is this project's when the registry's own metadata for it names this repository, or this project's repository before a move. That metadata is written by the last publish, so a move reaches those registries only with the next release, and accepting only the current repository would lock the project out of its own names.

A crates.io name that exists and is not proven this project's stops the release with a non-zero exit, in one of four distinguishable ways, each naming the owner endpoint it asked, because the operator's next move differs in each. An owner record that is well-formed but holds no trusted user, an empty owner set or only untrusted owners, reports `error: crates.io/<name> is already taken without trusted owner vjovanov`. Owner data that is missing, malformed or of the wrong shape reports that the owner evidence could not be read. An owner endpoint that answers `404`, or any other non-`200`, reports that ownership could not be determined and names the status. A request that never completes reports the transport failure. None falls back to the package's declared metadata.

Availability, project identity and publishing authority are three gates, not one: the registry-name check answers the first two, and a name that passes both is still not one this project may publish to until the authority of [§FS-distribution-candidate.8.2](FS-distribution-candidate.md#82-identity-is-not-authority) exists.

### 1.2 Support packages point at the CLI

A support package, one that is not the primary user-facing install, such as `grund-core`, publishes registry README content that links users to the `grund` CLI and names sibling packages such as `grund-lsp` once they exist. The npm platform packages are support packages too: each README says which row it serves and that its umbrella installs it, and links to the CLI. Every npm and PyPI package carries the licence and says truthfully what it installs ([§FS-distribution-candidate.2.4](FS-distribution-candidate.md#24-every-package-says-what-it-holds)).

### 1.3 The CLI does not pull in the LSP

No registry's CLI install pulls in `grund-lsp`, directly or transitively: they are independent packages ([§DA-lsp-optional](../decisions/architectural/DA-lsp-optional.md#da-lsp-optional-lsp-server-ships-as-a-separate-optional-binary)). A user who only runs `grund check` in CI installs the CLI alone, and a user who wants editor integration installs `grund-lsp` separately and configures their editor to launch it ([§FS-lsp.2](FS-lsp.md#2-installation-and-lifecycle)). On npm and PyPI this is proved on the installed packages rather than read off a manifest: every registry row installs the CLI without the LSP and the LSP without the CLI, and each works alone ([§FS-distribution-candidate.5.2](FS-distribution-candidate.md#52-every-registry-row-installs-fresh-and-without-rust)).

## 2. CLI parity

The `grund` binary behaves the same however it was installed, on Linux, macOS and Windows: the same flags, the same exit codes and the same report bytes ([§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes)). Every path it reports is logical, spelled from the report base ([§FS-cli.3.4](FS-cli.md#34---path-base--where-report-paths-are-spelled-from), [§FS-config.3.6](FS-config.md#36-output--report-format)) with `/` on every platform ([§FS-errors.4](FS-errors.md#4-determinism)), in text reports, JSON fields and `sites`, an E2E case's fixture list and a stub's link target alike, and so are the cross-reference URLs it generates; only a launch-time error about a path outside the scanned repository prints a native platform path, having no repository-relative one to print. "However it was installed" includes the npm launcher, the wheel's executable and the archive, and the proof for those is the installed artifact rather than a checkout build: the launcher adds nothing between the user and the payload ([§FS-distribution-candidate.3.2](FS-distribution-candidate.md#32-the-npm-launcher-is-invisible)), and the rehearsal compares every installed `grund` byte for byte against the goldens on every registry row ([§FS-distribution-candidate.5.3](FS-distribution-candidate.md#53-every-install-of-the-cli-answers-alike)).

## 3. API surfaces

Each binding offers the CLI's conceptual operations as calls that return data, with a check-and-iterate path so the engine can be embedded in test runners and editor servers. An operation a binding leaves out is named among its exclusions and never advertised as supported.

### 3.0 Language-neutral data shapes

Every binding returns the same data, spelled the way its language spells it. The shapes below are normative: the report and its findings as the CLI projects them, which each binding's own records extend, and the show options every binding takes. Serves [§GOAL-multi-language](../goals.md#goal-multi-language-same-engine-three-platforms).

#### 3.0.1 Report and Finding

Host results keep all of the engine's data: nullable columns, multi-site locations, rule authority, scan-error status, output-format metadata and run cautions, which stay apart from the report's warnings even when setup fails. Node's records also carry the complete and the selected report and partial outputs, with empty collections as arrays and optional fields present as null ([§FS-distribution.3.2.2](FS-distribution.md#322-typed-readonly-host-records)). The block below is the CLI's projection of that data, which stays frozen: it omits columns and projects empty sites and authority to null, so it is not the complete host schema ([§FS-distribution.3.0.3](FS-distribution.md#303-complete-data-and-canonical-parity)).

```
Report {
  errors:      [Finding]
  warnings:    [Finding]
  suggestions: [Finding]  // filled only under `check --suggestions` (FS-check.2.3)
}

Finding {
  severity: "error" | "warning"  // absent on a suggestion, whose JSON carries "channel": "suggestion"
                                 // in its place (FS-errors.5)
  code:     // a `check` finding — one code of the sorted supported catalog in FS-errors.5
            // — or, on a failed ID query (FS-show.3, rendered with this same shape on stderr,
            //   path/line null) — "not-found" | "missing-section" | "broken-stub" | "ambiguous"
            //                   | "ambiguous-section" | "invalid-id" | "query-failed"
  path:     string?        // relative to config root (FS-config.3.6); null for a run-level warning in the
                           // report or a failed ID query (FS-errors.5.2, FS-errors.5.2.3)
  line:     u32?           // 1-indexed; null for a file-level finding with no line (e.g. an unreadable file, FS-check.2)
  message:  string         // the human-readable text
  sites:    [{ path, line }]?  // null for a single-site finding; a list naming every site for a multi-site
                               // finding (a duplicate declaration) or an ambiguous-ID / ambiguous-section
                               // query failure that names sites; null for the number-only shorthand's
                               // `ambiguous` refusal, which names candidates instead (FS-errors.5.2)
  authority: [string]?         // last key; the bytewise-sorted rule origins that authored this finding
                               // (FS-rules.7.6) — a declared rule's ID, or "--rule" for a `check --rule`
                               // trial sentence, or both where they reached the same meaning (FS-rules.6);
                               // null for a finding no rule authored, and always null on a failed ID query
                               // and a run-level warning (FS-errors.5.2, FS-errors.5.2.3)
}
```

#### 3.0.2 ShowOpts

The show options are a section, a slice of the `show` ladder ([§FS-show.1.6](FS-show.md#16-how-much---brief-the-default---toc---full)) and a format. Python takes the query operands positionally and these as keywords ([§FS-distribution.3.3.5](FS-distribution.md#335-complete-inventory)); Node takes them in an options object ([§FS-distribution.3.2.1](FS-distribution.md#321-operations-options-and-refusals)).

```
ShowOpts {
  section: string?    // dotted section path, e.g. "3.1.2"
  mode:    "lead" | "brief" | "toc" | "full"
                      // the same mutually exclusive show ladder as §FS-show.1;
                      // "lead" is the CLI's no-flag default
                      // default: "lead"
  format:  "text" | "md" | "json"
}
```

#### 3.0.3 Complete data and canonical parity

The Rust engine, Python and Node give the same complete data for one tree, every field present and every array in the engine's order, including failures and run cautions. The shared corpus `tests/bindings/` compares each binding with the same Rust oracle through the bindings' data APIs, never through a CLI subprocess or a comparison of messages. Both sides must come from the same source, engine version, config, tree, invocation and ignore state: a wrong version fails the comparison and is never normalized away.

Complete data is compared as a canonical UTF-8 JSON envelope with exactly the keys `failure`, `result` and `run_cautions`, one of `failure` and `result` null, except that an empty successful result may be null. Run cautions sit in the top-level `run_cautions` alone, never again inside `result`. Object keys sort recursively by UTF-8 bytes and arrays keep engine order; nullable fields are present, empty collections are arrays, separators are compact, and exactly one LF ends the record. Unicode and `/` stay literal; quote, backslash, LF, CR and tab use their JSON escapes, and every other U+0000–U+001F character uses lowercase `\u00xx`, backspace and form feed included. The Node adapter writes this envelope itself rather than through `JSON.stringify`, whose escapes and key order differ, and maps the binding's fixed camelCase field names to snake_case while authored IDs and config keys stay opaque. Structured equality and exact canonical bytes are separate assertions.

The same host records, projected into the CLI's wire form, give the CLI's stdout and stderr byte for byte, and its exit status where it has one. That projection is frozen, and these are its only differences from the envelope:

- A check projects its selected report in the CLI's one global order and with its keys ([§FS-errors.4.1](FS-errors.md#41-ordering), [§FS-errors.5.1](FS-errors.md#51-on-stdout--the-commands-output)), ties in warning, error, suggestion order, with no `column` key and with empty `sites` and `authority` as null; a clean non-empty JSON check prints nothing on either stream ([§FS-output-shapes.1](FS-output-shapes.md#1-finding-object)).
- A finding with a line goes to stdout and one without to stderr. Run cautions render as `warning: <message>` text on stderr, even under JSON, before any later refusal or report, while a config warning carried as one of the report's warnings stays a JSON finding ([§FS-errors.5.2.2](FS-errors.md#522-launch-time-messages-stay-text)): neither channel is flattened into the other.
- A single query failure is the frozen diagnostic object on stderr ([§FS-errors.5.2](FS-errors.md#52-on-stderr--what-is-not-output)); a batch's failed query is its query, result and error record on stdout ([§FS-errors.5.3](FS-errors.md#53-show---batch)). A launch failure stays `error:` text, with its hints and known lists ([§FS-errors.5.2.2](FS-errors.md#522-launch-time-messages-stay-text)).
- Every other command keeps its own JSON fields and key order ([§FS-output-shapes](FS-output-shapes.md#fs-output-shapes-machine-readable-output-shapes)), and no host field or null optional metadata adds a CLI key; size measurements become `lead_<unit>` and `full_<unit>` pairs in the caller's unit order ([§FS-output-shapes.5.2](FS-output-shapes.md#52-list---size---formatjson)).
- The CLI escapes quote, backslash, LF, CR and tab, and every `char::is_control` character as lowercase `\uNNNN`, DEL and C1 included, which differs on purpose from the envelope's C0 rule.

Paths keep the engine's `/` and its report base. Neither the projection nor any binding parses a message to infer a class or a location. Serializers and comparators are test code: a binding returns records and never prints, and the CLI stays the renderer.

### 3.1 Rust (`grund-core` crate)

```rust
let report = grund_core::check(&path)?;
let body = grund_core::show("FS-check", ShowOpts::default())?;
```

`Report` and the underlying `Catalog` are plain data, so a caller can iterate, filter, or render its own output. The effective configuration is reachable through `effective_project`, as a `Project` whose `schema`, `rules` and `presentation` hold one concern each ([§FS-config.concerns](FS-config.md#concerns-every-key-belongs-to-exactly-one-concern)), but `Project` is `#[doc(hidden)]` and outside the promise of [§FS-distribution.3.1.2](FS-distribution.md#312-what-10-freezes), because its records still move ([§FS-schema.13](FS-schema.md#13-what-is-absent)). Every function on this surface returns data and writes to no stream, so one answer renders alike in a terminal, an editor and an embedder; this includes the report's warning channel, so a caution settled before any report exists still reaches a caller as a warning rather than as a line on its stderr ([§FS-check.4.10](FS-check.md#410-include_root--false-leaves-the-blocks-own-files-unread), [§FS-workspace.6.1.7](FS-workspace.md#617-a-claiming-block-that-cannot-answer-fails-the-run)).

#### 3.1.2 What 1.0 freezes

`grund-core` 1.0 freezes exactly the root names this point reaches: the entry points `check`, `check_with_opts`, `scan`, `show`, `list`, `list_sizes`, `refs`, `refs_outcome`, `cover`, `cover_text`, `cover_lines`, `format_references`, `propose_id`, `complete_ids`, `init`, `fetch_snapshot` and `integrations` ([§FS-distribution.3.1.3](FS-distribution.md#313-integrations-is-the-one-integrations-entry-point)), the typed refusals `ShowQueryError`, `FmtScanAbort` and `FmtConflictAbort` ([§FS-fmt.3.2](FS-fmt.md#32-a-run-that-needs-the-whole-declaration-set-rewrites-nothing)), and every root type their signatures reach, transitively. The rows whose disposition is `frozen at 1.0` in the [public-surface inventory](../discussions/proposals/2026-09-22-grund-core-public-surface-inventory.md) are the full list. An entry point with no typed refusal returns `anyhow::Result`. None is a process entry point: no root function parses argv, prints or returns an exit code, and a caller that needs a process runs the `grund` CLI package.

Every other root name is `#[doc(hidden)]` and outside the promise, and so are the `test-*` Cargo features and what they export. Hiding retires nothing and changes no type: a hidden name still links and still works. Frozen means what Cargo reads into a `1.0.0`: a `1.x` release may add a name, a field or a variant and may deprecate a name, and only `2.0` removes or retypes one ([§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)).

An option record is built from `Default` with its fields assigned; an output record or a refusal is read, never built. That is what lets `1.x` add a field or a variant, so every promised struct and enum is `#[non_exhaustive]`, and no other type and no variant is: outside the crate, building a promised record with a struct literal, or matching a promised enum without a wildcard arm, does not compile.

#### 3.1.3 `integrations` is the one integrations entry point

`integrations(opts: IntegrationsOpts) -> Result<IntegrationsOutput, IntegrationsError>` answers what `grund integrations` answers, as data; its options, output and refusal are [§FS-integrations.1.3](FS-integrations.md#13-which-side-owns-what)'s. The closed vocabularies it is spelled in have exactly these methods: `IntegrationClient` has `ALL`, `name` and `from_name`; `InstallKind` has `name`; `ConversationRendering` has `name` and `from_name`; `ConversationTarget` has `ALL`, `name` and `from_name`; and `IntegrationAgent`, the six agents of [§FS-integrations.4.3.8](FS-integrations.md#438-the-global-instruction-files), has `ALL`, `name` and `from_name`.

### 3.2 Node (`grund-cli` npm package)

```js
import { check, show } from 'grund-cli';

const result = await check('./repo'); // findings are result.report.errors, warnings, suggestions
const body = await show('FS-check', { mode: 'brief' });
```

The binding is a native Node-API addon over the shared engine, and every call returns a Promise of a typed, readonly record.

#### 3.2.1 Operations, options and refusals

Every function returns a Promise, and an invalid argument or a failed load rejects it. Required strings are positional, and options are one optional plain object. Unknown keys, wrong types, NUL, unpaired UTF-16 surrogates, non-finite numbers, unsafe integers, invalid enum values and conflicting options reject `input`. An omitted optional value means its default; an explicit `null` is allowed only for `section`, batch queries and options whose declarations make them nullable. Arrays are copied at entry. There is no URL, Buffer, config-injection or argv API.

`RootOptions` is `{root?: string}`. A tree operation snapshots the working directory at entry and resolves a relative operand against that snapshot, keeping whether the root was omitted or given, and it never calls `chdir`. `check` takes its root positionally, `init` its target positionally and `scan` an explicit root; every other call takes `root` in its options. Report paths keep the engine's base, the config root or the invocation operand as `relative_paths` decides ([§FS-config.3.6](FS-config.md#36-output--report-format)).

A result carries `runCautions: Finding[]` at its top level, apart from the report's warnings, and a refusal carries the same array on `GrundError.failure`. Every call that loads a tree can fail with the common `config`, `io` and `operation` classes; the table names the cases particular to each operation.

| Public Promise signature | Options and defaults beyond root | Result and refusal behavior |
| --- | --- | --- |
| `check(root?: string, options?: CheckOptions): Promise<CheckResult>` | requireGrounding=false, suggestions=false, full=false, rule=null, only=[], ignore=[], onlyRule=false | A completed run resolves, its findings and per-file I/O failures included; a failed setup rejects. Selection is validated before discovery and narrows `selectedReport` alone, by `check`'s rules ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore), [§FS-check.2.1.2](FS-check.md#212-selection-filters-the-complete-report)), `onlyRule` requiring `rule`. `selectedReport` also holds one selected-check-off warning per `only` code the run left off, which the complete report does not ([§FS-check.1.4.1](FS-check.md#141-a-selected-check-the-run-leaves-off-says-so)). |
| `scan(root: string): Promise<ScanResult>` | Explicit path; no checker flags | Declarations, citations and scan facts as arrays, not a live iterator; a strict scan failure rejects with the partial facts and cautions it has. |
| `show(id: string, options?: ShowOptions): Promise<ShowResult>` | section=null, mode=lead, format=text; modes lead/brief/toc/full, formats text/md/json | A query refusal rejects `query` with its sites or candidate IDs; a setup failure rejects `config` or `io`. An inline section beside the `section` option is a query failure the engine decides, never a JS parser's reinterpretation. |
| `showBatch(queries: readonly (string \| ShowQuery)[] \| null, options?: BatchOptions): Promise<BatchResult>` | null means all coordinates; [] is empty without config loading; mode=lead, globs=[] (null with globs generates and filters; explicit queries with globs reject `input`) | Ordered `{query,ok:true,result}` or `{query,ok:false,failure}` records, plus note and cautions. The whole input is validated before discovery; a failed query does not stop the rest, and a setup failure rejects once. No stdin or NDJSON input. |
| `refs(id: string \| null, options?: RefsOptions): Promise<RefsResult>` | section=null, descendants=false, globs=[], from=null; id, globs and from select one mode, and mixing them rejects `input` ([§FS-coordinate-globs.bindings](FS-coordinate-globs.md#bindings-node-and-python)) | A valid undeclared target resolves with its hits and note; an invalid ID, an ambiguity, a refused glob or a refused `from` source rejects `query` with the context it has. |
| `list(options?: ListOptions): Promise<ListResult>` | kinds=[], projects=[], unused=false, selector=null, globs=[] | An unknown or non-citable kind, an unknown project, a project filter without a workspace, an invalid or unresolved selector, and a refused glob ([§FS-coordinate-globs.bindings](FS-coordinate-globs.md#bindings-node-and-python)) reject rather than resolve empty. |
| `listSizes(options?: SizeOptions): Promise<SizeResult>` | list options (globs included) plus units=[lines,words,bytes], top=null; units nonempty, unique, caller ordered; top positive | Each entry has one nullable lead and full measurement per unit, in the caller's order; a broken stub's stay null. Filters apply before `top`, the first unit ranks, and ties keep the CLI's stable order. |
| `cover(options?: CoverOptions): Promise<CoverResult>` | globs=[] ([§FS-coordinate-globs.bindings](FS-coordinate-globs.md#bindings-node-and-python)) | Entries include empty files, and every citation, site, project and enclosing coordinate, or those the globs select. `cover`'s text rendering is not another operation: the complete data supplies it. |
| `fmt(options?: FmtOptions): Promise<FmtResult>` | write=false, marker=false, crossRefs=false | Previews by default and adds no check flag or exit decision. A partial run the engine completes resolves with its diagnostics; a fatal preflight or a later write error rejects with the partial output. A conflict refusal rejects with kind `operation` and code `conflict-marker`, carrying path, line, sites, empty partial changes and `details.conflicts` as Python does ([§FS-distribution.3.3.2](FS-distribution.md#332-operational-failures-and-invalid-arguments)). Config-enabled cross-references still run when crossRefs=false. |
| `proposeId(kind: string, title: string, options?: IdOptions): Promise<IdResult>` | width=3, nonnegative safe integer | An unknown or non-citable kind rejects `operation` with the known kinds; an empty slug or a collision rejects `query`. No write and no implicit declaration. |
| `init(target?: string, options?: InitOptions): Promise<InitResult>` | name=null, description=null, docs=false, force=false, dryRun=false, check=false, noVcs=false, agents=null (automatic) | Writes by default, unlike fmt; check implies dryRun. A preflight refusal rejects; completed validation findings resolve; a partial write failure rejects with its events. The home, global and VCS guards, protected config and companion bytes hold as in the CLI. |
| `completeIds(options?: CompleteOptions): Promise<CompleteResult>` | prefix="", sections=false | Sorted, deduplicated candidates, with the engine's aliases and separators. An engine error rejects; only the shell's keystroke frontend stays silent on errors. |
| `effectiveConfig(options?: RootOptions): Promise<ConfigResult>` | none | The discovered config with effective defaults; no declaration scan and no Rust `Config` object. |
| `validateConfig(options?: RootOptions): Promise<ConfigResult>` | none | The same record; expands member configs as the engine does and scans no declaration body. An invalid config rejects `config` with its location and earlier cautions. |
| `fetch(id: string, options?: RootOptions): Promise<FetchResult>` | The call is the write authorization; no preview flag | Captures integration output as the engine does, with typed `query`, `config`, `io` and `operation` failures. The only operation that runs a configured integration; nothing fetches implicitly. |
| `integrations(options?: IntegrationOptions): Promise<IntegrationResult>` | client=null, write=false, conversation=null, conversationTarget=null, agent=null; client set codium/iterm2/kitty/tmux/vscode/wezterm; conversation plain/link; target file/path/web/vscode/vscodium/cursor | User-global, with no root. Validation, user config before write, the active-agent gates and owned paths hold as in the CLI, and only an explicit write mutates. No JSON install mode the CLI lacks. |
| `referenceStyle(path: string): Promise<StyleResult>` | explicit document path | Member configuration wins exactly as in the engine. |
| `agentSetupInstructions(): Promise<SetupResult>` | none | The whole setup guide, the text `grund guide --full` prints ([§FS-init.5.5](FS-init.md#55-the-core-and-its-sections)), with empty cautions, returned and never printed. No section argument: the caller chooses what of the guide reaches a model. |

The table's option names and defaults are the exact public ones. `agents` is null or omitted for automatic selection, otherwise a nonempty readonly array of canonical/claude/gemini/pi/copilot/cursor/windsurf/zed, each one of `init`'s agent entrypoint selections, and `[]` rejects rather than silently selecting automatically. An integration agent name is one of the closed set the CLI knows, and any other rejects `input`. conversation, conversationTarget and agent require write=true, agent also requires conversationTarget, and a write with no client requires conversation or conversationTarget. Validation precedes any mutation.

CLI switches that only shape output, such as `refs`' total and summary, `id`'s explain and `fmt`'s exit code, become returned fields rather than options: `refs`' `summaries` and `totals` are derived from its hits, with byte-sorted paths and site counts. `showBatch` takes no format, because typed results and canonical JSON are always returned, and the bindings project the CLI's JSON batch bytes, the one batch format that is data; its `text` and `md` framing is for reading ([§FS-show.3.3](FS-show.md#33-batch-query)).

The process and editor surfaces are excluded. Help and version dispatch, argv, exit codes, SIGPIPE, static shell completion-script printing and the CLI and LSP launchers stay processes. LSP snapshots, overlays, hover, on-type edits and the protocol lifecycle stay the LSP's, so completion and `referenceStyle` are the only editor operations here, and there is no generated editor plugin and no watch or streaming API. Build and engine version metadata serves package matching privately and is not another engine API. `cover` line ownership ([§FS-cover.6](FS-cover.md#6-line-ownership)) is not in the inventory, so `cover` takes no `lines` option.

#### 3.2.2 Typed readonly host records

Records are ordinary JS objects whose TS properties are readonly, and their arrays iterate synchronously once the Promise resolves: no persistent native handle, session object or live iterator, and no promise to mirror a Rust layout. An optional result field is present as null, never omitted or undefined, and an empty collection is `[]`. An integer crossing as a number stays within `Number.MAX_SAFE_INTEGER`: an overflow rejects `operation` with code `numeric-overflow` and never truncates. Coordinates keep the engine's one-based byte columns and are never converted to UTF-16.

```ts
type Site = { readonly path: string; readonly line: number };
type Finding = {
  readonly code: string; readonly message: string;
  readonly path: string | null; readonly line: number | null;
  readonly column: number | null;
  readonly sites: readonly Site[]; readonly authority: readonly string[];
} & ({readonly severity: 'error' | 'warning'} |
     {readonly channel: 'suggestion'});
type Report = {
  readonly errors: readonly Finding[];
  readonly warnings: readonly Finding[];
  readonly suggestions: readonly Finding[];
};
type Cautions = { readonly runCautions: readonly Finding[] };
type ScanError = { readonly path: string; readonly message: string; readonly code: string };
type FailureKind = 'input' | 'query' | 'config' | 'io' | 'unparsable-file' | 'operation'
                 | 'load' | 'worker' | 'native' | 'busy';
type Failure = {
  readonly kind: FailureKind; readonly code: string;
  readonly operation: string; readonly message: string;
  readonly path: string | null; readonly line: number | null;
  readonly column: number | null; readonly sites: readonly Site[];
  readonly authority: readonly string[];
  readonly causes: readonly string[];
  readonly details: Readonly<Record<string, JsonValue>>;
  readonly partial: PartialOutcome | null;
  readonly runCautions: readonly Finding[];
};
type JsonValue = null | boolean | number | string |
  readonly JsonValue[] | {readonly [key: string]: JsonValue};
// The emitted declarations enumerate these operation-specific unions;
// they do not publish unknown/any as the payload.
type PartialOutcome =
  | {readonly operation: 'scan'; readonly result: ScanSnapshot}
  | {readonly operation: 'refs'; readonly result: RefsData}
  | {readonly operation: 'fmt'; readonly result: FmtData}
  | {readonly operation: 'init'; readonly result: InitData}
  | {readonly operation: 'integrations'; readonly result: IntegrationInstallData};
declare class GrundError extends Error { readonly failure: Failure }
```

`kind` is the stable discriminant, and `code` is a documented machine code, not a localized sentence. Query codes are `not-found`, `missing-section`, `broken-stub`, `ambiguous`, `ambiguous-section`, `invalid-id` and `query-failed`; other codes identify `invalid-argument`, `unknown-option`, `conflicting-options`, `path-encoding`, `invalid-config`, `filesystem`, `operation-failed`, `native-load`, `worker-failed`, `native-panic`, `writer-busy` and `numeric-overflow`. Engine codes and details can extend this catalog compatibly, so callers branch on `kind` and the codes they know. A query failure's details carry the candidates and `formatHint` where the engine supplies them, an I/O failure the OS code when there is one, and a load failure the expected target, addon and versions, the locations tried and the cause chain; no source location is invented. Import succeeds without a compatible addon, and the first call rejects `load`; a JS package too corrupt to import is a module-system error outside any operation's Promise. Errors do not print, and ESM and CJS share one `GrundError` class.

Every result record below adds `runCautions`, and warnings a native output carries move there once rather than repeating in the data. Other fields have fixed camelCase host names; the canonical corpus converts them to snake_case and never renames an authored ID or configuration key ([§FS-distribution.3.0.3](FS-distribution.md#303-complete-data-and-canonical-parity)). The engine's output format is kept as metadata, never as leave to return a string in place of typed data.

- CheckResult: `report`, `selectedReport`, `hadScanErrors`, `outputFormat`. `report` is complete under the requested scope and options, and `selectedReport` is selected from it by `check`'s selection, so with no filter the two are equal. `hadScanErrors=true` means the run is incomplete, even when `selectedReport` has no errors.
- ShowResult: `id`, `section:string|null`, `kindTitle:string|null`, `body`, `path`, `line`, `sections: {path,title,depth}[]`, `json: string|null` (the engine's exact JSON when requested), `manifest:{kind:'E2E',args:string[],expectedExit:number,fixtures:string[]}|null`. Source, E2E and value bodies keep the engine's meaning rather than reading as prose, and all four slice modes and all three single-show formats exist.
- BatchResult: `records: {query:{id,section:string|null},ok:true,result:ShowData} | {query,ok:false,failure:Failure}[]`, `note:string|null`. ShowData has ShowResult's fields but its cautions, the mode is the call's, and each query's canonical JSON is kept.
- RefsData: `outputFormat`, `workspace`, `kindTitle:string|null`, `note:string|null`, `hits`, `scanErrors`, `summaries:{project:string|null,path,count,lines:number[]}[]`, `totals:{sites,files}`. Every RefHit has project/path/line/column/id/section/marker/text/enclosingDeclaration/enclosingSection, a coordinate it lacks staying null. Summary lines keep the CLI's duplicate-line policy and order.
- ListResult: `outputFormat`, `workspace`, `entries`, `summaries`, `scanErrors`, `note:string|null`. Entry: project/id/section/sectionSeparator/kind/path/line/title/stub/defines/refs/duplicate/valueRoots:{id,valid}[], of which project, section, title and defines are nullable. Summary: project/kind/title/home/count. The wire form omits a key only where the CLI's shape does, and kind titles and the authored project and kind order are kept.
- SizeResult: outputFormat/workspace/scanErrors/note:string|null/entries. Size entry: project/id/section/sectionSeparator/kind/path/line/stub/defines/duplicate/measurements:{unit:'lines'|'words'|'bytes',lead:number|null,full:number|null}[].
- CoverResult: outputFormat/scanErrors/note:string|null/entries:{project,path,citations:RefHit[]}[], losing neither empty files nor the citing side's membership.
- FmtData: changes:{path,line,label,counts,lockFile}[]/scanErrors/refusedWrites:string[]. A completed partial run carries its changes and scan errors, and a refusal's partial output may hold no change; no rollback or atomicity across files is promised. `label` is the CLI's preview row after `path:line: `, byte for byte, with its kinds ([§FS-fmt.3.5](FS-fmt.md#35-the-dry-run-report)), the text an expansion or a link writes ([§FS-fmt.3.6](FS-fmt.md#36-an-expansion-or-a-link-names-the-text-it-writes)) and any `; creates the file` or `; deletes the file` ([§FS-section-lock.write.report](FS-section-lock.md#writereport-what-fmt-reports)), so a caller comparing `label` exactly breaks as an exact-line script does; `counts` and `lockFile` are the stable reading. `counts:{triggersMarked,citationsMarked,localSectionsExpanded,shorthandsExpanded,linksWritten,sectionLockEntries}` are the change's occurrences of each kind, the numbers the `--write` totals sum ([§FS-fmt.3.7](FS-fmt.md#37-the---write-report)); `lockFile:'created'|'deleted'|null` is set on a lock row whose write creates or deletes its file, and null elsewhere.
- IdResult: id/kind/number:number|null/slug/folder:string|null/file:string|null/e2eCaseDir:string|null/fileHoldsSingleDeclaration:boolean.
- InitData: events:{verb,path}[]/errors:Finding[]/notes:string[]/next:{docs:boolean,declaresKinds:boolean,entrypoint:string,scanReadsFile:boolean,fsHome:Home,allocation:{kind:string,home:Home|null,homeExists:boolean}|null}|null/pendingChanges:boolean, where Home is {kind:'file',path:string,headingName:string,headingMarker:string} | {kind:'folder',path:string}.
- CompleteResult: ids:string[]. FetchResult: id:string (the requested operand).
- ConfigResult: root:string/configFile:string|null/values:ConfigValues. ConfigValues is a readonly, typed tree of the current public configuration keys, version/project_name/project_description/reference/id/kinds/scan/fmt/output/workspace, spelled as the TOML schema spells them. It holds only effective keys the schema knows, optional entries null where absent, and rules, kind order, resolution, fetch and inheritance keep their meaning. `root` and `configFile` say where the configuration was discovered; no per-key provenance is given.
- StyleResult: marker/trigger. SetupResult: instructions.
- IntegrationResult is discriminated by `mode:'detection'|'artifact'|'install'`. Detection has detected:string[] and clients:ClientDescriptor[]; artifact has client:ClientDescriptor/artifacts:{path:string|null,content:string}[]/manualSteps:string[]; install has events:{verb,path}[]/notes:string[]/preferences:{conversation,conversationTarget,agentOverrides}/manualSteps. Descriptor: client/kind/detected/installed:boolean|null/installKind/configTarget/resolverTarget:string|null, a manual client keeping installed=null. User-global paths may be absolute, as integrations already documents.

ScanResult exposes `snapshot:ScanSnapshot`, whose arrays keep the engine's deterministic order: declarations, citations, escapedCitations, scannedFiles, walkedDirs, fileStructures, valueBindings, invalidValueDeclarations, invalidValueBindings, sectionHeadingsOutsideDeclarations, unmarkedHeadings and nearMissHeadings. Declaration groups are flattened by rendered ID, path and line, keeping every duplicate site, and sections are ordered entries keeping duplicate section claims apart. Declaration records: id/kind/number/slug/path/line/headingLevel/title/sections/duplicateSections/stub/defines/e2eCase/bodyStart/bodyEnd/bodyHasContent/source/valueValid. Citation records: namespace/id/section/path/line/column/marker/shorthand/localSection/shorthandRewritable/numericRun/text/inlineSite/enclosingDeclaration/enclosingSection. A front-matter coordinate is a Section record like any other, with `headingLevel` `0` and no new field ([§FS-fields.front-matter.surfaces](FS-fields.md#front-mattersurfaces-what-each-surface-does-with-them)). An `e2eCase`'s `dir` is the case directory spelled exactly as the same record's `path`, from the same report base, reading `.` when the case directory is that base ([§FS-cli.3.4](FS-cli.md#34---path-base--where-report-paths-are-spelled-from)), so it does not change with where the tree sits; its `fixtures` stay relative to that directory. Each structural or value fact is a fixed typed record of its source fields in camelCase, with paths rendered by the engine; `source` is discriminated `kind: 'text'` versus `kind: 'json'` with memberSlice/keyColumn/keyText/format. `kind: 'json'` means a member of a structured value source ([§FS-values.terms](FS-values.md#terms-terms)) of either format, and `format: 'json' | 'yaml'` says which, so a consumer that reads only `kind` keeps working. Value numbers stay normalized coefficient and exponent strings or source strings, never lossy JS floating values. A strict scan failure rejects with the snapshot it has as its partial, and the first failure's message is its reason and *Fix*, as `check` prints it ([§FS-check.2.4.3](FS-check.md#243-what-failed-and-where-decide-the-code-and-the-fix)).

#### 3.2.3 Workers, loading and source builds

Engine work, config, filesystem, scan, query and write alike, runs off the JS thread and never in a subprocess of the CLI, so the event loop keeps turning during a call. A call validates and copies its operands before its first `await` and captures the working directory once, so changing either afterwards changes nothing it does, and `worker_threads` environments are supported. The binding never changes the global working directory, parses argv, exits its host or writes to stdout or stderr.

A second mutating call while one runs rejects `busy` with code `writer-busy` rather than queueing, across roots: `fmt` with write, `init` unless check or dryRun, `fetch`, and `integrations` with write. Previews and reads take no lease, and the lease is held from before the engine's first side effect until the call ends, on success, error or unwind. Independent reads overlap, and a read concurrent with a writer may observe its changes: no snapshot across calls or against an external writer is promised, so callers serialize reads and writes when they need a fixed tree. Separate installed copies and processes are separate native instances and coordinate nothing.

A recoverable panic in the engine or at a native boundary rejects `native` with code `native-panic`, prints nothing, and leaves the panic hook of any other native Rust caller in place; a typed engine error stays an error, never a panic. An abort, an out-of-memory or a corrupted process is not contained. Each Node environment has its own addon context: at teardown it accepts no new job, discards callbacks to the destroyed environment, and releases a job's resources and lease once its running work completes, and a pending call keeps the host alive. A forced worker termination may let a native write in flight finish, with no rollback, and there is no cancellation, `AbortSignal` or watch API. The binding refuses a workspace member whose unnamed path is not Unicode before deriving its alias, through the preflight the Python binding runs and the CLI does not ([§FS-distribution.3.3.3](FS-distribution.md#333-per-call-scope-and-path-encoding)).

The binding supports Node 22.x and 24.x with N-API 8, as ESM and CJS, and every registry row proves both ([§FS-distribution-candidate.1.2](FS-distribution-candidate.md#12-every-registry-row-proves-every-runtime-it-promises)). It has a prebuilt addon for each of the five registry rows of [§FS-distribution-candidate.1.1](FS-distribution-candidate.md#11-six-rows-five-of-them-registry-rows), whose npm suffixes are `linux-x64-gnu`, `linux-arm64-gnu`, `darwin-x64`, `darwin-arm64` and `win32-x64-msvc`, and none for musl or Windows arm64; npm `engines` is `^22.0.0 || ^24.0.0`, which declares those majors and promises no newer Node host, nor Electron, Bun, Deno or a browser.

ESM and CJS share one lazy loader, so they share errors and option semantics, and the package's public exports are exactly the functions of [§FS-distribution.3.2.1](FS-distribution.md#321-operations-options-and-refusals)'s table and `GrundError`, named under both `import` and `require`. The loader takes the first of a local source-built `native/grund.node` whose metadata matches and the optional platform package of exactly the same version selected by OS, CPU and libc ([§FS-distribution-candidate.2.2](FS-distribution-candidate.md#22-npm-selects-one-platform-package-by-os-cpu-and-libc)), and it validates an addon's metadata before accepting a request. A present local artifact that is incompatible or corrupt fails rather than falling back, and there is no architecture substitution, network download, CLI subprocess or silent rebuild: a selection or load failure rejects `load` with the target, the paths tried and the original cause. A host without a platform package gets that actionable error and an explicit `npm run build:source`, never a download or an automatic compile ([§FS-distribution-candidate.3.3](FS-distribution-candidate.md#33-a-missing-payload-is-an-error-never-a-substitute), [§FS-distribution-candidate.4.1](FS-distribution-candidate.md#41-npm-builds-from-source-only-when-asked)). `import` maps to `index.mjs` with `index.d.mts` and `require` to `index.cjs` with `index.d.cts`, each `types` condition before its implementation, and the root `types` may point to `index.d.mts`. The package holds the JS and TS declarations, the loader, its README and licence, the native payload when built, and, for the explicit fallback, a minimal complete Cargo source closure: the workspace and package manifests, the lockfile, the toolchain file, the core and node sources and the core assets. It has no LSP dependency and no `grund-lsp` bin, the assembled package's `grund` launcher and platform `optionalDependencies` are [§FS-distribution-candidate.3.1](FS-distribution-candidate.md#31-payloads-sit-where-each-registry-runs-them)'s and [§FS-distribution-candidate.2.2](FS-distribution-candidate.md#22-npm-selects-one-platform-package-by-os-cpu-and-libc)'s, and no addon or build path is a public export.

`npm run build:source` runs `node build/source.mjs`, which builds the addon from the package's bundled `build/sources/` closure and accepts an explicit `--target-dir`. It runs Cargo with `--locked` at the native builder's pinned toolchain, 1.95.0, for the host target by default, and needs that toolchain, a linker and SDK, Node and npm, and dependency access or populated caches. From a checkout, `crates/grund-node/stage.mjs` stages a disposable API-only package from the workspace version, labelled a rehearsal and claiming no CLI executable, and leaves its generated files outside the checkout. The package takes the workspace version by the one mapping of [§FS-distribution-candidate.6.2](FS-distribution-candidate.md#62-one-version-across-every-package), and a removal or rename of its records or operations takes [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path), or before `1.0` [§REQ-backwards-compatibility.4](../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise).

### 3.3 Python (`grund` PyPI package)

```python
from grund import check, show

repo = "tests/e2e/cases/json-report/repo"
result = check(repo)
for finding in result.report.errors:
    print(finding.code, finding.line)  # dangling 3
assert "FS-999-missing" in show("FS-001-alpha", root=repo, mode="brief").body
```

The binding is a PyO3 extension built with maturin, distributed and imported as `grund` ([§FS-distribution.1.1](FS-distribution.md#11-package-names)), whose calls are synchronous and return frozen records. One `cp310-abi3` wheel per registry row carries the extension and the `grund` executable and is proved on CPython 3.10 through 3.14 ([§FS-distribution-candidate.1.2](FS-distribution-candidate.md#12-every-registry-row-proves-every-runtime-it-promises)), and an sdist that builds both is the explicit fallback ([§FS-distribution-candidate.4.2](FS-distribution-candidate.md#42-python-builds-from-the-sdist-only-when-asked)).

#### 3.3.1 Typed immutable results

Results and options are frozen dataclasses; collections are tuples, with explicit None for optional fields, and an empty collection never becomes None. Effective config is a read-only schema-keyed mapping, nested mappings included, rather than an exported internal Rust config record. CheckResult carries `report`, `selected_report`, `had_scan_errors`, `output_format` and separate `run_cautions`. Report has errors/warnings/suggestions tuples, and iteration yields those groups in that order, each in engine order. Findings keep severity or channel, code, path, line, column, message, sites and authority. Scan returns a snapshot of the catalog, the citations and the scan diagnostics. Every other operation's record keeps every supported engine output field and exposes run_cautions separately. ShowQuery is a frozen record with `id` and a nullable `section=None`. Batch records have nullable `result` and `failure` fields. Refs includes file summaries and site and file totals; list includes summaries.

#### 3.3.2 Operational failures and invalid arguments

GrundError has ConfigError, FilesystemError, QueryError and fallback OperationError subclasses. Each carries a typed `.failure`: code, message, nullable path/line/column, sites, authority, causes, run_cautions, partial_output and structured details (such as candidates and OS error codes). Locations come from engine data, never text parsing. Completed checks containing findings return normally. A single unsuccessful query raises QueryError; a batch continues with the same failure per record, while a setup failure raises once, and empty batch input succeeds without config loading. An unknown ID kind in propose_id raises OperationError; rejected queries raise QueryError. An fmt refused for unresolved merge conflicts ([§FS-fmt.3.2](FS-fmt.md#32-a-run-that-needs-the-whole-declaration-set-rewrites-nothing)) raises OperationError with code conflict-marker: path and line are the first block's first line, sites holds each block's first line, partial_output holds empty changes, scan_errors and refused_writes, and details.conflicts holds every block's finding ([§FS-declarations.checks.conflict-marker](FS-declarations.md#checksconflict-marker-unresolved-merge-conflict)). An fmt whose walk fails to read a file carries partial_output with empty changes ([§FS-fmt.7.4](FS-fmt.md#74-no-write-without-a-complete-model)). Wrong Python types raise TypeError; invalid option values raise ValueError. PathEncodingError is a ValueError subclass for rejected path encodings.

#### 3.3.3 Per-call scope and path encoding

Paths accept str or os.PathLike[str]. root=None snapshots the working directory at entry and means omitted scope, while explicit files and directories keep the engine's explicit-path meaning. Config is discovered as the engine discovers it, never injected, and calls against different roots are independent. Report paths are the engine's logical strings with `/`, whatever the native filesystem inputs. Before any work, bytes or a byte-returning PathLike raise TypeError, surrogates PathEncodingError and an embedded NUL ValueError, and the bindings' preflight, through workspace discovery, rejects a workspace member whose unnamed path is not Unicode with PathEncodingError before deriving its alias. Only the bindings run that preflight: the CLI derives such a member's alias without it, a known deviation. The guard promises no recovery from an arbitrary Rust panic.

#### 3.3.4 Silent synchronous calls

Calls read no process argv, print nothing, never exit Python and never change the global working directory. Synchronous Rust work releases the GIL. Independent reads may overlap; callers serialize writers to the same files. Pending Python interrupts are checked on return, so Rust may finish before KeyboardInterrupt, and an interrupt arriving after the call is not evidence of loss. No async API, mid-call cancellation or additional rollback is promised.

#### 3.3.5 Complete inventory

Required operands are positional and all other arguments are keywords, and the names and defaults below are the exact public ones. Every disk-tree operation also accepts root=None, and check accepts its root positionally; scan requires its root positionally, init takes target in place of root, and integrations and the setup instructions have no root.

| Operation signature (besides common root) | Result/core meaning |
| --- | --- |
| `check(root=None, *, require_grounding=False, suggestions=False, full=False, rule=None, only=(), ignore=(), only_rule=False)` | Complete and selected report, warning-preserving check |
| `scan(root)` | Raw scanner snapshot |
| `show(id, *, section=None, mode="lead", format="text")` | Scoped show result |
| `show_batch(queries=None, *, mode="lead", globs=())` | None means all, filtered by any `globs`; ordered strings/ShowQuery records |
| `refs(id=None, *, section=None, descendants=False, globs=(), from_=None)` | Warning-preserving metadata/totals; one of id, `globs`, `from_` selects the mode |
| `list_ids(*, kinds=(), projects=(), unused=False, selector=None, globs=())` | Entries and summaries |
| `list_sizes(*, kinds=(), projects=(), unused=False, selector=None, globs=(), units=("lines","words","bytes"), top=None)` | Lead/full sizes |
| `cover(*, text=False, lines=(), globs=())` | Structured or text coverage data, `globs` filtering its citations; non-empty `lines` — strings `"N"` or `"N-M"`, each as `--lines` takes it, with `root` naming the file and `text` false — gives line ownership ([§FS-cover.6](FS-cover.md#6-line-ownership)) |
| `fmt(*, write=False, marker=False, cross_refs=False)` | Format preview or managed writes |
| `propose_id(kind, title, *, width=3)` | Warning-preserving ID proposal |
| `init(target=None, *, name=None, description=None, docs=False, force=False, write=False, check=False, no_vcs=False, agents=None)` | Scaffold output; None agents auto-selects |
| `effective_config()` / `validate_config()` | Config/schema data and cautions |
| `fetch(id, *, write=False)` | Materialized snapshot; explicit write required |
| `integrations(client=None, *, write=False, conversation=None, conversation_target=None, agent=None)` | Detection/artifacts or managed install |
| `complete_ids(prefix="", *, sections=False)` / `reference_style()` | Completion/style data |
| `agent_setup_instructions()` | Canonical setup payload: the whole guide, the text `grund guide --full` prints ([§FS-init.5.5](FS-init.md#55-the-core-and-its-sections)) |

Modes are lead/brief/toc/full; formats text/md/json. Filters and selectors are string sequences. Units are lines/words/bytes; top is positive, width non-negative. Selectors narrow only selected_report, by `check`'s rules ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore), [§FS-check.2.1.2](FS-check.md#212-selection-filters-the-complete-report)), and selected_report also holds one selected-check-off warning per `only` code the run left off, which report does not ([§FS-check.1.4.1](FS-check.md#141-a-selected-check-the-run-leaves-off-says-so)). Scope, workspace aliases, exclusions and kind overrides are the engine's decisions, as they are the CLI's. Config-enabled cross-reference formatting applies even when cross_refs=False. Shell scripts, stdin/NDJSON transport, watch and process lifecycle, exit codes and LSP transport are frontend concerns, and the optional overlay, on-type and hover editor utilities are excluded from this disk-backed API; no CLI conceptual operation is dropped.

#### 3.3.6 Explicit mutation opt-ins

fmt previews by default. init maps write=False to a dry run, and check=True suppresses writes and reports pending changes. Force never replaces config. fetch refuses write=False with ValueError before execution; it promises no preview. Integration reads return artifacts or detection, and writes keep preference validation, the agent gates, managed ownership and manual steps. Reads never execute fetchers.

#### 3.3.7 Interpreters, source installs and typing

The binding requires CPython 3.10 or later with the GIL, and abi3-py310; PyPy and free-threaded Python support are not promised. The package ships its types and `py.typed`, `grund._native` is private, and a checkout installs the binding with `python -m pip install crates/grund-py`. A clean checkout and an independently unpacked sdist both install an importable grund without registry credentials; the sdist includes the necessary workspace manifests, the lockfile, Rust sources and assets, the Python code and types, and the licence. Python-only dependencies do not become ordinary Cargo CLI requirements, and the binding adds no competing CLI console entrypoint.

On supported desktop platforms, enabling `grund-py/extension-module` in a raw Cargo build produces an extension the supported interpreter imports, with no linker arguments from maturin, including when the Python and Node natives are built together from one checkout for command agreement ([§REQ-one-answer.proof.matrix](../requirements/REQ-one-answer.md#proofmatrix-the-complete-comparison-matrix)); on macOS the importing interpreter supplies the Python symbols. Ordinary workspace executables and tests get no extension-only linking.

## release-floor: One shared minimum publishable release

The tree records one minimum publishable release, the single line of
`scripts/release-floor.txt`, which every release transition shares; none adds a floor of
its own. The release guard, `scripts/check_release_ramps.py`
([§FS-distribution.4.2.2](FS-distribution.md#422-the-release-guard-reads-the-releases-the-tree-names)), reads it from the tree being checked and refuses every version
below it, even when no message names a landed change, and it still binds the tree a helper
rewrites for a release. The floor joins the landed bound the messages name as the release
window's inclusive lower bound, so a release at the floor is allowed when no pending line or
plan deadline ([§FS-distribution.4.2.2](FS-distribution.md#422-the-release-guard-reads-the-releases-the-tree-names)) forbids it, and a refusal names the floor's file and
value beside the messages it contradicts ([§FS-distribution.4.2.5](FS-distribution.md#425-the-refusal-names-the-window-left)). A development version
need not equal the floor.

## 4. Release process

`release.yml` publishes a release from a commit that already carries its version: a tag
push or a manual dispatch runs it directly, and two helper workflows make that commit from
`main` on a candidate they validate first, then dispatch it.

### 4.1 Between releases, main carries a dev version

After publishing `X.Y.Z`, the release helpers push `X.Y.(Z+1)-dev` to `main` as the
release's last act, a commit CI passed before it reached `main` ([§FS-distribution.4.4.1](FS-distribution.md#441-nothing-reaches-main-that-ci-has-not-passed));
a run that releases nothing opens none. So a build from `main` says by its suffix that it
is ahead of the last tag, and a fix merged but not installed cannot pass for one installed.
A `-dev` version is never published: `release.yml` publishes only a commit that carries
exactly the version it releases ([§FS-distribution.4.3](FS-distribution.md#43-releaseyml-publishes-a-commit-that-already-carries-its-version)).

#### 4.1.1 A development build names its commit

A development `grund`, one whose version carries the `-dev` suffix, reports it with the
commit it was built from as SemVer build metadata: `X.Y.Z-dev+<commit>`, or
`X.Y.Z-dev+<commit>.dirty` when a tracked file differs from that commit, in content or
mode, in the index or the working tree. Untracked and ignored files never make it dirty.
`<commit>` is exactly the first ten lowercase hexadecimal digits of the full SHA. Every
other version answer — `grund-lsp --version`, the server's `serverInfo.version`, and the
Node binding's `engineVersion` and `packageVersion` — is the bare version, and a release
version reads neither git nor `GRUND_BUILD_COMMIT`.

The build takes the commit from the first of these it has:

1. qualifying git: this source tree's own repository, whose top level is the workspace
   root, which git can read, which has a commit, and which tracks the CLI's manifest in
   `crates/grund-cli/`, the crate's own directory; a tree inside another repository and a
   crate copy nested in a checkout have none;
2. `GRUND_BUILD_COMMIT`: ten lowercase hexadecimal digits, optionally followed by
   `.dirty`, used as given;
3. nothing: the build reports the bare version.

A development build fails when `GRUND_BUILD_COMMIT` is set to anything else, the empty
string included, even when git supplies the commit.

A rebuild refreshes the label only when Cargo sees a watched input change: the
modification time of HEAD, the refs, the index or a tracked or changed file, whether
qualifying git exists, or `GRUND_BUILD_COMMIT`. A change that moves no watched modification
time, such as an unstaged executable-bit change or an edit whose time was restored, can
leave the old label. `cargo clean -p grund` from the workspace root, then the same build
with the same target directory and profile, forces a fresh one.

The label names where a build came from, not what it holds: two development builds are the
same version for [§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree) exactly when built from the same source contents,
uncommitted changes included, with the same build inputs, whatever their labels say
([§DF-dev-build-names-its-commit](../decisions/functional/DF-dev-build-names-its-commit.md#df-dev-build-names-its-commit-a-development-builds---version-names-its-commit-and-what-a-development-build-prints-between-releases-was-never-a-promise)).

### 4.2 A release may not contradict the releases the tree's own messages name

A ramp is a promise written into a message: a warning names the release it becomes an
error in, or the release a scalar status moves in ([§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)), and the
error that lands names the release it landed in. Shipping a pending warning at its deadline,
or a landed change below the release it names, breaks that promise. A test can hold only the
pending half: on `main` a landed message rightly names the release still to be cut, and the
release version exists only on a candidate, whose suite runs only once it is pushed. So the
release guard reads the release each message names and refuses a version that contradicts
one ([§DF-index-compatibility-ramp.2.3](../decisions/functional/DF-index-compatibility-ramp.md#23-both-findings-name-their-versions-and-a-test-keeps-the-names-honest)), and on a helper's path it asks before a candidate is
pushed ([§FS-distribution.4.2.6](FS-distribution.md#426-every-publication-path-runs-the-release-guard)).

#### 4.2.2 The release guard reads the releases the tree names

`scripts/check_release_ramps.py <version>`, the release guard, reads the release each
message names out of the tree's message text — the Rust sources under `crates/` and the
`expected.stdout` and `expected.stderr` goldens under `tests/e2e/cases/` — and refuses a
version that contradicts one or is below the floor ([§FS-distribution.release-floor](FS-distribution.md#release-floor-one-shared-minimum-publishable-release)).
`<release>` may also be written `grund <release>`:

| clause | what the line claims | the version being cut |
|---|---|---|
| `becomes an error in <release>` | the change has not been made yet | must be **below** that release |
| `are warnings until <release>` | the warning window has not closed yet | must be **below** that release |
| `became an error in <release>` | the change has been made | must be **at or above** it |
| `was removed in <release>` | the change has been made | must be **at or above** it |
| `is removed in <release>` | a named removal has not been made yet | must be **below** that release |
| `stopped loading in <release>` | the change has been made | must be **at or above** it |
| `unchecked in <release>` | the change has been made | must be **at or above** it |
| `an error in <release>` | the change has been made | must be **at or above** it |
| `will exit <status> ... in <release>` | a scalar exit-status change has not been made yet | must be **below** that release |
| `wording changes in <release>` | the message wording has not changed yet | must be **below** that release |

Given a version, the guard also treats it as shipped and refuses it where the releases
section of the core-concerns plan ([§DISC-core-concerns.9](../discussions/proposals/2026-09-30-core-concerns.md#9-the-releases)) still calls something due by it
unshipped: a `planned` notice in the deprecation ledger ([§DISC-core-concerns.9.9](../discussions/proposals/2026-09-30-core-concerns.md#99-the-deprecation-ledger)) whose
notice release is at or below it, or a phase whose lead opens `Not shipped` and names a
release at or below it, a series `0.N.x` being due once the version is past `0.N`. The plan
is the guard's only source of these dates and statuses. That refusal names the plan's file,
the notice's surface or the phase's number, and the release it is planned for.

#### 4.2.3 The vocabulary is closed

The clauses of [§FS-distribution.4.2.2](FS-distribution.md#422-the-release-guard-reads-the-releases-the-tree-names) are the whole vocabulary, on purpose: they are the
wording the messages already use, so a ramp written in them is read the day its message is
written, and one written outside them names no release the guard can read.
`is removed in <release>`, not "will be removed in", is the pending half of
`was removed in <release>`.

The bare `an error in <release>` is the landed half of the attribution pair a moved verdict
prints, `unchecked in grund <prior>, an error in <release>`. It is a substring of
`becomes an error in` and `became an error in`, so it is read only where it opens a clause
— at the start of a line, or after a comma, semicolon or colon — and each tense keeps the
direction it spells; a tense the vocabulary does not carry, such as `become an error in`,
is not read as landed.

The scalar clause `will exit <status> ... in <release>` has no landed half: once its change
is made the command prints its ordinary bytes, naming no release, and a contract test holds
that phase ([§FS-refs.4](FS-refs.md#4-exit-codes)).

#### 4.2.5 The refusal names the window left

A refusal names every line that disagrees and the window the tree may still be cut as: at
or above its lower bound, the greater of the landed lines and the floor
([§FS-distribution.release-floor](FS-distribution.md#release-floor-one-shared-minimum-publishable-release)), and below its ceiling, the earliest pending line. The window
is empty when the floor or a landed ramp reaches a pending line, and nothing may then be
published until the promises are kept or the changes reverted; its refusal names every
pending line that sets the ceiling, each once, and the version-free check names every landed
line that sets the lower bound too, so the lines listed are the whole set to land or revert.

`scripts/check_release_ramps.py [--root <tree>]`, given no release, checks only that the
window is not empty, reading the same sources and floor, excluding no ramp, and evaluating
none of the plan's deadlines ([§FS-distribution.4.2.2](FS-distribution.md#422-the-release-guard-reads-the-releases-the-tree-names)). It exits `1` exactly when the window
is empty, its lower bound at or above its ceiling, reporting `no release at all` and both
conflicting bounds with their source locations, `scripts/release-floor.txt:1` among them
when the floor sets the lower bound, and exits `0` for a nonempty or unbounded window, even
when the next patch would be refused.
A configuration or I/O error is an error, never an empty window.

#### 4.2.6 Every publication path runs the release guard

`release.yml`'s verify job runs the release guard on the version it is about to publish, on
every path into it: a `vX.Y.Z` tag push, a manual dispatch, and the non-publishing dry run
the helpers wait on before they touch `main`. `auto-bump.yml` and `release-minor.yml` run it
on the version they compute or select, so a bump that could not be published fails before
it pushes a candidate branch; selecting `1.0.0` bypasses neither the floor nor a pending
promise. The cross-registry publisher runs it on its manifest's version before it uploads
anything ([§FS-distribution-candidate.8.1](FS-distribution-candidate.md#81-publication-is-disabled-and-gated)). Every pull request runs the version-free check
([§FS-distribution.4.2.5](FS-distribution.md#425-the-refusal-names-the-window-left)) on the repository tree, with every scanned ramp, so one that would
leave `main` an empty window fails before it merges.

### 4.3 `release.yml` publishes a commit that already carries its version

A `vX.Y.Z` tag push runs `.github/workflows/release.yml`, and so does a manual dispatch from
the release commit, which takes the version, publishes crates by default, and creates the
tag when it does not exist. When the tag exists, the run publishes from it once the tagged
Cargo package versions match, so a failed release is recovered from a newer workflow commit
without moving its tag. The verify job checks the version against the three Cargo packages
it publishes ([§FS-distribution.4.10](FS-distribution.md#410-cratesio-publishes-in-dependency-order-and-artifacts-upload-last)) and every other version source
`scripts/distribution/candidate.py versions` reads ([§FS-distribution-candidate.6.2](FS-distribution-candidate.md#62-one-version-across-every-package)), runs a
fail-fast preflight on the crates.io token when publishing crates, and re-runs
`scripts/check-registry-names.sh`, so each claimed package name must still be available or
owned by this project ([§FS-distribution.1.1](FS-distribution.md#11-package-names)). It then builds and self-checks
profile-guided-optimized binaries ([§FS-distribution.4.9](FS-distribution.md#49-distributed-binaries-are-profile-guided-optimized)) for the six Rust targets of
[§FS-distribution-candidate.1.1](FS-distribution-candidate.md#11-six-rows-five-of-them-registry-rows). `release.yml` never bumps a
version, and it publishes the Cargo crates and the GitHub release and nothing else
([§FS-distribution-candidate.8.7](FS-distribution-candidate.md#87-the-cargo-release-cannot-reach-the-publisher)).

A manual run may name `expected_release_sha`, the full commit it is to publish; the verify
job then refuses, before anything is built, a source that is not that commit, naming both.
A release helper always names it ([§FS-distribution.4.4.1](FS-distribution.md#441-nothing-reaches-main-that-ci-has-not-passed)). A publishing run deletes the
`release-candidate/` branch it was dispatched from once it has published; the dry run never
does, because the publisher is dispatched on that branch afterwards.

### 4.4 Two helper workflows bump the version on a validated candidate

`.github/workflows/release-minor.yml`, dispatched by hand, selects `vX.(Y+1).0` from the
latest `vX.Y.Z` tag, or the first stable release ([§FS-distribution.4.4.2](FS-distribution.md#442-deliberately-select-the-first-stable-release-from-the-dispatch-main-tip)), once `main` has
commits since that tag and CI has passed on its tip. `.github/workflows/auto-bump.yml`,
scheduled or dispatched, selects `vX.Y.(Z+1)` when `main` has substantive non-doc/CI changes
since the latest tag and CI has passed on that exact tip. A patch below the lower bound of
the tree's release window ([§FS-distribution.4.2.5](FS-distribution.md#425-the-refusal-names-the-window-left)), which `scripts/release_earliest.py` prints,
is skipped like a run with nothing substantive and left to the manual helper; a patch the
window refuses for any other reason fails at the release guard.

Neither publishes directly: each commits the version bump ([§FS-distribution.4.5](FS-distribution.md#45-what-the-version-bump-includes)) to a
release-candidate branch, validates it ([§FS-distribution.4.4.1](FS-distribution.md#441-nothing-reaches-main-that-ci-has-not-passed)), and only then fast-forwards
`main` and dispatches `release.yml`, pushing with the release credential configured to
bypass branch protection. Neither waits for anything to be written, since the bump builds
the release section from what the repository holds ([§FS-distribution.4.6](FS-distribution.md#46-the-release-lists-the-pull-requests-merged-since-the-previous-tag));
`release-minor.yml` refuses a range with no merged pull request through the bump's refusal
([§FS-distribution.4.6.3](FS-distribution.md#463-a-refused-release-leaves-the-tree-as-it-was)). Neither dispatches the cross-registry rehearsal or publisher,
which are started by hand ([§FS-distribution-candidate.8.7](FS-distribution-candidate.md#87-the-cargo-release-cannot-reach-the-publisher)).

#### 4.4.1 Nothing reaches `main` that CI has not passed

CI on the `main` tip says nothing about the commits a helper writes on top of it, so each
helper builds both before it pushes either: the release commit on a release-candidate
branch, and the `-dev` advance over it on a second. It runs the CI a push to `main` gets on
each, and the `release.yml` dry run on the release commit, and fails on the first that does
not pass; only then does `main` move, to exactly those commits ([§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path)).

A helper not pinned to a tip — `auto-bump.yml`, and `release-minor.yml` without
`expected_main_sha` ([§FS-distribution.4.4.2](FS-distribution.md#442-deliberately-select-the-first-stable-release-from-the-dispatch-main-tip)) — follows a `main` the merge queue moves, in
rounds ([§DF-release-candidates-follow-main](../decisions/functional/DF-release-candidates-follow-main.md#df-release-candidates-follow-main-a-release-helper-follows-a-main-the-merge-queue-moves-rebuilding-and-re-running-its-candidates)). A round fetches `main` and the tags, builds both
candidates on that tip by the whole preparation — the bump ([§FS-distribution.4.5](FS-distribution.md#45-what-the-version-bump-includes)), the
release section ([§FS-distribution.4.6](FS-distribution.md#46-the-release-lists-the-pull-requests-merged-since-the-previous-tag)), the release guard ([§FS-distribution.4.2.6](FS-distribution.md#426-every-publication-path-runs-the-release-guard)) and the
advance — pushes them to two new branches, each one path component under
`release-candidate/` and never a name an earlier round used,
`release-candidate/vX.Y.Z-<run>-<attempt>-r<n>` and the same with `-dev`, and dispatches the
three runs, counting only the runs it started. When `main` moves, the round is given up, its
runs cancelled and its branches deleted, and once `main` has held still for three minutes
the next round rebuilds on the new tip and runs all three again.
`main` only fast-forwards, to the commits the last round's runs passed, so a release
dispatched while the queue lands pull requests ships them, and its section lists them. The
version the helper resolves, CI on the dispatch tip and the scheduled helper's
substantive-change filter are decided once; more merges cannot make the last two false, and
each rebuilt candidate's own CI covers the new tip. Before a round pushes, a target tag that
now exists, or version sources that no longer read what they read at the start, stop the
helper, naming the tag or both versions, with nothing published.

At most six rounds run, within the job's 300-minute timeout, and a round starts only while
at least 90 minutes remain. A round's watch ends early enough for the push, the dispatch and
the advance to fit after it, cancelling its runs and stopping the helper with its own
message. A helper out of rounds or time publishes nothing, says which ran out, and says to
dispatch it again.

The release commit goes to `main` by a plain push, never a forced one. When git refuses it,
the helper reads `main` back: at the release commit, the push landed; at the round's base,
the helper stops with git's error; at any other commit, the queue landed first, which is a
move. Once `main` holds the release commit, `release.yml` is dispatched on the round's
release-candidate branch with `expected_release_sha` set to it ([§FS-distribution.4.3](FS-distribution.md#43-releaseyml-publishes-a-commit-that-already-carries-its-version)), and
the advance follows; a failed dispatch says that the release commit has landed and how to
publish it, and does not start over. A rejected push of the advance repeats the same commit
a few times, with a short backoff, while `main` is still the release commit; a moved `main`
is refused as moved, and an unmoved one that keeps refusing names git's own error.

Only a failure keeps a round's two branches, for a person to read; a run that did not pass,
which cancels the rest and wins over a move seen on the same look, and a push git refused
onto the round's base are among them, and leave `main` untouched. A round given up for a
move, a helper out of rounds or time, and a pinned run refused on a move delete them. A
rebuild the preparation refuses, such as a ramp a merged pull request landed, stops the
helper before the round pushes a branch or starts a run.

#### 4.4.2 Deliberately select the first stable release from the dispatch main tip

`release-minor.yml` takes a `version` choice, `next-minor` by default or `1.0.0`, and an
optional `expected_main_sha`, a full 40-hex SHA that must equal the dispatch SHA, the
checkout's `HEAD` and the fetched `origin/main` tip; `1.0.0` requires it. It names the tip
the owner dispatches from, not the freeze candidate recorded for 1.0, and comparing the two
is the owner's responsibility.

Before any write or push, first-stable selection requires dispatch from `refs/heads/main`;
one `0.x` version across every source `scripts/distribution/candidate.py versions` reads
([§FS-distribution-candidate.6.2](FS-distribution-candidate.md#62-one-version-across-every-package)); a previous release, the highest stable `v0.x.y` tag
reachable from the tip ([§FS-distribution.4.6.1](FS-distribution.md#461-the-range-is-every-commit-since-the-previous-tag)), with commits since it; no stable tag at or
above `v1.0.0` on any branch; and every higher stable tag the tip does not reach being a
patch of the previous release on a branch off its tag. Prerelease tags are not stable
history. A missing or malformed input, an unsupported selection or an unmet requirement
refuses with a diagnostic naming the condition, and writes nothing.
`scripts/release_version.py` owns the selection, reading only local history and source
versions, and returns the helper's `current`, `next`, `current_tag` and `next_tag`; the
release guard runs on the result ([§FS-distribution.4.2.6](FS-distribution.md#426-every-publication-path-runs-the-release-guard)).

Both choices then take the path of [§FS-distribution.4.4.1](FS-distribution.md#441-nothing-reaches-main-that-ci-has-not-passed) unchanged, `1.0.0` adding no
wait on publication; for `1.0.0` the advance is `1.0.1-dev`, Python `1.0.1.dev0`. A pinned run
follows no move of `main`: a move cancels its candidate runs, deletes its branches and names
the new tip, to be dispatched again with a fresh pin. A failed publication is retried from
its release tag, as for a minor.

### 4.5 What the version bump includes

The version bump, in both helpers and for either manual choice, writes:

- the version into the Cargo manifests, the lockfile and `pyproject.toml` (in PEP 440),
  through `scripts/distribution/candidate.py set-version`; the npm packages take theirs at
  assembly, so no committed npm file carries one ([§FS-distribution-candidate.6.2](FS-distribution-candidate.md#62-one-version-across-every-package));
- every checked fixture whose expected output embeds `grund --version`, including
  `tests/e2e/cases/cli-version/expected.stdout` and its `inputFiles` bytes in
  `tests/integration/command_agreement_facts/cli-version.json`, through the same
  `set-version`, for a release and a development version alike. `set-version` replaces only
  the version's bytes in them, leaving every other input and independent expected answer as
  it was ([§REQ-one-answer.proof.evidence](../requirements/REQ-one-answer.md#proofevidence-execution-and-independent-truth)), and repeating it leaves them consistent. The
  golden holds the bare version even for a development version: a development build's test
  expects that line followed by its own `+<commit>` or `+<commit>.dirty`
  ([§FS-distribution.4.1.1](FS-distribution.md#411-a-development-build-names-its-commit)), and writes the bare line back when it refreshes the golden;
- every ramp constant whose window a message states as a version
  ([§DF-index-compatibility-ramp.2.3](../decisions/functional/DF-index-compatibility-ramp.md#23-both-findings-name-their-versions-and-a-test-keeps-the-names-honest));
- the changelog rotation of `scripts/prepare_changelog_release.py prepare`: the section
  [§FS-distribution.4.6](FS-distribution.md#46-the-release-lists-the-pull-requests-merged-since-the-previous-tag) builds becomes the new inline release section of
  `docs/changelog.md`, in the former one's place; the former one is archived under
  `docs/changelog/<version>.md`, and the older-release section gains
  `- [<version>](changelog/<version>.md) — <date>: N pull requests, M compatibility notices.`,
  M counting the archived bullets under `### Compatibility notices`, N every other bullet,
  and a count of one written in the singular. `preview` prints the body `prepare` would
  write under the new release heading, and writes nothing.

Each release and development candidate commit stages the whole update, and the bump refuses
before it writes anything in every case [§FS-distribution.4.6.3](FS-distribution.md#463-a-refused-release-leaves-the-tree-as-it-was) names, so a candidate is
e2e-clean and changelog-clean before `release.yml` runs. A test of the bump that cannot
remove its temporary git history cleanly fails, whatever its assertions said
([§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path)).

### 4.6 The release lists the pull requests merged since the previous tag

No change is gated on the changelog: no hook, CI job or release step asks a change, or a
person, for a changelog entry, and nothing waits in the tree for a release. The bump
([§FS-distribution.4.5](FS-distribution.md#45-what-the-version-bump-includes)) builds the release section from what the repository already holds —
the pull requests merged since the previous tag, by their titles, and the compatibility
notices the decision records gained since it — or refuses and writes nothing. So a pull
request's title is its release line, and the one record a change writes for a release, what
a verdict change breaks, is a notice on the decision that makes it ([§FS-distribution.4.6.4](FS-distribution.md#464-compatibility-notices-come-from-the-decisions),
[§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path)).

#### 4.6.1 The range is every commit since the previous tag

`HEAD` is read once, first, and everything after reads that commit. The previous release is
the highest `vX.Y.Z` tag reachable from it, never a higher tag on a branch `HEAD` does not
contain, and it must name the release `docs/changelog.md` keeps inline. For every commit in
`<tag>..HEAD` the bump asks the forge which pull requests it belongs to, and lists, once,
each that is merged, has base `main` and has its merge commit in the range; merges are
rebase-only, so that finds every pull request merged into it. Every one is listed, with no
docs or CI filter, because a pull request that changes only `docs/` can change what the tool
promises. A commit in no pull request adds no line; any but the
`Open X.Y.Z-dev for development` commit ([§FS-distribution.4.1](FS-distribution.md#41-between-releases-main-carries-a-dev-version)) is named, by short hash and
subject, in one warning on standard error, and the release goes on. The list is newest
first, by where each pull request's last commit sits on `main`, read from git and never from
the forge's timestamps.

#### 4.6.2 One line per pull request, its title linked

Each pull request is one line, `- [<title>](<url>) (PR #N)`. The title is the pull request's
own, with every run of whitespace collapsed to one space and the ends trimmed; each of
`` \ ` * _ [ ] < > & `` in it is escaped with a backslash, and `§` is written `&sect;`,
because `docs/changelog.md` is scanned ([§FS-check](FS-check.md#fs-check-grund-validates-every-citation-in-a-repo)) and a title must not become a live
citation the release is then held to. The URL is `<repository>/pull/N` for the same `N`,
where the repository is the one the forge answers for; a pull request whose URL is anything
else is refused ([§FS-distribution.4.6.3](FS-distribution.md#463-a-refused-release-leaves-the-tree-as-it-was)).

#### 4.6.3 A refused release leaves the tree as it was

The bump reads everything it needs — the range, every commit's pull requests and every
notice it would publish — before it writes anything. It refuses, exits non-zero, names the
case it met and leaves every file byte for byte as it was, when:

- `gh` is missing, a forge call still fails after three attempts, or any commit in the
  range goes unanswered;
- the checkout is shallow, or the previous tag does not name the release
  `docs/changelog.md` keeps inline;
- a notice it would publish is malformed ([§FS-distribution.4.6.4](FS-distribution.md#464-compatibility-notices-come-from-the-decisions)), or a pull request's URL
  is not this repository's ([§FS-distribution.4.6.2](FS-distribution.md#462-one-line-per-pull-request-its-title-linked));
- the range holds no merged pull request.

A forge call, for the repository or for a commit's pull requests, is tried at most three
times, one second after the first failed attempt and two after the second. An attempt fails
when `gh` exits non-zero, whatever it printed, or exits zero with output that is not JSON,
both against the one budget; after the third, the bump refuses with that attempt's own
diagnostic, `the forge did not answer for …` or `the forge's answer for … is not JSON: …`.
A missing `gh`, and an answer that parses but has the wrong shape — not a list for a
commit, no address for the repository, a URL that is not this repository's — are refused at
once, never retried.

#### 4.6.4 Compatibility notices come from the decisions

A decision record, a `DF` or `DA` declaration, may carry the named section
`## release-note: Release note`, holding exactly one bullet (`- ` and its text,
continuation lines indented two spaces) and nothing else but blank lines, its links
relative to the record's own file. A verdict correction must carry one, as the release
record [§REQ-backwards-compatibility.5](../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids) condition 3 and [§REQ-backwards-compatibility.3](../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) ask
for; `tests/integration/test_backwards_compatibility_routes.py` reads it there, offline, on
the pull request that makes the correction.

The bump publishes, under `### Compatibility notices` after the list, the bullet of every
decision whose `release-note` section exists at `HEAD` and did not at the previous tag,
ordered by decision ID; the two trees are compared by decision ID, so a moved or renamed
record is the same record. Each bullet is published as written, its relative links rebased
from the record's directory to `docs/`, and a link to an anchor of its own file gaining that
file's path. A notice present at the tag is never published again, however it was edited
since, and a release that adds none has no `### Compatibility notices`.

The repository's Python gate, at every commit and on every pull request, reads every
decision record's `release-note` section with the bump's own reading and fails, naming every
such decision and its file in one run, when one is not a single well-formed bullet; a
decision that writes none is not read. The correction-route meter audits every decision
whose text holds the live citation `§REQ-backwards-compatibility.5`, even one that discusses
or rejects the route, and a refusal says so, keeps every placement or proof reason, and
names the two ways to mention the route without invoking it:
`<§>REQ-backwards-compatibility.5` ([§FS-citations.recognized.escapes](FS-citations.md#recognizedescapes-an-id-in-an-escape-position)), or
`§REQ-backwards-compatibility` with section 5 named in prose.

### 4.7 The changelog is the source of release notes

Before it creates or updates a GitHub release, `release.yml` extracts the requested
`vX.Y.Z` section from `docs/changelog.md` at the selected release ref and passes that body to
`gh release create` or `gh release edit`; a missing or empty section fails the release
before any artifact is published to the GitHub release.

### 4.8 One old glibc baseline, one pinned toolchain

Both Linux binaries build inside a `manylinux2014_<arch>` container pinned by digest, not by
tag, so the artifact targets an old glibc baseline rather than whatever glibc
`ubuntu-latest` ships. Every job, on a host runner or in a container, installs the same
pinned Rust toolchain, so the six binaries come from one compiler version. Every candidate
payload is built the same way, for the six rows of [§FS-distribution-candidate.1.1](FS-distribution-candidate.md#11-six-rows-five-of-them-registry-rows) and the
floors and runners each row pins ([§FS-distribution-candidate.1.3](FS-distribution-candidate.md#13-runtime-floors-are-inspected-not-assumed)); six `grund-lsp` archives
join the six CLI archives.

### 4.9 Distributed binaries are profile-guided-optimized

Each distributed `grund` binary is profile-guided-optimized: `scripts/pgo-build.sh` builds an
instrumented binary, records a profile running the benchmarks' self-repo hot commands
against `grund`'s own tree, and rebuilds against it ([§DA-pgo-release](../decisions/architectural/DA-pgo-release.md#da-pgo-release-distributed-binaries-are-pgo-built-trained-on-the-benchmark-workload)). A platform whose PGO
training is broken on its hosted runner, while its ordinary release build self-checks
cleanly, may publish a self-checked LTO-only binary instead; Windows arm64, where rustc
crashes compiling the instrumented build, is the one such platform, and in a candidate only
that identified crash qualifies ([§FS-distribution-candidate.7.4](FS-distribution-candidate.md#74-only-an-identified-windows-arm64-compiler-crash-falls-back)). Each candidate payload
trains on its own product's workload ([§FS-distribution-candidate.7.1](FS-distribution-candidate.md#71-each-product-trains-on-its-own-workload),
[§FS-distribution-candidate.7.3](FS-distribution-candidate.md#73-the-packaged-payload-is-the-profile-use-build)). Development builds and push and pull-request CI are never
PGO-built. `cargo install grund` is built with LTO but no PGO, since `cargo install` runs no
custom build step; it behaves byte for byte as the distributed binary does, and only its
speed differs.

### 4.10 crates.io publishes in dependency order, and artifacts upload last

After every platform binary passes its self-check, the workflow publishes `grund-core`,
`grund` and `grund-lsp` to crates.io when crate publishing is enabled, `grund-core` first; a
run that still has `grund` or `grund-lsp` to publish waits up to 30 minutes for Cargo to
resolve the matching `grund-core` before publishing it. GitHub release artifacts upload only
after the platform PGO builds pass and, when enabled, crates.io publishing succeeds. The
cross-registry publisher uploads nothing until the matching crates resolve
([§FS-distribution-candidate.8.4](FS-distribution-candidate.md#84-order-and-readiness)) and never overwrites on a rerun ([§FS-distribution-candidate.8.5](FS-distribution-candidate.md#85-reruns-resume-and-never-overwrite)).

### 4.11 The full-ecosystem release

The full-ecosystem release publishes one rehearsed candidate ([§FS-distribution.4.13](FS-distribution.md#413-a-candidate-is-rehearsed-before-anything-is-published)): the
crates through `release.yml`, then the npm and PyPI packages of
[§FS-distribution-candidate.2.1](FS-distribution-candidate.md#21-thirty-nine-artifacts-from-one-version) through the publisher, from the same commit at the same
version. It is complete only when every artifact the candidate's plan names is published;
until then it is partial, and the publisher says so, listing what is published and what is
still missing, and resumes the same candidate on a rerun ([§FS-distribution-candidate.8.5](FS-distribution-candidate.md#85-reruns-resume-and-never-overwrite)).
The CLI and the LSP move together within a release: each `grund-lsp` package pins or bundles
the `grund-core` version the matching CLI release ships.

### 4.13 A candidate is rehearsed before anything is published

Every npm and PyPI artifact comes from one candidate: assembled from one commit, recorded in
an immutable `manifest.json`, installed fresh and proved on every registry row by a
rehearsal that holds no credential, and receipted per row. Only those bytes may ever be
uploaded, by a publisher that is disabled and reachable only by a gated manual dispatch
naming the candidate's SHA and manifest digest ([§FS-distribution-candidate](FS-distribution-candidate.md#fs-distribution-candidate-one-candidate-is-assembled-rehearsed-and-verified-before-any-registry-sees-it)).

## 5. What a release promises

From `1.0.0` the pre-release licence of [§REQ-backwards-compatibility.4](../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise) is spent: user-visible
surfaces move only as [§REQ-backwards-compatibility.1](../requirements/REQ-backwards-compatibility.md#1-what-is-covered) says, and the `grund-core` names only as
[§FS-distribution.3.1.2](FS-distribution.md#312-what-10-freezes) says. Not promised:

- Identical APIs across languages. Each binding is idiomatic to its host (camelCase for
  Node, snake_case for Python, `Result<T,E>` for Rust): the behavior is identical, and the
  surface fits each ecosystem.
- A stable ABI for a C-level FFI. Bindings link against the Rust core at compile time, and
  no separate C library ships.
