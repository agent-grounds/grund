# FS-rules: grounded declarations state and enforce chapter rules

A repository may state chapter and citation constraints once as grounded rule
declarations, have `grund check` enforce them, and render the same sentences to
agents before they write. This closes the checked-guidance loop of
[§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work) while keeping the strict, deterministic, fixed-severity behavior of the
existing checker. The product decision is
[§DF-chapter-rules](../decisions/functional/DF-chapter-rules.md#df-chapter-rules-chapter-rules-are-grounded-controlled-english-declarations-over-producer-neutral-facts).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, citable, body, section, coordinate, catalog),
[§FS-terms.terms.2](FS-terms.md#terms2-citations) (marker, citation, shorthand, citation site), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, alias),
[§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding, severity, suggestion, verdict), and [§FS-terms.terms.6](FS-terms.md#terms6-rules-and-directions) (direction,
level, rule, grounded).

- **subject** — The authored phrase that opens a rule sentence and names the units the rule is
  about.
- **selector** — What a subject or target spelling matches: a kind, a named chapter, or one
  exact ID.
- **chapter** — A rule's unit inside a declaration: one named child heading of it, as against
  the declaration's whole body. The rule grammar's subject-unit word.
- **display name** — The label an author wrote after a chapter's coordinate: `Terms` in
  `## terms: Terms`. What a presence rule names, whatever file the heading is in
  ([§FS-rules.5.1.1](FS-rules.md#511-a-chapters-display-name-is-the-label-its-author-wrote)).
- **family** — One accepted rule grammar. A sentence outside every family is refused rather than
  reinterpreted.
- **unverifiable here** — A rule sentence every component of which is well-formed, whose
  object alias the scope the command ran in holds no vocabulary for at all. Not an
  invalid rule ([§FS-rules.4.1](FS-rules.md#41-a-rule-this-scope-cannot-verify)).
- **facts** — The `RuleFacts` an evaluator reads: the versioned, immutable record of the units,
  chapters and counts one scan produced.

## 1. Rule declarations and opt-in

One or more citable Markdown kinds may set the optional Boolean
`rules = true` on their `[[kinds]]` rows
([§FS-config.3.4.12](FS-config.md#3412-rules--rule-declaration-kinds)). Every
declaration of an enabled kind is one rule: its ID identifies the authority,
its title is the executable sentence, and its body is a non-empty rationale
scanned, cited, checked, and formatted as ordinary Markdown. A repository that
wants rule rationales to cite goals says so with the existing
`[citations.<rule-kind>]` surface; no kind name is hard-coded.

The rule title is a grammar island. The ID before the title delimiter remains a
normal declaration ID, but title tokens are never citations under either
`[reference] strict` setting. Marker insertion, shorthand expansion, and
cross-reference wrapping never rewrite the sentence, reusing the declaration-
heading exclusion of [§FS-fmt.2.3](FS-fmt.md#23-what-is-never-rewritten). The
rationale body remains ordinary Markdown in every respect.

Without any `rules = true` row, scanning, findings, exit status, CLI text and
JSON, LSP diagnostics, formatter output, `config show`, and managed-block bytes
are byte-for-byte those of the same repository before this feature. The config
schema remains version 1.

## 2. Subject selectors

Phase-1 rule subjects select local declaration or named-chapter units only.
They use each configured kind's effective ID grammar
([§FS-config.3.2](FS-config.md#32-id--id-grammar)) and the scanner's accepted
body-local section records. A declaration unit is its owned body. A chapter
unit is the named heading and its subtree through the line before the next
heading of the same or shallower depth. Rejected outside-body and unmarked
headings are not units.

The four accepted subject spellings are:

| Authored subject | Selector and match |
|---|---|
| `Each KIND` | every local declaration of that configured citable kind |
| `ID` | the one local declaration with that full ID |
| `The NAME chapter of each KIND` | that named chapter of every local declaration of the kind that has one |
| `ID.NAME[.NAME…]` | the one local named chapter with that exact coordinate |

A token exactly equal to a configured citable kind name is the quantified kind
selector; any longer token must parse as a full ID under that kind's effective
grammar. Named components require `[id] named_sections = true` and obey the
configured named-section grammar. An exact literal subject must resolve to one
unit; zero matches or ambiguity produces `invalid-rule` ([§FS-rules.7.1](FS-rules.md#71-invalid-rule)). A quantified
subject selects only the units that exist: a chapter subject selects the named
chapter of every local declaration of the kind that has one, so a declaration
without it contributes no unit rather than a failing one. Contributing no unit
is not being outside the rule. A quantified chapter subject selects only the
chapters that exist, and a declaration of the kind that has none is reported
rather than passed over, so a chapter-scoped citation rule reaches every
declaration of its kind. That finding is
[§FS-rules.checks.unreached-declaration](FS-rules.md#checksunreached-declaration-unreached-declaration).

The two spellings therefore part on the same absence, and they stay two
different kinds of finding. Deleting the `requirements` chapter makes
`FS-login.requirements must cite at least one REQ.` an `invalid-rule` at the
rule heading, because an exact literal that does not resolve is a defect in the
sentence and is reported where the sentence is written. It makes
`The requirements chapter of each FS must cite at least one REQ.` an
`unreached-declaration` at `FS-login`, because a quantified subject that selects
nothing for a declaration is a gap in coverage and is reported where the
declaration is. Neither finding displaces the other, and where both spellings
stand over the same absent chapter both fire. Kind-level existence is not
expressible in phase 1.

Subject-side aliases, including `*/`, component wildcards,
numbered chapter literals, `Each chapter of each KIND`, and file, directory, or
path subjects are refused ([§FS-rules.3.5](FS-rules.md#35-strict-refusals)). The future component wildcard, if admitted,
will consume exactly one accepted section component; phase 1 accepts no such
token.

Object kind targets use the existing target-entry grammar of
[§FS-config.3.9.3](FS-config.md#393-alias-matching) unchanged, including a
pinned `alias/KIND` and `*/KIND`. Alternatives joined with `or` are one target
set. Repeated or differently ordered kinds normalize to the same byte-sorted
set for meaning and deduplication.

## 3. The five sentence families

The normative language is the following closed controlled-English grammar.
Fixed words and kind names are case-sensitive, every rule ends with exactly one
terminal `.`, and one sentence contains exactly one semantic verb. `N` is a
positive base-10 integer. `one` takes singular `chapter`; every numeric `N`
other than one takes `chapters`. A lower bound and an exact count spell one as
the word `one` and refuse the numeral, so `at least 1` and `exactly 1` are not
sentences; an upper bound spells it as the numeral, so `at most 1` is the only
spelling of a ceiling of one. The only accepted families are the following.

### 3.1 Chapter presence

```text
<subject> must|should have at least one <NAME> chapter.
<subject> must|should have at least N <NAME> chapters.
<subject> must|should have at most N <NAME> chapter|chapters.
<subject> must|should have exactly one <NAME> chapter.
<subject> must|should have exactly N <NAME> chapters.
```

The subject may be a kind or exact declaration, not a chapter. The rule counts
the subject declaration's accepted direct chapters whose display name is
`NAME`. A count outside the stated interval produces `chapter-cardinality`.
This family is how a repository states the *count* of a chapter — `exactly
one`, `at most N` — which no citation rule checks. A chapter-scoped citation
rule reaches a declaration that has no such chapter on its own
([§FS-rules.2](FS-rules.md#2-subject-selectors)) and reports it as `unreached-declaration`; a presence rule beside
it reports the same absence as `chapter-cardinality`, naming its own rule and
suppressing nothing ([§FS-rules.checks.unreached-declaration](FS-rules.md#checksunreached-declaration-unreached-declaration)). The two families
differ in the code they raise and in the counts they can state, not in whether
the absence is seen.

### 3.2 Outbound citation count

```text
<subject> must|should cite at least one <KIND> [or <KIND> ...].
<subject> must|should cite at least N <KIND> [or <KIND> ...].
<subject> must|should cite at most N <KIND> [or <KIND> ...].
<subject> must|should cite exactly N <KIND> [or <KIND> ...].
```

Every resolved physical citation site inside the subject unit whose target is
in the object set counts once. An ordinary `must cite at least one` failure at
zero reuses `missing-citation`; every other count failure uses
`citation-cardinality`.

### 3.3 Per-target coverage

```text
<subject> must|should cite each <KIND> at least once.
<subject> must|should cite each <KIND> at least N times.
<subject> must|should cite each <KIND> at most N times.
<subject> must|should cite each <KIND> exactly once.
<subject> must|should cite each <KIND> exactly N times.
```

For every declaration matched by the object kind selector, the rule separately
counts physical citation sites from the subject unit to that declaration. It
emits one `citation-cardinality` row per target whose count is outside the
constraint. If the subject is itself in the target set it is included; there is
no implicit self-exception. A target selector matching nothing yields no rows.

### 3.4 Inbound citation count and prohibition

Inbound counts use the same canonical counts as ordinary outbound citations:

```text
<subject> must|should be cited by at least one <KIND> [or <KIND> ...].
<subject> must|should be cited by at least N <KIND> [or <KIND> ...].
<subject> must|should be cited by at most N <KIND> [or <KIND> ...].
<subject> must|should be cited by exactly N <KIND> [or <KIND> ...].
```

Each resolved physical citation from a declaration of a source kind to the
subject unit counts once. A violation produces `uncited-unit`.

Prohibition has one spelling and no count synonym:

```text
<subject> must not|should not cite any <KIND> [or <KIND> ...].
```

Every offending physical citation site produces the existing
`forbidden-citation` or `discouraged-citation` finding ([§FS-rules.7.5](FS-rules.md#75-prohibition-and-recommendation-reuse)).

### 3.5 Strict refusals

No paraphrase or unlisted production is accepted. A refusal identifies the
failed production and gives its canonical accepted rewrite; an ambiguity names
every candidate and chooses none. The documentation and parser tests carry at
least these exact rows:

| Refused sentence | Exact reason and accepted rewrite(s) |
|---|---|
| `Each FS may not cite any AR.` | `modality "may not" is not accepted; accepted form: Each FS must not cite any AR.` |
| `Each FS must cite no AR.` | `"cite no" is not accepted; accepted form: Each FS must not cite any AR.` |
| `Each FS must cite a GOAL.` | `quantifier "a" is ambiguous; accepted forms: "Each FS must cite at least one GOAL." or "Each FS must cite exactly one GOAL."` |
| `Each FS must cite at least 1 GOAL.` | `numeric "at least 1" is not canonical; accepted form: Each FS must cite at least one GOAL.` |
| `Each FS must cite exactly 1 GOAL.` | `numeric "exactly 1" is not canonical; accepted form: Each FS must cite exactly one GOAL.` |
| `Each FS must cite at least one GOAL and must not cite any AR.` | `conjunctions are not accepted; accepted forms: "Each FS must cite at least one GOAL." and "Each FS must not cite any AR."` |
| `Each FS must cite at least one GOAL` | `rule must end with "."; accepted form: Each FS must cite at least one GOAL.` |
| `each FS must cite at least one GOAL.` | `fixed word "Each" is case-sensitive; accepted form: Each FS must cite at least one GOAL.` |
| `Each file in vendor/ must cite at least one FS.` | `path subjects are not accepted in phase 1; accepted form: Each FS must cite at least one GOAL.` |
| `Each */FS must cite at least one GOAL.` | `subject namespaces must be local in phase 1; accepted form: Each FS must cite at least one GOAL.` |
| `FS-login.* must cite at least one REQ.` | `section-component wildcards are not accepted in phase 1; accepted form: FS-login.requirements must cite at least one REQ.` |
| `Each chapter of each FS must cite at least one REQ.` | `chapter-quantified subjects are not accepted in phase 1; accepted form: The requirements chapter of each FS must cite at least one REQ.` |
| `FS-login.2 must cite at least one REQ.` | `numbered chapter subjects can detach when headings move; accepted form: FS-login.requirements must cite at least one REQ.` |
| `FS-login.requirements must cite at least one REQ.` with named sections off | `named chapter subjects require [id] named_sections = true; accepted form after enabling it: FS-login.requirements must cite at least one REQ.` |
| `Each POLICY must cite at least one GOAL.` | `unknown kind "POLICY"; accepted form: Each FS must cite at least one GOAL.` |

`FS-missing must cite at least one GOAL.` is syntactically valid and therefore
is not a pre-scan refusal. After scanning it produces the exact resolution
message `literal subject FS-missing does not resolve` ([§FS-rules.4](FS-rules.md#4-validation-lifecycle)).

## 4. Validation lifecycle

`config validate` validates only the optional `rules` key and its relationship
to the kind row. Rule titles are Markdown catalog data and are parsed after the
shared scan has built the catalog, then resolved before evaluation or managed-
block rendering. A missing title, empty rationale, refused title, unresolved or
ambiguous literal, or disabled named component is a located `invalid-rule` at
the rule heading. One invalid rule does not prevent other scanned findings from
being reported, and no surface silently omits it.

`check --rule` is different only where a title declaration would have supplied
a location. Grammar or vocabulary refusal is decided after config selection
and before scanning, prints `error: <reason>; accepted form: <rewrite>` (or
`accepted forms:`), writes nothing to stdout, and exits 2. A syntactically valid
unresolved literal requires the catalog, so after scanning it yields an
`invalid-rule` attributed to `--rule` and the ordinary finding exit 1.

`init` parses and resolves configured rules before writing. Any `invalid-rule`
is printed in its located form, the existing managed block remains byte-for-
byte untouched, and the command exits nonzero. It never renders an invalid
sentence and never silently skips one. A sentence that is well-formed and only
unverifiable from where the command ran is not an invalid one: that is
[§FS-rules.4.1](FS-rules.md#41-a-rule-this-scope-cannot-verify)'s case, and only that case.

If any scan or fact producer is incomplete, every closed-world rule conclusion
about absence or count is suppressed. Already-known positive site findings may
remain, all ordinary scan findings are retained, and the run keeps exit 2:
no incomplete tree is presented as a complete rule verdict.

### 4.1 A rule this scope cannot verify

An object kind pinned at an alias (`workshop/OP`) or matched across every
alias (`*/OP`) resolves wherever the run holds the workspace that alias
belongs to, and that is the workspace the run's own effective config declares —
never one climbed to from above. `check` and `init` read one vocabulary there,
so no two commands in one directory disagree about one sentence.

A run whose effective config declares no `[workspace]` holds no alias at
all. A member-scoped run is the ordinary case: its effective config is the
member's own `grund.toml` ([§FS-workspace.2](FS-workspace.md#2-workspace-configuration), [§FS-workspace.5.1](FS-workspace.md#51-a-member-run)), so a pinned or
any-member object kind names something the scope cannot judge either way. Such
a sentence is **unverifiable here** rather than invalid. It is reported at the
rule's own heading, keeps the code `invalid-rule`, takes the wording
[§FS-errors.3.7](FS-errors.md#37-the-rule-site-unknown-alias-wording-migration) fixes, and the run still exits nonzero.

Unverifiability is a verdict about the object kind's alias and about nothing
else. Every other part of the sentence is judged against the facts the scope
already holds, exactly as it is when the object kind is local — the grammar, the
rationale, and a literal subject that has to resolve. A rule that fails for any
of those reasons is an invalid rule with [§FS-rules.4](FS-rules.md#4-validation-lifecycle)'s consequences in full,
whichever alias its object names, and the run reports the fact it can act on
rather than the one it cannot.

#### 4.1.1 The distinction is mechanical

What makes a sentence unverifiable is the absence of every workspace alias,
which the run knows without judging anything:

| written object kind | the run's workspace vocabulary | verdict |
|---|---|---|
| `workshop/OP` | holds no alias at all | unverifiable here |
| `workshop/OP` | holds `workshop`, which declares `OP` | resolves |
| `workshop/NOPE` | holds `workshop` | invalid rule |
| `typo/OP` | holds aliases, none named `typo` | invalid rule |
| `*/OP`, `OP` not local | holds no alias at all | unverifiable here |
| `*/OP`, `OP` not local | holds aliases, none declaring `OP` | invalid rule |
| `*/OP`, `OP` declared locally | either | resolves |

A config that declares `[workspace]` holds at least its own project's alias,
whatever its member list expands to — only a config that declares none holds
nothing at all. So an empty member list and one naming a member the run cannot
reach come to the same verdict about the same rule, and no run standing at a
workspace root is told that no workspace is in scope where it stands.

The exception is therefore never a relaxation of resolution. Where the scope
could judge the alias and the answer was no, the sentence stays invalid and
keeps every consequence the paragraph above gives it.

#### 4.1.2 What `init` writes anyway

`init` withholds the managed-block write for an invalid rule and for nothing
else. A run whose every unresolved rule is unverifiable here writes the block,
renders each unverifiable sentence as authored ([§FS-rules.9.1](FS-rules.md#91-one-tree-renders-one-block)), reports each one,
and exits nonzero: it is the write that is not withheld, not the failure that is
forgiven. One genuinely invalid rule beside an unverifiable one puts the whole
run back under [§FS-rules.4](FS-rules.md#4-validation-lifecycle) — nothing written, exit nonzero.

Withholding the write is what made [§REQ-agents-md.2](../requirements/REQ-agents-md.md#2-the-managed-block-stays-current) unsatisfiable from inside a
member holding such a rule. `check` there reports the managed block out of date
and names `grund init` as the remedy; a refusal to write leaves no exit from
that loop, and no edit inside the member opens one.

## 5. Relational meaning

The normative meaning is Datalog over one finite, complete snapshot. The first
implementation is a hand-written evaluator; no Datalog surface or runtime is
part of the release.

### 5.1 Facts and identity

`RuleFacts` has an immutable header containing `schema_version`, project
identity, producer identity, and completeness. Opaque keys are scoped by
project and producer. Side metadata maps each key to its stable authored label
and repository-relative anchor; metadata does not participate in logical
equality. The relations are:

```text
decl(node, kind)
chapter(node, section_path, display_name)
contains(parent_node, child_node)
cites(site, immediate_from_node, target_node)
site_in(site, unit_node)
```

Every declaration and accepted chapter has its node facts whether or not it
contains a citation, so citations never define the quantified universe. Every
physical citation has a distinct site key, preserving multiplicity. `cites`
uses the immediate enclosing declaration or chapter as `from`; `site_in`
relates the site to every rule unit that contains it. Unresolved or ambiguous
citations retain their ordinary findings and contribute no `cites` fact.

#### 5.1.1 A chapter's display name is the label its author wrote

A `chapter` fact's `display_name` is the label the author wrote after the
chapter's coordinate: `Terms` in `## terms: Terms`. It is the same label
whether the declaration's body lives in a Markdown file or in a source
doc-comment. A heading in a doc-comment sits behind the comment that carries
it — `///`, `//!`, a block comment's ` * `, a hash comment's `#`, or any other
configured prefix
([§FS-config.3.5.14](FS-config.md#3514-comment_prefixes-compose-with-extensions))
— and that envelope says where the body lives, not what the chapter is called,
so it is never part of the label. `/// ## terms: Terms` in a Rust file and
`## terms: Terms` in a Markdown file are therefore the same fact,
`chapter(c, terms, Terms)`, and a presence rule naming `Terms` counts either
one ([§FS-rules.3.1](FS-rules.md#31-chapter-presence)). The same label is the
title of that chapter's `list --selector` row
([§FS-rules.8](FS-rules.md#8-command-surfaces)).

The envelope has a closing half too. A block comment's closing `*/` on the
heading's own line, and the whitespace before it, closes the comment that
carries the heading, so it is no more part of the label than the ` * ` that
opens the line: ` * ## terms: Terms */` on the last line of a C or Java block
comment is `chapter(c, terms, Terms)`, the fact `/// ## terms: Terms` gives.
Only the C-family closer is envelope, because only a C-family block comment
carries a heading on a line that can close it. A Python docstring's heading is
read as Markdown, and a Markdown heading is never trimmed, so an author who
writes `*/` at the end of either keeps it in the label. The `title` a
declaration's `--toc --format json` section map gives that chapter is the same
label ([§FS-show.3.1.3](FS-show.md#313-json)).

### 5.2 Family clauses

For subject set `S`, target set `T`, physical site `P`, chapter name `N`, the
subject's kind `K`, and the interval predicate `within`, the five families
mean:

```text
chapter_count(s, N, n) :- s in S, n = count { c : chapter(c, _, N), contains(s, c) }.
outbound_count(s, T, n) :- s in S, n = count { p : cites(p, _, t), t in T, site_in(p, s) }.
per_target_count(s, t, n) :- s in S, t in T,
                             n = count { p : cites(p, _, t), site_in(p, s) }.
inbound_count(s, T, n) :- s in S,
                          n = count { p : cites(p, f, s), f in declarations(T) }.
forbidden_site(s, p) :- s in S, cites(p, _, t), t in T, site_in(p, s).
```

Positive families report where `not within(n, cardinality)`. Prohibitions
report every `forbidden_site`. Negation and aggregates are closed-world only
when the snapshot is complete ([§FS-rules.4](FS-rules.md#4-validation-lifecycle)).

A chapter subject `The N chapter of each K` carries a second premise, taken
over the declarations of the kind rather than over the chapters `S` selected:

```text
chapter_handle(c, N) :- chapter(c, q, _), last_component(q) = N.
chapter_of(d, N)     :- chapter_handle(c, N), decl(d, K), contains(d, c).
unreached(d, N)      :- decl(d, K), not chapter_of(d, N).
```

`chapter_handle` is the subject selector of [§FS-rules.2](FS-rules.md#2-subject-selectors) read as a relation. A
subject's `N` is an accepted section component, so it joins the last component
of the section path — `chapter`'s second position — and not the display name in
its third, which is what the presence family's `chapter_count` counts
([§FS-rules.3.1](FS-rules.md#31-chapter-presence)). The two clauses therefore
join different positions of the same relation, and this one follows the
selector rather than `chapter_count`: `chapter_of(d, N)` holds exactly when `d`
owns one of the chapters `S` selected, so contributing no unit and being
unreached are one set rather than two, and no declaration can both hand a
relation a unit and be reported as out of the rule's reach.

The outbound-count, per-target-coverage and inbound-count families report every
`unreached(d, N)` beside every `not within(n, cardinality)`
([§FS-rules.checks.unreached-declaration](FS-rules.md#checksunreached-declaration-unreached-declaration)). Two families do not, for two different
reasons. Chapter presence cannot reach the premise at all: its subject is a
kind or an exact declaration and never a chapter
([§FS-rules.3.1](FS-rules.md#31-chapter-presence)), so `S` is already the
declarations of the kind and nothing is absent from it. Prohibition admits a
chapter subject and still stays silent, because `forbidden_site` ranges over
`cites` facts and a citation site inside the chapter body is deleted along with
that body: for `must not cite` the forbidden site genuinely no longer exists.
That, rather than vacuity over an empty selection, is why the same argument
does not excuse the other three families.

## 6. Semantic deduplication

Before evaluation, constraints deduplicate by `(subject selector, modality,
relation, normalized target set, cardinality)`. Normalization preserves no
authored spelling beyond what appears in a finding.

Rules-only duplicates produce one finding per failing unit or site. The tail
contains every contributing rule ID in bytewise order, for example
`(RULE-a, RULE-b)`; an ad-hoc `--rule` origin participates in the same way.
The collapsed group's contributing origins are also the finding's `authority`
([§FS-rules.7.6](FS-rules.md#76-selection-json-ordering-and-exits)), in that
same bytewise order, so the tail and the field are one fact rendered twice and
cannot disagree. A jointly authored finding therefore names `--rule` in its
`authority` beside every declared rule that reached the same meaning, and its
tail is unchanged by that: there is no marker, no reordering, and no second
line.

Config-to-rule deduplication is intentionally narrower. It applies only where
an existing `[citations]` entry and a rule express the same local bare citing
kind, declaration-wide unit, modality/level, `cite` relation, normalized target
entry, and cardinality. If config participates, its existing
`missing-citation`, `suggested-citation`, `forbidden-citation`, or
`discouraged-citation` finding wins byte-for-byte: no rule ID is appended, its
`authority` stays empty, and JSON gains no `sites` list. `[citations]` remains
authored, rendered, and interpreted as before. A rule that loses to config this
way authored nothing, so a trial sentence duplicating a `[citations]` direction
is invisible to a report scoped by `authority`
([§FS-rules.8](FS-rules.md#8-command-surfaces)).

## 7. Findings and channels

All rule findings use the ordinary text and NDJSON schemas of
[§FS-errors](FS-errors.md#fs-errors-grund-emits-messages-in-fixed-shapes).
`must` and `must not` are errors. `should` and `should not` are suggestions:
they appear only with `--suggestions`, carry `"channel":"suggestion"` in JSON,
and never affect exit status. Structural recommendations retain their
structural code on that channel. No rule finding is a warning: the one that was
is an error from 0.16.0, and the channel went with it ([§FS-rules.7.7](FS-rules.md#77-one-required-level-finding-is-a-warning-until-0160)).

### 7.1 Invalid rule

`invalid-rule` is located at the rule heading. A configured parse failure is:

```text
<RULE-ID> is not a valid rule: <reason>; accepted form: <canonical template>
```

An ambiguous production uses `accepted forms:`. A resolution failure is:

```text
<RULE-ID> is not a valid rule: literal subject <selector> does not resolve
```

For an ad-hoc rule, `<RULE-ID>` is `--rule`.

### 7.2 Chapter cardinality

`chapter-cardinality` is located at the subject declaration title and includes
zero and surplus chapters:

```text
<subject> has <actual> <name> chapters; <RULE-ID> requires <count>
```

### 7.3 Outbound citation cardinality

An ordinary hard `at least one` rule with zero matches reuses
`missing-citation`:

```text
<subject> must cite <target-set> (<RULE-ID>)
```

Other ordinary counts and every per-target `cite each` miss use
`citation-cardinality`, located at the subject declaration or chapter title:

```text
<subject> cites <target-set> <actual> times; <RULE-ID> requires <count>
```

For `cite each`, `<target-set>` is the one canonical target ID and one row
is emitted per off-count target.

A chapter subject whose declaration has no such chapter reports
`unreached-declaration` instead, located at the declaration rather than at a
chapter title there is none of
([§FS-rules.checks.unreached-declaration](FS-rules.md#checksunreached-declaration-unreached-declaration)). It never reuses the
`cites <target-set> 0 times` line, which would name a unit that does not exist.

### 7.4 Inbound citation cardinality

`uncited-unit` is located at the subject declaration or chapter title:

```text
<subject> is cited by <source-set> <actual> times; <RULE-ID> requires <count>
```

The code covers zero, surplus, and other off-count inbound cardinalities. A
chapter subject whose declaration has no such chapter reports
`unreached-declaration` at the declaration instead, on the same reading as
[§FS-rules.7.3](FS-rules.md#73-outbound-citation-cardinality)'s.

### 7.5 Prohibition and recommendation reuse

A hard prohibition reuses the existing site-anchored `forbidden-citation`
wording of [§FS-check.3.12](FS-check.md#312-forbidden-citation), with
`(<RULE-ID>)` in place of `(citation direction)`. A negative recommendation
likewise reuses `discouraged-citation`, and an ordinary positive citation
recommendation reuses `suggested-citation`. Existing config-only bytes never
change.

### 7.6 Selection, JSON, ordering, and exits

`--only` and `--ignore` accept `invalid-rule`, `chapter-cardinality`,
`citation-cardinality`, `uncited-unit`, and `unreached-declaration` like every
other public code; their
value grammar takes codes only and never a rule identity
([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)).
Rule-derived JSON adds no `sites` list — a rule finding names one site, so the
multi-site field stays `null` on every one of them. It does carry the rule
authority as a field: every rule-derived message names its rule authority, and
the same origins are the record's `authority`, a bytewise-sorted list of rule
origins ([§FS-errors.5.1](FS-errors.md#51-on-stdout--the-commands-output),
[§FS-output-shapes.1](FS-output-shapes.md#1-finding-object)). A chapter subject
is rendered as its canonical qualified coordinate.

A rule-derived *diagnostic* carries the authority of the one rule that produced
it rather than a group's: an `invalid-rule`
([§FS-rules.7.1](FS-rules.md#71-invalid-rule)) names that rule, so a trial
sentence whose literal subject does not resolve is attributed to `--rule` and is
retained by a report scoped to it. A finding no rule authored — a citation
direction, a dangling or duplicate declaration, a run-level warning — carries an
empty `authority`.

The existing bytewise `(path, line, message)` ordering is the sole ordering
authority. A `cite each` message places its target ID immediately after the
fixed `<subject> cites ` prefix, before the actual count, so same-anchor rows
sort by target-ID bytes even when targets share a prefix. Findings affect exits
under the ordinary mapping: hard findings exit 1, suggestions never move the
exit, invocation/config failures exit 2, and incomplete scans stay 2.

### 7.7 One required-level finding is a warning until 0.16.0

The ramp this heading names is spent: it closed in grund 0.16.0, where
`unreached-declaration` became an error on the ordinary `must` channel and the
rules report stopped having a warnings channel. What the check does is
specified at the section its code names,
[§FS-rules.checks.unreached-declaration](FS-rules.md#checksunreached-declaration-unreached-declaration). This address, heading text included, is
kept only so citations written before the promotion still resolve.

## 8. Command surfaces

`grund check --rule "<sentence>" [<path>]` adds exactly one ad-hoc rule to all
configured rules. It never disables configured rules and deduplicates against
an identical one. Its validation and exit behavior are [§FS-rules.4](FS-rules.md#4-validation-lifecycle)'s.

`grund check --only-rule` narrows that run's report to what the trial sentence
authored: a finding is retained when its `authority`
([§FS-rules.7.6](FS-rules.md#76-selection-json-ordering-and-exits)) contains the
`--rule` origin. It is a boolean with no `=value` form, repeats harmlessly, and
is `check`-only ([§FS-cli.3](FS-cli.md#3-cross-subcommand-flags)). It requires
`--rule`: given alone it is an invocation error — `error: --only-rule requires
--rule`, empty stdout, exit `2`, decided where the other selector values are
decided, before config discovery or scanning
([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)) —
because a run that scoped to no sentence would print `success` and exit `0`,
which reads as a verdict rather than as the mistake it is. It composes with
`--only` and `--ignore` by intersection, and `--ignore` still wins
([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)).
Bare `--rule` is unchanged: a run that passes no selector prints the same bytes
and exits the same way it does today.

Three consequences of scoping follow from what `authority` is rather than from
the flag, and each is behavior a caller should be able to rely on.

A finding the trial sentence and a declared rule authored jointly — one finding,
one tail naming both origins
([§FS-rules.6](FS-rules.md#6-semantic-deduplication)) — **is** retained, with its
authority and its message bytes exactly as an unscoped run renders them. The
sentence did author it, and scoping is not the place to relitigate the merge.

A trial sentence duplicating a `[citations]` direction yields an **empty** scoped
report. The config finding wins byte-for-byte and the rule authored nothing
([§FS-rules.6](FS-rules.md#6-semantic-deduplication)), so there is nothing for
`authority` to name, and the run prints `success` and exits `0`
([§FS-check.2.1.3](FS-check.md#213-the-success-line)) while the tree's own
findings still stand. This is known behavior, recorded here so it cannot change
by accident; making such a sentence visible is a separate question this point
does not answer.

A `should`-level trial sentence produces findings only in the suggestions
channel, which the default run withholds
([§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in)), so `--only-rule`
alone prints `success` and `--suggestions` is needed to see it. Nothing is
special here — `--only` behaves the same — but it is the one combination whose
empty report reads like a sentence that found nothing.

`grund list --selector "<selector>" [<path>]` filters the shared catalog to
matched declaration and chapter units and composes by intersection with the
existing path, kind, project, unused, summary, size, top, and format selectors
where their output modes admit unit rows. Text prints the canonical coordinate,
two spaces, location, two spaces, and title. A chapter JSON row uses the list
object's existing fields in their existing order, adds `"section"` immediately
after `"id"`, and puts the declaration ID in `id` and exact component path in
`section`; a declaration row remains byte-for-byte the ordinary list row. In
both forms a chapter row's title is the chapter's display name
([§FS-rules.5.1.1](FS-rules.md#511-a-chapters-display-name-is-the-label-its-author-wrote)).
Invalid syntax, unknown vocabulary, disabled named sections, and ambiguous
exact literals are exit-2 invocation errors. A valid selector with no matches
prints nothing and exits 0. `--selector` is a flag; the positional remains the
scan path.

## 9. Managed guidance and editor parity

With at least one rule kind, `grund init` renders `### Chapter rules`
immediately after `### Citation directions`. It repeats the existing
`must`/`should` legend and emits one bullet per rendered rule in qualified
rule-ID order — every valid rule, and every rule unverifiable here
([§FS-rules.4.1](FS-rules.md#41-a-rule-this-scope-cannot-verify)) — each bullet the exact authored sentence followed by its live
rule citation. A rule-enabled block uses v11 on the v10 base. With no rule kind,
`init` retains the v10 block byte-for-byte. `grund check` re-renders and
byte-compares this config-derived section whether or not every rule resolved;
drift is `agents-init`, and the comparison is never skipped in silence
([§FS-check.3.5.4](FS-check.md#354-no-config-derived-section-is-exempt-from-the-comparison)).

Hard rule findings and `invalid-rule` travel through the same core report to
the LSP with the same message, code, title/citation range, and severity as CLI
JSON. The LSP adds no rule parser or evaluator. Suggestions remain CLI-only and
opt-in.

### 9.1 One tree renders one block

The rendered section is a function of the authored sentences and of how the
entrypoint renders their citations ([§FS-init.2.3.5.10](FS-init.md#23510-chapter-rules)), never of the scope a
run was given, so the bullet an unverifiable rule earns is the sentence exactly
as its heading spells it. A member-scoped `init` therefore writes the bytes a
run that held the whole workspace would write for that same file, and a `check`
from either scope compares against the same render ([§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes)).
Omitting the bullet where the alias could not be judged would make the two
scopes disagree about one file: the member would write a block its own workspace
root then reports as drifted.

#### 9.1.1 A path-scoped check compares against the same render

A path is a third scope the same sentence covers. `grund check <path>` compares
the managed block's `### Chapter rules` against the render `grund check .`
compares against, because the render is built from the rules the run's
resolution scope read ([§FS-check.1.3.6.1](FS-check.md#1361-a-path-scope-narrows-the-report-not-the-resolution)) — the project's ordinary scope with its
rule-kind homes, wherever the path points — and never from its report scope. A
path that holds no rule declaration therefore still renders every bullet, and
the `agents-init` finding the comparison raises reaches the narrowed report by
its code ([§FS-check.1.3.6.1](FS-check.md#1361-a-path-scope-narrows-the-report-not-the-resolution)).

So the two runs agree about `AGENTS.md` in both directions: neither calls the
block `init` wrote stale, and both call a block missing a bullet stale, with the
same finding. Rendering from the report scope would turn such a path into a
bulletless section, and the narrowed run and the whole-tree run would then ask
for opposite bytes in one file, which no `AGENTS.md` can satisfy.

## 10. Documentation and executable examples

The release includes one guide at `docs/user-facing/rules.md`, one runnable
golden example at `examples/rules/`, links from the root README and
`examples/README.md`, and no second skill. The guide teaches opt-in and rationale
bodies, every subject and family, counts and modalities, every finding and both
channels, ordering, both deduplication directions, every command flag,
validation lifecycle, every explicit phase-1 absence, what a quantified subject
does not select, and the declaration a chapter-scoped citation rule reports
rather than passes over.

The guide has a marked `### Chapter rules` writing section. Both repository and
binary-embedded copies of `skills/grund-init/SKILL.md` contain a marked byte-
identical copy of that section and remain wholly byte-identical to one another.
The section includes every accepted family, every [§FS-rules.3.5](FS-rules.md#35-strict-refusals) refusal with its exact
rewrite, the finding each example produces, and the exact
`unreached-declaration` error a chapter-scoped citation rule produces about a
declaration that has no such chapter, and the two actions
that answer it ([§FS-rules.2](FS-rules.md#2-subject-selectors),
[§FS-rules.checks.unreached-declaration](FS-rules.md#checksunreached-declaration-unreached-declaration)). Its leading sentence on
that reach is the specification's word for word, differing at most in where it
wraps, so neither can be reworded without the other. The section
also quotes the reason clause of the `invalid-rule` message an exact chapter
subject produces once its chapter is gone, and that quote is the binary's own
wording.

The runnable example contains at least one passing and one violated instance of
all five families. Its guide quotes every violated instance's exact finding and
its goldens cover `invalid-rule`, `chapter-cardinality`, rule-derived
`missing-citation`, ordinary and per-target `citation-cardinality`,
`uncited-unit`, `forbidden-citation` with its rule tail,
`suggested-citation`, `discouraged-citation`, and `unreached-declaration` for
both a single rule and a two-rule group; both channels and suggestion-
neutral exit behavior; two same-anchor off-count targets with shared-prefix
IDs; config-to-rule and rule-to-rule deduplication; and the refusal set.

Five independent pins prevent drift:

1. `examples/rules/expected.*` run through the shared e2e runner.
2. A marked-row extraction test submits every accepted/refused guide row to the
   released parser and asserts acceptance or exact refusal, rewrite, code, and
   channel.
3. Asset-sync tests compare guide section to repository skill and whole
   repository skill to the embedded copy.
4. The managed block's existing re-render byte comparison checks generated
   guidance.
5. A quoted-message test runs the exact chapter subject over a declaration
   whose chapter is gone and asserts the section's quoted reason clause is
   part of the `invalid-rule` message the binary emits.

## 11. Functional architecture constraint

Only two representations cross the parsing/evaluation boundary. The sentence
front end receives a title, rule identity/anchor, and config vocabulary and
returns `ParsedRule`: origin, anchor, subject selector, modality, relation,
normalized target set, and cardinality. It may not know `RuleFacts`, evaluate,
deduplicate, or synthesize findings.

Fact producers return only complete, immutable, versioned `RuleFacts` as [§FS-rules.5.1](FS-rules.md#51-facts-and-identity)
defines. The Markdown producer is phase 1's only producer, but the scanner is
rule-blind and contributes structural records rather than evaluating a rule.
The logic engine evaluates only `ParsedRule` over `RuleFacts`, deduplicates
semantic constraints, and emits located findings through the shared report
boundary. It may not know sentence text, Markdown, scanner records, or file
layout beyond fact anchors. Architecture ground and dependency/replacement
tests specify the component placement separately.

## 12. Deliberate phase-1 absences

There is no `[settings]`, `config show --at`, path/folder/file subject,
exception phrase, `grund:allow` comment, definition, derived term, component
wildcard, wildcard subject alias, new command verb, suppression mechanism,
SCIP/LSIF ingestion, symbol vocabulary, on-disk fact format, or Datalog
runtime. Settings must reuse this selector parser and independently answer
[§DF-fmt-suppression.2.2](../decisions/functional/DF-fmt-suppression.md#22-an-in-text-region-not-a-rule-keyed-by-declaration-section). A future adjacent-site exception may rely on rule prohibitions retaining their exact citation-site anchors. A future program producer inherits opaque identities, versioned immutable complete snapshots, repository-relative anchors, committed offline input, and evaluator independence, but no exchange format is chosen here.

## checks: Checks

A check this specification raises is a section named by its diagnostic code
([§REQ-spec-section-names.code](../requirements/REQ-spec-section-names.md#code-a-check-is-named-by-its-diagnostic-code)). The four codes the rule families reuse or raise —
`invalid-rule`, `chapter-cardinality`, `citation-cardinality` and
`uncited-unit` — keep the positional addresses they shipped at
([§FS-rules.7](FS-rules.md#7-findings-and-channels)) until the migration's own
`FS-rules` slice moves them in beside the one below.

### checks.unreached-declaration: Unreached declaration

A declaration of a kind that a chapter-scoped citation rule over that kind
cannot reach. The rule's subject is `The <NAME> chapter of each <KIND>`, the
declaration is a local declaration of `<KIND>`, and it has no accepted direct
chapter named `<NAME>`, so it contributes no unit to the selection
([§FS-rules.2](FS-rules.md#2-subject-selectors)) and the rule's relation says
nothing about it.

One finding per semantic rule group per unreached declaration. The group's
contributing origins are the finding's `authority` and its message tail exactly
as every other rule finding's are
([§FS-rules.6](FS-rules.md#6-semantic-deduplication)), so two byte-identical
rules yield one finding naming both, and two rules that mean different things
yield one finding each. Nothing suppresses either.

It fires for the three positive citation families — outbound count
([§FS-rules.3.2](FS-rules.md#32-outbound-citation-count)), per-target coverage
([§FS-rules.3.3](FS-rules.md#33-per-target-coverage)) and inbound count
([§FS-rules.3.4](FS-rules.md#34-inbound-citation-count-and-prohibition)) — and
for no other family, for the two reasons
[§FS-rules.5.2](FS-rules.md#52-family-clauses) gives. Like every other
closed-world conclusion it is withheld from an incomplete snapshot
([§FS-rules.4](FS-rules.md#4-validation-lifecycle)).

It is located at the subject declaration's title line, because the chapter
title that would otherwise anchor it is the thing that is missing. At the
required level the message is:

```text
<declaration> has no <name> chapter, so <authority> cannot reach it; add the chapter, or narrow the rule to the declarations that have one; this became an error in grund 0.16.0
```

carried as an error on the ordinary `must` channel ([§FS-rules.7](FS-rules.md#7-findings-and-channels)). At the
recommended level the same row is a suggestion with the landed clause dropped,
so the message ends at `have one` and is visible only under `--suggestions`.

The two actions the message names are the only two that answer it: add the
chapter, or narrow the rule's subject to the declarations that have one. A
chapter-presence rule standing beside the citation rule is not a third — it
raises its own `chapter-cardinality`
([§FS-rules.7.2](FS-rules.md#72-chapter-cardinality)) and suppresses nothing,
so one absent chapter under a paired presence and citation rule prints both
lines, each naming its own rule.

The code is selectable on the same surfaces as every other
([§FS-rules.7.6](FS-rules.md#76-selection-json-ordering-and-exits)), so
`--ignore unreached-declaration` is the opt-out for a repository that wants the
absence reported by nothing, and a `should` rule is the opt-out for one that
wants it reported without failing the run.
