# AR-ci-pip-cache: A test's pip install leaves no built wheel in the user's pip cache

A test that builds grund with `pip install` and puts it in a throwaway environment leaves
no wheel it built in pip's cache. That cache is the contributor's, shared with every other project
they have, and nothing in this repository reclaims it, so what a test writes there stays
until a person removes it. [§AR-ci.3.4.5](AR-ci.md#345-the-python-acceptances-shared-build-target) rules out the same accumulation for the Cargo
target, and [§FS-repository-maintenance.2.2](../functional-spec/FS-repository-maintenance.md#22-automatic-reclamation) says of the Node target that contributors need
no separate prune command. This is that rule for a store the repository does not own.

## placement: Where the rule applies

```text
a test ─► pip install <local source> ─► [ throwaway environment ]
                     │
                     └─╳ a built wheel in the user's pip cache
```

The rule binds this repository's test harnesses, in the Python gate and in the manual
rehearsal lane alike. It says nothing of the shipped package, which a person installing
grund may cache as they like. These harnesses are meters outside the shipped CLI and core
([§AR-system.5](README.md#5-what-holds-the-shape)).

## terms: Terms

Leans on [§FS-terms.terms.8](../functional-spec/FS-terms.md#terms8-the-architectures-own-words) (meter).

- **Wheel cache**: where pip files a wheel it built, in the user's cache directory, under a
  key taken from the source's URL.
- **Install that builds grund**: a `pip install` whose source is this repository's Python
  package in source form, so that pip compiles it, rather than a wheel file or a registry
  package.

## 1. The installs it covers

Four `pip install` calls build grund, and each runs with pip's wheel cache off:

- the module's install of `crates/grund-py`, in
  `tests/integration/test_python_binding_acceptance.py` ([§AR-bindings.6.2](AR-bindings.md#62-the-python-acceptance-runs-in-the-gate));
- the clean checkout's install and the unpacked sdist's, in
  `tests/bindings/python/test_build.py` ([§FS-distribution.3.3.7](../functional-spec/FS-distribution.md#337-local-source-and-typing-handoff));
- `PythonSourceTests.install`, in `tests/integration/rehearsal/source_install.py`, of the
  candidate's sdists ([§FS-distribution-candidate.4.2](../functional-spec/FS-distribution-candidate.md#42-python-builds-from-the-sdist-only-when-asked)). It runs only in the manual rehearsal
  lane ([§AR-ci.6](AR-ci.md#6-pgo-stays-out-of-development-ci)), never in the Python gate. The rule reaches it because its builds run in
  this host's own environment rather than `fresh_env`'s, so its cache is the contributor's.

Each passes `--no-cache-dir` on its own command rather than `PIP_NO_CACHE_DIR` in its
environment. pip's nested build-environment install inherits the environment and is not given
the option, so the variable would also turn off the cache that the isolated builds' fetches of
maturin read: measured from cold on a tiny source, on pip 23.2.1 and 26.3, four downloads of its
wheel across the acceptance where the option leaves one, each outside the harness's one retried
fetch ([§AR-bindings.6.2.2](AR-bindings.md#622-every-step-fails-with-its-output-and-the-harnesss-one-fetch-is-retried)).

No other pip call a test makes builds a grund wheel. `rehearsal_support.venv` and the pipx
install in `candidate_install.py` install a wheel file as it stands, under a `fresh_env`
whose cache sits in a throwaway home. `test_build.install_build` fetches the build frontend,
and `python -m build --sdist` writes an sdist and no wheel.

## 2. Why nothing is written, rather than reclaimed

pip files a wheel it built from a local source under a key from the source's URL, when the
source's name looks like `name-version`, which any hyphenated name does. The clean checkout
and the unpacked sdist sit at paths fresh for the run, as does the sdist of a candidate the
lane builds itself, so no later run's key matches and every run adds its wheels, 45 MB each
at the dev profile on Linux. The
module's source path is stable, so it holds one wheel per checkout path, and that wheel
outlives the checkout. A test cannot prune them afterwards: a store shared with the
contributor's other projects is not this repository's to delete from. A build requirement's
download is another matter, which the rule does not reach: it is one entry per file, under a
key every later run shares, so it does not grow with the runs.

Which calls this reaches depends on the pip. pip 26.2 stopped caching a build from a local
directory, and the pips before it, which older interpreters bundle, cache one. Every pip
measured when this was written, 23.2.1 to 26.3, caches a build from a local sdist. A newer
pip therefore narrows the rule's reach and does not retire it.

## 3. What holds it

`tests/integration/test_python_pip_cache.py` holds this without compiling the crate. A child
imports each call site with its runner replaced and reports the options and environment pip
would have got, starting from an environment that holds none of the contributor's `PIP_*`
settings, so neither a `PIP_NO_CACHE_DIR` exported as a workaround nor an exported
`PIP_CACHE_DIR` decides the verdict. A real pip with no index then runs with exactly those
options and that environment on a tiny sdist, with `PIP_CACHE_DIR` naming an empty directory
so that the test reads a cache of its own on every operating system, and must leave no wheel
there. The test names no option: the verdict is what pip did with what the site gave it.

The source is an sdist for every call, the directories included, because on a directory a
pip from 26.2 on would pass whatever the call said. The same sdist with the cache left on
must leave exactly one wheel, so a pip that stopped caching a local sdist fails that control
rather than turning every call green for a reason that has nothing to do with them.
