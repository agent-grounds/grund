# Front matter: relationships kept in YAML still count as citations

A Markdown file may open with the YAML or TOML block that Jekyll, Hugo, Zola, Pandoc and
Docusaurus read. grund reads that block as a region of its own
([§FS-check.1.1.5.2](../../docs/functional-spec/FS-check.md#1152-front-matter-holds-no-heading)): no line of it is a heading, its `§` citations are live and belong to the
file's first declaration ([§FS-check.1.1.5.2.1](../../docs/functional-spec/FS-check.md#11521-its-citations-belong-to-the-files-first-declaration)), and `grund fmt` rewrites nothing in it. This
example is for a repository whose documents already carry front matter, in the Open
Knowledge Format's shape or a static-site generator's, and that wants the relationships it
writes there checked without repeating each one in the body.

The fixture is the same in both runs: a version-2 `grund.toml` with a `METRIC` kind and a
`POL` kind, the policy [`POL-revenue-recognition`](1-refs-from/repo/bundle/policies/revenue-recognition.md), and the metric
[`METRIC-net-revenue`](1-refs-from/repo/bundle/metrics/net-revenue.md), whose front matter names the policy as a source:

```markdown
---
type: Metric
title: Net revenue, monthly
# METRIC-gross: the gross figure is a separate metric
sources:
  - id: rev-policy
    resource: §POL-revenue-recognition
---

# METRIC-net-revenue: Net revenue, monthly
```

The line `# METRIC-gross: …` is a YAML comment. It declares nothing, so `grund list` shows
`METRIC-net-revenue` and `POL-revenue-recognition` and no `METRIC-gross`. The example pins
two runs ([§FS-examples.5.5](../../docs/functional-spec/FS-examples.md#55-an-example-may-pin-several-runs)). Neither writes anything.

## Step 1: what the metric cites

Run it inside the fixture:

```bash
cd examples/front-matter/1-refs-from/repo
grund refs --from METRIC-net-revenue
```

```text
bundle/metrics/net-revenue.md:7: §POL-revenue-recognition
```

The citation sits above the declaration's heading, in the front matter, and still counts
as the metric's: the front matter belongs to the file's first declaration, under no section.
`grund refs POL-revenue-recognition --format json` names `METRIC-net-revenue` as its
`enclosing_declaration` for the same reason, and `grund cover --lines 1-7` names it as the
owner of every line of the block.

## Step 2: a rule that reads it

```bash
grund check examples/front-matter/2-rule/repo \
  --rule "Each METRIC must cite at least one POL." --only-rule
```

```text
success
```

The metric satisfies the rule through its front matter alone, and citation directions read
the citation the same way.

## Trade-offs

- **Only the first declaration.** Every front-matter format keeps one document per file, so
  the block belongs to the declaration it sits above. A file that declares nothing gives
  its front-matter citations no owner; they still resolve and count as uses.
- **Never rewritten.** `grund fmt` leaves the block byte for byte, because a Markdown link is
  not a valid YAML or TOML value. `fmt --check` reports nothing there, and a `$$` typed in
  the block stays as typed.
- **No values read.** grund parses no YAML or TOML. A citation is found in the text as it is
  anywhere else, so a value that is not a citation is just text.
- **The extent is exact.** Line 1 must be exactly `---` or `+++`, closed by the same line or
  by `...` for `---`. A block that never closes is not front matter, and the file reads as
  it would without it.
