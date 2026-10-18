# Review a checklist: count it, read it, map a change back to it

A reviewer holding a change to a login requirement wants three answers: how many points the
requirements chapter has, what each one says, and which of them the change touched
([§FS-examples.2](../../docs/functional-spec/FS-examples.md#2-canonical-use-cases)). Each is one `grund` command over a coordinate glob
([§FS-coordinate-globs](../../docs/functional-spec/FS-coordinate-globs.md#fs-coordinate-globs-a-coordinate-glob-selects-catalog-coordinates-by-pattern)), with no `jq` and no list of IDs typed by hand.

The fixture is the same in every run: a version-2 `grund.toml` with one row, the `FS` kind in
`docs/functional-spec/`, and [`FS-login`](1-count/repo/docs/functional-spec/FS-login.md),
whose `requirements` chapter holds three named points. The example pins three runs, one per
step ([§FS-examples.5.5](../../docs/functional-spec/FS-examples.md#55-an-example-may-pin-several-runs)). None writes anything.

## Step 1: count the checklist

```bash
grund list --glob 'FS-login.requirements{,.**}' --total examples/review-checklist/1-count/repo
```

```text
4
```

`{,.**}` is the point and everything beneath it: the empty alternative keeps the chapter, and
`.**` adds every point under it at any depth
([§FS-coordinate-globs.grammar.globstar](../../docs/functional-spec/FS-coordinate-globs.md#grammarglobstar--is-a-whole-component)). `--total` prints the number of rows `list`
would print and nothing else ([§FS-list.total](../../docs/functional-spec/FS-list.md#total---total-a-count-rung)), so the count is the cheap rung before the
bodies. Drop `--total` to see the rows.

## Step 2: read every point in one scan

```bash
grund show --batch --glob 'FS-login.requirements.*' examples/review-checklist/2-bodies/repo --format json
```

`.*` is the chapter's children, one level down. The batch reads the three bodies in one scan,
one JSON record per line, in the order `show --batch --all` would generate them
([§FS-show.1.9](../../docs/functional-spec/FS-show.md#19---batch---all-every-coordinate-in-scope)); `2-bodies/expected.stdout` holds them. Add `--toc` for each point's section
map instead, or `--brief` for its first paragraph.

## Step 3: name the points a change touched

A diff that changed lines 9 to 13 of the declaration is mapped back to its points:

```bash
cd examples/review-checklist/3-owners/repo
grund cover docs/functional-spec/FS-login.md --lines 9-13
```

```text
docs/functional-spec/FS-login.md:9-13
  9-10  FS-login.requirements.lockout
  11-13  FS-login.requirements.reset
```

Each sub-range is printed with the point that owns it ([§FS-cover.6](../../docs/functional-spec/FS-cover.md#6-line-ownership)), so the reviewer
re-reads exactly those two bodies from step 2.

## Trade-offs

- A glob selects declarations and their sections, never citations: who cites a point is still
  `grund refs`.
- A pattern that matches nothing is not an error; when no configured kind could start it, a
  `note:` on stderr says so, as for `fs-*` in a project whose kind is `FS`
  ([§FS-coordinate-globs.errors.note](../../docs/functional-spec/FS-coordinate-globs.md#errorsnote-a-pattern-no-kind-can-start-gets-a-note)).
- Quote the pattern: an unquoted `*` or `{` is the shell's before it is `grund`'s.
