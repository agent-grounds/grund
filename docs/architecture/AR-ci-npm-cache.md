# AR-ci-npm-cache: A test's throwaway npm package leaves nothing in the user's npm cache

A test that packs a package, or installs one, under a name no later run can ask for gives npm
a cache that goes away with the run. npm's default cache is the contributor's, shared with
every other project they have, and nothing in this repository reclaims it, so what a test
writes there stays until a person removes it. [§AR-ci.3.4.1.4](AR-ci.md#3414-one-pristine-installed-consumer-per-staged-source) says the consumer installed from
such a tarball is discarded with the run and is no new persistent cache, and
[§FS-repository-maintenance.2.2](../functional-spec/FS-repository-maintenance.md#22-automatic-reclamation) says of the Node target that contributors need no separate
prune command. This is that rule for a store the repository does not own.

## placement: Where the rule applies

```text
a test ─► npm pack, npm install ─► [ a cache that goes with the run ]
                 │
                 └─╳ an entry in the user's npm cache
```

The rule binds this repository's test harnesses, on the Node side of the gate and in the
manual rehearsal lane alike. It says nothing of the shipped package, which a person installing
grund may cache as they like. These harnesses are meters outside the shipped CLI and core
([§AR-system.5](README.md#5-what-holds-the-shape)).

## terms: Terms

Leans on [§FS-terms.terms.8](../functional-spec/FS-terms.md#terms8-the-architectures-own-words) (meter).

- **npm cache**: the `_cacache` directory where npm files what it packs and fetches, under the
  directory its `cache` setting names: by default `~/.npm`, and `%LOCALAPPDATA%\npm-cache` on
  Windows.
- **Throwaway key**: a key in that cache naming something no later run can ask for again: a
  temporary path that has been deleted, or the port of a server that has stopped.

## 1. The calls it covers

Three npm calls file a package under a throwaway key, and each runs with an npm cache in a
temporary directory of its own run:

- `source_package`'s `npm pack --json`, in `tests/bindings/node/support.py`. It files the
  tarball under a `pacote:tarball:file:` key naming the `grund-node-source-*` directory it
  packed in, which is removed when the process exits.
- `installed_consumer`'s `npm install` of that tarball, in the same file ([§AR-ci.3.4.1.4](AR-ci.md#3414-one-pristine-installed-consumer-per-staged-source)). It
  files the tarball again under two more keys naming that directory.
- `consumer`'s `npm install` under `build_env`, in
  `tests/integration/rehearsal/source_install.py` ([§FS-distribution-candidate.4.1](../functional-spec/FS-distribution-candidate.md#41-npm-builds-from-source-only-when-asked)). It runs
  only in the manual rehearsal lane ([§AR-ci.6](AR-ci.md#6-pgo-stays-out-of-development-ci)), never on the Node side of the gate. It fetches
  from `LoopbackNpm`, whose port the kernel picks anew for each process
  ([§FS-distribution-candidate.5.1](../functional-spec/FS-distribution-candidate.md#51-the-rehearsal-holds-no-credential)), so its `make-fetch-happen:request-cache:` keys name a port
  no later run listens on. The rule reaches it because `build_env` is this host's own
  environment rather than `fresh_env`'s, so its cache is the contributor's.

The two calls in `support.py` read one helper for that environment. It starts from the
environment they already run under, because `launch.command` reads `PATH` from a given
environment on Windows and `node_consumer_fixture.py` keeps npm offline through it, and it is
not `dev_env`, because these calls build nothing.

Each cache sits beside `package` and `consumer` in their temporary directories, never inside
them. Inside the stage, a package that lists no `files` packs npm's logs into its tarball, and
a second pack the cache itself; inside the consumer every `Rehearsal`'s copy of the template
would copy the tarball with it. Both directories are already removed at exit, and the
consumer's on a failed install too, so nothing new has to be reclaimed. `build_env`'s cache
takes a `keep` directory, as its Cargo target does.

## 2. The calls it leaves out

- `typescript_compiler`'s `npm install typescript@5.9.3`, in `support.py` ([§AR-ci.3.4.1.3](AR-ci.md#3413-the-typescript-compiler-a-prepared-input)). Its
  keys name the registry's URL, so a later install reads those entries rather than adding to
  them: from a cold cache on npm 10.9.8, a second install added no entry and no blob. It runs
  only on a cold compiler, and a stable key is what a cache is for.
- Every call under `fresh_env`, which is what `npm_install`, `candidate_install.py`,
  `installed_api_parity.py` and the missing toolchain's test run. `fresh_env` names a cache in
  a home of its own, which `keep` removes with the run ([§FS-distribution-candidate.5.2](../functional-spec/FS-distribution-candidate.md#52-every-registry-row-installs-fresh-and-without-rust)).
- `npm run build:source`, in `support.py`, `test_package.py` and `source_install.py`. It runs a
  script, which fetches and files nothing: with the real package built and pack and install
  isolated, no entry is left.

## 3. Why the cache goes away, rather than being kept or reclaimed

No run reads a throwaway key back, so a cache holding one saves nothing, and content addressing
does not rescue it: two builds of one source packed to tarballs of 41988393 and 41988372 bytes,
so each test process that packs the real package adds one blob of about 42 MB and three index
entries. Why the bytes differ does not matter to this rule.

Isolating only the install leaves the pack's entries and the same blob, 2 blobs after two
processes. A cache kept at `node_root() / "npm-cache"` moves the growth without ending it, 6
entries and 2 blobs after two processes. A test cannot prune the contributor's cache afterwards:
a store shared with their other projects is not this repository's to delete from, and an
entry has no owner to select it by. npm has no setting that files nothing, so the one thing left
is to send the cache somewhere that goes away.

## 4. What holds it

`tests/integration/node_npm_cache_fixture.py` holds this, called from
`NodeAcceptanceWorkflowTests` so that it runs on the Node side of the gate selection
([§AR-ci.3.4.3](AR-ci.md#343-selecting-the-binding-acceptances-and-the-slow-tests)), because it needs real npm. Each of its sites runs in a child process that exits
normally before anything is read, since the temporary directories are removed at exit. The
child's every home and scratch directory, `HOME`, `USERPROFILE`, `APPDATA`, `LOCALAPPDATA`,
`XDG_CACHE_HOME`, the temporary directory and `GRUND_NODE_TARGET`, sits under one throwaway
root, and it inherits no `npm_config_*`.

The sites are the real ones with real npm. The real `Rehearsal` reaches the real
`source_package` and `installed_consumer`, with only what compiles stubbed: `node stage.mjs`
writes a tiny package and `npm run build:source` does nothing. The real `consumer` runs under
the real `build_env`, against a real `LoopbackNpm` serving one tiny tarball, with only the
candidate replaced.

The verdict is every `_cacache` under that whole root: no index entry and no content blob. It
does not look only at `~/.npm`, or a cache kept under the Node target root would pass. It names
no setting: what counts is what npm did with what the site gave it. Two controls keep the read
honest: in the same environment, a plain `npm pack` and a plain install from `LoopbackNpm` must
each show up in that read. So a default cache that lies somewhere this read does not reach, or an
npm that stopped caching one of these, fails a control rather than turning a site green.
