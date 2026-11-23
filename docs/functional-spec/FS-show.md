# FS-show: grund reads a single declaration body by ID

The `show` subcommand prints a slice of one declaration's body by its ID: the cheap
way to pull one grounded fact into context, widened one flag at a time. Its batch
forms answer an ordered stream of coordinates, or every coordinate in scope, from one
load. Serves
[§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible),
[§GOAL-token-economy](../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file),
and [§GOAL-fast-feedback.1](../goals.md#1-performance-targets).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, home, citable, body, section, coordinate,
lead, index, catalog), [§FS-terms.terms.2](FS-terms.md#terms2-citations) (marker, citation, shorthand, canonical form),
[§FS-terms.terms.3](FS-terms.md#terms3-source-forms) (source declaration, stub, doc-comment), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, scope,
workspace, alias), [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding), and [§FS-terms.terms.7](FS-terms.md#terms7-values-and-integrations) (value, fetcher,
snapshot).

- **slice** — How much of a body a read returns — `--brief`, the default, `--toc`, `--full`, and
  the section forms. The slices are strictly nested, so escalating is one more flag.
- **brief** — The cheapest slice: the heading line and the first paragraph, the hover preview.
- **section map** — What `--toc` returns: the nested map of the coordinates a body records,
  without their prose.
- **batch query** — One `--batch` invocation answering an ordered stream of coordinates, or
  every coordinate in scope, from a single load, in any `--format`.
- **header** — The `==> <coordinate> <==` line a `text` or `md` batch prints above each answer:
  framing between slices, never part of one.

## requirements: What every read holds to

What `show` holds to with every other command is stated once, where they share it: it
answers from the scan `check` judges and refuses a duplicate ID, a duplicate section or a
broken stub exactly where `check` reports one ([§REQ-one-answer.identity.declarations](../requirements/REQ-one-answer.md#identitydeclarations-every-command-reads-one-declaration-set),
[§REQ-one-answer.sections.ambiguity](../requirements/REQ-one-answer.md#sectionsambiguity-check-and-show-judge-a-section-path-from-one-record), [§REQ-one-answer.stubs.verdict](../requirements/REQ-one-answer.md#stubsverdict-every-command-reads-one-verdict-per-stub)); it fails rather than
guesses ([§REQ-no-wrong-citation.1](../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution)); and it reads offline, writes nothing and prints the same
bytes on every run ([§REQ-runs-offline.1](../requirements/REQ-runs-offline.md#1-read-and-verification-paths-execute-nothing), [§REQ-no-data-loss.1](../requirements/REQ-no-data-loss.md#1-the-query-surface-never-writes), [§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes)).

### requirements.single-body: A finding fails a read only when it leaves no single body

`show` refuses on three findings `check` reports, a duplicate declaration, a duplicate
section on the requested path and a broken stub, because each leaves something other than
one body to print ([§FS-show.2.2.1](FS-show.md#221-ambiguous-id), [§FS-show.2.2.2](FS-show.md#222-ambiguous-section), [§FS-show.2.3.4](FS-show.md#234-broken-stub)). Every other finding
leaves the read as it was: an off-grammar declaration reads in every slice ([§FS-show.1.1](FS-show.md#11-id)), a
duplicate on another path does not stop this one ([§FS-show.2.2.2](FS-show.md#222-ambiguous-section)), a dangling citation in
the body is printed, and flattened, like a resolving one ([§FS-show.3.2](FS-show.md#32-cross-reference-links-are-flattened-in-text-and-json)), and a stub is
refused only where its target, scanned, would be ([§FS-show.2.3](FS-show.md#23-source-declarations-in-code-and-doc-comments)). So an agent can read a
declaration in a tree that does not pass `check` yet. Serves [§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work).

### requirements.authored-bytes: A slice is the author's bytes, selected and never rewritten

Every slice selects text the author wrote and generates none of its own: the lead, the
section map and the full body are cuts of one body ([§FS-show.2.1](FS-show.md#21-whole-declaration-default), [§FS-show.2.2](FS-show.md#22-section)); a fetched
snapshot and a value-source declaration print their committed source slice, never
synthesized Markdown ([§FS-show.2](FS-show.md#2-behavior)); `md` injects nothing, not even the kind's title ([§FS-show.3.1.2](FS-show.md#312-md)); and a batch's
header sits between slices, never inside one ([§FS-show.3.3.1](FS-show.md#331-the-header)). Three rewrites are named and
are the only ones: a source declaration loses its comment wrapper, leaving the Markdown its
author wrote ([§FS-show.2.3.2](FS-show.md#232-stripping-comment-prefixes)); `text` and `json` flatten cross-reference wrappers back to the
bare citation ([§FS-show.3.2](FS-show.md#32-cross-reference-links-are-flattened-in-text-and-json)); and a relative heading prints its canonical path on the lines a
reader copies a coordinate from, each line of the section map and a selected section's own
first line ([§FS-declarations.section.relative.shown](FS-declarations.md#sectionrelativeshown-how-a-relative-heading-is-shown),
[§FS-declarations.section.relative.shown.selected](FS-declarations.md#sectionrelativeshownselected-a-selected-sections-own-heading-names-its-path)). An E2E case, which has no prose, prints a
manifest of its case directory instead ([§FS-show.2.4](FS-show.md#24-e2e-cases)). Serves [§GOAL-token-economy.2](../goals.md#2-what-this-rules-out-1).

### requirements.next-step: A failed query names its next step

Every failed query tells its reader what to do. A missing ID, a missing section, an ID the
grammar rejects, an existing path passed to the bare form and a missing path that reads as a
second coordinate each get a `hint:` naming the command that answers it, and the hint is
withheld where the result line already names the answer ([§FS-show.3.5](FS-show.md#35-the-hint-for-each-failure)). An ambiguity or a
broken stub names instead the sites to edit or the candidates to choose from
([§FS-show.2.2.1](FS-show.md#221-ambiguous-id), [§FS-show.3.4](FS-show.md#34-what-a-failed-query-prints)), and a failed query in a `text` or `md` batch keeps its hint,
in its place among the answers ([§FS-show.3.3.2](FS-show.md#332-a-failure-stays-in-its-place)). One case is deferred: for a missing fetched
snapshot `show` prints the generic missing-ID hint, not the `grund fetch` hint [§FS-show.3.5](FS-show.md#35-the-hint-for-each-failure)
specifies. Serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible).

## 1. Inputs

```
grund [show] <ID> [<path>] [--section <s>] [--brief | --toc | --full] [--format <text|md|json>]
grund show --batch [<path>] [--brief | --toc | --full] [--format <text|md|json>] [--path <path>]
grund show --batch --all [<path>] [--brief | --toc | --full] [--format <text|md|json>] [--path <path>]
grund show --batch --glob <pattern>... [<path>] [--brief | --toc | --full] [--format <text|md|json>] [--path <path>]
```

The first form reads one coordinate; the `--batch` forms answer many from one load.

### 1.1 `<ID>`

`<ID>` is the full ID without the marker (e.g. `FS-check`). It may include an inline section (`FS-check.3.1`) after the configured section separator ([§FS-config.3.2](FS-config.md#32-id--id-grammar)). It also accepts an exact off-grammar ID the catalog retains, read in every slice ([§FS-config.3.2.5](FS-config.md#325-off-grammar-declarations-stay-readable)); an inline section that could start at two places fails rather than guesses ([§FS-config.3.2.6](FS-config.md#326-resolving-an-off-grammar-citation)); and a missing fetched snapshot fails without running its fetcher ([§REQ-runs-offline.1](../requirements/REQ-runs-offline.md#1-read-and-verification-paths-execute-nothing)).

A configured literal such as `:` stays part of a conforming ID, as `list` prints it
([§REQ-one-answer.identity.declarations](../requirements/REQ-one-answer.md#identitydeclarations-every-command-reads-one-declaration-set), [§FS-declarations.line.configured-literals](FS-declarations.md#lineconfigured-literals-configured-literals-belong-to-the-canonical-id)).

### 1.2 The number-only shorthand

In a kind whose format carries both `{number}` and `{slug}`, `grund FS-042` and
`grund FS-042.1` read `FS-042-user-login`: a query persists nothing, so the shorthand is
accepted here whatever the shorthand policy says of a file ([§FS-citations.shorthand.cli](FS-citations.md#shorthandcli-the-shorthand-as-a-cli-id-argument),
[§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation)). Shorthand, duplicate, exact-ID and section readings fail rather than guess,
in [§FS-config.3.2](FS-config.md#32-id--id-grammar)'s precedence ([§FS-show.2.2.1](FS-show.md#221-ambiguous-id)).

### 1.3 `show` is optional

`show` may be omitted when the first non-flag word is the ID: `grund FS-check --toc` and
`grund --toc FS-check` print exactly what `grund show FS-check --toc` prints ([§FS-cli.1](FS-cli.md#1-the-default-subcommand)).

### 1.4 `<path>` and `--path`

What `<path>` means is [§FS-scan.path.show](FS-scan.md#pathshow-show-resolves-the-id-against-the-project-the-path-discovers). `--path <path>` is an accepted alias for scripts that prefer to pass it as a flag; the two forms are equivalent.

### 1.5 `--section <s>`

`--section <s>` names the section path (`3.1`, `goals.performance`) in place of the dotted
form, and composes with every slice flag as the dotted form does ([§FS-show.2.2](FS-show.md#22-section)). It is
mutually exclusive with the dotted form.

### 1.6 How much: `--brief`, the default, `--toc`, `--full`

`--brief`, `--toc` and `--full` are mutually exclusive; with none, `show` prints the default.
Each is one rung of a ladder whose bodies nest, so widening a read is one more flag
([§DF-show-default-token-cheap](../decisions/functional/DF-show-default-token-cheap.md#df-show-default-token-cheap-grund-show-defaults-to-the-cheap-read-the-full-body-is-opt-in)):

- `--brief` — the heading line and the first paragraph, a hover preview ([§FS-show.2.1.1](FS-show.md#211-brief---brief)).
- (no flag) — the lead, enough to know whether to read deeper ([§FS-show.2.1](FS-show.md#21-whole-declaration-default)).
- `--toc` — the lead and the section map, to choose the section to read next
  ([§FS-show.2.1.2](FS-show.md#212-section-map---toc)).
- `--full` — the whole body ([§FS-show.2.1.3](FS-show.md#213-full-body---full)).

In `text`, on a one-paragraph lead, `--brief` prints more than the default
([§FS-show.3.1.1](FS-show.md#311-text)).

### 1.7 `--format`

`--format` picks `text`, the default, `md` or `json` ([§FS-show.3.1](FS-show.md#31-format-variants)), for a single read and a
batch alike. Any other value exits `2` with ``error: unsupported show format `<f>` ``.

### 1.8 `--batch`: an explicit query stream

`--batch` needs the explicit `show` subcommand. Without `--all` or `--glob` it reads one
query object per non-empty stdin line, `{"id":"api/FS-login","section":"3.1"}`: `id` is a
required string and accepts every spelling the single read accepts; `section` is optional,
a string or `null`. Unknown fields, malformed JSON and any other shape are refused, the
whole input before any configuration is discovered: `error: batch input line <N>: <reason>`,
exit `2`. A non-null `section` beside an inline section in `id` is a well-formed record whose query
fails ([§FS-show.2.6](FS-show.md#26-batch-resolution)). `--section` is refused in every batch, and one slice flag applies to
every record.

### 1.9 `--batch --all`: every coordinate in scope

`--batch --all` reads no stdin and answers every coordinate in scope ([§FS-show.2.6](FS-show.md#26-batch-resolution)).
`--batch --glob <pattern>`, repeatable, answers the coordinates `--all` would that a pattern
matches ([§FS-coordinate-globs](FS-coordinate-globs.md#fs-coordinate-globs-a-coordinate-glob-selects-catalog-coordinates-by-pattern)); `--all` beside it is redundant and accepted, and stdin is
ignored. `--all` and `--glob` each need `--batch`. A pattern is refused and noted as `list`
refuses and notes it ([§FS-coordinate-globs.errors.shape](FS-coordinate-globs.md#errorsshape-one-refusal-line), [§FS-coordinate-globs.errors.note](FS-coordinate-globs.md#errorsnote-a-pattern-no-kind-can-start-gets-a-note)).
`<path>` and `--path` are equivalent and mutually exclusive.

## 2. Behavior

Which bytes a read selects, from each kind of home a body can have, and how a batch answers
its queries. A fetched snapshot is read like any declaration: every slice is cut from its
committed body, and no query checks its freshness ([§REQ-runs-offline.3](../requirements/REQ-runs-offline.md#3-no-implicit-freshness)). A value-source
declaration gives every slice its exact member or element bytes, never synthesized Markdown
([§FS-values.6.2](FS-values.md#62-value-source-declarations-in-the-catalog)).

### 2.1 Whole declaration (default)

`grund FS-check` prints the declaration's lead ([§FS-declarations.lead](FS-declarations.md#lead-what-cuts-a-lead)): enough to name what
the declaration is about without paying for the whole body. The opening heading is omitted
in `text` format and included in `md`. A declaration with no lead prints nothing and exits
`0`; `show` does not widen the slice on its own, so a caller that sees empty output asks for
`--toc` or `--full`. Decided in [§DF-show-default-token-cheap](../decisions/functional/DF-show-default-token-cheap.md#df-show-default-token-cheap-grund-show-defaults-to-the-cheap-read-the-full-body-is-opt-in).

#### 2.1.1 Brief (`--brief`)

`grund --brief FS-check` prints the declaration's heading line, kept in every format
([§FS-show.3.1.1](FS-show.md#311-text)), and the first paragraph below it: the first run of non-blank lines, ended by a
blank line, a child heading or the end of the body. With no lead prose it prints the heading
line alone, exit `0`, and a selected section's `--brief` cuts the same way below the
section's own heading ([§FS-show.2.2](FS-show.md#22-section)).

#### 2.1.2 Section map (`--toc`)

`grund --toc FS-check` prints the lead, a blank line, then every heading of the section map
([§FS-declarations.section.map](FS-declarations.md#sectionmap-the-section-map-is-the-declarations-body-span)), one per line, in document order, at the depth it was
written, with its complete path. A complete-path heading prints as `--full` shows it, a
relative one with its canonical path ([§FS-declarations.section.relative.shown](FS-declarations.md#sectionrelativeshown-how-a-relative-heading-is-shown)), so the
coordinate to read next is on the line; a `json` entry's `title` is the one [§FS-list.3.1](FS-list.md#31---format-text-default)
defines ([§FS-show.3.1.3](FS-show.md#313-json)).
Nothing is generated ([§DF-show-default-token-cheap.2.3](../decisions/functional/DF-show-default-token-cheap.md#23-slices-stay-structural-never-generated)). With an empty lead the blank line is
dropped and the map stands alone; with an empty map `--toc` prints what the default prints;
with both empty it prints nothing and exits `0`. A selected section's map is the headings
nested under it, after the section's own heading and lead ([§FS-show.2.2](FS-show.md#22-section)).

##### 2.1.2.1 The map is the declaration's body span

Which headings the map holds is [§FS-declarations.section.map](FS-declarations.md#sectionmap-the-section-map-is-the-declarations-body-span), and where the body span that
bounds it ends is [§FS-declarations.body](FS-declarations.md#body-where-a-declarations-body-ends). The map also holds front-matter coordinates,
outside that span ([§FS-fields.front-matter.coordinates](FS-fields.md#front-mattercoordinates-its-coordinates)).

#### 2.1.3 Full body (`--full`)

`grund --full FS-check` prints the whole body, to the end of its body span
([§FS-declarations.body](FS-declarations.md#body-where-a-declarations-body-ends)); a selected section prints its whole extent ([§FS-show.2.2](FS-show.md#22-section)).

### 2.2 Section

`grund FS-check.3.1` and `grund FS-check --section 3.1` select a section, at any depth
([§FS-config.3.3](FS-config.md#33-section-paths--arbitrary-nesting-depth)); a named path such as `goals.performance` selects the same way where named
sections are enabled ([§FS-config.3.2.7](FS-config.md#327-named_sections--the-gate-for-explicit-section-names), [§FS-config-v2.defaults.1](FS-config-v2.md#defaults1-what-v2-fixes)). The flag
determines how much of it is printed:

- `--brief`: the section heading and its first paragraph.
- (default): the section heading and its lead ([§FS-declarations.lead.section](FS-declarations.md#leadsection-a-sections-lead)).
- `--toc`: the section heading, its lead and the map of the headings nested under it.
- `--full`: the section heading and the section's whole extent ([§FS-declarations.section](FS-declarations.md#section-what-a-section-is)).

The selected section's heading is printed in every mode and format, as written, except
that a relative heading prints its canonical path
([§FS-declarations.section.relative.shown.selected](FS-declarations.md#sectionrelativeshownselected-a-selected-sections-own-heading-names-its-path)). A section that does not exist is
`section not found` ([§FS-show.3.5](FS-show.md#35-the-hint-for-each-failure)), and a front-matter coordinate prints only its own slice
([§FS-fields.front-matter.surfaces](FS-fields.md#front-mattersurfaces-what-each-surface-does-with-them)).

#### 2.2.1 Ambiguous ID

When an ID has more than one home, the duplicate of [§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration),
`show` does not pick one. A stub and the source declaration it points at are one home,
scanned or not and however many stubs point at it, and a home reached through stubs is named
at its target ([§FS-declarations.stubs.pairing.1](FS-declarations.md#stubspairing1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned), [§FS-declarations.stubs.pairing.2](FS-declarations.md#stubspairing2-stubs-to-one-target-stand-for-its-declarations-once),
[§FS-declarations.stubs.pairing.3](FS-declarations.md#stubspairing3-a-declaration-reached-through-stubs-is-named-at-its-target)). The read exits `1` with one bare line on stderr
([§FS-errors.2.3](FS-errors.md#23-bare-query-failure)), its sites in the order of [§FS-errors.4.1.1](FS-errors.md#411-the-order-of-a-duplicates-sites), and under `--format=json` the
same sites in the finding's `sites` field ([§FS-errors.5](FS-errors.md#5-json-format)):

```
ambiguous ID: <ID> (declared at <path>:<line>, <path>:<line>[, ...])
```

An ambiguous number-only shorthand fails the same way but names its candidate IDs, in ID
order, since the caller rather than the tree has to choose
([§DF-number-only-citation-shorthand.2.7](../decisions/functional/DF-number-only-citation-shorthand.md#27-ambiguity-is-reported-never-guessed)):

```
ambiguous ID: FS-042 (matches FS-042-user-login, FS-042-user-logout)
```

#### 2.2.2 Ambiguous section

When two headings of the selected declaration claim the requested path, numeric or named,
the duplicate of [§FS-declarations.checks.duplicate-section](FS-declarations.md#checksduplicate-section-duplicate-section-path), `show` does not pick one either
([§REQ-one-answer.sections.ambiguity](../requirements/REQ-one-answer.md#sectionsambiguity-check-and-show-judge-a-section-path-from-one-record)); a stub's sections are its target's ([§FS-check.3.2.1](FS-check.md#321-a-stubs-sections-are-its-targets-scanned-or-not)).
It exits `1` with the same bare-line form and site order, under its own JSON code,
`ambiguous-section`, because the fix is a renumbering inside one declaration rather than a
second home to remove ([§DF-duplicate-section-path.2.5](../decisions/functional/DF-duplicate-section-path.md#25-show-refuses-the-ambiguous-section-instead-of-printing-a-pick)):

```
ambiguous section: FS-001-login.1 (declared at docs/functional-spec/FS-001-login.md:5, docs/functional-spec/FS-001-login.md:9)
```

Only the requested path can collide:
`grund FS-001-login.2` answers while `grund FS-001-login.1` refuses. The whole-declaration
`--toc` map still lists both headings, while `grund FS-001-login.1 --toc` refuses like every
other slice of that coordinate.

#### 2.2.3 Every surface reads the same section map

Every surface reads the section map of [§FS-declarations.section.map](FS-declarations.md#sectionmap-the-section-map-is-the-declarations-body-span) ([§REQ-one-answer.sections.map](../requirements/REQ-one-answer.md#sectionsmap-every-surface-reads-one-section-map)).
For a heading the map rejects, a query for its coordinate has the ordinary `section not found` result and hint
from [§FS-show.3](FS-show.md#3-outputs), emits none of the outside heading's body, and never substitutes the
located `check` finding for query semantics.

### 2.3 Source declarations in code and doc-comments

When an ID's home is in code, read directly or through a stub, `show` prints the
declaration's comment block without its comment wrapper ([§FS-show.2.3.2](FS-show.md#232-stripping-comment-prefixes)) and cuts every
slice and section from that text as from a Markdown body. Its declaration line drops the `#`
([§FS-declarations.line](FS-declarations.md#line-what-a-line-in-declaration-position-declares), [§DF-code-declarations-drop-hash](../decisions/functional/DF-code-declarations-drop-hash.md#df-code-declarations-drop-hash-code-resident-declarations-may-drop-the--prefix)), and one doc-comment may declare
several IDs, each with its own body ([§FS-languages.comment-block.early-end](FS-languages.md#comment-blockearly-end-terminate-early-on-another-declaration)). An index link
that enrolls the declaration ([§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index)) creates no declaration, so it changes no
query's answer ([§FS-check.3.18.3](FS-check.md#3183-an-external-source-declaration-enrolls-by-its-canonical-link)).

Section depth counts from the declaration line, so under `/// AR-<event-bus>: …` a `## 1.`
heading is a depth-1 section, while `/// # 1. Sub` is a same-level heading that no query
selects, `--toc` does not list and no lead is cut at ([§FS-config.3.3.2](FS-config.md#332-section_heading_levels--heading-depth-against-path-depth),
[§FS-languages.comment-wrapper](FS-languages.md#comment-wrapper-the-comment-wrapper)).

A stub that is not broken ([§FS-show.2.3.4](FS-show.md#234-broken-stub)) reads as the declaration it pairs with in its
target: the body, the sections and the JSON `path` and `line` are that declaration's, and
`show` refuses only where the same target, scanned, would ([§FS-declarations.stubs.pairing.1](FS-declarations.md#stubspairing1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned),
[§FS-declarations.stubs.pairing.4](FS-declarations.md#stubspairing4-every-surface-shows-a-paired-declaration-at-its-target)). Lying outside `[scan] include` changes none of that
([§REQ-one-answer.stubs.outside-include](../requirements/REQ-one-answer.md#stubsoutside-include-a-target-outside-the-include-answers-as-if-scanned)), while a target the scan never reads, by its name
or its extension, declares nothing, so its stub is broken ([§FS-declarations.stubs.target.3](FS-declarations.md#stubstarget3-a-target-the-scan-does-not-read-declares-nothing)).

#### 2.3.1 What counts as the "comment block"

A declaration is found on a "declaration line" — a line in declaration position ([§FS-declarations.line](FS-declarations.md#line-what-a-line-in-declaration-position-declares)) that also sits inside a comment or docstring. Which of the lines around it form its block is [§FS-languages.comment-block](FS-languages.md#comment-block-the-comment-block).

##### 2.3.1.1 Find the open boundary

Where the block opens is [§FS-languages.comment-block.open](FS-languages.md#comment-blockopen-find-the-open-boundary).

##### 2.3.1.2 Find the close boundary

Where the block closes is [§FS-languages.comment-block.close](FS-languages.md#comment-blockclose-find-the-close-boundary).

##### 2.3.1.3 Terminate early on another declaration

Where another declaration line ends the block early is [§FS-languages.comment-block.early-end](FS-languages.md#comment-blockearly-end-terminate-early-on-another-declaration).

#### 2.3.2 Stripping comment prefixes

After the block is selected, its comment wrapper is removed line by line ([§FS-languages.comment-wrapper](FS-languages.md#comment-wrapper-the-comment-wrapper)), and a docstring's content is kept as it stands ([§FS-languages.docstring](FS-languages.md#docstring-docstrings)).

The result is the markdown that the declaration's author wrote, identical to what would have lived in a `.md` file had the declaration been doc-resident instead of inline. This is the property that makes [§FS-show.2.3](FS-show.md#23-source-declarations-in-code-and-doc-comments) round-trip-stable across the in-docs and in-code homes.

#### 2.3.4 Broken stub

When an ID's only home is a stub whose `<path>` does not exist or holds no source
declaration of `<ID>` ([§FS-declarations.checks.broken-stub](FS-declarations.md#checksbroken-stub-broken-inline-spec-stub)), `show` has no body to print. It
exits `1` with one bare line on stderr ([§FS-errors.2.3](FS-errors.md#23-bare-query-failure)), the first for a missing target and
the second for every other broken stub, by the verdict `check` reports
([§FS-declarations.stubs.verdict](FS-declarations.md#stubsverdict-one-verdict-per-stub-reached-once-per-scan)):

```
broken stub: <ID> (stub at <path>:<line> points at <target>, which does not exist)
broken stub: <ID> (stub at <path>:<line> points at <target>, which contains no inline declaration of <ID>)
```

### 2.4 E2E cases

`grund E2E-<name>` prints the case's manifest, composed from its case directory: the
invocation, which is `grund check` when the case has no `command.args` and bare `grund` when
they hold no words; the expected exit; and one line per file in the directory, relative to
it and sorted lexicographically.

```
grund <args…>
expected exit: <code>
fixtures:
- <relative path>
…
```

A manifest has no headings, so its lead is the whole manifest: the default, `--toc` and
`--full` each print all of it, and `--brief` its first line. A case has no sections, so `grund E2E-<name>.1` is
`section not found` ([§FS-declarations.e2e-case.id](FS-declarations.md#e2e-caseid-a-case-directory-and-the-id-it-declares)).

#### 2.4.2 The manifest as JSON

`--format=json` prints one object in every slice mode, its keys in this order:

```
{"id":"E2E-<name>","kind":"E2E","path":"e2e/cases/<name>","args":[…],"expected_exit":<code>,"fixtures":[…],"anchor":null}
```

`path` is the case directory under the configured `E2E` home, `args` the parsed
`command.args`, empty when there are none, `fixtures` the manifest's sorted list, and
`anchor` always `null`. The optional `kind_title` closes the object ([§FS-config.3.4.3](FS-config.md#343-title)),
and the `id`, `kind`, `path` prefix stays first because the installed resolver's E2E fallback
reads it ([§DF-configured-title-metadata.2](../decisions/functional/DF-configured-title-metadata.md#2-decision)).

### 2.5 A heading inside a fenced code block is an example

In a Markdown body, a line inside a fenced block (```` ``` ````, `~~~`) is content, never
structure: it does not end a lead, bound a section, enter a `--toc` map, or open or close a
declaration, and the fence and everything in it print verbatim, a link wrapper included
([§FS-show.3.2](FS-show.md#32-cross-reference-links-are-flattened-in-text-and-json)). A slice reads fences by the rule the scan bounds sections by
([§FS-languages.fence](FS-languages.md#fence-fences)), so it never cuts a body where the recorded section map says no
section starts. A line inside a raw-text HTML block such as `<pre>` is never structure
either ([§FS-check.1.1.5.1](FS-check.md#1151-a-raw-html-block-holds-no-heading)), but its citations stay live, so its wrappers are flattened.
Inside a comment block a fence hides only its headings ([§FS-languages.fence.comment-block](FS-languages.md#fencecomment-block-a-fence-inside-a-comment-block-hides-its-headings)).

### 2.6 Batch resolution

A batch loads one workspace context and answers every query from it; the loader is never
called per coordinate. An empty explicit stream succeeds without loading configuration or
scanning, while `--all` always loads, so an empty catalog succeeds with no records.

Explicit queries keep their input order and duplicates, and each well-formed one is
attempted even after an earlier one fails, by the same project selection, resolution,
slicing and flattening as the single read, in the one slice mode the invocation names.

`--all` generates one query for every declaration and every legal numeric or named section
it records, sorted bytewise by workspace-qualified ID, the whole declaration first and its
section paths byte-sorted after it. An ID is unqualified in the current project and carries
its project's alias in any other; when a workspace excludes its root, every ID is qualified.
A declaration or section with several claimants generates one coordinate, which fails as
ambiguous. `--glob` keeps the generated queries a pattern matches, in the same order, failed
records included, and a pattern's scope decides which projects' coordinates it can keep
([§FS-coordinate-globs.scope](FS-coordinate-globs.md#scope-scope)).

A well-formed query that names an invalid, missing or ambiguous ID, a missing or ambiguous
section, a broken stub, an unknown project alias, or both an inline and a non-null separate section
fails in its own record and stops nothing ([§FS-show.3.3.2](FS-show.md#332-a-failure-stays-in-its-place)). Malformed input or invocation, a
missing path, and a configuration or scan failure are run-level: no query is attempted,
stdout stays empty, and stderr carries the error.

## 3. Outputs

A single read prints its slice on stdout, one JSON object under `--format=json`, never NDJSON
([§FS-errors.5.1.1](FS-errors.md#511-query-results)), and exits `0`; a failed one prints on stderr and leaves stdout empty
([§FS-show.3.4](FS-show.md#34-what-a-failed-query-prints)). A batch is the one exception, in every format ([§FS-show.3.3](FS-show.md#33-batch-query)).

### 3.1 Format variants

Verbose `json` examples, failed queries included, are [§FS-output-shapes](FS-output-shapes.md#fs-output-shapes-machine-readable-output-shapes)'s.

#### 3.1.1 `text`

The slice alone. The declaration's own heading line is omitted in every mode but `--brief`,
which keeps it so the slice labels itself; a selected section opens with its own heading line
([§FS-show.2.2](FS-show.md#22-section)). For a source
declaration the slice is the comment-stripped text ([§FS-show.2.3.2](FS-show.md#232-stripping-comment-prefixes)), and for an E2E case the
manifest ([§FS-show.2.4](FS-show.md#24-e2e-cases)). Link wrappers are flattened to the bare citation ([§FS-show.3.2](FS-show.md#32-cross-reference-links-are-flattened-in-text-and-json)).

#### 3.1.2 `md`

What `text` prints, with the declaration's heading line in front of it in every mode but
`--brief`, whose slice already opens with a heading, and with link wrappers kept as written,
since this is the form that renders ([§FS-show.3.2](FS-show.md#32-cross-reference-links-are-flattened-in-text-and-json)). Nothing else is added, not even the
kind's title, which `json` carries as `kind_title`. A source declaration's heading line is
the one its doc-comment writes, without the comment prefix
(`AR-<event-bus>: In-process event broadcaster`).

#### 3.1.3 `json`

A single object on stdout: `{"id":<ID>,"section":<section-path or null>,"body":<string>,"kind_title":<optional string>,"anchor":<string or null>,"path":<declaring file>,"line":<1-indexed>}`.
`body` is the text `text` prints, wrappers flattened. `section` is `null` when the whole
declaration was read and otherwise the exact numeric or named path. With `--toc` the object
also carries `sections`, before `kind_title`: one
`{"path":<section path>,"title":<heading title after its coordinate>,"depth":<integer>,"anchor":<string or null>}`
per heading of the map, in document order. `kind_title` is omitted when the kind has no
effective title. The `path`, `line` pair always closes the object, because installed
`grund-open` copies read the location from that tail ([§DF-show-anchor-data.2.1](../decisions/functional/DF-show-anchor-data.md#21-the-anchor-is-a-field-of-the-read-not-a-link)). An E2E case prints the manifest object of [§FS-show.2.4.2](FS-show.md#242-the-manifest-as-json) instead. The wire form is
stable per [§GOAL-no-silent-breakage.1](../goals.md#1-what-counts-as-user-visible).

`anchor` is the fragment a web link to the selected heading takes, without its `#`: the slug
[§FS-fmt.6.2](FS-fmt.md#62-form) makes of that heading, as `fmt` writes it ([§REQ-one-answer.writes.anchors](../requirements/REQ-one-answer.md#writesanchors-shows-anchor-is-the-anchor-fmt-writes)), under
the `[fmt.cross_refs] anchor_format` profile of the project that declares the target
([§FS-fmt.6.7.1](FS-fmt.md#671-anchor_format-names-a-renderer-profile)). Every site anchors on its own heading, each `--toc` entry included, so a
heading listed twice carries each site's anchor, and a caller composes
`<base>/blob/<ref>/<path>#<anchor>` from one read without slugging anything
([§DF-show-anchor-data](../decisions/functional/DF-show-anchor-data.md#df-show-anchor-data-show-json-carries-the-heading-anchor-grund-already-derives)). `anchor` is `null` where the site has no heading anchor: under the
`none` profile, for a declaration whose home is a source file, read directly or through a
stub, for a value-source declaration ([§FS-output-shapes.4.3](FS-output-shapes.md#43-value-source-declarations)), for a front-matter coordinate
([§FS-output-shapes.4.4](FS-output-shapes.md#44-front-matter-coordinates)), and for a `--toc` entry inside a source declaration; a caller then
links by `line`, as `#L<line>`. `[fmt.cross_refs] enabled = false` and the formatter's
exclusions govern what `fmt` writes, not what a heading's anchor is, so neither makes it
`null`.

### 3.2 Cross-reference links are flattened in `text` and `json`

A tree that has run `grund fmt --cross-refs` ([§FS-fmt.6](FS-fmt.md#6-cross-reference-emission)) wraps each citation in a Markdown
file in a link, `[§FS-check.1](FS-check.md#1-inputs)` for `§FS-check.1`. For an agent the
wrapper is noise and its relative path the wrong pointer, so `text` and the `json` `body`
flatten each wrapper back to the bare citation, a qualified one such as
`[<§>api/FS-login](...)` included ([§FS-workspace.8.5](FS-workspace.md#85-grund-fmt---cross-refs)). Decided in
[§DF-show-cross-ref-flattening](../decisions/functional/DF-show-cross-ref-flattening.md#df-show-cross-ref-flattening-grund-show-flattens-cross-reference-link-wrappers).

A wrapper is exactly what `fmt` recognizes as one ([§FS-citations.wrap.recognized](FS-citations.md#wraprecognized-an-existing-wrap-is-recognized-by-its-brackets),
[§FS-citations.wrap.enclosing.wrap-wins](FS-citations.md#wrapenclosingwrap-wins-a-wrap-inside-a-longer-text-is-a-wrap)); a link its author wrote around a citation, and an image,
are none and print as written ([§FS-citations.wrap.enclosing](FS-citations.md#wrapenclosing-a-citation-inside-a-link-its-author-wrote-is-left-to-the-author), [§FS-citations.wrap.enclosing.images](FS-citations.md#wrapenclosingimages-an-image-is-not-a-link)).
Nothing in a Markdown fence is flattened ([§FS-show.2.5](FS-show.md#25-a-heading-inside-a-fenced-code-block-is-an-example)), but only a Markdown body has fences: in
a source doc-comment, a value-source slice or an E2E manifest a wrapper is flattened even
between fence-looking lines. Flattening resolves nothing: a dangling wrapper is flattened
like any other, and `grund check` still reports it.

### 3.3 Batch query

A batch prints every well-formed query's outcome on stdout as one record, in query order, a
failure in its place among the answers, so no outcome hides a later one: `json` as one NDJSON
envelope per record ([§FS-output-shapes.4.1](FS-output-shapes.md#41-show---batch---formatjson)), `text` and `md` as a header above the record
([§FS-show.3.3.1](FS-show.md#331-the-header)). Stderr carries no per-query outcome in any format, only the run's warnings,
a glob's note ([§FS-coordinate-globs.errors.note](FS-coordinate-globs.md#errorsnote-a-pattern-no-kind-can-start-gets-a-note)) and a run-level error. The batch exits `0`
when every query succeeds, an empty one included, `1` after printing every record when any
fails, and `2` with empty stdout on a run-level failure ([§FS-show.2.6](FS-show.md#26-batch-resolution)). Decided in
[§DF-show-format-orthogonal](../decisions/functional/DF-show-format-orthogonal.md#df-show-format-orthogonal-shows---format-answers-every-query-form-and-a-text-or-md-batch-heads-each-answer--coordinate-).

#### 3.3.1 The header

Each record of a `text` or `md` batch opens with a line of its own, `==> <coordinate> <==`,
and the next line starts exactly the bytes the single read prints for that coordinate in that
format and mode; a single read prints no header. Every header but the first follows one
empty line: the previous record's bytes, a line break where they do not end in one, then one
more. Nothing follows the last record, and a declaration with no lead read in the default
mode gets its header alone. `md` frames as `text` does, so there the header renders as a
paragraph above the slice.

A succeeded query is headed by the coordinate it resolved to, spelled as `--all` generates
coordinates ([§FS-show.2.6](FS-show.md#26-batch-resolution)): the full ID, unqualified in the current project and prefixed
with its project's alias and `/` in any other, then any section path joined by the section
separator of the project that declares the ID ([§FS-config.3.2](FS-config.md#32-id--id-grammar)). So `{"id":"FS-042"}`
resolving to `FS-042-user-login` is headed `==> FS-042-user-login <==`, and section `1` of
`FS-api` in a member `api` whose separator is `:` is headed `==> api/FS-api:1 <==`, the
coordinate a reader pastes back into `grund show`. A failed query is headed by the query as
written, its `id` joined to a separate `section` by the separator of the configuration the
run discovered at its path, so `{"id":"FS-show","section":"9.9"}` is headed
`==> FS-show.9.9 <==`.

A body may hold any line, a header's shape included, so no text framing splits the answers
unambiguously: a program that separates them reads `--format json`
([§DF-show-format-orthogonal.2.2](../decisions/functional/DF-show-format-orthogonal.md#22-the-header-is--coordinate-)).

#### 3.3.2 A failure stays in its place

In `text` and `md`, a failed query's header is followed by its message, the bytes the JSON
envelope's `error.message` carries ([§FS-output-shapes.4.1](FS-output-shapes.md#41-show---batch---formatjson)), then the `hint:` lines the single
read prints for the same failure ([§FS-show.3.5](FS-show.md#35-the-hint-for-each-failure)), all on stdout and none with an `error:`
prefix. Each hint reads the configuration of the project its query names, from the batch's
one load. A query with a separate `section` fails as the single read naming it with
`--section` does, so `{"id":"FS-show","section":"9.9"}` is hinted `grund FS-show --toc` where
`{"id":"FS-show.9.9"}` is hinted `grund FS-show.9 --toc`. A failure the single read refuses
at launch, an unknown project alias among them, keeps its message and gets no hint, and the
two lines only the bare `grund <ID>` form prints, the subcommand hint and the `grund check`
breadcrumb, never appear. `json` keeps the failure inside its envelope, without hints
([§FS-output-shapes.4.1](FS-output-shapes.md#41-show---batch---formatjson)).

```text
==> FS-missing <==
ID not found: FS-missing
hint: run `grund list` to see every declared ID, or `grund id <KIND> "<title>"` to propose a new one

==> missing/FS-alpha <==
unknown project alias `missing`
known aliases: root, member
```

### 3.4 What a failed query prints

A failed single read exits `1` and prints one bare line on stderr ([§FS-errors.2.3](FS-errors.md#23-bare-query-failure)) and,
where the next step is obvious, its `hint:` lines below it ([§FS-show.3.5](FS-show.md#35-the-hint-for-each-failure)); stdout stays empty.
With `--format=json` stderr carries one finding object instead, its `path` and `line` `null`
([§FS-errors.5.2](FS-errors.md#52-on-stderr--what-is-not-output)). The failures are a
missing ID, a missing section, an invalid ID, an ambiguous ID ([§FS-show.2.2.1](FS-show.md#221-ambiguous-id)), an ambiguous
section ([§FS-show.2.2.2](FS-show.md#222-ambiguous-section)) and a broken stub ([§FS-show.2.3.4](FS-show.md#234-broken-stub)); the last three get no hint,
since their line already names the sites to edit. A failure that stops the read before it
runs exits `2` with an `error:` line: an I/O error, an invalid invocation, or a qualified ID
naming a project the run does not hold, `unknown project alias`, however many segments its
path has ([§FS-workspace.8.1](FS-workspace.md#81-grund-aliasid)).

### 3.5 The hint for each failure

Each hint is one line below the failure's message:

- `ID not found: <ID>` gets
  ``hint: run `grund list` to see every declared ID, or `grund id <KIND> "<title>"` to propose a new one``,
  except in a workspace run whose refusal already ends in a `did you mean <alias>/<ID>?`
  clause ([§FS-workspace.8.1.1](FS-workspace.md#811-an-unqualified-id-another-project-declares)).
- `section not found: <ID><sep><s>`, which names the declared ID without an alias, joined to
  the section by its project's separator, gets
  ``hint: run `grund <query> --toc` to print the lead with the section map``, where `<query>`
  is the query as written, alias included, cut before its last section separator: the
  separator of the project the query names. So in a member `api` whose separator is `:`,
  `grund api/FS-001-api:9` fails as `section not found: FS-001-api:9` and gets
  `grund api/FS-001-api --toc`.
- A missing snapshot of an ID whose kind carries `fetch` gets
  `hint: run grund fetch <qualified-ID>` ([§FS-check.4.12](FS-check.md#412-missing-snapshot), [§FS-fetch](FS-fetch.md#fs-fetch-grund-materializes-one-external-fact-snapshot),
  [§FS-show.requirements.next-step](FS-show.md#requirementsnext-step-a-failed-query-names-its-next-step)), unless the kind's `file` home is one [§FS-fetch.4](FS-fetch.md#4-file-home-write) refuses
  ([§FS-check.3.1.3.1](FS-check.md#3131-a-home-fetch-refuses)).
- ``invalid ID `<arg>` `` gets a hint naming the ID template the kind was read against and the
  key that sets it, the kind's override where it has one and else the global template:
  `[schema.kinds.<KIND>] id_format` or `[schema] id_format` in a version-2 config
  ([§FS-config-v2.schema.1](FS-config-v2.md#schema1-schema)), `[[kinds]] format` or `[id] format` in a version-1 config
  ([§FS-config.3.4.10](FS-config.md#3410-format-resolve-and-fetch--external-snapshot-kinds)), and the defaults' `[id] format` when none is loaded. Each project of a
  workspace is asked in its own grammar ([§FS-workspace.8.1.1.4](FS-workspace.md#8114-each-project-is-asked-in-its-own-grammar)). A section the grammar
  rejects, `FS-widget.1.nope` where sections are numbered only, makes the ID invalid, while a
  section that parses but is absent is `section not found`:

  ```text
  invalid ID `FS-widget.1.nope`
  hint: this repo's [id] format is `{kind}-{slug}` (run `grund config show`); `grund list` shows the IDs that exist
  ```

  When no config was loaded but a project config lies below the root
  ([§FS-check.2.2.2](FS-check.md#222-the-message-names-the-likely-cause)), the template is not the cause: the one hint line
  names those configs and the routes `check`'s message offers ([§FS-check.2.2.2](FS-check.md#222-the-message-names-the-likely-cause)), without
  `grund check`, in place of the template and the `grund list` route:

  ```text
  invalid ID `FS-login`
  hint: no grund.toml applies here, so grund read `FS-login` against its defaults. `suite/grund.toml` declares a project this run does not read: run grund from `suite/`, or add a grund.toml here whose [workspace] has members = ["suite"] and include_root = false
  ```

- In the bare `grund <arg>` form alone, an `<arg>` refused as an unknown project alias that
  exists as a path relative to the working directory is refused as an invalid ID instead,
  exit `1`, with the invalid-ID hint and then
  ``hint: run `grund check <arg>` to validate a path``. No other refusal gets that line.
- `error: path does not exist: <path>`, for the operand `show` reads as the path to scan
  ([§FS-cli.3.2](FS-cli.md#32-at-most-one-path)), gets a hint appended as [§FS-errors.3](FS-errors.md#3-message-text) permits when `<path>` is the positional
  operand, nothing exists there, and it is coordinate-shaped: an unqualified ID, optionally
  with an inline section, that the grammar of the configuration discovered from the working
  directory accepts, or else the default grammar, never a spelling with a `/`. Only its shape
  is checked, and nothing is resolved. Only such a path gets the hint, and never in a batch.
  It is built from the read's own coordinates, `--section` included, and asks the batch for
  the read's format; stdout stays empty, the exit stays `2`, and stderr stays raw text under
  `--format=json` ([§FS-errors.2.2](FS-errors.md#22-cli-level-message), [§FS-cli.5](FS-cli.md#5-exit-code-mapping-is-fixed)). So `grund FS-alpha FS-beta` prints:

  ```text
  error: path does not exist: FS-beta
  hint: `show` reads one coordinate, so `FS-beta` is the path; read both with `printf '%s\n' '{"id":"FS-alpha"}' '{"id":"FS-beta"}' | grund show --batch`
  ```

## 4. Why this matters

Without `show`, an agent that needs one section of a declaration either loads the whole file
or reimplements the parser. With it, `grund FS-check.3.1` pulls exactly that section into a
prompt, and a citation with no section costs its lead until the reader asks for the map or
the body: an agent pays for the fact it needs, not the file it lives in. This is the
agent-grounding loop: declarations live in one place, and any agent can fetch one, or just its
lead, or just its map, with a single command.
