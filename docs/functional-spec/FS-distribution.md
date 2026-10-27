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

A build from `main` reports the next version with its `-dev` suffix rather than the release it is ahead of ([§FS-distribution.4.1](FS-distribution.md#41-between-releases-main-carries-a-dev-version)); a released binary carries its tag's version, which every package of the release shares ([§FS-distribution.4.3](FS-distribution.md#43-releaseyml-publishes-a-commit-that-already-carries-its-version), [§FS-distribution-candidate.6.2](FS-distribution-candidate.md#62-one-version-across-every-package)); and no release is cut at a version that contradicts a release its own messages name ([§FS-distribution.4.2](FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name)). The release named in a warning is what buys a repository its time, and it buys nothing if it is false. Serves [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path).

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
                               // `ambiguous` refusal, which names candidates instead (FS-errors.5.2.1)
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
  preserves input/exhaustive order and the JSON-only CLI mode. Refs/list/cover
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

`grund-core` 1.0 freezes exactly the root names this point reaches: the entry points `check`, `check_with_opts`, `scan`, `show`, `list`, `list_sizes`, `refs`, `refs_outcome`, `cover`, `cover_text`, `cover_lines`, `format_references`, `propose_id`, `complete_ids`, `init`, `fetch_snapshot` and `integrations` ([§FS-distribution.3.1.3](FS-distribution.md#313-integrations-is-the-one-integrations-entry-point)), the typed refusals `ShowQueryError`, `FmtScanAbort` and `FmtConflictAbort` ([§FS-fmt.3.2](FS-fmt.md#32-a-run-that-needs-the-whole-declaration-set-rewrites-nothing)), and every root type their signatures reach, transitively. That is 115 names, `InitAllocation` and `InitAllocationHome` among them, which `InitNext.allocation` reaches ([§FS-init.2.2.2.3](FS-init.md#2223-the-kind-the-guidance-names)), and the rows whose disposition is `frozen at 1.0` in the [public-surface inventory](../discussions/proposals/2026-09-22-grund-core-public-surface-inventory.md) are the full list. An entry point with no typed refusal returns `anyhow::Result`.

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
the existing engine batch CLI operation is json-only.

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
type ScanError = { readonly path: string; readonly message: string };
type FailureKind = 'input' | 'query' | 'config' | 'io' | 'operation'
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
  ([§FS-fmt.3.5.1](FS-fmt.md#351-a-line-with-several-rewrites-carries-the-first-label)) and the same `; creates the file` or `; deletes the file`
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
the semantic edges. Strict scan errors reject, with available snapshot partial.

Declaration records: id/kind/number/slug/path/line/headingLevel/title/sections/
duplicateSections/stub/defines/e2eCase/bodyStart/bodyEnd/bodyHasContent/source/
valueValid. Citation records retain namespace/id/section/path/line/column/marker/
  shorthand/localSection/shorthandRewritable/numericRun/text/inlineSite/
  enclosingDeclaration/enclosingSection.
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

Current workspace version supplies local npm-valid dev version by the agreed
single mapping (currently 0.16.2-dev); no repository version bump. Initial API
is pre-1.0: additive records/operations allowed; removals/renames require the
existing documented compatibility/deprecation path or an accepted explicit
pre-1.0 decision. #466 adaptation must not silently alter approved host/wire
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

The tree records one explicit minimum publishable release in the single-line
`scripts/release-floor.txt`. The release guard, `scripts/check_release_ramps.py`,
reads that value from the tree being checked and refuses every version below it,
even when no diagnostic message names a landed change. All release transitions
share this one value; none adds a floor of its own. The floor remains in force
after a helper rewrites the development manifest for a release.

The effective lower bound is the greater of this floor and the message-derived
landed bound of [§FS-distribution.4.2.2](FS-distribution.md#422-the-release-guard-reads-the-releases-the-tree-names). The pending-message ceiling still applies.
At the floor a release is allowed when no independent promise forbids it; when
the lower bound meets or exceeds the ceiling, no release is possible. The
refusal identifies the floor file and value as well as contradictory messages
and the resulting release window ([§FS-distribution.4.2.5](FS-distribution.md#425-the-refusal-names-the-window-left)). Development versions
need not equal the floor; every publication path runs the guard
([§FS-distribution.4.2.6](FS-distribution.md#426-every-publication-path-runs-the-release-guard)).

## 4. Release process

The implemented release workflow publishes the Cargo CLI today and builds downloadable PGO binaries for every supported desktop CI platform. `release.yml` publishes a commit that already carries its version ([§FS-distribution.4.3](FS-distribution.md#43-releaseyml-publishes-a-commit-that-already-carries-its-version)), and two helper workflows make that commit on a candidate they validate first ([§FS-distribution.4.4](FS-distribution.md#44-two-helper-workflows-bump-the-version-on-a-validated-candidate)), with a version bump that covers the manifests, the lockfile, `--version` fixtures, ramp constants and the changelog rotation ([§FS-distribution.4.5](FS-distribution.md#45-what-the-version-bump-includes)). The release writes its own changelog section, listing the pull requests merged since the previous tag and the compatibility notices the decision records added since it ([§FS-distribution.4.6](FS-distribution.md#46-the-release-lists-the-pull-requests-merged-since-the-previous-tag)), so no change waits in the tree for it ([§FS-distribution.4.12](FS-distribution.md#412-pending-changelog-entries-are-one-file-each)); the bump writes that section inline ([§FS-distribution.4.5](FS-distribution.md#45-what-the-version-bump-includes)), and the changelog supplies the GitHub release notes ([§FS-distribution.4.7](FS-distribution.md#47-the-changelog-is-the-source-of-release-notes)). Linux binaries build in digest-pinned `manylinux2014` containers and every job on one pinned toolchain ([§FS-distribution.4.8](FS-distribution.md#48-one-old-glibc-baseline-one-pinned-toolchain)); distributed binaries are PGO-built, with an LTO-only fallback for a platform whose PGO training is broken ([§FS-distribution.4.9](FS-distribution.md#49-distributed-binaries-are-profile-guided-optimized)); crates publish to crates.io in dependency order before artifacts upload ([§FS-distribution.4.10](FS-distribution.md#410-cratesio-publishes-in-dependency-order-and-artifacts-upload-last)). npm and PyPI take a lane of their own rather than joining `release.yml`: a candidate is assembled and rehearsed without credentials, and a separate, disabled publisher may one day upload exactly it ([§FS-distribution.4.13](FS-distribution.md#413-a-candidate-is-rehearsed-before-anything-is-published), [§FS-distribution.4.11](FS-distribution.md#411-the-full-ecosystem-release)).

### 4.1 Between releases, main carries a dev version

A release leaves `main` holding the version it just published, so every build from `main` until the next release reports the tag it is already ahead of. Nothing then distinguishes a binary built from `main` from the released one, and a fix that is merged but not installed looks exactly like one that is installed. So the release advances `main` as its last act: after publishing `X.Y.Z` both helpers push `X.Y.(Z+1)-dev`, a commit they built over the release commit and ran CI on before either reached `main` ([§FS-distribution.4.4.1](FS-distribution.md#441-nothing-reaches-main-that-ci-has-not-passed)). The suffix is what makes `grund --version` say which side of the tag a build came from. A `-dev` manifest is never publishable — `release.yml` still verifies that the selected commit carries the exact version being released ([§FS-distribution.4.3](FS-distribution.md#43-releaseyml-publishes-a-commit-that-already-carries-its-version)), and the helpers set that clean version on their candidate branch — so the rule that a released version matches its tag is unchanged.

### 4.2 A release may not contradict the releases the tree's own messages name

A ramp is a promise written into a message: a warning names the release it becomes an error in, or names the release in which a scalar status will move ([§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)), and once an error ramp lands the error that replaced it names the release the change was made in. Both halves are claims about a version, and each is false at the wrong one. A pending warning shipped *at* its deadline breaks the promise it makes; a landed change shipped *below* the release its own message names is worse, because it puts a breaking change in a release whose version says there is none.

A unit test against the running version can hold only the pending half ([§FS-distribution.4.2.1](FS-distribution.md#421-a-test-can-hold-only-the-pending-half)), so the release guard reads the release each message names and refuses a version that contradicts one ([§FS-distribution.4.2.2](FS-distribution.md#422-the-release-guard-reads-the-releases-the-tree-names)), in a closed vocabulary ([§FS-distribution.4.2.3](FS-distribution.md#423-the-vocabulary-is-closed)) whose scalar clause is the `refs` warning's ([§FS-distribution.4.2.4](FS-distribution.md#424-the-scalar-clause-is-the-refs-warnings)). Its refusal names every line that disagrees and the window of releases left ([§FS-distribution.4.2.5](FS-distribution.md#425-the-refusal-names-the-window-left)), and it runs on every publication path and checks that some release remains possible on every pull request ([§FS-distribution.4.2.6](FS-distribution.md#426-every-publication-path-runs-the-release-guard)).

#### 4.2.1 A test can hold only the pending half

A unit test can hold the pending half — the bump that reaches a deadline bumps the running version, and a test can read it — but not an ordinary landed-message half: on `main` a landed message rightly names the release still to be cut, and it is false only at the version the helpers set on a candidate branch that is neither `main` nor a pull request ([§FS-distribution.4.4](FS-distribution.md#44-two-helper-workflows-bump-the-version-on-a-validated-candidate)). The suite does run on that candidate ([§FS-distribution.4.4.1](FS-distribution.md#441-nothing-reaches-main-that-ci-has-not-passed)), but only once it is pushed; the release guard asks before anything is.

This limits checks against a particular candidate version, not checks that any version remains possible. The version-free check of [§FS-distribution.4.2.5](FS-distribution.md#425-the-refusal-names-the-window-left) can hold both message bounds on `main` without choosing a release.

#### 4.2.2 The release guard reads the releases the tree names

The release path asks directly: the release guard, `scripts/check_release_ramps.py <version>`, reads the release each message names out of the tree's own message text — the Rust sources under `crates/`, and the checked `expected.stdout` and `expected.stderr` goldens under `tests/e2e/cases/` that pin the bytes a user sees — and refuses a version that contradicts one, together with the shared minimum of [§FS-distribution.release-floor](FS-distribution.md#release-floor-one-shared-minimum-publishable-release):

| clause | what the line claims | the version being cut |
|---|---|---|
| `becomes an error in <release>` | the change has not been made yet | must be **below** that release |
| `became an error in <release>` | the change has been made | must be **at or above** it |
| `was removed in <release>` | the change has been made | must be **at or above** it |
| `is removed in <release>` | a named removal has not been made yet | must be **below** that release |
| `stopped loading in <release>` | the change has been made | must be **at or above** it |
| `unchecked in <release>` | the change has been made | must be **at or above** it |
| `an error in <release>` | the change has been made | must be **at or above** it |
| `will exit <status> ... in <release>` | a scalar exit-status change has not been made yet | must be **below** that release |
| `wording changes in <release>` | the message wording has not changed yet | must be **below** that release |

Before constructing a candidate, the same preflight also reads the deprecation
ledger at [§DISC-core-concerns.9.9](../discussions/proposals/2026-09-30-core-concerns.md#99-the-deprecation-ledger) and rejects a `planned` notice whose notice
release is at or below the requested release. This applies the ledger's existing
consistency rule to the release about to be recorded as shipped. A future
`planned` notice and a `shipped` notice do not themselves refuse that release;
the message bounds and shared release floor still apply. The ledger remains
the source of notice dates and statuses, rather than a second deadline table.

#### 4.2.3 The vocabulary is closed

The vocabulary is closed on purpose: it is the wording the warnings and errors already use, so a ramp written in it is seen and a ramp written outside it names no release the release guard can read. It asks the general question rather than naming any one ramp, so a ramp that lands later is covered the day its message is written. `is removed in <release>` is the pending half of `was removed in <release>` — the clause a deprecation names its removal release with, where the two named-error clauses name a verdict's — and it is written in that spelling rather than "will be removed in" so the pending and landed halves of one removal read as the same claim in two tenses.

##### 4.2.3.1 The landed half of an attribution pair is read where it opens a clause

A verdict that moved names both of its releases in one message: the rules that carry an attribution pair print `unchecked in grund <prior>, an error in <release>`, so the same line says when the finding went unchecked and when it became an error. Both halves are claims, and the landed one is the bare `an error in <release>` — which is why the vocabulary carries that clause and not only the two it is a substring of.

Being a substring is what gives it its left boundary. `becomes an error in <release>` and `became an error in <release>` both contain it, so the bare clause is read only where it **opens a clause** — at the start of a line, or after a comma, semicolon or colon — and never where a word stands in front of it. A tense therefore keeps the direction it spells rather than also yielding a bare landed claim, and a tense the vocabulary does not carry, such as the infinitive `become an error in <release>` of a promise not yet made, is not read as landed. The boundary enumerates no tenses, so a tense added to a message later cannot silently change what the guard reads.

#### 4.2.4 The scalar clause is the `refs` warning's

The scalar clause is the one the `refs` warning of grund 0.14.0 and 0.15.0 was written in ([§FS-output-shapes.6.1.1](FS-output-shapes.md#611-in-0140)). Its replacement, from 0.16.0, is the ordinary failed-query bytes, so it gains no historical release suffix and leaves the guard no landed clause to read; a contract test holds the landed phase instead, expecting exit `1` and the warning's absence ([§FS-refs.4](FS-refs.md#4-exit-codes)).

#### 4.2.5 The refusal names the window left

The refusal names every line that disagrees and the window of releases the tree may still be cut as, combining message bounds with [§FS-distribution.release-floor](FS-distribution.md#release-floor-one-shared-minimum-publishable-release). That window can be empty when the shared floor or a landed ramp reaches an independent pending promise. Nothing may be published until the conflicting promises are fulfilled or the changes are reverted.

`scripts/check_release_ramps.py [--root <tree>]`, with no positional release, checks whether that message/shared-floor window permits any release. It exits `1` exactly when the inclusive floor is at or above the exclusive ceiling, reporting `no release at all`, both conflicting bounds and their source locations, including `scripts/release-floor.txt:1` when the shared floor conflicts. It exits `0` for a nonempty or unbounded window, even if the next patch would be refused. It reads the same sources and shared floor as the version-specific invocation, without excluding any ramp or choosing a candidate version to evaluate ledger notice deadlines. Configuration and I/O errors retain their error handling; they are not empty-window verdicts. The existing `<release>` invocation keeps its output, exit behavior, shared-floor enforcement and version-specific ledger preflight unchanged.

`main` must never hold an empty release window: a pull request that would leave one must fail before it merges, until the conflicting promises are fulfilled or the changes are reverted.

#### 4.2.6 Every publication path runs the release guard

`release.yml`'s verify job runs the release guard on the version it is about to publish, which is every publication path — a `vX.Y.Z` tag push, a manual dispatch, and the non-publishing dry run both bump helpers wait on before they touch `main`. `auto-bump.yml` and `release-minor.yml` run it again on the version they compute or explicitly select, beside their existing gates, so a bump that could not be published fails before it pushes a candidate branch rather than after. Selecting `1.0.0` supplies no bypass of the shared floor or pending retirement promises. The cross-registry publisher is a publication path too, and runs it on its manifest's version before it uploads anything ([§FS-distribution-candidate.8.1](FS-distribution-candidate.md#81-publication-is-disabled-and-gated)).

Every pull request also runs the version-free check of [§FS-distribution.4.2.5](FS-distribution.md#425-the-refusal-names-the-window-left) against the actual repository tree, with all scanned ramps included. This checks that some release exists before merging; publication preflights still check the particular version they will publish.

### 4.3 `release.yml` publishes a commit that already carries its version

A `vX.Y.Z` tag triggers `.github/workflows/release.yml` directly. The same workflow can also be run manually from the release commit: the operator enters the version, crate publishing is enabled by default, and the workflow creates `vX.Y.Z` if that tag does not already exist. If the requested tag already exists, that tag becomes the release source ref after the workflow verifies the tagged Cargo package versions match the requested version; this lets a failed release be recovered from a newer workflow commit without moving the release tag. In either entry path, the workflow verifies the tag/version matches the versions of all three Cargo packages it publishes ([§FS-distribution.4.10](FS-distribution.md#410-cratesio-publishes-in-dependency-order-and-artifacts-upload-last)), runs a fail-fast preflight on the crates.io token when crate publishing is enabled, re-runs `scripts/check-registry-names.sh` so claimed package names must still be available or owned by this project ([§FS-distribution.1.1](FS-distribution.md#11-package-names)), then builds and self-checks profile-guided-optimized binaries ([§FS-distribution.4.9](FS-distribution.md#49-distributed-binaries-are-profile-guided-optimized)) on six targets — `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`, and `aarch64-pc-windows-msvc`. `release.yml` does not bump versions: the selected commit already carries the version being released, in every version source `scripts/distribution/candidate.py versions` reads ([§FS-distribution-candidate.6.2](FS-distribution-candidate.md#62-one-version-across-every-package)), and its verify job checks them all. It publishes the Cargo crates and the GitHub release and nothing else: no entry into it reaches an npm or PyPI upload ([§FS-distribution-candidate.8.7](FS-distribution-candidate.md#87-the-cargo-release-cannot-reach-the-publisher)).

A manual run may also be given `expected_release_sha`, the full commit it is to publish. Given, the verify job refuses, before anything is built, a source that is not that commit, naming both; omitted, nothing changes, so a failed release is still recovered by hand from its tag. A release helper always gives it, and dispatches the publisher on the release-candidate branch its runs passed ([§FS-distribution.4.4.1.3](FS-distribution.md#4413-the-push-to-main-reads-main-back)). A publishing run removes the `release-candidate/` branch it was dispatched from once it has published; the non-publishing dry run never does, because it runs on the very branch and commit the publisher is dispatched from afterwards.

### 4.4 Two helper workflows bump the version on a validated candidate

The manual helper defaults to the next minor, and also accepts the deliberate first-stable selection defined in [§FS-distribution.4.4.2](FS-distribution.md#442-deliberately-select-the-first-stable-release-from-the-dispatch-main-tip). Either choice follows the same preparation, candidate validation and publication sequence below; the scheduled patch algorithm is unchanged.

Version bumping lives in separate helper workflows, and neither publishes directly. `.github/workflows/release-minor.yml` is a manual helper that starts from the latest `vX.Y.Z` tag, computes `vX.(Y+1).0`, verifies that `main` has commits since the previous tag and that CI is green on the current `main` tip, commits the workspace version bump ([§FS-distribution.4.5](FS-distribution.md#45-what-the-version-bump-includes)) to a temporary release-candidate branch, runs `release.yml` as a non-publishing dry run on that candidate and CI on it and on the `-dev` advance built over it ([§FS-distribution.4.4.1](FS-distribution.md#441-nothing-reaches-main-that-ci-has-not-passed)), and only then fast-forwards `main` and dispatches `release.yml` for the real publish. `.github/workflows/auto-bump.yml` is the scheduled/manual patch helper with the same shape: when `main` has substantive non-doc/CI changes since the latest `vX.Y.Z` tag and CI is green on that exact tip, it computes `vX.Y.(Z+1)`, validates the version bump on a temporary release-candidate branch, and publishes the bump to `main` only after the dry run and both CI runs succeed. Neither helper waits for anything to be written first: the section a release publishes is built by the bump from what the repository already holds ([§FS-distribution.4.6](FS-distribution.md#46-the-release-lists-the-pull-requests-merged-since-the-previous-tag)), so the scheduled helper's filter decides only *whether* a patch release happens and never what it lists, and either helper can run at any moment. `release-minor.yml`, dispatched by a person, refuses a range in which no pull request was merged, through the bump's own refusal ([§FS-distribution.4.6.3](FS-distribution.md#463-a-refused-release-leaves-the-tree-as-it-was)). In both, the advance to the next `-dev` version ([§FS-distribution.4.1](FS-distribution.md#41-between-releases-main-carries-a-dev-version)) is built only for a release and pushed only after it, so a run that releases nothing opens no `-dev` version. Both helpers use the release push credential configured for branch-protection bypass. Both hand everything from the candidates on to one controller, `scripts/release_candidates.py`: the rounds of [§FS-distribution.4.4.1.1](FS-distribution.md#4411-a-moved-main-is-followed-not-refused), the push to `main`, the publisher's dispatch and the `-dev` advance. What is decided once stays in each workflow, ahead of it: the version the helper resolves, the scheduled helper's substantive-change filter, and CI on the dispatch tip. Neither dispatches the rehearsal or the publisher: the scheduled patch release keeps shipping the crates and the GitHub release alone, and the cross-registry lane is started by hand ([§FS-distribution-candidate.8.7](FS-distribution-candidate.md#87-the-cargo-release-cannot-reach-the-publisher)).

#### 4.4.1 Nothing reaches `main` that CI has not passed

The gate on the current `main` tip says nothing about the commits a helper writes on top of it. The release commit rewrites every version source and the changelog, and the `-dev` advance rewrites the version again, so a test that reads either can pass on the tip and fail on the commit pushed after it. grund 0.17.0 did exactly that: its release section moved the release the tree counts as shipped, two compatibility rows still planned for it became violations, and `main` went red on the release commit and on the advance, neither of which any suite had run on. A red `main` is a release blocker nobody was told about ([§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path)).

So each helper builds both commits before it pushes either: the release commit on its release-candidate branch, and the `-dev` advance over it on a second candidate branch. It dispatches the CI workflow — the one a push to `main` runs — on each, beside the `release.yml` dry run on the release commit, and waits on all three, failing on the first that does not pass. Only then does `main` move, to exactly those commits: the release commit, which `release.yml` is dispatched to publish, and then the advance. No commit is made after the runs, so nothing reaches `main` that they did not pass. A `main` the merge queue moves while the runs are going is followed rather than refused ([§FS-distribution.4.4.1.1](FS-distribution.md#4411-a-moved-main-is-followed-not-refused)), and only a run that did not pass leaves candidate branches for a person to read, with `main` as it was ([§FS-distribution.4.4.1.4](FS-distribution.md#4414-only-a-failure-leaves-branches)). A rejected push of the advance is not yet a refusal: while `main` is still the release commit the advance was built over, the same tested commit is pushed again, a few times with a short backoff; only a `main` that has moved is refused as moved, and a push that keeps failing onto an unmoved `main` names git's own error rather than a cause nobody checked.

##### 4.4.1.1 A moved `main` is followed, not refused

Every pull request lands through [the merge queue](../architecture/AR-ci.md#12-a-merge-queue-lands-every-pull-request), so `main` moves whenever the queue lands one, and the candidate runs take an hour. A helper that refused every moved `main` could release only in an hour nothing merged in. So a helper not pinned to a tip — `auto-bump.yml`, and `release-minor.yml` dispatched without `expected_main_sha` ([§FS-distribution.4.4.2](FS-distribution.md#442-deliberately-select-the-first-stable-release-from-the-dispatch-main-tip)) — works in rounds, and follows `main` from one to the next ([§DF-release-candidates-follow-main](../decisions/functional/DF-release-candidates-follow-main.md#df-release-candidates-follow-main-a-release-helper-follows-a-main-the-merge-queue-moves-rebuilding-and-re-running-its-candidates)).

A round fetches `main` and the tags and builds both candidates on that tip by the whole preparation: the version bump and lockfile ([§FS-distribution.4.5](FS-distribution.md#45-what-the-version-bump-includes)), the release section from the range since the previous tag ([§FS-distribution.4.6](FS-distribution.md#46-the-release-lists-the-pull-requests-merged-since-the-previous-tag)), the release guard ([§FS-distribution.4.2.6](FS-distribution.md#426-every-publication-path-runs-the-release-guard)), and the `-dev` advance over the release commit. It pushes them to two branches named for the helper's run, its attempt and the round — one path component under `release-candidate/`, such as `release-candidate/v0.20.0-<run>-<attempt>-r<n>` and the same with `-dev`, never a name an earlier round used — and dispatches the three runs, keeping their run IDs, so that a round counts only the runs it started.

While the runs are going, the helper reads `main` on every look at them. When `main` has moved, the round is given up: its runs still going are cancelled, its two branches deleted, and the helper waits until `main` has held still for three minutes before the next round builds on the new tip. That round runs all three runs again on the commits it rebuilt; nothing one round passed counts for another. `main` only ever fast-forwards, and only to the commits the last round's runs passed. A release dispatched while the queue is landing pull requests therefore ships them under it, and its section lists them.

What is decided once is not decided again: CI on the dispatch tip, and the scheduled helper's substantive-change filter, which more merges cannot make false ([§FS-distribution.4.4](FS-distribution.md#44-two-helper-workflows-bump-the-version-on-a-validated-candidate)). The full CI on each rebuilt candidate covers the new tip. What a merge can change is checked on every round, before the round pushes anything: when the target tag now exists, or the version sources on the new tip no longer read what they read when the helper started, the helper stops, naming the tag or both versions, and publishes nothing.

##### 4.4.1.2 Six rounds within five hours

Rounds are bounded twice. At most six run, and the helper's job has a timeout of 300 minutes, of which a round starts only while at least 90 remain. The watch on a round has a deadline of its own, early enough that the push to `main`, the publisher's dispatch and the `-dev` advance still fit after it; when it passes, the round's runs are cancelled and the helper stops with its own message, rather than being killed by the timeout with runs still going, or between the push and the dispatch.

A helper that runs out of rounds or of time publishes nothing and says which ran out. Six rounds cannot promise a release against a queue that never holds still for an hour in five; when they are spent, the helper is dispatched again.

##### 4.4.1.3 The push to `main` reads `main` back

The release commit the last round's runs passed goes to `main` by a plain push, never a forced one. When git refuses it, the helper reads `main` before it blames anything, as the advance does: `main` at the release commit means the push landed and the helper goes on; `main` still at the round's base means git refused for a reason of its own, and the helper stops with git's error and keeps the round's branches; any other commit means the queue landed first, which is a move like any other ([§FS-distribution.4.4.1.1](FS-distribution.md#4411-a-moved-main-is-followed-not-refused)) and costs a round.

Once `main` holds the release commit, `release.yml` is dispatched on the round's release-candidate branch with `expected_release_sha` set to that commit ([§FS-distribution.4.3](FS-distribution.md#43-releaseyml-publishes-a-commit-that-already-carries-its-version)), and then the advance follows it. A dispatch that fails says that the release commit has landed and how to publish it, and does not start the release over.

##### 4.4.1.4 Only a failure leaves branches

A run that did not pass ends the helper: the round's runs still going are cancelled, and that round's two branches stay for a person to read, with `main` untouched. A failed run seen on the same look as a move is a failure, not a move. Every other ending deletes the round's branches, because nothing on them is evidence of anything: a round given up for a move, a helper out of rounds or time, and a pinned run refused on movement ([§FS-distribution.4.4.2](FS-distribution.md#442-deliberately-select-the-first-stable-release-from-the-dispatch-main-tip)). A rebuild the preparation refuses — a ramp a merged pull request landed, say — stops the helper before the round pushes a branch or starts a run. A forge call that cancels a run the forge reports as already finished ends nothing.

#### 4.4.2 Deliberately select the first stable release from the dispatch main tip

`release-minor.yml` has a `version` choice, `next-minor` by default or `1.0.0`, and an optional `expected_main_sha` string. Omitting both inputs retains the next-minor algorithm in [§FS-distribution.4.4](FS-distribution.md#44-two-helper-workflows-bump-the-version-on-a-validated-candidate). Selecting `1.0.0` requires `expected_main_sha`: a full 40-hex commit SHA equal to the workflow's dispatch SHA, the checkout's `HEAD`, and the fetched `origin/main` tip. It identifies the **main tip the owner dispatches from**, not the freeze candidate recorded for 1.0; comparing that tip to the freeze evidence remains the owner's responsibility.

Before any version, fixture or changelog write or candidate push, explicit first-stable selection requires dispatch from `refs/heads/main`, agreement of all source versions read by `scripts/distribution/candidate.py versions` ([§FS-distribution-candidate.6.2](FS-distribution-candidate.md#62-one-version-across-every-package)), a `0.x` source version, and a latest stable `v0.x.y` tag reachable from that tip. There must be commits since that tag and no stable `vX.Y.Z` tag at or above `v1.0.0`, including on another branch. Missing or malformed required inputs, unsupported selections or refs, a stale or mismatched SHA, missing fetched main tip, disagreement between version sources, no stable tag, an unreachable latest stable tag, an empty range, or a source already at major 1 or later refuse with a diagnostic naming the condition and no writes. Prerelease tags are not stable history. An `expected_main_sha` supplied with `next-minor` must satisfy the same SHA agreement; omission preserves ordinary minor behavior.

One selector, `scripts/release_version.py`, owns that selection and preflight, reusing the candidate version reader. It returns the helper's existing `current`, `next`, `current_tag` and `next_tag` outputs; for a latest `v0.21.3`, default selection gives `0.22.0` while explicit first-stable selection gives `1.0.0`. The numbers in that example impose no retirement deadline. Selection only reads local history and source versions; the workflow fetches main and tags before selection and applies the release guard to the result ([§FS-distribution.4.2.6](FS-distribution.md#426-every-publication-path-runs-the-release-guard)).

Both choices use the complete bump ([§FS-distribution.4.5](FS-distribution.md#45-what-the-version-bump-includes)), exact dispatch-tip CI, both candidate commits' full CI and the non-publishing release dry run ([§FS-distribution.4.4.1](FS-distribution.md#441-nothing-reaches-main-that-ci-has-not-passed)). Only after those pass does the helper push the tested release commit, dispatch `release.yml`, then push the tested development commit; for `1.0.0` that advance is `1.0.1-dev`, Python `1.0.1.dev0`. There is no first-stable-only wait on publication. With `expected_main_sha`, main movement still refuses advancement: a pinned run follows no move ([§FS-distribution.4.4.1.1](FS-distribution.md#4411-a-moved-main-is-followed-not-refused)), and once its candidate runs are going it cancels them, deletes its branches and names the new tip, to be dispatched again with a fresh pin. Failed validation retains candidate branches. The publisher still requires matching manifests ([§FS-distribution.4.3](FS-distribution.md#43-releaseyml-publishes-a-commit-that-already-carries-its-version)) and a failed publication is retried from its release tag as for a minor. This selection adds no publisher and changes no publication authority; implementing or merging the helper does not authorize or perform a release.

### 4.5 What the version bump includes

In both helpers, including either manual version choice ([§FS-distribution.4.4.2](FS-distribution.md#442-deliberately-select-the-first-stable-release-from-the-dispatch-main-tip)), "the version bump" includes:

- the Cargo manifests and the lockfile, and `pyproject.toml`'s PEP 440 spelling of the same version, all written by `scripts/distribution/candidate.py set-version`; the npm packages take theirs from those at assembly, so no committed npm file carries one ([§FS-distribution-candidate.6.2](FS-distribution-candidate.md#62-one-version-across-every-package));
- any checked fixture whose expected output embeds `grund --version`, including
  `tests/e2e/cases/cli-version/expected.stdout` and its corresponding authored
  `inputFiles` bytes in
  `tests/integration/command_agreement_facts/cli-version.json`. The same
  `candidate.py set-version` invocation owns both version-dependent writes, for
  release and development versions alike; repeating it leaves them consistent.
  It preserves unrelated fixture inputs, fact inputs and independent expected
  answers ([§REQ-one-answer.proof.evidence](../requirements/REQ-one-answer.md#proofevidence-execution-and-independent-truth)), rather than rebuilding an inventory
  from observed files or execution. Both helpers stage the complete version
  update in each release and development candidate commit, so a later commit
  cannot conceal an incomplete release commit;
- every **ramp constant** whose window is stated as a version in message text — a deprecation deadline is a promise about a release, and a bump is the only moment it can come due ([§DF-index-compatibility-ramp.2.3](../decisions/functional/DF-index-compatibility-ramp.md#23-both-findings-name-their-versions-and-a-test-keeps-the-names-honest) states the rule). Both ramps that rule was written for have since landed; the live ones are the absorbed-scan warning, the narrowed-alias scope suffix, and the `agents-init` compatibility tail, and the absorbed-scan ramp has a unit test that fails the bump which reaches its deadline rather than letting it pass silently ([§FS-distribution.4.2.1](FS-distribution.md#421-a-test-can-hold-only-the-pending-half));
- the deterministic changelog rotation performed by `scripts/prepare_changelog_release.py prepare`: the section [§FS-distribution.4.6](FS-distribution.md#46-the-release-lists-the-pull-requests-merged-since-the-previous-tag) builds — the list of pull requests and, after it, its compatibility notices — becomes the new inline release section, written where the former inline latest release stood; the former is archived under `docs/changelog/<version>.md`, and the older-release section gains its archive link, `- [<version>](changelog/<version>.md) — <date>: N pull requests, M compatibility notices.`, where M counts the archived section's bullets under `### Compatibility notices` and N every other bullet it holds, so a release archived in the earlier Keep-a-Changelog shape counts every bullet as a pull request; a count of one is written in the singular. `scripts/prepare_changelog_release.py preview` prints the body `prepare` would write under the new release heading, and writes nothing.

The helper refuses rather than publish an incomplete or invented section in every case [§FS-distribution.4.6.3](FS-distribution.md#463-a-refused-release-leaves-the-tree-as-it-was) names, before it writes anything, so the release candidate is already e2e-clean and changelog-clean before `release.yml` runs.

#### 4.5.1 Release-helper verification owns its temporary history through cleanup

The tests of release listing, compatibility notices and archive rotation use real
temporary Git histories.
Their result includes strict removal of those histories: a successful release-helper
assertion followed by a background Git writer racing cleanup is a failed test, not
a release-helper failure to dismiss by rerunning. Automatic repository maintenance
must not outlive the fixture's commands or write during cleanup. The fixture keeps
cleanup errors visible; suppressing them does not satisfy the gate that protects
[§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path).

### 4.6 The release lists the pull requests merged since the previous tag

No change is gated on the changelog, and nothing is written for it before a release either: no hook, no CI job and no release step asks a change, or a person, for an entry. The bump ([§FS-distribution.4.5](FS-distribution.md#45-what-the-version-bump-includes)) builds the release section from two things the repository already holds — the pull requests merged since the previous tag, each by its title ([§FS-distribution.4.6.1](FS-distribution.md#461-the-range-is-every-commit-since-the-previous-tag), [§FS-distribution.4.6.2](FS-distribution.md#462-one-line-per-pull-request-its-title-linked)), and the compatibility notices the decision records added since that tag ([§FS-distribution.4.6.4](FS-distribution.md#464-compatibility-notices-come-from-the-decisions)) — or refuses and writes nothing ([§FS-distribution.4.6.3](FS-distribution.md#463-a-refused-release-leaves-the-tree-as-it-was)). So a pull request's title is its release line, and the one thing a title cannot carry, what a verdict change breaks and for whom, is written where the change is decided.

The list is every merged pull request, with no docs or CI filter. `docs/` holds the specification, so a pull request that changes only docs can change what the tool promises, and a filter would hide exactly those. The scheduled helper's own filter ([§FS-distribution.4.4](FS-distribution.md#44-two-helper-workflows-bump-the-version-on-a-validated-candidate)) decides whether a patch release happens at all, not what a release lists. A list built from the merged pull requests cannot leave one out, which a hand-written record could only ask its writer to avoid; that is the release half of [§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path).

#### 4.6.1 The range is every commit since the previous tag

`HEAD` is read once, first, and everything after reads that commit. The previous release is the highest `vX.Y.Z` tag reachable from it — a higher tag on a branch `HEAD` does not contain is not a previous release — and that tag must name the release `docs/changelog.md` keeps inline. For every commit in `<tag>..HEAD` the bump asks the forge which pull requests the commit belongs to, and keeps a pull request only when it is merged, its base is `main`, and its merge commit is in the range; a pull request is listed once however many of its commits the range holds.

Merges are rebase-only, so every commit of a merged pull request lands on `main`, and a list that has asked about every commit in the range has seen every pull request merged into it. A commit that belongs to no pull request contributes no line. The commit that opens the next development version, `Open X.Y.Z-dev for development` ([§FS-distribution.4.1](FS-distribution.md#41-between-releases-main-carries-a-dev-version)), is expected to be one; any other is named in one warning on standard error, by its short hash and subject, and the release goes on.

The list is newest first, ordered by where each pull request's last commit sits on `main`. The order is read from git, never from the forge's timestamps.

#### 4.6.2 One line per pull request, its title linked

Each pull request is one line, `- [<title>](<url>) (PR #N)`. The title is the pull request's own, with every run of whitespace collapsed to one space and the ends trimmed; each of `` \ ` * _ [ ] < > & `` in it is escaped with a backslash, and `§` is written `&sect;`, because `docs/changelog.md` is scanned ([§FS-check](FS-check.md#fs-check-grund-validates-every-citation-in-a-repo)) and a title must not become a live citation the release is then held to. The URL is the pull request's address on this repository, `<repository>/pull/N` for the same `N`, where the repository is the one the forge answers for; a pull request whose URL is anything else is refused ([§FS-distribution.4.6.3](FS-distribution.md#463-a-refused-release-leaves-the-tree-as-it-was)), because a line that links elsewhere than it says is worse than none.

#### 4.6.3 A refused release leaves the tree as it was

The bump reads everything it needs — the range, every commit's pull requests, and every notice it would publish — before it writes anything. It refuses, exits non-zero, names which case it met, and leaves every file byte for byte as it found it, when:

- `gh` is missing, a call to the forge still fails after its three attempts ([§FS-distribution.4.6.3.1](FS-distribution.md#4631-a-forge-call-is-tried-three-times)), or any commit in the range goes unanswered;
- the checkout is shallow, or the previous tag does not name the release `docs/changelog.md` keeps inline;
- a compatibility notice it would publish is malformed ([§FS-distribution.4.6.4](FS-distribution.md#464-compatibility-notices-come-from-the-decisions)), or a pull request's URL is not this repository's ([§FS-distribution.4.6.2](FS-distribution.md#462-one-line-per-pull-request-its-title-linked));
- the range holds no merged pull request at all.

A partial list is never written: a commit the forge did not answer for may be exactly the pull request a reader needed to see.

##### 4.6.3.1 A forge call is tried three times

A release asks the forge one question per commit in its range, hundreds of them in a row, so a refusal on the first dropped connection would make a failed release likelier with every commit, and would say nothing about the commits themselves. Each forge call the bump makes — for the repository, and for each commit's pull requests ([§FS-distribution.4.6.1](FS-distribution.md#461-the-range-is-every-commit-since-the-previous-tag)) — is therefore tried at most three times, waiting one second after the first failed attempt and two seconds after the second; a call answered on its first attempt waits for nothing.

- An attempt fails when `gh` exits non-zero, whatever it printed, or when it exits zero with output that does not parse as JSON, which is what a truncated answer looks like. Both kinds count against the one budget of three, in whatever order they come. No message is read to tell a transient failure from a lasting one: a lasting one costs three seconds and the same refusal.
- When the third attempt fails too, the bump refuses with that attempt's own diagnostic, unchanged — `the forge did not answer for …` or `the forge's answer for … is not JSON: …` — and writes nothing ([§FS-distribution.4.6.3](FS-distribution.md#463-a-refused-release-leaves-the-tree-as-it-was)).
- A missing `gh` is refused before any attempt. An answer that parses but is the wrong shape — not a list for a commit, no address for the repository, a pull request whose URL is not this repository's ([§FS-distribution.4.6.2](FS-distribution.md#462-one-line-per-pull-request-its-title-linked)) — is refused at once and never retried: the forge did answer, and asking again would mistake bad data for an interrupted reply.

#### 4.6.4 Compatibility notices come from the decisions

A decision record — a `DF` or a `DA` declaration — may carry the named section `## release-note: Release note`, holding exactly one bullet (`- ` and its text, continuation lines indented two spaces) and nothing else but blank lines, with its links written relative to the record's own file. A verdict correction must carry one: it is the release record [§REQ-backwards-compatibility.5](../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids) condition 3 and [§REQ-backwards-compatibility.3](../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) ask for, and the meter `tests/integration/test_backwards_compatibility_routes.py` reads it there, offline, on the pull request that makes the correction.

The bump publishes, under `### Compatibility notices` after the list, the bullet of every decision whose `release-note` section exists at `HEAD` and did not exist at the previous tag, ordered by decision ID. The two trees are compared by decision ID, so a record moved or renamed since the tag is the same record. Each bullet is published as written, with every relative link rebased from the record's directory to `docs/`, and a link to an anchor of the record's own file gaining that file's path. A notice already present at the tag is not published again, however it was edited since, because it shipped with the release that added it. A release that adds no notice has no `### Compatibility notices` subsection. A `release-note` section it would publish that is not one well-formed bullet is refused ([§FS-distribution.4.6.3](FS-distribution.md#463-a-refused-release-leaves-the-tree-as-it-was)).

##### 4.6.4.1 The shape is held on the pull request that writes it

The bump is not the first to read a notice's shape. The repository's Python gate, which runs at every commit and on every pull request, reads every decision record's `release-note` section in the working tree with the bump's own reading, and fails when one is not a single well-formed bullet, naming each such decision and its record's file in the one run rather than stopping at the first. It reads every section, not only those the next bump would publish, because the shape above is every section's. So a notice the bump would refuse fails on the pull request that writes it, while its author can still fix it, rather than at the next release, where only the person cutting it could. This asks no change for a notice — a decision that writes none is not read — so it gates no change on the changelog ([§FS-distribution.4.6](FS-distribution.md#46-the-release-lists-the-pull-requests-merged-since-the-previous-tag)); it holds only a section a change chose to write.

##### 4.6.4.2 A correction-audit refusal explains its citation trigger

The repository's correction-route meter audits every decision whose text contains
the live citation `§REQ-backwards-compatibility.5`, including a sentence that
discusses or rejects that route. Selection does not interpret that sentence's
intent. The five conditions remain those of
[§REQ-backwards-compatibility.5](../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids).

Whenever that audit refuses a selected decision, the contributor-visible failure
must explain that the live section-5 citation put the decision under the
correction-route audit. It must retain the placement or proof reasons and name
both ways to discuss or reject the route without invoking it: write
`<§>REQ-backwards-compatibility.5`, using the existing
[§FS-citations.recognized.escapes](FS-citations.md#recognizedescapes-an-id-in-an-escape-position)
convention, or cite `§REQ-backwards-compatibility` and name section 5 in prose.
The guidance must remain visible in the actual test failure, including when the
rejecting sentence is already in a numbered section and the refusal lists
multiple missing proofs. These alternatives leave the decision outside this
audit; a live rejecting citation still fails without the required proof, an
unproved correction still fails, and a valid correction still passes.

### 4.7 The changelog is the source of release notes

`release.yml` treats the changelog as the source of GitHub release notes. Before creating or updating a GitHub release, it extracts the requested `vX.Y.Z` section from `docs/changelog.md` at the selected release ref and passes that body to `gh release create` or `gh release edit`; if the section is missing or empty, the release fails before artifacts are published to the GitHub release.

### 4.8 One old glibc baseline, one pinned toolchain

Both Linux binaries are built on GitHub inside a `manylinux2014_<arch>` container (pinned by digest, not by tag) so the release artifact targets an old glibc baseline instead of inheriting whatever glibc happens to ship on `ubuntu-latest`. Every job — host-runner or in-container — installs the same pinned Rust toolchain so the six binaries are produced by the same compiler version. Every candidate payload is built the same way, for the six rows of [§FS-distribution-candidate.1.1](FS-distribution-candidate.md#11-six-rows-five-of-them-registry-rows), which pins each row's payload floor, runtime floors and installed-artifact runner ([§FS-distribution-candidate.1.3](FS-distribution-candidate.md#13-runtime-floors-are-inspected-not-assumed)); six `grund-lsp` archives join the six CLI archives.

### 4.9 Distributed binaries are profile-guided-optimized

The distributed `grund` binaries are profile-guided-optimized: each platform build runs `scripts/pgo-build.sh`, which builds an instrumented binary, runs the [AR-benchmarks](../architecture/AR-benchmarks.md#ar-benchmarks-instruction-counting-benchmarks-for-the-hot-cli-commands) self-repo hot command list (the commands agents and CI invoke most) against `grund`'s own conformant tree to record a profile, then rebuilds against it. The rationale, and why the self-repo benchmark workload is also the PGO training corpus, is [§DA-pgo-release](../decisions/architectural/DA-pgo-release.md#da-pgo-release-distributed-binaries-are-pgo-built-trained-on-the-benchmark-workload). If a hosted runner's PGO training is platform-broken while the ordinary release build still self-checks cleanly, that platform may publish a self-checked LTO-only fallback binary instead of blocking the whole release; Windows arm64 is the one such platform, where rustc crashes compiling the instrumented build, and in a candidate only that identified crash qualifies ([§FS-distribution-candidate.7.4](FS-distribution-candidate.md#74-only-an-identified-windows-arm64-compiler-crash-falls-back)). Every candidate payload — both executables, the Node addon and the Python extension — trains on its own product's workload, and the manifest proves the packaged bytes are the profile-use build ([§FS-distribution-candidate.7.1](FS-distribution-candidate.md#71-each-product-trains-on-its-own-workload), [§FS-distribution-candidate.7.3](FS-distribution-candidate.md#73-the-packaged-payload-is-the-profile-use-build)). PGO is not part of development builds or push/PR CI; it is a release-packaging step, and an explicit benchmarking step when comparing the optimized release artifact. A `cargo install grund` from source is LTO-optimized but not PGO'd — `cargo install` runs no custom build step — and is byte-for-byte behavior-identical to the distributed binary; only its performance differs.

### 4.10 crates.io publishes in dependency order, and artifacts upload last

After every platform binary passes its self-check, the workflow publishes `grund-core`, `grund`, and `grund-lsp` to crates.io when crate publishing is enabled. The dependency order is fixed: `grund-core` publishes first, and any run that still needs to publish `grund` or `grund-lsp` waits up to 30 minutes for Cargo to resolve the matching `grund-core` version before publishing the dependent crate. GitHub release artifacts are uploaded only after the platform PGO builds pass and, when enabled, crates.io publishing succeeds. The cross-registry publisher keeps Cargo core-first: it uploads nothing until the matching crates resolve ([§FS-distribution-candidate.8.4](FS-distribution-candidate.md#84-order-and-readiness)), and a rerun skips what a registry already holds with the same digest and stops on a different one rather than overwriting ([§FS-distribution-candidate.8.5](FS-distribution-candidate.md#85-reruns-resume-and-never-overwrite)).

### 4.11 The full-ecosystem release

The full-ecosystem release publishes one rehearsed candidate ([§FS-distribution.4.13](FS-distribution.md#413-a-candidate-is-rehearsed-before-anything-is-published)): the crates through `release.yml` as today, then the npm and PyPI packages of [§FS-distribution-candidate.2.1](FS-distribution-candidate.md#21-thirty-nine-artifacts-from-one-version) through the publisher, from the same commit at the same version.

A release is complete only when every artifact the candidate's plan names is published; until then it is partial, and the publisher says so, listing what is published and what is still missing, and resumes the same candidate on a rerun ([§FS-distribution-candidate.8.5](FS-distribution-candidate.md#85-reruns-resume-and-never-overwrite)). Versions across the CLI and the LSP move together within a release; each `grund-lsp` package pins or bundles the same `grund-core` version the matching CLI release ships, so a CLI/LSP version mismatch from the official packages is structurally avoided.

### 4.12 Pending changelog entries are one file each

Retired. No change waits in the tree for a release: the release lists the pull requests merged since the previous tag itself ([§FS-distribution.4.6](FS-distribution.md#46-the-release-lists-the-pull-requests-merged-since-the-previous-tag)), and the one record a change still writes for a release, a compatibility notice, lives on the decision record that makes the change ([§FS-distribution.4.6.4](FS-distribution.md#464-compatibility-notices-come-from-the-decisions)). `docs/changelog/unreleased/`, the store of pending entries this section specified, is gone with it. The number and its heading stay so that neither is reused, the way [§FS-config.2](FS-config.md#2-precedence) outlived its withdrawal.

### 4.13 A candidate is rehearsed before anything is published

Every npm and PyPI artifact comes from one candidate: assembled from one commit, recorded in an immutable `manifest.json`, installed fresh and proved on every registry row by a rehearsal that holds no credential, and receipted per row. Only those bytes may ever be uploaded, by a publisher that is disabled today and reachable only by a gated manual dispatch naming the candidate's SHA and manifest digest. The detail is [§FS-distribution-candidate](FS-distribution-candidate.md#fs-distribution-candidate-one-candidate-is-assembled-rehearsed-and-verified-before-any-registry-sees-it).

## 5. What we do not promise

- 100% identical APIs across languages. Each binding is idiomatic to its host (camelCase for Node, snake_case for Python, `Result<T,E>` for Rust). The *behavior* is identical; the surface fits each ecosystem.
- Stable ABI for the C-level FFI. Bindings link against the Rust core at compile time; we do not ship a separate C library.
