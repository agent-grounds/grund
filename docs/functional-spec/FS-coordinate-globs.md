# FS-coordinate-globs: a coordinate glob selects catalog coordinates by pattern

A coordinate glob names many points at once — `FS-config.requirements.**`, every point beneath
a chapter; `FS-*.terms`, every top-level Terms chapter — so a reviewer can count a checklist,
read every body of it in one scan and find who cites it, without a `jq` pipe or a list of IDs
typed by hand. `list --glob` selects and `list --total` counts the rows ([§FS-list.1](FS-list.md#1-inputs),
[§FS-list.total](FS-list.md#total---total-a-count-rung)), `show --batch --glob` reads their bodies ([§FS-show.1.9](FS-show.md#19---batch---all-every-coordinate-in-scope)), `refs --glob` lists
the sites that cite them and `cover --glob` keeps only those in its per-file view
([§FS-refs.globs](FS-refs.md#globs---glob-lists-who-cites-a-set), [§FS-cover.globs](FS-cover.md#globs---glob-keeps-the-citations-a-pattern-matches)), and the Node and Python bindings take the same
patterns. Serves [§GOAL-token-economy.1](../goals.md#1-what-this-requires), whose ladder needs one set named at every rung, and
[§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible). The rulings behind it are [§DF-coordinate-globs](../decisions/functional/DF-coordinate-globs.md#df-coordinate-globs-coordinates-are-selected-by-a-glob-of-their-own-and-a-brace-before-a-separator-opens-a-pattern) and, for the citation
sites, [§DF-citation-site-queries](../decisions/functional/DF-citation-site-queries.md#df-citation-site-queries-a-coordinate-glob-selects-citation-sites-and---from-reads-what-a-point-cites).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, section, coordinate, catalog),
[§FS-terms.terms.2](FS-terms.md#terms2-citations) (citation, marker), and [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scope, workspace, member,
alias).

- **coordinate glob** — A pattern over coordinates, the catalog's on `list` and `show` and the
  cited ones on `refs` and `cover`: an optional project qualifier, then components joined by
  separators, each matched against one component of a coordinate.
- **cited coordinate** — The coordinate a citation site names: its record's `id` and `section`,
  declared or not, in the project it targets ([§FS-coordinate-globs.matching](FS-coordinate-globs.md#matching-matching)).
- **component (narrowed)** — [§FS-terms.terms.7](FS-terms.md#terms7-values-and-integrations). In this spec, within a coordinate or a pattern,
  the ID atom or one section path component; elsewhere the shared definition applies.
- **ID atom** — A pattern's first component. It is matched against the whole rendered ID, never
  against a part of it.
- **alternative** — One of the patterns a brace group stands for; a pattern without a group is
  its own one alternative.

## grammar: Grammar

A pattern is `[<qualifier>/]<body>`, the qualifier ending at the last unescaped `/`
([§FS-coordinate-globs.scope](FS-coordinate-globs.md#scope-scope)). The body's operators are the citation guard's, `*`, `?`, `[` and
`{` ([§FS-citations.recognized.globs](FS-citations.md#recognizedglobs-a-glob-operator-never-shortens-a-marked-citation)), except that `[^…]` opens a negated class here where the guard
reads a footnote reference, plus `**` and `\`, which makes the next character literal
everywhere, a class and a brace group included.

Inside one project, the first unescaped occurrence of its `section_separator`
([§FS-config.3.3](FS-config.md#33-section-paths--arbitrary-nesting-depth)) ends the ID atom; with none, the first unescaped `.` does, and a dot inside
the ID is escaped. Section components are always separated by `.`. Each alternative is split
on its own, and a `.` inside a class is a member of the class.

- `*` matches any run of characters inside one component, the empty run included, and `?`
  exactly one character. A class matches one character: `[…]` one of its members or ranges,
  `[!…]` and `[^…]` one that is none of them. A `]` written first in a class, after any `!` or
  `^`, is a member; `a-z` is the range of code points from `a` to `z`; a `-` written first or
  last is a member.
- A component written exactly `**` matches whole components: zero or more where it leads the
  pattern or stands between two components, one or more where it ends it. So `X.*` is the
  children of `X`, `X.**` everything strictly beneath `X`, `X{,.**}` the point and its
  subtree, and `**` alone every coordinate in scope. A run of two or more `*` that is not the
  whole of its component in every alternative is refused.
- `{a,b}` stands for each of its comma-separated alternatives, which may be empty and may
  cross `.`, as in `FS-config{.terms,.requirements.1}`. A group may hold one alternative;
  groups do not nest and take no `{N..M}` range, and outside a group `,` and `}` are ordinary
  characters.

It reads like the gitignore dialect `exclude` uses ([§FS-config-v2.schema.sources.exclude](FS-config-v2.md#schemasourcesexclude-exclude-is-a-gitignore-style-glob-list)) and
departs from it on purpose ([§DF-coordinate-globs.1](../decisions/functional/DF-coordinate-globs.md#1-context-and-decision)): `x**` is refused where gitignore reads two
ordinary stars, braces do not nest where the `ignore` crate nests them, `{,…}` keeps the empty
alternative `ignore` drops, a pattern is always anchored at the ID with no basename matching,
components are separated by `.` and `/` only ends a qualifier, and there is no `!` negation
line or `#` comment.

## matching: Matching

On `list` and `show` a glob selects catalog coordinates, a mixed section such as `1.goals`
([§FS-config.3.3.1](FS-config.md#331-named-components), [§REQ-one-answer.sections.map](../requirements/REQ-one-answer.md#sectionsmap-every-surface-reads-one-section-map)) and a front-matter coordinate
([§FS-fields.front-matter.surfaces](FS-fields.md#front-mattersurfaces-what-each-surface-does-with-them)) among them, and a healthy stub and its inline home are one
coordinate, at the home ([§FS-list.2.5](FS-list.md#25-inline-homes-stay-canonical)).

On `refs` and `cover` it selects citation sites instead, by their cited coordinate, from the
sites the command already lists: ID mode's set on `refs` ([§FS-refs.2](FS-refs.md#2-behaviour)) and every citation of a
scanned file on `cover` ([§FS-cover.2](FS-cover.md#2-behaviour)). A site matches whether or not its coordinate is declared
([§DF-citation-site-queries.2](../decisions/functional/DF-citation-site-queries.md#2-the-rulings-it-carries)): a dangling ID or section as written, and an owned
declaration-local numeric citation ([§FS-citations.recognized.local-sections](FS-citations.md#recognizedlocal-sections-declaration-local-numeric-section-candidates)) or a shorthand
citation ([§FS-citations.shorthand](FS-citations.md#shorthand-the-number-only-shorthand)) as the coordinate it resolves to.

Every component matches whole. Matching is case-sensitive and compares Unicode code points,
with no locale and no normalization, so a pattern selects the same rows on every machine
([§REQ-deterministic-output.1](../requirements/REQ-deterministic-output.md#1-what-determinism-requires)). The alternatives of a group and repeated `--glob` flags union:
whatever any of them matches is selected once. Matching nothing is not an error, and the run
exits `0` with its ordinary empty result. Groups are never expanded, so the cost of a match
grows with the pattern's length and the catalog's size, never with the product of its
alternatives ([§REQ-never-crashes.1](../requirements/REQ-never-crashes.md#1-every-failure-is-a-diagnostic)).

## scope: Scope

An unqualified pattern matches the current project only, `alias/` the one loaded project with
that exact alias, a nested `group/api/` included, and `*/` every loaded project; none loads a
project the run would not load ([§FS-workspace.8](FS-workspace.md#8-other-commands)). In a workspace that excludes its root there
is no current project, so an unqualified pattern matches nothing.

On `list` the path and every other filter intersect with the glob
([§FS-list.requirements.one-selection](FS-list.md#requirementsone-selection-every-filter-narrows-every-mode-alike--realized-directional-for-the-next-filter)), and `--unused` and `--kind` judge a section row by its
declaration ([§FS-list.3.4.5](FS-list.md#345-order-and---top)). `--project` never widens a pattern: an unqualified glob under
`--project api` lists nothing when the current project is not `api`.

On `refs` and `cover` a pattern is judged against the project a citation targets, the loaded
project its alias names or, with no alias, the project it sits in ([§FS-workspace.8.2](FS-workspace.md#82-grund-refs)), never
against the citing project. So `api/FS-login` selects every site that cites `api`'s
`FS-login`, wherever the site sits, while the site's record's `project` still names the citing
project ([§FS-workspace.8.2.4](FS-workspace.md#824---format-json), [§FS-workspace.8.6](FS-workspace.md#86-grund-cover)). A citation whose alias names no loaded project
targets none and matches no pattern, `*/` and `**` included, though `cover` without `--glob`
and `refs --from` list it.

## errors: Refusals, notes and hints

### errors.shape: One refusal line

A refused pattern prints one line on stderr, nothing on stdout, and exits `2`; where an
accepted input exists it follows the reason as [§FS-errors.2.5](FS-errors.md#25-refused-input) spells it:

```text
error: invalid coordinate glob "<pattern>" at column <N>: <reason>
```

`<N>` is the one-based byte column in the pattern as typed. What the grammar alone refuses is
refused before config discovery ([§FS-cli.requirements.before-load](FS-cli.md#requirementsbefore-load-a-usage-mistake-is-answered-before-the-tree-is-read--realized-directional-for-the-next-flag)). The component checks, of a
`**` that is not a whole component and of an empty component, need the separator: they are
made there too when the body holds no unescaped literal outside a class, so every project
splits it at its dots, and otherwise once the config is read. The marker and alias checks are
made once the config is read as well, before anything is printed, and a refusal under any
project in scope refuses the pattern. Of several patterns, the first on the command line
refused before discovery is reported, else the first refused once the config is read. Within
one pattern a refusal before discovery likewise wins, and within a stage the leftmost is
reported, except that a bracket is refused as a whole, for never closing or for a range, only
when nothing inside it is, and components are checked only once every bracket and escape
reads.

An accepted input is the pattern as typed with only the failed part rewritten, parsed again
through every check of the stage that refused it before it is offered, as
[§FS-rule-refusals.requirements.typed](FS-rule-refusals.md#requirementstyped-an-offer-is-built-from-what-was-typed-and-what-is-configured-and-nothing-else--realized) has it for selectors. An offer made before discovery, where no
separator is known yet, is parsed again as the `.`-separated reading with the default marker.
A run of three or more stars that is a whole component is offered as `**`, and any other run
as `*`, with the `{,.**}` alternative offered beside it when the run ends the pattern. An
empty component is rewritten only where dropping one `.` mends it, so the empty pattern, an
empty component at column 1, is offered nothing.

Each reason, with the column it is reported at:

| Pattern | Column | Reason and accepted input |
|---|---|---|
| `FS-**` | 4 | `"**" must be a whole component; accepted globs: "FS-*" or "FS-*{,.**}"` |
| `FS-**.terms` | 4 | `"**" must be a whole component; accepted glob: "FS-*.terms"` |
| `FS-convoy.***` | 11 | `"**" must be a whole component; accepted glob: "FS-convoy.**"` |
| `FS-convoy..requirements` | 11 | `empty component; accepted glob: "FS-convoy.requirements"` |
| `FS-convoy.` | 11 | `empty component; accepted glob: "FS-convoy"` |
| `FS-{a,{b,c}}` | 7 | `braces do not nest` |
| `FS-{a,b` | 4 | `"{" is never closed` |
| `FS-[a-z` | 4 | `"[" is never closed` |
| `FS-{1..3}` | 4 | `braces take no range; list each alternative` |
| `FS-[z-a]*` | 5 | `the range "z-a" runs backwards; accepted glob: "FS-[a-z]*"` |
| `FS-a\` | 5 | `"\" escapes nothing` |
| `group/*/FS-*` | 7 | `a project qualifier is one alias or "*"; accepted glob: "*/FS-*"` |
| `charlie/FS-*` | 1 | `unknown project alias "charlie"`, then the `known aliases:` line in a workspace |

A pattern that begins with the configured citation marker is refused at column 1, because a
glob names coordinates, not a citation:

```text
error: invalid coordinate glob "§FS-*" at column 1: coordinate globs do not take a citation marker; accepted glob: "FS-*"
```

### errors.note: A pattern no kind can start gets a note

A valid pattern that no configured citable kind in scope can start gets one line on stderr, on
every `--glob` surface, and changes neither stdout nor the exit ([§DF-coordinate-globs.2](../decisions/functional/DF-coordinate-globs.md#2-the-rulings-it-carries)):

```text
note: no configured kind in scope starts with "fs-"; did you mean "FS-*"?
```

The quoted prefix is the ID atom's text before its first operator, a `{` included. A kind can
start it when that prefix and the kind head ([§FS-config.3.2.9](FS-config.md#329-kind-head)) are each a prefix of the other,
and a kind whose format does not begin with `{kind}` can start every pattern. The
`; did you mean "…"?` clause ([§FS-errors.3](FS-errors.md#3-message-text)) appears only when exactly one kind matches with
ASCII case ignored, and offers the pattern with that kind's spelling.

### errors.hint: A glob typed as PATH gets a hint

`list`'s positional stays the scan path. When it names no path, holds `*`, `?`, `[` or `{`,
and is a valid pattern, the missing-path error gains one hint line:

```text
error: path does not exist: FS-*
hint: `list` reads `FS-*` as a scan path; for a coordinate pattern, use `grund list --glob 'FS-*'`
```

### errors.flags: Flag errors

`--glob` takes its value as the next argument or after `=`, on every surface that takes it.
Without one it is `error: --glob requires a value`, and on `show` without `--batch` it is
`error: --glob requires --batch`, both launch errors with exit `2` ([§FS-cli.4](FS-cli.md#4-errors-with-no-source-location)). What `--glob`
cannot be combined with on `refs` and `cover` is [§FS-refs.1.2](FS-refs.md#12-refusals) and [§FS-cover.6.4](FS-cover.md#64-errors).

## bindings: Node and Python

### bindings.options: The `globs` option

Node's `list`, `listSizes`, `showBatch`, `refs` and `cover` take `globs: string[]`, and
Python's `list_ids`, `list_sizes`, `show_batch`, `refs` and `cover` take `globs=()`, empty by
default, selecting as repeated `--glob` flags do ([§FS-distribution.3.2.1](FS-distribution.md#321-operations-options-and-refusals),
[§FS-distribution.3.3.5](FS-distribution.md#335-complete-initial-inventory)). There is no `total` option: a host counts `entries` or `hits`
([§DF-coordinate-globs.1](../decisions/functional/DF-coordinate-globs.md#1-context-and-decision)).

`refs` also takes the source of [§FS-refs.from](FS-refs.md#from---from-lists-what-one-point-cites), `from: string` in Node and `from_=None` in
Python ([§DF-citation-site-queries.2](../decisions/functional/DF-citation-site-queries.md#2-the-rulings-it-carries)), and its ID becomes optional. Explicit batch queries
beside `globs`, an ID beside `globs` or a source, `globs` beside a source, `section` or a true
`descendants` beside either, and Python's `cover` given `globs` beside `lines` are conflicting
options: `input` with code `conflicting-options` in Node, `ValueError` in Python. A `refs` with
no ID, no `globs` and no source is refused as a null ID is: `invalid-argument` in Node,
`TypeError` in Python. Empty `globs` select no mode, so `refs(id, globs=())` is ID mode.

### bindings.note: The `note` field

ListResult, SizeResult, BatchResult and CoverResult carry `note: string | null`
([§FS-distribution.3.2.2.2](FS-distribution.md#3222-result-fields)): the text of the notes of [§FS-coordinate-globs.errors.note](FS-coordinate-globs.md#errorsnote-a-pattern-no-kind-can-start-gets-a-note) after
`note: `, one per line, or `null` where the run printed none. RefsResult keeps the `note` it
has: ID mode's note of [§FS-refs.2.1](FS-refs.md#21-an-id-with-no-citations), the glob notes under `globs`, and `null` under a source.
Under `globs` or a source its `kindTitle` (`kind_title`) is `null` ([§FS-refs.3.2](FS-refs.md#32---format-json)).

### bindings.refusals: A refused glob is a query failure

A refused pattern rejects as a `query` failure with code `invalid-coordinate-glob`, whose
message is the CLI's line after `error: ` byte for byte and whose `column` is the same
one-based byte column, in Node and Python alike ([§FS-distribution.3.0.3](FS-distribution.md#303-complete-data-and-canonical-parity)).

## workspace: Separators and workspaces

Each project in scope splits the pattern with its own separator, so one `*/` pattern reads
correctly across members whose separators differ, and a character that is not a project's
separator is literal there, as `#` is in a `.`-separated project. Under
`section_separator = "#"`, `RFC.042.draft#3.1` and `RFC\.042\.draft.3.1` select the same
coordinate.
