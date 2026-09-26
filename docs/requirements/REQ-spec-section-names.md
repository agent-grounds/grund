# REQ-spec-section-names: a section name says what the thing is, and never moves

A coordinate is the address this repository hands out: a finding names it, an agent
reads it, and a thousand citing sites store it. So the name of a section is a published
surface of the specification and not a private convenience of whoever wrote the file.
Two properties make it worth reading: it says what the thing *is*, so a reader who holds
a diagnostic code or a concept can guess the address instead of paging through a file
([§GOAL-token-economy.1](../goals.md#1-what-this-requires)); and it does not move, so what is stored stays resolvable
([§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration)). The clauses below are what this repository's own declarations
are held to. They say nothing about what `grund` accepts from anyone else — the grammar
of a section path is [§FS-config.3.3](../functional-spec/FS-config.md#33-section-paths--arbitrary-nesting-depth), and this is a contract on how this project uses it.

## names: A name says what the thing is

A section name names its subject, never the section's place in the file and never how
severe its finding is. `errors`, `warnings`, `misc`, `other`, `notes` and `details` are
not names: the first two name a severity, the rest name nothing. Ordinals are not names
either. A chapter called *Errors detected* has to be re-read to learn what is in it, and
a check that is a warning today sits in it only until the day it is promoted.

## shape: Where a number is allowed

A new declaration has no numbered top-level section. The grammar admits a number under
a name but not a name under a number, so a numbered chapter makes its whole subtree
positional for good. Numbers therefore go beneath a named prefix, and only where the
content is genuinely ordered or is a sub-point of one named thing: a precedence ladder,
a sequence of steps, the parts of one check.

A term or a check is two components below the ID, and a third component only for a
sub-point of a check. Depth is not free: a note that cites a coordinate stays inside the
100-column cap of [§FS-inline-citation-style.4.1](../functional-spec/FS-inline-citation-style.md#41-errors--hard-caps), and each component spends columns that the
prose beside it then does not have.

Existing declarations keep the numbered chapters they already published. A number that
sites already store is an address, and [§REQ-spec-section-names.permanence](REQ-spec-section-names.md#permanence-a-name-is-permanent) applies to it
exactly as to a name.

## code: A check is named by its diagnostic code

Every check `grund check` enforces is a named section of the concept spec it constrains,
under that spec's `checks` chapter, and the section's name is the diagnostic code
verbatim — the same token `--only` and `--ignore` take, as [§FS-errors.5.5](../functional-spec/FS-errors.md#55-the-check-code-catalog) publishes it.
A reader who holds a finding then holds its address: the code `duplicate` is the section
`checks.duplicate` of the spec that governs declarations, written `<§>FS-declarations.checks.duplicate`.

Two consequences, and they are the point of naming a check for its code. A code cannot be documented in a
section whose prose never contains it, because the section is named for it. And severity
is not part of the address: a check's row in the catalog of [§FS-errors.5.5](../functional-spec/FS-errors.md#55-the-check-code-catalog) carries its
severity, so promoting a warning to an error edits a cell and moves no coordinate.

## reserved: Reserved and forbidden names

These names mean one thing wherever they appear, and a check may not take one of them:

| Name | What it means, everywhere |
|---|---|
| `terms` | the per-spec chapter [§FS-terms.terms](../functional-spec/FS-terms.md#terms-terms) obliges: the lean line naming the vocabulary groups the spec draws on |
| `checks` | the chapter holding the checks a concept spec's rules are enforced by, one section per diagnostic code |
| `why` | the closing chapter that says why the declaration exists |
| `examples` | worked examples of the thing the declaration specifies |
| `inputs` | what a command reads: its arguments, flags, config and scope |
| `behavior` | what a command does between its inputs and its output |
| `output` | what a command prints |
| `exit` | how a command exits |
| `principle` | the invariant the declaration turns on, as [§FS-config.principle](../functional-spec/FS-config.md#principle-a-setting-written-at-a-narrower-scope-wins) is |
| `requirements` | what the declaration holds itself to, citing the requirements it serves, as [§FS-config.requirements](../functional-spec/FS-config.md#requirements-what-the-config-contract-holds-to) is |

Two names are forbidden as chapter names outright. `rules` is the chapter-rule
vocabulary's word — [§FS-terms.terms.6](../functional-spec/FS-terms.md#terms6-rules-and-directions) gives *rule* to a chapter rule alone and gives
checker behavior the word *check* — so a chapter of checks is `checks`, and `rules` stays
free for what the rule language names. `catalog` belongs to the ID catalog
([§FS-terms.terms.1](../functional-spec/FS-terms.md#terms1-declarations-and-coordinates)), and the one catalog of diagnostic codes is [§FS-errors.5.5](../functional-spec/FS-errors.md#55-the-check-code-catalog); a second
place that calls itself a catalog is a second place a code is written down.

## permanence: A name is permanent

There is no rename map, so a rename is a break. A name that has shipped is kept, and a
section that has to move leaves its old coordinate behind as a one-line pointer to the
new one for as long as `grund refs` still finds a site on it. The pointer is retired
when nothing cites it, and retiring it is a change of its own — not a side effect of the
move that made it.

## why: Why this exists

The specification is read by address far more often than it is read whole. Every
property above is what makes an address cheap: guessable from what the reader already
holds, stable once stored, and unambiguous because one word means one thing. Held by
nothing but review, the three drift in the same direction — a chapter named for a
severity, a coordinate that encodes a position, a word used for two things — and each
drift is paid for by every citing site at once. This is a requirement rather than a
decision record because it is a contract on the repository's own surfaces, like
[§REQ-readme](REQ-readme.md#req-readme-the-readme-is-the-grounded-shop-window) and [§REQ-agents-md](REQ-agents-md.md#req-agents-md-the-agent-entrypoint-stays-managed-and-grounded), and because what enforces it is a test rather than an
argument.
