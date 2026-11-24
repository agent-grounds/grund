# FS-list: grund lists every declared ID

The `list` subcommand prints the repository's ID catalog, every declaration with its home and
title, or the slice a filter selects: as rows, as one count per kind, as one total, or as each
coordinate's lead and body sizes. It answers "what IDs are there?" where `grund refs` answers
"who cites this one?", so an agent or a person can find the IDs to read. Serves
[§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible), [§GOAL-token-economy](../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file), and the agent-grounding loop in
[§GRUND-grund](../grund.md#grund-grund-agents-stay-grounded-in-the-spec).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, home, citable, body, section, coordinate,
lead, index, catalog), [§FS-terms.terms.2](FS-terms.md#terms2-citations) (citation), [§FS-terms.terms.3](FS-terms.md#terms3-source-forms) (source declaration,
stub, doc-comment), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, scope, workspace, member, alias),
[§FS-terms.terms.6](FS-terms.md#terms6-rules-and-directions) (rule), and [§FS-terms.terms.7](FS-terms.md#terms7-values-and-integrations) (value, fetcher, snapshot).

- **coordinate-size** — One row of `--size`, for one declaration or citable
  section: its lead and full-body measurement, or the site alone where there
  is nothing to measure.

## requirements: What every listing holds to

What `list` holds to with every other command is stated once, where they share it: it lists
exactly the declarations and sections `check` validates and `show` resolves, and counts what
`refs` and `show` answer ([§REQ-one-answer.contract](../requirements/REQ-one-answer.md#contract-one-question-one-tree-one-answer), [§REQ-one-answer.identity.declarations](../requirements/REQ-one-answer.md#identitydeclarations-every-command-reads-one-declaration-set),
[§REQ-one-answer.sections.map](../requirements/REQ-one-answer.md#sectionsmap-every-surface-reads-one-section-map)); it reads offline, writes nothing and prints the same bytes on
every run ([§REQ-runs-offline.1](../requirements/REQ-runs-offline.md#1-read-and-verification-paths-execute-nothing), [§REQ-no-data-loss.1](../requirements/REQ-no-data-loss.md#1-the-query-surface-never-writes), [§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes)); and a
narrowed listing is a rung of the escalation ladder ([§GOAL-token-economy.1](../goals.md#1-what-this-requires)).

### requirements.flagged-rows: A finding marks a row and never removes one

Whatever `check` reports about a declaration, `list` still shows it and picks no winner: a
duplicated ID, a broken stub and an off-grammar declaration keep their rows ([§FS-list.2](FS-list.md#2-behaviour)), and
so does each claimant of a duplicated section in size mode ([§FS-list.3.4](FS-list.md#34---size--per-coordinate-lead-and-full-body-measurements)). No finding fails
the run ([§FS-list.4](FS-list.md#4-exit-codes)). The located error is left to `check`, so the catalog shows the
situation `check` judges, never a tree tidied of what it objects to. Serves [§REQ-no-wrong-citation.1](../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution) and
[§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work).

### requirements.one-selection: Every filter narrows every mode alike

The filters make one selection and every output mode reads it: `--kind`, `--project`,
`--unused`, `--selector` and `--glob` narrow the per-declaration rows, the `--summary`
counts, the `--total` count and the `--size` rows alike, compose with one another by
intersection, and apply before `--top` keeps its largest rows ([§FS-list.1.2](FS-list.md#12---project), [§FS-list.3.3](FS-list.md#33---summary),
[§FS-list.3.4.5](FS-list.md#345-order-and---top), [§FS-rules.8](FS-rules.md#8-command-surfaces)). A refusal reads it too: the `known kinds:` line is built from
the same selection in every mode ([§FS-list.1.1](FS-list.md#11---kind)). It binds the next filter as well: a filter
added to `list` narrows this one selection, so it reaches every mode at once. Serves
[§GOAL-token-economy.1](../goals.md#1-what-this-requires).

## 1. Inputs

```
grund list [<path>] [--selector <selector>] [--glob <pattern>]… [--kind <KIND>[,<KIND>…]]… [--project <alias>[,<alias>...]] [--unused]
           [--summary | --total] [--size[=<unit>[,<unit>…]]] [--top <N>] [--format text|json]
```

Which entries `<path>` reports, and what they resolve against, is [§FS-scan.path.list](FS-scan.md#pathlist-list-reports-the-paths-entries-and-resolves-against-the-project).
`--format` picks the output shape: `text` by default ([§FS-list.3.1](FS-list.md#31---format-text-default)) or `json` ([§FS-list.3.2](FS-list.md#32---format-json)).

`--selector <selector>` keeps the declaration and named-chapter units a rule subject selects
([§FS-rules.8](FS-rules.md#8-command-surfaces)), and `--glob <pattern>`, repeatable, the catalog coordinates its patterns match
([§FS-coordinate-globs](FS-coordinate-globs.md#fs-coordinate-globs-a-coordinate-glob-selects-catalog-coordinates-by-pattern)). Both are flags, so the positional stays the scan path
([§FS-coordinate-globs.errors.hint](FS-coordinate-globs.md#errorshint-a-glob-typed-as-path-gets-a-hint)), and they compose as [§FS-coordinate-globs.scope](FS-coordinate-globs.md#scope-scope) says.

### 1.1 `--kind`

`--kind <KIND>[,<KIND>…]` lists only declarations of the named kinds, each a configured
*citable* kind ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)); comma-separated or repeated, the kinds union. A kind is
accepted when any selected project has it citable ([§FS-scan.projects.kinds](FS-scan.md#projectskinds-whose-kinds-a-run-reads)).

An unknown kind anywhere in the selection is a CLI-level error ([§FS-list.4](FS-list.md#4-exit-codes)), so a typo never
reads as an empty or short catalog. A configured non-citable kind ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)), and
`code` where no row declares the homeless kind ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)), are refused the same way
with [§FS-errors.2.5.1](FS-errors.md#251-a-kind-that-declares-no-ids)'s reason in place of `unknown kind`. Each refusal ends in a
`known kinds:` line naming the citable kinds of the selected projects ([§FS-scan.projects.kinds](FS-scan.md#projectskinds-whose-kinds-a-run-reads)).
Where that scope has no citable kind, as under a config that declares none
([§FS-config.3.4.4](FS-config.md#344-the-default-kinds)), the line names none rather than ending empty, in every refusal that
prints it: `--kind`, `--selector` ([§FS-rule-refusals.selectors](FS-rule-refusals.md#selectors-a-refused-selector-is-answered-with-a-selector)) and `grund id` ([§FS-id.1](FS-id.md#1-inputs)). There
`grund list --kind FS` prints:

```text
error: unknown kind `FS`
known kinds: none — declare one in grund.toml first
```

### 1.2 `--project`

`--project <alias>[,<alias>...]` narrows a workspace catalog to the named projects, and an
unknown alias is refused as an unknown `--kind` is ([§FS-workspace.8.3.2](FS-workspace.md#832---project)); outside workspace
mode the flag is a CLI-level error, exit `2` ([§FS-list.4](FS-list.md#4-exit-codes)). It narrows the kinds `--kind`
accepts too, so a kind only another loaded project configures is unknown here
([§FS-scan.projects.kinds](FS-scan.md#projectskinds-whose-kinds-a-run-reads)). Workspace qualification, size behavior, and member-local invocation are
specified in [§FS-workspace.8.3](FS-workspace.md#83-grund-list).

### 1.3 `--unused`

`--unused` lists only the declarations no recognized citation points at, as `check` warns
([§FS-check.4.1](FS-check.md#41-unused-declaration)), a kind's own index entry counting as no use, within the bounds `check` draws ([§FS-check.4.1](FS-check.md#41-unused-declaration)). It leaves out the cases of a
configured `E2E` kind ([§FS-config.3.4.4](FS-config.md#344-the-default-kinds)) unless `--kind` selects `E2E` explicitly, since a
case is exercised by being run, not cited ([§FS-check.4.1](FS-check.md#41-unused-declaration)). The default kinds declare no
`E2E` kind ([§FS-config.3.4.4](FS-config.md#344-the-default-kinds)), so `grund list --help` ([§FS-cli.2.3](FS-cli.md#23-a-subcommands-help-page)) states that rule for a
configured `E2E` kind, and its example of `--unused` with `--kind` selects a default kind,
such as `grund list --unused --kind FS`, never `--kind E2E`.

### 1.4 `--summary`

`--summary` prints [§FS-list.3.3](FS-list.md#33---summary)'s per-kind counts in place of the rows.

### 1.5 `--size`

`--size[=<unit>[,<unit>…]]` prints [§FS-list.3.4](FS-list.md#34---size--per-coordinate-lead-and-full-body-measurements)'s rows in place of the declaration rows. A value is read only in the same argument, after `=`, so a following bare
word stays the `<path>`. Bare `--size` selects `lines,words,bytes`, in that order; an
explicit list is case-sensitive, keeps the caller's order and drops a repeated unit after its
first occurrence. The units are `lines`, `words` and `bytes` ([§FS-list.3.4.2](FS-list.md#342-units)) and no other:
an empty item or any other value, `tokens` included, is a CLI-level error ([§FS-list.1.7](FS-list.md#17-size-selector-errors)).

### 1.6 `--top`

`--top <N>` or `--top=<N>` keeps at most the `N` rows with the largest lead in the first
selected unit, after every filter ([§FS-list.3.4.5](FS-list.md#345-order-and---top)). `N` is one positive base-10 integer.

### 1.7 Size-selector errors

The syntax and combination errors of `--size` and `--top` are decided before config discovery
or scanning. Each is one raw `error:` line on stderr, exit `2` ([§FS-list.4](FS-list.md#4-exit-codes)):
`--size may only appear once`, `--size requires one or more units`,
``unknown size unit `<unit>` (expected lines, words, or bytes)``, `--top may only appear once`,
`--top requires a positive integer`, `--top requires --size`,
`--size cannot be combined with --summary`, or `--top cannot be combined with --summary`.

## 2. Behaviour

`list` prints one row for every declaration the scan finds, the entries of an opted-in kind's
value sources ([§FS-values.6.2](FS-values.md#62-value-source-declarations-in-the-catalog)) and the cases of a configured
`E2E` kind included, and one for every citable section in size mode ([§FS-list.3.4](FS-list.md#34---size--per-coordinate-lead-and-full-body-measurements)) and
wherever `--glob` or `--selector` selects one ([§FS-coordinate-globs.matching](FS-coordinate-globs.md#matching-matching),
[§FS-rules.8](FS-rules.md#8-command-surfaces)). A
value root stays metadata on its declaration's row, never a row of its own ([§FS-values.6.1](FS-values.md#61-marked-roots-in-the-catalog)).
A fetched kind is listed from its committed snapshot, never by running its fetcher, so a
missing snapshot is no row ([§FS-config.3.4.10](FS-config.md#3410-format-resolve-and-fetch--external-snapshot-kinds), [§REQ-runs-offline.1](../requirements/REQ-runs-offline.md#1-read-and-verification-paths-execute-nothing)). An ID that is cited
but declared nowhere has no row either; `refs` and `check` report it.

A stub that pairs with its target is no row of its own: its ID is listed at the target's
declarations, scanned or not ([§FS-declarations.stubs.pairing](FS-declarations.md#stubspairing-how-a-stub-pairs-with-what-it-points-at)). A broken stub is a row at its own site, noted
with its target ([§FS-list.3.1](FS-list.md#31---format-text-default)). An ID declared in several independent homes
([§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration)) is one flagged line per home, and an off-grammar
declaration retained under [§FS-config.3.2](FS-config.md#32-id--id-grammar) a normal row, rendered as written: `list` picks no
winner and drops no row ([§FS-list.requirements.flagged-rows](FS-list.md#requirementsflagged-rows-a-finding-marks-a-row-and-never-removes-one)).

### 2.4 Order

Rows come out sorted by ID: kind name, then number, then slug. Names compare in byte order and numbers by value, and an ID with no number comes first, so `FS-9` lists before `FS-10`, and `FS` before `GOAL` whatever order the kinds are configured in. A duplicated ID's homes follow by path, then line, and a declaration's section rows follow it in byte order of their path. In a workspace, rows sort by alias path first, in byte order.

## 3. Outputs

### 3.1 `--format text` (default)

One line per catalog entry on **stdout**:

```
AR-001-auth     docs/architecture/AR-001-auth.md:1  The auth module
FS-001-login    docs/functional-spec/FS-001-login.md:1  User can log in
FS-002-session  docs/functional-spec/FS-002-session.md:1  A session is created on login
```

The columns are the ID, in the repository's `[id] format` or in its kind's own `format`
([§FS-config.3.4.10](FS-config.md#3410-format-resolve-and-fetch--external-snapshot-kinds)) and qualified as `<alias>/<ID>` in a workspace ([§FS-workspace.8.3.1](FS-workspace.md#831-the-id-column-is-always-qualified)), left-padded so the column aligns, up to a capped width; then the `<path>:<line>` of the home declaration; then the title. A
title, on a declaration row and a section row alike, is the heading's text after its
coordinate, as written with its links kept, without a closing `#` run
([§FS-declarations.heading.rendered-text](FS-declarations.md#headingrendered-text-a-headings-rendered-text)). It is empty where the heading has nothing after its
coordinate, and on an E2E case, which no heading declares ([§FS-declarations.e2e-case.id](FS-declarations.md#e2e-caseid-a-case-directory-and-the-id-it-declares)). A
broken stub shows `→ <target>` in place of a title, a duplicated ID's lines carry a
`(duplicate declaration — grund check)` note, and a row with value roots, marked
([§FS-values.2.4](FS-values.md#24-embedded-section-value-roots)) or chapter-declared ([§FS-values.2.5](FS-values.md#25-chapter-declared-value-roots)), appends
` [value roots: <ID.path>, <ID.path> (invalid)]`, ordered by canonical section path and not by
origin. An empty selection prints nothing and exits `0`, in every mode but `--total`
([§FS-list.total](FS-list.md#total---total-a-count-rung)). Stderr is empty on success.

### 3.2 `--format json`

NDJSON on stdout: one object per row, each the record of [§FS-output-shapes.5](FS-output-shapes.md#5-list---formatjson).

Each row's `refs` field counts the recognized citations of the row's coordinate, exactly the sites `grund refs`
lists for it, a section's without its descendants ([§FS-refs.2](FS-refs.md#2-behaviour)). It is not the `--unused`
predicate: where a kind's index is checked ([§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index)) the count includes the ID's index
entry, which `--unused` discounts ([§FS-list.1.3](FS-list.md#13---unused)), so a row `--unused` selects may carry
`refs: 1`, and a tool looking for uncited IDs passes `--unused` rather than thresholding on
`refs` ([§DF-index-not-an-inbound-citation.2.3](../decisions/functional/DF-index-not-an-inbound-citation.md#23-refs-in-grund-list---format-json-stays-the-total)).

A row with value roots of either origin adds
`"value_roots":[{"id":"FS-001-pricing.1","valid":true},{"id":"FS-001-pricing.2","valid":false}]`,
in canonical section-path order and recording no origin, and omits the member, rather than
emitting an empty array, when the declaration has none. No root changes `refs`.

### 3.3 `--summary`

`grund list --summary` prints one line per kind, in the configured kind order, for the kinds
that have at least one declaration in the selection:

```
FS      2  docs/functional-spec
AR      1  docs/architecture
```

The columns are the kind; the count of its declarations after every filter
([§FS-list.requirements.one-selection](FS-list.md#requirementsone-selection-every-filter-narrows-every-mode-alike)), where each home of a duplicated ID counts once and a
declaration counts once however many of its rows `--glob` or `--selector` selects; and its
home: the kind's places in the row's place order ([§FS-config-v2.schema.places](FS-config-v2.md#schemaplaces-rows-and-their-places)),
comma-separated, each spelled as `list` spells a path ([§FS-cli.3.4](FS-cli.md#34---path-base--where-report-paths-are-spelled-from)), so a member's place
carries the member's directory. In a workspace each line is one project's kind, led by its
alias ([§FS-workspace.8.3.4](FS-workspace.md#834---summary)). `--format json` prints
[§FS-output-shapes.5](FS-output-shapes.md#5-list---formatjson)'s summary record.

### 3.4 `--size` — per-coordinate lead and full-body measurements

Size mode prints one row for each declaration site and each citable section site, with its
lead and full body measured in the selected units. A stub that pairs with its target has no
row of its own: its rows are the ones the same tree prints with the target scanned, wherever
the target's file sits, each measured as `show` reads it and in JSON with `stub` `false` and
`defines` `null` ([§FS-declarations.stubs.pairing](FS-declarations.md#stubspairing-how-a-stub-pairs-with-what-it-points-at),
[§FS-show.2.3](FS-show.md#23-source-declarations-in-code-and-doc-comments)), so a target that declares the ID twice gives two measured rows marked
duplicate, never one unmeasured row at the stub. The unmeasured `(broken stub → <target>)` row
of [§FS-list.3.4.3](FS-list.md#343-text-rows), at the stub's own site, is exactly a stub `check` reports broken
([§FS-declarations.stubs.verdict](FS-declarations.md#stubsverdict-one-verdict-per-stub-reached-once-per-scan)). Measuring a home outside the scan does not bring its file into
the scan, so `check`'s lead warning judges no site there
([§FS-declarations.checks.oversized-lead](FS-declarations.md#checksoversized-lead-oversized-lead-opt-in)). Duplicate declaration homes and the sites that
claim one duplicated section coordinate stay separate rows, each marked duplicate and measured
from its own site only, never merged and none taken as the winner, and `show` still refuses
the ambiguous coordinate ([§FS-show.2.2.2](FS-show.md#222-ambiguous-section)). JSON rows are NDJSON, each the record
of [§FS-output-shapes.5.2](FS-output-shapes.md#52-list---size---formatjson), with `section` `null` on a declaration row.

#### 3.4.1 Measured strings

The measured strings are exactly the text bodies `show` returns for that site in its default
and `--full` modes ([§FS-show.2.1](FS-show.md#21-whole-declaration-default), [§FS-show.2.3](FS-show.md#23-source-declarations-in-code-and-doc-comments), [§FS-show.3.2](FS-show.md#32-cross-reference-links-are-flattened-in-text-and-json)): a declaration's without its
declaration heading, and a section's with its own heading as `show` prints it, so a selected
relative heading is measured with its canonical path
([§FS-declarations.section.relative.shown](FS-declarations.md#sectionrelativeshown-how-a-relative-heading-is-shown)). A value-source declaration or section
row measures its exact available source slice, so its lead equals its full body.

#### 3.4.2 Units

Every unit is defined over the UTF-8 bytes of that measured string, with no locale or Unicode-table input:

- `lines` is the count of non-blank LF-delimited segments, including a final segment without a terminating LF. A segment is blank only when it contains ASCII whitespace bytes (`09`–`0D`, `20`).
- `words` is the count of maximal non-empty byte runs separated by those same ASCII whitespace bytes.
- `bytes` is the UTF-8 byte length.

#### 3.4.3 Text rows

Text output is headerless and line-oriented:

```text
<coordinate>  <path>:<line>  <unit>=<lead>/<full> [<unit>=<lead>/<full> ...]
```

The coordinate and location use the ordinary list padding ([§FS-list.3.1](FS-list.md#31---format-text-default)) and relative-path
rules ([§FS-cli.3.4](FS-cli.md#34---path-base--where-report-paths-are-spelled-from)). A duplicate row ends in `  (duplicate — site-local)`. An unmeasurable broken stub instead renders every selected pair as `-/-` and ends in `  (broken stub → <target>)`.

#### 3.4.5 Order and `--top`

Rows come in [§FS-list.2.4](FS-list.md#24-order)'s order. Every filter applies before `--top`, and `--unused` selects
declarations by its ordinary rule and keeps every section row of a declaration it selects.
`--top` sorts descending by the lead value of the first requested unit, with normal row order
as the total tie-break, and leaves out rows without a measurement.

## total: `--total`, a count rung

`--total` prints one count in place of the rows: the number of rows `list` would print under
the same path, `--selector`, `--glob`, `--kind`, `--project` and `--unused`. Text is the bare
number and a newline, comparable with `wc -l` of the rows ([§DF-coordinate-globs.1](../decisions/functional/DF-coordinate-globs.md#1-context-and-decision)), JSON the
count object of [§FS-output-shapes.5](FS-output-shapes.md#5-list---formatjson), and an empty selection prints `0`. Beside `--summary`, `--size` or `--top` it
is a launch error, decided before config discovery and before the size-selector errors
([§FS-list.1.7](FS-list.md#17-size-selector-errors), [§FS-cli.4](FS-cli.md#4-errors-with-no-source-location)), in that order: `--total cannot be combined with --summary`,
`--total cannot be combined with --size`, `--total cannot be combined with --top`. Repeating
it is harmless.

## 4. Exit codes

- `0` — the scan succeeded; the listed catalog, possibly empty, is the result.
- `2` — a scan or I/O error, an incomplete scan included, whose catalog may be short
  ([§FS-check.2.4](FS-check.md#24-an-incomplete-run)); an unknown `--kind`, any value of a comma-separated or repeated one; an
  invalid size selector or combination ([§FS-list.1.7](FS-list.md#17-size-selector-errors)); a refused coordinate glob
  ([§FS-coordinate-globs.errors](FS-coordinate-globs.md#errors-refusals-notes-and-hints)); an unsupported `--format`; or any other CLI-level error
  ([§FS-cli.4](FS-cli.md#4-errors-with-no-source-location)).

There is no `1`: unlike `show`, `list` has no single result to miss, so a possibly empty
catalog is always its answer.

## 5. Why this exists

`grep -RhoE '^#+ [A-Z]+-[a-z0-9-]+'` across `docs/` finds declaration headings, but not
source declarations in doc-comments; it cannot collapse a stub onto its target, honour a
customised `[id]` grammar or tell which declarations are uncited, and gives an agent no stable
shape to program against.
