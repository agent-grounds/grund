# AR-ci-fixture-cleanup: A fixture's cleanup failure never aborts its test binary

A test fixture removes the tree it created when it is dropped ([§AR-ci.10.2](AR-ci.md#102-the-root-is-claimed-not-adopted)), and
a fixture that fails its test when that removal fails is right to. What it must
never do is raise that failure while its thread is already panicking. Rust
aborts a whole process on a panic during unwinding, so one failed test whose
tree is held open takes every test its binary has not yet reported with it.
On windows-latest the abort leaves an exit code and nothing else. The gate then
shows a process that died with no reason given, and a rerun passes. That is
how agent-grounds/grund#1226 arrived, and it is [§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path) failing
at the gate the way [§AR-ci.10](AR-ci.md#10-a-test-fixture-owns-the-tree-it-scans) describes.

## placement: Where the rule applies

```text
a test fails ─► unwinding ─► [ fixture Drop ] ─► removal fails
                                    │
                                    ├─► stderr, beside the first panic   (unwinding)
                                    └─╳ a second panic: the binary aborts
```

The rule binds every fixture `Drop` in this repository's Rust tests that removes
a tree, in any crate. It says nothing of the shipped CLI and core, whose
behaviour no part of it changes. These fixtures are the repository's own test
machinery, a meter outside the shipped CLI and core ([§AR-system.5](README.md#5-what-holds-the-shape)).

## terms: Terms

Leans on [§FS-terms.terms.8](../functional-spec/FS-terms.md#terms8-the-architectures-own-words) (meter).

- **Fixture Drop**: an `impl Drop` in a test that removes the tree its fixture
  created.
- **Unwinding**: the state in which the thread running the `Drop` is already
  panicking, as `std::thread::panicking()` reports, because the test that holds
  the fixture has failed.
- **Passing path**: a `Drop` that runs while its thread is not panicking, at the
  end of a test that has not failed.

## 1. A failure while unwinding is written, not raised

`libtest` runs a binary's tests as threads of one process. A panic raised by a
destructor while its thread unwinds is not a second test failure: Rust aborts
the process with "panic in a destructor during cleanup", before libtest reports
the failing test or runs the next one. On Windows MSVC the abort is a fast-fail,
exit `0xC0000409` (`STATUS_STACK_BUFFER_OVERRUN`), and libtest never flushes the
output it captured, so the first failure's message is lost as well. The removal
failing in the first place is ordinary on Windows, where another process may
still hold a handle in the tree.

So a fixture `Drop` that is unwinding writes the removal error, naming the tree,
to stderr and raises nothing. The test is already failing, its own panic is what
it reports, and the error beside it says what was left behind.

Stated so a test can hold a fixture to it: **a test that fails while its fixture
cannot be removed is reported as one failed test with its own panic, and the
binary runs on.**

## 2. The passing path stays strict

Nothing here makes cleanup more lenient. A fixture `Drop` that fails a passing
test when its tree cannot be removed still does, and the removal is not ignored,
retried or put off until it succeeds, for the reason [§AR-ci.10.4](AR-ci.md#104-git-descendants-stay-inside-the-fixture-lifetime) gives. Only the
second panic goes, and only while the thread is unwinding.

Stated so a test can hold a fixture to it: **a passing test whose fixture cannot
be removed still fails, with the removal error.**

## 3. One helper decides

The decision is made once ([§AR-system.principle](README.md#principle-one-fact-one-point-one-implementation)), by the helper
`remove_fixture_tree` in the CLI tests' shared support, which a fixture `Drop`
calls rather than restating. It removes the tree and, if the removal fails,
writes the error to stderr while unwinding and panics otherwise. A fixture with a
passing-path rule of its own makes the same decision inline. The copy of
`crates/grund-cli/tests/support/case_copy.rs` is one: it tolerates a tree that
is already gone, so it tests `std::thread::panicking()` itself before it raises.

## 4. What holds it

`tests/integration/test_fixture_cleanup.py` reads every `impl Drop` under
`crates/` whose body removes a tree, either by naming `remove_dir_all` or by
calling `remove_fixture_tree`. It holds each to one of three shapes. The `Drop`
raises nothing, it calls the helper, or it tests `std::thread::panicking()`
before the first thing it raises. It also holds every definition of the helper
to both points. The helper tests `std::thread::panicking()` before anything it
raises, writes with `eprintln!` while unwinding, and still raises on the passing
path. Strings and comments are blanked before the source is read, so a word
inside a message decides nothing. A `Drop` that removes no tree, such as a child
process's or a mutex's, is outside the rule.

That check reads source. At run time, the `rules_contract` binary's
`offer_scope::cleanup` holds the reported fixture, `Configured`, to
[§AR-ci-fixture-cleanup.1](AR-ci-fixture-cleanup.md#1-a-failure-while-unwinding-is-written-not-raised) and [§AR-ci-fixture-cleanup.2](AR-ci-fixture-cleanup.md#2-the-passing-path-stays-strict). It takes write permission
from a directory inside a real fixture root on a thread of its own, so the
removal fails without touching anything outside that root. It also checks that
the tree is still there afterwards, so a removal that quietly succeeded cannot
pass for one that failed.
