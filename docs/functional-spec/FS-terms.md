# FS-terms: the shared vocabulary of the functional spec

The words the functional spec shares are settled here, once, and every spec leans on
the groups below instead of defining them again. A slice read on its own only answers
on its own if its words mean one thing, which is what the escalation ladder of
[§GOAL-token-economy.1](../goals.md#1-what-this-requires) assumes: the cheap read stops being cheap the moment a reader has
to open twenty-three other files to learn which sense of *declaration*, *coordinate* or
*finding* is meant. A group here is a coordinate and a term is a **bold label**, so the
vocabulary joins that ladder at the group and no individual word becomes a citation
target.

## terms: Terms

**One definition per word.** A word defined here is not defined again elsewhere. A
spec's Terms section may narrow a shared word only in this form:
`- **<term> (narrowed)** — §FS-terms.terms.<N>. In this spec, within <scope>, <what this spec adds>; elsewhere the shared definition applies.`
The entry states only what it adds and neither restates nor contradicts the shared
definition.

Every declaration the functional-spec index lists carries exactly one
`## terms: Terms` chapter, placed immediately after its lead, and that chapter opens
with a **lean line**: a citation of each group below that the spec takes a word from,
naming those words in parentheses.

```text
Leans on §FS-terms.terms.1 (declaration, ID, section, coordinate),
§FS-terms.terms.2 (marker, citation), and §FS-terms.terms.5 (finding, severity).
```

The line is exhaustive over the shared words the spec uses in the shared sense: groups
in ascending order, the words inside a group's parentheses in the order of their rows,
and a group the spec takes no word from omitted entirely. After the lean line the
chapter carries only the words that spec introduces or narrows, as bold labels in a
list with no child headings. A chapter holding nothing but a lean line is correct
rather than incomplete — the chapter's presence is what is required, and inventing a
word to fill it is the drift this file exists to close.

The *Displaces* sentences below bind prose. They do not reach a shipped surface or a
frozen text: the `value_chapter` config key, the `unknown reference` message, the
`namespace` bytes `grund init` writes from `templates/`, the ID `DF-chapter-rules`, and
the accepted GOAL, REQ and decision-record texts keep the words they have, and a
retired word is rewritten as prose is written or rewritten rather than in a pass of its
own.

[§GRUND-consistency](../grund.md#grund-consistency-the-structure-stays-consistent) promises that every cited ID and section coordinate resolves. A term
here is a bold label, not a coordinate, so `grund check` holds the sections that contain
the vocabulary and the citations that reach those sections — not the labels, the names
in a lean line's parentheses, their uniqueness or their meaning, or whether a spec's
lean line is complete. Three failures therefore pass the gate: a label renamed here
while other specs still use the word; one word defined under two groups, or in a spec's
own Terms section, with two meanings; and a lean line that names a group the spec no
longer leans on, or omits one it does.

Those failures are what this file asks a reviewer to verify by hand. For a change that
touches shared-term prose, this file, or a Terms section: before adding a label, search
every Terms section for a second definition; before renaming or removing one, search
functional-spec prose for the old word and reconcile the results with
`grund refs FS-terms.terms.N`; check every narrowing against the one-definition rule
above; and add to that spec's lean line, under its group, every shared word the change
newly gives the spec in the shared sense. The review accounts for that and for any
pre-existing omission it is shown; certifying the completeness of unchanged prose beyond
that is not part of it.

The comparison is over senses, not tokens. A spec that writes `note:` for a stderr hint
line, or "CLI-level error", is not leaning on *note* or on *level*, and a word used only
in the sense its own row retires is not a lean either.

### terms.1: Declarations and coordinates

- **declaration** — A heading or doc-comment line that introduces an ID. Its body is the fact. Displaces *spec*, *fact* and *declaring spec*.
- **ID** — The stable name: kind and slug, with a number where the format has one. Displaces *handle*.
- **kind** — One `[[kinds]]` row: a class of declarations with a home, citable or not. Displaces *prefix*, and *namespace* as a name for an ID space.
- **home** — The file or folder a kind's declarations live in. Never where one ID is declared: say *declaring file* for that.
- **citable** — Can be the target of a citation.
- **body** — The declaration heading through the heading that closes it.
- **section** — A numbered or named heading inside a body. Its section path is the dotted tail. Displaces *chapter* and *point* in prose; *chapter* remains the rule grammar's subject-unit word.
- **coordinate** — An ID with an optional section path: the complete target of a citation. Displaces *section reference*.
- **lead** — The prose of a declaration or section, cut at its first child section.
- **index** — A kind's index file, and its entries. Every other *index* becomes a table, a map or a cache.
- **catalog** — The scan's shared set of declarations and their citable sections, which query commands select or render. The ID-catalog sense only: FS-errors keeps *code catalog* as a compound of its own.

### terms.2: Citations

- **marker** — The `§` character and nothing else. Every other *marker* becomes a delimiter (the managed block), a tag (a value), or a noun of its own.
- **citation** — Marker plus coordinate, wherever it appears. A bare citation only exists under `strict = false`. Displaces *reference*, *ref* and *cross-reference* in prose; the `[reference]` config table keeps its name.
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
- **workspace, member, alias** — The cross-project tree, one project in it, and the name a citation uses to reach it. *alias* displaces *namespace* in that sense.

### terms.5: Findings

- **finding** — one item of a report, or the single item a failed ID query emits on stderr: a `code`, a `message`, and nullable `path`, `line` and `sites`, where `sites` names every location only when one item has several. On the errors and warnings channels it carries `severity` `error` or `warning`; on the suggestions channel it carries no severity and `"channel":"suggestion"` stands in its place. Displaces *diagnostic* in prose except where FS-lsp names the protocol object; the internal `Diagnostic` type is unchanged.
- **severity** — the `error`-or-`warning` classification a finding carries on the errors or warnings channel; a suggestion carries none. The set is frozen at exactly those two, so a consumer filtering on `severity` never sees a suggestion. Displaces *level* for this sense.
- **suggestion** — a finding on the suggestions channel, emitted only under `grund check --suggestions`, carrying `"channel":"suggestion"` in place of a severity; it never changes exit status or replaces the `success` marker. A channel, not a third severity.
- **caution** — a run-level message with no location. Displaces *CLI-level warning* and *announcement*.
- **verdict** — whether a run passes.
- **anchor** — a Markdown heading fragment only. Findings are located: replace *anchors at* / *anchored at* with *is located at* / *located at*; rule facts are keyed.

### terms.6: Rules and directions

- **direction, level** — A `[citations]` rule that one kind cites, or never cites, another, and its must / should / avoid / never strength. Displaces *rule* for these; *level* carries this sense alone, and heading depth and severity take their own words.
- **rule** — A chapter rule only: a declaration whose title is one sentence. Checker behavior is a *check*.
- **grounded** — A grounding unit that cites at least one declaration.

### terms.7: Values and integrations

- **value, component, binding** — An opted-in declaration, one of its child headings, and the comment form that cites one component. Displaces *value field*.
- **fetcher, snapshot** — The configured fetch executable and the file it writes. *Integration* stays for rendering-layer clients only.
