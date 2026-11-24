# FS-examples: examples teach canonical user workflows

The `examples/` tree is a user-facing learning surface, not a dump of test data: each
maintained example teaches a canonical way to adopt and operate `grund`, and each runnable
one also runs as an end-to-end test, so what it teaches stays true. This serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible) by
making the common path legible before a user reads the implementation or the e2e suite, and
[§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work) because agents learn the intended workflow from maintained, runnable
material.

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, home, index), [§FS-terms.terms.2](FS-terms.md#terms2-citations) (citation),
[§FS-terms.terms.3](FS-terms.md#terms3-source-forms) (doc-comment), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (workspace), [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding,
verdict), [§FS-terms.terms.6](FS-terms.md#terms6-rules-and-directions) (rule), and [§FS-terms.terms.7](FS-terms.md#terms7-values-and-integrations) (value, binding, fetcher, snapshot).

## requirements: What every case run holds to

That captured output stays true is a maintenance contract, not a runner requirement
([§FS-examples.4](FS-examples.md#4-maintenance-contract)).

### requirements.verdict: A case passes only by matching what it says

The shared runner never reports a pass for a case it did not run as written. A
`command.args` line a shell would read differently is refused before `grund` runs, never run
as other arguments ([§FS-examples.5.3](FS-examples.md#53-commandargs-is-read-as-shell-words-or-refused)); a malformed `command.external` is refused before any
fixture is copied, and a failed spawn never counts as golden success ([§FS-examples.5.4](FS-examples.md#54-commandexternal-invokes-an-explicit-json-argv)); and
the harness's own wrong-golden probes compare even under an inherited refresh, so a refresh
cannot hide a broken mismatch verdict ([§FS-examples.5.1](FS-examples.md#51-synthetic-verdict-probes-always-compare)). Each refusal names the case. A
manifest field the runner gains is held to the same: read exactly or refused, naming the
case, and never turning a mismatch into a pass. Serves [§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work): an example
teaches the workflow only while its run is true.

## 1. Scope

An example is any maintained directory or document under `examples/` that is advertised from
the README, from `examples/README.md`, or from a functional spec.

## 2. Canonical use-cases

Every maintained example maps to at least one canonical `grund` use-case and names it in user
terms in its README or manifest. The canonical use-cases include:

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

An example may cover several use-cases, but stays small enough that a new user can tell which
workflow it teaches without reading unrelated files. Some examples carry obligations of their
own:

- **`examples/values/`**, the optional first-class-values repository, shows `values = true`,
  Markdown declarations and value sources ([§FS-values.terms](FS-values.md#terms-terms)) — a JSON one and, beside it, a
  YAML one whose comments say where its numbers came from — exact numeric equality, prose and
  scanned-comment bindings, direct runtime JSON use, and a caught mismatch. It demonstrates
  the deliberate unbackticked non-binding without implying interpolation or value inference
  ([§FS-values](FS-values.md#fs-values-opted-in-kinds-bind-authored-components-to-one-declared-value)).
- **`examples/external-tickets/`** uses a deterministic local fake integration, so it runs
  without network access, and its final-repository golden holds the exact snapshot
  declaration the fetcher printed.
- **`examples/rules/`**, the chapter-rules repository, has its runnable goldens and
  documentation coverage specified by [§FS-rules.10](FS-rules.md#10-documentation-and-executable-examples).
- **`examples/review-checklist/`** is a multi-run example ([§FS-examples.5.5](FS-examples.md#55-an-example-may-pin-several-runs)) whose runs,
  listed in its README, are the review recipe of [§FS-coordinate-globs](FS-coordinate-globs.md#fs-coordinate-globs-a-coordinate-glob-selects-catalog-coordinates-by-pattern), [§FS-refs.globs](FS-refs.md#globs---glob-lists-who-cites-a-set) and
  [§FS-refs.from](FS-refs.md#from---from-lists-what-one-point-cites).
- **`examples/downstream-citers/`** is a multi-run example ([§FS-examples.5.5](FS-examples.md#55-an-example-may-pin-several-runs)) for a project
  whose citers live in a repository its own run cannot read: the downstream's
  `cover --format json` keeps each cross-project alias it cannot resolve ([§FS-cover.2](FS-cover.md#2-behaviour)); the
  upstream with the committed page of those citations counts each as a use ([§FS-check.4.1](FS-check.md#41-unused-declaration)); and
  the same upstream after a listed point is deleted is refused at the page. The repository options guide
  teaches the recipe and its limits where it covers workspaces.
- **`examples/cochange/`**, the opt-in Git co-change workflow, is specified by
  [§FS-cochange-recipe.examples](FS-cochange-recipe.md#examples-maintained-walkthrough-tests-and-opt-in-guidance).

## 3. Required explanation

Each maintained example carries a detailed explanation for users, which names at least the
scenario, the intended audience, the files worth opening first, the commands to run from the
repository root, the expected exit code and stream behavior, and the lesson to take from the
output. Where the example demonstrates a tradeoff, such as an ID scheme choice, its README
describes both the benefit and the cost in practical terms. The explanation is
self-contained: a user needs neither `tests/` nor the implementation to see why the example
exists or how to run it, and a link back to the relevant declaration supplements the
explanation rather than replacing it.

## 4. Maintenance contract

The root `README.md` is the concise public guide to shipped behavior. A functional change that
alters the workflow it advertises ([§REQ-readme.1](../requirements/REQ-readme.md#1-what-it-must-say)) updates it alongside the most-specific
functional declaration, and any change, functional or not, that invalidates one of its code
excerpts or captured command outputs updates it in the same change, so they stay real
([§REQ-readme.2](../requirements/REQ-readme.md#2-every-example-is-real)).

Where an example has `expected.exit`, `expected.stdout` and `expected.stderr`, those files
are the golden contract for the documented command. A behavior change that affects a
canonical workflow updates the corresponding example's explanation and golden output in the
same change. A canonical workflow added to the README or the functional spec gains a
maintained example, or the examples tree explicitly links it to an existing example that
already teaches it; an example is removed only with the advertised workflow it teaches, or
replaced by an equivalent maintained path.

## 5. E2E reuse without duplication

Every runnable example is also an end-to-end test. Its manifest may be lighter than a
`tests/e2e/cases/` case's where that keeps the directory readable, but its command
invocation, expected exit code, stdout and stderr comparison, mutable-repository handling and
final-tree comparison (`expected.repo/`) run through the same runner logic as an ordinary
case. There is no second, example-only implementation of that contract: every e2e capability,
golden refresh and the deterministic-output check included, reaches examples through the
shared code, never through a copied harness.

### 5.1 Synthetic verdict probes always compare

The harness tests its mismatch verdict with scratch cases whose goldens are deliberately
wrong. Those probes compare even when the process that invoked the suite has
`UPDATE_EXPECTED` in its environment, so an inherited refresh never rewrites their goldens or
turns their mismatches into passes. The probe collects every case before deciding and names
every mismatched case and surface.

### 5.3 `command.args` is read as shell words, or refused

The shared runner reads a case's `command.args` as the words a POSIX shell would split it
into, honouring the shell's quoting and nothing else:

- whitespace outside quotes separates arguments, and leading or trailing whitespace, the
  file's final newline included, adds none;
- `'…'` and `"…"` each make what they enclose part of one argument, with the quotes removed,
  and inside one kind of quote the other kind is an ordinary character;
- quoted and unquoted parts with no whitespace between them are one argument, so
  `--rule="a b"` is the single argument `--rule=a b`, and `''` or `""` standing alone is an
  empty argument;
- a backslash is an ordinary character inside single quotes and is refused anywhere else,
  asking for the argument to be quoted;
- an unclosed quote is refused;
- no other character means anything: there is no expansion, comment or operator, so `$`,
  `` ` ``, `*`, `?`, `~`, `#`, `|`, `;` and braces reach `grund` as written, except that the
  `{repo}` and `{repo_copy}` placeholders expand where one is a whole argument or begins one
  followed by `/`, as in `{repo_copy}/docs`, and nowhere else.

A refusal is the runner's, not `grund`'s. It fails the case before `grund` runs, and its
message names the case and `command.args` first, then says what is wrong and quotes the
line. So a line becomes the arguments a shell would give its quoted words, or is refused; it
is never run as other arguments.

### 5.4 `command.external` invokes an explicit JSON argv

A case or example may supply `command.external` in place of `command.args`: a nonempty JSON
array of strings whose first string is a nonempty executable. The runner invokes it directly,
with no shell, splitting, expansion or `grund` argument interpretation, so literal empty
arguments, quotes, spaces and shell punctuation reach the child unchanged. Both manifests
together, malformed JSON, a non-string item, an empty array or an empty executable is refused
before any fixture is copied or anything spawned, and the refusal names the case and the
manifest or manifests.

`{repo}` and `{repo_copy}` expand as in `command.args` ([§FS-examples.5.3](FS-examples.md#53-commandargs-is-read-as-shell-words-or-refused)), `{grund}` as a
whole argument is the built binary the runner uses, and every other brace stays literal. The
working-directory default and `command.cwd`, `command.stdin`, copying, symlink handling, the
exit, stream and tree comparison, deterministic reruns and refresh accounting are the shared
ones ([§FS-examples.5](FS-examples.md#5-e2e-reuse-without-duplication)), and an external command that uses `{repo_copy}` is mutating by the
same criterion as `command.args`. A failed spawn names the case and the executable and never
counts as golden success.

### 5.5 An example may pin several runs

An example whose walkthrough has more than one step keeps one run per step in its own
subdirectory, each shaped exactly like a single-run example (`repo/`, `expected.*`, an
optional `command.args`), with the example's `README.md` beside them. Discovery takes a
top-level example directory that holds no `expected.exit` and runs each direct subdirectory
that does, in name order, through the same shared runner; nothing deeper is searched. An
example directory with neither an `expected.exit` nor such a subdirectory is not a runnable
example.

### 5.6 Mutable fixtures belong to one invocation

The shared runner gives each invocation of a mutable case or example its own repository copy.
Two invocations of the same case may overlap, and neither removes, overwrites, creates links
in or inspects the other's copy: a case name identifies a logical fixture, not ownership of a
mutable directory.

The runner validates a case's manifests before it copies the fixture or spawns anything. One
copy then serves argument expansion, `command.cwd`, execution, final-tree comparison, golden
refresh and any post-run inspection, and lives until all of them are done; removing it
removes only that invocation's copy, on failure or unwind too, so repeated runs leave no
abandoned copies. A mutable case has no deterministic rerun, since its second run would see
the tree the first wrote.

The copy's unique directory never changes an authored exit, stdout, stderr or final-tree
golden: comparison and refresh alike normalize only the exact invocation-owned copy prefix,
to the logical `target/e2e-work/<case name>/repo` path, or to that path's absolute form where
the output spells the copy absolutely, so an unrelated path or a substantive mismatch is
never hidden.

The runner finishes writing every file it materializes into a copy, closing it included,
before any child process it spawns can inherit a descriptor to that file, so a fixture script
a case executes, directly or through the command under test, never fails to start as busy
(`ETXTBSY`) because a sibling invocation's spawn carried the runner's write descriptor.
Retrying the case or the spawn on that error does not meet this, since it hides the overlap
rather than removing it.
