# FS-distribution: grund distribution targets

`grund` is written in Rust; the target distribution is **all three** major language ecosystems — cargo, npm, and PyPI — with idiomatic API bindings on each. The check engine stays a single shared library; only the surfaces differ. Today the implemented release path publishes the Rust crates: `grund-core`, the Cargo CLI package `grund`, and the optional Cargo LSP package `grund-lsp`. The npm and PyPI packages, `grund-lsp` among them, are assembled and rehearsed as a local candidate and never published by any path that exists today ([§FS-distribution.4.13](FS-distribution.md#413-a-candidate-is-rehearsed-before-anything-is-published)); their publication is tracked in [§RM-distribution](../roadmap.md#rm-distribution-cargo--npm--pypi-from-one-engine). Serves [§GOAL-multi-language](../goals.md#goal-multi-language-same-engine-three-platforms) and [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, body, section, lead), [§FS-terms.terms.2](FS-terms.md#terms2-citations)
(shorthand), [§FS-terms.terms.3](FS-terms.md#terms3-source-forms) (stub), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, scope, config root, workspace,
alias), and [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding, severity, suggestion, caution, verdict).

- **compatibility notice** — The one line a release publishes about a change that a pull
  request's title cannot carry: a verdict the release moves, in the words of the decision
  record that moves it, written as that record's `release-note` section.

## requirements: What every distribution of grund holds to

Five requirements on the distributions as a whole, carrying the marks of [§FS-config.requirements](FS-config.md#requirements-what-the-config-contract-holds-to). None restates that every install, operating system and binding gives the same bytes and the same answer, which [§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes) and [§REQ-one-answer](../requirements/REQ-one-answer.md#req-one-answer-every-command-gives-the-same-answer-about-one-tree) already require, or that an API name is removed only along [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path). Nor does [§FS-distribution.4](FS-distribution.md#4-release-process) earn a point of its own: how this repository cuts, bumps and lists a release, the release notes of [§FS-distribution.4.6](FS-distribution.md#46-the-release-lists-the-pull-requests-merged-since-the-previous-tag) among them, is its process rather than anything `grund` does, and the one effect of it a user can observe is [§FS-distribution.requirements.versions](FS-distribution.md#requirementsversions-what-a-distributed-binary-says-about-versions-is-true-of-it--realized).

### requirements.ecosystems: Every ecosystem the targets name installs `grund` — realized on cargo, deferred on npm and PyPI

The three crates publish today ([§FS-distribution.1](FS-distribution.md#1-targets), [§FS-distribution.4.10](FS-distribution.md#410-cratesio-publishes-in-dependency-order-and-artifacts-upload-last)). The npm and PyPI packages are assembled and rehearsed as a candidate ([§FS-distribution.4.13](FS-distribution.md#413-a-candidate-is-rehearsed-before-anything-is-published)), and none is published: the publisher is disabled ([§FS-distribution-candidate.8.1](FS-distribution-candidate.md#81-publication-is-disabled-and-gated)) and the authority to publish is not yet held ([§FS-distribution-candidate.8.2](FS-distribution-candidate.md#82-identity-is-not-authority)). Their publication is **deferred** to [§RM-distribution](../roadmap.md#rm-distribution-cargo--npm--pypi-from-one-engine). Serves [§GOAL-multi-language](../goals.md#goal-multi-language-same-engine-three-platforms).

### requirements.library: A binding answers as a library, never as a process — realized

Every binding returns data and leaves its host as it found it: it prints nothing, reads no argv, never exits and never changes the working directory ([§FS-distribution.3.1](FS-distribution.md#31-rust-grund-core-crate), [§FS-distribution.3.1.1](FS-distribution.md#311-main_entry-is-absent-from-the-embedding-api), [§FS-distribution.3.2.3.1](FS-distribution.md#3231-execution), [§FS-distribution.3.3.4](FS-distribution.md#334-silent-synchronous-calls)). A failure is data too, carrying the class and location the engine gave it rather than ones parsed from rendered text ([§FS-distribution.3.2.2](FS-distribution.md#322-typed-readonly-host-records), [§FS-distribution.3.3.2](FS-distribution.md#332-operational-failures-and-invalid-arguments)), and the CLI stays the only renderer ([§FS-distribution.3.0.3.1](FS-distribution.md#3031-node-adapter-mapping-and-frozen-cli-wire-projection)). Serves [§GOAL-multi-language.2](../goals.md#2-idiomatic-surfaces).

### requirements.own-payload: An install runs the payload built for its host at its own version, or refuses saying how to get one — realized in the candidate

npm installs the one platform package matching the host's OS, CPU and libc at exactly the umbrella's version, and the command and the addon share it ([§FS-distribution-candidate.2.2](FS-distribution-candidate.md#22-npm-selects-one-platform-package-by-os-cpu-and-libc)); a wheel carries the executable and the extension together ([§FS-distribution-candidate.3.1](FS-distribution-candidate.md#31-payloads-sit-where-each-registry-runs-them)). A platform package that is missing, of another version or for an unlisted row is an error naming the fix, never another row's payload, a download or a compile ([§FS-distribution-candidate.3.3](FS-distribution-candidate.md#33-a-missing-payload-is-an-error-never-a-substitute), [§FS-distribution.3.2.3.2](FS-distribution.md#3232-moduleruntimeload-interface)), and an npm install never compiles ([§FS-distribution-candidate.4.1](FS-distribution-candidate.md#41-npm-builds-from-source-only-when-asked)). A target the matrix defers may build from source and gains no support by succeeding ([§FS-distribution-candidate.1.4](FS-distribution-candidate.md#14-deferred-targets-gain-no-support)). Serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible).

### requirements.versions: What a distributed binary says about versions is true of it — realized

A build from `main` reports the next version with its `-dev` suffix rather than the release it is ahead of, and `grund` names its commit too ([§FS-distribution.4.1](FS-distribution.md#41-between-releases-main-carries-a-dev-version), [§FS-distribution.4.1.1](FS-distribution.md#411-a-development-build-names-its-commit)); a released binary carries its tag's version, which every package of the release shares ([§FS-distribution.4.3](FS-distribution.md#43-releaseyml-publishes-a-commit-that-already-carries-its-version), [§FS-distribution-candidate.6.2](FS-distribution-candidate.md#62-one-version-across-every-package)); and no release is cut at a version that contradicts a release its own messages name ([§FS-distribution.4.2](FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name)). The release named in a warning is what buys a repository its time, and it buys nothing if it is false. Serves [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path).

### requirements.next-operation: The next operation reaches every binding, or is excluded there by name — directional

Each binding offers the CLI's conceptual operations, and one that is absent is never advertised ([§FS-distribution.3](FS-distribution.md#3-api-surfaces)), so Node and Python each list their inventory and their exclusions ([§FS-distribution.3.2.1](FS-distribution.md#321-operations-options-and-refusals), [§FS-distribution.3.2.1.1](FS-distribution.md#3211-exclusions-and-later-adaptation), [§FS-distribution.3.3.5](FS-distribution.md#335-complete-initial-inventory)). A subcommand added from here on joins each binding's inventory or, with its reason, that binding's exclusions, never neither; a field the engine adds to a result reaches each binding's record in that binding's spelling ([§FS-distribution.3.0.1](FS-distribution.md#301-report-and-finding), [§FS-distribution.3.3.1](FS-distribution.md#331-typed-immutable-results)). This binds what is added next, not the inventories already written. Serves [§GOAL-multi-language](../goals.md#goal-multi-language-same-engine-three-platforms).

## 1. Targets

Local Python API support is required by [§FS-distribution.3.3](FS-distribution.md#33-python-grund-pypi-package) independently of registry
availability. The initial binding supplies a locally installable extension and source
distribution; the PyPI release matrix, CLI payload and publication remain separate work.
An approved contract or a failing acceptance test does not establish implemented support.

Local API readiness means a source-built, packed and freshly installed consumer
passes [§FS-distribution.3.2.4](FS-distribution.md#324-acceptance-evidence). It does not mean npm registry availability,
prebuilt-platform installation evidence or authorization to publish.

Candidate readiness is the next rung and still not publication: one commit assembles
every package below for the support matrix of [§FS-distribution-candidate.1.1](FS-distribution-candidate.md#11-six-rows-five-of-them-registry-rows), in the
inventory of [§FS-distribution-candidate.2.1](FS-distribution-candidate.md#21-thirty-nine-artifacts-from-one-version), and a credential-free rehearsal installs
and proves it on every registry row ([§FS-distribution.4.13](FS-distribution.md#413-a-candidate-is-rehearsed-before-anything-is-published)). A `candidate` status in the
table means exactly that. Only the disabled publisher of
[§FS-distribution-candidate.8.1](FS-distribution-candidate.md#81-publication-is-disabled-and-gated), once separately authorized, can make one `published`.

| Registry | Package name        | Status      | Contents                                                                  |
|----------|---------------------|-------------|---------------------------------------------------------------------------|
| cargo    | `grund-core`          | implemented | Shared engine library used by the CLI, LSP, and future bindings.            |
| cargo    | `grund`               | implemented | CLI crate depending on `grund-core`; installs the `grund` binary.              |
| cargo    | `grund-lsp`           | implemented | Optional LSP server crate ([§FS-lsp](FS-lsp.md#fs-lsp-grund-ships-an-optional-lsp-server)) depending on `grund-core`; installs the `grund-lsp` binary. |
| npm      | `grund-cli`           | candidate   | `grund` launcher + Node API (via `napi-rs`); five `@grund-cli/<suffix>` platform packages carry the CLI and addon. |
| npm      | `grund-lsp`           | candidate   | `grund-lsp` launcher; five `@grund-lsp/<suffix>` platform packages carry the server. |
| PyPI     | `grund`               | candidate   | Five `cp310-abi3` wheels carrying the `grund` executable + Python API (via `PyO3` / `maturin`), and an sdist. |
| PyPI     | `grund-lsp`           | candidate   | Five `py3-none` wheels carrying the `grund-lsp` executable (`pipx install grund-lsp`), and an sdist. |

The planned PyPI wheel installs the `grund` command and the import module `grund`, npm's CLI package is `grund-cli` because the unscoped `grund` is externally occupied, and every name is re-verified against the live registries before publish ([§FS-distribution.1.1](FS-distribution.md#11-package-names)). Support packages publish registry README content that links users to the `grund` CLI ([§FS-distribution.1.2](FS-distribution.md#12-support-packages-point-at-the-cli)), and the CLI install on each registry does not transitively pull in `grund-lsp` ([§FS-distribution.1.3](FS-distribution.md#13-the-cli-does-not-pull-in-the-lsp)).

### 1.1 Package names

On PyPI the planned CLI package is also `grund`, whose wheel installs the `grund` command and the import module `grund` ([§FS-distribution.3.3](FS-distribution.md#33-python-grund-pypi-package)). On npm the planned CLI package is `grund-cli`, because the unscoped `grund` package is externally occupied; it still installs the `grund` command. [§DA-pypi-uses-grund-as-the-package-name](../decisions/architectural/DA-pypi-uses-grund-as-the-package-name.md#da-pypi-uses-grund-as-the-package-name-pypi-uses-grund-as-the-package-name) sets the final PyPI name and records why PyPI uses the bare name while npm keeps `grund-cli`; the tool itself was renamed from its pre-release working title `gnd` ([§DA-rename-to-grund](../decisions/architectural/DA-rename-to-grund.md#da-rename-to-grund-rename-gnd-to-grund-before-first-publish)). The release process re-verifies every one of these names — and the remaining unreserved npm/PyPI LSP slots — against the live registries before publish ([§FS-distribution.4.3](FS-distribution.md#43-releaseyml-publishes-a-commit-that-already-carries-its-version)). A name that no registry answers for is free, and the package's own endpoint is the only thing that can say so: a `404` there passes, while a failure to query that endpoint is a query failure and never an availability. Once it has answered that the package exists, nothing later turns the name back into a free one. What makes an existing package *this project's* is then registry-specific. On crates.io ownership is read from the registry's own owner record for the name ([§FS-distribution.1.1.1](FS-distribution.md#111-cratesio-ownership-is-the-registrys-owner-record)), never from metadata the package declares about itself, and ownership that cannot be established stops the release rather than being worked around ([§FS-distribution.1.1.2](FS-distribution.md#112-unproven-cratesio-ownership-stops-the-release)). On npm and PyPI a claimed name is this project's when the registry's own metadata for it names this repository; because that metadata is written by the last publish, it names the repository the package was published *from*, and a repository move reaches those registries only with the next release — the one this guard stands in front of — so there the former repository is accepted alongside the current one, or the move would lock the project out of its own names.

The npm platform packages are claimed too: the five `@grund-cli/<suffix>` and five `@grund-lsp/<suffix>` names of [§FS-distribution-candidate.2.2](FS-distribution-candidate.md#22-npm-selects-one-platform-package-by-os-cpu-and-libc), one per registry row, re-verified by the same guard under the same npm rule. Their scopes and names are provisional until held. Availability, project identity and publishing authority are three gates, not one: the guard answers the first two, and a name that passes both is still not one this project may publish to until the authority of [§FS-distribution-candidate.8.2](FS-distribution-candidate.md#82-identity-is-not-authority) exists.

#### 1.1.1 crates.io ownership is the registry's owner record

For the three crates.io names — `grund-core`, `grund`, and `grund-lsp` — an existing package is this project's only when the registry's owner endpoint for that name, `/api/v1/crates/<name>/owners`, returns a `users` entry whose `kind` is exactly `user` and whose `login` is exactly `vjovanov`. That login is this project's crates.io identity, and it is carried as a named constant rather than spelled into a pattern, because it is a credential and not a shape. Nothing else in the response establishes ownership: the mutable display `name`, the URL text, the opaque numeric id and the `github_username_matches` flag are publisher-supplied or meaningless out of context; a bare organization string such as `agent-grounds` is neither a user record nor Cargo's `github:org:team` team form; and team records establish nothing here, because no team is part of the identity this project declares — admitting one is an ownership-policy change of its own, not a widening that happens by accident. The package's own `repository` metadata never participates in the decision. It is written by the last publish, so after a repository move it can only be corrected *through* the publish this guard stands in front of; and it is publisher-supplied, so any package at all may copy this repository's URL into it.

#### 1.1.2 Unproven crates.io ownership stops the release

A crates.io name that exists and is not proven this project's fails the release, and the message says which of four things went wrong, because the operator's next move differs in each. An owner record that is well-formed but carries no trusted user — an empty owner set, or only untrusted ones — reports `error: crates.io/<name> is already taken without trusted owner vjovanov` and names the owner endpoint. Owner data that is missing, malformed or the wrong shape reports that the owner evidence could not be read. An owner endpoint that answers `404`, or any other non-`200`, reports that ownership could not be determined and names the status. A request that never completes reports the transport failure rather than falling through as an unexplained non-zero exit. Each of the four exits non-zero, each is distinguishable from the other three, and each names the owner endpoint it asked. None of them falls back to the package's declared metadata: that fallback is what both the move deadlock and the copied-URL acceptance are made of, and ownership decided from unauthoritative data is not decided at all.

### 1.2 Support packages point at the CLI

Support packages that are not the primary user-facing install, such as `grund-core`, publish registry README content that links users to the `grund` CLI and names sibling packages such as `grund-lsp` once they exist. The npm platform packages are support packages too: each README says which row it serves, that its umbrella installs it, and links to the CLI, and every npm and PyPI package carries the licence and says truthfully what it installs ([§FS-distribution-candidate.2.4](FS-distribution-candidate.md#24-every-package-says-what-it-holds)).

### 1.3 The CLI does not pull in the LSP

The CLI install on each registry does **not** transitively pull in `grund-lsp` — they are independent packages, per [§DA-lsp-optional](../decisions/architectural/DA-lsp-optional.md#da-lsp-optional-lsp-server-ships-as-a-separate-optional-binary). A user who only runs `grund check` in CI installs the CLI alone; a user who wants editor integration installs `grund-lsp` separately and configures their editor to launch it ([§FS-lsp.2](FS-lsp.md#2-installation-and-lifecycle)). The same holds on npm and PyPI, and it is proved on the installed packages rather than read off a manifest: every registry row installs the CLI without the LSP and the LSP without the CLI, and each works alone ([§FS-distribution-candidate.5.2](FS-distribution-candidate.md#52-every-registry-row-installs-fresh-and-without-rust)).

## 2. CLI parity

The `grund` binary behaves identically regardless of how it was installed: the same flags, the same exit codes, the same byte-for-byte report format ([§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes)). Users on Linux, macOS, and Windows who run `grund check .` against the same repo get the same answer.

CLI reports use logical paths, relative to the base `--path-base` selects, else `relative_paths` ([§FS-cli.3.4](FS-cli.md#34---path-base--where-report-paths-are-spelled-from), [§FS-config.3.6](FS-config.md#36-output--report-format)), with `/` as the separator, even on Windows. This applies to text reports, JSON fields, `sites`, ID-query e2e fixture lists, stub-link targets, and generated cross-reference URLs; native platform paths may appear only in launch-time errors about paths outside the scanned repo, where there is no repo-relative path to print. The CI build/test matrix is the proof for this contract: every normal e2e case must pass on Linux, macOS, and Windows.

"However it was installed" includes the npm launcher, the wheel's executable and the archive, and the proof for those is the installed artifact rather than a checkout build: the launcher adds nothing between the user and the payload ([§FS-distribution-candidate.3.2](FS-distribution-candidate.md#32-the-npm-launcher-is-invisible)), and the rehearsal compares every installed `grund` byte for byte against the goldens on every registry row ([§FS-distribution-candidate.5.3](FS-distribution-candidate.md#53-every-install-of-the-cli-answers-alike)).

## 3. API surfaces

Each binding exposes the same conceptual operations as the CLI subcommands, plus a programmatic check-and-iterate path so the engine can be embedded inside test runners and editor servers.

The initial Python inventory is normative in [§FS-distribution.3.3.5](FS-distribution.md#335-complete-initial-inventory). Process
transport and editor-only exclusions must be explicit; an absent operation cannot be
advertised as supported. Node's complete disk-backed operation inventory and
process/editor exclusions are normative in [§FS-distribution.3.2.1](FS-distribution.md#321-operations-options-and-refusals) and
[§FS-distribution.3.2.1.1](FS-distribution.md#3211-exclusions-and-later-adaptation).

### 3.0 Language-neutral data shapes

Every binding returns the same data, only spelled idiomatically: a report and its
findings ([§FS-distribution.3.0.1](FS-distribution.md#301-report-and-finding)), and show
options ([§FS-distribution.3.0.2](FS-distribution.md#302-showopts)). These fields are
normative. Complete host data and the frozen CLI JSON projection are compared
separately under [§FS-distribution.3.0.3](FS-distribution.md#303-complete-data-and-canonical-parity), serving
[§GOAL-multi-language](../goals.md#goal-multi-language-same-engine-three-platforms)
and tracked in [AR-goal-measurement.2](../architecture/AR-goal-measurement.md#2-goal-meters).

#### 3.0.1 Report and Finding

Host results retain all engine data, including nullable columns, multi-site locations,
rule authority, scan-error status, output-format metadata and run cautions. Run cautions
remain distinct from report warnings, including when setup fails. Node's records also
carry complete and selected reports and partial outputs, with empty collections as
arrays and optional fields present as null ([§FS-distribution.3.2.2](FS-distribution.md#322-typed-readonly-host-records)). The CLI projection
below stays frozen: it omits columns and projects empty sites/authority to null; it is
not the complete host schema. [§FS-distribution.3.0.3](FS-distribution.md#303-complete-data-and-canonical-parity) compares both forms separately.

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

Python takes required query operands positionally and section/mode/format/root as
keywords ([§FS-distribution.3.3.5](FS-distribution.md#335-complete-initial-inventory)). Roots and query failures follow
[§FS-distribution.3.3.3](FS-distribution.md#333-per-call-scope-and-path-encoding) and [§FS-distribution.3.3.2](FS-distribution.md#332-operational-failures-and-invalid-arguments). Batch setup failure raises
once; individual failed queries remain ordered outcomes. An empty batch succeeds
without loading configuration. Node's exact option defaults and typed query failures
are in [§FS-distribution.3.2.1](FS-distribution.md#321-operations-options-and-refusals) and [§FS-distribution.3.2.2](FS-distribution.md#322-typed-readonly-host-records).

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

The shared `tests/bindings/` corpus compares complete Rust and Python result,
failure and run-caution data, preserving every field and engine array order. The Node
adapter (`tests/bindings/node/`) runs against the same Rust oracle and protocol,
extended by the oracle's `--metadata` reply and no-`root` framing
([§FS-distribution.3.2.4](FS-distribution.md#324-acceptance-evidence)); neither binding's evidence claims the other's coverage.
Adapters use supported data APIs, not CLI subprocesses or message-only comparisons.

The canonical UTF-8 JSON envelope has exactly `failure`, `result`, `run_cautions`:
one of failure/result is null, except that an empty successful result may be null.
Objects sort keys recursively by UTF-8 bytes; arrays keep engine order. Nullable
fields are present, empty collections are arrays, separators are compact, and exactly
one LF ends the record. Unicode and `/` remain literal. Quote, backslash, LF, CR and
tab use JSON escapes; every other U+0000–U+001F character uses lowercase `\u00xx`
(including backspace and form feed). No raw Rust serialization substitutes for this
specified projection. Full structured equality and exact canonical bytes are separate
assertions.

Compare the frozen CLI projection separately to existing stdout/stderr JSON goldens.
It keeps global stable path/line/message ordering (null path first, absent line as
zero, warning/error/suggestion tie order), suggestion channel instead of severity,
null empty sites/authority, authority last, and no column key. Run cautions are
accounted for separately from report warning records and raw stderr cautions.

Corpus acceptance includes clean/error trees, config failures, caution-only runs and
cautions before failure, suggestions enabled/disabled, selector authority/safety,
missing/ambiguous single and batch queries, workspaces, Unicode/control characters
and logical separators. Every operation has success/refusal coverage. Writers run
on isolated copies and compare preview/no-execution behavior and resulting bytes
with core, including isolated integration user homes.

##### 3.0.3.1 Node adapter mapping and frozen CLI wire projection

The Node adapter writes the envelope above itself, with no arbitrary
JSON.stringify guarantee: that uses different escapes for backspace/form-feed
and object order. Node conversion explicitly maps
fixed host camelCase fields to snake_case; authored IDs/config keys are opaque.
Separate top-level runCautions into run_cautions, do not duplicate them; complete
check result keeps report/selected_report/had_scan_errors/output_format. Compare
every native-only field too, including column, sites, authority and query details.
Require same source SHA/engine version/config/tree/invocation and ignore state;
wrong version fails comparison, never normalization of findings or messages.

Second channel projects those same host records into the EXISTING CLI wire form.
The comparator produces byte-identical stdout AND stderr, plus CLI status
expectations where meaningful. Frozen wire projection is explicitly distinct
from the lossless host envelope; these are the only permitted differences:

- Check uses selectedReport, not the entire object. Stable sort by bytewise
  `(path,line-or-0,message)`, null path before paths; ties retain initial
  warning/error/suggestion chain order. Keys: severity OR suggestion channel,
  path,line,code,message,sites,authority. Native-only column and envelope metadata
  do not add CLI keys; sites/authority empty arrays become null only in this
  frozen wire projection. Actual values/sites/order/authority do not disappear.
- Line-present findings go to stdout; line-less findings go to stderr. Separate
  run_cautions render as existing `warning: <message>\n` text on stderr even in
  JSON mode, before any later refusal/report. Config diagnostic report warnings
  remain JSON findings where existing CLI requires them. No generic warning
  flattening between these channels. Clean nonempty JSON check has zero bytes;
  empty scan has its existing stderr finding.
- Single query failure uses the frozen diagnostic object on stderr; a batch
  per-query failure uses the exact query/result/error envelope on stdout. Launch
  failures stay CLI `error:` text, including existing hints/known lists. No
  frontend message parsing to infer classes or locations.
- Show uses existing engine query JSON when available and typed metadata to pin
  it; optional kind_title placement and E2E/value shapes are preserved. Batch
  preserves input/exhaustive order and the CLI's JSON batch bytes. Refs/list/cover
  and size rows use their fixed command fields/key orders; host envelope fields
  and null optional metadata do not invent extra CLI keys. Size measurements
  lower to lead_<unit>/full_<unit> pairs in caller unit order. Test total/summary
  projections separately.
- CLI escaping is its actual json_escape contract: quote/backslash/LF/CR/tab
  escapes plus ALL Rust char::is_control characters as lowercase `\\uNNNN`,
  including DEL/C1. This differs deliberately from the lossless corpus C0 rule;
  cover control-character fixtures so JSON.stringify cannot pass by accident.
- Paths keep core '/' rendering and invocation/config base; no absolute-path
  replacement, regex message repair, platform basename stripping or arbitrary
  sort is allowed. Mutation/config/TOML/text surfaces use exact existing command
  goldens where no JSON command exists; complete-data comparison is still required.

Comparator/serializers are test infrastructure. Production bindings return
records and never print; the CLI remains the renderer. Reuse tests/e2e cases,
particularly json-report, show/refs ambiguities, rule authority, config warnings,
workspace root/member modes, Unicode and formatter/init refusal fixtures. Fixture names in tests refer to concrete test inputs; future measured coverage is recorded only after the tests execute.

### 3.1 Rust (`grund-core` crate)

```rust
let report = grund_core::check(&path)?;
let body = grund_core::show("FS-check", ShowOpts::default())?;
```

`Report` and the underlying `Catalog` are exposed as plain data structures so callers can iterate, filter, or render their own output. The effective configuration is reachable through `effective_project`, as a `Project` whose `schema`, `rules` and `presentation` hold one concern each ([§FS-config.concerns](FS-config.md#concerns-every-key-belongs-to-exactly-one-concern)), but `Project` is `#[doc(hidden)]` and outside the promise of [§FS-distribution.3.1.2](FS-distribution.md#312-what-10-freezes), because its records still move ([§FS-schema.13](FS-schema.md#13-what-is-absent)). The former root names `Findings`, `Config`, `KindConfig` and `CitationRules` were deprecated aliases from `0.17.0` and were removed in `0.19.0` ([§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)): `Catalog` replaces `Findings`, and the configuration aliases have no promised replacement. Every function on this surface returns data and writes to no stream, which is what lets one answer be rendered by a terminal, an editor and an embedder alike — the report's warning channel included, so a caution settled before any report exists still reaches a caller as a warning rather than as a line on its stderr ([§FS-check.4.10](FS-check.md#410-include_root--false-leaves-the-blocks-own-files-unread), [§FS-workspace.6.1.7.6](FS-workspace.md#6176-how-the-undecidable-claim-warning-travels)). Process callers use the `grund` CLI package; embedders call the data-returning entry points [§FS-distribution.3.1.2](FS-distribution.md#312-what-10-freezes) names ([§FS-distribution.3.1.1](FS-distribution.md#311-main_entry-is-absent-from-the-embedding-api)).

#### 3.1.1 `main_entry()` is absent from the embedding API

`grund_core::main_entry()` is not exported in 0.15.0 or later. It was a process entry point — it parsed argv, printed, and returned an exit code — kept for `grund-core = "0.4"` consumers until 0.14.0 shipped a deprecation note naming its removal in 0.15.0, the sequence [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) requires. A former caller uses `check`, `show`, `scan`, or another data-returning engine API when embedding grund, and invokes the `grund` CLI package when it needs a process entry point. Removing the engine adapter changes none of the shipped CLI's commands, flags, rendered bytes, or exit decisions, and changes no LSP behavior.

#### 3.1.2 What 1.0 freezes

`grund-core` 1.0 freezes exactly the root names this point reaches: the entry points `check`, `check_with_opts`, `scan`, `show`, `list`, `list_sizes`, `refs`, `refs_outcome`, `cover`, `cover_text`, `cover_lines`, `format_references`, `propose_id`, `complete_ids`, `init`, `fetch_snapshot` and `integrations` ([§FS-distribution.3.1.3](FS-distribution.md#313-integrations-is-the-one-integrations-entry-point)), the typed refusals `ShowQueryError`, `FmtScanAbort` and `FmtConflictAbort` ([§FS-fmt.3.2](FS-fmt.md#32-a-run-that-needs-the-whole-declaration-set-rewrites-nothing)), and every root type their signatures reach, transitively. That is 118 names, `InitAllocation` and `InitAllocationHome` among them, which `InitNext.allocation` reaches ([§FS-init.2.2.2.3](FS-init.md#2223-the-kind-the-guidance-names)), and the rows whose disposition is `frozen at 1.0` in the [public-surface inventory](../discussions/proposals/2026-09-22-grund-core-public-surface-inventory.md) are the full list. An entry point with no typed refusal returns `anyhow::Result`.

Every other root name is `#[doc(hidden)]` and outside the promise, and so are the `test-*` Cargo features and what they export. Hiding retires nothing and changes no type: a hidden name still links and still works. Frozen means what Cargo reads into a `1.0.0`: a `1.x` release may add a name, a field or a variant and may deprecate a name, and only `2.0` removes or retypes one ([§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)).

An option record is built from `Default` with its fields assigned; an output record or a refusal is read, never built. That is what lets `1.x` add a field or a variant, so since `0.21.0` every promised struct and enum is `#[non_exhaustive]`, and no other type and no variant is: outside the crate, building a promised record with a struct literal, or matching a promised enum without a wildcard arm, does not compile. The crate documentation gave that notice from `0.19.0`.

#### 3.1.3 `integrations` is the one integrations entry point

`integrations(opts: IntegrationsOpts) -> Result<IntegrationsOutput, IntegrationsError>` answers what `grund integrations` answers, as data; its options, output and refusal are [§FS-integrations.1.3.1](FS-integrations.md#131-the-engines-answer)'s. The closed vocabularies it is spelled in keep exactly these methods: `IntegrationClient` keeps `ALL`, `name` and `from_name`; `InstallKind` keeps `name`; `ConversationRendering` keeps `name` and `from_name`; `ConversationTarget` keeps `name` and `from_name`, and its `ALL` became public in `0.19.0`, an addition; and `IntegrationAgent`, the six agents of [§FS-integrations.4.3.8](FS-integrations.md#438-the-global-instruction-files), has `ALL`, `name` and `from_name`. The 30 root names the CLI assembled `--write` from before `0.19.0`, and the 9 other methods of those vocabularies, were deprecated in `0.19.0` and removed in `0.21.0` ([§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)): the inventory lists them under its removed names, and `crates/grund-core/src/lib.rs` keeps one line for each that names what answers it now.

### 3.2 Node (`grund-cli` npm package)

```js
import { check, show } from 'grund-cli';

const result = await check('./repo'); // findings are result.report.errors, warnings, suggestions
const body = await show('FS-check', { mode: 'brief' });
```

The native binding uses napi-rs over the shared core. This contract authorizes
local buildable/installable API artifacts; registry publication remains pending.
Supported prebuilt targets are packaging's GNU Linux x64/arm64, macOS x64/arm64
and Windows x64 matrix. Local evidence does not certify that whole matrix.

Packaging consumes that contract unchanged. Node 22.x and 24.x with N-API 8
([§FS-distribution.3.2.3.2](FS-distribution.md#3232-moduleruntimeload-interface)) are what every registry row proves
([§FS-distribution-candidate.1.2](FS-distribution-candidate.md#12-every-registry-row-proves-every-runtime-it-promises)); one exact-version platform package per row is
selected by OS, CPU and libc ([§FS-distribution-candidate.2.2](FS-distribution-candidate.md#22-npm-selects-one-platform-package-by-os-cpu-and-libc)); and a host without one
gets an actionable error and an explicit `npm run build:source`, never a download or an
automatic compile ([§FS-distribution-candidate.3.3](FS-distribution-candidate.md#33-a-missing-payload-is-an-error-never-a-substitute), [§FS-distribution-candidate.4.1](FS-distribution-candidate.md#41-npm-builds-from-source-only-when-asked)).

#### 3.2.1 Operations, options and refusals

All functions below return Promises, including argument-validation and loader
failures. Required strings are positional; options are an optional plain object.
Unknown keys, wrong types, NUL, unpaired UTF-16 surrogates, non-finite numbers,
unsafe integers, invalid enum values and conflicting options reject `input`.
No URL, Buffer, config injection or argv API. Omitted optional values mean default;
explicit null is allowed only for `section`, batch queries and nullable options
whose declarations explicitly permit it. Arrays are copied at entry.

`RootOptions = {root?: string}`. Every tree operation snapshots cwd at entry,
resolves a relative operand against that snapshot, and passes an absolute path
plus the original omitted/explicit bit to core. It never calls chdir. `check`
takes root positionally; init takes target positionally; other calls take root
in options. `scan` requires an explicit root. Output bases remain the core's:
config-root or invocation operand per existing relative_paths semantics. Any
needed path-base carrier is additive core data, not a JS path resolver.

Results are typed records with `runCautions: Finding[]` at their top level;
these are separate from report warnings. A refusal carries the same array on
`GrundError.failure`. Common config/I/O/operation failure classes apply to all
calls that load a tree; the table names additional operation-specific cases.

| Public Promise signature | Options/defaults beyond root | Reused core; minimum additive seam | Result and refusal behavior |
| --- | --- | --- | --- |
| `check(root?: string, options?: CheckOptions): Promise<CheckResult>` | requireGrounding=false, suggestions=false, full=false, rule=null, only=[], ignore=[], onlyRule=false | `check_with_run_warnings`, `CheckFindingSelection`; typed `check_outcome` | complete report, selectedReport, outputFormat, hadScanErrors, runCautions; findings and partial file I/O resolve; failed setup rejects. Selection validated before discovery; ignore wins, rule-produced codes carry invalid-rule and every empty-citation-obligation row, safety io always survives, onlyRule requires rule. selectedReport also holds one selected-check-off warning per `only` code the run left off, which the complete report does not ([§FS-check.1.4.1](FS-check.md#141-a-selected-check-the-run-leaves-off-says-so)). |
| `scan(root: string): Promise<ScanResult>` | Explicit path; no checker flags | `scan`/strict scan, additive `scan_outcome` over the same pipeline with warning and normalized snapshot carriers | declarations/citations/scan facts available as arrays, not a live iterator; strict scan failures reject with available partial facts and cautions. No parallel JS scan. |
| `show(id: string, options?: ShowOptions): Promise<ShowResult>` | section=null, mode=lead, format=text; modes lead/brief/toc/full, formats text/md/json | `show_with_scope`, typed query errors, additive `show_outcome` | body/path/line/sections/json plus cautions. Query refusal rejects query with sites or candidate IDs; setup rejects config/io. Inline section plus section option is a query failure, not a JS parser reinterpretation. |
| `showBatch(queries: readonly (string \| ShowQuery)[] \| null, options?: BatchOptions): Promise<BatchResult>` | null means all coordinates; [] is empty without config loading; mode=lead, globs=[] (null with globs generates and filters; explicit queries with globs reject `input`) | `show_batch_with_scope`'s single-context machinery; additive `show_batch_data_outcome` before the existing JSON rendering | ordered `{query,ok:true,result}` or `{query,ok:false,failure}` records, plus note and cautions. Validate entire input before discovery. Per-query failures continue; setup failure rejects once. No stdin/NDJSON input API. |
| `refs(id: string \| null, options?: RefsOptions): Promise<RefsResult>` | section=null, descendants=false, globs=[], from=null; id, globs and from select one mode, and mixing them rejects `input` ([§FS-coordinate-globs.bindings](FS-coordinate-globs.md#bindings-node-and-python)) | `refs_with_metadata`/`refs_outcome`, typed `refs_metadata_outcome` | hits, workspace, outputFormat, note, scanErrors, kindTitle, derived summaries/totals, cautions. Valid undeclared target retains hits/note; invalid ID/ambiguity, refused glob or refused `from` source rejects query with available context. No imposed show-not-found rule for an ID. |
| `list(options?: ListOptions): Promise<ListResult>` | kinds=[], projects=[], unused=false, selector=null, globs=[] | `list_with_run_warnings`, typed `list_outcome` | entries/summaries/workspace/outputFormat/scanErrors/note/cautions. Unknown/non-citable kinds, unknown project, project filter without workspace, invalid/unresolved selector, refused glob ([§FS-coordinate-globs.bindings](FS-coordinate-globs.md#bindings-node-and-python)) reject; not empty success. |
| `listSizes(options?: SizeOptions): Promise<SizeResult>` | list options (globs included) plus units=[lines,words,bytes], top=null; units nonempty, unique, caller ordered; top positive | `list_sizes`, typed warning-preserving `list_sizes_outcome` | entries with ordered unit/nullable lead/full measurements, workspace/outputFormat/scanErrors/note/cautions. Filters precede top; first unit ranks; existing stable tie-break; broken stub measures remain null. |
| `cover(options?: CoverOptions): Promise<CoverResult>` | globs=[] ([§FS-coordinate-globs.bindings](FS-coordinate-globs.md#bindings-node-and-python)) | `cover`, typed `cover_outcome` | entries including empty files, all citation/site/project/enclosing coordinates or those globs select, outputFormat, note, scanErrors/cautions. `cover_text` is a rendering-oriented lower-detail projection, not an additional public operation; complete cover data can supply it. |
| `fmt(options?: FmtOptions): Promise<FmtResult>` | write=false, marker=false, crossRefs=false | `format_references`, `FmtScanAbort`, `FmtConflictAbort`, typed `format_outcome` | changes/path/line/label, scanErrors, refusedWrites, cautions. Core completed partial runs resolve with diagnostics; fatal preflight or later write errors reject with partial output. A conflict refusal rejects with kind operation and code conflict-marker, carrying path, line, sites, empty partial changes and details.conflicts as Python does ([§FS-distribution.3.3.2](FS-distribution.md#332-operational-failures-and-invalid-arguments)). Config-enabled cross refs still runs when crossRefs=false. Preview is default; no added check flag/exit decision. |
| `proposeId(kind: string, title: string, options?: IdOptions): Promise<IdResult>` | width=3, nonnegative safe integer | `propose_id_with_run_warnings`, typed `propose_id_outcome` | id/kind/number/slug/folder/file/e2eCaseDir/fileHoldsSingleDeclaration/cautions. Unknown/non-citable kind rejects operation with known kinds; empty slug or collision rejects query. No writes or implicit declaration creation. |
| `init(target?: string, options?: InitOptions): Promise<InitResult>` | name=null, description=null, docs=false, force=false, dryRun=false, check=false, noVcs=false, agents=null (automatic) | `init`, `InitError`, additive typed `init_outcome` | events/errors/notes/next/pendingChanges/cautions. Explicit call writes by default, unlike fmt. check implies dryRun. Core preflight refusals reject; completed validation findings resolve; partial write failure rejects with events. Home/global/VCS guards, protected config and companion bytes preserved. |
| `completeIds(options?: CompleteOptions): Promise<CompleteResult>` | prefix="", sections=false | `complete_ids_with_run_warnings`, typed `complete_ids_outcome` | ids/cautions; deterministic sorted deduplicated candidates, aliases and separators from core. Core errors reject here; only shell keystroke frontend has silent-error policy. |
| `effectiveConfig(options?: RootOptions): Promise<ConfigResult>` | none | `effective_config`, config warning helpers, additive `effective_config_outcome` plus schema-keyed values projection | root/configFile/values/cautions; discovered config with effective defaults; no declaration scan, no broad Rust Config object. |
| `validateConfig(options?: RootOptions): Promise<ConfigResult>` | none | `validate_config`, additive `validate_config_outcome` | same record; expands member configs exactly as core, scans no declaration bodies; invalid config rejects config with location and earlier cautions. |
| `fetch(id: string, options?: RootOptions): Promise<FetchResult>` | Explicit fetch is the write authorization; no invented preview flag | `fetch_snapshot_with_run_warnings`, additive typed `fetch_outcome` before string flattening | id/cautions; typed query/config/io/operation failures; captures integration outputs as core does. Only this operation executes configured integration; no implicit fetching. |
| `integrations(options?: IntegrationOptions): Promise<IntegrationResult>` | client=null, write=false, conversation=null, conversationTarget=null, agent=null; client set codium/iterm2/kitty/tmux/vscode/wezterm; conversation plain/link; target file/path/web/vscode/vscodium/cursor | Existing IntegrationClient/detection/artifact/managed writers; additive shared `integrations_outcome` orchestration | detected clients/descriptors/artifact data or events/notes/preferences/manualSteps, cautions; user-global operation, no root. Preserve validation, user-config-before-write, active-agent gates and owned paths. Only explicit write mutates. No fabricated CLI JSON-install mode. |
| `referenceStyle(path: string): Promise<StyleResult>` | explicit document path | `reference_style`, typed `reference_style_outcome` | marker/trigger/cautions; member configuration wins exactly as core. |
| `agentSetupInstructions(): Promise<SetupResult>` | none | the whole setup guide through `canonical_template_text`: the text `grund guide --full` prints ([§FS-init.5.5](FS-init.md#55-the-core-and-its-sections)) | instructions, empty cautions; existing packaged workflow returned as data, never printed. No section argument: the caller chooses what of the guide reaches a model. |

`summaries` and `totals` in refs are transport projections over existing hits,
with byte-sorted paths and site counts; no new graph query or resolution rule.
All flags affecting engine behavior map to current core mechanisms. Output-only
CLI switches (refs total/summary, id explain, format exit code) become returned
records/derived fields, not fake process options. The omitted showBatch mode
format is deliberate: typed results plus canonical json are always returned;
the bindings project the CLI's JSON batch bytes, the one batch
format that is data; its `text` and `md` framing is for reading ([§FS-show.3.3](FS-show.md#33-batch-query)).

##### 3.2.1.1 Exclusions and later adaptation

LSP snapshots/overlays/hover/on-type edits and protocol lifecycle remain the LSP
frontend's: only disk-backed batch operations, completion and referenceStyle are
public here. No generated first-party editor plugin or Node watch/streaming API.
Help/version dispatch, argv, exit codes, SIGPIPE, static shell completion-script
printing and CLI/LSP launchers remain process surfaces. Build/engine version
metadata is available privately for package matching, not another engine API.
`cover` line ownership ([§FS-cover.6](FS-cover.md#6-line-ownership)) is excluded from the initial inventory,
pending later adaptation: `cover` takes no `lines` option.
#459 format/path-base/v2 settings and #463 schema-view wait for their actual core
implementation, then require a separate contract extension. No placeholder API
or JS interpretation of those proposals. #453/#454/#466 are later internal
adaptation, not landing blockers. #470 shared corpus and #471 assembly are
coordination interfaces; no separate GitHub issue must close first.

#### 3.2.2 Typed readonly host records

Records
are ordinary JS objects, arrays iterate synchronously after the Promise resolves,
and TS properties are readonly. No persistent native handles, session objects,
live iterators or permanent promise to mirror Rust `Config`/`Findings` layout.
Optional result fields are present as null, never silently omitted/undefined;
collections are [] when empty. Schema-controlled input defaults are in [§FS-distribution.3.2.1](FS-distribution.md#321-operations-options-and-refusals).
Integers crossing as number must be within Number.MAX_SAFE_INTEGER; unexpected
overflow rejects operation with code `numeric-overflow`, never truncates.
Coordinates keep core one-based byte-column units; do not convert to UTF-16.

##### 3.2.2.1 Common records

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

`kind` is the stable discriminant; `code` is a documented machine code, not a
localized sentence. Expected query codes preserve not-found, missing-section,
broken-stub, ambiguous, ambiguous-section, invalid-id and query-failed. Other
codes identify invalid-argument/unknown-option/conflicting-options/path-encoding,
invalid-config, filesystem, operation-failed, native-load, worker-failed,
native-panic, writer-busy and numeric-overflow. Specific engine codes/details
can extend this string catalog compatibly; callers branch on kind and known codes.
Query details include candidates/formatHint where core supplies them; I/O includes
OS code when available; load includes expected target/addon/versions, attempted
locations and cause chain. No unsupported source location is invented.

Input validation and synchronous napi conversion errors are caught inside async
public wrappers; therefore callers always receive rejected Promises. Lazy load
lets import succeed even without a compatible addon; the first call rejects
load. A corrupt JS package that cannot be parsed/imported is a module-system
error outside an operation Promise. Errors do not print. Same class identity
is shared by ESM and CJS. Batch query failures use the same Failure data without
throwing; batch setup failures reject GrundError once.

##### 3.2.2.2 Result fields

Every following record adds Cautions; warning vectors present on native outputs
are moved there once, not repeated in data. Other fields use fixed camelCase
host names, listed here. The shared canonical corpus converts them to snake_case
and preserves authored arbitrary keys; it never mechanically renames IDs or
configuration keys. The existing core output_format is preserved as metadata,
not permission to return a string in place of typed data.

- CheckResult: `report`, `selectedReport`, `hadScanErrors`, `outputFormat`.
  `report` is complete under the requested scope/options, selectedReport is
  computed with core CheckFindingSelection. With no filters they are equal.
  hadScanErrors=true means incomplete, even if selectedReport has no errors.
- ShowResult: `id`, `section:string|null`, `kindTitle:string|null`, `body`,
  `path`, `line`, `sections: {path,title,depth}[]`,
  `json: string|null` (core's exact existing JSON when requested),
  `manifest:{kind:'E2E',args:string[],expectedExit:number,fixtures:string[]}|null`.
  The additive outcome supplies query metadata from the same loaded context,
  before rendering; no second scan or parsing rendered prose.
  JSON source/E2E/value bodies preserve core semantics rather than treating all
  output as prose. All four slice modes and all three single-show formats exist.
- BatchResult: `records: {query:{id,section:string|null},ok:true,result:ShowData}
  | {query,ok:false,failure:Failure}[]`, `note:string|null`. ShowData has ShowResult's fields except
  cautions, mode is invocation-wide, canonical per-query JSON is preserved.
- RefsData: `outputFormat`, `workspace`, `kindTitle:string|null`, `note:string|null`,
  `hits`, `scanErrors`, `summaries:{project:string|null,path,count,lines:number[]}[]`,
  `totals:{sites,files}`. Every RefHit has project/path/line/column/id/
  section/marker/text/enclosingDeclaration/enclosingSection; nullable coordinates
  stay null. Summary lines retain the CLI's duplicate-line policy and ordering.
- ListResult: `outputFormat`, `workspace`, `entries`, `summaries`, `scanErrors`, `note:string|null`.
  Entry: project/id/section/sectionSeparator/kind/path/line/title/stub/defines/
  refs/duplicate/valueRoots:{id,valid}[]. Summary: project/kind/title/home/count.
  project/section/title/defines nullable; output wire omits keys only where the
  existing CLI shape does. Keep kind titles and authored project/kind order.
- SizeResult: outputFormat/workspace/scanErrors/note:string|null/entries. Size entry:
  project/id/section/sectionSeparator/kind/path/line/stub/defines/duplicate/
  measurements:{unit:'lines'|'words'|'bytes',lead:number|null,full:number|null}[].
- CoverResult: outputFormat/scanErrors/note:string|null/entries:{project,path,citations:RefHit[]}[];
  no loss of empty files or citing-side membership.
- FmtData: changes:{path,line,label,counts,lockFile}[]/scanErrors/refusedWrites:string[].
  Completed partial run carries changes and scan errors; refusal partial may
  contain no changes. No made-up rollback or atomic-all-files guarantee.
  `label` is the CLI row after `path:line: `, with the same kinds appended
  ([§FS-fmt.3.5.1](FS-fmt.md#351-a-line-with-several-rewrites-carries-the-first-label)), the same text an expansion or a link writes
  ([§FS-fmt.3.6](FS-fmt.md#36-an-expansion-or-a-link-names-the-text-it-writes)), and the same `; creates the file` or `; deletes the file`
  ([§FS-section-lock.write.report](FS-section-lock.md#writereport-what-fmt-reports)), so a caller comparing `label` exactly breaks
  as an exact-line script does; it should read `counts` and `lockFile` instead.
  `counts:{triggersMarked,citationsMarked,localSectionsExpanded,shorthandsExpanded,
  linksWritten,sectionLockEntries}` are the change's occurrences of each kind, the
  numbers the `--write` totals sum ([§FS-fmt.3.7](FS-fmt.md#37-the---write-report)); `lockFile:'created'|'deleted'|null`
  is set on a lock row whose write creates or deletes its file, and null elsewhere.
- IdResult: id/kind/number:number|null/slug/folder:string|null/file:string|null/
  e2eCaseDir:string|null/fileHoldsSingleDeclaration:boolean.
- InitData: events:{verb,path}[]/errors:Finding[]/notes:string[]/
  next:{docs:boolean,declaresKinds:boolean,entrypoint:string,scanReadsFile:boolean,fsHome:Home,
  allocation:{kind:string,home:Home|null,homeExists:boolean}|null}|null/pendingChanges:boolean, where Home is
  {kind:'file',path:string,headingName:string,headingMarker:string} | {kind:'folder',path:string}.
- CompleteResult: ids:string[]. FetchResult: id:string (the requested operand).
- ConfigResult: root:string/configFile:string|null/values:ConfigValues.
  ConfigValues is a readonly, typed tree of current public configuration keys:
  version/project_name/project_description/reference/id/kinds/scan/fmt/output/
  workspace. Key spelling is the TOML schema, intentionally not renamed; this
  is inspectable configuration, not a Rust memory layout. Only effective keys
  known to the schema are included; optional entries remain null where absent;
  rules, kind order, resolution/fetch and inheritance preserve current meaning.
  Root/configFile give discovery provenance; no invented per-key provenance.
- StyleResult: marker/trigger. SetupResult: instructions.
- IntegrationResult is discriminated by `mode:'detection'|'artifact'|'install'`.
  Detection has detected:string[] and clients:ClientDescriptor[]; artifact
  has client:ClientDescriptor/artifacts:{path:string|null,content:string}[]/
  manualSteps:string[]; install has events:{verb,path}[]/notes:string[]/
  preferences:{conversation,conversationTarget,agentOverrides}/manualSteps.
  Descriptor: client/kind/detected/installed:boolean|null/installKind/
  configTarget/resolverTarget:string|null; manual clients keep installed=null.
  These are transport facts from existing helpers; orchestration policy stays
  in core. User-global paths may be absolute, as integrations already documents.

##### 3.2.2.3 Scan snapshot

ScanResult exposes `snapshot:ScanSnapshot`, not arbitrary object-valued native
Findings. Snapshot includes declarations, citations, escapedCitations,
scannedFiles, walkedDirs, fileStructures, valueBindings, invalidValueDeclarations,
invalidValueBindings, sectionHeadingsOutsideDeclarations, unmarkedHeadings and
nearMissHeadings. Arrays preserve deterministic core order. Declaration groups
are flattened by rendered ID/path/line, retaining every duplicate site; sections
are ordered entries, retaining duplicate section claims separately. Private
legacy/local candidate lists remain private; promoted citations already expose
the semantic edges. Strict scan errors reject, with available snapshot partial, and the first failure's message is its reason and *Fix*, as `check` prints it ([§FS-check.2.4.3](FS-check.md#243-what-failed-and-where-decide-the-code-and-the-fix)).

Declaration records: id/kind/number/slug/path/line/headingLevel/title/sections/
duplicateSections/stub/defines/e2eCase/bodyStart/bodyEnd/bodyHasContent/source/
valueValid. Citation records retain namespace/id/section/path/line/column/marker/
  shorthand/localSection/shorthandRewritable/numericRun/text/inlineSite/
  enclosingDeclaration/enclosingSection. A front-matter coordinate is a Section
record like any other, with `headingLevel` `0` and no new field
([§FS-fields.front-matter.surfaces](FS-fields.md#front-mattersurfaces-what-each-surface-does-with-them)).
An `e2eCase`'s `dir` is the case directory spelled exactly as the same record's
`path`, from the same report base, reading `.` when the case directory is that
base ([§FS-cli.3.4](FS-cli.md#34---path-base--where-report-paths-are-spelled-from)), so it does not change with where the tree sits; its
`fixtures` stay relative to that directory.
Each structural/value fact is a fixed typed record carrying the actual source
fields under camelCase, with paths rendered through core; source is discriminated
`kind: 'text'` versus `kind: 'json'` memberSlice/keyColumn/keyText/format. `kind:
'json'` means a member of a structured value source (§FS-values.terms) of either
format, and `format: 'json' | 'yaml'` says which, so a consumer that reads only
`kind` keeps working. Value numbers stay normalized
coefficient/exponent strings or source strings, never lossy JS floating values.
No new checker verdict is derived from these facts in JS.

##### 3.2.2.4 Option record details

Options in [§FS-distribution.3.2.1](FS-distribution.md#321-operations-options-and-refusals) are the exact public names and defaults. agents
is null/omitted for automatic selection, otherwise a nonempty readonly array
of canonical/claude/gemini/pi/copilot/cursor/windsurf/zed. Each maps to existing
InitAgentEntrypointSelection flags; [] rejects rather than silently auto-selects.
Integration agent names reuse known_agent's closed supported set; unsupported
names reject input. conversation/target/agent require write=true; agent also
requires conversationTarget; write with no client requires conversation or
conversationTarget. Validation precedes mutation and is reused from shared core.

#### 3.2.3 Workers, loading and source builds

##### 3.2.3.1 Execution

Use napi-rs AsyncTask over Node-API async work. Native `invoke` snapshots and
owns Rust inputs before dispatch; compute performs all config, filesystem,
scan/query/write and host-neutral record construction on libuv workers. Resolve
only marshals owned results on the live JS thread. No Tokio runtime, JS engine
logic or subprocess CLI. JS async wrappers validate/copy operands before the
first await; cwd is captured synchronously once and passed as request data.
Support worker_threads environments; never retain JS values/napi_env in compute.

An atomic writer lease shared by the loaded addon rejects a second simultaneous
mutating call with `busy/writer-busy`, rather than queueing pool workers on a
mutex. This is deliberately conservative across roots: fmt(write), init unless
check/dryRun, fetch and integrations(write). Preview/read calls do not acquire
it. Lease acquired before engine side effects, released by RAII on success,
error or unwind. Independent reads overlap. Reads concurrent with a writer may
observe filesystem changes; no atomic snapshot across calls or external writers
is promised. Callers serialize read/write groups when they need a fixed tree.
Multiple installed copies/processes are separate native instances and remain
the caller's coordination responsibility. No filesystem lock/cache is introduced.

Pin unwind-capable addon builds. Catch unwinding panics in compute and every
native conversion/resolve/reject boundary, returning native/native-panic; typed
engine errors remain errors, never panics. Build failure if panic=abort is chosen
for this supported builder. To keep contained panics silent, a once-installed
Rust panic-hook dispatcher suppresses printing only under a thread-local binding
request guard and chains the previous hook otherwise; do not repeatedly swap
hooks per call. The dispatcher captures no napi_env and lasts for the addon
lifetime. Run core parallel work inside an addon-owned bounded Rayon pool using
`ThreadPool::install`; mark its workers with the same suppression guard through
start/exit handlers. Otherwise nested scanner worker panics would print before
the compute catch sees their propagated unwind. This reuses the core's Rayon
walk, without changing its global pool or placing Node dependencies in core.
Pool ownership is reference-counted by environments and in-flight Rust jobs;
drop it after the last one finishes. TLS and writer guards drop on unwind; panic payload handling itself
must not re-panic. Validate the known non-Unicode unnamed-member case through
shared core preflight before the existing alias derivation, preserving CLI's
documented deviation until its separate fix. Recoverable panics are contained;
abort/OOM/process corruption are not an in-process isolation promise. Do not
claim catch_unwind handles those. Test hook chaining alongside another native
Rust caller, including worker completion and teardown.

Addon context is per Node environment; request Rust data lives independently
until completion. Register cleanup through Node-API/napi-rs mechanisms; stop
accepting jobs on teardown, discard callbacks to destroyed environments, and
release job resources/leases after any already-running compute completes.
Normal pending calls keep the host alive. Forceful worker termination may let
in-flight native writes finish; no rollback. No AbortSignal/cancellation/watch
API this delivery; napi's queued-only cancellation would not stop running writes.
No global cwd change, argv parsing, host exit or unexpected stdout/stderr.

##### 3.2.3.2 Module/runtime/load interface

Support Node 22.x and 24.x, N-API 8, ESM and CJS. npm engines is
`^22.0.0 || ^24.0.0`; this declares majors, not an automatic promise of all newer
Node hosts or Electron/Bun/Deno/browser runtimes. Platform support is packaging's
five addon targets: linux-x64-gnu/linux-arm64-gnu/darwin-x64/darwin-arm64/
win32-x64-msvc; no musl or Windows-arm prebuilt claim. Payload/runtime floors
follow the approved #471 matrix; this machine proves only its local source build.

Artifact name `grund.node`, internal exports `invoke` (owned request → async
native outcome) and `metadata` (apiSchemaVersion=1, engineVersion, packageVersion,
target, napiVersion=8). Each platform package exposes its exact-version addon
path and metadata. These are private ABI glue, not a standalone C ABI contract.
Local build metadata is also written as `native/metadata.json`.
The optional platform module exposes `{addonPath, metadata}`, with an absolute
addon path and the same metadata record; packaging owns its package name and
exact-version dependency. These private descriptor field names fix the loader
handoff without creating a public package subpath.
Public wrappers/types provide exactly the function inventory in the matrix.

Lazy CJS loader is the shared implementation; ESM facade uses explicit named
exports from it so require/import share errors/options semantics. Precedence:
1. Local source-built `native/grund.node` when matching metadata exists.
2. Exact same-package-version optional platform payload selected by OS/CPU/libc.
No architecture substitution, network download, CLI subprocess or silent rebuild.
Selection/loading failures reject GrundError(load), with target/attempted paths
and original cause details. A present incompatible/corrupt local artifact fails,
not silent fallback. Validate metadata before accepting requests. #471 owns
platform dependency generation and distribution, Node provides loader protocol.

Conditional exports map import to index.mjs with index.d.mts, require to
index.cjs with index.d.cts; types conditions precede implementations. Root types
may point to index.d.mts for tools reading package metadata. Private addon/build
paths are not public package exports. Files include JS/TS declarations, loader,
README/license, local native payload when built, and builder/source closure for
explicit fallback. No LSP dependency/bin. #471 alone adds grund launcher/bin,
optional platform dependencies and CLI payloads to the shared assembly manifest.

##### 3.2.3.3 Source build/local package ownership

#469 owns `crates/grund-node` Rust binding/build.rs, JS/TS assets, source builder,
and `package-api.json`: an API export/file/build metadata fragment, not a second
package manifest. #471 owns the single assembly manifest/recipe. A local staging
helper materializes a disposable API-only `grund-cli/package.json` from that
fragment and workspace version when no assembly recipe exists yet; it never
commits a competing umbrella manifest. #471 later consumes the same fragment,
adds CLI assembly and rejects divergent export/source-build metadata. Local
API-only package is explicitly labelled a rehearsal; no CLI executable claim.

Builder contract:
`node crates/grund-node/build.mjs --source-root <repo> --out-dir <stage>/native
 --target <rust-triple> --target-dir <external-cargo-target> --profile release`.
It calls Cargo with --locked against bundled workspace/core/node sources, then
copies the cdylib to grund.node and writes metadata. Default target is host;
release is ordinary LTO, not new PGO training. Require Rust/Cargo at the native
builder's pinned toolchain (1.95.0), linker/SDK and Node/npm; dependency access
or populated caches required for source builds. Node API construction stays
napi-rs; no node-gyp dependency or Node header scraping.

Disposable package has script `build:source` invoking `node build/source.mjs`.
The local helper is `node crates/grund-node/stage.mjs --out-dir <package>
--target-dir <external-cargo-target>`. The source entry point accepts an
explicit `--target-dir` and uses its bundled `build/sources/` closure.
Unpacked package includes build/source.mjs and the minimal complete Cargo source
closure (workspace/package manifests/lock/toolchain, core/node sources and core
assets), with copied workspace membership adjusted for that source bundle only.
External target-dir and output-dir are explicit; generated files stay outside
checkout. Run `npm run build:source`, `npm pack`, install tarball in a fresh
consumer, then prove ESM/CJS/TS. Also remove native artifacts from an unpacked
tarball and repeat source build/import to prove the bundled fallback, not just
a build that accidentally reaches the checkout. #471 composes this entry point
with its CLI builder; it does not invent a second native invocation. npm install
does no automatic source compilation. Lock fragment/build tool versions, record
input SHA and artifact engine/package versions for packaging and corpus matching.

The package takes the workspace version by the one mapping of
[§FS-distribution-candidate.6.2](FS-distribution-candidate.md#62-one-version-across-every-package); no repository version bump. A removal or rename
of its records or operations takes
[§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path), or before `1.0`
[§REQ-backwards-compatibility.4](../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise). #466 adaptation must not silently alter approved host/wire
behavior. Full ecosystem release alignment and a later binding-only maintenance
cadence remain #471's policy; no publication, registry changes or PGO evidence
are created here.

#### 3.2.4 Acceptance evidence

Usage/API documentation lives in `docs/user-facing/node-api.md`, with the
captured `examples/node-api/example.mjs` and `expected.stdout` example and
honest local-versus-registry status in README/guide/example indexes.
`crates/grund-node/README.md` and packaged JS/TS guidance document prerequisites,
loader/export metadata and the #471 handoff. Public examples execute against the
locally built package; no unpublished registry availability is implied.

| Ticket criterion | Failing test before implementation | Future implementation proof |
| --- | --- | --- |
| Surface and local example | `node_local_package` attempts the reproducible local builder/pack/install entry point and asserts a real package contract; fails today because the capability is absent, not because an existing addon was never built | fresh npm consumer runs check/show example and verifies dangling code/body; each exported operation and matching declaration exists |
| All operations/core/data failures | parameterized success/refusal fixtures over all 18 declared operations | Rust/Node complete-data equality, warning-preserving errors, Promise rejection for invalid args, completed failed check resolves; selection from same complete report; strict scan, partial fmt/init and batch distinctions |
| Responsive event loop/native safety | timer advances before a controlled long native compute finishes; test-only native failure seam | AsyncTask engine work off JS thread; catch unwind/native conversion and worker rejection; writer lease drops; host alive, no unwinding FFI; no production fault-injection export |
| Canonical parity | Native Node path absent; serializer/corpus assertions are blocked | same-version stdout/stderr bytes and complete envelopes; null/multi-sites/authority, Unicode/control chars, query/warning-before-failure/partial scan and repeated determinism |
| Core read/write/config semantics | copied tree mutation fixtures and forbidden fetch sentinel | compare read-only tree digest; core-owned exact post-write bytes; preview/force/dryRun/check/protected config/external symlink/whole-set preflight/partial-run refusals; reads execute no fetcher |
| Concurrency/lifecycle/silence | independent two-root options loop; barrier before writer lease release; worker-thread teardown | cwd unchanged, input mutations cannot change owned requests, repeat/concurrent reads match isolated runs; overlapping writes reject busy without side effects; cleanup safe, controlled panic silent, unrelated hook preserved; stdout/stderr empty |
| Source build/import/TS | clean staging/unpacked tarball consumer contract | pinned Node 22 and 24, Rust toolchain/linker; explicit npm build:source with missing addon; npm pack/install; ESM/CJS both work; strict tsc NodeNext and CJS consumers, wrong options rejected by TS and runtime |
| Isolation/handoff/docs | add frontend graph/engine-boundary expectations | core lacks napi/Node/frontends; Node direct core dependency only; CLI/LSP cargo builds work without npm/Node; native metadata matches fragment; package contents/types/sources; documentation examples actually captured |

Tests must use deterministic barriers for writer overlaps, not scheduling luck.
Event-loop proof measures timer progress while compute is active, not a timer
that fires before dispatch. Hook tests include an actual panic in a subprocess
consumer (the API implementation itself invokes no CLI subprocess), so unintended
host termination/streams are caught without terminating the test runner. Defer
all installed-prebuilt platform matrix, npx/LSP/PGO/publisher evidence to #471.
Local host evidence never certifies the entire support matrix.

No task-specific plan/run.sh reproducer exists. The triage capability inventory
is evidence, not the final failing user test. Specify commits spec/failing tests
first, then implement adapters/types/builder, then review/green runs binding
checks and the ordinary `pre-commit run --all-files`. Node acceptance must be
part of the documented checked workflow, not only a developer's manual command;
ordinary CLI/LSP builds still require no Node. `fissile check --staged` is required
before final completion. Local readiness is measured independently of registry availability.

The native source-build proof runs with the bounded, profile-aware harness
deadlines and supplemental mechanism checks defined in the
[Node acceptance architecture](../architecture/AR-ci.md#3411-one-release-build-and-every-build-on-one-line), preserving
the real release/LTO build and the development consumer proofs.

The documented checked workflow runs `python tests/bindings/node/run.py` as
well as `pre-commit run --all-files` and `fissile check --staged`. The binding
suite requires Node/npm, the pinned Rust toolchain, linker/SDK and dependency
access or populated caches; ordinary CLI/LSP Cargo builds need no Node tooling.
No cancellation, rollback, external-writer snapshot, abort/OOM recovery,
publishing, registry reservation, version bump or release dispatch is promised.

### 3.3 Python (`grund` PyPI package)

```python
from grund import check, show

repo = "tests/e2e/cases/json-report/repo"
result = check(repo)
for finding in result.report.errors:
    print(finding.code, finding.line)  # dangling 3
assert "FS-999-missing" in show("FS-001-alpha", root=repo, mode="brief").body
```

The Python binding requires PyO3 and maturin. Future release wheels use cibuildwheel;
the local source/build handoff is [§FS-distribution.3.3.7](FS-distribution.md#337-local-source-and-typing-handoff). Distribution and import
names are both `grund` ([§DA-pypi-uses-grund-as-the-package-name](../decisions/architectural/DA-pypi-uses-grund-as-the-package-name.md#da-pypi-uses-grund-as-the-package-name-pypi-uses-grund-as-the-package-name)).

Packaging consumes the binding's stable-ABI choice ([§FS-distribution.3.3.7](FS-distribution.md#337-local-source-and-typing-handoff)): one
`cp310-abi3` wheel per registry row carries the extension and the `grund` executable, is
proved on CPython 3.10 through 3.14 ([§FS-distribution-candidate.1.2](FS-distribution-candidate.md#12-every-registry-row-proves-every-runtime-it-promises)), and an sdist that
builds both is the explicit fallback ([§FS-distribution-candidate.4.2](FS-distribution-candidate.md#42-python-builds-from-the-sdist-only-when-asked)).

The Python frontend may require additive warning-preserving, structured-failure,
workspace-path-preflight and managed-integration adapters. They belong in core and
preserve supported Rust entry points and existing CLI defaults, verdicts and bytes.
Classification originates where the failure occurs; frontends do not parse Display
messages for diagnostic fields. The public Python schema does not freeze the hidden
Rust records it is built from. Carry “Adapt Python marshalling to #466/#453/#454” when
Python precedes that transition: replace internal adapters and rerun parity while
preserving the approved Python schema.

The additive `ApiOutcome<T>` and source-classified `ApiFailure` carriers, which
are `#[doc(hidden)]` and outside [§FS-distribution.3.1.2](FS-distribution.md#312-what-10-freezes), retain
earlier run cautions on both success and failure. Typed batch records, schema
config values and integration-install orchestration reuse current engine
loaders, queries and write policies. All supported Rust signatures and existing
Display/CLI bytes remain unchanged. Classification occurs at the producing
source, never by parsing rendered messages; unexpected recoverable failures
retain an operation class and cause chain. No Node dependency enters the core.

The following is the required local contract; registry availability remains pending.

#### 3.3.1 Typed immutable results

Results/options are frozen dataclasses; collections are tuples, with explicit None
for optional fields. Effective config is a read-only schema-keyed mapping, including
nested mappings, rather than an exported internal Rust Config record. CheckResult
carries `report`, `selected_report`, `had_scan_errors`, `output_format` and separate
`run_cautions`. Report has errors/warnings/suggestions tuples; iteration yields those
groups in that order, preserving engine order within each. Findings retain
severity/channel, code, path, line, column, message, sites and authority. Scan returns
a catalog/citations/scan-diagnostic snapshot. Other operation records retain every
supported engine output field and expose run_cautions separately. ShowQuery is a
frozen record with `id` and nullable `section=None`. Batch records have nullable
`result`/`failure` fields. Refs includes file summaries and site/file totals; list
includes summaries. Empty collections never become None.

#### 3.3.2 Operational failures and invalid arguments

GrundError has ConfigError, FilesystemError, QueryError and fallback OperationError subclasses. Each carries typed `.failure`: code,
message, nullable path/line/column, sites, authority, causes, run_cautions, partial_output and structured details (such as candidates
and OS error codes). Locations come from engine data, never text parsing. Completed checks containing findings return normally. Single
unsuccessful queries raise QueryError; a batch continues with the same failure per record, while setup failure raises once. Empty
batch input succeeds without config loading. Unknown ID kind in propose_id raises OperationError; rejected queries raise QueryError.
An fmt refused for unresolved merge conflicts ([§FS-fmt.3.2](FS-fmt.md#32-a-run-that-needs-the-whole-declaration-set-rewrites-nothing)) raises OperationError with code conflict-marker: path and line are the
first block's first line, sites holds each block's first line, partial_output holds empty changes, scan_errors and refused_writes, and
details.conflicts holds every block's finding ([§FS-declarations.checks.conflict-marker](FS-declarations.md#checksconflict-marker-unresolved-merge-conflict)). An fmt whose walk fails to read a file
carries partial_output with empty changes ([§FS-fmt.7.4.3](FS-fmt.md#743-nothing-is-written-before-the-last-refusal)). Wrong Python types raise TypeError; invalid option values raise ValueError.
PathEncodingError is a ValueError subclass for rejected path encodings.

#### 3.3.3 Per-call scope and path encoding

Paths accept str or os.PathLike[str]. root=None snapshots cwd at entry and means
omitted scope; explicit files/directories keep explicit-path engine semantics.
Reuse config discovery without config injection. Calls against different roots are
independent. Report paths are engine logical strings with `/`, independently of
native filesystem inputs. Reject bytes/byte-returning PathLike with TypeError,
surrogates with PathEncodingError and embedded NUL with ValueError before work.
Core preflight reuses workspace discovery to reject the known unnamed non-Unicode
member alias case with PathEncodingError before unsafe alias derivation. This bounded
protection changes no CLI behavior and promises no arbitrary Rust panic recovery.

#### 3.3.4 Silent synchronous calls

Calls read no process argv, print nothing, never exit Python or change global cwd.
Synchronous Rust work releases the GIL. Independent reads may overlap; callers serialize
writers to the same files. Pending Python interrupts are checked on return; Rust may finish
before KeyboardInterrupt. An interrupt arriving after the call is not evidence of loss.
No async API, mid-call cancellation or additional rollback is promised.

#### 3.3.5 Complete initial inventory

Required operands are positional; all other arguments are keywords. Every disk-tree
operation below also accepts root=None; check accepts root positionally. scan requires
its root positionally. init uses target in place of root; integrations and setup
instructions have no root. Defaults below are normative, not merely examples.

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

Modes are lead/brief/toc/full; formats text/md/json. Filters/selectors are string
sequences. Units are lines/words/bytes; top is positive, width non-negative.
Selectors narrow only selected_report; ignore wins, rule-produced codes carry
invalid-rule and every empty-citation-obligation row, rule selection requires rule,
and safety io findings remain. selected_report also holds one selected-check-off
warning per `only` code the run left off, which report does not
([§FS-check.1.4.1](FS-check.md#141-a-selected-check-the-run-leaves-off-says-so)). Kind overrides, workspace aliases, exclusions and
scope remain engine decisions. Config-enabled cross-reference formatting applies
even when cross_refs=False. No unimplemented inventory entry counts as support.

Shell scripts, stdin/NDJSON transport, watch/process lifecycle, exit codes and LSP
transport are frontend concerns. Optional overlay/on-type/hover editor utilities
are excluded from the initial disk-backed API; no CLI conceptual operation is dropped.

#### 3.3.6 Explicit mutation opt-ins

fmt previews by default. init maps write=False to dry-run; check=True suppresses
writes and reports pending changes. Force never replaces config. fetch refuses
write=False with ValueError before execution; it promises no preview. Integration
reads return artifacts/detection; writes preserve preference validation, agent gates,
managed ownership and manual steps. Reads never execute fetchers. Existing engine
data-preservation contracts and CLI defaults remain unchanged.

#### 3.3.7 Local source and typing handoff

Require CPython 3.10+ with GIL and abi3-py310. PyPy and free-threaded Python support
are not promised. The binding is one directory, `crates/grund-py/`: its pyproject.toml
selects maturin, its python/grund supplies the public API/types/py.typed, and
grund._native is private. The repository root carries no pyproject.toml or python/, and
a checkout installs the binding with `python -m pip install crates/grund-py`. Clean
checkout and independently unpacked sdist both install importable grund without registry
credentials. The sdist includes necessary workspace manifests, lockfile, Rust
sources/assets, Python/types and licence. Python-only dependencies do not become ordinary Cargo CLI requirements.
Add no competing CLI console entrypoint; release matrix, prebuilt CLI placement and
publication belong to #471. Accurate signatures/type information and runnable
check/iteration/show examples belong in user documentation and example indexes.
Local support is described independently of unpublished PyPI availability.

##### 3.3.7.1 Raw Cargo extension builds

On supported desktop platforms, enabling `grund-py/extension-module` in a raw
Cargo build must produce a Python extension that the supported interpreter can
import, including when Python and Node natives are built together from one
checkout for command agreement ([§REQ-one-answer.proof.matrix](../requirements/REQ-one-answer.md#proofmatrix-the-complete-comparison-matrix)).
The build must supply the platform's extension-module linker setup itself;
macOS must resolve Python symbols through the importing interpreter without
requiring maturin to inject linker arguments. This setup applies to the extension
cdylib without imposing extension-only linking on ordinary workspace executables
or tests. The interpreter and ABI boundary remains the one in
[§FS-distribution.3.3.7](FS-distribution.md#337-local-source-and-typing-handoff).

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
