# Git co-change evidence

This optional workflow implements
[§FS-cochange-recipe.examples](../../docs/functional-spec/FS-cochange-recipe.md#examples-maintained-walkthrough-tests-and-opt-in-guidance).
It compares exact tracked Git trees and reports related spec/test edits for
one shared directly cited declaration. Copy all Python modules and `policy.json`
together; no third-party Python dependency is required. Use Python 3.11+, Git
2.32+ and Grund 0.16.1 (or a compatible newer build).

```bash
cargo install grund --version 0.16.1 --locked
python3 examples/cochange/walkthrough.py --grund "$(command -v grund)"
```

The walkthrough creates disposable Git repositories under `~/ag/tmp`, using
`repo/` as its template. It prints summaries of actual recipe JSON results:
missing grounding, absent/spec-only/test-only evidence, unrelated and split
targets, complete evidence, a fix to an unchanged contract, and a refactor.
[expected.stdout](expected.stdout) captures that output. The complete recipe
reports also contain exact tree/commit identities and deterministic error details.
The presentation omits those identities so the comparison is easy to read.

`command.external` runs this walkthrough through the shared example runner with
the build's `{grund}` binary. `expected.exit`, `expected.stdout` and
`expected.stderr` are its maintained goldens, refreshed with the existing
`UPDATE_EXPECTED` mechanism. No separate executor compares example output.

For your repository, use the [hook/CI guide](../../docs/user-facing/cochange.md),
which explains explicit new/amend/root bases, whole-PR comparisons, merge and
squash messages, runtime setup, classification and reason-bearing trailers.
Grund's own contributions do not acquire this gate.

Both spec and test edits must match one target's project and declaration root;
duplicate bare IDs in other members remain distinct. A waiver can only excuse
named missing edit obligations on exact commit-local paths. Ordinary Grund
errors and missing grounding always remain failures. This is file-level related
edit evidence: unrelated edits inside matched files may pass, valid unchanged
contracts can require waivers, and it proves no changed-line coverage, semantic
correctness or test execution.

The existing Python gate installs a separate released 0.16.1 binary and builds
this checkout before exercising fresh synthetic Git clones against both:

```bash
python scripts/run_python_gate.py
```

To reuse artifacts, set `GRUND_BUILT` and `GRUND_RELEASED` to absolute binary
paths; the release version is checked and setup errors do not skip compatibility.
For only the co-change contract, install the release to a separate prefix and run
the committed acceptance entry point:

```bash
cargo install grund --version 0.16.1 --locked --root "$HOME/ag/tmp/grund-release" \
  --target-dir "$HOME/ag/tmp/grund-release-build"
python tests/e2e/run_cochange_contract.py \
  --released "$HOME/ag/tmp/grund-release/bin/grund" \
  --target-dir "$HOME/ag/tmp/grund-cochange-target"
```
