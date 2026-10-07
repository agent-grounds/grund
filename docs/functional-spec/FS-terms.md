# FS-terms: the shared vocabulary of the functional spec and the architecture

The words the functional spec and the architecture share are settled here, once, and
every spec and every architecture page leans on the groups below instead of defining
them again. A slice read on its own only answers on its own if its words mean one
thing, which is what the escalation ladder of
[§GOAL-token-economy.1](../goals.md#1-what-this-requires) assumes: the cheap read stops being cheap the moment a reader has
to open thirty-seven other files to learn which sense of *declaration*, *coordinate* or
*finding* is meant. A group here is a coordinate and a term is a **bold label**, so the
vocabulary joins that ladder at the group and no individual word becomes a citation
target.

## terms: Terms

**One definition per word.** A word defined here is not defined again elsewhere. A
document's Terms section may narrow a shared word only in this form:
`- **<term> (narrowed)** — §FS-terms.terms.<N>. In this document, within <scope>, <what this document adds>; elsewhere the shared definition applies.`
The entry states only what it adds and neither restates nor contradicts the shared
definition.

Every declaration the functional-spec index lists carries exactly one
`## terms: Terms` chapter, placed immediately after its lead. Every page the
architecture index links carries the same chapter, placed immediately after its
`placement` chapter, the index page itself excepted. That chapter opens with a **lean
line**: a citation of each group below that the document takes a word from, naming
those words in parentheses.

```text
Leans on §FS-terms.terms.1 (declaration, ID, section, coordinate),
§FS-terms.terms.2 (marker, citation), and §FS-terms.terms.5 (finding, severity).
```

The line is exhaustive over the shared words the document uses in the shared sense:
groups in ascending order, the words inside a group's parentheses in the order of their
rows, and a group the document takes no word from omitted entirely. After the lean line
the chapter carries only the words that document introduces or narrows, as bold labels
in a list with no child headings. A chapter holding nothing but a lean line is correct
rather than incomplete — the chapter's presence is what is required, and inventing a
word to fill it is the drift this file exists to close.

The *Displaces* sentences below bind prose. They do not reach a shipped surface or a
frozen text: the `value_chapter` config key, the `unknown reference` message, the
`namespace` bytes `grund init` writes from `templates/`, the ID `DF-chapter-rules`, and
the accepted GOAL, REQ and decision-record texts keep the words they have. The debt
they create is never a precondition: a retired word is rewritten as prose is written or
rewritten, so no change owes a pass of its own and none is held for one. A pass
commissioned on purpose is the other way to pay it and is permitted; it answers for the
words it names and for nothing else.

[§GRUND-links.2](../grund.md#2-holding-every-edit-to-them) promises that every cited ID and section coordinate resolves. A term
here is a bold label, not a coordinate, so `grund check` holds the sections that contain
the vocabulary and the citations that reach those sections — not the labels, the names
in a lean line's parentheses, their uniqueness or their meaning, or whether a document's
lean line is complete.

What holds the rest is written down home by home, so a reader knows what a green run has
said and what it has not. In the functional spec the chapter's presence is
[§RULE-terms](../rules/RULE-terms.md#rule-terms-each-fs-must-have-exactly-one-terms-chapter), a chapter rule this repository declares
([§FS-rules.3.1](FS-rules.md#31-chapter-presence)), so a spec that drops the chapter fails `grund check` rather than
passing it, and four fixed-syntax invariants gate beside that
rule: a word defined under two groups, a parenthesised word the group it cites does not
define, a word defined twice in one document's own chapter, and a shared word redefined
without the `(narrowed)` form. Two failures are left over — a label renamed here while
other documents still use the word, and a lean line that names a group the document no
longer leans on or omits one it does — and both are reported rather than asserted,
because [§FS-terms.senses](FS-terms.md#senses-what-counts-as-a-use) excludes two senses no machine recognizes and a gate over them
would ask authors to name words they do not use. In the architecture only the chapter's
presence is held, by a test rather than by a rule, because no rule sentence can except
the index page: all three failures still pass the gate there.

The two left over are what this file asks a reviewer to verify by hand, and the report of
[§FS-terms.senses.5](FS-terms.md#senses5-what-the-advisory-reports) is what they read instead of searching. For a change that touches
shared-term prose, this file, or a Terms chapter: run that report, reconcile what it says
about the documents the change touches with `grund refs FS-terms.terms.N`, and add to
each document's lean line, under its group, every shared word the change newly gives the
document in the shared sense. The review accounts for that and for any pre-existing
omission it is shown; certifying the completeness of unchanged prose beyond that is not
part of it.

### terms.1: Declarations and coordinates

- **declaration** — A heading or doc-comment line that introduces an ID. Its body is the fact. Displaces *spec* and *declaring spec*.
- **ID** — The stable name: kind and slug, with a number where the format has one. Displaces *handle*.
- **kind** — One `[[kinds]]` row: a class of declarations with a home, citable or not. Displaces *prefix* as a name for a class of declarations, and *namespace* as a name for an ID space; *prefix* stays for the literal letters an ID starts with (`MEFS-`), a path prefix, and any other string prefix.
- **home** — The file or folder a kind's declarations live in. Never where one ID is declared: say *declaring file* for that.
- **citable** — Can be the target of a citation.
- **body** — The declaration heading through the heading that closes it.
- **section** — A numbered or named heading inside a body. Its section path is the dotted tail. Displaces *chapter* and *point* in prose: a heading inside a body is a section, a whole declaration is a *declaration*, and a measurement taken per declaration or per section site is *per-coordinate*. *chapter* remains the rule grammar's subject-unit word, and stays for a named second-level section that a chapter rule or an obligation requires by name — the Terms chapter, the `placement` chapter; a numbered heading is a section.
- **field** — A kind's requirement that every declaration in it carry a given named section, as a `[[kinds]]` row or a chapter rule writes it. The section itself is a *chapter*; the field is the requirement on it. Does not displace the JSON output *field*, which is always qualified by the object it is on.
- **coordinate** — An ID with an optional section path: the complete target of a citation. Displaces *section reference*.
- **lead** — The prose of a declaration or section, cut at its first child section.
- **index** — A kind's index file, and its entries. Every other *index* becomes a table, a map or a cache.
- **catalog** — The scan's shared set of declarations and their citable sections, which query commands select or render. Bare *catalog* is that ID catalog; a compound whose qualifying noun names what it catalogs — the code catalog, a finding catalog, a rule catalog — is a term of its own.

### terms.2: Citations

- **marker** — The `§` character and nothing else. Every other *marker* becomes a delimiter (the managed block), a tag (a value), or a noun of its own.
- **citation** — Marker plus coordinate, wherever it appears. A bare citation only exists under `strict = false`. Displaces *reference*, *ref* and *cross-reference* in prose; the `[reference]` config table and the `--cross-refs` pass keep their names, and *cross-reference* stands where it means the markup wrapper rather than the citation inside it.
- **qualified citation** — A citation carrying an alias: `<§>alias/ID`. Displaces *cross-project reference* and *namespaced citation*.
- **shorthand** — The number-only citation form.
- **canonical form** — The persisted ID form `fmt` rewrites toward.
- **citation site** — One occurrence of a citation, located by path, line and column.

### terms.3: Source forms

- **source declaration** — A declaration inside a doc-comment. Displaces *inline declaration*, *code-form declaration* and *code-resident declaration*.
- **stub** — The one-line heading in a home that links to a source declaration. Every other *stub* keeps its own name: `id` writes a skeleton, `init` writes starter files.
- **doc-comment** — A comment documenting the definition after it, or the file. Hyphenated everywhere; never *doc comment*.
- **note** — An inline comment block that carries a citation and its rationale. Never a doc-comment. Displaces *inline citation site*.

### terms.4: Scanning and project structure

- **scan** — What a run does to its scope. A scan root is where it starts. Displaces *walk* and *walk root*.
- **scope** — The files a run reads. Default scope, full scope. Displaces *scan set*, *configured scope* and *tier*.
- **config root** — The directory holding `grund.toml`.
- **place** — A kind's home, or the complement of every home: one of the regions of a tree a `[[kinds]]` row governs. A per-place key written on a row says what one place does; written at the top of the file it says what every place does.
- **complement place** — The place that is every scanned file no home claims. v1 spells it the *homeless kind* and names it `code` by default.
- **language** — The class a file's syntax puts it in, and therefore how a declaration, a citation and a note are recognized in it. The class, never the keys that spell it; *host language* stays for the language a source declaration is embedded in.
- **workspace, member, alias** — The cross-project tree, one project in it, and the name a citation uses to reach it. *alias* displaces *namespace* in that sense.

### terms.5: Findings

- **finding** — one item of a report, or the single item a failed ID query emits on stderr: a `code`, a `message`, and nullable `path`, `line` and `sites`, where `sites` names every location only when one item has several. On the errors and warnings channels it carries `severity` `error` or `warning`; on the suggestions channel it carries no severity and `"channel":"suggestion"` stands in its place. Displaces *diagnostic* in prose except where FS-lsp names the protocol object; the internal `Diagnostic` type is unchanged.
- **severity** — the `error`-or-`warning` classification a finding carries on the errors or warnings channel; a suggestion carries none. The set is frozen at exactly those two, so a consumer filtering on `severity` never sees a suggestion. Displaces *level* for this sense.
- **suggestion** — a finding on the suggestions channel, emitted only under `grund check --suggestions`, carrying `"channel":"suggestion"` in place of a severity; it never changes exit status or replaces the `success` line. A channel, not a third severity.
- **caution** — a run-level message with no location. Displaces *CLI-level warning* and *announcement*.
- **verdict** — whether a run passes.
- **anchor** — a Markdown heading fragment only. Findings are located: replace *anchors at* / *anchored at* with *is located at* / *located at*; rule facts are keyed.

### terms.6: Rules and directions

- **direction, level** — The two parts of a `[citations]` rule: that one kind cites, or never cites, another, and its must / should / avoid / never strength. *rule* names the entry as a whole. *level* carries this sense alone, and heading depth and severity take their own words.
- **rule** — One rule the checker applies, as the config and the rule grammar name it: a `[citations]` entry as a whole, a `[[kinds]]` rule, or a chapter rule — a declaration whose title is one sentence. Checker behavior is a *check*.
- **unit** — What a rule is applied once per: the node its subject selects, so that a finding names that node. The compound *grounding unit* is the same word at `require_grounding`'s scale and keeps its own name, and a measurement's `unit` key names what is counted rather than what a rule is applied to.
- **grounded** — A grounding unit that cites at least one declaration.

### terms.7: Values and integrations

- **value, component, binding** — An opted-in declaration, one of its child headings, and the comment form that cites one component, or the root whose components it joins. Displaces *value field*.
- **fetcher, snapshot** — The configured fetch executable and the file it writes. *Integration* stays for rendering-layer clients only.

### terms.8: The architecture's own words

- **box** — A component's place in the system diagram: what feeds it, what it feeds. The diagram sense only; *out of the box* and *black-box* are idioms, not this word.
- **meter** — What measures a goal or a requirement from outside it, and is therefore not a component.

### terms.9: The configuration's concerns

- **schema** — The concern a config key belongs to when it says what exists and what a well-formed one looks like: a finding from a schema key is about one node on its own. The concern, never the whole key set — that list is [§FS-config.3](FS-config.md#3-keys).
- **rules** — The concern a config key belongs to when it says how nodes relate: a finding from a rules key needs at least two. Always plural and always the concern; one *rule* is [§FS-terms.terms.6](FS-terms.md#terms6-rules-and-directions)'s word, and a rules key is what makes one.
- **presentation** — The concern a config key belongs to when it decides bytes `grund` writes or shows: it produces no finding except where a written byte has drifted from what the config now renders. *rendering* stays the integrations' word for their own layer.
- **envelope** — The config keys outside the three concerns: the file's version and identity, and the workspace membership that says which projects there are. Read before any concern and constraining no node, so not a fourth one.

## senses: What counts as a use

The comparison a lean line stands or falls by is over senses, not tokens: a document
leans on a shared word where it uses that word in the shared sense, and an occurrence
that is not that sense is not a lean. Four exclusions say which occurrences those are.
Two of them are mechanical and the report of [§FS-terms.senses.5](FS-terms.md#senses5-what-the-advisory-reports) applies them; two are
judgements a reader makes, which is why that report is advice and the completeness of a
lean line is never a gate.

### senses.1: A word written in a different sense is not a lean

A document that writes `note:` for a stderr hint line is not leaning on *note*, whose row
gives the word to an inline comment block carrying a citation and its rationale. The
exclusion is over the sense the row defines, so recognizing it means reading the sentence
the word stands in. Not mechanical: the report names such an occurrence as a use, and the
reader it is written for discards the line.

### senses.2: A word occurring only inside a compound is not a use of that word

The exclusion runs in both directions, and what spells a compound is the hyphen: a
document that writes "CLI-level error" is not leaning on *level*, and a document that
writes *declaration-local* is not using *declaration*. A hyphenated compound is one name,
so the words inside it are not separately in play and a word counts only where it stands
on its own, with no word character and no hyphen on either side of it. Mechanical, and the
report applies it. The price is the occurrence where a compound really does carry its
parts' sense, and that price is paid on purpose: the report never gates, and reading a
compound as its parts would contradict the exclusion this one sits beside.

### senses.3: A word used only in the sense its own row retires is not a lean

A row that retires a word retires one sense of it, and prose still carrying that sense is
a debt the vocabulary has already named rather than a lean the lean line is missing. Not
mechanical, for the reason [§FS-terms.senses.1](FS-terms.md#senses1-a-word-written-in-a-different-sense-is-not-a-lean) gives: which sense an occurrence carries is
read, not matched.

### senses.4: A token that names rather than uses is not a lean

A token occurring only inside a fenced block, a link target, a heading anchor or a frozen
code name is a name, not a use — the `value_chapter` config key, the anchor a heading
rename moves, the example a fence holds. Mechanical, and the report applies it. It is the
same exclusion `tests/integration/test_functional_spec_retired_words.py` already reads
prose through, so one implementation serves both and the exclusion has one place to drift
from.

### senses.5: What the advisory reports

The report is per document and has two line forms, the first for a shared word the
document uses without leaning on it and the second for a word it leans on without using:

```text
FS-cli: uses *catalog* in prose but does not lean on it (terms.1)
FS-cli: leans on *kind* but the token appears nowhere in its prose
```

It reads a document's prose outside that document's own Terms chapter, applies
[§FS-terms.senses.2](FS-terms.md#senses2-a-word-occurring-only-inside-a-compound-is-not-a-use-of-that-word) and [§FS-terms.senses.4](FS-terms.md#senses4-a-token-that-names-rather-than-uses-is-not-a-lean), and cannot apply [§FS-terms.senses.1](FS-terms.md#senses1-a-word-written-in-a-different-sense-is-not-a-lean) or
[§FS-terms.senses.3](FS-terms.md#senses3-a-word-used-only-in-the-sense-its-own-row-retires-is-not-a-lean). Its order is fixed — every line of the first form before every line of
the second, documents in ascending ID order, and inside one document the shared words in
group order and then in row order — so two runs over one tree print the same bytes
([§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes)). It is silent unless it is asked for, and it is never an exit
condition: it changes no verdict, and a line it prints is a question for an author rather
than a failure. The count it reaches is not a contract and is never asserted, because a
frozen count is a gate wearing another name.
