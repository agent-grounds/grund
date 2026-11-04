# AR-ci-suite-environment: A suite wrapper hands its tests no repository location

Git tells a hook which repository it runs for through the environment, and a
test that builds a repository of its own must not inherit that. The two suite
wrappers, `scripts/run_python_gate.py` ([§AR-ci.3.4](AR-ci.md#34-the-python-gates-inputs)) and
`scripts/run_cargo_suite.py` ([§AR-ci.requirements.5.2](AR-ci.md#requirements52-the-cargo-suites-go-through-one-wrapper)), take those variables off
before anything they start can see them. A gate that rewrites the contributor's
branch on its way to a verdict damages what it guards, and says nothing about it:
that is [§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path) failing at the gate itself.

## placement: What the wrappers decide

```text
git commit ─► hook environment ─► [ suite wrapper ] ─► tests, and their fixtures' git
                                         │
                                         └─► its own builds and `git rev-parse HEAD`
```

Both wrappers take their copy of the environment from one function in `scripts/`.
This is the repository's own gate, a meter outside the shipped CLI and core
([§AR-system.5](README.md#5-what-holds-the-shape)).

## terms: Terms

Leans on [§FS-terms.terms.8](../functional-spec/FS-terms.md#terms8-the-architectures-own-words) (meter).

- **Repository variable**: a name `git rev-parse --local-env-vars` prints.
- **Hooked worktree**: the linked worktree whose `git commit` is running the hook.

## 1. What comes off

A wrapper starts everything from a copy of its environment without the repository
variables. It asks git for the list rather than keeping one, because the list is
git's and grows with git, and git answers that question whatever the variables
hold, so the run that needs the answer most can still get it.

Where no `git` can be started there is no hook to defend against and nothing to
remove, so the copy is the environment as it stands. The wrapper does not fail for
want of git: [§AR-ci.requirements.5.2](AR-ci.md#requirements52-the-cargo-suites-go-through-one-wrapper) keeps it to the standard library and the
toolchain.

## 2. Why the fixtures do not do it themselves

From a linked worktree, `git commit` gives its hooks
`GIT_DIR=<common>/worktrees/<name>` and an absolute `GIT_INDEX_FILE`. With `GIT_DIR`
set, git does not look for a repository from its working directory, so a fixture's
`git` that names only a working directory acts on the hooked worktree. Its
`git commit` lands on that worktree's branch and reflog. Its `git add -A` replaces
that worktree's index. Its `git init` takes a `GIT_DIR` not named `.git` for a bare
repository and writes `core.bare=true` into the config every checkout shares, after
which git refuses to work in any of them.

A plain repository is spared only by accident: there the hook gets no `GIT_DIR` and
a relative `GIT_INDEX_FILE`, which resolves inside whichever repository a fixture is
in. The checkout layout here is one linked worktree per branch, so the accident
cannot be relied on.

Each fixture could scrub for itself, but that is dozens of sites, and the next
fixture written reopens the leak. The wrappers are the one place every test's
environment passes through. The partial scrubs some tests already carry stay, as
they are harmless.

## 3. What stays

`GIT_AUTHOR_*`, `GIT_EDITOR`, `GIT_EXEC_PATH`, `GIT_CONFIG_GLOBAL` and
`GIT_CONFIG_NOSYSTEM` say who and how, not where, and git does not list them. A
test that needs one of them keeps seeing it.

## 4. Every child a wrapper starts

The copy reaches everything the wrapper runs, not only its tests. In the Python gate
that is the discovery run, both of `inputs()`'s cargo builds and its
`git rev-parse HEAD`: the last has `cwd` at this checkout, so without a `GIT_DIR` it
finds this checkout, and the oracle names this checkout's commit even when the
wrapper's caller named another repository. In the cargo wrapper it is the build, the
listing and the suite, and a shard's runner ([§AR-ci.13.7](AR-ci.md#137-a-legs-cargo-suite-runs-in-shards)) inherits it through cargo.

## 5. Where it is held

`tests/integration/test_suite_environment.py` runs each wrapper with the environment
a hook gets, on a fixture of a main repository and a linked worktree, over a suite of
its own whose test does what a fixture does. The hooked branch, both reflogs, the
worktree's index and the shared config must come out as they went in, and the test's
commit must land in the repository the test made. A second case gives the wrapper
every repository variable, and what each of its children saw must hold none of them
and still hold `GIT_CONFIG_GLOBAL` and `GIT_AUTHOR_NAME`. A third runs `inputs()` with
cargo answered in-process, so its own two builds and the commit it hands the oracle
are checked without building anything.
