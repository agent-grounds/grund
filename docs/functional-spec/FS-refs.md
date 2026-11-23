# FS-refs: grund lists every citation of an ID

`grund refs <ID>` lists every site that cites an ID: the reverse of `grund <ID>`, for an
author about to change or delete a declaration who needs to know what leans on it. `--glob`
asks it of every coordinate a pattern matches ([§FS-refs.globs](FS-refs.md#globs---glob-lists-who-cites-a-set)), and `--from` turns it round to
list what one point cites ([§FS-refs.from](FS-refs.md#from---from-lists-what-one-point-cites)). Serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible),
[§GOAL-token-economy](../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file), and [§GRUND-grund](../grund.md#grund-grund-agents-stay-grounded-in-the-spec)'s loop, in which an agent verifying a change reads the
cited bodies and the citations that reach them.

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, body, section, coordinate, catalog),
[§FS-terms.terms.2](FS-terms.md#terms2-citations) (marker, citation, shorthand, citation site), [§FS-terms.terms.3](FS-terms.md#terms3-source-forms) (doc-comment),
[§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, workspace, alias), [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding), and [§FS-terms.terms.7](FS-terms.md#terms7-values-and-integrations)
(value, binding, snapshot).

## requirements: What every citation lookup holds to

What `refs` holds to with every other command is stated once, where they share it: it lists
exactly the citations `check` validates and refuses a duplicate home as `show` does
([§REQ-one-answer.contract](../requirements/REQ-one-answer.md#contract-one-question-one-tree-one-answer), [§REQ-one-answer.identity.declarations](../requirements/REQ-one-answer.md#identitydeclarations-every-command-reads-one-declaration-set)); it refuses an ambiguous
operand with its candidates rather than guess ([§REQ-no-wrong-citation.1](../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution)); it reads offline,
writes nothing and prints the same bytes on every run ([§REQ-runs-offline.1](../requirements/REQ-runs-offline.md#1-read-and-verification-paths-execute-nothing),
[§REQ-no-data-loss.1](../requirements/REQ-no-data-loss.md#1-the-query-surface-never-writes), [§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes)); and `--summary` and `--total` are rungs of
the escalation ladder ([§GOAL-token-economy.1](../goals.md#1-what-this-requires)).

### requirements.one-set: Every flag answers from one citation set

The invocation fixes one set: every citation of the ID, narrowed by `--section` or the dotted
form and widened by `--descendants` ([§FS-refs.1](FS-refs.md#1-inputs), [§FS-refs.2](FS-refs.md#2-behaviour)); every site whose cited
coordinate a `--glob` matches ([§FS-refs.globs](FS-refs.md#globs---glob-lists-who-cites-a-set)); or every site the `--from` source owns
([§FS-refs.from](FS-refs.md#from---from-lists-what-one-point-cites)). `--summary` folds that set to a row per file and `--total` to one row
([§FS-refs.3.3](FS-refs.md#33---summary), [§FS-refs.3.4](FS-refs.md#34---total)), and neither adds a refusal or an exit status: each inherits the
result of the operand that fixed the set ([§FS-refs.4](FS-refs.md#4-exit-codes)). So `--section <s> --descendants --total`
is the count an editor shows on that section's heading ([§FS-lsp.1.2.6](FS-lsp.md#126-what-a-section-heading-counts)). A flag added to `refs`
narrows or widens that set, or folds over it, and inherits the operand's result. Serves
[§GOAL-token-economy.1](../goals.md#1-what-this-requires).

### requirements.unresolved: A citation is listed whether or not it resolves

Whether a site reaches anything is `check`'s question, never a reason to drop it: the
citations of an ID nothing declares are listed, the dangling ones `check` reports, and so are
those of a section no heading declares ([§FS-refs.2](FS-refs.md#2-behaviour)) and of a stub that cannot be paired
([§FS-refs.4.1](FS-refs.md#41-a-stubs-homes-are-found-in-its-target)); a missing fetched snapshot lists the same sites before it is fetched as after,
without running its fetcher ([§REQ-runs-offline.1](../requirements/REQ-runs-offline.md#1-read-and-verification-paths-execute-nothing)). In ID mode `refs` refuses only an operand it
cannot name one target for (a grammar rejection, a duplicate home, a duplicate on the selected
section), and deciding that needs no declaration to exist ([§FS-refs.4](FS-refs.md#4-exit-codes)). Serves
[§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration).

## 1. Inputs

```
grund refs <ID> [<path>] [--section <s>] [--descendants] [--summary] [--total] [--format text|json]
grund refs --glob <pattern> [--glob …] [<path>] [--summary] [--total] [--format text|json]
grund refs --from <coordinate> [<path>] [--summary] [--total] [--format text|json]
```

- `<ID>` — the ID without the marker, with an optional inline section after the configured
  `[id] section_separator` (`FS-check.3.1`, or `FS-plan.goals.performance` where named
  sections are enabled), parsed by the named kind's effective format against the selected
  project's catalog ([§FS-config.3.2](FS-config.md#32-id--id-grammar)). An exact off-grammar declaration is a valid query
  ([§FS-config.3.2.5](FS-config.md#325-off-grammar-declarations-stay-readable)), and the number-only shorthand is accepted where the format has one
  ([§FS-citations.shorthand.cli](FS-citations.md#shorthandcli-the-shorthand-as-a-cli-id-argument)).
- `<path>` — what it scans is [§FS-scan.path.refs](FS-scan.md#pathrefs-refs-scans-the-paths-tree), in every mode.
- `--glob <pattern>` — list every site whose cited coordinate the pattern matches, instead of
  one ID's ([§FS-refs.globs](FS-refs.md#globs---glob-lists-who-cites-a-set)). Repeatable, and the patterns union
  ([§FS-coordinate-globs.matching.union](FS-coordinate-globs.md#matchingunion-repeated-globs-union)).
- `--from <coordinate>` — list the sites the coordinate's own body holds, instead of the ones
  that cite it ([§FS-refs.from](FS-refs.md#from---from-lists-what-one-point-cites)). Given once, spelled as for `grund <coordinate>`, its value the next
  argument or after `=`.
- `--section <s>` — list only the citations of exactly that numeric or named section path,
  `3.1` or `goals.performance`. Mutually exclusive with the dotted inline form; without
  either, every citation of `<ID>` is listed, bare-ID ones included.
- `--descendants` — widen the section filter from the exact coordinate to it and every section
  beneath it, however the section was spelled ([§FS-refs.2](FS-refs.md#2-behaviour)): the set an editor returns on that
  section's heading ([§FS-lsp.1.3.1](FS-lsp.md#131-citations-from-declarations)). On a query with no section it is a no-op, never a usage
  error, so a caller may pass it with an operand it did not build.
- `--summary` — one line per citing **file** instead of one per site ([§FS-refs.3.3](FS-refs.md#33---summary)).
- `--total` — the size of the set instead of its members: sites and files, one line
  ([§FS-refs.3.4](FS-refs.md#34---total)).
- `--format text|json` — output shape ([§FS-refs.3](FS-refs.md#3-outputs)). Default `text`.

### 1.1 Three modes, one path

`--glob` and `--from` each select a mode; without either, `refs` is in ID mode, which takes
`<ID>` and then at most one `<path>`. Under `--glob` or `--from` every positional is the path,
wherever it stands on the line, and with none the run covers the project
([§FS-scan.path.none](FS-scan.md#pathnone-a-run-with-no-path-covers-the-project-discovery-finds)): `grund refs FS-login --glob 'FS-audit.**'` scans a path named
`FS-login` and filters on no ID ([§FS-refs.1.3](FS-refs.md#13-an-id-typed-as-the-path-gets-a-hint), [§DF-citation-site-queries.2](../decisions/functional/DF-citation-site-queries.md#2-the-rulings-it-carries)). `--section` and
`--descendants` belong to ID mode; `--summary`, `--total` and `--format` work in every mode.

### 1.2 Refusals

Each is a usage error ([§FS-cli.4](FS-cli.md#4-errors-with-no-source-location)), answered before config discovery, so a broken `grund.toml`
hides none of them ([§FS-cli.requirements.before-load](FS-cli.md#requirementsbefore-load-a-usage-mistake-is-answered-before-the-tree-is-read--realized-directional-for-the-next-flag)). A missing value is reported where the
command line reads it; once the whole line is read, a bad `--format` is answered first
([§FS-cli.3.5](FS-cli.md#35-a-bad-run-flag-is-answered-before-the-load)), then the first row of this table that applies, and only then a pattern's own
refusal ([§FS-coordinate-globs.errors.shape](FS-coordinate-globs.md#errorsshape-one-refusal-line)).

| Command line | Refusal |
|---|---|
| `refs --from` or `refs --from=` | `error: --from requires a value` |
| `refs --from FS-login --from FS-audit` | `error: --from may only appear once` |
| `refs --glob 'FS-*' --from FS-login` | `error: --glob cannot be combined with --from` |
| `refs --glob 'FS-login' --section 3` | `error: --section cannot be combined with --glob; accepted glob: "FS-login.3"` |
| `refs --glob 'FS-login' --descendants` | `error: --descendants cannot be combined with --glob; accepted glob: "FS-login{,.**}"` |
| `refs --glob 'FS-login' --section 3 --descendants` | `error: --section cannot be combined with --glob; accepted glob: "FS-login.3{,.**}"` |
| `refs --from FS-login --section 3` | `error: --section cannot be combined with --from` |
| `refs --from FS-login --descendants` | `error: --descendants cannot be combined with --from` |
| `refs --glob 'FS-*' docs src`, `refs --from FS-login docs src` | `error: refs takes at most one path argument` |

The accepted glob ([§FS-errors.2.5](FS-errors.md#25-refused-input)) is offered only beside exactly one `--glob`: the pattern as
typed, then `.<s>` for `--section`, with every operator in `<s>` escaped, then `{,.**}` for
`--descendants`, and only when the result passes every check the grammar makes before
discovery ([§FS-coordinate-globs.errors.shape](FS-coordinate-globs.md#errorsshape-one-refusal-line)). Otherwise, and always beside `--from`, the line
ends after the flag's name. `--glob` with no value is [§FS-coordinate-globs.errors.flags](FS-coordinate-globs.md#errorsflags-flag-errors). ID
mode refuses a missing ID with `error: refs requires an ID` and a second path with
`error: refs takes an ID and at most one path argument`.

### 1.3 An ID typed as the path gets a hint

Under `--glob` or `--from`, a positional that names no path, holds none of `*`, `?`, `[` or
`{`, and is a valid pattern ([§FS-coordinate-globs.grammar](FS-coordinate-globs.md#grammar-grammar)) gets one hint line after the
missing-path error:

```text
$ grund refs FS-login --glob 'FS-audit.**'
error: path does not exist: FS-login
hint: with `--glob`, `refs` reads `FS-login` as a scan path; to add its citations, use `--glob 'FS-login{,.**}'`
$ grund refs FS-login --from FS-audit
error: path does not exist: FS-login
hint: with `--from`, `refs` reads `FS-login` as a scan path; to list its citations, use `grund refs FS-login`
```

The grammar is read with the `.` separator and the default marker, because no config is read
for a path that does not exist, as for `list` ([§FS-coordinate-globs.errors.hint](FS-coordinate-globs.md#errorshint-a-glob-typed-as-path-gets-a-hint)).

## 2. Behaviour

`refs <ID>` lists every citation site of the ID, exactly the set `check` validates
([§REQ-one-answer.contract](../requirements/REQ-one-answer.md#contract-one-question-one-tree-one-answer)): what counts as a citation, in prose, in a doc-comment or in a
value binding, and what does not, such as an ID-shaped string literal, is
[§FS-citations.recognized](FS-citations.md#recognized-recognized-citations) ([§FS-values.3.2](FS-values.md#32-recognized-text-contexts), [§FS-check.1.1.3](FS-check.md#113-string-literals-in-source-files)). It never lists the declaration
itself.

The set also holds each citation that a stub's target outside `[scan] include` lends the
declaration the stub pairs with, at its line in the target ([§FS-declarations.stubs.target.5](FS-declarations.md#stubstarget5-what-a-target-outside-the-include-gives-and-what-stays-unjudged)).
Unlike a scanned citation, one there that resolves to nothing is not lent, so it is listed
nowhere. A declaration-local numeric citation ([§FS-check.1.1.8](FS-check.md#118-declaration-local-numeric-section-candidates)) is listed, with its authored
token, under the full ID and section it resolves to, and an ownerless, ambiguous or
unsupported one under none.

With `--descendants` the section filter keeps a citation whose section is the requested path or
lies beneath it, by whole dotted components and never by string prefix, whatever the outer
`section_separator` ([§FS-config.3.3.4](FS-config.md#334-the-outer-section-separator)): `3.16` reaches `3.16.1` and every depth below it and
never `3.160`, and `checks.duplicate` never reaches `checks.duplicate-section`. A cited section
no heading declares counts, because it is exactly the citation a move of its parent breaks.

Sites go to **stdout** in either format ([§FS-errors.1](FS-errors.md#1-streams)), sorted by `(path, line, column)`
([§FS-errors.4](FS-errors.md#4-determinism)). stderr never carries a site: it holds errors and their hints, warnings, and
notes such as those of [§FS-refs.2.1](FS-refs.md#21-an-id-with-no-citations).

### 2.1 An ID with no citations

An ID with no citations is an empty answer and exit `0`; `check` warns about an uncited
declaration ([§FS-check.4.1](FS-check.md#41-unused-declaration)). Beside it `refs` may print one `note:` line, on stderr only in
either format, so a caller reading stdout never sees it. A declared ID gets the pointer of
[§FS-refs.2.1.1](FS-refs.md#211-a-declared-id-nothing-cites-points-at-list---unused) where its conditions hold. An ID declared nowhere in the scanned tree
([§FS-scan.path.refs](FS-scan.md#pathrefs-refs-scans-the-paths-tree)) gets the typo note, even where it is declared outside the path; without a
path it reads, unqualified and alias-qualified:

```text
note: <ID> is neither declared nor cited — run `grund list` to see every declared ID
note: <alias>/<ID> is neither declared nor cited — run `grund list --project <alias>` to see <alias>'s declared IDs
```

With a path, the note names it exactly as supplied, `.` included, and says that the
declarations its command lists may lie outside the scan:

```text
note: <ID> is neither declared nor cited in scan path `<path>` — run `grund list` to see project declarations, including outside this scan
note: <alias>/<ID> is neither declared nor cited in scan path `<path>` — run `grund list --project <alias>` to see <alias>'s declared IDs, including outside this scan
```

The text after `note: ` of either note is also the `note` of the result the Rust API and both
bindings return ([§FS-distribution.3.2.2.2](FS-distribution.md#3222-result-fields)).

#### 2.1.1 A declared ID nothing cites points at `list --unused`

A whole-ID query of a declared ID that lists no site prints one `note:` line on stderr naming
`grund list --unused` ([§FS-list.1.3](FS-list.md#13---unused)), which answers the question for every declaration at once
([§GOAL-token-economy.1](../goals.md#1-what-this-requires)):

```text
$ grund refs FS-001-alpha
note: FS-001-alpha is declared but nothing cites it; `grund list --unused` lists every such declaration
```

The ID is the declaration's catalog spelling, whatever form the operand took, a number-only
shorthand included. The command is one whose listing holds that declaration
([§REQ-one-answer.contract](../requirements/REQ-one-answer.md#contract-one-question-one-tree-one-answer)): an alias-qualified query names its project, an unqualified one,
member-local included, the bare command, and an `E2E` case adds `--kind E2E`, with or without
the alias, because `--unused` leaves cases out unless that kind is selected ([§FS-list.1.3](FS-list.md#13---unused)).

```text
note: <alias>/<ID> is declared but nothing cites it; `grund list --project <alias> --unused` lists every such declaration
note: <alias>/E2E-<name> is declared but nothing cites it; `grund list --project <alias> --unused --kind E2E` lists every such declaration
```

The note is withheld where that listing answers another question: when the query selects a
section, inline or by `--section`; when a path is given, `.` included; when the scan is
incomplete, which exits `2` ([§FS-refs.4](FS-refs.md#4-exit-codes)); under `--glob` or `--from`; and when the answer has a
site. So an ID whose only site is its kind's index entry is listed with that site labelled
([§FS-refs.3.1](FS-refs.md#31---format-text-default)) and earns no note, though `--unused` lists it ([§FS-list.3.2.1](FS-list.md#321-refs-counts-citations)). Decided in
[§DF-refs-unused-note](../decisions/functional/DF-refs-unused-note.md#df-refs-unused-note-an-uncited-declarations-empty-refs-answer-points-at-grund-list---unused).

## globs: `--glob` lists who cites a set

`grund refs --glob <pattern>` lists every site whose cited coordinate a pattern matches, as
[§FS-coordinate-globs.matching.citations](FS-coordinate-globs.md#matchingcitations-on-refs-and-cover-a-glob-selects-citation-sites) matches it, from the set ID mode lists from
([§FS-refs.2](FS-refs.md#2-behaviour)), in its order and shapes ([§FS-refs.3](FS-refs.md#3-outputs)), each site once however many patterns match
it:

```text
$ grund refs --glob 'FS-login.requirements.{lockout,reset}'
src/login.py:5: §FS-login.requirements.lockout
src/login.py:11: §FS-login.requirements.reset
```

### globs.descendants: One ID asked both ways gives one answer

For an ID `X` the run resolves, written with its operators escaped and qualified as the run
needs it, `refs X` and `refs X --descendants` list exactly the sites of `--glob 'X{,.**}'`;
`refs X.S --descendants` those of `--glob 'X.S{,.**}'`; and `refs X.S` those of
`--glob 'X.S'`. A pattern's bare ID is not ID mode's bare ID: `--glob 'X'` selects only the
citations of `X` that carry no section, and `--glob 'X.**'` only those that carry one.

### globs.unresolved: A pattern resolves no target

A pattern names no one target, so nothing it matches is resolved or refused: a coordinate that
is undeclared, dangling, or claimed by two homes is listed as matched, and the mode has no
exit `1` ([§DF-citation-site-queries.2](../decisions/functional/DF-citation-site-queries.md#2-the-rulings-it-carries)). A refused pattern exits `2`
([§FS-coordinate-globs.errors.shape](FS-coordinate-globs.md#errorsshape-one-refusal-line)); a pattern no kind in scope can start gets its note on
stderr ([§FS-coordinate-globs.errors.note](FS-coordinate-globs.md#errorsnote-a-pattern-no-kind-can-start-gets-a-note)); [§FS-refs.2.1](FS-refs.md#21-an-id-with-no-citations)'s notes are never printed; and an
empty set prints nothing and exits `0`.

## from: `--from` lists what one point cites

`grund refs --from <coordinate>` lists the citation sites in the source coordinate's own body:
what it cites, where ID mode lists what cites it. The source resolves as
`grund <coordinate> <path>` resolves it ([§FS-show.2](FS-show.md#2-behavior)), in the same project and with the same
shorthand, stub and qualifier rules. It resolves in the whole project, but only the sites in
the path's tree are listed, so a source whose home ([§FS-refs.from.owned](FS-refs.md#fromowned-a-point-owns-its-lead-its-ancestors-and-itself)) lies outside the path
succeeds with nothing to list ([§FS-scan.path.refs](FS-scan.md#pathrefs-refs-scans-the-paths-tree)). Each site appears once, sorted by
`(path, line, column)`, in the shapes of [§FS-refs.3](FS-refs.md#3-outputs).

### from.owned: A point owns its lead, its ancestors and itself

A site is the source's when the scan gives it to the source's declaration in that
declaration's home under no section, under a proper ancestor of the source's section, or under
that section itself. That is the unit `cover` names in `enclosing_declaration` and
`enclosing_section`, by the ownership rules of [§FS-cover.6.2](FS-cover.md#62-ownership-rules) ([§FS-cover.3.2](FS-cover.md#32---format-json)). So
`--from FS-login.requirements.lockout` lists the sites of `FS-login`'s lead, of `requirements`
above its first child, and of `requirements.lockout`, never a sibling's or a descendant's, and
`--from FS-login` lists its lead's alone ([§DF-citation-site-queries.2](../decisions/functional/DF-citation-site-queries.md#2-the-rulings-it-carries)). Ancestry is by whole
path components, numbered, named and mixed alike ([§FS-config.3.3.4](FS-config.md#334-the-outer-section-separator)). The lead is every site
the scan gives the declaration under no section, a site under a rejected or duplicate heading
included, and a source declaration's lead is its comment or docstring block, never the code
below it. A stub's home is the declaration it pairs with, so `--from` on a stub reads the
inline home's sites; a home outside `[scan] include` gives the citations it lends, each once
it resolves, at their lines in the target ([§FS-declarations.stubs.target.5](FS-declarations.md#stubstarget5-what-a-target-outside-the-include-gives-and-what-stays-unjudged)).

### from.records: What a record and a total say

A record's `id` and `section` name what the site cites ([§FS-refs.3.2](FS-refs.md#32---format-json)), and a citation whose
alias names no loaded project is listed all the same. Under `--total` the sentence is the
source's, not a target's
([§DF-citation-site-queries.2](../decisions/functional/DF-citation-site-queries.md#2-the-rulings-it-carries)):

```text
$ grund refs --from FS-login.1.1 --total
cites at 5 sites across 1 file
```

It is inflected as [§FS-refs.3.4](FS-refs.md#34---total) inflects `cited at`, and an empty set reads `cites nothing`;
JSON is `{"sites":<n>,"files":<m>}`.

## 3. Outputs

### 3.1 `--format text` (default)

One line per site on **stdout**, in the located-finding shape ([§FS-errors.2.1](FS-errors.md#21-located-finding)), whose
`<message>` is the citation token exactly as written, marker or not, section included:

```text
$ grund refs FS-login
docs/functional-spec/README.md:5: §FS-login (index entry)
docs/guide.md:3: FS-login
```

A line whose exact site is an index entry that [§FS-check.4.1.2](FS-check.md#412-an-index-entry-does-not-count) excludes from use ends in the
`index entry` annotation shown above, after the complete `<path>:<line>: <token>`, in every
mode and even beside ordinary uses of the ID. Which sites carry it is that rule applied to the
exact site, never the file's name ([§FS-check.4.1.2.1](FS-check.md#4121-a-section-citation-or-a-value-binding-in-the-index-is-not-the-entry)). The annotation is text alone: JSON
records, counts and exits carry none.

### 3.2 `--format json`

NDJSON on stdout, one object per site, in the record shape of [§FS-output-shapes.5.1](FS-output-shapes.md#51-refs---formatjson):

```json
{"path":"docs/functional-spec/FS-001-alpha.md","line":11,"column":23,"id":"FS-002-beta","section":null,"marker":true,"text":"§FS-002-beta","enclosing_declaration":"FS-001-alpha","enclosing_section":"terms","kind_title":"What: behavior, requirements, and constraints"}
```

`section` is `null` for a citation with no section, and `text` is the token as written.
`enclosing_declaration` and `enclosing_section` follow `text` and name the unit the site sits
in, always present and each `string | null`, as [§FS-cover.3.2](FS-cover.md#32---format-json) specifies for both commands.
`kind_title` is the final field, the queried kind's effective title ([§FS-config.3.4.3](FS-config.md#343-title)): omitted
when the kind has none, kept when it is configured empty, and in a workspace the target
project's, even for a valid ID nothing declares, while the record's `project` names the citing
project ([§FS-workspace.8.2.4](FS-workspace.md#824---format-json), [§DF-configured-title-metadata.2](../decisions/functional/DF-configured-title-metadata.md#2-decision)). Under `--glob` and `--from` no
one target was queried, so a record carries no `kind_title`, and its `id` keeps the alias the
citation was written with, as `cover` renders it ([§FS-cover.3.2](FS-cover.md#32---format-json)).

### 3.3 `--summary`

`--summary` prints one line per citing **file** instead of one per site, sorted by path, over
the set [§FS-refs.3.1](FS-refs.md#31---format-text-default) lists ([§FS-refs.requirements.one-set](FS-refs.md#requirementsone-set-every-flag-answers-from-one-citation-set)):

```text
$ grund refs FS-001-alpha --section 1 --descendants --summary
docs/functional-spec/FS-002-beta.md: 3 (lines 3, 5)
docs/notes.md: 1 (line 1)
```

The line is `<path>: <count> (lines <l1>, <l2>, …)`, with `line` where there is one: the count
is the file's sites, and the lines are the sorted, de-duplicated lines that hold them. Under
`--descendants` a file is still one line, with no breakdown by section. A file holding
excluded index sites ([§FS-refs.3.1](FS-refs.md#31---format-text-default)) ends its line in an annotation that names them line by
line in sorted order, with the columns of a line an ordinary use shares; its count and lines
still include them, and the annotation says nothing of the file's other sites:

```text
<path>: <count> (lines <l1>, …) (index entries at line <l>; line <l>, column <c>, …)
```

With `--format json` each line is `{"path":<path>,"count":<n>,"lines":[<l1>,<l2>,…]}`, in the same
order, with no annotation and no `kind_title` ([§DF-configured-title-metadata.3](../decisions/functional/DF-configured-title-metadata.md#3-compatibility-and-alternatives)).

### 3.4 `--total`

`--total` prints the size of the set instead of its members: one line with the number of
citation **sites** and of the distinct **files** they live in.

```text
$ grund refs FS-001-alpha --section 1 --descendants --total
cited at 4 sites across 2 files
```

The sentence is [§FS-lsp.1.2.4](FS-lsp.md#124-the-title-hovers-text)'s clause word for word, `across` at every count and `site` and
`file` plural at every count but one, and an empty set reads `not cited`, because the terminal
and the editor state one fact ([§FS-lsp.4](FS-lsp.md#4-determinism-and-parity-with-the-cli)). `--glob` prints the same sentence, and `--from` its
own ([§FS-refs.from.records](FS-refs.md#fromrecords-what-a-record-and-a-total-say)). `--summary` beside `--total` is ignored rather than refused, and
the output and status are `--total`'s ([§FS-refs.requirements.one-set](FS-refs.md#requirementsone-set-every-flag-answers-from-one-citation-set)). With `--format json` it
prints one object at every count, `{"sites":<n>,"files":<m>}`, `{"sites":0,"files":0}`
included, with no `kind_title`, and no `project` in a workspace, whose paths are already
distinct ([§FS-workspace.8.2](FS-workspace.md#82-grund-refs)).

## 4. Exit codes

- `0` — the scan completed and the operand was not refused; the listed sites, possibly none,
  are the result.
- `1` — the operand names no one target, and stdout is empty: the selected project's grammar
  rejects it, as an ID off the effective `[id] format` or a number-only shorthand naming more
  than one declaration ([§FS-check.1.2](FS-check.md#12-the-number-only-shorthand), [§DF-refs-resolver-rejection](../decisions/functional/DF-refs-resolver-rejection.md#df-refs-resolver-rejection-an-id-rejected-by-a-selected-grammar-is-a-failed-query)); or the ID has more than
  one home, or the selected section, and only it, is claimed by more than one heading, which
  `refs` refuses as `show` does ([§FS-show.2.2.1](FS-show.md#221-ambiguous-id), [§FS-show.2.2.2](FS-show.md#222-ambiguous-section)).
- `2` — the scan was incomplete, so the lookup is not trustworthy as complete ([§FS-check.2.4](FS-check.md#24-an-incomplete-run)),
  or the run failed: an unsupported `--format`, an unknown alias, or any other setup or usage
  error ([§FS-cli.4](FS-cli.md#4-errors-with-no-source-location)).

Text prints `show`'s bare query-failure line ([§FS-errors.2.3](FS-errors.md#23-bare-query-failure)): `invalid ID` with the
configured-format `hint:`, or, where a loaded member declares the ID, with a qualified command
([§FS-workspace.8.2.2.1](FS-workspace.md#8221-a-member-only-kind-replaces-the-text-format-hint)); or an ambiguity naming its candidates or every claimant's path and
line in `show`'s order, with no hint. JSON prints one failed-query object on stderr instead,
with no hint ([§FS-errors.5](FS-errors.md#5-json-format)): `invalid-id`, or `ambiguous` for a shorthand, with `sites` `null`;
`ambiguous` for independent homes or `ambiguous-section` for a colliding section, with `sites`
naming every claimant ([§FS-errors.5.2](FS-errors.md#52-on-stderr--what-is-not-output)). `--descendants` inherits the refusal and never combines
the claimants' subtrees ([§FS-refs.requirements.one-set](FS-refs.md#requirementsone-set-every-flag-answers-from-one-citation-set)).

Under `--glob` the exits are those of [§FS-refs.globs.unresolved](FS-refs.md#globsunresolved-a-pattern-resolves-no-target). Under `--from` a source
that `grund <coordinate>` refuses is refused as it refuses it, exit `1` with stdout empty,
with `show`'s text or its one JSON failed-query object on stderr ([§FS-show.3](FS-show.md#3-outputs), [§FS-errors.5](FS-errors.md#5-json-format)).

### 4.1 A stub's homes are found in its target

`refs` finds the homes of a stub's ID in its target, as `show` does, from the scan's one
verdict on each stub, whether or not the target is in `[scan] include` ([§FS-show.2.3](FS-show.md#23-source-declarations-in-code-and-doc-comments),
[§FS-declarations.stubs.verdict](FS-declarations.md#stubsverdict-one-verdict-per-stub-reached-once-per-scan), [§REQ-one-answer.stubs.outside-include](../requirements/REQ-one-answer.md#stubsoutside-include-a-target-outside-the-include-answers-as-if-scanned)): a target that declares
the ID twice is an ambiguous ID at the target's sites, and a selected section is checked
against the target's own headings.

A stub that cannot be paired refuses nothing, because `refs` reads no body. Where its target is
missing, lacks the ID, or is a file the scan does not read ([§FS-declarations.stubs.target.3](FS-declarations.md#stubstarget3-a-target-the-scan-does-not-read-declares-nothing)),
`refs` lists the citations of the stub's ID and exits `0`, where `show` refuses with
`broken stub:` ([§FS-show.2.3.4](FS-show.md#234-broken-stub)). Beside a real home the broken stub does not count towards the
duplicate-home refusal: `refs` gives the stdout, `--summary` and `--total` the repaired tree
gives, exit `0`, while `show` refuses the same tree as an ambiguous ID ([§FS-show.2.2.1](FS-show.md#221-ambiguous-id)).

Under `--format text`, each broken stub set aside beside a real home earns one stderr line, in
`show`'s order of the stubs: `warning: ` and the stub's `broken stub:` clause, the repair
`check` gives for a missing target ([§FS-declarations.stubs.repair](FS-declarations.md#stubsrepair-a-missing-target-beside-another-declaration-explains-the-repair)) and otherwise the line
`show` refuses that stub alone with ([§FS-show.2.3.4](FS-show.md#234-broken-stub)):

```text
warning: broken stub: <stub path>:<line> links to missing <target>; <ID> is declared at <home path>:<line>; point the stub link there
warning: broken stub: FS-a (stub at docs/a.md:1 points at ../notes/.a.md, which contains no inline declaration of FS-a)
```

Under `--format json` stderr stays empty, and `check` reports the stub as a finding. Broken
stubs with no real home beside them earn no warning.

## 5. Why this exists

A `grep` for `§` finds marked text, not citations: it cannot tell a citation from the same text
in a fenced example or behind a `<§>` escape, and gives an agent no stable result to program
against. `refs` answers from the scan `check`
judges, so the two never disagree on what a citation is, and with `grund <ID>` it closes the
loop: the ID query reads what an ID promises, and `refs` lists what took the promise.
