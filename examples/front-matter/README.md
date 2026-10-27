# Front matter: relationships kept in YAML still count as citations

A Markdown file may open with the YAML or TOML block that Jekyll, Hugo, Zola, Pandoc and
Docusaurus read. grund reads that block as a region of its own
([§FS-check.1.1.5.2](../../docs/functional-spec/FS-check.md#1152-front-matter-holds-no-heading)): no line of it is a heading, its `§` citations are live and belong to the
file's first declaration ([§FS-check.1.1.5.2.1](../../docs/functional-spec/FS-check.md#11521-its-citations-belong-to-the-files-first-declaration)), and `grund fmt` rewrites nothing in it. This
example is for a repository whose documents already carry front matter, in the Open
Knowledge Format's shape or a static-site generator's, and that wants the relationships it
writes there checked without repeating each one in the body.

The first two runs share one fixture: a version-2 `grund.toml` with a `METRIC` kind and a
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
`METRIC-net-revenue` and `POL-revenue-recognition` and no `METRIC-gross`. The last two runs
declare the front matter as a field of the `METRIC` kind, so grund reads its keys too. The
example pins four runs ([§FS-examples.5.5](../../docs/functional-spec/FS-examples.md#55-an-example-may-pin-several-runs)). None writes anything.

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

## Step 3: hold the front matter to a shape

A kind that declares the `front-matter` field reads its first declaration's YAML block, and
the block's top-level keys become the field's sections ([§FS-fields.front-matter](../../docs/functional-spec/FS-fields.md#front-matter-front-matter-is-a-field)). The third
run's `grund.toml` declares it on `METRIC`, closed, with three keys, and holds `type` to one
value:

```toml
[schema.kinds.METRIC.fields.front-matter]
presence = "must"
closed = "must"

[schema.kinds.METRIC.fields.front-matter.keys.type]
presence = "must"
one_of = ["Metric"]

[schema.kinds.METRIC.fields.front-matter.keys.title]

[schema.kinds.METRIC.fields.front-matter.keys.sources]
```

Beside the net-revenue metric sits a second one,
[`METRIC-gross-bookings`](3-fields/repo/bundle/metrics/gross-bookings.md), whose block calls it
a `Measure` and carries an `owner` key the field does not declare:

```markdown
---
type: Measure
title: Gross bookings, monthly
owner: finance
sources:
  - id: rev-policy
    resource: §POL-revenue-recognition
---
```

```bash
grund check examples/front-matter/3-fields/repo
```

```text
bundle/metrics/gross-bookings.md:2: error: METRIC-gross-bookings.front-matter.type is "Measure"; kind METRIC admits Metric; change the value to one of them
bundle/metrics/gross-bookings.md:4: error: METRIC-gross-bookings.front-matter.owner is not a key of kind METRIC's front matter, which is closed; its keys: type, title, sources; rename it to one of them, remove it, or add it to the field's keys
```

Each finding is at its key's line. The net-revenue metric passes: its `type` is `Metric`,
and the YAML comment between its keys is no key. A metric with no front matter at all would
be reported too, because the field's `presence` is `must`.

## Step 4: a rule over one key

The keys are sections, so a chapter rule can name one by its whole path ([§FS-rules.2.1](../../docs/functional-spec/FS-rules.md#21-a-chapters-name-is-its-whole-path)):

```bash
grund check examples/front-matter/4-key-rule/repo \
  --rule "The front-matter.sources chapter of each METRIC must cite at least one POL." --only-rule
```

```text
success
```

Step 2's rule counts a citation anywhere in the metric. This one reads only the citations
inside the `sources` key, so a policy cited in the body alone would fail it.
`grund show METRIC-net-revenue.front-matter.sources` prints that key as written, and
`grund list --glob 'METRIC-*.front-matter.*'` lists every key of every metric.

## Trade-offs

- **Only the first declaration.** Every front-matter format keeps one document per file, so
  the block belongs to the declaration it sits above. A file that declares nothing gives
  its front-matter citations no owner; they still resolve and count as uses.
- **Never rewritten.** `grund fmt` leaves the block byte for byte, because a Markdown link is
  not a valid YAML or TOML value. `fmt --check` reports nothing there, and a `$$` typed in
  the block stays as typed.
- **Values read only where a kind asks.** A kind that declares no `front-matter` field parses
  no YAML or TOML: a citation is found in the block as it is anywhere else, and a value
  that is not a citation is just text. A kind that declares it parses the YAML, and a
  key's value is the text it decodes to, never a typed value, so `one_of` compares
  `2026-02-03` and `no` as written. A TOML block is not read as a field yet; it is
  reported as `unreadable-front-matter`.
- **The extent is exact.** Line 1 must be exactly `---` or `+++`, closed by the same line or
  by `...` for `---`. A block that never closes is not front matter, and the file reads as
  it would without it.
