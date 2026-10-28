# FS-examples: examples teach canonical user workflows

The `examples/` tree is a user-facing learning surface, not an incidental test-data dump. Its maintained examples explain the canonical ways users are expected to adopt and operate `grund`: choosing an ID scheme, declaring facts, citing them, resolving citations, checking a tree, and using reports during review. This serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible) by making the common path legible before a user reads the implementation or the e2e suite, and it serves [§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work) because agents can learn the intended workflow from maintained, runnable material.

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, home, index), [§FS-terms.terms.2](FS-terms.md#terms2-citations) (citation),
[§FS-terms.terms.3](FS-terms.md#terms3-source-forms) (doc-comment), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (workspace), [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding,
verdict), [§FS-terms.terms.6](FS-terms.md#terms6-rules-and-directions) (rule), and [§FS-terms.terms.7](FS-terms.md#terms7-values-and-integrations) (value, binding, fetcher, snapshot).

## requirements: What every case run holds to

One requirement on the shared runner as a whole, carrying a mark of [§FS-config.requirements](FS-config.md#requirements-what-the-config-contract-holds-to).
That captured output is real and moves with the change that moves it is [§REQ-readme.2](../requirements/REQ-readme.md#2-every-example-is-real) for the
README and [§FS-examples.4](FS-examples.md#4-maintenance-contract) for an example's goldens, so it is not repeated.

### requirements.verdict: A case passes only by matching what it says — realized, directional for the next manifest field

The shared runner never reports a pass for a case it did not run as written. A
`command.args` line a shell would read differently is refused before `grund` runs, never run
as other arguments ([§FS-examples.5.3](FS-examples.md#53-commandargs-is-read-as-shell-words-or-refused)); a malformed `command.external` is refused before any
fixture is copied, and a failed spawn never counts as golden success ([§FS-examples.5.4](FS-examples.md#54-commandexternal-invokes-an-explicit-json-argv)); and
the harness's own wrong-golden probes compare even under an inherited refresh, so a refresh
cannot hide a broken mismatch verdict ([§FS-examples.5.1](FS-examples.md#51-synthetic-verdict-probes-always-compare)). Each refusal names the case. A
manifest field added from here on is read exactly or refused, naming the case, and never
turns a mismatch into a pass. Serves [§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work): an example teaches the workflow
only while its run is true.

## 1. Scope

An example is any maintained directory or document under `examples/` that is advertised from the README, from `examples/README.md`, or from a functional spec. Scratch fixtures, private experiments, and e2e-only test cases are not examples until they are linked from one of those user-facing surfaces.

## 2. Canonical use-cases

Every maintained example must map to at least one canonical `grund` use-case. The use-case must be named in the example's README or manifest in user terms, such as:

- choosing between supported ID schemes;
- setting up a conformant repository;
- writing declarations in Markdown or source doc-comments;
- citing declarations from prose, code, or executable tests;
- resolving a citation with `grund <ID>`;
- finding a declaration's blast radius with `grund refs`;
- grouping citations by file with `grund cover`;
- reviewing a checklist: counting its points with `grund list --glob --total`,
  reading their bodies with `grund show --batch --glob`, and naming the points a
  changed line range belongs to with `grund cover FILE --lines`;
- normalizing citations with `grund fmt`;
- validating a clean or broken tree with `grund check`;
- validating cross-project citations in a workspace;
- citing an external fact, materializing its committed snapshot explicitly,
  and resolving it offline afterward ([§FS-fetch](FS-fetch.md#fs-fetch-grund-materializes-one-external-fact-snapshot)).

An example may cover multiple use-cases, but it must stay small enough that a new user can tell which workflow it is teaching without reading unrelated files.

The examples also include an optional first-class-values repository, specified in [§FS-examples.2.1](FS-examples.md#21-the-values-example).

They include the chapter-rules repository at `examples/rules/`, whose runnable
goldens and documentation coverage are normative in
[§FS-rules.10](FS-rules.md#10-documentation-and-executable-examples).

They include the review-checklist walkthrough at `examples/review-checklist/`, a
multi-run example ([§FS-examples.5.5](FS-examples.md#55-an-example-may-pin-several-runs)) whose runs `1-count`, `2-bodies`, `3-owners`, `4-citers` and `5-cites`
are the review recipe of [§FS-coordinate-globs](FS-coordinate-globs.md#fs-coordinate-globs-a-coordinate-glob-selects-catalog-coordinates-by-pattern), [§FS-refs.globs](FS-refs.md#globs---glob-lists-who-cites-a-set) and [§FS-refs.from](FS-refs.md#from---from-lists-what-one-point-cites), one step each.

They include the downstream-citers walkthrough at `examples/downstream-citers/`,
a multi-run example ([§FS-examples.5.5](FS-examples.md#55-an-example-may-pin-several-runs)) for a project whose citers live in a repository its
own run cannot read. Run `1-cover` is the downstream's `cover --format json`,
which keeps each cross-project alias it cannot resolve ([§FS-cover.2.1](FS-cover.md#21-a-cross-project-citation-counts)); run
`2-check` is the upstream with the committed page of those citations, which
counts as a use like any citing file ([§FS-check.4.1.4](FS-check.md#414-a-citation-anywhere-in-the-resolution-scope-counts)); run `3-deleted` is the same
upstream after a listed point is deleted, refused at the page. The recipe and its
limits are taught where the repository options guide covers workspaces.

They include the opt-in Git co-change workflow at `examples/cochange/`, specified
by [§FS-cochange-recipe.examples](FS-cochange-recipe.md#examples-maintained-walkthrough-tests-and-opt-in-guidance), with commit-msg/CI evidence and visible waivers.

### 2.1 The values example

The examples include an optional first-class-values repository showing `values = true`, Markdown declarations and value sources ([§FS-values.terms](FS-values.md#terms-terms)) — a JSON one and, beside it, a YAML one whose comments say where its numbers came from — exact numeric equality, prose and scanned-comment bindings, direct runtime JSON use, and a caught mismatch. It demonstrates the deliberate unbackticked non-binding without implying interpolation or value inference ([§FS-values](FS-values.md#fs-values-opted-in-kinds-bind-authored-components-to-one-declared-value)).

## 3. Required explanation

Each maintained example must include a detailed explanation for users. At minimum it names the scenario, the intended audience, the files worth opening first, the commands to run from the repository root, the expected exit code and stream behavior, and the lesson the user should take from the output. If the example demonstrates a tradeoff, such as an ID scheme choice, the README must describe both the benefit and the cost in practical terms.

The explanation must be self-contained: a user should not need to inspect `tests/` or implementation code to understand why the example exists or how to run it. Links back to the relevant declaration are allowed, but they supplement the explanation rather than replacing it.

## 4. Maintenance contract

The root `README.md` is the concise public guide to shipped behavior. A functional change that alters the workflow the README advertises ([§REQ-readme.1](../requirements/REQ-readme.md#1-what-it-must-say)) must update `README.md` alongside the most-specific functional declaration, so the advertised workflow and the grounded contract do not drift. Any change that invalidates a README code excerpt or captured command output, functional or not, updates the README in the same change ([§REQ-readme.2](../requirements/REQ-readme.md#2-every-example-is-real)).

Runnable examples must stay executable and regression-tested. If an example has `expected.exit`, `expected.stdout`, and `expected.stderr`, those files are the golden contract for the documented command. A behavior change that affects a canonical workflow must update the corresponding example explanation and golden output in the same change.

When a new canonical workflow becomes part of the README or functional spec, the examples tree must either gain a maintained example for it or explicitly link to an existing example that already teaches it. Removing an example requires either removing the advertised workflow or replacing the example with an equivalent maintained path.

The rules workflow is held as one unit: guide rows, every accepted family and
listed refusal, exact findings and both channels, deterministic shared-anchor
ordering, both deduplication directions, README/index links, skill copies, and
managed rendering move together under [§FS-rules.10](FS-rules.md#10-documentation-and-executable-examples).

## 5. E2E reuse without duplication

Runnable examples must also be executable end-to-end tests. They may have a lighter manifest than `tests/e2e/cases/` when that keeps the user-facing directory readable, but their command invocation, expected exit code, stdout/stderr comparison, mutable-repo handling, and final-repo snapshot comparison must be run by the same test runner logic used for ordinary e2e cases.

The repo must not maintain a second, example-only implementation of the e2e contract. Adding a new e2e capability such as `command.args`, `{repo_copy}`, `expected.repo`, deterministic-output checks, or golden-output refresh must make that capability available to examples through shared code, not through a copied harness.

`examples/rules/expected.*` is run by this shared runner, not a rules-specific
harness ([§FS-rules.10](FS-rules.md#10-documentation-and-executable-examples)).

The external-ticket example runs offline; its contract is [§FS-examples.5.2](FS-examples.md#52-the-external-ticket-example-runs-offline).

How a manifest's `command.args` becomes the arguments `grund` runs with is [§FS-examples.5.3](FS-examples.md#53-commandargs-is-read-as-shell-words-or-refused).

An explicit external command uses [§FS-examples.5.4](FS-examples.md#54-commandexternal-invokes-an-explicit-json-argv) through this same runner.

### 5.1 Synthetic verdict probes always compare

The harness tests its mismatch verdict with scratch cases whose goldens are
deliberately wrong. Those probes are comparison runs even when the process that
invoked the test suite has `UPDATE_EXPECTED` in its environment: inherited
refresh selection must not rewrite the scratch goldens or turn their mismatches
into successful runs. The probe collects every case before deciding and names
every mismatched case and surface, so a caller's refresh workflow cannot mask a
regression in the verdict it relies on.

### 5.2 The external-ticket example runs offline

The external-ticket example uses a deterministic local fake integration. It
must run without network access and its final-repository golden must contain
the exact snapshot declaration the fetcher printed.

### 5.3 `command.args` is read as shell words, or refused

The shared runner turns a case's `command.args` into the arguments it runs
`grund` with, one way for an e2e case and an example alike. It reads the line
as the words a POSIX shell would split it into, honouring the shell's quoting
and nothing else:

- whitespace outside quotes separates arguments, and leading or trailing
  whitespace, the file's final newline included, adds none;
- `'…'` and `"…"` each make what they enclose part of one argument, with the
  quotes removed. Inside one kind of quote the other kind is an ordinary
  character, so `'say "hi"'` and `"it's"` are one argument each;
- quoted and unquoted parts with no whitespace between them are one argument,
  as in a shell: `--rule="a b"` is the single argument `--rule=a b`, and `''`
  or `""` standing alone is an empty argument;
- a backslash is an ordinary character inside single quotes and is refused
  anywhere else. A shell would read it as an escape, and rather than read it
  that way the runner asks for the argument to be quoted;
- an unclosed quote, one still open at the end of the line, is refused;
- no other character means anything. There is no expansion, no comment and no
  operator, so `$`, `` ` ``, `*`, `?`, `~`, `#`, `|`, `;` and every brace other
  than the `{repo}` and `{repo_copy}` placeholders reach `grund` as written.

A refusal is the runner's, not `grund`'s. It fails the case before `grund`
runs, and its message names the case and `command.args` first, the way
`command.cwd` and the `symlinks` manifest name theirs, then says what is wrong
and quotes the line. So every line either becomes the arguments a shell would
give its quoted words, or the runner says why it will not. It is never run as a
different set of arguments, which would leave the author reading `grund`'s
complaint about its argument count against a case written correctly.

### 5.4 `command.external` invokes an explicit JSON argv

A case or example may supply `command.external` instead of `command.args`.
Its content is a nonempty JSON array of strings; its first string is a nonempty
executable. Invoke it directly without a shell, splitting, expansion or Grund
argument interpretation. Literal empty arguments, quotes, spaces and shell
punctuation reach the child unchanged. Both manifests together, malformed JSON,
non-string items, an empty array or empty executable refuse before fixture copying
or spawning; the refusal names the case and manifest(s).

Apply the existing `{repo}` and `{repo_copy}` path placeholders to whole arguments
and their `/suffix` form. `{grund}` as a whole argument denotes the built binary
used by the runner; other braces remain literal. The working-directory default
and `command.cwd`, `command.stdin`, copying, symlink handling, exit/stream/tree
comparison, deterministic reruns and refresh accounting stay shared under
[§FS-examples.5](FS-examples.md#5-e2e-reuse-without-duplication). An external command using `{repo_copy}` is mutating by the same
criterion as `command.args`. Ordinary `command.args` and its default continue
to invoke Grund under [§FS-examples.5.3](FS-examples.md#53-commandargs-is-read-as-shell-words-or-refused) without interpretation changes.

[§FS-cochange-recipe.examples](FS-cochange-recipe.md#examples-maintained-walkthrough-tests-and-opt-in-guidance) uses this seam for its maintained Git walkthrough;
there is no separate example executor. Executable-spawn failures name the case
and external executable and never count as golden success.

### 5.5 An example may pin several runs

An example whose walkthrough has more than one step keeps one run per step in
its own subdirectory, each shaped exactly like a single-run example (`repo/`,
`expected.*`, an optional `command.args`), with the example's `README.md` beside
them. Discovery takes a top-level example directory that holds no
`expected.exit` and runs each direct subdirectory that does, in name order,
through the same shared runner; nothing deeper is searched. An example directory
with neither an `expected.exit` nor such a subdirectory is not a runnable
example, as before. `examples/migrate/` is the first: its two runs are the
preview that refuses and the preview after the repairs it names
([§FS-config-migrate.3.1](FS-config-migrate.md#31-a-refusal-names-a-v1-repair)).

### 5.6 Mutable fixtures belong to one invocation

The shared runner gives each invocation of a mutable case or example its own
repository copy. Two invocations of the same case may overlap; neither may
remove, overwrite, create links in, or inspect the other's copy. A case name
identifies a logical fixture, not ownership of a mutable directory.

#### 5.6.1 One owner lasts through every use of the copy

One invocation owner supplies the actual copy directory to argument expansion
(whole `{repo_copy}` and `/suffix` forms), `command.cwd`, execution, final-tree
comparison and golden refresh. It keeps that directory alive from preparation
through output and tree handling and any intentional post-run inspection. A
post-run reader must retain that owner or inspect while it is alive, rather than
reconstruct a directory from the case name. Releasing the owner cleans up only
its invocation's scratch directory, including on failure or unwind; repeated
runs must not accumulate abandoned copies.

Manifest validation still precedes copying or spawning. Read-only fixtures,
the determinism pass's skip of mutable cases, and explicit unsupported-symlink
skips keep their existing behavior. Retrying link creation, accepting an existing
link, or locking preparation alone does not establish ownership during use.

Native-platform ownership evidence must show that the coordinated regression
actually executed its overlap and logical-path assertions. A successful
unsupported-symlink skip is not ownership evidence. Windows CI retains a focused,
unfiltered invocation of this regression so its executed-test count and explicit
unsupported-symlink notice can be checked at the tested commit.

#### 5.6.2 Runtime uniqueness preserves logical output paths

Unique runtime directory components must not change authored exit, stdout,
stderr or final-tree goldens. At the shared output seam, both comparison and
refresh normalize the exact invocation-owned copy prefix to the existing
logical `target/e2e-work/<case name>/repo` path, including its absolute form
where the output uses an absolute path. Other text and paths remain significant:
normalization must not erase an unrelated path or hide a substantive mismatch.
Examples and ordinary e2e cases use this same normalization.

#### 5.6.3 A materialized fixture is closed before any spawn can inherit it

The shared runner finishes writing every file it materializes into an
invocation's copy, closing it included, before any child process the runner
spawns can inherit a descriptor to that file. A fixture script that a case
executes, directly or through the command under test, therefore never fails
to start as busy (`ETXTBSY`) because a sibling invocation's spawn carried the
runner's write descriptor. Retrying the case or the spawn on that error does
not meet this point: it hides the overlap rather than removing it.

##### 5.6.3.1 Its regression counts attempts, never time

The regression that holds this point copies an executable fixture with the
runner's own copy, starts it at once, and repeats that a fixed number of times
while sibling invocations spawn through the runner's own spawn. Every attempt
runs to completion, and the regression fails only when an attempt could not
start as busy. How many attempts fit within a time is never part of whether it
passes: that number measures the machine rather than the overlap, so a slower
runner would fail it with no attempt busy. The number of attempts is a constant
of the regression, not a reading of the clock, and is large enough that a runner
which lets a sibling spawn inherit the copy's write descriptor fails well within
it. Lowering the number while keeping a time limit does not meet this point: it
moves the speed at which the regression fails rather than removing it.
