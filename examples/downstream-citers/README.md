# Downstream citers: an open project guards the points a closed repository cites

An open project, `substratevm`, declares `FS-sec`, `FS-rel` and `FS-old`. A closed
repository, `ee`, cites two of them, and `substratevm`'s gate never sees `ee`. Checked
alone, `substratevm` would warn that all three are never cited, and nothing would stop
someone deleting the two a shipped product depends on. This example is for the maintainer
of such an open project. Its use-cases are cross-project citations, `cover` and the unused
signal ([§FS-examples.2](../../docs/functional-spec/FS-examples.md#2-canonical-use-cases)): `ee` lists its citations into `substratevm`, and
`substratevm` commits that list as a page its own `check` reads. The recipe and its limits
are in the repository options guide, under
[Citers in a repository you cannot read](../../docs/user-facing/repository-options.md#citers-in-a-repository-you-cannot-read).

The example pins three runs ([§FS-examples.5.5](../../docs/functional-spec/FS-examples.md#55-an-example-may-pin-several-runs)). None writes anything.

## Step 1: the closed side lists its citations

[`1-cover/repo`](1-cover/repo/) is `ee`. Its [`docs/tck.md`](1-cover/repo/docs/tck.md) cites
`substratevm/FS-sec.1` and `substratevm/FS-rel.access.1`, and its `grund.toml` lists no
workspace, so the alias resolves nowhere. `cover` keeps it anyway
([§FS-cover.2.1](../../docs/functional-spec/FS-cover.md#21-a-cross-project-citation-counts)):

```bash
cd examples/downstream-citers/1-cover/repo
grund cover --format json
```

```json
{"path":"docs/tck.md","citations":[{"path":"docs/tck.md","line":3,"column":21,"id":"substratevm/FS-sec","section":"1","marker":true,"text":"§substratevm/FS-sec.1","enclosing_declaration":null,"enclosing_section":null},{"path":"docs/tck.md","line":4,"column":20,"id":"substratevm/FS-rel.access","section":"1","marker":true,"text":"§substratevm/FS-rel.access.1","enclosing_declaration":null,"enclosing_section":null}]}
```

One `jq` step keeps the citations into `substratevm` and drops the alias. The example
runner does not run `jq`, so the pinned run stops at `cover`, and the filter is here:

```bash
grund cover --format json \
  | jq -r '.citations[] | select(.id | startswith("substratevm/"))
           | "- §\(.id | ltrimstr("substratevm/"))\(if .section then "." + .section else "" end)"' \
  | sort -u
```

```text
- §FS-rel.access.1
- §FS-sec.1
```

`ee`'s own `check` still needs those citations to resolve, for example through a parent
workspace that lists `substratevm` as a member. Run alone it reports
`docs/tck.md:3: error: unknown project alias substratevm`; only `cover` works here alone.

## Step 2: the open side commits the page

[`2-check/repo`](2-check/repo/) is `substratevm` with the list committed as
[`docs/downstream-citers.md`](2-check/repo/docs/downstream-citers.md): a heading, one
sentence naming where the list came from and how to regenerate it, and the lines `jq`
printed. `grund fmt --write` then linked each line and recorded the cited relative section
`FS-rel.access.1` in [`.grund/sections/FS-rel.lock`](2-check/repo/.grund/sections/FS-rel.lock).

```bash
cd examples/downstream-citers/2-check/repo
grund check
echo $?    # 0
```

```text
docs/functional-spec/old.md:1: warning: declared but never cited: FS-old; cite it
```

The page is an ordinary citing file: a citation counts wherever the run reads it
([§FS-check.4.1.4](../../docs/functional-spec/FS-check.md#414-a-citation-anywhere-in-the-resolution-scope-counts)). The one warning left is the point nobody cites.

## Step 3: a listed point is deleted

[`3-deleted/repo`](3-deleted/repo/) is the same tree after someone deleted
`docs/functional-spec/security-providers.md`, which declared `FS-sec`. The page still
cites it, so the open gate refuses the deletion at the page:

```bash
cd examples/downstream-citers/3-deleted/repo
grund check
echo $?    # 1
```

```text
docs/downstream-citers.md:9: error: unknown reference FS-sec; did you mean FS-rel?
docs/functional-spec/old.md:1: warning: declared but never cited: FS-old; cite it
```

Swapping the text of `FS-rel`'s two `### .1` sections is refused the same way, by the
section lock, naming the page as the citing site
([§FS-section-lock](../../docs/functional-spec/FS-section-lock.md#fs-section-lock-a-cited-relative-section-keeps-its-text-or-check-says-where-the-text-went)).

## Trade-offs

- The page only records that a point is cited, not what `ee` relies on it for.
- It goes stale until someone regenerates it: a point `ee` stopped citing stays "used"
  until then. Regenerate it whenever `ee` changes, and replace the list rather than edit it.
- Under a closed `[rules.citations] default = "must-not"`, the page sits in the homeless
  `code` row, which must `may` the kinds it lists, or every line is refused with
  `code must not cite FS (citation direction)`.
- Never write the page by hand or use it to quiet a warning for a point nobody cites: that
  empties the unused signal the page exists to keep. The page is the project's own file;
  grund neither keeps nor refreshes it.
