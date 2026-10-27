# FS-coordinate-globs: a coordinate glob selects catalog coordinates by pattern

A coordinate glob names many points at once — `FS-config.requirements.**`, every point beneath
a chapter; `FS-*.terms`, every top-level Terms chapter — so a reviewer can count a checklist,
read every body of it in one scan, find who cites it, and map changed lines back to its points
without a `jq` pipe or a list of IDs typed by hand. `grund list --glob` selects the rows ([§FS-list.1](FS-list.md#1-inputs)),
`grund list --total` counts them ([§FS-list.total](FS-list.md#total---total-a-count-rung)), `grund show --batch --glob` reads their
bodies ([§FS-show.1.9](FS-show.md#19---batch---all-every-coordinate-in-scope)), `grund refs --glob` lists the sites that cite them ([§FS-refs.globs](FS-refs.md#globs---glob-lists-who-cites-a-set)), `grund cover
--glob` keeps only those sites in its per-file view ([§FS-cover.globs](FS-cover.md#globs---glob-keeps-the-citations-a-pattern-matches)), and the Node and Python
bindings take the same patterns as `globs` ([§FS-coordinate-globs.bindings](FS-coordinate-globs.md#bindings-node-and-python)). The owner step of
that review is `grund cover FILE --lines N-M` ([§FS-cover.6](FS-cover.md#6-line-ownership)), and what one point cites is
`grund refs --from` ([§FS-refs.from](FS-refs.md#from---from-lists-what-one-point-cites)). Serves [§GOAL-token-economy.1](../goals.md#1-what-this-requires), whose ladder needs one set named
at every rung, and [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible). The rulings behind it are [§DF-coordinate-globs](../decisions/functional/DF-coordinate-globs.md#df-coordinate-globs-coordinates-are-selected-by-a-glob-of-their-own-and-a-brace-before-a-separator-opens-a-pattern) and,
for the citation sites, [§DF-citation-site-queries](../decisions/functional/DF-citation-site-queries.md#df-citation-site-queries-a-coordinate-glob-selects-citation-sites-and---from-reads-what-a-point-cites).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, section, coordinate, catalog),
[§FS-terms.terms.2](FS-terms.md#terms2-citations) (citation, marker), and [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scope, workspace, member,
alias).

- **coordinate glob** — A pattern over coordinates, the catalog's on `list` and `show` and the
  cited ones on `refs` and `cover`: an optional project qualifier, then components joined by
  separators, each matched against one component of a coordinate.
- **cited coordinate** — The coordinate a citation site names: its record's `id` and `section`,
  declared or not, in the project it targets ([§FS-coordinate-globs.matching.citations](FS-coordinate-globs.md#matchingcitations-on-refs-and-cover-a-glob-selects-citation-sites)).
- **component (narrowed)** — [§FS-terms.terms.7](FS-terms.md#terms7-values-and-integrations). In this spec, within a coordinate or a pattern,
  the ID atom or one section path component; elsewhere the shared definition applies.
- **ID atom** — A pattern's first component. It is matched against the whole rendered ID, never
  against a part of it.
- **alternative** — One of the patterns a brace group stands for; a pattern without a group is
  its own one alternative.

## grammar: Grammar

A pattern is `[<qualifier>/]<body>`. The body's operators are the citation guard's operator set
(`*`, `?`, `[`, `{`), shared with it rather than restated ([§FS-citations.recognized.globs](FS-citations.md#recognizedglobs-a-glob-operator-never-shortens-a-marked-citation)),
plus `\`, which escapes the character after it, and `**`. Its informative reference is the
gitignore dialect `exclude` already uses ([§FS-config-v2.schema.sources.exclude](FS-config-v2.md#schemasourcesexclude-exclude-is-a-gitignore-style-glob-list)); where the two
differ is [§FS-coordinate-globs.grammar.departures](FS-coordinate-globs.md#grammardepartures-where-it-departs-from-the-exclude-dialect).

### grammar.components: Where the ID atom ends

Inside one project, the first unescaped occurrence of its outer separator ends the ID atom.
With none, the first unescaped `.` ends it, and a dot inside the ID is escaped. Section
components are always separated by `.`. The outer separator is the project's
`section_separator` ([§FS-config.3.3](FS-config.md#33-section-paths--arbitrary-nesting-depth)), so the split is made once per project in scope
([§FS-coordinate-globs.workspace.separators](FS-coordinate-globs.md#workspaceseparators-each-project-splits-with-its-own-separator)). The rule is applied to each alternative, and a
`.` inside a class is a member of the class, never a separator.

The qualifier is the text before the pattern's last unescaped `/`. Exactly `*` means every
loaded project, and any other qualifier is one exact alias ([§FS-coordinate-globs.scope](FS-coordinate-globs.md#scope-scope)).

### grammar.operators: Wildcards, classes and escapes

`*` matches any run of characters inside one component, the empty run included, and `?`
matches exactly one character. A class matches one character: `[…]` one of its members,
`[!…]` and `[^…]` one character that is none of them. A `]` written first in a class, after
any `!` or `^`, is a member; `a-z` is the range of code points from `a` to `z`; a `-` written
first or last is a member. `\` makes the next character literal everywhere, a class and a brace
group included.

### grammar.globstar: `**` is a whole component

A component written exactly `**` matches whole components: zero or more where it leads the
pattern or stands between two components, one or more where it ends it. So `X.*` is the
children of `X`, `X.**` everything strictly beneath `X`, `X{,.**}` the point and its subtree,
and `**` alone every coordinate in scope. A run of two or more `*` that is not the whole of
its component in every alternative is refused ([§FS-coordinate-globs.errors.reasons](FS-coordinate-globs.md#errorsreasons-every-refusal-its-column-and-its-rewrite)).

### grammar.braces: Brace groups

`{a,b}` stands for each of its comma-separated alternatives, which may be empty and may cross
`.`: `FS-config{.terms,.requirements.1}` is two coordinates, and `FS-config{,.**}` keeps its
empty alternative. A group with one alternative is allowed. Groups do not nest, and a group
takes no `{N..M}` range. Outside a group, `,` and `}` are ordinary characters. The union of
the alternatives is the pattern's match, each coordinate once.

### grammar.departures: Where it departs from the exclude dialect

Each difference from `exclude` is deliberate:

- `x**` is refused, where gitignore reads it as two ordinary stars.
- Braces do not nest, where the `ignore` crate accepts nesting.
- `{,…}` keeps its empty alternative, where `ignore` drops it.
- A pattern is always anchored at the ID: there is no basename matching.
- There is no `!` negation line and no `#` comment.
- Components are separated by `.`, not `/`, and `/` only ends a qualifier.

The one difference from the citation guard is `[^`: in prose it opens a Markdown footnote
reference, and in a coordinate glob a negated class.

## matching: Matching

### matching.catalog: A glob selects catalog coordinates

On `list` and `show` a glob selects from the catalog `list` prints and `show` resolves: every
declaration and every recorded numbered, named or mixed section of it, `1.goals` included
([§FS-config.3.3.1](FS-config.md#331-named-components), [§REQ-one-answer.sections.map](../requirements/REQ-one-answer.md#sectionsmap-every-surface-reads-one-section-map)). A front-matter coordinate is one of those
sections ([§FS-fields.front-matter.surfaces](FS-fields.md#front-mattersurfaces-what-each-surface-does-with-them)). There it selects no citation, and never an ID that is
cited but not declared; on `refs` and `cover` it selects citation sites instead
([§FS-coordinate-globs.matching.citations](FS-coordinate-globs.md#matchingcitations-on-refs-and-cover-a-glob-selects-citation-sites)). A healthy stub and its inline home are one coordinate, at
the home ([§FS-list.2.5](FS-list.md#25-inline-homes-stay-canonical)).

### matching.citations: On refs and cover, a glob selects citation sites

On `refs` and `cover` a glob selects the citation sites whose cited coordinate it matches,
from the sites the command already lists: ID mode's set on `refs` ([§FS-refs.2](FS-refs.md#2-behaviour)), strict
markers, the string carve-out and stub-lent citations included, and on `cover` every citation
of a scanned file ([§FS-cover.2](FS-cover.md#2-behaviour)). The cited coordinate is the `id` and `section` the
site's record carries ([§FS-cover.3.2](FS-cover.md#32---format-json)), so a site matches whether or not the coordinate is
declared: a dangling ID or section matches as written, an owned declaration-local numeric
citation under the full ID and section it resolves to ([§FS-check.1.1.8](FS-check.md#118-declaration-local-numeric-section-candidates)), and a shorthand
citation under the ID it resolves to ([§FS-check.1.2](FS-check.md#12-the-number-only-shorthand)). A citation of a bare ID has no section
component, so `X` matches it and `X.**` does not. A site is selected once however many
patterns or alternatives match it, and two citations on one line are two sites, told apart by
their columns.

### matching.anchored: Anchored, case-sensitive, by code point

Every component matches whole. Matching is case-sensitive and compares Unicode code points,
with no locale and no normalization, so `?` and a class each match one code point and the
same pattern gives the same rows on every machine ([§REQ-deterministic-output.1](../requirements/REQ-deterministic-output.md#1-what-determinism-requires)).

### matching.union: Repeated globs union

Repeated `--glob` flags, like the alternatives of one group, union: a coordinate or a site any
of them matches is selected, once. A pattern that matches nothing is not an error.

### matching.bounded: Matching is bounded

A pattern is matched without expanding its groups into their product, so a pattern whose
alternatives multiply into millions answers in time proportional to its length and the
catalog's size rather than to that product ([§REQ-never-crashes.1](../requirements/REQ-never-crashes.md#1-every-failure-is-a-diagnostic)).

## scope: Scope

An unqualified pattern matches the current project only. `alias/` pins the one loaded project
with that exact alias, a nested `group/api/` included, and `*/` covers every loaded project.
Neither loads a project the run would not load ([§FS-workspace.8](FS-workspace.md#8-other-commands)). In a workspace that
excludes its root there is no current project, so an unqualified pattern matches nothing.

### scope.filters: Filters intersect and never widen

`list`'s path, `--kind`, `--project`, `--unused` and `--selector` intersect with the glob, as
they do with `--selector` ([§FS-list.requirements.one-selection](FS-list.md#requirementsone-selection-every-filter-narrows-every-mode-alike--realized-directional-for-the-next-filter)). `--project` never widens a pattern:
an unqualified glob under `--project api` lists nothing when the current project is not `api`.
`--unused` and `--kind` are judged on the owning declaration, so a section row stays exactly
when its declaration would ([§FS-list.3.4.5](FS-list.md#345-order-and---top)).

### scope.targets: On a citation, the qualifier names the project it targets

A citation's cited coordinate belongs to the project the citation targets: the loaded project
its alias names, or, with no alias, the project it sits in ([§FS-workspace.8.2](FS-workspace.md#82-grund-refs)). An unqualified
pattern, `alias/` and `*/` are judged against that target, never against the citing project,
so `api/FS-login` selects every site that cites `api`'s `FS-login`, wherever the site sits.
A citation whose alias names no loaded project targets none and matches no pattern, `*/` and
`**` included, but stays where nothing filters it: in `cover` without `--glob` and in `refs
--from`. The record's `project` still names the citing project
([§FS-workspace.8.6](FS-workspace.md#86-grund-cover)).

## errors: Refusals, notes and hints

### errors.shape: One refusal line

A refused pattern prints one line on stderr, nothing on stdout, and exits `2`
([§FS-errors.2.5](FS-errors.md#25-refused-input)):

```text
error: invalid coordinate glob "<pattern>" at column <N>: <reason>
error: invalid coordinate glob "<pattern>" at column <N>: <reason>; accepted glob: "<glob>"
error: invalid coordinate glob "<pattern>" at column <N>: <reason>; accepted globs: "<a>" or "<b>"
```

`<N>` is the one-based byte column in the pattern as typed. A refusal the grammar alone decides
is made before config discovery; one that needs the separator, the marker or the loaded
aliases is made once the config is read and before anything is printed. A refusal under any
project in scope refuses the pattern. Of several refused patterns, the first the grammar alone
refuses is reported, before discovery, so a broken `grund.toml` never hides it
([§FS-cli.requirements.before-load](FS-cli.md#requirementsbefore-load-a-usage-mistake-is-answered-before-the-tree-is-read--realized-directional-for-the-next-flag)); when the grammar refuses none, the first on the
command line refused once the config is read is reported. One pattern is read in the same
order: `FS-**.{a,b` is refused at column 7, for its unclosed brace, though its `**` at column 4
would be refused once the config is read. Within either stage the leftmost refusal is
reported, except that a bracket is refused as a whole, for never closing or for a range, only
when nothing inside it is, and components are checked only once every bracket and escape
reads. An accepted glob is the pattern as typed with only the failed part rewritten, parsed
again through every check of the stage that refused it before it is offered, as
[§FS-rule-refusals.requirements.typed](FS-rule-refusals.md#requirementstyped-an-offer-is-built-from-what-was-typed-and-what-is-configured-and-nothing-else--realized) has it for selectors. An offer made before discovery,
where no separator is known yet, is parsed again as the `.`-separated reading with the
default marker.

### errors.reasons: Every refusal, its column and its rewrite

| Pattern | Column | Reason and accepted input |
|---|---|---|
| `FS-**` | 4 | `"**" must be a whole component; accepted globs: "FS-*" or "FS-*{,.**}"` |
| `FS-**.terms` | 4 | `"**" must be a whole component; accepted glob: "FS-*.terms"` |
| `FS-config.***` | 11 | `"**" must be a whole component; accepted glob: "FS-config.**"` |
| `FS-config..requirements` | 11 | `empty component; accepted glob: "FS-config.requirements"` |
| `FS-config.` | 11 | `empty component; accepted glob: "FS-config"` |
| `FS-{a,{b,c}}` | 7 | `braces do not nest` |
| `FS-{a,b` | 4 | `"{" is never closed` |
| `FS-[a-z` | 4 | `"[" is never closed` |
| `FS-{1..3}` | 4 | `braces take no range; list each alternative` |
| `FS-[z-a]*` | 5 | `the range "z-a" runs backwards; accepted glob: "FS-[a-z]*"` |
| `FS-a\` | 5 | `"\" escapes nothing` |
| `group/*/FS-*` | 7 | `a project qualifier is one alias or "*"; accepted glob: "*/FS-*"` |
| `charlie/FS-*` | 1 | `unknown project alias "charlie"`, then the `known aliases:` line in a workspace |

A run of three or more stars that is a whole component is rewritten to `**`; any other run to
`*`, with the `{,.**}` alternative offered beside it when the run ends the pattern. An empty
component is rewritten only where dropping one `.` mends it. The empty pattern is an empty
component at column 1.

### errors.marker: A glob takes no citation marker

A pattern that begins with the configured citation marker is refused at column 1, because a
glob names coordinates, not a citation, and the accepted glob is the pattern without it. With
the default marker:

```text
error: invalid coordinate glob "§FS-*" at column 1: coordinate globs do not take a citation marker; accepted glob: "FS-*"
```

### errors.note: A pattern no kind can start gets a note

A valid pattern that no configured citable kind in scope can start exits `0` with its ordinary
empty result, and stderr gets one line for it:

```text
note: no configured kind in scope starts with "fs-"; did you mean "FS-*"?
note: no configured kind in scope starts with "POLICY-"
```

The quoted prefix is the ID atom's text before its first operator, a `{` included. A kind can
start it when that prefix and the kind followed by the literal its format writes after `{kind}`
are each a prefix of the other; a kind whose format does not begin with `{kind}` can start
every pattern. The `; did you mean "…"?` clause ([§FS-errors.3.2](FS-errors.md#32-the-unknown-project-recovery-shape)) appears only when exactly
one kind matches with ASCII case ignored, and offers the pattern with that kind's spelling.
The note is printed on every `--glob` surface, never changes stdout or the exit, and a pattern
whose prefix a kind can start gets none, however little it matches: `FS-{001,002}.requirements`
is silent where `FS` is configured.

### errors.hint: A glob typed as PATH gets a hint

`list`'s positional stays the scan path. When it names no path, holds `*`, `?`, `[` or `{`,
and is a valid pattern, the existing error gains one hint line:

```text
error: path does not exist: FS-*
hint: `list` reads `FS-*` as a scan path; for a coordinate pattern, use `grund list --glob 'FS-*'`
```

A missing path without an operator, or one that is not a valid pattern, gets no hint, and a
path that exists is read as today.

### errors.flags: Flag errors

`--glob` takes its value as the next argument or after `=`, on every surface that takes it.
Without one it is `error: --glob requires a value`, and on `show` without `--batch` it is
`error: --glob requires --batch`. Both are launch errors with exit `2`
([§FS-cli.4](FS-cli.md#4-errors-with-no-source-location)); what `--glob` cannot be combined with on `refs` and `cover` is [§FS-refs.1.2](FS-refs.md#12-refusals)
and [§FS-cover.6.4](FS-cover.md#64-errors).

## bindings: Node and Python

### bindings.options: The `globs` option

Node's `list`, `listSizes`, `showBatch`, `refs` and `cover` take `globs: string[]`, and
Python's `list_ids`, `list_sizes`, `show_batch`, `refs` and `cover` take `globs=()`, empty by
default, which selects as repeated `--glob` flags do ([§FS-distribution.3.2.1](FS-distribution.md#321-operations-options-and-refusals),
[§FS-distribution.3.3.5](FS-distribution.md#335-complete-initial-inventory)). A null batch query list with globs generates and filters as `--batch
--glob` does; explicit queries with globs are conflicting options, refused as `input` with code
`conflicting-options` in Node and with `ValueError` in Python. There is no `total` option: a
host counts `entries` or `hits`.

`refs` also takes the source of [§FS-refs.from](FS-refs.md#from---from-lists-what-one-point-cites): `from: string` in Node and `from_=None` in
Python, PEP 8's spelling of a name that is a keyword. Its ID operand becomes optional, `null`
in Node and `None` in Python, so `refs(null, {globs: ["FS-login.**"]})` and
`refs(from_="FS-login.3")` ask what `--glob` and `--from` ask. An ID beside `globs` or a
source, `globs` beside a source, and `section` or a true `descendants` beside either are
conflicting options, refused as above, as Python's `cover` refuses `globs` beside `lines`;
Python's `cover(text=True, globs=…)` filters the text view. A `refs` with no ID, no `globs` and
no source is refused as a null ID is today: `invalid-argument` in Node, `TypeError` in Python.
Empty `globs` select no mode, so `refs(id, globs=())` is ID mode.

### bindings.note: The `note` field

ListResult, SizeResult, BatchResult and CoverResult carry `note: string | null`
([§FS-distribution.3.2.2.2](FS-distribution.md#3222-result-fields)): the text of the notes of [§FS-coordinate-globs.errors.note](FS-coordinate-globs.md#errorsnote-a-pattern-no-kind-can-start-gets-a-note)
after `note: `, one per line, or `null` where the run printed none. RefsResult keeps the
`note` it has: ID mode's note of [§FS-refs.2.1](FS-refs.md#21-an-id-with-no-citations), the glob notes under `globs`, and `null` under a
source. Under `globs` or a source its `kindTitle` (`kind_title`) is `null` ([§FS-refs.3.2](FS-refs.md#32---format-json)).

### bindings.refusals: A refused glob is a query failure

A refused pattern rejects as a `query` failure with code `invalid-coordinate-glob`, whose
message is the CLI's line after `error: ` byte for byte and whose `column` is the same
one-based byte column. Node and Python return the canonical envelopes of the core, errors
included ([§FS-distribution.3.0.3](FS-distribution.md#303-complete-data-and-canonical-parity)).

## workspace: Separators and workspaces

### workspace.separators: Each project splits with its own separator

Each project in scope splits the pattern with its own outer separator, so one `*/` pattern
reads correctly across members whose separators differ. A character that is not a project's
separator is literal there: `#` is an ordinary character in a `.`-separated project.

### workspace.escapes: Two spellings of one coordinate

Under `section_separator = "#"`, `RFC.042.draft#3.1` and `RFC\.042\.draft.3.1` select the same
coordinate: the first ends the ID atom at `#`, the second, having no `#`, at its first
unescaped `.`.

### workspace.rows: Qualified rows

A row from another project is printed as that surface already prints it, its ID qualified by
the project's alias ([§FS-workspace.8.3](FS-workspace.md#83-grund-list)), and a generated batch query is qualified as
[§FS-show.2.6.2](FS-show.md#262-exhaustive-generation) says.
