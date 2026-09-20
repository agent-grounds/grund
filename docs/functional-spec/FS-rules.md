# FS-rules: grounded declarations state and enforce chapter rules

A repository may state chapter and citation constraints once as grounded rule
declarations, have `grund check` enforce them, and render the same sentences to
agents before they write. This closes the checked-guidance loop of
[§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work) while keeping the strict, deterministic, fixed-severity behavior of the
existing checker. The product decision is
[§DF-chapter-rules](../decisions/functional/DF-chapter-rules.md#df-chapter-rules-chapter-rules-are-grounded-controlled-english-declarations-over-producer-neutral-facts).

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
| `The NAME chapter of each KIND` | that named chapter of every local declaration of the kind |
| `ID.NAME[.NAME…]` | the one exact local named-chapter handle |

A token exactly equal to a configured citable kind name is the quantified kind
selector; any longer token must parse as a full ID under that kind's effective
grammar. Named components require `[id] named_sections = true` and obey the
configured named-section grammar. An exact literal subject must resolve to one
unit; zero matches or ambiguity produces `invalid-rule` (§7.1). A quantified
kind or chapter subject matching no declarations is valid, vacuously true, and
silent. Kind-level existence is not expressible in phase 1.

Subject-side namespaces, including aliases and `*/`, component wildcards,
numbered chapter literals, `Each chapter of each KIND`, and file, directory, or
path subjects are refused (§3.5). The future component wildcard, if admitted,
will consume exactly one accepted section component; phase 1 accepts no such
token.

Object kind targets use the existing target-entry grammar of
[§FS-config.3.9.3](FS-config.md#393-namespace-matching) unchanged, including a
pinned `alias/KIND` and `*/KIND`. Alternatives joined with `or` are one target
set. Repeated or differently ordered kinds normalize to the same byte-sorted
set for meaning and deduplication.

## 3. The five sentence families

The normative language is the following closed controlled-English grammar.
Fixed words and kind names are case-sensitive, every rule ends with exactly one
terminal `.`, and one sentence contains exactly one semantic verb. `N` is a
positive base-10 integer. `one` takes singular `chapter`; every numeric `N`
other than one takes `chapters`. The only accepted families are the following.

### 3.1 Chapter presence

```text
<subject> must|should have at least one <NAME> chapter.
<subject> must|should have at most N <NAME> chapter|chapters.
<subject> must|should have exactly one <NAME> chapter.
<subject> must|should have exactly N <NAME> chapters.
```

The subject may be a kind or exact declaration, not a chapter. The rule counts
the subject declaration's accepted direct chapters whose display name is
`NAME`. A count outside the stated interval produces `chapter-cardinality`.

### 3.2 Outbound citation count

```text
<subject> must|should cite at least one <KIND> [or <KIND> ...].
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
`forbidden-citation` or `discouraged-citation` finding (§7.5).

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
message `literal subject FS-missing does not resolve` (§4).

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
sentence and never silently skips one.

If any scan or fact producer is incomplete, every closed-world rule conclusion
about absence or count is suppressed. Already-known positive site findings may
remain, all ordinary scan diagnostics are retained, and the run keeps exit 2:
no incomplete tree is presented as a complete rule verdict.

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
citations retain their ordinary diagnostics and contribute no `cites` fact.

### 5.2 Family clauses

For subject set `S`, target set `T`, physical site `P`, chapter name `N`, and
the interval predicate `within`, the five families mean:

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
when the snapshot is complete (§4).

## 6. Semantic deduplication

Before evaluation, constraints deduplicate by `(subject selector, modality,
relation, normalized target set, cardinality)`. Normalization preserves no
authored spelling beyond what appears in a finding.

Rules-only duplicates produce one finding per failing unit or site. The tail
contains every contributing rule ID in bytewise order, for example
`(RULE-a, RULE-b)`; an ad-hoc `--rule` origin participates in the same way.

Config-to-rule deduplication is intentionally narrower. It applies only where
an existing `[citations]` entry and a rule express the same local bare citing
kind, declaration-wide unit, modality/level, `cite` relation, normalized target
entry, and cardinality. If config participates, its existing
`missing-citation`, `suggested-citation`, `forbidden-citation`, or
`discouraged-citation` diagnostic wins byte-for-byte: no rule ID is appended
and JSON gains no source-list field. `[citations]` remains authored, rendered,
and interpreted as before.

## 7. Findings and channels

All rule findings use the ordinary text and NDJSON schemas of
[§FS-errors](FS-errors.md#fs-errors-grund-emits-messages-in-fixed-shapes).
`must` and `must not` are errors. `should` and `should not` are suggestions:
they appear only with `--suggestions`, carry `"channel":"suggestion"` in JSON,
and never affect exit status. Structural recommendations retain their
structural code on that channel.

### 7.1 Invalid rule

`invalid-rule` anchors at the rule heading. A configured parse failure is:

```text
<RULE-ID> is not a valid rule: <reason>; accepted form: <canonical template>
```

An ambiguous production uses `accepted forms:`. A resolution failure is:

```text
<RULE-ID> is not a valid rule: literal subject <selector> does not resolve
```

For an ad-hoc rule, `<RULE-ID>` is `--rule`.

### 7.2 Chapter cardinality

`chapter-cardinality` anchors at the subject declaration title and includes
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
`citation-cardinality`, anchored at the subject declaration or chapter title:

```text
<subject> cites <target-set> <actual> times; <RULE-ID> requires <count>
```

For `cite each`, `<target-set>` is the one canonical target handle and one row
is emitted per off-count target.

### 7.4 Inbound citation cardinality

`uncited-unit` anchors at the subject declaration or chapter title:

```text
<subject> is cited by <source-set> <actual> times; <RULE-ID> requires <count>
```

The code covers zero, surplus, and other off-count inbound cardinalities.

### 7.5 Prohibition and recommendation reuse

A hard prohibition reuses the existing site-anchored `forbidden-citation`
wording of [§FS-check.3.12](FS-check.md#312-forbidden-citation), with
`(<RULE-ID>)` in place of `(citation direction)`. A negative recommendation
likewise reuses `discouraged-citation`, and an ordinary positive citation
recommendation reuses `suggested-citation`. Existing config-only bytes never
change.

### 7.6 Selection, JSON, ordering, and exits

`--only` and `--ignore` accept `invalid-rule`, `chapter-cardinality`,
`citation-cardinality`, and `uncited-unit` like every other public code.
Rule-derived JSON adds no source-list field. Every rule-derived message names
its rule authority; a chapter subject is rendered as its canonical qualified
handle.

The existing bytewise `(path, line, message)` ordering is the sole ordering
authority. A `cite each` message places its target ID immediately after the
fixed `<subject> cites ` prefix, before the actual count, so same-anchor rows
sort by target-ID bytes even when targets share a prefix. Findings affect exits
under the ordinary mapping: hard findings exit 1, suggestions never move the
exit, invocation/config failures exit 2, and incomplete scans stay 2.

## 8. Command surfaces

`grund check --rule "<sentence>" [<path>]` adds exactly one ad-hoc rule to all
configured rules. It never disables configured rules and deduplicates against
an identical one. Its validation and exit behavior are §4's.

`grund list --selector "<selector>" [<path>]` filters the shared catalog to
matched declaration and chapter units and composes by intersection with the
existing path, kind, project, unused, summary, size, top, and format selectors
where their output modes admit unit rows. Text prints the canonical handle,
two spaces, location, two spaces, and title. A chapter JSON row uses the list
object's existing fields in their existing order, adds `"section"` immediately
after `"id"`, and puts the declaration ID in `id` and exact component path in
`section`; a declaration row remains byte-for-byte the ordinary list row.
Invalid syntax, unknown vocabulary, disabled named sections, and ambiguous
exact literals are exit-2 invocation errors. A valid selector with no matches
prints nothing and exits 0. `--selector` is a flag; the positional remains the
scan path.

## 9. Managed guidance and editor parity

With at least one rule kind, `grund init` renders `### Chapter rules`
immediately after `### Citation directions`. It repeats the existing
`must`/`should` legend and emits one bullet per valid rule in qualified rule-ID
order: the exact authored sentence followed by its live rule citation. A
rule-enabled block uses v11 on the v10 base. With no rule kind, `init` retains
the v10 block byte-for-byte. `grund check` re-renders and byte-compares this
config-derived section; drift is `agents-init`.

Hard rule findings and `invalid-rule` travel through the same core report to
the LSP with the same message, code, title/citation range, and severity as CLI
JSON. The LSP adds no rule parser or evaluator. Suggestions remain CLI-only and
opt-in.

## 10. Documentation and executable examples

The release includes one guide at `docs/user-facing/rules.md`, one runnable
golden example at `examples/rules/`, links from the root README and
`examples/README.md`, and no second skill. The guide teaches opt-in and rationale
bodies, every subject and family, counts and modalities, every finding and both
channels, ordering, both deduplication directions, both command flags,
validation lifecycle, and every explicit phase-1 absence.

The guide has a marked `### Chapter rules` writing section. Both repository and
binary-embedded copies of `skills/grund-init/SKILL.md` contain a marked byte-
identical copy of that section and remain wholly byte-identical to one another.
The section includes every accepted family, every §3.5 refusal with its exact
rewrite, and the finding each example produces.

The runnable example contains at least one passing and one violated instance of
all five families. Its guide quotes every violated instance's exact finding and
its goldens cover `invalid-rule`, `chapter-cardinality`, rule-derived
`missing-citation`, ordinary and per-target `citation-cardinality`,
`uncited-unit`, `forbidden-citation` with its rule tail,
`suggested-citation`, and `discouraged-citation`; both channels and suggestion-
neutral exit behavior; two same-anchor off-count targets with shared-prefix
IDs; config-to-rule and rule-to-rule deduplication; and the refusal set.

Four independent pins prevent drift:

1. `examples/rules/expected.*` run through the shared e2e runner.
2. A marked-row extraction test submits every accepted/refused guide row to the
   released parser and asserts acceptance or exact refusal, rewrite, code, and
   channel.
3. Asset-sync tests compare guide section to repository skill and whole
   repository skill to the embedded copy.
4. The managed block's existing re-render byte comparison checks generated
   guidance.

## 11. Functional architecture constraint

Only two representations cross the parsing/evaluation boundary. The sentence
front end receives a title, rule identity/anchor, and config vocabulary and
returns `ParsedRule`: origin, anchor, subject selector, modality, relation,
normalized target set, and cardinality. It may not know `RuleFacts`, evaluate,
deduplicate, or synthesize findings.

Fact producers return only complete, immutable, versioned `RuleFacts` as §5.1
defines. The Markdown producer is phase 1's only producer, but the scanner is
rule-blind and contributes structural records rather than evaluating a rule.
The logic engine evaluates only `ParsedRule` over `RuleFacts`, deduplicates
semantic constraints, and emits located diagnostics through the shared report
boundary. It may not know sentence text, Markdown, scanner records, or file
layout beyond fact anchors. Architecture ground and dependency/replacement
tests specify the component placement separately.

## 12. Deliberate phase-1 absences

There is no `[settings]`, `config show --at`, path/folder/file subject,
exception phrase, `grund:allow` marker, definition, derived term, component
wildcard, wildcard subject namespace, new command verb, suppression mechanism,
SCIP/LSIF ingestion, symbol vocabulary, on-disk fact format, or Datalog
runtime. Settings must reuse this selector parser and independently answer
[§DF-fmt-suppression.2.2](../decisions/functional/DF-fmt-suppression.md#22-an-in-text-region-not-a-rule-keyed-by-declaration-section). A future adjacent-site exception may rely on rule prohibitions retaining their exact citation-site anchors. A future program producer inherits opaque identities, versioned immutable complete snapshots, repository-relative anchors, committed offline input, and evaluator independence, but no exchange format is chosen here.
