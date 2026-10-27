# FS-refs: grund lists every citation of an ID

The `refs` subcommand answers the reverse of `grund <ID>`: not "what does this ID say?" but "who points at it?". An agent about to change a declaration — or delete one — needs to know what leans on it; `grund refs FS-check` is that lookup, scheme-aware in the ways a `grep` cannot be, with a compact `--summary` for a quick blast-radius read. Two more modes ask it of more than one ID: `--glob` lists every site that cites a coordinate a pattern matches ([§FS-refs.globs](FS-refs.md#globs---glob-lists-who-cites-a-set)), and `--from` turns the question round and lists what one point cites ([§FS-refs.from](FS-refs.md#from---from-lists-what-one-point-cites)). Serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible), [§GOAL-token-economy](../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file), and the agent-grounding loop in [§GRUND-grund](../grund.md#grund-grund-agents-stay-grounded-in-the-spec) (an agent verifying a change reads the cited bodies *and* the citations that reach them).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, body, section, coordinate, catalog),
[§FS-terms.terms.2](FS-terms.md#terms2-citations) (marker, citation, shorthand, citation site), [§FS-terms.terms.3](FS-terms.md#terms3-source-forms) (doc-comment),
[§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, workspace, alias), [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding), and [§FS-terms.terms.7](FS-terms.md#terms7-values-and-integrations)
(value, binding, snapshot).

## requirements: What every citation lookup holds to

Two requirements on `refs` as a whole, each carrying a mark as [§FS-config.requirements](FS-config.md#requirements-what-the-config-contract-holds-to) defines them. There is no third for what the tool states once for every query: listing exactly the citations `check` validates, and refusing a duplicate home as `show` does, is [§REQ-one-answer.contract](../requirements/REQ-one-answer.md#contract-one-question-one-tree-one-answer) and [§REQ-one-answer.identity.declarations](../requirements/REQ-one-answer.md#identitydeclarations-every-command-reads-one-declaration-set); rejecting an ambiguous operand with its candidates is [§REQ-no-wrong-citation.1](../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution); reading offline, read-only and to the same bytes on every run is [§REQ-runs-offline.1](../requirements/REQ-runs-offline.md#1-read-and-verification-paths-execute-nothing), [§REQ-no-data-loss.1](../requirements/REQ-no-data-loss.md#1-the-query-surface-never-writes) and [§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes); and `--summary` and `--total` as rungs of the escalation ladder are [§GOAL-token-economy.1](../goals.md#1-what-this-requires).

### requirements.one-set: Every flag answers from one citation set — realized, directional for the next flag

The invocation fixes one set — every recognised citation of the ID, narrowed by `--section` or the dotted form and widened by `--descendants` ([§FS-refs.1](FS-refs.md#1-inputs), [§FS-refs.2](FS-refs.md#2-behaviour)); every site whose cited coordinate a `--glob` matches ([§FS-refs.globs](FS-refs.md#globs---glob-lists-who-cites-a-set)); or every site the `--from` source owns ([§FS-refs.from](FS-refs.md#from---from-lists-what-one-point-cites)) — and each output flag folds over that set rather than over one of its own: `--summary` to a row per file, `--total` to one row ([§FS-refs.3.3](FS-refs.md#33---summary), [§FS-refs.3.4](FS-refs.md#34---total)). None adds a refusal or an exit status: each inherits the result of the operand that fixed the set ([§FS-refs.4](FS-refs.md#4-exit-codes)). So no combination needs a rule of its own, and `--section <s> --descendants --total` is a count a reader can hold against the editor's ([§FS-lsp.1.2.6](FS-lsp.md#126-what-a-section-heading-counts)). This binds a flag added from here on: it narrows or widens that set, or folds over it, and inherits the operand's result. Serves [§GOAL-token-economy.1](../goals.md#1-what-this-requires).

### requirements.unresolved: A citation is listed whether or not it resolves — realized

Whether a site reaches anything is `check`'s question, never a reason to drop it: an ID with no declaration lists its dangling citations, a cited section no heading declares counts under `--descendants` ([§FS-refs.2](FS-refs.md#2-behaviour)), a missing fetched snapshot lists its sites without running the integration ([§FS-refs.2.3](FS-refs.md#23-fetch-backed-snapshots)), and a stub that cannot be paired refuses nothing ([§FS-refs.4.1](FS-refs.md#41-a-stubs-homes-are-found-in-its-target)). What `refs` refuses is an operand it cannot name one target for — a grammar rejection, a duplicate home, a duplicate on the selected section — and deciding that needs no declaration to exist ([§FS-refs.4](FS-refs.md#4-exit-codes)). So `refs` says what would break before a target exists and after it is gone. Serves [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration).

## 1. Inputs

```
grund refs <ID> [<path>] [--section <s>] [--descendants] [--summary] [--total] [--format text|json]
grund refs --glob <pattern> [--glob …] [<path>] [--summary] [--total] [--format text|json]
grund refs --from <coordinate> [<path>] [--summary] [--total] [--format text|json]
```

- `<ID>` — the ID to look up, without the marker. It may carry an inline section in the configured `[id] section_separator` (`FS-check.3.1`, or `FS-plan.goals.performance` when named sections are enabled), or pass it as `--section 3.1` / `--section goals.performance`. Parsing uses the named kind's effective format and the selected project's shared catalog ([§FS-config.3.2](FS-config.md#32-id--id-grammar)): an exact off-grammar declaration is a valid read query and keeps its raw spelling, while an off-grammar argument with no exact declaration keeps the ordinary invalid-ID hint. The number-only shorthand is accepted where the effective format has one — `grund refs FS-042` lists the citations of `FS-042-user-login`, including those written in the shorthand ([§FS-check.1.2](FS-check.md#12-the-number-only-shorthand)) — and shorthand, duplicate, and section ambiguities are rejected with their candidates rather than resolved to a guess.
- `<path>` — what it scans is [§FS-scan.path.refs](FS-scan.md#pathrefs-refs-scans-the-paths-tree), in every mode.
- `--glob <pattern>` — list every site whose cited coordinate the pattern matches, instead of one ID's ([§FS-refs.globs](FS-refs.md#globs---glob-lists-who-cites-a-set)). Repeatable, and the patterns union ([§FS-coordinate-globs.matching.union](FS-coordinate-globs.md#matchingunion-repeated-globs-union)).
- `--from <coordinate>` — list the sites the coordinate's own body holds, instead of the ones that cite it ([§FS-refs.from](FS-refs.md#from---from-lists-what-one-point-cites)). Given once, spelled as for `grund <coordinate>`, and its value is the next argument or follows `=`.
- `--section <s>` — list only citations of exactly that numeric or named section path; without it, every citation of `<ID>` is listed, bare-ID ones included. Mutually exclusive with the dotted inline form. The filter reads the scanner's shared citation record, so it cannot disagree with `check`, `show`, completion, or the LSP about a named coordinate.
- `--descendants` — widen the section filter from the exact coordinate to that coordinate **and every section beneath it** ([§FS-refs.2](FS-refs.md#2-behaviour)), whichever way the section was spelled: it widens `--section` and the dotted inline form alike, and says nothing about which of the two may be written. It is the CLI spelling of the set a declaration-side section title already returns in an editor ([§FS-lsp.1.3.1](FS-lsp.md#131-citations-from-declarations)), so the two transports ask one question. A query that carries no section has nothing to widen, and there the flag changes nothing ([§FS-refs.4](FS-refs.md#4-exit-codes)).
- `--summary` — one line per citing **file** instead of one per site ([§FS-refs.3.3](FS-refs.md#33---summary)).
- `--total` — the citation set's size instead of its members: sites and files, one line ([§FS-refs.3.4](FS-refs.md#34---total)).
- `--format text|json` — output shape ([§FS-refs.3](FS-refs.md#3-outputs)). Default `text`.

`refs` is a query, like `show` — non-interactive, no prompts ([§FS-non-goals.10](FS-non-goals.md#10-interactive-mode)).

### 1.1 Three modes, one path

`--glob` and `--from` each select a mode of their own; without either, `refs` is in ID mode and reads every command line as it did before they existed, to the byte and the exit status. The positionals are collected before they are read. ID mode takes `<ID>`, then at most one `<path>`. Under `--glob` or `--from` every positional is the path, and with none the run covers the project ([§FS-scan.path.none](FS-scan.md#pathnone-a-run-with-no-path-covers-the-project-discovery-finds)), so `grund refs FS-login --glob 'FS-audit.**'` scans a path named `FS-login` and filters on no ID ([§FS-refs.1.3](FS-refs.md#13-an-id-typed-as-the-path-gets-a-hint)). `--section` and `--descendants` belong to ID mode; `--summary`, `--total` and `--format` fold and render every mode alike.

### 1.2 Refusals

Each of these is a usage error: one `error:` line on stderr, nothing on stdout, exit `2`, before config discovery, so a broken `grund.toml` hides none of them ([§FS-cli.4](FS-cli.md#4-errors-with-no-source-location), [§FS-cli.requirements.before-load](FS-cli.md#requirementsbefore-load-a-usage-mistake-is-answered-before-the-tree-is-read--realized-directional-for-the-next-flag)). A missing value is reported where the command line reads it, as every flag's is. Once the whole line is read, a bad `--format` is answered first ([§FS-cli.3.5](FS-cli.md#35-a-bad-run-flag-is-answered-before-the-load)), then the first row of this table that applies, and only then a pattern's own refusal ([§FS-coordinate-globs.errors.shape](FS-coordinate-globs.md#errorsshape-one-refusal-line)).

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

The accepted glob is [§FS-errors.2.5](FS-errors.md#25-refused-input)'s rewrite, offered only beside exactly one `--glob`: the pattern as typed with `.<s>` appended for `--section`, then `{,.**}` for `--descendants`, so both flags together offer `.<s>{,.**}` and the line names `--section`; `<s>` is the section with every operator in it escaped, and the glob is offered only when the result passes every check the grammar makes before discovery ([§FS-coordinate-globs.errors.shape](FS-coordinate-globs.md#errorsshape-one-refusal-line)). Otherwise, and beside `--from`, whose source a glob cannot replace, the line ends after the flag's name. `--glob` with no value is [§FS-coordinate-globs.errors.flags](FS-coordinate-globs.md#errorsflags-flag-errors). In ID mode every refusal keeps its bytes: `error: refs requires an ID` and `error: refs takes an ID and at most one path argument`.

### 1.3 An ID typed as the path gets a hint

Under `--glob` or `--from` a positional is the path. When it names no path, holds none of `*`, `?`, `[` or `{`, and is a valid pattern ([§FS-coordinate-globs.grammar](FS-coordinate-globs.md#grammar-grammar)), the missing-path error gains one hint line:

```text
$ grund refs FS-login --glob 'FS-audit.**'
error: path does not exist: FS-login
hint: with `--glob`, `refs` reads `FS-login` as a scan path; to add its citations, use `--glob 'FS-login{,.**}'`
$ grund refs FS-login --from FS-audit
error: path does not exist: FS-login
hint: with `--from`, `refs` reads `FS-login` as a scan path; to list its citations, use `grund refs FS-login`
```

No config is read for a path that does not exist, so whether the positional reads as an ID is the grammar's answer alone, with the `.` separator and the default marker, as [§FS-coordinate-globs.errors.hint](FS-coordinate-globs.md#errorshint-a-glob-typed-as-path-gets-a-hint) decides for `list`. A positional with an operator, or one that is no valid pattern, gets no hint; a path that exists is scanned; and ID mode's missing path keeps its bytes.

## 2. Behaviour

`refs` runs the same scan as `check` ([AR-scanner](../architecture/AR-scanner.md#ar-scanner-how-grund-discovers-declarations-and-citations)) and lists, for the requested `<ID>`, every recognised citation site — the set `check` would validate, so it honours `[reference] strict` (bare tokens are listed only in non-strict mode), the source-string exclusions ([§FS-check.1.1.3](FS-check.md#113-string-literals-in-source-files)), and citations inside doc-comments. In particular, a real Python module, function, or method docstring after assigned triple-quoted data remains in the set, while the assigned-data span itself is absent ([§FS-check.1.1.3.1](FS-check.md#1131-assigned-python-triple-quoted-data)). It does **not** list the *declaration* of `<ID>` — that is `grund <ID> --format=json` (the README documents that one-liner). An ID with no declaration still lists its citations, exactly the ones `check` flags as dangling, so `refs` is also the "what would break if I never create this ID" tool.

The set also holds each citation a stub's target outside `[scan] include` lends the declaration
the stub pairs with ([§FS-declarations.stubs.target.5](FS-declarations.md#stubstarget5-what-a-target-outside-the-include-gives-and-what-stays-unjudged)), once it resolves to `<ID>`: `refs` lists it
at its line in the target, `src/x.rs:3`, as the same tree with the target scanned lists it, though
`check` judges none of that file's own hygiene. A citation of that file the stubbed declaration
does not make is not in the set, and neither is one that resolves to nothing, so an ID it cites
that is declared nowhere is not listed from there. `--summary`, `--total`, `list`'s `refs` count
and the editor's count of a declaration's citing sites count the same set, and so does the
`unused` warning ([§FS-check.4.1](FS-check.md#41-unused-declaration)).

An owned declaration-local numeric citation ([§FS-check.1.1.8](FS-check.md#118-declaration-local-numeric-section-candidates)) is in that same set under its resolved full ID and exact section. Whole-ID and `--section` queries include it, while text and JSON retain the authored local token. Ownerless and unsupported local candidates have no guessed target and therefore appear in no target's result.

With `--descendants` ([§FS-refs.1](FS-refs.md#1-inputs)) the section filter keeps a citation whose section coordinate *is* the requested path or lies beneath it — the set [§FS-lsp.1.3.1](FS-lsp.md#131-citations-from-declarations) already returns on that section's heading, one relation read by both transports ([§FS-lsp.4](FS-lsp.md#4-determinism-and-parity-with-the-cli)). Three clauses fix what *beneath* means, and they are written here rather than left implicit in a comparison because a lookup that two rules could satisfy must say which one picks ([§REQ-no-wrong-citation.1](../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution)):

- **Descendant is a relation on dotted path components, never a string prefix** ([§FS-config.3.3.4](FS-config.md#334-the-outer-section-separator)). `3.16` reaches `3.16.1` and every depth below it and never the sibling `3.160`; `checks.duplicate` reaches `checks.duplicate.1` and never the sibling `checks.duplicate-section`. The components stay `.`-separated whatever the outer `section_separator` is, so the relation is the same one at every depth.
- **A dangling descendant counts.** A cited `<§>FS-refs.2.9` that no heading declares is in the set, because it is exactly the citation a move of `FS-refs.2` breaks. This is the existing rule of this section — `refs` lists the citations `check` flags — applied one level down.
- **An owned declaration-local numeric citation counts under its resolved section**, as it already does for a bare `--section`.

Nothing else widens with it: the scan, the sort, the rendering, the `note:` of [§FS-refs.2.1](FS-refs.md#21-an-id-with-no-citations) and the exit policy of [§FS-refs.4](FS-refs.md#4-exit-codes) are the ones a bare `--section` query gets.

Output is sorted by `(path, line, column)` ([§FS-errors.4](FS-errors.md#4-determinism)). The list is the command's *result*, so it goes to **stdout** — text lines and `--format json` NDJSON alike, as for `grund list` and `grund cover` ([§FS-errors.1](FS-errors.md#1-streams)). A text line has the `<path>:<line>: <message>` located-finding shape ([§FS-errors.2.1](FS-errors.md#21-located-finding)) so an editor can jump to it, but it is an *answer*, not a finding; stderr is left for errors and the notes of [§FS-refs.2.1](FS-refs.md#21-an-id-with-no-citations): the typo note, and the pointer to `list --unused` for a declared ID nothing cites ([§FS-refs.2.1.1](FS-refs.md#211-a-declared-id-nothing-cites-points-at-list---unused)).

### 2.1 An ID with no citations

An ID with zero citations produces empty output and exit `0` — or, under `--total`, the single line `not cited` and exit `0` ([§FS-refs.3.4](FS-refs.md#34---total)): an as-yet-uncited declaration is normal, and `check` already warns about it ([§FS-check.4.1](FS-check.md#41-unused-declaration)). Beside that empty answer `refs` may print one `note:` line to **stderr**, and still exits `0`. If the ID is *also* declared nowhere in the scanned tree, it is the typo note below. If it is declared, it is the pointer to the one call that lists every such declaration, under the conditions of [§FS-refs.2.1.1](FS-refs.md#211-a-declared-id-nothing-cites-points-at-list---unused). The scanned tree is the one [§FS-scan.path.refs](FS-scan.md#pathrefs-refs-scans-the-paths-tree) defines; the typo note does not test declaration existence outside it.

Without an explicit path, the typo hint remains `note: <ID> is neither declared nor cited — run \`grund list\` to see every declared ID`. For an alias-qualified workspace query it remains `note: <alias>/<ID> is neither declared nor cited — run \`grund list --project <alias>\` to see <alias>'s declared IDs`.

With an explicit directory or file path, the note names the path exactly as supplied, including `.` relative to the current directory, and distinguishes the scan from the project declarations the advice lists:

```text
note: <ID> is neither declared nor cited in scan path `<path>` — run `grund list` to see project declarations, including outside this scan
note: <alias>/<ID> is neither declared nor cited in scan path `<path>` — run `grund list --project <alias>` to see <alias>'s declared IDs, including outside this scan
```

Thus an uncited ID declared in `docs/` can earn this scoped note when the path is `src` or a file in it; without a path it earns the note of [§FS-refs.2.1.1](FS-refs.md#211-a-declared-id-nothing-cites-points-at-list---unused) instead. A genuinely absent ID still earns the hint, scoped when a path is supplied. Either note is a hint, not part of the result: it never joins stdout, in text or JSON, which stays empty, or carries only `--total`'s own answer, so machine consumers that read only stdout never see it. It mirrors the `ID not found` hint the ID query gives for a typo ([§FS-show.3](FS-show.md#3-outputs)) without that query's exit `1`. A resolver rejection is different: once the selected project's grammar rejects the operand there is no citation-list result, and from 0.16.0 that failed query exits `1` ([§FS-refs.4](FS-refs.md#4-exit-codes)).

#### 2.1.1 A declared ID nothing cites points at `list --unused`

A whole-ID query of a declared ID that lists no citation site prints one `note:` line to **stderr** naming the query that answers the same question for every declaration at once, `grund list --unused` ([§FS-list.1.3](FS-list.md#13---unused)), and changes nothing else: stdout is the empty answer, or `not cited` or `{"sites":0,"files":0}` under `--total`, in either format and with or without `--summary`, and the exit is `0`. One call then answers what would otherwise take one `refs` per declaration ([§GOAL-token-economy.1](../goals.md#1-what-this-requires)).

```text
$ grund refs FS-api-audit
note: FS-api-audit is declared but nothing cites it; `grund list --unused` lists every such declaration
```

The ID is the declaration's catalog spelling, whatever form the operand took: a number-only shorthand, or other digits for the same number, names the declaration it resolved to. The command is written so that its listing holds that declaration, so the advice and the listing give one answer ([§REQ-one-answer.contract](../requirements/REQ-one-answer.md#contract-one-question-one-tree-one-answer)):

- An alias-qualified workspace query names its project as the typo note does, in the first form below. An unqualified query, a member-local one included, keeps the bare command, whose listing holds the current project's declarations.
- An `E2E` case appends `--kind E2E`, because `--unused` leaves cases out unless that kind is selected ([§FS-list.1.3](FS-list.md#13---unused)), with or without the alias, as in the second form. Every other citable kind, a custom one included, is in the plain listing and takes no filter.

```text
note: <alias>/<ID> is declared but nothing cites it; `grund list --project <alias> --unused` lists every such declaration
note: <alias>/E2E-<name> is declared but nothing cites it; `grund list --project <alias> --unused --kind E2E` lists every such declaration
```

The note is given only where that listing answers the question the query asked. Otherwise it is withheld, and the query's output, its other notes and its exit are what they are without this point:

1. **The query selects a section**, inline or by `--section`, with or without `--descendants`: `--unused` judges whole declarations. On a query with no section `--descendants` changes nothing ([§FS-refs.4](FS-refs.md#4-exit-codes)), so `grund refs <ID> --descendants` gets the note exactly as `grund refs <ID>` does.
2. **A path is given**, a file, a directory or `.`: an empty answer about part of the tree says nothing about the whole project the listing reads. The scoped typo note above is unchanged.
3. **The scan is incomplete**: that run exits `2`, and its empty answer is not trustworthy as complete ([§FS-refs.4](FS-refs.md#4-exit-codes)).
4. **The mode is `--glob` or `--from`**: neither names one target ([§FS-refs.globs.unresolved](FS-refs.md#globsunresolved-a-pattern-resolves-no-target)).
5. **The answer is not empty.** An ID whose only citation site is its kind's index entry is listed with that site labelled ([§FS-refs.3.1.1](FS-refs.md#311-label-the-excluded-index-sites)) and earns no note, though `--unused` lists it, because the listing discounts the entry ([§FS-list.3.2.1](FS-list.md#321-refs-counts-citations)). That misses a hint and never gives a wrong one, so the citation query does not take on the index rule to close it.

The note is the text after `note: `, carried as the `note` of the result the Rust API and both bindings return, as the typo note is ([§FS-distribution.3.2.2.2](FS-distribution.md#3222-result-fields)); a caller that reads only the sites never sees it. Decided in [§DF-refs-unused-note](../decisions/functional/DF-refs-unused-note.md#df-refs-unused-note-an-uncited-declarations-empty-refs-answer-points-at-grund-list---unused).

### 2.2 Value bindings

A citation inside a recognized value binding, local or alias-qualified, is an ordinary citation in this set; value sources ([§FS-values.terms](FS-values.md#terms-terms)) themselves never contribute citations ([§FS-values.3.2](FS-values.md#32-recognized-text-contexts)).

### 2.3 Fetch-backed snapshots

The undeclared-ID rule of [§FS-refs.2](FS-refs.md#2-behaviour) includes a missing fetch-backed snapshot: `refs` reports its citation sites without executing the configured integration. Once fetched, the result set is unchanged; only the target now resolves.

## globs: `--glob` lists who cites a set

`grund refs --glob <pattern>` lists every site whose cited coordinate a pattern matches, as [§FS-coordinate-globs.matching.citations](FS-coordinate-globs.md#matchingcitations-on-refs-and-cover-a-glob-selects-citation-sites) matches it, from the set ID mode lists from ([§FS-refs.2](FS-refs.md#2-behaviour)), in the same `(path, line, column)` order and the same shapes ([§FS-refs.3](FS-refs.md#3-outputs)). Repeated, the patterns union, and each site is listed once:

```text
$ grund refs --glob 'FS-login.requirements.{lockout,reset}'
src/login.py:5: §FS-login.requirements.lockout
src/login.py:11: §FS-login.requirements.reset
```

### globs.descendants: One ID asked both ways gives one answer

For an ID `X` the run resolves, written with its operators escaped and qualified as the run needs it, `refs X` and `refs X --descendants` list exactly the sites of `--glob 'X{,.**}'`; `refs X.S --descendants` those of `--glob 'X.S{,.**}'`; and `refs X.S` those of `--glob 'X.S'`. A pattern's bare ID is not ID mode's bare ID: `--glob 'X'` selects only the citations of `X` that carry no section, and `--glob 'X.**'` only those that carry one.

### globs.unresolved: A pattern resolves no target

A pattern names no one target, so nothing it matches is resolved or refused: a coordinate that is undeclared, dangling, or claimed by two homes is listed as matched, and the mode has no exit `1` of its own ([§FS-refs.4](FS-refs.md#4-exit-codes)). A refused pattern exits `2` ([§FS-coordinate-globs.errors.shape](FS-coordinate-globs.md#errorsshape-one-refusal-line)), a pattern no kind in scope can start prints its note on stderr and changes nothing else ([§FS-coordinate-globs.errors.note](FS-coordinate-globs.md#errorsnote-a-pattern-no-kind-can-start-gets-a-note)), and [§FS-refs.2.1](FS-refs.md#21-an-id-with-no-citations)'s notes are ID mode's and are never printed here. An empty set prints nothing and exits `0`, and under `--total` reads `not cited` as ID mode's does ([§FS-refs.3.4](FS-refs.md#34---total)).

## from: `--from` lists what one point cites

`grund refs --from <coordinate>` lists the citation sites in the source coordinate's own body: what it cites, where ID mode lists what cites it. The source is resolved as `grund <coordinate> <path>` resolves it ([§FS-show.2](FS-show.md#2-behavior)), in the same project and with the same shorthand, stub and qualifier rules, and a source that query would refuse is refused with its bytes ([§FS-refs.4](FS-refs.md#4-exit-codes)). Each site appears once, sorted by `(path, line, column)`, in the shapes of [§FS-refs.3](FS-refs.md#3-outputs).

For example, under [§FS-scan.path.refs](FS-scan.md#pathrefs-refs-scans-the-paths-tree), if `FS-a` is declared in `docs/functional-spec/FS-a.md` and owns citations only in that file, `grund refs --from FS-a src` from the project root and `grund refs --from FS-a .` from `src/` both succeed with empty output; `--total` prints `cites nothing` ([§FS-refs.from.records](FS-refs.md#fromrecords-what-a-record-and-a-total-say)). The same holds for a valid section source and for `.` from an empty `docs/sub/`. If the source is a stub outside `src/` paired with an inline home inside `src/`, the home's owned sites inside the path are listed even when their cited targets are declared in `docs/`. A source absent from the project is still refused.

### from.owned: A point owns its lead, its ancestors and itself

A site is the source's when the scan gives it to the source's declaration in that declaration's home — its `enclosing_declaration` and `enclosing_section` ([§FS-cover.3.2](FS-cover.md#32---format-json)), by the ownership rules of [§FS-cover.6.2](FS-cover.md#62-ownership-rules) — under no section, under a proper ancestor of the source's section, or under that section itself. So `--from FS-login.requirements.lockout` lists the sites of `FS-login`'s lead, of `requirements` above its first child, and of `requirements.lockout`, and never those of a sibling or a descendant; `--from FS-login` lists its lead's alone. Ancestry is by whole path components, numbered, named and mixed alike ([§FS-config.3.3.4](FS-config.md#334-the-outer-section-separator)). The lead is every site the scan gives the declaration under no section, a site under a rejected or duplicate heading included, and a source declaration's lead is its comment or docstring block, never the code below it.

### from.stubs: A stub reads its paired declaration

A stub's source is the declaration it pairs with, so `--from` reads the sites of the inline home. Where that home lies outside `[scan] include`, its sites are the citations it lends, each once it resolves ([§FS-declarations.stubs.target.5](FS-declarations.md#stubstarget5-what-a-target-outside-the-include-gives-and-what-stays-unjudged)), at their lines in the target, as [§FS-refs.2](FS-refs.md#2-behaviour) lists them.

### from.records: What a record and a total say

A record's `id` and `section` name what the site cites, rendered as `cover` renders them, the alias it was written with kept ([§FS-cover.2.1](FS-cover.md#21-a-cross-project-citation-counts)); a citation whose alias names no loaded project is listed all the same. Under `--total` the sentence is the source's, not a target's:

```text
$ grund refs --from FS-login.requirements.lockout --total
cites at 2 sites across 1 file
```

inflected as [§FS-refs.3.4](FS-refs.md#34---total) inflects `cited at`, and `cites nothing` for an empty set. JSON keeps `{"sites":<n>,"files":<m>}`.

## 3. Outputs

### 3.1 `--format text` (default)

One line per citation site on **stdout**, in the located-finding shape ([§FS-errors.2.1](FS-errors.md#21-located-finding)):

```
$ grund refs FS-check.1
crates/grund-core/src/scanner/file_pass.rs:142: FS-check.1
docs/functional-spec/FS-show.md:11: §FS-check.1
```

`<message>` is the citation token exactly as it appears in the source — marker-prefixed or bare, with its section suffix — so the reader sees the form on disk. Exit `0` whenever the scan succeeds and the operand is not refused, regardless of how many citations were found; a refused operand exits `1` ([§FS-refs.4](FS-refs.md#4-exit-codes)).

#### 3.1.1 Label the excluded index sites

Append an `index entry` annotation to each text citation row whose exact site is excluded from use by [§FS-check.4.1.2](FS-check.md#412-an-index-entry-does-not-count). Preserve the complete existing `<path>:<line>: <token>` row as a contiguous prefix, with one row per site in the existing order. Label excluded sites even when the ID also has ordinary uses. A section citation, a recognized value binding (including a whole-root binding), or an ordinary citation beside an external canonical enrollment is labeled only if that exact site is excluded under that rule; the index filename alone cannot decide it.

This rendering applies wherever text `refs` lists these sites, including `--glob` and `--from`; it changes neither the selected citation set nor JSON records, public record fields, flags, defaults, counts or exit codes. Together with [§FS-check.4.1.2](FS-check.md#412-an-index-entry-does-not-count), it explains why an index-only ID can be both listed here and reported unused.

### 3.2 `--format json`

NDJSON on stdout — one object per citation, in the record shape of [§FS-output-shapes.5.1](FS-output-shapes.md#51-refs---formatjson), which carries the verbatim token and the optional target-kind metadata. These examples select a kind without an effective title:

```json
{"path":"crates/grund-core/src/scanner/file_pass.rs","line":142,"column":12,"id":"FS-check","section":"1","marker":false,"text":"FS-check.1","enclosing_declaration":null,"enclosing_section":null}
{"path":"docs/functional-spec/FS-show.md","line":11,"column":42,"id":"FS-check","section":"1","marker":true,"text":"§FS-check.1","enclosing_declaration":"FS-show","enclosing_section":"terms"}
```

`section` is `null` for a bare-ID citation with no section coordinate.
`enclosing_declaration` and `enclosing_section` follow `text` and name the unit
the citing site sits in, always present and each `string | null`, in the shape
[§FS-cover.3.2](FS-cover.md#32---format-json) specifies for both commands.
`kind_title` is the record's **final** field, after that pair, and appears when
the resolved queried kind has an effective title
([§FS-config.3.4.3](FS-config.md#343-title)); absent titles omit
it, while a configured empty string is retained. A workspace record's existing
`project` still names the citing project, whereas `kind_title` belongs to the
selected target project, even when the queried ID is undeclared but valid.
All existing fields retain their values and their order relative to one another.
Under `--glob` and `--from` a record names no one queried target: its `id` keeps
the alias the citation was written with, as `cover` renders it
([§FS-cover.3.2](FS-cover.md#32---format-json)), and it carries no `kind_title`. ID mode's records keep the
bytes above.

#### 3.2.1 Why `kind_title` stays last

The sentence this replaces said `kind_title` was appended *after* `text`, which
fixed where it sat when nothing followed it — a statement about one past append
rather than a promise to a caller. What a caller could have relied on is that
`kind_title` is the last key, and that still holds: the pair goes between `text`
and it, so every existing field keeps its value, its type and its position
relative to every other existing field
([§REQ-backwards-compatibility.4](../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise)).
Placing the pair after `kind_title` instead would move `kind_title` off the end
for no gain, and would break the field-order parity `cover` holds with this
record, which is stated where that parity lives and is not restated here
([§FS-cover.3.2](FS-cover.md#32---format-json)).

### 3.3 `--summary`

`grund refs <ID> --summary` emits one line per citing **file** instead of one per citation site, sorted by path:

```
$ grund refs FS-check --summary
crates/grund-core/src/scanner/file_pass.rs: 1 (line 142)
docs/functional-spec/FS-show.md: 3 (lines 11, 142, 200)
```

The shape is `<path>: <count> (lines <l1>, <l2>, …)`. The count is the number of sites from exactly the citation set [§FS-refs.3.1](FS-refs.md#31---format-text-default) lists, so `--summary` honours `[reference] strict`, the string-literal carve-out, and doc-comment citations the same way; the line list is the sorted, de-duplicated set of source lines holding them, so two citations on line 10 read `path: 2 (line 10)`. `grund refs <ID> --summary | wc -l` is then the number of files that lean on `<ID>`, while the line list still points an editor at every line with a site. With `--section`, the aggregate is over that section's citations only; with `--descendants` ([§FS-refs.1](FS-refs.md#1-inputs)) it is over the subtree, because `--summary` folds over whatever set [§FS-refs.3.1](FS-refs.md#31---format-text-default) lists rather than over a set of its own. A file carrying citations of three different sections beneath the requested one is therefore **one** line, with the count of all its sites and one de-duplicated line list — there is no grouping by section and no per-section breakdown. No citations prints nothing, with exit `0` and the [§FS-refs.2.1](FS-refs.md#21-an-id-with-no-citations) `note:` unaffected. With `--format json`: NDJSON, one object per file, `{"path":<path>,"count":<n>,"lines":[<unique l1>,<unique l2>,…]}`, in the same order; without `--summary` it is the per-citation form of [§FS-refs.3.2](FS-refs.md#32---format-json). Summary objects omit `kind_title`, including for a titled target kind. Exit codes ([§FS-refs.4](FS-refs.md#4-exit-codes)) are unchanged — `--summary` renders the same scan result, not a different query.

#### 3.3.1 Identify the excluded sites within a summary

A text summary row containing excluded index sites appends an `index entry` annotation identifying their source lines. Preserve the entire existing `<path>: <count> (lines <l1>, <l2>, …)` prefix (including singular `line`), its count and sorted line list, and keep one row per file. Identify excluded lines in sorted order; if an excluded site and an ordinary use share a line, distinguish the excluded site by column as well. A mixed file's annotation must not imply that its ordinary uses are excluded. Rows without excluded sites stay unchanged.

The exclusion is exactly [§FS-check.4.1.2](FS-check.md#412-an-index-entry-does-not-count)'s, as for individual rows ([§FS-refs.3.1.1](FS-refs.md#311-label-the-excluded-index-sites)). The annotation applies to summaries in every text `refs` mode. JSON summary objects retain their fields and values without display annotations; totals and citation membership are unchanged.

### 3.4 `--total`

`grund refs <ID> --total` answers the citation set with its size instead of its members: one line carrying the number of citation **sites** and the number of distinct **files** those sites live in, and no rows at all.

```
$ grund refs FS-refs --total
cited at 140 sites across 50 files
```

The clause is the one [§FS-lsp.1.2.4](FS-lsp.md#124-the-title-hovers-text) fixes for a declaration-side title hover, word for word and inflected the same way — `across` at every count, `site` and `file` taking an `s` at every count except one — because the two transports render one fact and a reader who has met it in an editor should not have to recognise a second phrasing in a terminal ([§FS-lsp.1.2.5](FS-lsp.md#125-what-a-whole-id-title-counts), [§FS-lsp.4](FS-lsp.md#4-determinism-and-parity-with-the-cli)). An empty set reads `not cited`, for the reason the hover reads it: `cited at 0 sites across 0 files` is a sentence pretending to be a count. `--glob` keeps this sentence, and `--from`, whose set is what a point cites, reads its own ([§FS-refs.from.records](FS-refs.md#fromrecords-what-a-record-and-a-total-say)).

`--total` is the third rung of the fold `--summary` is the second of — one row per site ([§FS-refs.3.1](FS-refs.md#31---format-text-default)), one row per citing file ([§FS-refs.3.3](FS-refs.md#33---summary)), one row — and it folds over whatever set [§FS-refs.3.1](FS-refs.md#31---format-text-default) lists for the invocation rather than over a set of its own. So it composes with `--section` and the dotted inline form and widens with `--descendants`, needing no rule of its own for either: `grund refs <ID> --section <s> --descendants --total` is the terminal spelling of the count a section heading's hover shows ([§FS-lsp.1.2.6](FS-lsp.md#126-what-a-section-heading-counts)). It is a scoped query over that set and not a presentation mode, which is what [§FS-cli.6](FS-cli.md#6-what-is-deliberately-absent) requires of anything added to this surface — it changes which question is answered, not how the answer is printed.

Each rung replaces the one below it, so `--summary` beside `--total` is subsumed and silently ignored: the same bytes and the same status as `--total` alone, not the usage error of [§FS-cli.4](FS-cli.md#4-errors-with-no-source-location), on the [§FS-refs.4](FS-refs.md#4-exit-codes) precedent that lets a caller pass `--descendants` without first learning whether its operand carries a section. `grund refs <ID> --summary | wc -l` is a different invocation and is unaffected ([§FS-refs.3.3](FS-refs.md#33---summary)).

With `--format json`: exactly one object on stdout, `{"sites":<n>,"files":<m>}`, emitted at every count including `{"sites":0,"files":0}` ([§FS-output-shapes.5.1](FS-output-shapes.md#51-refs---formatjson)). The text form follows the hover because a person reads it; a caller that asked for JSON asked for numbers and gets them. The object carries no `project` member even in a workspace — the fold is over the whole listed set, and the paths it counts are already workspace-root-relative and distinct ([§FS-workspace.8.2](FS-workspace.md#82-grund-refs)) — and no `kind_title`, for the reason a summary object omits one: the pair is not a record of the target ([§FS-refs.3.2](FS-refs.md#32---format-json)).

Exit codes are unchanged ([§FS-refs.4](FS-refs.md#4-exit-codes)). `--total` renders the same scan result, so a refused operand still leaves stdout empty under the staged contract, and an incomplete scan still exits `2`.

## 4. Exit codes

- `0` — scan succeeded; the listed citations (possibly none) are the result.
- `1` — the selected project's resolver rejected the ID
  operand: it does not match the effective `[id] format`, or it is a
  number-only shorthand naming more than one declaration
  ([§FS-check.1.2](FS-check.md#12-the-number-only-shorthand)). Both text and
  JSON leave stdout empty. Text uses the bare query-failure shape
  ([§FS-errors.2.3](FS-errors.md#23-bare-query-failure)); only invalid format
  keeps the configured-format `hint:`. JSON emits exactly one failed-query
  finding object on stderr, with `code` `invalid-id` or `ambiguous` and
  `sites:null`, and emits no hint ([§FS-errors.5](FS-errors.md#5-json-format)).
  It is also exit `1` when the ID has more than one home or its section path
  is claimed by more than one heading — a stub that cannot be paired is no home
  here ([§FS-refs.4.1.1](FS-refs.md#411-a-broken-stub-beside-a-real-home-is-set-aside-with-a-warning)): `refs` refuses both as `show` does
  ([§FS-show.2.2.1](FS-show.md#221-ambiguous-id),
  [§FS-show.2.2.2](FS-show.md#222-ambiguous-section)), and under JSON that
  object names its `sites` ([§FS-errors.5.2.1](FS-errors.md#521-sites-on-an-ambiguity-refusal)).
- `2` — scan / I/O error ([§FS-check.2](FS-check.md#2-outputs) partial-scan
  semantics apply: an incomplete scan exits `2` and the lookup is not
  trustworthy as complete), an unsupported `--format`, an unknown alias, or
  any other setup or CLI-level error ([§FS-cli.4](FS-cli.md#4-errors-with-no-source-location)).

Duplicate-home and requested-section refusals are classified the same way as
the two resolver rejections. They leave stdout empty before
listing citations or rendering `--summary` or `--total`; `--descendants` inherits
the same refusal rather than combining the claimants' children. Text uses show's
bare ambiguity message, and JSON emits exactly one failed-query finding on
stderr: `ambiguous` for independent ID homes, `ambiguous-section` for a colliding
section, with every claimant's path and line in show's deterministic order.

Only the selected section coordinate is checked for a section collision, whether
selected inline or by `--section`. A different duplicated section does not
invalidate a clean selected path, and a bare ID remains queryable when only a
section collides ([§FS-show.2.2.2.4](FS-show.md#2224-only-the-requested-path-can-collide)).
A valid stub paired with its inline declaration remains one home
([§FS-show.2.2.1](FS-show.md#221-ambiguous-id)). These ambiguity checks do not
require a declaration or selected section to exist: `refs` still lists citations
to absent targets ([§FS-refs.1](FS-refs.md#1-inputs)), and extracts no body to decide them. A
stub's homes are found in its target, scanned or not ([§FS-refs.4.1](FS-refs.md#41-a-stubs-homes-are-found-in-its-target)).

Grund 0.14.0 and 0.15.0 kept the former exit `2` and `error:` prefix for the two
resolver rejections and warned that they would become failed queries; that
compatibility form retired in grund 0.16.0. `--summary`, `--section`,
`--descendants` and `--total` do not introduce another classification: after
context and grammar selection they inherit the same operand result. This
boundary is the decision in [§DF-refs-resolver-rejection](../decisions/functional/DF-refs-resolver-rejection.md#df-refs-resolver-rejection-an-id-rejected-by-a-selected-grammar-is-a-failed-query).

The flag-selected modes add two rules of their own. Under `--glob` nothing is
resolved, so there is no exit `1`: a refused pattern exits `2` and an empty set
exits `0` ([§FS-refs.globs.unresolved](FS-refs.md#globsunresolved-a-pattern-resolves-no-target)). Under `--from` the source is the query
`grund <coordinate>` makes, refused as it refuses: stdout stays empty and the
run exits `1`, with show's text — `ID not found: <coordinate>` or `section not
found: <coordinate>` and its `hint:`, `invalid ID`, or an ambiguity with its
candidates — or, under JSON, show's one failed-query object on stderr, with code
`not-found`, `missing-section`, `invalid-id`, `ambiguous` or `ambiguous-section`
([§FS-show.3](FS-show.md#3-outputs), [§FS-errors.5](FS-errors.md#5-json-format)). `--summary` and `--total` inherit both.

`--descendants` adds no case of its own to the list above. On a query that
carries no section there is nothing to widen, so it is a silent no-op — stdout,
stderr and exit status are what the same invocation without the flag produces —
and not the usage error exit `2` of [§FS-cli.4](FS-cli.md#4-errors-with-no-source-location), so a caller holding an
operand it did not build may pass the flag without first learning whether that
operand carries a section.

### 4.1 A stub's homes are found in its target

`refs` finds the homes of a stub's ID as `show` does
([§FS-show.2.3.7](FS-show.md#237-a-stubs-target-is-found-by-its-id)), from the one verdict the scan reaches on each stub
([§FS-declarations.stubs.verdict](FS-declarations.md#stubsverdict-one-verdict-per-stub-reached-once-per-scan)): it reads the stub's
target for that ID, and a target outside `[scan] include` answers exactly as it
would were it scanned. A target that declares the ID twice is an ambiguous ID at
the target's sites ([§FS-show.2.2.1](FS-show.md#221-ambiguous-id)), refused for a bare `<ID>`
exactly as for `<ID>.<section>` or `--section`, and a selected section is checked
against the target's own headings ([§FS-show.2.2.2](FS-show.md#222-ambiguous-section)). So the
answer never depends on whether the target is in the scan.

A stub that cannot be paired refuses nothing. Where its target does not exist, or
exists and declares no such ID, `refs` lists the citations of the stub's ID and
exits `0`, as it does for any absent target ([§FS-refs.1](FS-refs.md#1-inputs)). A
target the scan does not read, by its name, extension, exclusion or enabled ignore rule, declares no such ID
however many headings of it the target holds
([§FS-declarations.stubs.target.3](FS-declarations.md#stubstarget3-a-target-the-scan-does-not-read-declares-nothing)),
so its stub refuses nothing either. The `broken stub:` refusals belong to `show`,
which needs the target's body ([§FS-show.2.3.4](FS-show.md#234-broken-stub));
`refs` reads no body.

#### 4.1.1 A broken stub beside a real home is set aside, with a warning

A stub that cannot be paired refuses nothing beside a real home either. Where the
ID is also declared somewhere the stub does not point — the inline declaration a
file move left behind its stale stub — the broken stub does not count towards the
"more than one home" refusal of [§FS-refs.4](FS-refs.md#4-exit-codes): `refs` lists every citation
of the ID and exits `0`, with the stdout, `--summary` and `--total` the repaired
tree gives. Every one of those sites cites the ID whichever home is real, and
`refs` reads no body to choose between them. What still refuses is what refused
before among the homes that remain: two real declarations, or a target that
declares the ID twice ([§FS-refs.4.1](FS-refs.md#41-a-stubs-homes-are-found-in-its-target)). `show` is not narrowed, because it
returns a body: it keeps refusing the same tree as an ambiguous ID with its
`broken stub:` clause ([§FS-show.2.2.1](FS-show.md#221-ambiguous-id)).

The stub it set aside is not hidden. Under `--format text`, for each broken stub
set aside beside a real home, `refs` writes one line to stderr, in show's
deterministic order of the stubs: `warning: ` followed by the stub's `broken stub:`
line. For a missing target that is the clause the ambiguity refusal carried,
`warning: broken stub: <stub path>:<line> links to missing <target>; <ID> is
declared at <home path>:<line>; point the stub link there` — the diagnosis `check`
makes of the same stub. For a target that exists but cannot pair — it lacks the
ID, or the scan never reads it — the refusal carried no such clause, and the line
is the one `show` refuses that stub alone with ([§FS-show.2.3.4](FS-show.md#234-broken-stub)). Under
`--format json` stderr stays empty, as stdout is the repaired tree's records:
`refs` has no warning object, and `check` is where the stub is reported as a
finding. A broken stub, or several, with no real home beside them earns no
warning: nothing was set aside, and the listing is that of any absent target
([§FS-refs.1](FS-refs.md#1-inputs)).

## 5. Why this exists

`grep -oE '§…'` gives a contributor a rough citation list but cannot tell a real citation from an ID-shaped substring in a string literal, respect `strict` mode, reach citations inside block doc-comments without language-specific regex, or produce a stable, machine-shaped result for an agent to program against. `refs` is the scheme's own answer, sharing the scanner with `check` so the two never disagree on what counts as a citation. `--summary` folds a wide citation set to one line per file, so the blast radius before changing a declaration is legible at a glance — token-cheap for an agent that needs the count and the file list, not every column. With `grund <ID>` it closes the loop: the ID query reads the body an ID promises, `refs` enumerates the code and docs that took the promise.
