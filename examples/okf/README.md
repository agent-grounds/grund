# OKF: a knowledge bundle held to a grund schema

The [Open Knowledge Format](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md)
(OKF, v0.2) stores knowledge as a bundle: a directory of Markdown concepts, each
opening with YAML frontmatter whose one required key is `type`, linked by
ordinary Markdown links. OKF fixes what every consumer must accept and, on
purpose, nothing more. It has no taxonomy of types, no required body
sections, and broken links are tolerated. So a bundle that agents keep rewriting
can drift in ways no OKF consumer will complain about.

This example is for someone who publishes an OKF bundle, or has an agent write
one, and wants `grund check` as the gate. It sets up a conformant repository
whose `grund.toml` is a producer's house rules on top of OKF
([§FS-examples.2](../../docs/functional-spec/FS-examples.md#2-canonical-use-cases)):

- each OKF `type` is a kind;
- each conventional body heading is a field;
- each `index.md` is the kind's index;
- each link a concept owes is a citation direction.

The bundle stays an OKF bundle, and grund holds it to more than OKF does.

The fixture is the income statement of OKF's Appendix A: a metric that narrates
two figures, an attested computation for each, the policies and the table they
answer to, the executor that runs them, and the attester that checks a run. The
example pins two runs over the same fixture ([§FS-examples.5.5](../../docs/functional-spec/FS-examples.md#55-an-example-may-pin-several-runs)): the schema, and
the check. Neither writes anything.

## The bundle

The bundle is `finance/`. `grund.toml` sits beside it, never inside it,
because OKF reads every `.md` in a bundle other than `index.md` and `log.md` as
a concept that must open with frontmatter.

```text
grund.toml                          the schema
finance/                            the OKF bundle
  index.md                          okf_version: "0.2", the only frontmatter an index may carry
  log.md                            dated update history; its entries cite what changed
  metrics/income-statement.md       MET-income-statement      type: Metric
  computations/revenue.md           COMP-revenue              type: Attested Computation
  computations/profit.md            COMP-profit               type: Attested Computation
  policies/revenue-recognition.md   POL-revenue-recognition   type: Policy
  policies/cost-allocation.md       POL-cost-allocation       type: Policy
  tables/recognized-revenue.md      TBL-recognized-revenue    type: BigQuery Table
  references/skills/run-on-bq.md    SKILL-run-on-bq           type: Skill
  references/attesters/sql-equality.py                        cites SKILL-run-on-bq.receipt
```

Every folder also holds an `index.md`. Open [`grund.toml`](1-schema/repo/grund.toml)
first, then [`computations/revenue.md`](1-schema/repo/finance/computations/revenue.md):
it carries every OKF family (trust, lifecycle, provenance with credibility
signals, and the computation contract), and its body is what the schema
holds it to.

## How OKF maps onto grund

| OKF | grund | In this bundle |
|---|---|---|
| a concept's `type` (OKF 4.1) | a kind in the folder that type lives in, titled with the type | `type: Attested Computation` is `COMP` in `finance/computations/` |
| the concept ID, its path less `.md` | the `<KIND>-<slug>` its first heading declares, the slug being the file's stem | `computations/revenue` is `COMP-revenue` |
| frontmatter | text before the declaration; grund reads none of it | `verified`, `stale_after`, `sources` |
| a conventional body heading (OKF 4.2) | a field, at H2 under the declaration ([§FS-fields](../../docs/functional-spec/FS-fields.md#fs-fields-a-v2-kind-states-the-shape-of-its-declarations-and-check-holds-every-declaration-to-it)) | a table must open with `## schema: Schema` |
| the inline computation (OKF 10.3) | the `computation` field of a closed kind | nothing else may sit beside it |
| `index.md` (OKF 8) | the kind's index, `index = "index.md"` ([§FS-check.3.18](../../docs/functional-spec/FS-check.md#318-declaration-missing-from-its-kinds-index)) | every concept is listed in its folder's index |
| a link (OKF 6.1) | a citation, which `grund fmt --cross-refs --write` writes as the link ([§FS-fmt.6](../../docs/functional-spec/FS-fmt.md#6-cross-reference-emission)) | `[§TBL-recognized-revenue](../tables/recognized-revenue.md#…)` |
| a per-claim footnote keyed to `sources` (OKF 5.1) | a footnote whose text carries the citation | `[^rev-policy]: Revenue recognition policy, [§POL-revenue-recognition.1](…)` |
| an attester, which is code (OKF 10.2) | a place that cites and declares nothing | `sql-equality.py` must cite the receipt it reads |

An index entry is an OKF entry and a grund entry at once:
`* [§COMP-revenue Revenue for fiscal year](revenue.md) - <description>` is
OKF's `* [Title](url) - description`, and its link text holds the citation
([§FS-citations.wrap.enclosing.scope](../../docs/functional-spec/FS-citations.md#wrapenclosingscope-every-form-a-link-is-written-in)).

## Step 1: what the bundle is held to

```bash
grund schema examples/okf/1-schema/repo
echo $?    # 0
```

`grund schema` reads the configuration alone ([§FS-schema](../../docs/functional-spec/FS-schema.md#fs-schema-grund-schema-shows-what-a-repository-is-held-to-read-from-its-configuration-alone)):

```text
okf-finance · grund.toml · v2

kinds and places
  MET       kind                 finance/metrics/
              field definition   must · first
  COMP      kind                 finance/computations/
              field computation  must · first
              closed must
  POL       kind                 finance/policies/
  TBL       kind                 finance/tables/
              field schema       must · first
              field examples     may · after schema
  SKILL     kind                 finance/references/skills/
              field steps        must · first
              field receipt      must · after steps
  attester  place                finance/references/attesters/
  code      complement           every scanned file no other row claims

citations
  MET       must      COMP
  COMP      must      POL
            must      TBL
            must      SKILL
  POL       may       POL (itself)
            must not  everything else
  TBL       —
  SKILL     —
  attester  must      SKILL
  code      —

must = check error · warn = check warning · should = suggestion (check --suggestions)
may = allowed · must not / warn not / should not = forbidden at that strength
A|B = one obligation, met by citing either · — = no direction rule names it
```

Read it as OKF:

- **MET → COMP** is OKF 10.4: a metric narrates, and each figure links to the
  attested computation that produces it.
- **COMP → POL, TBL, SKILL** are three obligations, not one. A computation
  names the policy it answers to (its provenance), the table it reads, and the
  executor that runs it.
- **POL must not** cite anything but a policy. A policy is a source of truth,
  so derived concepts cite it and it never cites them back. A policy's
  `Cited by` list is `grund refs POL-<slug>`, never written by hand.
- **attester → SKILL** grounds the code: `sql-equality.py` cites the
  `receipt` chapter of the executor whose receipt it inspects.
- `code` is the rest of the bundle, the root `index.md` and `log.md`.

## Step 2: the bundle holds

```bash
grund check examples/okf/2-check/repo
echo $?    # 0
```

```text
success
```

The same tree is a conformant OKF v0.2 bundle (OKF 11). Every concept opens
with frontmatter carrying a non-empty `type`. No `index.md` carries frontmatter
except the root's `okf_version`. `log.md` groups its entries under
`YYYY-MM-DD` headings. Its trust and lifecycle families reach the two verdicts
the appendix designed: `COMP-revenue` is human-reviewed and fresh, while
`COMP-profit` is only machine-confirmed and past its `stale_after`. Those come
from frontmatter, which grund leaves to OKF.

## Writing a concept

Write a relationship as a bare citation, `§COMP-revenue`, and run
`grund fmt --cross-refs --write` from the project root. Each citation becomes a
relative link to the concept's file, with the declaration's anchor:

```markdown
It runs through
[§SKILL-run-on-bq](../references/skills/run-on-bq.md#skill-run-on-bq-run-an-attested-computation-on-bigquery),
```

That is the form OKF's own reference viewer draws as a graph edge. Its link
pattern accepts the anchor, and it skips links that start with `/`. So the
bundle's link graph and its citation graph are one graph. The frontmatter's
path fields (`sources[].resource`, `executor.resource`, `attester.resource`)
use OKF's recommended bundle-rooted form, `/references/skills/run-on-bq.md`,
because grund never reads them.

Before the first `grund init`, keep `[presentation] rules = "root"`
([§FS-config-v2.presentation](../../docs/functional-spec/FS-config-v2.md#presentation-the-presentation-concern)). The default, `"home"`, writes an `AGENTS.md`
into each kind's folder, which puts six concepts without frontmatter inside the
bundle. With `"root"`, the one `AGENTS.md` sits beside `grund.toml`.

## Break it

Copy the fixture somewhere writable and make three edits an agent plausibly
makes. Every one leaves a conformant OKF bundle, and `grund check` refuses all
three:

```bash
demo_root=$(mktemp -d)
cp -R examples/okf/2-check/repo/. "$demo_root/"
cd "$demo_root"
```

1. In `finance/computations/revenue.md`, delete the line
   `## computation: Computation`, so the SQL becomes part of the lead.
2. Append the back-link list OKF's sample bundles give their policies to
   `finance/policies/cost-allocation.md`:

   ```markdown
   ## 3. Cited by

   - §COMP-profit computes gross profit under this standard.
   ```

3. Add `finance/tables/orders.md`, using OKF's bare heading and no index entry:

   ```markdown
   ---
   type: BigQuery Table
   ---

   # TBL-orders: Orders

   One row per order.

   ## Schema

   | Column | Type |
   |---|---|
   | `order_id` | STRING |
   ```

```console
$ grund check
finance/computations/revenue.md:33: error: COMP-revenue has no computation field; kind COMP declares it
finance/policies/cost-allocation.md:31: error: POL must not cite COMP (citation direction) — re-point the citation or downgrade it to a plain Markdown link
finance/tables/orders.md:5: error: TBL-orders has no schema field; kind TBL declares it
finance/tables/orders.md:5: error: TBL-orders is not listed in finance/tables/index.md: an entry is a `§`-marked Markdown link to the declaration; write `[§TBL-orders](orders.md#tbl-orders-orders)` in finance/tables/index.md — became an error in grund 0.13.0
finance/tables/orders.md:9: error: unmarked heading inside TBL-orders; number it (## 1. Schema) as TBL-orders.1, declare an ID, or use a bold label; this became an error in grund 0.16.0
finance/tables/orders.md:5: warning: declared but never cited: TBL-orders
```

The repair for the table is the field's form, `## schema: Schema`, plus the
index entry the finding writes out. Numbering the heading would leave the field
missing.

## Trade-offs

What the schema buys:

- **Structure an agent cannot quietly drop.** The computation stays in its
  fence, a table keeps its `Schema`, and a skill keeps the receipt its attester
  depends on.
- **Links a concept owes.** OKF's links are untyped and optional. Here, a
  metric with no computation, or a computation with no policy, is an error.
- **Navigation for free.** `grund refs`, `grund show` and `grund schema` work
  on the bundle as on any grounded repository. A policy's readers come from
  `grund refs`, so its back-links never go stale.

What it costs:

- **Frontmatter is invisible to grund.** Nothing checks that `type` matches the
  kind, that a `sources` entry's `resource` names the policy its footnote
  cites, or that `executor.resource` names the skill the body cites. Each fact
  is written twice, and only the body copy is checked. The title is also
  written twice: in frontmatter, and in the declaration heading.
- **Field headings show their handle.** A field is `## schema: Schema` at H2,
  not OKF's bare `# Schema`. OKF calls those headings conventional rather than
  required, so the bundle stays conformant. A consumer that matches heading
  text exactly, though, reads `schema: Schema`.
- **Stricter than OKF on links.** OKF reads a broken link as knowledge not yet
  written, while grund reports a citation that resolves nowhere. Write a
  placeholder as a plain link with no `§`.
- **One form of computation.** OKF also lets a computation live in a file named
  by its `computation` key, with no fence. This schema chooses the inline form;
  a bundle that uses both sets the `computation` field's `presence` to `warn`.
