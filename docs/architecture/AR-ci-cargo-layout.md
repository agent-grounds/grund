# AR-ci-cargo-layout: A test reads only what Cargo promises, so a new stable Cargo cannot break it

The test jobs install whichever stable Rust the runner's toolchain step takes, and
the repository holds no `rust-toolchain` file: [§AR-ci.5](AR-ci.md#5-benchmark-job) pins a compiler only for
the benchmark jobs, and "the test jobs keep tracking stable". So a new Cargo
reaches every pull request with no change of its own, and a test that leans on
something Cargo never promised turns the required check red on all of them at
once. That is how agent-grounds/grund#1377 arrived, and it is
[§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path) failing at the gate rather than in the change under review.

## placement: Where the rule applies

```text
a test ─► reads ─┬─► what Cargo promises      (manifest dir, CARGO_BIN_EXE_*, uplifted binaries,
                 │                              an executable's <target>-<hash> file name)
                 └─╳ Cargo's layout             (deps/ vs build/<pkg>/<hash>/out/,
                                                 build-script-build vs build_script_build)
```

The rule binds every test in this repository that finds a built artifact or reads
Cargo's own output, in any crate, and the suite wrappers under `scripts/`. It says
nothing of the shipped CLI and core, whose behaviour no part of it changes. These
tests are the repository's own test machinery, a meter outside the shipped CLI and
core ([§AR-system.5](README.md#5-what-holds-the-shape)).

## terms: Terms

Leans on [§FS-terms.terms.8](../functional-spec/FS-terms.md#terms8-the-architectures-own-words) (meter).

- **Promised**: what Cargo documents to a test and keeps across releases —
  `CARGO_MANIFEST_DIR`, `CARGO_BIN_EXE_<name>` for the test's own package's
  binaries, the uplifted `<target>/<profile>/<name>` that `cargo build` writes,
  and an executable's file name `<target>-<hash>`.
- **Layout**: where Cargo keeps everything else — a test executable, a build
  script, intermediate artifacts — and how it spells their paths in a `-v` log.
  It changes between releases.
- **Rerun detector**: the part of a test that decides from `cargo build -v`
  whether a build script ran.

Cargo 1.100.0 changed the layout in both places a test here leaned on. A test
executable moved from `target/<profile>/deps/<test>-<hash>` to
`target/<profile>/build/<package>/<hash>/out/<test>-<hash>`, and a build script's
run is printed as `` Running `…/build/<package>/<hash>/out/build_script_build` ``
where 1.99 printed `` Running `…/build/<package>-<hash>/build-script-build` ``.
The uplifted binaries stayed where they were.

## 1. Another package's binary is found without the test executable's own directory

A test that drives another package's binary, as `tests/integration/binaries.rs`
does for `grund` and `grund-lsp`, finds `<target>/<profile>/` under either layout
above and builds the binary there on demand as before, `--release` included. It
never asserts that the running test executable sits in `deps/`; when no layout it
knows matches, it fails naming the test executable's path.

## 2. A build script's run is recognised under every spelling Cargo has used

A rerun detector, as `crates/grund-cli/tests/build_identity_freshness.rs` has for
the rebuild behaviour of [§FS-distribution.4.1.1](../functional-spec/FS-distribution.md#411-a-development-build-names-its-commit), counts a `Running` line that
names the script as `build-script-build` or as `build_script_build`, and states
neither as the one spelling Cargo prints.

## 3. A detector is seen to see before its silence counts

A verdict that the build script did not run is only as good as the detector's
sight: a detector that misses Cargo's line passes every "must not rerun" check
over nothing, and under 1.100.0 the freshness suite's did exactly that. So the
first build of each fixture workspace into an empty target directory, which
always runs the script, must be seen running it; when it is not, the suite fails
at that build, naming the detector rather than any later step. The next spelling
Cargo adopts then fails loudly in both directions instead of turning the "must not
rerun" half green.

Stated so a test can hold the suite to it: **under a Cargo whose spelling the
detector does not know, the freshness suite fails at its first build, and never
passes a "must not rerun" check.**

## 4. The shard wrapper reads only a file name

The shard wrapper of [§AR-ci.13.7](AR-ci.md#137-a-legs-cargo-suite-runs-in-shards) already keeps to this: it reads an executable's
file name off a `Running` line, drops the hash, and reads nothing of its
directory, so both layouts deal the same targets into the same shards.
