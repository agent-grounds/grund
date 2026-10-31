# Native Node-cache measurement for #1122

Implements [§AR-ci.requirements.1.verification](../docs/architecture/AR-ci.md#requirements1verification-a-cost-correction-measures-comparable-full-gates)
and the isolated-experiment boundary of [§AR-ci.3.4.1.5](../docs/architecture/AR-ci.md#3415-a-dependency-only-publication-view-with-the-live-root-intact).
Fix round 4 recovers the cold metadata diagnostic and verifies the focused
publication correction locally. R3-03's three-OS hosted validation and paired
elapsed-time acceptance remain outstanding; no performance pass is claimed.

## Scope and transport

Checkpoint 6 authorizes temporary cache transactions only in
`vjovanov/grund-fork`, on existing dispatch ref `fix/issue-1122`, with explicit
`experiment: grund-1122` configuration. This supersedes the earlier blanket
prohibition on experimental branch caches. Production guards/keys are unchanged.
Every service key begins `grund-1122-experiment-`, then identifies run, attempt,
OS, full source SHA, operation and cache selection. Nothing restores production
prefixes. The save-path condition requests a new experimental primary, then
permits only its recorded seed's exact key as the fallback; the service's returned
key is checked **before downloading/extracting**. The record distinguishes this
controlled primary miss/seed fallback from a production cache hit.

The previously demonstrated registered `pre-release-checks.yml` dispatch still
calls `node-cache-measure.yml` at the dispatch ref. Empty input preserves ordinary
release checks. The reusable workflow positively checks fork/ref/mode/source/
harness/native OS, and requires `actions: write` solely for exact cache cleanup.
It needs no new workflow registration, branch or repository setting. Its new
service path still needs hosted validation; a registered route alone is no pass.

Archive artifacts are no longer experimental cache transport. The pinned
`@actions/cache` 5.1.0 toolkit calls the actual v2 service, using the production
native glob/tar/zstd implementation and Node 24 runtime from `github-script@v8`.
The production [cache v5 package](https://github.com/actions/cache/blob/v5/package.json)
uses that toolkit series; its [action runtime](https://github.com/actions/cache/blob/v5/action.yml)
is Node 24. The committed npm lock fixes the independent harness's transitive
dependencies. Record resolved action commits from raw job setup logs too; stop
if a moved production action or toolchain makes this equivalence invalid.

The wrapper records complete toolkit setup/restore/save times and service
lookup/create/upload/finalize/download/extract phases, beginning before the first
transfer. It hashes the **actual uploaded/downloaded archive**, verifies seed bytes
before extraction and checks the entire restored file manifest. No signed URLs
or runtime credentials enter evidence. Instrumentation, checksum scans, REST
queries, evidence uploads, cleanup and checkout post actions all stay charged in
the full job; no calibration of the unused artifact transport is necessary.

## Immutable inputs and focused verification

The candidate includes both collision histories in the existing real-Cargo
fixture: simultaneous workspace/external names and stale workspace output whose
name **and version** become an external package's identity at another source.
Both workflow copies, ordinary Cargo hit/miss, current/mixed toolchains,
dev/release, renamed library, nested targets, exact excluded bytes, third-party
freshness and live/owner preservation remain covered. The candidate's fixture
cost is part of every complete candidate gate; do not substitute the older suite.

```bash
base=d338c4814a095fef54708e83367597c4e7955aad
candidate=6fe17f9d51b666758df0d75f66ea689cffdc434b
# The final fix-4 result records this separate, complete harness SHA literally.
harness=$(git log -1 --format=%H -- scripts/node-cache-measurement.md)
gh api repos/vjovanov/grund-fork/git/ref/heads/fix/issue-1122 --jq .object.sha
mkdir -p /var/tmp/grund-1122-tmp
```

The candidate also includes the offline ledger safety tests, so their macOS
suite cost stays in the source measurements. The fork ref must equal `$harness`
and the fix-4 result. Source and harness are
separate identities; source builds are checked against the pinned source after
checkout. Run focused regressions downstream before spending native repetitions:

```bash
TMPDIR=/var/tmp/grund-1122-tmp PYTHONDONTWRITEBYTECODE=1 RUSTUP_TOOLCHAIN=1.99.1 \
  python3 -m unittest discover -s tests/integration -p test_node_binding_acceptance.py -k publishes_dependencies -v
TMPDIR=/var/tmp/grund-1122-tmp PYTHONDONTWRITEBYTECODE=1 \
  python3 -m unittest discover -s tests/integration -p test_node_cache_experiment_ledger.py -v
```

The whole local gate belongs to green. Workflow regeneration is
`PYTHONDONTWRITEBYTECODE=1 python3 scripts/node_cache_measure_workflow.py`
(PyYAML 6.0.2). Compare the generated steps to production: measured seed/exact/save
jobs keep every source command, including all four macOS Python selections,
Rust shards, watch proof, builds and self-check. Only source checkout, experimental
transport/identities and experimental publication conditions change. `validate`
is an explicitly separate transaction preflight, never a full-gate sample.

## Cold publication regression and continuation evidence

The previous Linux candidate seed failed because `--all-features` requested
`bincode 1.3.3` from the unused CLI benchmark chain while publication was offline.
A local cold reconstruction on source `b534ccbbadbd995998d0d39bf9ce57f894f9c4ae`
matches all 194 native downloaded crate archives byte for byte and recovers that
diagnostic. The correction is specified by
[§AR-node-cache-metadata.resolution](../docs/architecture/AR-node-cache-metadata.md#resolution-locked-and-offline-with-observed-workspace-features) and [§AR-node-cache-metadata.failures](../docs/architecture/AR-node-cache-metadata.md#failures-cargo-diagnostics-explain-an-unready-publication).
It keeps locked/offline resolution and source-qualified artifact identity.

The focused publication command above exercises an unused optional registry
archive that remains absent, a compiled optional dependency that must remain
fresh after restore, and diagnostic/failure cleanup. For its independent negative
control, use a disposable checkout of the new candidate with only the production
helper replaced by the previous immutable candidate's helper:

```bash
negative="$HOME/ag/tmp/grund-1122-review4-negative"
git worktree add --detach "$negative" "$candidate"
git show b534ccbbadbd995998d0d39bf9ce57f894f9c4ae:scripts/node_cache_publication.py \
  > "$negative/scripts/node_cache_publication.py"
(cd "$negative" && TMPDIR=/var/tmp/grund-1122-tmp PYTHONDONTWRITEBYTECODE=1 \
  RUSTUP_TOOLCHAIN=1.99.1 python3 -m unittest discover -s tests/integration \
  -p test_node_binding_acceptance.py -k publishes_dependencies -v)
```

Expect exit 1 with eight unready-publication subcases; the corrected candidate
must pass all four methods with their original semantic assertions. This is a
mechanism regression, not native time evidence. Full cold seeds below run the
unchanged source/fallback/CLI proof and all source suites on each native OS.

Preserve the failed `runtime/native/` evidence and independent absence journals.
Use a fresh `runtime/native-fix4/` continuation set for successful analysis inputs;
retain failed continuation runs separately with complete collection and recovery.
Do not mix old-source or failed seeds into the successful analyzer input. All
seeds and samples must use the new source/harness pair, including the new fixture
cost. The transaction, cleanup and performance criteria below are unchanged.

The retained harness derives complete Node/macOS jobs from production workflows
and measures actual cache-service transactions, provenance and exact-entry
recovery. That gives subsequent Node-cache cost corrections the same executable
full-gate comparison and cleanup procedure, instead of reconstructing this
experiment from logs. It is scoped infrastructure, not a general benchmark:
future comparisons must explicitly repin sources and authorization boundaries.
Retention and that limited repository purpose must be stated in the PR description.

## Workflow context check

After regeneration, use `actionlint` 1.7.12's
[context-availability check](https://github.com/rhysd/actionlint/blob/v1.7.12/docs/checks.md#availability-of-contexts-and-special-functions)
on both the reusable workflow and its dispatch caller:

```bash
actionlint -shellcheck= -pyflakes= \
  .github/workflows/node-cache-measure.yml .github/workflows/pre-release-checks.yml
```

This is a static workflow check, independent of the measured source suites.
It must exit 0. As a regression witness, the same linter rejects the original
harness at both reported lines (159 and 374), with exit 1:

```bash
git show c2ef6cacd66f15cc1ca57b3b8ff4a72a9710d333:.github/workflows/node-cache-measure.yml \
  | actionlint -shellcheck= -pyflakes= -
```

GitHub's [context availability](https://docs.github.com/en/actions/reference/workflows-and-actions/contexts#context-availability)
excludes `runner` from job-level `env`. The existing preservation step now derives
the tools directory from `RUNNER_TEMP`, copies the pinned harness there, then
exports that exact path through the UTF-8
[`GITHUB_ENV` file](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-commands#setting-an-environment-variable)
before source checkout. Forward slashes from `Path.as_posix()` support native
Windows Python, npm, Git Bash and JavaScript children. Later shell steps use
`env.MEASURE_TOOLS`; the JavaScript action and its children inherit the same
`MEASURE_TOOLS` value. There is no job-level runner expression or second tools-path
calculation. The static check is not hosted dispatch, native transaction, recovery
or performance evidence; establish those below before repeated samples.

## Dispatch, wait and collect

Use authenticated `gh` with fork Actions read/write/dispatch permissions, cache
service availability and native hosted capacity. The fix state has not exercised
those operations. Start a fresh session; never rerun an old seed attempt in place.

```bash
session="fix4-$(date -u +%Y%m%dT%H%M%SZ)"
evidence="$HOME/ag/tmp/grund-1122-native-$session"
mkdir -p "$evidence"
dispatch_one() {
  local label="$1" os="$2" source="$3" condition="$4" seed="${5:-}" id=""
  printf '%s\n' "$label" >> "$evidence/labels.txt"
  PYTHONDONTWRITEBYTECODE=1 python3 scripts/node_cache_measure.py dispatch \
    --source "$source" --harness "$harness" --os "$os" --condition "$condition" \
    --seed-run "$seed" --label "$label" || return
  for attempt in $(seq 1 30); do
    id=$(gh run list --repo vjovanov/grund-fork --workflow pre-release-checks.yml \
      --branch fix/issue-1122 --event workflow_dispatch --limit 100 \
      --json databaseId,displayTitle --jq ".[] | select(.displayTitle == \"Node cache 1122 $label\") | .databaseId")
    [ -z "$id" ] || break
    sleep 2
  done
  [[ "$id" =~ ^[0-9]+$ ]] || return 1
  printf '%s\n' "$id" >> "$evidence/runs.txt"
  if ! gh run watch "$id" --repo vjovanov/grund-fork --exit-status >&2; then
    python3 scripts/node_cache_measure.py collect "$id" --out "$evidence/$id" >&2
    python3 scripts/node_cache_measure.py cleanup "$id" --out "$evidence/$id/recovery" >&2
    return 1
  fi
  python3 scripts/node_cache_measure.py collect "$id" --out "$evidence/$id" >&2 || return
  printf '%s\n' "$id"
}
```

Process **one OS at a time**, serially. This bounds retained seeds to six entries;
each validation trial retains one additional entry, and each full save batch at
most three. The reusable workflow serializes experiment runs; do not queue
multiple pending dispatches. Before reservation, the wrapper checks total fork
occupancy plus actual archive bytes below 8 GiB (2 GiB headroom) and at most nine
experiment entries. It refuses instead of deleting other caches. If capacity
refuses or images/toolchains change, collect/recover this OS's recorded entries
and report the concrete problem before continuing. No unmeasured eviction or
input substitution is permitted.

For **each** `os=Linux`, `os=Windows`, then `os=macOS`, run this block. Both seeds
execute complete native gates cold and save all three selections using the same
source's actual representation. Keep registry/ordinary Cargo and TypeScript
seed contents comparable across sides; verify their manifests and explain any
differences before sampling. Node representation differs by the intended policy.

```bash
os=Linux # repeat the complete block with Windows, then macOS
seed_base=$(dispatch_one "$session-$os-base-seed" "$os" "$base" seed) || exit 1
seed_candidate=$(dispatch_one "$session-$os-candidate-seed" "$os" "$candidate" seed) || exit 1
validation_base=$(dispatch_one "$session-$os-base-validate" "$os" "$base" validate "$seed_base") || exit 1
validation_candidate=$(dispatch_one "$session-$os-candidate-validate" "$os" "$candidate" validate "$seed_candidate") || exit 1
python3 scripts/node_cache_measure.py analyze "$evidence" --preflight --os "$os" --out "$evidence/$os-preflight.json"
```

Before continuing, inspect both validation runs' native environment and raw
transaction records: three save/clear/restore trials for **each** real Cargo,
Node and TypeScript selection. Each restored archive must equal its uploaded
hash/size and every restored file must match. Each trial deletes its exact entry
before the next. Record per-OS/side/selection/operation means, range and standard
deviation, using an empirical envelope at least the range or twice the sample
standard deviation. Large variation needs more trials/new dispatches before
claiming an envelope. Missing phases, failed/mismatched bytes, scope or cleanup
failures invalidate the route: stop before full-gate repetitions. No performance
success follows merely from a successful validation job.

After independent route/seed acceptance, alternate sides for each condition:

```bash
for condition in exact save; do
  for sample in 1 2; do
    dispatch_one "$session-$os-base-$condition-$sample" "$os" "$base" "$condition" "$seed_base" || exit 1
    dispatch_one "$session-$os-candidate-$condition-$sample" "$os" "$candidate" "$condition" "$seed_candidate" || exit 1
  done
done
python3 scripts/node_cache_measure.py cleanup "$seed_base" --out "$evidence/$seed_base/recovery"
python3 scripts/node_cache_measure.py cleanup "$seed_candidate" --out "$evidence/$seed_candidate/recovery"
```

Repeat the whole OS block only after its exact-entry absence checks pass.
Three OS blocks yield at least **24 successful measured jobs**, apart from seeds
and validation. Exact restores source-matched warm inputs and skips publication;
save restores the same seed through a controlled primary miss/fallback and
executes all three preparation/save paths. This deliberately conservative
save condition includes ordinary Cargo pruning and TypeScript publication.
Declare weighting before sampling: default 5/8 exact, 3/8 save, reflecting the
preserved historical eight-job split. Report both condition means separately.

## Analysis and cancellation recovery

```bash
python3 scripts/node_cache_measure.py analyze "$evidence" --out "$evidence/analysis.json"
```

Analysis requires two complete jobs per side/OS/condition and three validation
transactions per side/OS/selection/operation. It retains full-job and individual
step means/spread, source/harness/run/job URLs, and candidate weighted margin
below 900 seconds after adding empirical job and transaction envelopes. A useful
margin is at least 30 seconds here; this empirical screen is **not** a statistical
confidence interval. Inspect logs for unchanged proof/suite counts, explain
seconds added/removed by mechanism, and collect more samples if variance or seed
differences obscure the result. Failed checks or inadequate margin mean
inconclusive/insufficient evidence, never permission to drop fixture or proof cost.

Every writer first uploads `OS-attempt-ledger` with all intended exact keys.
Evidence includes actual finalized ID/ref/version, requested/matched keys, file
and archive checksums, native versions/image/path, phase times and complete forge
step records/logs. Successful save/validation batches clean locally; successful
seeds stay only until their paired samples finish. Recovery reconciles an
interrupted finalization using **only** planned exact keys/ref, writes remote
identities before deletion, and verifies absence. Unfinalized service reservations
have no visible REST cache ID; their intended keys remain recorded, and a later
recovery detects any finalized entry.

After any partial failure, cancellation or client interruption, use `runs.txt`
(or find the unique label in `labels.txt`). A runner's `always()` step cannot
cover a vanished runner. This command runs independently on the operator machine;
it requires a terminal run, saves full logs/jobs and the remote intention ledger
before deletion, and deletes only matching exact IDs:

```bash
run_id=REPLACE_WITH_RECORDED_RUN_ID
gh run cancel "$run_id" --repo vjovanov/grund-fork # only if still running
gh run watch "$run_id" --repo vjovanov/grund-fork || true
python3 scripts/node_cache_measure.py cleanup "$run_id" --out "$evidence/$run_id/recovery"
```

Do this for both seed runs too, even when a later sample failed. Check every
`remaining.json` is empty. Never blanket-delete, edit production policy, write
upstream refs/caches, change the meter/window/threshold or push a default branch.
Retain this complete evidence under the workspace's `runtime/` before scratch
cleanup. The supervisor owns scheduling and acceptance; this handoff claims no
native measurements or elapsed-time result.
