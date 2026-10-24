# AR-ci: CI mirrors the local pre-commit gate

The CI workflow is the remote form of the local pre-commit gate. Anything that can abort a local commit must also abort CI, so a contributor cannot bypass the repository's local checks by skipping hooks or editing through a web UI. This supports [§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path) and keeps the link-checking boundary from [§FS-non-goals.1](../functional-spec/FS-non-goals.md#1-markdown-link-validation) enforced alongside `grund check`.

## placement: What CI measures

```text
pre-commit hook list ─► [ CI ] ─► one verdict per push, pull request or merge-queue group
                         │
                         └─ runs the tree's own grund through cargo run; outside the pipeline
```

Not a component: CI is the remote form of the local gate over the whole tree, and it measures the system rather than sits in it ([§AR-system.5](README.md#5-what-holds-the-shape)). It takes the pre-commit hook list and gives one verdict per push, pull request or merge-queue group ([§AR-ci.12](AR-ci.md#12-a-merge-queue-lands-every-pull-request)). The only `grund` it runs is the one the tree builds, through `cargo run`, so the gate never depends on a released binary; it knows nothing of the pipeline ([§AR-system.1](README.md#1-the-system)) beyond the commands the hooks spell (section 1).

## terms: Terms

Leans on [§FS-terms.terms.1](../functional-spec/FS-terms.md#terms1-declarations-and-coordinates) (ID, home, citable, section, lead),
[§FS-terms.terms.2](../functional-spec/FS-terms.md#terms2-citations) (citation, citation site), [§FS-terms.terms.4](../functional-spec/FS-terms.md#terms4-scanning-and-project-structure) (scan),
[§FS-terms.terms.5](../functional-spec/FS-terms.md#terms5-findings) (verdict), and [§FS-terms.terms.8](../functional-spec/FS-terms.md#terms8-the-architectures-own-words) (meter).

## 1. Pre-commit is the source of truth

The hook list lives in `.pre-commit-config.yaml`. CI must invoke that list directly with `pre-commit run --all-files`, rather than hand-copying each hook into separate workflow steps. The workflow may install hook prerequisites first, but the set of checks is defined by the pre-commit config. It may also skip, through pre-commit's own `SKIP`, exactly the hooks whose command another job of the workflow already runs ([§AR-ci.13.3](AR-ci.md#133-every-hook-runs-once)): the config still decides what is checked, and the workflow decides only where each check runs, never whether.

When a new pre-commit hook is added, the same change must ensure CI can run it. If the hook needs an external binary, the CI workflow installs that binary before the pre-commit step. If the hook is intentionally local-only, it does not belong in `.pre-commit-config.yaml`; put it in a developer-local hook instead.

### 1.1 Stages without a file list

`pre-commit run --all-files` reproduces only the stages whose input is a file list. A hook bound to a stage with different input — `commit-msg`, whose argument is a message file, or `pre-push`, whose arguments are the refs being pushed — is silently absent from that run, so "CI runs the config" is not by itself parity. Every hook bound *only* to such a stage needs an explicit CI counterpart that feeds it the input the stage would have supplied **and reaches the same verdict on the same tree**: [§AR-ci.8](AR-ci.md#8-commit-message-attribution-gate) is the `commit-msg` one, and no hook is bound only to `pre-push` any more ([§AR-ci.7](AR-ci.md#7-no-changelog-gate)). A hook that also runs at `pre-commit` — `cargo-test`, the one hook `pre-push` still runs — is reproduced by the all-files run and is not implicated. The verdict half is the half that bites: a counterpart fed different input is worse than a missing one, because it passes locally and then fails remotely on a tree the contributor cannot change.

### 1.2 The parity test

All of this is checked rather than remembered: `tests/integration/test_ci_precommit_parity.py` reads the hook list, workflow and Python wrapper and fails when CI stops invoking the hook list, a hook's binary has no pinned install step before that gate, a `commit-msg` hook has no range-scanning counterpart, the Rust hooks and the workflow's own steps spell different commands, [§AR-ci.3](AR-ci.md#3-current-hooks) does not spell a command a Rust hook runs, warnings are denied on one side only, the Python gates invoke different wrappers or discover tests from the wrong directory, a hook bound only to a stage without a file list has no CI step naming the same `scripts/` script, or the pre-commit step's `SKIP` names a hook no other job runs, or leaves out one that a job does ([§AR-ci.13.3](AR-ci.md#133-every-hook-runs-once)). What it holds is the *existence* of a counterpart, which is line-shaped; the *same verdict* half of [§AR-ci.1.1](AR-ci.md#11-stages-without-a-file-list) is held by each gate's own test, which runs both invocations against one fixture tree and asserts they agree — for the attribution gate, `tests/integration/test_check_no_claude_attribution.py`.

## 2. Platform scope

The full Rust build and test matrix runs on every configured operating system: on Ubuntu and Windows for every push, pull request and merge-queue group, and on macOS once a day and on every release candidate ([§AR-ci.14](AR-ci.md#14-macos-runs-once-a-day)). The pre-commit gate may run on one representative CI platform when the hooks are platform-independent, because its job is policy parity with local commits, not cross-platform behavior coverage. Platform-specific behavior belongs in the build and test jobs.

CI dependency and build caches are performance optimizations only. A cache restore/save failure must not abort the matrix before the actual checks run; the job should continue cold and let `fmt`, Python tests, pre-commit, build, self-check, tests, or benchmarks decide pass/fail.

## 3. Current hooks

The current pre-commit gate runs the same Rust format/build/test commands that development CI runs: `cargo fmt --all -- --check`, `cargo build --quiet --workspace --all-targets --locked` with warnings denied, and `python scripts/run_cargo_suite.py --workspace --all-targets --locked --features grund/test-workspace-load-count`, which runs `cargo test` with those arguments and prints what [§AR-ci.requirements.5.1](AR-ci.md#requirements51-a-suite-prints-its-failures-its-check-ins-and-one-summary) asks rather than a line per test. CI runs that last command as its shards, one job each, which together run exactly the tests it runs ([§AR-ci.13.7](AR-ci.md#137-a-legs-cargo-suite-runs-in-shards)). A plain `cargo test --workspace --all-targets --locked` is green on the same tree but runs less: it reports every test that reads the workspace-load observer as `ignored`, naming the feature that test needs ([§AR-ci.3.3](AR-ci.md#33-test-only-observers)). The test hook also runs at `pre-push`, so a contributor who commits while a test is transiently broken still gets the same local stop before sending the branch. It is the one hook `pre-push` runs: no hook asks a change for a changelog entry at any stage, because no change is gated on the changelog ([§FS-distribution.4.6](../functional-spec/FS-distribution.md#46-the-release-lists-the-pull-requests-merged-since-the-previous-tag)).

The local Python gate runs `python scripts/run_python_gate.py`, and CI runs that
same command as its four selections, in three jobs ([§AR-ci.3.4.3](AR-ci.md#343-selecting-the-binding-acceptances-and-the-slow-tests)). The wrapper prepares
the built binary and released Grund 0.16.1 for the co-change compatibility cases
([§FS-cochange-recipe.examples](../functional-spec/FS-cochange-recipe.md#examples-maintained-walkthrough-tests-and-opt-in-guidance)) and the same-source binding oracle for the
Node parity tests ([§FS-distribution.3.0.3](../functional-spec/FS-distribution.md#303-complete-data-and-canonical-parity)), and the shared build target of the Node
acceptance, the inputs of [§AR-ci.3.4](AR-ci.md#34-the-python-gates-inputs), then runs unittest discovery of `tests/integration`
with the pattern `test_*.py` and that environment from the repository root, timing
each test ([§AR-ci.3.4.2](AR-ci.md#342-per-test-durations)). The parity test checks the shared wrapper invocation,
the four selections CI passes it, and its discovery directory and pattern.

### 3.1 Citations and links

The gate also runs `grund check --full`, including the grounding floor from [§FS-check.3.6](../functional-spec/FS-check.md#36-ungrounded-unit-opt-in), `grund fmt --write` for canonical citation links, and `lychee` for Markdown links. Running both in CI preserves the boundary [§FS-non-goals.1](../functional-spec/FS-non-goals.md#1-markdown-link-validation) draws: `grund` owns ID citations across docs and source, `lychee` regular Markdown links and URLs.

Links are checked in two places, through one wrapper, `scripts/check_links.py`.
The **gate** is the `lychee` hook, so it runs on every pull request, in every
merge-queue group and in the local `pre-commit run --all-files`. It makes two
passes. The **internal pass** checks every internal link of the whole tree
without the network: relative paths, fragments, and the same-repository links
below. The **changed-line pass** sends to the network only the external URLs on
lines the change adds or modifies ([§AR-ci.3.1.4](AR-ci.md#314-the-gate-checks-an-external-url-only-where-the-change-touches-it)), so a change that touches no
external URL makes no network request at all, and an outside host that is down
fails no change that leaves its links alone. The **sweep** is a scheduled job on
`main` that sends every external URL of the tree to the network
([§AR-ci.3.1.5](AR-ci.md#315-a-scheduled-sweep-checks-every-external-url)), through the same wrapper's `--all`, which is also how anyone
runs the full pass by hand. Both passes and the sweep read the same inputs: the
paths the hook names.

A canonical `https://github.com/agent-grounds/grund/blob/main/<path>` link in
the checked tree is a same-repository link, and so is the `tree/main/<path>`
form. The internal pass maps it to `<path>` in the current checkout and asks
`lychee` to validate that local file and any fragment. Only the exact URLs that
pass this local check are excluded from the network; missing paths, bad
fragments, malformed self URLs fail the internal pass, and links to every other
repository or host are external. This makes a durable default-branch URL
checkable on the branch that first adds its target without accepting a 404 or
pinning a shipped address to that branch. The internal pass runs before any
network request, on the gate and on the sweep alike, and a failure in it stops
the run before one is made.

`--full` ([§FS-check.1.3](../functional-spec/FS-check.md#13-the-full-tree-scope---full)) is here because a citation in a file outside `grund`'s own `[scan] include` and every kind home ([§FS-config.3.5](../functional-spec/FS-config.md#35-scan--what-gets-scanned)) is invisible to the plain run rather than merely unchecked — the drift this repository asks its users to guard against is one it can suffer too. The flag is purely additive, so the gate still asserts everything it asserted before.

#### 3.1.1 Bounded external-link tolerance

The repository's `scripts/check_links.py` wrapper and `lychee.toml` serve both
local pre-commit and Ubuntu CI, realizing
[§FS-non-goals.1.1](../functional-spec/FS-non-goals.md#11-this-repositorys-external-link-gate).
Every network pass, the gate's changed-line pass and the sweep alike, uses lychee 0.23.0 with explicit `timeout = 30`
(seconds for the whole request), `max_retries = 4` (after the initial attempt),
and `retry_wait_time = 1` (seconds for the exponential-backoff base).
Five attempts have nominal waits of 1, 2, 4, and 8 seconds, totalling 15 seconds.
A persistent whole-request timeout therefore consumes a nominal 165 seconds
(5 × 30 + 15), plus overhead and possible host-specific pacing, in each of the two
passes [§AR-ci.3.1.3](AR-ci.md#313-a-transient-failure-from-any-host-is-checked-once-more)
allows before a nonzero exit. HTTP 404 responses
remain failures, and unresolved timeouts are never accepted as successes.

The pinned binary retains a separate fixed 10-second connection timeout; the
30-second setting does not raise that ceiling. A persistent connection timeout
therefore consumes a nominal 65 seconds per pass (5 × 10 + 15): five attempts at 10 seconds
plus 15 seconds of backoff, plus overhead and possible host-specific pacing.
Both timeout phases enter
lychee's shared retry loop, so the extra retry offers either phase another
recovery opportunity. This does not identify which phase stalled at a public
host or guarantee immunity to arbitrary outages.

Real lychee tests against loopback HTTP must obtain a successful response after
a 21-second response-header delay, and after four retryable timeout failures.
They also retain permanent-timeout and HTTP 404 rejection, valid local
fragments, and invalid self-links stopping before any network request. Timing
may be scaled in supplementary fixtures only with an assertion of the exact
production policy and evidence exercising the unscaled policy. A subprocess
watchdog exceeds the selected finite budget; expiration is an infrastructure
failure, not evidence that lychee rejected a persistent link. The wrapper's
routing, exact self-link exclusions, hook invocation, checked documents, and
lychee version stay as described in [§AR-ci.3.1](AR-ci.md#31-citations-and-links), apart from the per-URL re-checks of
[§AR-ci.3.1.2](AR-ci.md#312-a-github-5xx-is-settled-through-the-api) and [§AR-ci.3.1.3](AR-ci.md#313-a-transient-failure-from-any-host-is-checked-once-more); no outer whole-tree retry is added.

#### 3.1.2 A GitHub 5xx is settled through the API

GitHub's HTML views fail to serve for long stretches while the resource behind
them exists: the blob pages of a very large repository answer 503 for hours, and
pull request and issue pages answer scattered 500s under load. Lychee's retries
cannot outlast that, so the wrapper asks GitHub's API instead, which is the
"successful answer about the same resource" of
[§FS-non-goals.1.1](../functional-spec/FS-non-goals.md#11-this-repositorys-external-link-gate).

The network pass writes lychee's JSON report (`--format json`, to a file through
`--output`). In lychee 0.23.0 the report's `error_map` lists, per input file,
each failing `url` with a `status` whose `code` is the HTTP status and whose
`text` is lychee's reason; a timeout carries no `code`, and the report carries
no line numbers. Lychee then prints nothing of its own, so the wrapper prints
every failure it keeps with its URL, source file, status and lychee's reason:
`text`, followed by the report's `details` when `text` does not already say it.

A failure is a **candidate** when its `code` is 500 to 599 and its host is
exactly `github.com`. No other failure is a candidate, and none is re-checked
through the API; [§AR-ci.3.1.3](AR-ci.md#313-a-transient-failure-from-any-host-is-checked-once-more) says which of them lychee checks a second time.
A candidate re-checks through the endpoint its URL shape names, a fragment
dropped, whether the changed-line pass or the sweep met it:

| URL on `github.com`             | `gh api` endpoint                        |
|---------------------------------|------------------------------------------|
| `/{owner}/{repo}/pull/{n}`        | `repos/{owner}/{repo}/pulls/{n}`           |
| `/{owner}/{repo}/issues/{n}`      | `repos/{owner}/{repo}/issues/{n}`          |
| `/{owner}/{repo}/blob/{ref}/{path}` | `repos/{owner}/{repo}/contents/{path}?ref={ref}` |

`{ref}` is the one segment after `blob`. The wrapper runs `gh api <endpoint>`
through the same runner it runs lychee with, once per candidate and without a
retry, and `gh` authenticates as it always does (`GH_TOKEN`, `GITHUB_TOKEN`, or
`gh auth login`). Each candidate ends in one of three outcomes:

- **confirmed** — `gh` exits 0. The link passes.
- **broken** — the API answers 404. The link fails, named as broken.
- **unsettled** — anything else: an API 5xx or other error, `gh` missing or
  unauthenticated, or a URL shape the table does not name. The link fails, as
  it does today.

The hook passes only when the network pass failed, its report was read, and every
failure in it is a confirmed candidate or passed the re-check of [§AR-ci.3.1.3](AR-ci.md#313-a-transient-failure-from-any-host-is-checked-once-more).
A missing or unreadable report keeps
lychee's exit code. Whenever there is a candidate, one line reports the re-check,
`N GitHub 5xx re-checked through the API: x confirmed, y broken, z unsettled`,
and each broken or unsettled candidate is named with its URL, source file and
status.

Nothing passes for lack of evidence, so the local hook and CI can now differ in
one direction only: a machine without `gh` or its authentication fails a 5xx
that CI confirms, never the reverse, and
[§AR-ci.1](AR-ci.md#1-pre-commit-is-the-source-of-truth) holds for every tree CI
passes. CI gives the pre-commit step the workflow's token as `GH_TOKEN`, which
`gh` (preinstalled on the Ubuntu runner) reads first; reading a public resource
needs nothing more, so the re-check works on a pull request from a fork too.

`tests/integration/test_check_links.py` holds the wrapper to each outcome
against a stubbed lychee report and a stubbed `gh`.

#### 3.1.3 A transient failure from any host is checked once more

Lychee 0.23.0 does not retry every failure its retry loop could cure: it sends a
request that is answered with a rejected status, a 5xx or a 403 among them,
exactly once, and so it does with a network error such as an HTTP/2 protocol
error or a malformed response. A host that answers one such failure and then
200 fails the network pass, and
[§FS-non-goals.1.1](../functional-spec/FS-non-goals.md#11-this-repositorys-external-link-gate) already counts a later
successful attempt as passing. So after the network pass, and after the API
re-check of [§AR-ci.3.1.2](AR-ci.md#312-a-github-5xx-is-settled-through-the-api), the wrapper checks each remaining failure of a
**transient** shape once more:

- a 5xx from any host other than exactly `github.com`, whose 5xx stays with the
  API re-check alone;
- a 403, 408 or 429 from any host;
- a timeout, or a failure without a code, on an `http` or `https` URL.

Everything else keeps its verdict and is never checked again: a 404, a 410 and
every other 4xx, a failure on any other scheme, a local file, and a missing
fragment, which lychee reports without a code as `Cannot find fragment`. A
transient URL that failed in several files is checked once and settles every
one of them.

The re-check is one lychee invocation over just those URLs, after a pause of
`RECHECK_PAUSE` seconds, 5 in the wrapper, so a server that shed load has a
moment to recover. It runs the same pinned lychee from the same directory, so
it reads the same `lychee.toml`, with the network pass's `--include-fragments`
and its JSON report written through `--output`, and the wrapper reads that
report as it reads the first. A URL the re-check does not list as failing
passes. A URL it does list, and a re-check whose report is missing or
unreadable, fails as it did, named with its URL, source file, the re-check's
status and lychee's reason. Whenever a URL is re-checked, one line reports it,
`N transient failures re-checked with lychee: x passed, y failed`, where `N`
counts URLs.

The re-check follows whichever network pass met the failure, the gate's
changed-line pass or the sweep. This is a re-check of named URLs, not the whole-tree retry
[§AR-ci.3.1.1](AR-ci.md#311-bounded-external-link-tolerance) rules out: the tree is not walked again, and a URL that passed the
network pass is not requested again. Nothing passes without a successful
response from the same URL, so a persistent timeout and a 404 still fail. A
persistent connection timeout spends the per-pass budget of
[§AR-ci.3.1.1](AR-ci.md#311-bounded-external-link-tolerance) twice, a nominal 135 seconds (2 × 65 + 5)
with the pause; a persistent whole-request timeout spends a nominal 335 seconds
(2 × 165 + 5). Both totals exclude overhead and possible host-specific pacing.

The real-lychee loopback tests of `tests/integration/test_check_links_network.py`
hold the re-check to a 503, a 502, a 403 and a malformed response once followed
by 200 passing, and to a persistent 503 failing with the report line and a
persistent 404 failing without a second request. Their watchdog exceeds the two
budgets. The stubbed tests of `tests/integration/test_check_links_recheck.py` hold the
classification above.

#### 3.1.4 The gate checks an external URL only where the change touches it

The changed-line pass judges the working tree against one **diff base**, taken
from one place: the commit `LINK_BASE` names when that variable is set and
non-empty, otherwise the merge base of `HEAD` with `origin/main`; a set
`LINK_BASE` that names no commit falls back to nothing. A line is
**changed** when it is added or modified between that base and the working
tree - committed, staged or not yet staged - and every line of a file that is
new since the base counts, an untracked file that git does not ignore among
them. A deleted line checks nothing. The pass collects the external URLs on
changed lines of the files under the hook's inputs, and only those, each once,
go to the network, with the tolerance of [§AR-ci.3.1.1](AR-ci.md#311-bounded-external-link-tolerance) and the re-checks of
[§AR-ci.3.1.2](AR-ci.md#312-a-github-5xx-is-settled-through-the-api) and [§AR-ci.3.1.3](AR-ci.md#313-a-transient-failure-from-any-host-is-checked-once-more). A URL on an unchanged line is not requested,
even in a file the change edits elsewhere.

When the variable names no commit, or it is empty and the merge base does not
resolve - no `origin/main`, a shallow clone that lacks the base, a directory
outside git - the gate checks no
external URL rather than all of them, still runs the internal pass, and says so
on one line beginning `no diff base:`. That is the right failure direction for
a gate: the sweep checks every external URL whatever the gate skipped.

This keeps the verdict parity of [§AR-ci.1.1](AR-ci.md#11-stages-without-a-file-list), because every invocation reads its
base by the same rule. A local run on a branch measures from the merge base
with `origin/main`, as a pull request does. CI's pre-commit job sets
`LINK_BASE` to `github.event.pull_request.base.sha` on a pull request and to
`github.event.merge_group.base_sha` in a merge-queue group, the bases the
`commit-messages` job reads ([§AR-ci.8.2](AR-ci.md#82-the-scanned-range)), and checks out with `fetch-depth: 0`
so that commit is present. A push to `main` sets nothing and measures from the
merge base of `HEAD` with `origin/main`, which is `HEAD` itself: it checks no
external URL, and the sweep covers `main`.

A dispatched run ([§AR-ci.11](AR-ci.md#11-a-release-candidate-is-dispatched-the-same-ci)) has no event base, and on a fork's runner `origin`
is the fork, whose `main` may lag upstream by weeks: measured from it, every
upstream edit since would count as changed. So before the hook runs, the job
asks the forge for the upstream repository - the repository itself, or its
parent when it is a fork - and `scripts/dispatch_link_base.py <upstream-url>`,
run in the checkout, fetches that repository's `main` and prints the merge base
of `HEAD` with it, which the job hands the hook as `LINK_BASE`. A dispatch on a
fork thus measures what a pull request from its branch would, and one on the
upstream measures what a push does. When that base cannot be resolved - the
upstream unreachable, a history without the merge base - the script prints
`none` and says why on stderr, and a `LINK_BASE` that is set but names no
commit is no diff base: the gate checks no external URL and prints its
`no diff base:` line rather than falling back to `origin/main`.

`tests/integration/test_check_links_scope.py` runs the wrapper with the real
lychee against a git fixture and a loopback host that counts every request: an
unchanged dead URL is never requested, an added or edited one fails the gate, a
broken internal link in an untouched file fails it, and the variable and the
merge-base fallback reach the same verdict on one fixture. `tests/integration/test_check_links_dispatch_base.py`
builds an upstream, a fork whose `main` is stale and a runner checked out from
the fork, and holds a dispatch to the URL its change added, an unresolvable
upstream to no request, and `ci.yml` to wiring the dispatch base.

#### 3.1.5 A scheduled sweep checks every external URL

`.github/workflows/links-nightly.yml` runs on a `schedule:` once a day and on
`workflow_dispatch:`, on `main`'s head, and runs
`scripts/check_links.py --all` with the hook's own inputs. `--all` is the
internal pass followed by one network pass over every external URL of the
tree, with the tolerance of [§AR-ci.3.1.1](AR-ci.md#311-bounded-external-link-tolerance) and both re-checks, the pass the gate
made before [§AR-ci.3.1.4](AR-ci.md#314-the-gate-checks-an-external-url-only-where-the-change-touches-it). It is never a check of a pull request or a
merge-queue group, the precedent [§AR-ci.5.4](AR-ci.md#54-the-nightly-tier) sets for the benchmarks: a link
that goes dead is fixed by a change of its own, and no change that leaves it
alone waits for that.

With `--report <path>`, `--all` writes a Markdown report of every failure it
keeps, one line each, `- <url> - <file>:<line> - <status>`, naming every
line that carries the URL in every input file. The workflow, with
`permissions: issues: write`, files that report when the sweep fails: if an open
issue titled `Nightly link sweep: external links failing` exists, the report is
added to it as a comment; otherwise one is opened with that title and the label
`bug`. One open issue is the record of what is dead, so a second run never opens
a second issue. A clean tree passes the sweep and files nothing.

`tests/integration/test_check_links_sweep.py` holds `--all` and its report
against a loopback host, and the workflow's triggers, command, permissions and
title against this section.

### 3.2 The managed block's text

Last of the `grund` hooks, and immediately after `grund fmt --write` because that hook rewrites the Markdown this one measures, comes `grund init --check` ([§FS-init.4](../functional-spec/FS-init.md#4-exit-codes)). Stale text anywhere in a managed block already fails the `grund check --full` hook that runs before it, because `check` compares the whole block ([§FS-check.3.5.4](../functional-spec/FS-check.md#354-no-part-of-the-block-is-exempt-from-the-comparison)). What this hook still catches is a file `init` would create that `check` does not require, such as an agent-directory companion ([§FS-check.3.5.1](../functional-spec/FS-check.md#351-which-entrypoints-are-verified)), and the workspace sections in a tree where `check` leaves them out ([§FS-check.3.5.5](../functional-spec/FS-check.md#355-the-workspace-sections-are-judged-at-the-outermost-root-of-a-full-checkout)). The hook writes nothing and fails only on a path `init` would have written; the fix is the same `init` run without the flag. This repository runs on its own users' behalf here — [§REQ-agents-md.2](../requirements/REQ-agents-md.md#2-the-managed-block-stays-current) requires its own managed block to stay current, which `grund check` enforces by its version and its text.

### 3.3 Test-only observers

Some tests count what the binary did rather than read what it printed, and the binary counts only when a test-only feature compiles the counter in. `test-workspace-load-count` is the one today: the workspace-load observer of [§AR-resolver.3.1](AR-resolver.md#31-grund-show---batch-loads-once), reached from the `grund` package as `grund/test-workspace-load-count`, and absent from every other build for the reason [§AR-ci.5.3](AR-ci.md#53-the-bench-feature) keeps the `bench` harness out of the regular matrix. The gate's test command enables every such feature, so the gate observes everything. A plain `cargo test` does not, and it is the run a contributor without the hooks types; its reader is the one who must not be sent the wrong way ([§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible)). Without the feature the observer's log is never written, so a count read from it is an observed zero rather than an absent observation: a test expecting one load fails naming a file, a line and two integers, which sends the search into the code under test instead of the command line, and a test expecting none passes having observed nothing.

Stated so a test can hold a test file to it: **a test that reads an observation compiled only under a feature is ignored in a build without that feature, and its ignore reason names the feature as the gate passes it.** It never asserts on the observation there, so a default run neither fails nor passes on something it could not have seen; the same case's other assertions — exit code, stdout, stderr — sit in a test that runs in every build. `tests/integration/test_feature_gated_observers.py` holds each test file that reads an observer to this, and holds the gate's test command to naming every feature such a file needs, because the gate itself, running with every feature on, cannot see a test that breaks the rule.

#### 3.3.1 An observation is read from a process the test's own build made

That rule is keyed to how the test was built, and a test built with the feature can still spawn a binary built without it. `env!("CARGO_BIN_EXE_<name>")` names the uplifted `<target>/<profile>/<name>`, one path per target, and every later build of the package into that target re-links it, whatever its features. Cargo releases the target's lock before it runs the test binaries, so such a build can land anywhere in the test phase; the Python gate's `GRUND_BUILT` ([§AR-ci.3.4](AR-ci.md#34-the-python-gates-inputs)) is one. A featured test that spawns the path after it reads the observed zero [§AR-ci.3.3](AR-ci.md#33-test-only-observers) names, in the very run that enabled the feature.

So **a test that reads an observation compiled only under a feature reads it from a process its own build made, never from an uplifted artifact another build of the package can re-link.** For the CLI that process is the test executable, re-spawned into the CLI's entry; a run that observes nothing keeps `CARGO_BIN_EXE_<name>`, whose output any build of the binary gives. `tests/integration/test_feature_gated_observers.py` holds each observer's test file, with the files it declares as modules, to this: no fn there reaches both the observer's variable and `CARGO_BIN_EXE_`, in its own body or through a helper it calls, so observed and unobserved runs are made by different fns, and some fn there still names the variable.

### 3.4 The Python gate's inputs

The wrapper hands its discovery run four inputs, each an absolute path in the
environment, and treats the four alike: a variable already set is used as
given, and an unset one is prepared from this checkout. `GRUND_BUILT` is the
tree's own `grund`, built by `cargo build -p grund --locked`. `GRUND_RELEASED`
is released Grund 0.16.1, installed once into the wrapper's scratch and held to
its `--version`. `GRUND_BINDINGS_ORACLE` is the same-source Rust oracle the Node
adapter is compared against ([§FS-distribution.3.0.3](../functional-spec/FS-distribution.md#303-complete-data-and-canonical-parity)): the `grund-binding-oracle`
example of `grund-core`, built with `GRUND_BINDINGS_SOURCE_SHA` set to
`git rev-parse HEAD`, so that its `--metadata` reply names the commit under test
and protocol version 1. That is how the workflow's own step builds it. The
wrapper builds it with the plain `cargo` its other build uses, because the
workflow's toolchain pin is CI's, and a local gate must not fail for want of it.
It prepares the oracle on every run that is not given one, and the build is
incremental, so a run after a new commit meets an oracle of that commit rather
than a stale one.

Preparing an input never turns into skipping the tests that need it. A build
that fails, or an executable missing where its build put it, fails the gate:
[§FS-cochange-recipe.examples](../functional-spec/FS-cochange-recipe.md#examples-maintained-walkthrough-tests-and-opt-in-guidance) asks this of the compatibility run, and the Node
parity tests ask the same of theirs. The workflow sets the oracle before the
wrapper runs, as [§AR-ci.1](AR-ci.md#1-pre-commit-is-the-source-of-truth) lets it install a hook's prerequisites first, and the
wrapper then uses that one. Nothing a local run lacks may stand between it and
the verdict CI reaches on the same tree.

`tests/integration/test_python_gate_inputs.py` holds the wrapper to this. With
the oracle unset and no CI in its environment, the oracle the wrapper returns
must answer `--metadata` with this checkout's `HEAD` and protocol version 1. An
oracle it is given must come back as that same file's absolute path.

#### 3.4.1 The Node acceptance's shared build target

`GRUND_NODE_TARGET` is the fourth input: the root of the Cargo target that every
build of the Node acceptance ([§AR-bindings.5](AR-bindings.md#5-grund-node-the-napi-rs-binding)) shares. Unset, the wrapper prepares
`~/ag/tmp/grund-node-target`, outside the checkout and kept between runs, and CI
restores that directory from a cache step of its own, so a job starts with the
dependency tree already compiled. The acceptance's own helper reads the
same variable and falls back to the same directory, so `tests/bindings/node/run.py`
run on its own shares it too. What the root holds is one target per source
checkout ([§AR-ci.3.4.1.2](AR-ci.md#3412-one-target-per-source-checkout)), which the helper derives under it, and every build
site names that checkout's target or a directory under it, never a fresh
temporary one: the source package and its bundled fallback rebuild, the
documented native builder and its test-seams build, and the CLI. The CLI the
acceptance compares against is built once per run and reused by every test
that needs it.

That cache is not the cargo cache's. Its key is
`<os>-node-target-v3-1.95.0-<manifest and lock hash>`: the runner's OS, a version to
bump when what it holds must be thrown away, the binding toolchain, and the
workspace manifest with the lock file ([§AR-ci.13.4.1](AR-ci.md#1341-a-key-moves-with-the-profile)), with the same key less its
hash as the restore prefix. A key of its own is
what makes it save at all: an entry is never overwritten, so a save to a primary
key that already has one saves nothing, and the cargo cache's key already hits
whenever neither the manifest nor `Cargo.lock` has changed, so a path added to it would never be written. It holds the release
binding's and the dev builds' compiled dependencies and no incremental state;
`v2` is the bump that came with the dev builds of [§AR-ci.3.4.1.1](AR-ci.md#3411-one-release-build-and-every-build-on-one-line), whose
dependencies an entry saved before them does not hold and, on an exact hit,
would never save. `v3` is the bump that came with the per-checkout targets of
[§AR-ci.3.4.1.2](AR-ci.md#3412-one-target-per-source-checkout): a `v2` entry holds its artifacts at the root rather than in the
checkout's directory under it, and an exact hit skips the save, so the
per-checkout target would never be cached.
It cannot leave that state out, because a cache archives a listed
directory whole, so the builds never write it: the CLI builds with
`CARGO_INCREMENTAL=0`, and so do the dev builds of the binding, the release
build is not incremental, and a fresh
checkout's workspace crates could not reuse the state anyway. After one run on
Linux the target is about 0.7 GB before compression, three caches in all,
against a repository limit of 10 GB the other caches already crowd; the
tarball's own crates are not worth caching, because they rebuild at their fresh
paths every run anyway.

Sharing a target changes what is cached, not what is proven. The builds keep
`--locked`, the bundled-sources closure, and the fallback
rebuild after the native artifacts are removed from the installed tarball, so
[§FS-distribution.3.2.3.3](../functional-spec/FS-distribution.md#3233-source-buildlocal-package-ownership) holds as written: the tarball's own workspace crates
still compile from its own sources, at their own paths, and generated files
still stay outside the checkout. What a warm target no longer repeats is the
third-party dependency tree, which that point already lets a source build take
from a populated cache.

`tests/integration/test_node_shared_target.py` holds this: with the variable set,
every `--target-dir` the source package and the CLI build name lies under it and
the CLI is built once; every other build site names the helper's target, the
rehearsal's, or a directory under one of them; unset, the wrapper prepares one
persistent root outside the checkout, the helper's target lies under it, and a
CI cache step of its own restores that root, under a key that is not the cargo
cache's.

##### 3.4.1.1 One release build, and every build on one line

A warm target saves only the dependency tree, and under the release profile
that is the cheap part: `lto = true` and one codegen unit make the optimisation
and link of the tarball's own cdylib about 48 s of a 50 s build on a
developer's machine, and two to four times that on a runner, with or without a
warm target. So the acceptance pays for that profile once. Exactly one build of
the binding uses `--profile release`: the documented native builder command of
[§FS-distribution.3.2.3.3](../functional-spec/FS-distribution.md#3233-source-buildlocal-package-ownership), in `tests/e2e/test_node_binding.py`, which is the
acceptance's proof that release is ordinary LTO. Every other build of the
binding passes `--profile dev`: the test-seams build, the source package's
`npm run build:source`, and the bundled-fallback rebuild in an unpacked
tarball, the last two through `build:source`'s pass-through to the builder.
What each of those proves does not depend on the optimiser: the test-seams build
proves the runtime seams, worker panics, the writer lease and teardown, and the
source package and its fallback prove the bundled `--locked` source closure, its
ESM, CJS and TypeScript surface, and that the fallback compiles from the
tarball's own sources without reaching the checkout. [§FS-distribution.3.2.3.3](../functional-spec/FS-distribution.md#3233-source-buildlocal-package-ownership)
names a profile for the native builder only, so it holds as written. A build
that only refuses before Cargo runs, or reaches a probe that stands in for it,
compiles nothing and is not counted.

The Node acceptance launcher gives a release build a finite wall-clock deadline
of 1800 seconds automatically; development builds and commands that build nothing
keep 900 seconds. An explicit caller-supplied timeout takes precedence. Both
`checked()` and direct `command()` calls use the same policy, derived from the
launcher's existing build-profile classification: direct `build.mjs` and
`npm run build:source` default to release, their `--profile dev` selects dev,
and Cargo builds default to dev unless `--release` is present. Continuing output
does not reset the deadline. This is a test-harness allowance, adding at most
900 seconds to the permitted release wait, with no extra build, retry or wait
on a successful command and no claim of faster compilation. It leaves the
directional gate-cost goal of [§AR-ci.requirements.1](AR-ci.md#requirements1-every-gate-averages-at-most-fifteen-minutes--directional)
and the deferred job ceiling of [§AR-ci.requirements.2](AR-ci.md#requirements2-no-run-of-a-gate-passes-twenty-minutes--deferred) unchanged.

`tests/integration/test_node_build_deadlines.py` exercises the documented native
builder test through its real `checked()` and `command()` path, substituting only
scratch and a controlled subprocess transport with scaled time. A progressing
child that completes after the old deadline but before the release deadline
must produce the artifact that test checks; a child exceeding the release
deadline must be terminated despite its output. Focused checks cover the build
profiles, both launcher entry points, the shorter budgets and explicit overrides.
These cheap mechanism checks supplement the one real release/LTO build; their
stand-in artifact is not a native addon or evidence of Windows compiler progress.

So that CI can attribute the acceptance's time without the captured output of
cargo, every build the acceptance runs prints one line to its standard error
as it finishes: the profile, the target directory and the wall seconds, in the
form `node build: <profile> <target-dir> <seconds>s`. It is one line per build
and none for anything else the harness runs. The gate reaches the acceptance
through `tests/integration/test_node_binding_acceptance.py`, which captures its
output to show only on failure; it passes these lines, and nothing else, on to
its own standard error on every run. The gate captures that too, and prints
the lines that begin `node build: ` as [§AR-ci.requirements.5.1](AR-ci.md#requirements51-a-suite-prints-its-failures-its-check-ins-and-one-summary) has a suite
step print them, so they reach the job log of a green leg.

`tests/integration/test_node_shared_target.py` holds this too: across the
support module's source package and every other build site, exactly one build
of the binding resolves to the release profile, and it is the documented native
builder's; a build the harness runs prints its one line naming its profile,
target and seconds; the wrapper shows those lines, and only those, on a green
run, and so does the gate when it runs the wrapper as CI's `node acceptance`
step does; and the CI cache key has moved past the `v2` this point bumped, to the
`v3` of [§AR-ci.3.4.1.2](AR-ci.md#3412-one-target-per-source-checkout). `tests/integration/test_ci_output.py` holds the
gate's half on a suite of its own: each build line that a passing test, its
child process, a skipped test or a module teardown writes appears once in the
green output, as does a `gate time: ` line a child process writes, and nothing
else they write does.

##### 3.4.1.2 One target per source checkout

The root is shared, a target is not: each source checkout builds into its own
directory under `GRUND_NODE_TARGET`, named by a short digest of the checkout's
resolved root, so the same checkout gets the same directory on every run and
two worktrees of the repository never get one directory between them. The
helper derives that directory once, in `checkout_target(root)` of
`tests/bindings/support.py`, which the Python acceptance's targets go through too
([§AR-ci.3.4.5](AR-ci.md#345-the-python-acceptances-shared-build-target)), and every build site reaches it through the helper; the wrapper
and CI still prepare, cache and restore only the root.

The helper also owns target lifetime, implementing
[§FS-repository-maintenance.2.1](../functional-spec/FS-repository-maintenance.md#21-ownership-and-reuse), [§FS-repository-maintenance.2.2](../functional-spec/FS-repository-maintenance.md#22-automatic-reclamation),
[§FS-repository-maintenance.2.3](../functional-spec/FS-repository-maintenance.md#23-conservative-deletion-boundary-and-legacy-entries) and [§FS-repository-maintenance.2.4](../functional-spec/FS-repository-maintenance.md#24-concurrent-callers-and-source-lifetime). Each target
records its resolved source root in `checkout.json`, a UTF-8 JSON object with
a `checkout` string. The directory key remains the first 16 hexadecimal
characters of SHA-256 over `os.fsencode(resolved_checkout_root)`. The helper
publishes the record by replacing it atomically from a private temporary file,
and checks other direct child targets for reclaimable ownership on every call.
Record validation and a filesystem lookup establish eligibility before recursive
removal; a missing record is never reverse-mapped from its digest. The shared
`tools` directory stays outside this ownership scheme. Cleanup tolerates another
caller having already removed a dead target; it supplies no running-build lease.

Cargo's lock does not make one target safe to share across source trees,
because it serialises builds and separates nothing else. Two things in a target
are keyed by neither the source tree nor the run. Cargo gives a workspace path
package the same unit in every checkout, then judges it fresh by modification
time, so a build from one tree, after another tree's newer build, links the
other tree's `grund-core` with no concurrency at all. And each uplifted artifact,
`debug/grund` and the binding's library, is one path per target, which another
tree's build overwrites between this run's build and its copy or execution of
it. Either way the acceptance compares an addon or an oracle built from sources
other than the commit under test, which is what the same-source parity of
[§FS-distribution.3.0.3](../functional-spec/FS-distribution.md#303-complete-data-and-canonical-parity) and the independent oracle of [§FS-distribution.3.2.3.1](../functional-spec/FS-distribution.md#3231-execution)
rule out. A lock held from build through copy would cure the second and not the
first, so the remedy is the separate target.

The price is the third-party dependency tree: each checkout compiles its own
once, where worktrees used to share one. A CI runner has one checkout path, so
its directory is the same on every run, and the cached root holds it once the
key has moved to `v3`, which this point bumped: an exact hit on a `v2` entry,
whose artifacts lie at the root, would skip the save and never cache the
checkout's directory.

`tests/integration/test_node_shared_target.py` holds this too: two checkouts
under one root get disjoint targets, both under it; one checkout gets the same
target on every call and every run; every `--target-dir` a checkout's
source package and CLI build name lies under that checkout's own target; and
the CI cache key carries the `v3` this point bumped.

`tests/integration/test_node_target_lifetime.py` exercises the real helper in
disposable checkouts and an isolated shared root, without compiling or fetching
dependencies: deleted-source reclamation, ownership adoption, retained artifacts,
untrusted records, symlinks, uncertain source lookups and concurrent processes.

##### 3.4.1.3 The TypeScript compiler, a prepared input

The acceptance's strict TypeScript consumer check ([§FS-distribution.3.2.3](../functional-spec/FS-distribution.md#323-workers-loading-and-source-builds)) needs
one package that is not built from this checkout: the TypeScript compiler, at the
version the harness pins, `support.TYPESCRIPT`. It is the only registry fetch in
the whole acceptance, since the tarball the consumer installs is local, so it is
prepared like the other inputs rather than fetched by the test. Installing it into
each test's fresh consumer on every run put a live registry fetch inside the
acceptance, and a fresh CI runner has no npm cache to fall back on, so one DNS
blip on the runner failed the whole Node job.

`support.typescript_compiler()` is the one place it is prepared, and it returns
the path of the compiler's `node_modules/typescript/bin/tsc`. The compiler lives
in `tools/typescript-<version>` under the root of [§AR-ci.3.4.1](AR-ci.md#341-the-node-acceptances-shared-build-target), beside the
per-checkout targets, so every checkout and every run on a machine share one copy
and a version bump gets a directory of its own. When that directory already holds
the compiler, the helper returns it and runs nothing, so a run with the directory
present never touches the network. Otherwise it installs the pinned version
exactly, once, into a fresh temporary sibling of that directory, and renames the
sibling into place only after the install succeeds. An interrupted or failed
install therefore never leaves a directory that a later run takes as ready.

That one fetch is retried, and only for what a retry can cure. An install that
fails with a network-class npm error code, `ENOTFOUND`, `EAI_AGAIN`, `ETIMEDOUT`,
`ECONNRESET` or `ECONNREFUSED`, is tried again after a short pause, up to three
attempts in all. Any other failure, and a third network failure, fails the test
that asked, with npm's own output, and never turns into a skip
([§AR-ci.3.4](AR-ci.md#34-the-python-gates-inputs)). npm's own `fetch-retries` does not replace this: it already ran on
the CI attempt that failed.

The strict check then compiles the consumer's ESM and CommonJS files with that
compiler from the consumer's directory, under the same flags as before, so the
package's types still resolve from the consumer's own install of the tarball.
Nothing about the check is weaker. Only where the compiler comes from has changed.

CI restores the directory with a cache step of its own in every job that runs the
Node acceptance, keyed `<os>-node-typescript-<version>`, and saves it under the
rules of [§AR-ci.13.6](AR-ci.md#136-only-a-push-to-main-saves-a-cache). It does not ride on the node-target cache, because that
key moves only with the manifest and `Cargo.lock`: a `v3` entry hits exactly whenever
neither has changed, and an exact hit never saves, so a directory added under the root
would not be cached until the next lock change, and a later TypeScript bump would
go uncached again. A key that names the version saves exactly when the version
changes. A node-target entry may also hold a copy of the directory, which is the
same pinned version and harmless. A cold cache still fetches once, with the
retry, so what remains exposed to the registry is a cache miss, not every run.

`tests/integration/test_node_typescript_input.py` holds the helper to this
against a stand-in npm: a prepared directory is returned with no command run, a
cold one is installed once into a sibling and renamed, a network error is retried
up to three attempts, any other error is not retried, and no failed install
leaves the directory behind. It also holds every workflow that runs the Node
acceptance to a restore and a save of that directory under a key naming the
pinned version, shared with no other cache step.
`test_package.NodePackageTests.test_strict_typescript_check_needs_no_registry_once_prepared`
runs the strict check once and then again with the registry unreachable and an
empty npm cache, which is a CI runner meeting a DNS blip after its cache
restore. The second run must pass.

##### 3.4.1.4 One pristine installed consumer per staged source

The Node acceptance installs each run's staged local tarball once into a pristine
consumer template. Every `Rehearsal` gets a fresh private copy of that installed
consumer, including its package metadata, bundled sources and native artifact;
it never shares writable files, links back to the template or another consumer,
or adopts another source's installation merely because its version matches.
The template belongs to the staged tarball and this run, outside the checkout,
and is discarded with the run. It is not a new persistent or forge cache.

The first consumer still comes through the real local `npm install`, with
`--ignore-scripts`, `--no-audit` and `--no-fund`, and must load through bare
package resolution from its own directory. Only a successful install can
publish a reusable template; a failed install fails with the command's output
and leaves nothing a later rehearsal can mistake for a completed installation.
A cold run therefore needs no prepared template, and different staged sources
at the same package version each require their own install.

This removes repeated package-manager startup and tarball extraction, without
removing any test, assertion or fresh consumer. Tests may corrupt or remove
their private native artifact, rebuild the bundled fallback, edit bundled
sources or package metadata, and start concurrent requests without affecting
another test. The real release build, dev and test-seams builds, independent
CLI and same-source data oracle, TypeScript check and installed-source fallback
of [§AR-ci.3.4.1](AR-ci.md#341-the-node-acceptances-shared-build-target) remain required.
The source package is still staged and built anew from the actual checkout.

`NodeAcceptanceWorkflowTests.test_rehearsals_prepare_each_source_once_and_isolate_consumers`
runs the real `Rehearsal` constructor and npm transport against two small local
tarballs with the same name and version and different source receipts. It
loads each consumer with Node, mutates one consumer's installed files and
removes its native artifact, then verifies the next consumer's pristine bytes
and disjoint paths. Four rehearsals of those two sources require two installs.
This regression runs on the Node side of the existing gate selection, so the
Python jobs do not acquire its preparation cost. It supplements the unchanged
real native-package acceptance; its small tarballs do not claim native proof.
Full Windows job evidence remains required by [§AR-ci.requirements.1.verification](AR-ci.md#requirements1verification-a-cost-correction-measures-comparable-full-gates).

#### 3.4.2 Per-test durations

The wrapper times every test it runs and ends with a short table of the slowest
ten, one `<seconds>s <test id>` line each under a `Slowest tests` heading, so where
the gate's time goes is measured rather than inferred from the timestamps of
`unittest`'s dot lines. A test's time runs from the previous test's end to its
own, so the class and module fixtures run before it, `setUpClass` and
`setUpModule` and the previous class's teardown, are charged to it, and the last
test carries the run's final teardown; a cargo build in a fixture shows up
against the first test of its class rather than nowhere. A discovery that finds
no tests exits 5, as `python -m unittest` does from Python 3.12. It needs no
Python newer than the gate's own floor, which is why it does not use
`unittest --durations`, new in Python 3.12. It adds those
ten lines and no line per test, because a passing test is not worth a line of
the log. `run_python_gate.run_tests(start, environment)` is the one entry that
runs a discovery and prints the table; the wrapper calls it on `tests/integration`
and returns its exit status.
`tests/integration/test_python_gate_durations.py` runs it on a small suite of its
own.

#### 3.4.3 Selecting the binding acceptances and the slow tests

The wrapper takes at most one argument, which selects what its discovery runs.
Each of three `only-` selections runs named modules alone:
`--only-node-acceptance` the module `test_node_binding_acceptance`,
`--only-python-acceptance` the module `test_python_binding_acceptance`, and
`--only-slow` the modules `test_rules_scaling` and `test_config_schema`, the
slowest of the rest: the first builds a release binary and times grund on it,
the second builds grund and runs it over every committed configuration. `--rest`
runs every test it discovers except those modules. No argument runs everything,
and that is what the local `python-test` hook runs, so the local verdict is the
one the four selections reach together. CI runs them as three jobs
([§AR-ci.13.1](AR-ci.md#131-one-job-per-language)) because these are the longest tests of the gate by minutes: the
Python acceptance alone took five to nine minutes of a Windows run while it
shared the Python job, and the slow selection four and a half of that job's
nineteen once the acceptance had left, holding it past
[§AR-ci.requirements.1](AR-ci.md#requirements1-every-gate-averages-at-most-fifteen-minutes--directional), so it runs after the Python acceptance in that job's minutes.
Any other argument, `--without-acceptance` and `--without-node-acceptance`
among them now that neither is a selection, or two at once, exits 2 with a
usage line before any input is prepared, so a mistyped selection costs nothing
and is never silently the whole suite.

The cut is by module. A module that fails to import is still in the selection
its name puts it in, so an acceptance that no longer imports fails its own job
rather than vanishing from every one. Each selection still ends with the slowest
table and still exits 5 when it finds nothing ([§AR-ci.3.4.2](AR-ci.md#342-per-test-durations)), so an
acceptance selection that matches no test fails rather than passing on nothing
run. `run_python_gate.select(suite, selection)` is the one function that makes
the cut: given a discovered `unittest` suite and `None`, `'rest'` or an `only-`
selection, it returns a suite of the tests that selection runs.
`run_python_gate.ONLY` maps each `only-` selection to its modules, and `'rest'`
is every module none of them names, so moving a module into a selection takes it
out of the rest in the same edit. `run_python_gate.run_tests(start, environment,
selection=None)` hands the selection to the one timed discovery, so a selection is
cut where the tests are found. `tests/integration/test_python_gate_selection.py`
holds it to this without running either acceptance itself.

#### 3.4.4 The CI oracle reuses restored dependencies

`scripts/prepare_binding_oracle.py` builds into this checkout's Cargo target:
`CARGO_TARGET_DIR` when supplied, otherwise `target` under the checkout. The
existing cargo cache restores and prunes it ([§AR-ci.13.6](AR-ci.md#136-only-a-push-to-main-saves-a-cache)); no extra cache is needed.
If old exact hits lack the pinned dependencies, bump this cache's key namespace
so immutable entries cannot prevent saving them. Always build with Cargo 1.95.0
and `--locked`, setting `GRUND_BINDINGS_SOURCE_SHA` from this checkout's `HEAD`,
including a pull request's merge checkout. Cargo's toolchain, feature, profile
and compiler-flag fingerprints decide dependency compatibility; a mismatch
rebuilds, never reuses incompatible artifacts. Targets remain private to each
checkout ([§AR-ci.3.4.1.2](AR-ci.md#3412-one-target-per-source-checkout)).

A cold target builds normally. After restoring compatible dependencies with
workspace artifacts removed, preparation rebuilds the oracle without recompiling
those dependencies. A later source commit rebuilds the workspace example and
reports the new SHA and source behavior; a previously built executable never
substitutes for that build ([§FS-distribution.3.0.3](../functional-spec/FS-distribution.md#303-complete-data-and-canonical-parity)). Preparation failure still
fails the gate ([§AR-ci.3.4](AR-ci.md#34-the-python-gates-inputs)). Cache event permissions, pruning and budget remain
those of [§AR-ci.13.6](AR-ci.md#136-only-a-push-to-main-saves-a-cache), and the oracle is not an extra binding release build.

`tests/integration/test_ci_avoidable_work.py` exercises the preparation script
with real Cargo on a small workspace: cold, dependency-only restored, and
changed-source builds. A dependency build-script receipt detects recompilation,
and executing the exported oracle checks its SHA and source behavior.

#### 3.4.5 The Python acceptance's shared build target

`GRUND_PYTHON_TARGET` names the root of the Cargo targets the Python acceptance
([§AR-bindings.6.2](AR-bindings.md#62-the-python-acceptance-runs-in-the-gate)) builds into. The root is shared, a target is not: each source
checkout builds into its own directory under the root, named, owned and reclaimed
as [§AR-ci.3.4.1.2](AR-ci.md#3412-one-target-per-source-checkout) names the Node acceptance's, and for its reason, by the one helper
`checkout_target(root)` in `tests/bindings/support.py` that both bindings call.
`python_target()` returns this checkout's. Every build of the run builds there: the
install of `crates/grund-py` that `tests/integration/test_python_binding_acceptance.py`
makes, and the clean-checkout and sdist installs `tests/bindings/python/test_build.py`
makes inside `run.py`, which the module hands the root so that it derives the same
target. None builds at a fresh temporary target or at the one pip makes for an
isolated build, so a run starts with the dependency tree already compiled instead
of compiling it cold, which is what a Windows run spent most of the acceptance's
minutes on ([§AR-ci.requirements.1](AR-ci.md#requirements1-every-gate-averages-at-most-fifteen-minutes--directional)). Unset, the root is `~/ag/tmp/grund-python-target`,
outside the checkout and kept between runs, so a local run shares it too.

CI restores that root in the job that runs the Python acceptance from a cache step
of its own, under the key `<os>-python-target-v2-<manifest and lock hash>`, with the
key less its hash as the restore prefix, for the reason the Node target's cache has
a key of its own ([§AR-ci.3.4.1](AR-ci.md#341-the-node-acceptances-shared-build-target)): the cargo cache's key already hits whenever
neither the manifest nor `Cargo.lock` has changed, so a path added to it would never
be written. The same step keeps `target/release`, where the slow selection's
release build lands ([§AR-ci.3.4.3](AR-ci.md#343-selecting-the-binding-acceptances-and-the-slow-tests)), and the key moved to `v2` when the root took a
directory per checkout and the step that path. It is saved on a push to main alone
([§AR-ci.13.6](AR-ci.md#136-only-a-push-to-main-saves-a-cache)) and holds no incremental state, because the job sets
`CARGO_INCREMENTAL: 0` as the workflow does.

Sharing a root changes what is cached, not what is proven. Every build stays at
the dev profile, from a local source, and the clean checkout and the unpacked sdist
still compile the package's own crates from their own sources at their own paths,
so what [§AR-bindings.6.2](AR-bindings.md#62-the-python-acceptance-runs-in-the-gate) and [§FS-distribution.3.3.7](../functional-spec/FS-distribution.md#337-local-source-and-typing-handoff) prove of the install
holds as written; a warm target no longer repeats the third-party dependency tree.
Because those paths are fresh on every run, each would leave another copy of the
workspace crates in a kept target, so the module removes them from its target when
the run ends, for the size of the target and its cache alone.
`tests/integration/test_python_shared_target.py` holds this: with the variable set,
the module's install and the clean checkout's name a `CARGO_TARGET_DIR` under it;
unset, they name one under `~/ag/tmp/grund-python-target`; two checkouts get two
targets under one root, each the same on every call and recording its checkout;
and every job that runs the Python acceptance restores the root under a key of its
own.

## 4. Performance smoke guard

CI carries a cheap floor on [§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible) that does not depend on any benchmarking toolchain: the `grund check . --full` self-check step runs the already-built binary under a generous wall-clock `timeout` — long enough never to flake on a loaded runner, short enough to fail the build on a catastrophic regression such as an accidental quadratic walk or a second read pass over every file. It is not the budget itself (the budget is tens of milliseconds; the ceiling is tens of seconds) — it is the difference between "we'd notice eventually" and "the build is red on the commit that did it". The precise per-commit meter is the [§AR-ci.5](AR-ci.md#5-benchmark-job) benchmark job, and this timeout stays as its catastrophic backstop.

## 5. Benchmark job

Three jobs run the instruction-counting harness ([§AR-benchmarks](AR-benchmarks.md#ar-benchmarks-instruction-counting-benchmarks-for-the-hot-cli-commands)) on a pull request, a push or a merge-queue group. The **build job**, `bench-build`, builds every binary the shards measure, once: on a pull request the base commit's `grund` and the head's `grund` and `grund-lsp`, on a push or a group the head's, each with the profile `cargo bench` builds, and it uploads them as one artifact. The **shard job**, `bench`, is a matrix over `shard: [1, …, n]`, where `n` is the catalog's `shards` ([§AR-benchmarks.1.2.2](AR-benchmarks.md#122-shards)); it `needs:` the build job, downloads its binaries, builds none of them again, and runs the harness on its shard with `GRUND_BENCH_SHARD=<shard>/<n>`. The **record job**, `bench-record`, `needs:` the shards, merges their partial records into one record per revision ([§AR-benchmarks.7.1.1](AR-benchmarks.md#711-one-record-per-revision-merged-from-its-shards)) and prints the comparison of the merged base and head to the job summary; a merge that refuses fails it. On a group it merges nothing ([§AR-ci.12](AR-ci.md#12-a-merge-queue-lands-every-pull-request)). Because the harness runs `grund` under Callgrind, each shard first installs `valgrind` and `cargo install`s `iai-callgrind-runner` pinned to the same version as the `iai-callgrind` dev-dependency. It runs on one representative platform (the harness measures instruction counts, not cross-platform behavior — [§AR-ci.2](AR-ci.md#2-platform-scope)'s reasoning applies) and on every push, pull request and merge-queue group, so every commit is measured before it lands, and the numbers are recorded per commit.

Everything the count depends on is pinned in these jobs, and only in them, so two counts recorded months apart were taken in one named environment. The build job and every shard carry the same container image and the same job `env:`, because a binary built under one pin and measured under another is a count of neither; Valgrind and the runner are installed in the shards, which run them:

- it runs in `container: rust:<release>-bookworm@sha256:<digest>`, an image named by its digest, which fixes the Rust compiler and the C library together;
- it runs that container with `--security-opt seccomp=unconfined`, because iai-callgrind starts Valgrind under `setarch -R` to turn ASLR off and Docker's default seccomp profile refuses that call, so every Callgrind run would exit 1; the option is scoped to this one job's container;
- it installs Valgrind as `valgrind=<exact Debian version>` and Valgrind's `libc6-dbg` at exactly the image's `libc6` version, both from `snapshot.debian.org` at one pinned timestamp (`20261001T000000Z`) that is the job's only package source: `libc6-dbg` must equal the image's C library, and bookworm's live archive keeps only its newest point release, so installing from it would break the required job at the next glibc update. The snapshot never moves, so the install keeps working and the C library still cannot float. If the snapshot is unreachable the step fails; rerun it. The timestamp moves only with the image digest, in a change of its own;
- it installs `iai-callgrind-runner` at 0.16.1, the version of the `iai-callgrind` dev-dependency;
- it sets `LC_ALL=C.UTF-8`, `TZ=UTC`, `RAYON_NUM_THREADS=4`, `CARGO_INCREMENTAL=0` and an empty `RUSTFLAGS` for the whole job; the first three are the runtime pins the harness hands the measured process ([§AR-benchmarks.3.1](AR-benchmarks.md#31-the-measured-process-runs-under-the-runtime-pins)), the last two shape the build;
- it has no `@stable` toolchain step: the image's compiler is the one it builds with. The test jobs keep tracking stable.

The cargo cache key of each of these jobs carries the image digest, so a `target/` another compiler built is never restored into it. What both benchmark workflows repeat is written once, as a composite action under `.github/actions/`: `bench-cache` restores, or prunes and saves, the build jobs' registry and `target/`, and `bench-shard` installs a shard's pins and unpacks the build job's binaries. A shard measures the build job's binaries and builds nothing of its own, and that is checked rather than assumed: the build job records the SHA-256 of the head `grund` it built, and each shard, after its runs, fails naming both digests when its `target/release/grund` differs from it. The tests that hold this section read each one in place of the step that uses it, so its pins are checked as the job's own text. Every identity field the run record carries ([§AR-benchmarks.7](AR-benchmarks.md#7-the-run-record-and-its-series)) is read from this environment.

### 5.1 Pull requests and pushes

On pull requests, base and head are measured by the pull request's own harness on one fixture bundle ([§AR-benchmarks.5](AR-benchmarks.md#5-comparing-two-revisions)); only the base *binary* comes from the base branch. The build job builds the base commit's `grund` binary, and nothing else, with the profile `cargo bench` builds, and keeps it outside `target/`. Each shard, on its own rows, runs `cargo bench -p grund --features bench --locked --quiet --bench instructions -- --save-baseline=main --save-summary=json` with `GRUND_BENCH_BIN` naming the base binary, then the same harness on the pull request's own binary with `-- --baseline=main --save-summary=json`. Each run writes a partial run record, and the record job prints the comparison of the merged two. A scenario the base binary refuses is not a failure: it is reported as `no base, not compared`, the head run still measures it, and only a scenario both binaries ran can fail the job on its relative limit ([§AR-benchmarks.5.1](AR-benchmarks.md#51-a-scenario-the-base-binary-refuses)); a scenario with no base is still held to its ceiling ([§AR-benchmarks.5.2](AR-benchmarks.md#52-a-scenario-with-no-base)).

On pushes to `main`, each shard runs its rows twice on the same binary, writes a partial record for each, and fails when a scenario's two counts differ by more than the same-binary bound of [§AR-benchmarks.7.3](AR-benchmarks.md#73-the-same-binary-bound). If that check ever fails on an unchanged binary, it is reported as a flaky test, with the labels `bug` and `flaky-test`. A push compares against nothing else. Once its run has completed, the store of [§AR-ci.5.5](AR-ci.md#55-the-store-keeps-a-pushs-record) adds the push's merged record to the `benchmark-data` branch, and nothing reads it ([§AR-benchmarks.7.5](AR-benchmarks.md#75-a-push-to-main-keeps-its-record)). On a merge-queue group ([§AR-ci.12](AR-ci.md#12-a-merge-queue-lands-every-pull-request)) each shard runs its rows once on the head's binary, held only to the absolute ceilings of [§AR-ci.5.2](AR-ci.md#52-regression-limits): it measures no base, because the pull request already compared, and writes no record, because the push the queue makes to `main` records.

### 5.2 Regression limits

The harness limits are 5% `Ir` for the generated 10k-file fixture and 12.5% `Ir` for the other hot commands, which scan the smaller generated fixtures rather than this repository's own tree ([§AR-benchmarks.1.1](AR-benchmarks.md#11-generated-fixtures-never-this-repository)). They are set per scenario in the catalog, not on the command line: each row of `crates/grund-cli/benches/scenarios.json` ([§AR-benchmarks.1.2](AR-benchmarks.md#12-the-benched-commands)) carries its `limit_pct`, and the harness gives that row's benchmark a Callgrind `soft_limits` of that much `Ir`. `check_large_10k` has 5%, and every other row has 12.5%. The PR rerun compares the PR commit against the `main` baseline under those limits, and growth past one makes `cargo bench` exit non-zero, so the bench job fails, and the `cargo test (ubuntu-latest)` required check with it ([§AR-ci.13.2](AR-ci.md#132-the-required-check-is-an-aggregate)). That is the gate that holds the [§GOAL-fast-feedback.1](../goals.md#1-performance-targets) targets.

Every limit stays at least five times the same-binary bound of [§AR-benchmarks.7.3](AR-benchmarks.md#73-the-same-binary-bound), so a limit is never close enough to the jitter for an unchanged binary to cross it.

A relative limit holds one step, not a level: a run of 12% regressions, each under the limit, passes every pull request. So each row also carries an absolute **ceiling**, `ceiling_ir`, and the harness gives the row's benchmark a Callgrind `hard_limits` of that many `Ir`. A count over its ceiling makes `cargo bench` exit non-zero and fails the job just as a relative limit does, and it needs no base ([§AR-benchmarks.5.2](AR-benchmarks.md#52-a-scenario-with-no-base)). Five rules govern the ceilings:

- A ceiling belongs to CI's series. Counts are comparable only within one series ([§AR-benchmarks.7.4](AR-benchmarks.md#74-a-local-run-is-a-series-of-its-own)), so a ceiling is set from a count the bench job recorded. The row names that count in `ceiling_from`, as its `ir` and the `commit` it was measured at. A ceiling holds a row's own count, which depends only on that row's fixture and the binary, so the tier whose series recorded it does not matter to it: the nightly tier holds a `pr` row to the same ceiling.
- A ceiling sits at least one relative limit above the count it was set from, `ceiling_ir ≥ ceiling_from.ir × (1 + limit_pct / 100)`. That headroom is far more than five times the same-binary bound, so jitter alone never crosses a ceiling, and a single change the relative limit admits never meets the ceiling at once. A ceiling is set two relative limits above that count, rounded up to a whole million `Ir`: `ceiling_ir = ⌈ceiling_from.ir × (1 + 2 × limit_pct / 100) / 10⁶⌉ × 10⁶`. One regression the relative limit admits may land under it, and a second may not compound on the first.
- The harness applies the ceilings only when `GRUND_BENCH_CEILINGS` is set, and the bench job sets it for every run. A local `cargo bench` is a series of its own whose counts may differ from CI's, so an unchanged local run does not fail against a CI ceiling. The `docker run` of `docs/benchmarks.md` sets the variable to reproduce the gate.
- The ceilings hold only the harness's own binary. Under `GRUND_BENCH_BIN` ([§AR-benchmarks.5](AR-benchmarks.md#5-comparing-two-revisions)) the harness sets no `hard_limits`, so saving the base binary's baseline never fails for the base's own counts.
- A new row takes its ceiling in the change that adds it, set from that change's own CI count. A `nightly` row is the one exception, because a pull request runs no `nightly` row and the nightly workflow cannot be dispatched before it is on the default branch: the change that adds the row sets its ceiling from a local count, and the row's ceiling is reset from the first nightly CI record, by a change of its own. A change that adds the nightly workflow or `nightly` rows dispatches the nightly workflow right after it merges, so that record exists.

A shard measures in the `target/release` the build job packed, and restores no `target/` of its own, so no `target/iai` from another commit reaches it: under limits, saving the baseline or recording a push would otherwise compare against those stale counts and could fail a step that only records. A limit or a ceiling is raised only by a change of its own that says why, never to let one pull request through. Raising a ceiling records the new CI count it was set from in `ceiling_from`.

### 5.3 The `bench` feature

The harness body is gated behind the `bench` Cargo feature, so the regular build and test matrix compiles only a no-op bench target and stays free of the Valgrind dependency — only this job needs it; that gate is also why [§AR-ci.1](AR-ci.md#1-pre-commit-is-the-source-of-truth)'s "CI installs hook prerequisites" pattern is followed here as a dedicated job rather than folded into the test matrix.

### 5.4 The nightly tier

`.github/workflows/bench-nightly.yml` runs the `nightly` tier of [§AR-benchmarks.1.2.1](AR-benchmarks.md#121-two-tiers): every row of both tiers, with `GRUND_BENCH_TIER=nightly`, on `main`'s head, in the pinned environment of section 5, against the absolute ceilings alone, since there is no base to compare with. It may split the rows into shards the way the pull-request tier does, and it writes one run record by the same merge ([§AR-benchmarks.7.1.1](AR-benchmarks.md#711-one-record-per-revision-merged-from-its-shards)), with `tier: nightly`. It runs on a `schedule:` once a day, on `workflow_dispatch:`, and on `workflow_call:`, and `pre-release-checks.yml` calls it, so the whole catalog has passed its ceilings before a release ships. It is no required check of a pull request: a ceiling a nightly row crosses is fixed by a change of its own.

### 5.5 The store keeps a push's record

`.github/workflows/bench-store.yml` keeps the record of each push to `main` on the `benchmark-data` branch ([§AR-benchmarks.7.5](AR-benchmarks.md#75-a-push-to-main-keeps-its-record)). It is a workflow of its own, never a job of `ci.yml`, and it runs on `workflow_run` of the workflow named `CI`, once that run has `completed`. `workflows: [CI]` matches `ci.yml` by its `name:`, so `ci.yml` stays named `CI`. It has one job, which calls `scripts/bench_store.py`.

A store runs after CI's verdict and outside it. A failed store is a red run of its own workflow: CI's run, its verdict and both required checks ([§AR-ci.13.2](AR-ci.md#132-the-required-check-is-an-aggregate)) are untouched. So is the duration tracker of [§AR-ci.requirements.3](AR-ci.md#requirements3-the-average-is-measured-and-drift-is-reported--realized), which reads only `ci.yml`'s runs; a store is no gate in the sense of [§AR-ci.requirements](AR-ci.md#requirements-what-cis-wall-clock-and-its-log-hold-to). The record is still in its artifact for 90 days, and re-running the failed store keeps it. No workflow but `bench-store.yml` names the branch or runs `bench_store.py`, and no hook does, so no job behind a required check and no local gate reads the store ([§REQ-runs-offline](../requirements/REQ-runs-offline.md#req-runs-offline-verification-never-depends-on-an-external-service)).

Two tests hold this section. `tests/integration/test_bench_store_workflow.py` reads `bench-store.yml` as text, the way [§AR-ci.13.5](AR-ci.md#135-the-layout-test) reads `ci.yml`, and runs each check against altered copies of the layout asked for, so a check that can no longer fail is caught. `tests/integration/test_bench_store.py` drives `bench_store.py` against a bare repository it makes.

#### 5.5.1 Only a push to main is stored

The job's `if:` admits a completed run only when `github.event.workflow_run.event == 'push'`, `github.event.workflow_run.head_branch == 'main'` and `github.event.workflow_run.head_repository.full_name == github.repository`, all three joined by `&&`. So a pull request, a merge-queue group ([§AR-ci.12](AR-ci.md#12-a-merge-queue-lands-every-pull-request)), a release candidate's dispatch ([§AR-ci.11](AR-ci.md#11-a-release-candidate-is-dispatched-the-same-ci)) and a run from a fork store nothing, and no fork's artifact reaches a job that holds a write token. The CI run's own conclusion is not in the guard: a record describes its commit, and the commit is on `main` whether another job of that run failed or not.

The job checks out `github.event.workflow_run.head_sha`, the commit the record measured, not the default branch's tip that `workflow_run` checks out otherwise. That commit is on `main`, so running its code is safe, and the record is read by the `bench_record.py` that wrote it. Under the tip, the series key would hash the tip's identity fields ([§AR-benchmarks.7.2](AR-benchmarks.md#72-the-series-key)), so the last push before a change to them would be refused, and a re-run, which replays the same event, would refuse it again.

The workflow grants nothing, `permissions: {}`. The job alone carries `contents: write`, to push to the branch, and `actions: read`, to read the CI run's jobs and download its artifact. Every field of the event reaches a command through `env:`, never as `${{ }}` inside a `run:`, so no value from the event is spliced into a shell script.

#### 5.5.2 No record is green only after the record job failed, was cancelled or was skipped

The store downloads `first.json` from the CI run's artifact `iai-callgrind-summaries`, the record the record job merged ([§AR-ci.5](AR-ci.md#5-benchmark-job)), and hands it to `bench_store.py` with the run's head commit and the conclusion of that job, which the forge names `cargo bench (record)`. When the record job ended `failure`, `cancelled` or `skipped` it wrote no record: the store says so, stores nothing and ends green, and CI is already red for that push. After every other conclusion a missing artifact or `first.json` fails the store: `success`, `timed_out`, `neutral`, `action_required`, `stale`, and a conclusion the forge did not report. None of those says the job wrote no record, so a missing one may be a record lost, the loss the store exists to end: failing closed costs one red store run and never hides one. A missing artifact does not stop the job before `bench_store.py` has judged it.

#### 5.5.3 A record is added, never changed

`bench_store.py` never checks the branch out. It fetches the branch's tip and builds one commit whose parent is that tip and whose tree is the tip's tree plus the one new file. It checks that the commit adds exactly that file and changes nothing else, and it pushes without force. It refuses, storing nothing, a record whose `commit` is not the commit it was given, whose `series` is not the key of its own identity fields, whose `environment` is not `ci`, or whose tier is not `pr`, a record of version 1 being read as `pr` (section 5.5.5). CI's record job of a push writes no other record, so a `local` or `nightly` record, stored by hand, never takes a commit's place before CI's own record of it ([§AR-benchmarks.7.5](AR-benchmarks.md#75-a-push-to-main-keeps-its-record)). When the file is already on the branch the first record stands, nothing is pushed, and the store ends green, so re-running CI on a stored commit stores nothing new. It never creates the branch: when `benchmark-data` is missing it fails and creates nothing.

Two pushes close together start two stores. When a store's push is refused because the tip moved, it fetches the new tip, adds its file there and pushes again, without force, a bounded number of times with a pause between. Each store adds a different file, so a retry never conflicts and the history stays linear. There is no `concurrency:` group: a group holds only one waiting run, so a third push would cancel the second push's store.

#### 5.5.4 A person makes the branch and its protection

Neither CI nor a run creates the branch or its ruleset; a person makes them, once, in this order, so the branch is never unprotected. A ruleset `benchmark-data` on `refs/heads/benchmark-data` forbids deletion and non-fast-forward updates, with no bypass actor. The `no new branches` ruleset gains an exclude of `refs/heads/benchmark-data`, the branch is pushed from an empty root commit, and the exclude is removed again, so that ruleset's `creation` rule covers the name from then on. `gh api repos/agent-grounds/grund/rules/branches/benchmark-data` then lists `creation`, `deletion` and `non_fast_forward`.

So a stored record stays readable for good at the commit that added it: no one can delete the branch or rewrite its history. The ruleset does not stop an ordinary later commit from changing a file at the tip. The store never makes one (section 5.5.3), and an edit a person made would leave the old bytes in history and show in the log.

#### 5.5.5 A record can be stored by hand

`python3 scripts/bench_store.py --record <first.json> --commit <sha>`, run in a checkout whose `origin` is this repository, stores a downloaded record of that commit exactly as the workflow does, and needs no workflow context. So a record whose store failed, or one from before the store existed, can be kept before its artifact expires. No dispatch backfills the store.

The series key is checked by the `bench_record.py` beside the script ([§AR-benchmarks.7.2](AR-benchmarks.md#72-the-series-key)): it hashes the values the record holds under the names that checkout's `IDENTITY_FIELDS` lists. So a checkout stores any record written under the same names, and a record written under other names is stored from a checkout that has them, such as a `git worktree` of the record's own commit, the way the workflow checks that commit out (section 5.5.1). A record of version 1 predates the `tier` field and was the `pr` tier's, as `bench_record.py merge` reads it, so it is stored as `pr/<commit>.json`. Its artifact predates the shards too, and holds it at `bench-records/first.json` rather than at the artifact's root.

## 6. PGO stays out of development CI

Development CI does **not** run the profile-guided-optimization pipeline. `scripts/pgo-build.sh` does an instrumented release build, runs the [§AR-benchmarks](AR-benchmarks.md#ar-benchmarks-instruction-counting-benchmarks-for-the-hot-cli-commands) self-repo hot command list as the training run, merges profiles with `llvm-profdata`, and rebuilds the optimized release artifact; that cost and toolchain belong to release packaging ([§FS-distribution.4](../functional-spec/FS-distribution.md#4-release-process), [§DA-pgo-release](../decisions/architectural/DA-pgo-release.md#da-pgo-release-distributed-binaries-are-pgo-built-trained-on-the-benchmark-workload)) and explicit benchmark work, not ordinary push/PR feedback. The manual **Pre-release checks** workflow runs the PGO script and self-checks the resulting binary before publish, and that job gates the platform it builds on; a platform whose hosted-runner PGO training is broken, which the release matrix marks `pgo_required: false`, publishes a self-checked LTO-only binary instead of blocking the release ([§FS-distribution.4.9](../functional-spec/FS-distribution.md#49-distributed-binaries-are-profile-guided-optimized)). The cross-registry candidate is a manual lane of the same kind: `candidate-rehearsal.yml` runs only on `workflow_dispatch`, and its PGO builds and its installed rows on each OS's runner never run on push or pull requests ([§FS-distribution-candidate.5.1](../functional-spec/FS-distribution-candidate.md#51-the-rehearsal-holds-no-credential), [§FS-distribution-candidate.5.6](../functional-spec/FS-distribution-candidate.md#56-a-row-runs-on-its-own-runner-and-a-skipped-row-is-named)). What push/PR CI does carry of it is cheap and offline — the inventory, manifest, launcher, mocked-publisher and workflow-isolation tests in `tests/integration/`, collected by the ordinary Python gate. The dev loop remains `fmt`, Python tests, pre-commit hooks, build, self-check, tests, the instruction-counting benchmark job ([§AR-ci.5](AR-ci.md#5-benchmark-job)), and the `commit-messages` job ([§AR-ci.8.1](AR-ci.md#81-ci-is-the-enforcing-layer)).

## 7. No changelog gate

Neither a hook nor CI asks a change for a changelog entry: no change is gated on the changelog, and the release builds the record itself, from the pull requests merged since the previous tag ([§FS-distribution.4.6](../functional-spec/FS-distribution.md#46-the-release-lists-the-pull-requests-merged-since-the-previous-tag)). A gate stood here — a `pre-push` hook and a pull-request job asking one question of local git, whether the branch added an entry — and released records cite this section for it, which is why the number stays. It went because the record moved: an entry per change cost every change a file, a number and a conflict on every rebase, and the hand-written release section that replaced it cost every release a pull request and held the scheduled one until somebody wrote it.

What the gate asked that the release still needs is answered elsewhere. Nothing waits in the tree for a release ([§FS-distribution.4.6](../functional-spec/FS-distribution.md#46-the-release-lists-the-pull-requests-merged-since-the-previous-tag)), so there is no pending entry to check and no hold: neither helper workflow numbers anything before the rotation, and the scheduled one no longer waits for a section to be written ([§FS-distribution.4.4](../functional-spec/FS-distribution.md#44-two-helper-workflows-bump-the-version-on-a-validated-candidate)). That nothing merged ships without its line is the release's own list to answer, since it asks the forge about every commit in the range and refuses rather than write a partial one ([§FS-distribution.4.6](../functional-spec/FS-distribution.md#46-the-release-lists-the-pull-requests-merged-since-the-previous-tag)). The one record a change still writes for a release, a verdict correction's `release-note` section ([§FS-distribution.4.6.4](../functional-spec/FS-distribution.md#464-compatibility-notices-come-from-the-decisions)), is asked of the tree by the Python suite — the backward-compatibility meter, `tests/integration/test_backwards_compatibility_routes.py` — which both `pre-commit run --all-files` and the test matrix run, so a correction without its notice fails on the pull request that makes it. The same suite holds every decision record's `release-note` section to the one bullet the bump publishes, reading it with the bump's own `release_notices` functions rather than a copy of them, in `tests/integration/test_release_note_shape.py` ([§FS-distribution.4.6.4.1](../functional-spec/FS-distribution.md#4641-the-shape-is-held-on-the-pull-request-that-writes-it)), so a notice the next release would refuse fails on the pull request that writes it rather than at that release.

## 8. Commit-message attribution gate

No AI-tool attribution boilerplate — a co-author trailer naming an assistant, a "generated with" marker, a session-link trailer, an assistant's no-reply address — may land in this repository's files or in its commit messages. `scripts/check_no_claude_attribution.py` owns the pattern set and scans three surfaces from one definition: staged file contents at `pre-commit`, a single message at `commit-msg`, and a revision range in CI. The patterns are narrow by design, because the target is machine-appended boilerplate rather than the subject matter: prose that discusses Claude is ordinary repository content and stays legal.

### 8.1 CI is the enforcing layer

The two local stages are advisory, and both failure modes are real. `pre-commit install --hook-type commit-msg` is a per-clone step a fresh clone has not run, and `--no-verify` skips whatever is installed; a repository history is the evidence that neither is hypothetical. CI is therefore the enforcing layer, and it runs as a dedicated `commit-messages` job rather than a step in the test matrix: the check needs the full history (`fetch-depth: 0`) that the matrix deliberately does not fetch, needs no Rust toolchain, and should report in seconds rather than behind a build.

### 8.2 The scanned range

The job scans what the event contributes, not the whole history — a pull request's `base..head`, a merge-queue group's `base_sha..head_sha` ([§AR-ci.12](AR-ci.md#12-a-merge-queue-lands-every-pull-request)), a push's `before..sha`. An unresolvable base is not a failure: the first push of a branch reports the all-zero SHA and a force-push reports a SHA the runner cannot reach, so the script narrows to the tip commit and says so. A gate on messages catches an offending commit only while it is still cheap to amend; once merged, the same trailer costs a history rewrite and a force-push of every descendant branch and tag, which is why this runs on the pull request rather than on the release.

## 9. File-size budget gate

Source files are read by agents far more often than they are written, and a file that has outgrown its subject is charged to every one of those reads. [§AR-core-module-layout.3](AR-core-module-layout.md#3-file-size) has stated the rule for implementation files since the core/CLI split — below 500 lines of code — but stated it with nothing to enforce it, and several files reached eight times that figure. Reviewers see diffs, not totals, so nothing pushes back at the moment a file crosses the line.

[`fissile`](https://github.com/agent-grounds/fissile) is that feedback loop, configured in `.agent-grounds/fissile.toml`. It measures and reports only; it never edits code, so how to split a flagged file stays a human decision.

### 9.1 Where the config lives

The config sits in `.agent-grounds/` rather than under `.agents/` because the two directories are different kinds of thing: `.agents/` holds agent *instructions*, and a managed agent sandbox may mount it read-only — a budget kept there is one the agent working in the checkout cannot repair. A config left at the old `.agents/` path is still read, as a deprecated fallback, so nothing breaks for a repository that has not moved — but the move here is only half a change on its own: discovery of the new home arrives in `fissile` 0.8.3, and any pin below it finds no config at all and falls back to built-in defaults. That is why [§AR-ci.1](AR-ci.md#1-pre-commit-is-the-source-of-truth)'s install step moves in the same commit as the file.

### 9.2 What a budget counts

Budgets are per file type, because the number that is right for an implementation module is wrong for a test file, a golden fixture, or a CI workflow. Line budgets count code lines only: this repository's method puts citations and rationale in doc comments ([§AR-core-module-layout.4](AR-core-module-layout.md#4-citation-placement)), and a budget that taxed comments would pay contributors to delete the grounding the project exists to keep. A byte budget is the backstop for a file that is expensive to read for some other reason.

### 9.3 Citable Markdown

Markdown carries two budgets, because `grund` decides who pays for a document's length. A spec is reached by citation and read as a lead, a section, or a table of contents ([§FS-show](../functional-spec/FS-show.md#fs-show-grund-reads-a-single-declaration-body-by-id)), so [§GOAL-token-economy](../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file) — the right amount of spec, not the whole file — is already answered at the read rather than at the file, and a ceiling in the hundreds of lines bought a reader nothing. It did have a price: a document whose numbered sections are live citation targets cannot be split without relocating those IDs and sweeping every citing site in the same change, so the budget was pushing a heavier move than the overage justified and collecting deferred exceptions instead.

The citable tree under `docs/` therefore gets soft 750 / hard 2000. The soft tier is about twice the code rules — `core-source` is 350/500 and `tests` 450/700 — and what it names there is the defect that still costs something: a document that has become two subjects. The hard tier is four times `core-source`'s ceiling, and that distance is deliberate: it is the tier with no override, so it has to sit far enough above the natural size of the documents it measures that no ordinary edit can reach it.

### 9.4 Entrypoints

Read-whole files, under the `entrypoints` rule, keep the tighter **soft** budget of 250, because nothing addresses `CLAUDE.md`, `AGENTS.md`, or a `SKILL.md` by citation: they are read whole, the agent entrypoints into every session's context, and every line is charged to every one of those reads. `README.md` is read whole too, but by whoever arrives first rather than in every session, and it teaches each feature by a real excerpt of a runnable example ([§REQ-readme.2](../requirements/REQ-readme.md#2-every-example-is-real)), so its own `readme` rule sets a soft budget of 450 under the same hard ceiling. That replaced a deferred exception accepting the README's 427 lines until its command walkthroughs moved into the guides, which they have. Their **hard** ceiling is raised, from 400 to 500, because `skills/grund-init/SKILL.md` then measured 387 lines and was held to be unsplittable, a shipped artifact whose bytes a test asserts, so the old ceiling stood 13 lines from blocking it. Its accepted size rose to 562 lines under a **structural** hard entry before the split showed the premise wrong: the skill is now a core and one file per reference section, each printed byte for byte by its own invocation ([§FS-init.5.5](../functional-spec/FS-init.md#55-the-core-and-its-sections)), so every file of it sits under the soft budget and the entry is gone. A section that outgrows the budget splits again or sheds its detail to a guide it links, as any entrypoint does.

### 9.5 One policy across the foundation projects

The four foundation projects state both budgets as one shared policy, which is the argument of [§AR-ci.9.3](AR-ci.md#93-citable-markdown) and [§AR-ci.9.4](AR-ci.md#94-entrypoints) generalized, and add that a changelog is an append-only record that is not line-measured at all. Adopting it moved the `docs` ceiling from 1500 to 2000, for the same reason the entrypoint ceiling moved from 400 to 500: `rhei`'s largest spec measures 1482 lines, so 1500 would have left it 18 lines — one ordinary edit from a block with no override, against documents that cannot be split cheaply ([§AR-ci.9.3](AR-ci.md#93-citable-markdown)). 2000 is not a new opinion either; it is the ceiling `rhei`'s own `AR-source-file-size.1` already declares for any file. The largest document today is `docs/functional-spec/FS-workspace.md`, at 1115 non-blank lines — 365 over the 750-line soft budget and 885 below the 2000-line hard ceiling — so re-measure it as the tree changes. When the limit was raised, no entry in either registry accepted a file against the `docs` rule, so raising it stranded none of them; the soft registry has since accepted this file against that rule.

### 9.6 Severities and registries

**soft** warns and exits 0 — the nudge to split next time you are in the file. **hard** fails the commit, and there is no override flag.

Each severity has its own registry, and an accepted file is recorded in the one matching the finding it silences: `docs/file-size-agent-exceptions.toml` for soft, `docs/file-size-human-exceptions.toml` for hard. The split is a permissions boundary, not bookkeeping — an agent may record soft debt on its own, while a hard entry is a human's to add. Every entry carries a reason, an `until` trigger that retires it, and the size it accepts. That last field is what makes this a ratchet rather than an amnesty: an accepted file is frozen at its recorded size, and one line past that ceiling brings the finding back at full severity. Adoption against an already-oversized tree is therefore honest in both directions — nothing is blocked today, and nothing can quietly get worse.

### 9.7 Structural and deferred entries

An entry's `kind` fixes what its reason has to establish, because "this file must not be split" and "this file has not been split yet" are different claims and only one of them is debt. A **structural** entry names the constraint that makes splitting illegal or harmful — a shipped artifact whose bytes are asserted, a type registry with no internal seam — and never expires, so its `until` is `indefinite`. A **deferred** entry names the boundary that is missing and what has to exist before the split can happen, and its `until` is that condition. A size threshold is not a condition: `until` may not say "revisit if it passes 500", because a bigger number retires nothing and the entry would outlive everyone who understood it. Keeping the two kinds apart is the point — a file that is merely large should not be able to borrow the language of one that must stay whole — and `fissile audit` reports them as separate counts so debt cannot hide inside necessity.

### 9.8 Soft twins

The kind also decides how far one entry reaches. A file over the hard budget is over the soft one too, and `fissile` leaves that soft finding standing under a **deferred** hard entry — by the tool's reasoning the split is still owed, so its nudge keeps arriving until something silences it. That is why this repository records such a file in both registries, the normal steady state for a file awaiting a split: the hard entry keeps the commit unblocked, and the soft entry stops the same known fact from being reprinted on every unrelated commit until someone stops reading the output. A warning nobody can act on today is noise, and noise is what teaches a reader to skip the one finding that mattered. A **structural** hard entry needs no second entry, because it silences both tiers itself: splitting the file is illegal, so a soft warning would name work nobody may do and no amount of work could clear.

The argument still lives in exactly one place. The hard entry carries it; the soft twin records the same ceiling, points at the entry that reasons, and retires when it does. Copying the prose across would give one file two arguments free to disagree, and the disagreement would surface as a reader trusting the stale half. The twin points by naming the hard registry, because an entry has no name of its own: what identifies it is the registry it lives in and the path it accepts, which is the same pair `fissile` leads its diagnostics with. An `EX-NNN` slug on top of that pair would be a second name for a fact the tool already computes, and the second name is the one that can go stale.

### 9.9 The registry test

Neither of those rules is visible to `fissile`, which validates the sizes but not what a reason may claim, so `tests/integration/test_file_size_exceptions.py` enforces them: kinds against their `until`, `until` against line-count triggers, twins against their originals — present for a deferred entry, absent for a structural one — and every recorded path against the tree, so an entry left behind by a rename fails the commit that renamed it. It also pins the registry schema version, so upgrading `fissile` without migrating the registries fails by name rather than as a parse error at commit time.

### 9.10 Remote enforcement

Per [§AR-ci.1](AR-ci.md#1-pre-commit-is-the-source-of-truth) the hook is in `.pre-commit-config.yaml` rather than a developer-local hook, so CI installs `lychee` beside `pre-commit` on the one leg that runs it, and `pre-commit run --all-files` enforces the same budgets remotely. The `fissile` binary is installed on every leg instead, ahead of the Python tests, because the ownership regression of [§AR-core-module-layout.1.5](AR-core-module-layout.md#15-split-ownership) measures with it. The hook takes its file list from pre-commit, so a commit measures what it touches while the CI run measures the whole tree; `[scan].exclude` applies either way, keeping generated asset mirrors and the e2e fixture trees out of both.

## 10. A test fixture owns the tree it scans

A test that writes a temporary repository and scans it must be the only thing writing, reading and removing that tree. Where two tests can name one root, the suite fails on a file it wrote itself, on one platform, and passes on the rerun — and a check that goes green on a rerun teaches everyone to rerun, which is how a genuine platform-specific regression gets waved through. That is [§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path) failing at the gate instead of at a release, and the gate cannot tell the two apart from the outside, so the rule is on the fixture: its root is unique by construction, and it holds that root exclusively.

### 10.1 Identity, not the clock

A fixture root's name is derived from what distinguishes the fixture — the process id together with the thread that builds it, or a process-wide atomic serial — and never rests on a reading of the wall clock. `libtest` runs a test binary's cases as threads of one process, so inside a binary the process id distinguishes nothing and a timestamp is the whole of such an identity; two cases whose readings land in one tick of the platform's clock resolution then name one directory. That resolution is the platform's to choose — macOS reports `clock_getres(CLOCK_REALTIME)` as 1000ns where Linux reports 1ns — so a clock-derived name is unique on a maintainer's machine and not on the runner's, which is the asymmetry [§AR-ci.2](AR-ci.md#2-platform-scope) leaves to the build-and-test jobs and the one no one is positioned to debug.

Stated so a test can hold a fixture to it, with no control over the clock: **two roots named from one clock reading still differ.** A thread id alone does not satisfy that, because two fixtures built on one thread share it; the process-wide atomic serial of the four `crates/grund-cli/tests` fixtures does, and so does a thread id carrying one. `crates/grund-core/src/testing.rs` (`test_root`) takes a caller-supplied name beside its thread id for the same reason.

This includes both `command_agreement` Tree constructors and its no-config control: repeated same-name allocations on one thread have distinct identities, even after an earlier instance is dropped. Live instances retain their own bytes and usable working directories through another instance's construction and either drop order, including construction paused before the buffer fixture's Git initialization. The agreement harness shares one root-ownership implementation across these three allocation sites.

### 10.2 The root is claimed, not adopted

A fixture creates its root exclusively — `create_dir` on the root itself, so a name already taken is an `AlreadyExists` the suite reports — rather than `create_dir_all` over whatever is already there. This is the half that closes the class rather than one instance of it: a unique name makes today's collision impossible, while an exclusive claim makes any later one a named failure at the moment it happens instead of a file vanishing mid-scan in an unrelated case.

Two things follow. A fixture removes only a tree it created, because a `Drop` that removes the root by name whether or not this fixture created it is what turns sharing into a deleted tree. And a refused claim leaves what it found untouched: clearing a root in order to take it moves the deletion earlier rather than removing it, so pre-clearing is not a substitute for refusing, however reasonable it looks as leftover hygiene.

For the agreement fixtures, this applies before writing any files, including an ordinary Tree with an empty file list and the no-config control's nested directories. An occupied candidate's claim fails without changing its sentinel bytes; unwinding or dropping a failed claimant cannot remove that root. A caller may choose a distinct candidate, but must still claim that candidate exclusively. Tests pin the shared claim operation's refusal directly as well as the constructors' independent lifetimes.

### 10.3 A fixture root inherits no config from above it

Discovery climbs from the path it is given to the filesystem root, probing both config names at every level ([§FS-config.1](../functional-spec/FS-config.md#1-file-location-and-discovery)), and a fixture root is no exception: a `grund.toml` anywhere above it, bare or under `.agents/`, becomes that fixture's config. A root built under the system temp directory therefore belongs to whatever tree `TMPDIR` happens to sit in, and a scratch directory inside a grounded project is an ordinary place for it to sit. A case asserting the zero-config fallback ([§GOAL-zero-config](../goals.md#goal-zero-config-works-on-any-conformant-tree)) then fails as though discovery had regressed, when what moved is the environment, and a contributor reads it as a regression in whatever change is under test. The walk is right and stays as it is; the rule is on the fixture: the root a fixture helper hands out is one that no config covers, under either name, at the root or through any of its ancestors.

The helper takes its base from a short candidate list, the system temp directory first and, on unix, `/tmp` where that differs, and uses the first candidate no config covers. It asks through discovery's own probe, so the guard reads exactly the names discovery reads. When every candidate is covered, the suite fails at the helper, naming `TMPDIR`, each candidate and the config that covers it, rather than handing out a covered root and letting a zero-config case invert into a failure about the fixture. This is the version-control guard `fixture_root` in `crates/grund-cli/tests/init_refused_targets.rs` already holds, applied to config files. The check belongs to the helper and not to the cases: no case sets `TMPDIR`, because the process environment is shared by every case running beside it.

Stated so a test can hold each fixture helper to it, `crates/grund-core/src/testing.rs` (`test_root`) and the base `crates/grund-lsp/tests/support/mod.rs` (`fixture_base`) gives every root grund-lsp's integration suites build, a zero-config one built by hand included: **no config under either name sits at or above a root the helper returns.** A zero-config case that does find one names the absolute path it found, so the failure says where the config came from.

The agreement harness's no-config control follows the same rule while sharing root ownership with the other agreement fixtures; it cannot inherit their configured worktree location merely to reuse the allocator.

### 10.4 Git descendants stay inside the fixture lifetime

Waiting for a Git command to return does not wait for its detached maintenance
descendants. A temporary Git fixture must prevent automatic GC and maintenance
from leaving a repository writer alive when strict cleanup starts, including
geometric repacking on newer Git versions. This supplies the lifetime half of
[§FS-distribution.4.5.1](../functional-spec/FS-distribution.md#451-release-helper-verification-owns-its-temporary-history-through-cleanup).

The changelog suites establish `gc.auto=0` and `maintenance.auto=false` as local
repository configuration immediately after initialization and before the first
commit. Shared initialization applies that policy to release listing, compatibility
notices and archive rotation; direct dated commits and rebases inherit it from the repository,
so command wrappers cannot be the only enforcement point. Explicit synchronous
Git work remains available to tests that need it.

`TemporaryDirectory.cleanup` stays strict. Ignoring cleanup errors, retrying removal
or sleeping until a detached writer finishes does not meet this contract. A
regression forces a real maintenance threshold and checks both the returned-command
boundary and strict removal; reading configuration alone is not evidence that the
tree has no outliving writer.

### 10.5 Version-history fixtures copy only version inputs

The offline version tests of [§FS-distribution-candidate.6.2](../functional-spec/FS-distribution-candidate.md#62-one-version-across-every-package) copy the complete
version-input closure, rather than a crate's implementation: the workspace
`Cargo.toml`, `Cargo.lock`, every present workspace member's `Cargo.toml`, and
the Python version manifest at `crates/grund-py/pyproject.toml`. The member list
comes from the workspace manifest and includes members outside `crates/`.
No source, asset, test corpus or native payload is copied or committed merely
to read or rewrite versions. The copied inputs retain their original bytes.

This changes fixture preparation alone. The tests still exercise disagreements,
all version rewrites, committed-ref versus dirty-worktree reads, and historical
layouts with the Python manifest at the root or absent. Fixture Git lifetime
remains [§AR-ci.10.4](AR-ci.md#104-git-descendants-stay-inside-the-fixture-lifetime). `tests/integration/test_ci_avoidable_work.py` checks the
copied bytes, the complete input set and the actual copy operations; the
existing version tests keep every behavioral assertion.

## 11. A release candidate is dispatched the same CI

Beside push, pull request and merge-queue group ([§AR-ci.12](AR-ci.md#12-a-merge-queue-lands-every-pull-request)), the workflow runs on `workflow_dispatch`, and that trigger is the release helpers': each dispatches it on its release commit and on the `-dev` advance built over it, and pushes neither to `main` until both have passed ([§FS-distribution.4.4.1](../functional-spec/FS-distribution.md#441-nothing-reaches-main-that-ci-has-not-passed)). A dispatched run is the CI a push to `main` gets and the macOS suites besides ([§AR-ci.14.2](AR-ci.md#142-a-release-candidate-runs-them-too)), never a narrower one, because the commit it judges is about to become `main`. It behaves as a push does: the benchmark job records counts without comparing ([§AR-ci.5.1](AR-ci.md#51-pull-requests-and-pushes)), and the commit-message job, given no `before` to scan from, narrows to the tip commit ([§AR-ci.8.2](AR-ci.md#82-the-scanned-range)). It records as a push does but stores nothing: the store of [§AR-ci.5.5](AR-ci.md#55-the-store-keeps-a-pushs-record) keeps a push's record alone, so a candidate's lasts as long as its artifact. `tests/integration/test_release_candidate_runs.py` holds the trigger to the helpers that use it.

A helper's runs belong to a round ([§FS-distribution.4.4.1.1](../functional-spec/FS-distribution.md#4411-a-moved-main-is-followed-not-refused)), and the controller both helpers call, `scripts/release_candidates.py`, starts and gives up rounds through `scripts/candidate_runs.py`. `dispatch` lists the runs already on the branch before it dispatches and names only a run that was not among them, because the publisher is dispatched on the branch and commit the dry run ran on and would otherwise be found as that dry run. `watch --main <sha> --deadline <unix-seconds> <run-id>...` reads `main` with `git ls-remote origin refs/heads/main` on every poll, as `scripts/advance_dev.sh` does, so five hours of polling spend nothing of the token's API budget. It exits 0 when every run passed; 1 on a run that did not pass, which wins over a move seen on the same poll; 3 when `main` is no longer `--main`; and 4 when the deadline passes first. Every exit but 0 first cancels the runs still going with `gh run cancel`, which the helpers' `actions: write` already allows, and a cancel the forge refuses because the run has just finished is not an error. A round given up early hands its runners back to the queue's groups ([§AR-ci.12](AR-ci.md#12-a-merge-queue-lands-every-pull-request)) within one poll rather than an hour later. The controller reads its waits from the environment, so a test does not wait them out: `GRUND_RELEASE_SETTLE` (how long `main` must hold still, 180 by default), `GRUND_RELEASE_BUDGET` (how long the helper has before its job's timeout), `GRUND_RELEASE_ROUND_MIN` (how much of it a round needs left to start, 5400) and `GRUND_RELEASE_RESERVE` (how much the watch's deadline leaves for publication), all in seconds. `tests/integration/test_release_candidates_follow_main.py` runs both helpers' whole jobs against a local origin and a stub forge that lands pull requests the way the queue does, and `tests/integration/test_release_candidate_watch.py` holds the watch, the dispatch and `release.yml`'s `expected_release_sha` to this.

## 12. A merge queue lands every pull request

`main` takes a pull request only through GitHub's merge queue. The queue tests each pull request on `main` together with every pull request queued ahead of it, under the checks a pull request must pass — the `cargo test` legs of Ubuntu and Windows, each the aggregate of its operating system's jobs ([§AR-ci.13.2](AR-ci.md#132-the-required-check-is-an-aggregate)) — and lands exactly the commit it tested, so nothing reaches `main` that CI has not passed, as [§FS-distribution.4.4.1](../functional-spec/FS-distribution.md#441-nothing-reaches-main-that-ci-has-not-passed) holds the release helpers to. That is why no pull request has to be current with `main` itself, and the up-to-date requirement is off: a pull request that went green on an older `main` is not rebased and run again for every merge ahead of it. The requirement is not merely dropped, because in this repository two pull requests that are each green often fail together — one moves a point the other still cites, and only their combination is red. The queue meets that combination on its own ref, before `main`; the requirement switched off without a queue would let it land, and `main` would go red on a commit no run had judged ([§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path)).

The workflow runs on `merge_group` for the queue. The test matrix runs there as it does on a push, and the commit-message job scans the group's `base_sha..head_sha` ([§AR-ci.8.2](AR-ci.md#82-the-scanned-range)). The three benchmark jobs of [§AR-ci.5](AR-ci.md#5-benchmark-job) run there too and hold the group's head to the absolute ceilings alone ([§AR-ci.5.1](AR-ci.md#51-pull-requests-and-pushes)), so a crossing that only the combination makes, or that a pull request measured on an older `main` could not see, fails in the queue instead of on `main`. Where a group's tree is its pull request's own head, its run is one more sample of a binary already measured, so a group does not reliably stop a crossing by same-binary jitter ([§AR-benchmarks.7.3](AR-benchmarks.md#73-the-same-binary-bound)). It writes no run record: the push the queue makes to `main` lands the same revision, stays the one record per revision, and keeps its two same-binary runs. No gate's time grows ([§AR-ci.requirements.1](AR-ci.md#requirements1-every-gate-averages-at-most-fifteen-minutes--directional)): no benchmark job does more on a group than on a push, which runs every shard twice, and the push runs [§AR-ci.requirements.3](AR-ci.md#requirements3-the-average-is-measured-and-drift-is-reported--realized) averages run as before. The cost is concurrency: while a group runs, its shards take runners from the organization's shared limit on concurrent jobs, so another run may wait to start. `tests/integration/test_merge_queue_runs.py` holds the trigger, the jobs and a group's benchmark routing to this.

## 13. Each operating system's suites run as parallel jobs

A leg that runs its suites one after another takes as long as their sum, and the
slowest leg is how long every pull request and every merge-queue group waits
([§AR-ci.requirements.1](AR-ci.md#requirements1-every-gate-averages-at-most-fifteen-minutes--directional)). So the suites of a leg run as jobs of their own, side by
side, and the leg takes as long as its longest one. The leg is written once, as the
jobs of the reusable workflow `.github/workflows/ci-leg.yml`, which `ci.yml` calls
once per operating system with that system as its `os` input: written out per
system, the two legs were one text twice and took `ci.yml` past fissile's limit for
a workflow. Nothing is generated by YAML anchors, because the tests that hold this
layout read the workflows as text: the CI Python has no YAML parser, and a local
call is one line such a reader can follow ([§AR-ci.13.5](AR-ci.md#135-the-layout-test)). A called job's check is
named `<caller's name> / <job's name>`, as `windows-latest / rust 3/4`, and a
called workflow takes none of its caller's `env:`, so the leg declares its own.

### 13.1 One job per language

On `ubuntu-latest` and `windows-latest` each leg is the **Rust** jobs and three
Python jobs. The Rust jobs are the shards of [§AR-ci.13.7](AR-ci.md#137-a-legs-cargo-suite-runs-in-shards): each runs one shard
of `python scripts/run_cargo_suite.py --workspace --all-targets --locked --features
grund/test-workspace-load-count` ([§AR-ci.3](AR-ci.md#3-current-hooks)), and shard 1 also runs `cargo fmt
--all -- --check`, the native watch contract, `cargo build --quiet --workspace
--all-targets --locked` and the grund self-check of §AR-ci.4. The **Python** job
runs `python scripts/run_python_gate.py --rest`, the **Node** job runs `python
scripts/run_python_gate.py --only-node-acceptance`, and the **Python acceptance**
job runs `python scripts/run_python_gate.py --only-python-acceptance` and then
`python scripts/run_python_gate.py --only-slow` ([§AR-ci.3.4.3](AR-ci.md#343-selecting-the-binding-acceptances-and-the-slow-tests)). No job on those two
runs more than one of those suites, a shard counts as one, and the Python
acceptance job's two are the one exception: the slow selection held the Python job
past [§AR-ci.requirements.1](AR-ci.md#requirements1-every-gate-averages-at-most-fifteen-minutes--directional), and a job of its own would spend its minutes again on
the checkout, the toolchain and the oracle the acceptance job has already prepared.

`macos-latest` is not a leg of `ci.yml`: its suites run once a day, in one job
of their own workflow ([§AR-ci.14](AR-ci.md#14-macos-runs-once-a-day)).

On every operating system each of those commands runs exactly once: none is
dropped by the split and none repeated by it. The plain `python
scripts/run_python_gate.py` runs in no job, because it would run both acceptances
and the slow tests a second time, and the unsharded cargo command runs in no job,
because it would run every shard a second time.

### 13.2 The required check is an aggregate

Ubuntu and Windows each keep one job named `cargo test (<os>)`, the name the
ruleset requires and the merge queue waits for, so neither changes. It runs no
suite of its own. It `needs:` its operating system's call of the leg, whose result,
as a job needing it reads it, is every called job's together, so it judges every
job that runs a command of [§AR-ci.13.1](AR-ci.md#131-one-job-per-language) on its operating system, every cargo shard
among them ([§AR-ci.13.7](AR-ci.md#137-a-legs-cargo-suite-runs-in-shards)) — on Ubuntu it needs the pre-commit job too, and the
three benchmark jobs of [§AR-ci.5](AR-ci.md#5-benchmark-job), so that the limits of [§AR-ci.5.2](AR-ci.md#52-regression-limits) hold a merge
on every shard — and no job of another. The shard job is a matrix over shards, not
operating systems: each of its instances runs on the leg's system, so its result,
all of them together, is exactly "every shard passed". That is why the leg is
called once per operating system rather than over a matrix of them: a matrix
job's result is all its instances' together, so an aggregate needing one would
judge every operating system's suites and wait for all of them.

It passes only when every job it needs succeeded. Its `if:` is `always()`,
because a job whose need failed is otherwise skipped, and a skipped required
check counts as passed; its step then fails on any result other than
`success`, so a need that failed, was cancelled or was skipped fails it, on every
event: the benchmark jobs run on a merge-queue group too ([§AR-ci.12](AR-ci.md#12-a-merge-queue-lands-every-pull-request)), so
the Ubuntu check accepts only their success there as everywhere else. It runs
on `ubuntu-latest` whichever operating system it judges.

### 13.3 Every hook runs once

The pre-commit step runs on Ubuntu with `SKIP` naming exactly the hooks whose
command another job already runs: `cargo-fmt-check`, `cargo-build` and
`cargo-test`, whose commands the Rust jobs' steps spell, the last as its shards
([§AR-ci.13.7](AR-ci.md#137-a-legs-cargo-suite-runs-in-shards)), and `python-test`, whose command the Python, Node and Python
acceptance jobs run between them as its four selections.
Every other hook runs there and only there. So each hook's command runs once on
Ubuntu, a skipped hook is one a named job runs, and the list in
`.pre-commit-config.yaml` stays what is checked ([§AR-ci.1](AR-ci.md#1-pre-commit-is-the-source-of-truth));
`tests/integration/test_ci_precommit_parity.py` computes the set it must skip
from the workflow rather than from a list of its own ([§AR-ci.1.2](AR-ci.md#12-the-parity-test)).

### 13.4 Cache keys are per job

Every job still restores the cargo cache and builds what it needs. No two jobs
on one operating system save one cache key: an entry is never overwritten, and a
save to a key that already has one saves nothing, so the first job to save a
shared key would fix what every other one restores, and theirs would never be saved. A key names its job, as
a literal does; a restore prefix may still fall back to another job's entries,
because reading one overwrites nothing, and a job may restore another's exact key
when it saves none under it. The Node target's cache of [§AR-ci.3.4.1](AR-ci.md#341-the-node-acceptances-shared-build-target) is the job's
that runs the Node acceptance.

The instances of one matrix job are one job here. The cargo shards of a leg
build the same tree, so they share one key, every shard restores it, and one shard
alone saves it: the one the deal of [§AR-ci.13.7](AR-ci.md#137-a-legs-cargo-suite-runs-in-shards) gives the command agreement, the
only shard that builds the agreement's 1.95.0 target under `target/agreement`, so
its entry holds what every shard's does and that target besides. The Python
acceptance job restores the Python job's exact key, the oracle's build among it,
and saves only its own target cache ([§AR-ci.3.4.5](AR-ci.md#345-the-python-acceptances-shared-build-target)).

#### 13.4.1 A key moves with the profile

A key whose job builds its cached target under the dev or test profile hashes the
workspace `Cargo.toml` with every `Cargo.lock`, because the manifest is where that
profile is set ([§AR-ci.15](AR-ci.md#15-dependencies-compile-optimised-in-a-dev-build)). A key hashing the lock alone would hit exactly on an
entry compiled under the profile before the change: every run would recompile what
the change touched, and the exact hit would never save the result, so no run would
start warm again until the lock next moved. The benchmark cache builds the release
profile alone and keeps the key of [§AR-ci.5](AR-ci.md#5-benchmark-job).

### 13.5 The layout test

`tests/integration/ci_workflow.py` reads `ci.yml` as text, puts each job of
`ci-leg.yml` in place of each local call to it, as `<caller>/<job>` with the call's
`os` for `inputs.os`, and expands each `matrix.os` and `matrix.shard` list; every
test that reads `ci.yml` reads it so, and `tests/integration/test_ci_language_jobs.py`
holds it to [§AR-ci.13.1](AR-ci.md#131-one-job-per-language), [§AR-ci.13.2](AR-ci.md#132-the-required-check-is-an-aggregate) and [§AR-ci.13.4](AR-ci.md#134-cache-keys-are-per-job). A step whose `if:` a matrix value or the system makes
false is not in that instance. The test runs its checks against fixture workflows
too, so a check that can no longer fail is caught. `tests/integration/test_ci_shards.py` holds the shards of
[§AR-ci.13.7](AR-ci.md#137-a-legs-cargo-suite-runs-in-shards) and the Python acceptance job the same way, and
`tests/integration/test_cargo_suite_shards.py` runs the wrapper's `--shard` on a
workspace of its own. `tests/integration/test_bench_tier_ci.py` holds the
benchmark jobs' shape: the build job, a shard matrix of the catalog's `shards`
legs, the record job, the aggregate needing all three and accepting only their
success, and their pins; `tests/integration/test_merge_queue_runs.py` holds a group's
routing: one ceilings-only head run per shard, no record, and on every event one
step that measures the head.

### 13.6 Only a push to main saves a cache

Every cache is restored on every event, and saved only by a push to
`refs/heads/main`, or by the daily macOS run of [§AR-ci.14](AR-ci.md#14-macos-runs-once-a-day), which the schedule starts on that
same ref. A run restores from its own ref or the default branch's and
nowhere else, so an entry saved from `refs/pull/<n>/merge`, from a merge queue's
`gh-readonly-queue/main/pr-<n>-<sha>`, from a tag or from a dispatch is never read
again; it only counts against the repository's 10 GB limit, and the forge makes
room for it by evicting the oldest entries, which are main's. That is what kept
every pull-request and queue run from finding main's exact key, so each one fell
back to a partial match, rebuilt, and saved gigabytes nobody reads. When this was
written the caches stood at 13.6 GB, 11.5 GB of it on such refs.

So no workflow under `.github/workflows/` uses the combined `actions/cache`, which
saves on every event its workflow runs on. Each cache is an `actions/cache/restore`
step with no event guard, and an `actions/cache/save` step after the work, with the
same path and key, guarded by `github.event_name == 'push' && github.ref ==
'refs/heads/main'` on the step or its job — `'schedule'` in place of `'push'` in a
scheduled workflow. A workflow that never runs on a push to
main has no save step, since its guard could never be true. `release.yml`, run from
tags and dispatches, caches nothing: no run ever saves a key it could read, so it
builds cold. `pre-release-checks.yml`, dispatched only, restores and never saves, so
its own key never hits and only its `<os>-cargo-` fallback finds anything: main's
Linux cargo caches. Keys stay per job ([§AR-ci.13.4](AR-ci.md#134-cache-keys-are-per-job)).

What main saves fits under the limit with room for a second set while a changed
`Cargo.lock` waits for the old entries to age out. So a cargo cache holds the
registry (and the git checkouts, should `Cargo.lock` ever gain a git dependency)
and the dependencies' compiled artifacts, and not the workspace crates' own
artifacts, which every run rebuilds from its own sources, nor incremental state: a workflow that caches cargo sets `CARGO_INCREMENTAL: 0`,
as the Node target's builds already do ([§AR-ci.3.4.1](AR-ci.md#341-the-node-acceptances-shared-build-target)).
The `rust` shards' target also holds the command agreement's 1.95.0 target, nested
under `agreement/` one per checkout, and `scripts/prune_cargo_cache.sh` prunes
each of those the same way before the save.

`tests/integration/test_ci_cache_saves.py` reads every workflow as text and holds
it to this point, and it runs its checks against fixture workflows too, so a check
that can no longer fail is caught.

### 13.7 A leg's cargo suite runs in shards

The cargo suite is the longest command of a leg. One Windows job running it, with
the build and checks before it, took fifteen to eighteen minutes on a fast runner
and twenty-five to thirty on a slow one, so no trimming of one test brings that
job under [§AR-ci.requirements.1](AR-ci.md#requirements1-every-gate-averages-at-most-fifteen-minutes--directional) with room. So on `ubuntu-latest` and
`windows-latest` it runs as `n` jobs, `n` at least two and one `n` per leg: the
instances of the leg's `rust` job, a matrix over `shard: [1, …, n]`, where the
instance for shard `k` runs

```text
python scripts/run_cargo_suite.py --shard <k>/<n> --workspace --all-targets --locked --features grund/test-workspace-load-count
```

with every `k` from 1 to `n` in exactly one job of the leg. The rest of the Rust
commands of [§AR-ci.13.1](AR-ci.md#131-one-job-per-language) run once per leg, on shard 1. Every shard restores the
leg's one cargo key and one saves it ([§AR-ci.13.4](AR-ci.md#134-cache-keys-are-per-job), [§AR-ci.13.6](AR-ci.md#136-only-a-push-to-main-saves-a-cache)), and the leg's
required check `cargo test (<os>)` keeps its name and needs every shard through
the leg's call ([§AR-ci.13.2](AR-ci.md#132-the-required-check-is-an-aggregate)), so the ruleset and the merge queue wait on what they did.
The `cargo-test` hook runs the unsharded command, so the local verdict is the one the
shards reach together, as with the selections of [§AR-ci.3.4.3](AR-ci.md#343-selecting-the-binding-acceptances-and-the-slow-tests).

The wrapper's `--shard <k>/<n>` comes before `cargo test`'s own arguments. It
lists the targets the rest of the arguments select, deals them into `n` shards, and
runs `cargo test` on the `k`-th shard's targets with the rest of the arguments
unchanged: the packages, `--locked`, the features and any libtest arguments after
`--`. The deal is a function of that list alone, so every job of a leg computes the
same cut, and every target is in exactly one shard: the shards together run exactly
the tests the unsharded command runs, none dropped and none run twice. The deal
keeps the shards' minutes close, because a leg takes as long as its longest job,
and a target's count of tests says little of its minutes. `scripts/cargo_shards.py`
names the slowest targets with the seconds each ran on Windows, and deals those
first, heaviest first, each to the shard least loaded so far, with shard 1 starting
from the seconds its other Rust checks cost; the targets it does not name follow in
order, round the shards. So the command agreement, the scan determinism test and
the core's unit tests never share a shard, and none of the heaviest runs on
shard 1. A target that turns slow is added to the table, not to the
workflow. A shard that selects no
target exits 5 rather than passing on nothing run, as an empty selection of
[§AR-ci.3.4.2](AR-ci.md#342-per-test-durations) does. A `--shard` that is not `<k>/<n>` with `1 <= k <= n` exits 2 with
a usage line before cargo runs, so a mistyped shard is never silently the whole
suite or none of it. Each shard prints its own summary of
[§AR-ci.requirements.5.1](AR-ci.md#requirements51-a-suite-prints-its-failures-its-check-ins-and-one-summary), counting the tests of its targets alone.

## 14. macOS runs once a day

The macOS suites are no gate. `.github/workflows/macos.yml` runs them once a day on
`main`, on a schedule, and on `workflow_dispatch`, and never on a push, a pull
request or a merge-queue group, so no pull request waits for them. They were the
slowest gate — `suites (macos-latest)` averaged 20.2 minutes over the eight pushes
agent-grounds/grund#659 measured — and the free plan's five concurrent macOS jobs
are shared by every run, the queue's groups included ([§AR-ci.12](AR-ci.md#12-a-merge-queue-lands-every-pull-request)). Ubuntu and Windows
still judge every change before it lands, so what this gives up is a regression
only macOS shows, which reaches `main` and is reported within a day instead of
refused. A release is the exception: no release ships what macOS fails
([§AR-ci.14.2](AR-ci.md#142-a-release-candidate-runs-them-too)). Serves [§GOAL-fast-feedback.3](../goals.md#3-measurable).

### 14.1 One job runs every suite

The one suite job, `suites (macos-latest)`, runs each command of [§AR-ci.13.1](AR-ci.md#131-one-job-per-language)
exactly once, one after another, every cargo shard and every selection of the
Python gate among them: nobody waits on it but a release, so splitting it
would spend macOS slots and buy little. It is written once, in the reusable workflow
`.github/workflows/macos-suites.yml`, which `macos.yml` calls on its schedule and
`ci.yml` calls on a dispatch ([§AR-ci.14.2](AR-ci.md#142-a-release-candidate-runs-them-too)). It saves its caches on the scheduled run alone
([§AR-ci.13.6](AR-ci.md#136-only-a-push-to-main-saves-a-cache)), so the next day's run finds them: a called workflow sees its caller's event,
so a release candidate's call saves nothing.

### 14.2 A release candidate runs them too

No release ships with macOS broken. `ci.yml` calls the suite job on
`workflow_dispatch`, the release helpers' trigger ([§AR-ci.11](AR-ci.md#11-a-release-candidate-is-dispatched-the-same-ci)), and on no other event,
so each candidate commit is judged on macOS beside Ubuntu and Windows before `main`
moves to it: a failed macOS job fails the dispatched run, and the helpers move
nothing until that run has passed ([§FS-distribution.4.4.1](../functional-spec/FS-distribution.md#441-nothing-reaches-main-that-ci-has-not-passed)). That holds whatever the
last daily run said, because the daily run judged an older commit. The reusable
workflow asks for `contents: read` alone, and the report job of [§AR-ci.14.3](AR-ci.md#143-a-failed-daily-run-opens-one-issue) stays
in `macos.yml`: a called workflow may not ask for more than its caller grants, and
`ci.yml`'s token grants no issue writes.

### 14.3 A failed daily run opens one issue

When the scheduled run's suite job fails, a report job of `macos.yml` after it, with
`issues: write`, runs `scripts/macos_daily.py`. It reads the run's jobs from the
forge and writes one report naming the commit, the run and each failed step, and
puts it on one open issue by the rule of [§AR-ci.requirements.3](AR-ci.md#requirements3-the-average-is-measured-and-drift-is-reported--realized), with its own
marker line, `<!-- macos-daily-failure -->`, and its own kind label, `bug`, because
a suite is red on `main`: it opens the issue when no open issue's body begins with
the marker, and edits the one that does instead of opening a second. The one
implementation of that rule is `scripts/ci_duration.py`'s, which this script calls
with its own marker and its own label. A green run writes nothing, so closing the
issue is a person's call, and a dispatched run reports nothing, because whoever
dispatched it is watching it.

### 14.4 The workflow test

`tests/integration/test_ci_macos_daily.py` reads the three workflows as text and
holds them to this point — `macos.yml`'s triggers and its report job's wiring,
`macos-suites.yml`'s one suite job and its permissions, and that `ci.yml` calls it
on a dispatch and on no other event and runs no macOS job of its own — and holds the
report, the open-or-edit decision and the issue's label to [§AR-ci.14.3](AR-ci.md#143-a-failed-daily-run-opens-one-issue) offline, against
canned jobs and a `gh` that records each call.

## 15. Dependencies compile optimised in a dev build

The workspace `Cargo.toml` sets `opt-level = 1` for every package that is not a
workspace member, under `[profile.dev.package."*"]`, so the dev profile and the
test profile that inherits it run optimised dependencies around unoptimised
workspace crates. A gate spends most of its test time inside those dependencies,
the regex engine, the parsers and the walker that every fixture of every suite runs
through, and they run about twice as fast at level 1. The workspace crates stay at
level 0, so an edit recompiles no faster or slower than before and a debugger still
sees them as written. The price is paid once per cache key: a cold dependency
compile takes longer, and the key moves with the profile ([§AR-ci.13.4.1](AR-ci.md#1341-a-key-moves-with-the-profile)), so a warm
job pays nothing. It is how the first S4 tranche pays inside the `rust` jobs for the
time its comparison adds ([§REQ-one-answer.proof.delivery.payment](../requirements/REQ-one-answer.md#proofdeliverypayment-each-tranche-pays-for-its-own-time)), with the
agreement's two natives built in one invocation into a target those jobs cache
([§AR-ci.13.6](AR-ci.md#136-only-a-push-to-main-saves-a-cache)).

## requirements: What CI's wall clock and its log hold to

Requirements on how long the gate takes (requirements.1–4) and on what it prints (requirements.5), each carrying a **mark**, as [§FS-config.requirements](../functional-spec/FS-config.md#requirements-what-the-config-contract-holds-to) marks its own: *realized* where CI behaves this way today and a section here says so; *directional* where it binds what is written from here on rather than describing what runs today; *deferred* where a named follow-up owes it. Every pull request waits for the slowest gate, and so does every merge, which the merge queue of [§AR-ci.12](AR-ci.md#12-a-merge-queue-lands-every-pull-request) lands only after a full run on its group, so a slow gate is the loop [§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible) says gets routed around. A **gate** here is one job of `ci.yml` as the forge names it, and each leg of a matrix job is a gate of its own: `cargo test (ubuntu-latest)` and `cargo test (windows-latest)` are two gates. The average is the target and the maximum is the hard limit: a gate that drifts past the average is a report, and a gate past the maximum is a failure.

### requirements.1: Every gate averages at most fifteen minutes — directional

Each gate's mean duration over the tracking window of [§AR-ci.requirements.3](AR-ci.md#requirements3-the-average-is-measured-and-drift-is-reported--realized) is at most the threshold [§AR-ci.requirements.4](AR-ci.md#requirements4-the-threshold-and-the-window-are-configuration--realized) configures, fifteen minutes unless it says otherwise. Directional: when this was written the three test legs averaged about 43, 42 and 25 minutes, so it binds every change from here on not to add to a gate's time without paying for it elsewhere, and it becomes realized once every gate is under it. Serves [§GOAL-fast-feedback.3](../goals.md#3-measurable).

#### requirements.1.verification: A cost correction measures comparable full gates

A correction to reported gate drift records at least two successful samples
per gate and per side, at one fixed base SHA and one fixed candidate SHA, on
the affected operating system. The suites, toolchains and cache conditions
must be comparable and recorded; skipped or missing proof is not a saving.
Record full job elapsed time, every setup, restore, build, test and save
contribution, run and job URLs, and machine-readable raw timing evidence.
Explain seconds removed and added by mechanism. Each affected candidate gate's
mean must meet [§AR-ci.requirements.1](AR-ci.md#requirements1-every-gate-averages-at-most-fifteen-minutes--directional) with useful margin; obtain more samples
when variance makes the minimum inconclusive. Agreement tranche costs also
remain subject to [§REQ-one-answer.proof.delivery.payment](../requirements/REQ-one-answer.md#proofdeliverypayment-each-tranche-pays-for-its-own-time).

Branch-local mechanism tests and paired candidate measurements establish the
correction before merge. The rolling successful-main-push measurement of
[§AR-ci.requirements.3](AR-ci.md#requirements3-the-average-is-measured-and-drift-is-reported--realized) remains the operational follow-up: changing a branch
cannot change past main durations, and old samples must neither be replaced
with fabricated passes nor treated as a branch-local regression test. The
configured threshold and window ([§AR-ci.requirements.4](AR-ci.md#requirements4-the-threshold-and-the-window-are-configuration--realized)), suite placement and
proof coverage ([§AR-ci.13.1](AR-ci.md#131-one-job-per-language), [§AR-ci.13.3](AR-ci.md#133-every-hook-runs-once)), and deferred timeout decision
([§AR-ci.requirements.2](AR-ci.md#requirements2-no-run-of-a-gate-passes-twenty-minutes--deferred)) remain unchanged by this verification protocol.

### requirements.2: No run of a gate passes twenty minutes — deferred

Every job of `ci.yml` carries `timeout-minutes: 20`, so a run past it fails like any other red check rather than holding a pull request for an hour. This bounds the job, and the self-check's own `timeout` of [§AR-ci.4](AR-ci.md#4-performance-smoke-guard) stays inside it as the catastrophic backstop for that one step; the two do not replace each other. It binds a dispatched release candidate as it binds a push, because a candidate runs the same `ci.yml` ([§AR-ci.11](AR-ci.md#11-a-release-candidate-is-dispatched-the-same-ci)). Deferred: a twenty-minute ceiling today would fail every pull request, so it lands after the Windows Python gate (agent-grounds/grund#585) and the Ubuntu `pre-commit` step's link check come under it, and the change that adds the timeouts flips this mark.

### requirements.3: The average is measured, and drift is reported — realized

The average is measured, not assumed. `scripts/ci_duration.py` reads the last *window* successful runs of `ci.yml` triggered by a push to `main`, newest first, and averages each gate's duration — a job's `completed_at` less its `started_at` — over the runs it appears in; a skipped job is not a sample. It averages each step of each gate the same way, which is where the time went. When any gate's average exceeds the threshold, it writes one report naming each such gate, its average, the window, and that gate's slowest steps, and puts it on one open issue: it opens the issue when no open issue's body begins with the marker line `<!-- ci-duration-drift -->`, and edits the body of the one that does instead of opening a second; an issue that only quotes the marker further down is not the drift issue. The issue carries its kind, as every issue here does: the tracker opens it with the label `tokens`, because a gate over the threshold is CI spending wall clock it should not, and adds that label again with each edit, so a drift issue opened without it carries it after the next report, and its other labels stay as they are. A query that returns no run measures nothing, so the tracker says so and exits 1 rather than reporting every gate under the threshold. A run in which every gate is under the threshold writes nothing; closing the issue is a person's call. The workflow `.github/workflows/ci-duration.yml` runs it on a schedule and on `workflow_dispatch`, never on a push or a pull request, with `actions: read` and `issues: write`. It stays out of the pre-commit hook list and out of `grund check`: it reads the forge, which [§REQ-runs-offline](../requirements/REQ-runs-offline.md#req-runs-offline-verification-never-depends-on-an-external-service) keeps out of any verdict and [§AR-ci.1](AR-ci.md#1-pre-commit-is-the-source-of-truth)'s parity keeps out of the local gate. Its averaging, its report and its open-or-edit decision are functions of the forge's JSON, so `tests/integration/test_ci_duration.py` holds them to this offline, against canned runs, and runs the tracker against a `gh` that records each call, which holds the label on the issue it opens and on the one it edits. This is requirements.1's meter in the sense of [§AR-goal-measurement](AR-goal-measurement.md#ar-goal-measurement-goal-and-requirement-meters-live-outside-goals).

### requirements.4: The threshold and the window are configuration — realized

The threshold and the window live in `.github/ci-duration.json`, which the tracker reads on every run and in which it finds no literal it would otherwise fall back to: `threshold_minutes`, the average a gate may not exceed (`15`), and `window`, how many runs it averages over (`8`), a whole number from 1 to 100 because the forge lists at most 100 runs a page. A missing file or a missing key is an error naming it, never a silent default. They are repository CI configuration and not a `grund.toml` key: `grund` never reads them, and a key it never reads would cross [§FS-config.requirements.8](../functional-spec/FS-config.md#requirements8-a-projects-meaning-is-self-contained--realized). The maximum of requirements.2 is not here, because the forge reads it from `ci.yml` and from nowhere else.

### requirements.5: A green step prints a screenful, a red one its failures — realized

What a CI step prints is part of how CI is built. Whoever reads a red job, a person or an agent running `gh run view --log-failed`, reads every line before the failure, so a line that says only that something passed is paid for on every pull request and every push to `main`: the cost [§GOAL-token-economy](../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file) says a read should not carry, on the loop [§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible) keeps short. Realized: before it, the workspace `cargo test` step printed about 2200 lines on Ubuntu, 1640 of them `test … ok`, the `test-watch` contract test about 270, and `pip install pre-commit` 442; every step below now prints what requirements.5.1–5.4 ask, and a step added from here on is held to the same.

#### requirements.5.1: A suite prints its failures, its check-ins and one summary

A suite step is the workspace `cargo test` on all three legs, which runs the e2e runner `crates/grund-cli/tests/e2e.rs` among its targets, the `test-watch` contract test of the `native watch contract` step, and the Python gate of [§AR-ci.3.4](AR-ci.md#34-the-python-gates-inputs). It prints only these, and exits with the suite's own status:

- each failure, under the failing test's name, with its full output: what it wrote to stdout and stderr, its own and its child processes', and the panic or assertion message;
- a check-in each time another 5% of the suite has finished, so at most twenty, one line each: `<done>/<total> tests, <failed> failed, <elapsed>s elapsed, slowest: <test> <seconds>s, …`, naming up to three of the slowest tests so far, slowest first, and the last check-in reads `<total>/<total>`;
- the slowest-ten table of [§AR-ci.3.4.2](AR-ci.md#342-per-test-durations), where the suite keeps one;
- each line that begins `gate time: `, which says what the gate's time went on that no
  test's own seconds separate: the cargo wrapper's one `gate time: build <seconds>s` once
  its build has finished, so a cold dependency compile is a phase of its own, and the
  lines a test's child process writes, as each command-agreement host writes how long it
  took to be ready, how much of that was the toolchain install and the natives' build,
  and how long it then served, and the one line each sweep a tranche pays with writes
  itself, past libtest's capture, `gate time: <sweep> <what it compared> on <w> worker(s)
  in <seconds>s`, as the scan-determinism, lsp/cli parity, e2e determinism and e2e cases
  sweeps do, because such a sweep need not be among the three slowest the check-ins name,
  and the seconds it costs or removed must still be read in the same job ([§REQ-one-answer.proof.delivery.payment](../requirements/REQ-one-answer.md#proofdeliverypayment-each-tranche-pays-for-its-own-time));
- each line that begins `node build: `, a build line of
  [§AR-ci.3.4.1.1](AR-ci.md#3411-one-release-build-and-every-build-on-one-line), one per build. Only the Python gate has these lines:
  when a passing or skipped test, its fixtures or its child processes write one, the gate
  prints it once, as a line of its own, and one that a class or module teardown writes
  after the last test is printed when the run ends;
- one summary as its last line: `<total> tests: <passed> passed, <failed> failed, <ignored> ignored, <elapsed>s`, where *ignored* counts libtest's ignored tests and `unittest`'s skipped ones. A red summary still adds up: before the elapsed time it adds `, <n> not run` for the tests that never reported, and `, <n> errors` for each failure no test owns — a test binary that crashed or a `harness = false` target that failed, a `unittest` class or module fixture that raised — which the check-ins count among the failures.

Nothing else: no line per passing test, no `unittest` dot stream, no other output a passing test or its child process wrote, no build progress. On success that is about twenty lines whatever the suite's size, plus its `gate time: ` and `node build: ` lines, and the slowest-tests figure is what attributes a gate's time per test for requirements.3. `tests/integration/test_ci_output.py` holds both kinds of suite to this on suites of its own, the `gate time: ` and `node build: ` lines included.

#### requirements.5.2: The cargo suites go through one wrapper

`scripts/run_cargo_suite.py` takes `cargo test`'s arguments, runs `cargo test` with them over libtest's stable output, and prints what requirements.5.1 asks. Both kinds of `ci.yml` step that run a cargo suite, `native watch contract` and each shard's `cargo test`, call it, and so does the `cargo-test` hook, so the hook and the workflow still spell one command, the workflow's with `--shard <k>/<n>` before it ([§AR-ci.13.7](AR-ci.md#137-a-legs-cargo-suite-runs-in-shards), [§AR-ci.1.2](AR-ci.md#12-the-parity-test)). It needs Python's standard library and the toolchain the repository already uses and nothing else: no test runner a contributor must install. It builds the tests before it runs them, as quietly as requirements.5.4 asks, so a compile error is printed in full and a green build prints only its `gate time: build` line.

The focused Windows ownership verification of [§FS-examples.5.6.1](../functional-spec/FS-examples.md#561-one-owner-lasts-through-every-use-of-the-copy) runs the exact
regression directly through cargo/libtest with `--nocapture`. Its raw output and
tested commit remain in the job log so an internal unsupported-symlink skip
cannot be mistaken for executed ownership assertions.

#### requirements.5.3: A hook prints its findings or one line — realized

A pre-commit hook prints its findings when it fails and one line when it passes. Realized: `pre-commit` captures each hook's output, prints it when the hook fails, and prints one `Passed` line otherwise.

#### requirements.5.4: Builds and installs are quiet unless they fail

Every `cargo build`, `cargo run`, `cargo install`, `cargo bench` and `pip install` in `ci.yml` passes `--quiet` (`-q` for pip), the binding oracle's `cargo +1.95.0 build` in `scripts/prepare_binding_oracle.py` included, and so does the `cargo-build` hook, so the hook and the workflow still spell one command. `--quiet` drops the progress lines — `Compiling`, `Downloading`, `Collecting` — and keeps every warning and error, so a failure still shows in full in the step that failed; cargo does not pass it on to a `harness = false` bench, so the benchmark job still prints its report. A suite's own builds and the `git` its fixtures run are held to the same: a green Python gate prints no `Compiling` line and no `LF will be replaced by CRLF` warning.
