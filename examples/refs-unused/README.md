# Uncited declarations: one `refs` answer points at the listing of them all

You ask `grund refs` whether one declaration is cited, and nothing cites it. The answer is
empty, and a `note:` on stderr names the one call that lists every such declaration at once,
`grund list --unused` ([§FS-refs.2.1.1](../../docs/functional-spec/FS-refs.md#211-a-declared-id-nothing-cites-points-at-list---unused)). This example is for anyone, person or agent,
checking IDs one at a time before a cleanup or a review. Its use-case is finding a
declaration's blast radius with `grund refs` ([§FS-examples.2](../../docs/functional-spec/FS-examples.md#2-canonical-use-cases)), and the case where that
radius is empty.

The fixture is the same in both runs: a version-2 `grund.toml` with an `FS` kind in
`docs/functional-spec/` and no kind index, [`FS-login`](1-refs/repo/docs/functional-spec/FS-login.md), which
`src/login.py` cites, and [`FS-audit`](1-refs/repo/docs/functional-spec/FS-audit.md), which nothing cites. The example
pins two runs ([§FS-examples.5.5](../../docs/functional-spec/FS-examples.md#55-an-example-may-pin-several-runs)). Neither writes anything.

## Step 1: ask about one declaration

Run it inside the fixture, with no path:

```bash
cd examples/refs-unused/1-refs/repo
grund refs FS-audit
echo $?    # 0
```

stdout is empty, and stderr is one line:

```text
note: FS-audit is declared but nothing cites it; `grund list --unused` lists every such declaration
```

The empty answer is still the answer. The exit is `0`, `--total` prints `not cited`, and
`--format json` prints nothing, as for any uncited ID. The note is a hint, not a result: a
script that reads only stdout never sees it. It names the declaration as the catalog spells
it, whatever digits or shorthand you typed.

## Step 2: list them all in one call

```bash
cd examples/refs-unused/2-list-unused/repo
grund list --unused
```

```text
FS-audit  docs/functional-spec/FS-audit.md:1  Audit log
```

That is the whole set ([§FS-list.1.3](../../docs/functional-spec/FS-list.md#13---unused)): one process for every declaration nothing
cites, where the per-ID loop costs one `grund refs` call per declaration.

## When the note is withheld

The note appears only where `list --unused` answers the question you asked:

- **A path.** `grund refs FS-audit .` or `grund refs FS-audit docs` asks about part of the
  tree, which says nothing about the whole project, so the empty answer has no note.
  `grund refs FS-audit src` still gets the scoped typo note, because no declaration of
  `FS-audit` is inside `src`.
- **A section.** `grund refs <ID>.<section>`, or `--section`, asks about one section, and
  `--unused` judges whole declarations. A bare `--descendants` with no section changes
  nothing, so `grund refs FS-audit --descendants` gets the note too.
- **An index entry.** An ID whose only citation is its kind's index entry has one site to
  list, labelled `(index entry)`, so it gets no note, even though `list --unused` lists it.
- **An incomplete scan.** If a file could not be read, the run exits `2` and its empty
  answer is not trusted as complete.

In a workspace, an alias-qualified query names its project:
`grund list --project <alias> --unused`. An `E2E` case adds `--kind E2E`, because
`--unused` leaves cases out unless that kind is selected.

## Trade-offs

- The note costs one stderr line on an answer that was silent before. A caller that treats
  any stderr as failure sees it; one that reads stdout and the exit code sees no change.
- It is a pointer, not a second answer. `refs` does not run the listing for you. The note
  tells you that the next step answers every ID at once.
