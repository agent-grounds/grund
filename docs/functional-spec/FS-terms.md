# FS-terms: the shared vocabulary of the functional spec and the architecture

The words the functional spec and the architecture share are settled here, once, and
every spec and every architecture page leans on the groups below instead of defining
them again, because a slice read on its own answers on its own only if its words mean
one thing ([§GOAL-token-economy.1](../goals.md#1-what-this-requires)). A group here is a coordinate and a term is a **bold label**,
so a page cites a group and no individual word becomes a citation target.

## terms: Terms

**One definition per word.** A word defined here is not defined again elsewhere. A
document's Terms section may narrow a shared word only in this form:
`- **<term> (narrowed)** — §FS-terms.terms.<N>. In this document, within <scope>, <what this document adds>; elsewhere the shared definition applies.`
The entry states only what it adds and neither restates nor contradicts the shared
definition.

Every declaration the functional-spec index lists carries exactly one
`## terms: Terms` chapter, placed immediately after its lead, and every page the
architecture index links, the index page excepted, carries one placed immediately after
its `placement` chapter. The chapter opens with a **lean line**: a citation of each group
below that the document takes a word from, naming those words in parentheses.

```text
Leans on §FS-terms.terms.1 (declaration, ID, section, coordinate),
§FS-terms.terms.2 (marker, citation), and §FS-terms.terms.5 (finding, severity).
```

The lean line names every shared word the document uses in the shared sense
([§FS-terms.senses](FS-terms.md#senses-what-counts-as-a-use)): groups in ascending order, the words inside a group's parentheses in
the order of its rows, and a group the document takes no word from left out. After it
the chapter lists only the words the document introduces or narrows, as bold labels with
no child headings, and a chapter holding nothing but a lean line is complete.

A *Displaces* sentence below binds prose, never a shipped surface or a frozen text: a
config key such as `value_chapter`, a message, the bytes `grund init` writes, an ID, and
the accepted GOAL, REQ and decision-record texts keep the words they have. A retired word
is rewritten when its prose is next written; no change owes or waits for a sweep, and a
sweep made on purpose answers only for the words it names.

`grund check` holds the sections that contain the vocabulary and the citations that
reach them ([§GRUND-links.2](../grund.md#2-holding-every-edit-to-them)), not the labels, the words in a lean line's parentheses, their
uniqueness or meaning, or whether a lean line is complete. In the functional spec
[§RULE-terms](../rules/RULE-terms.md#rule-terms-each-fs-must-have-exactly-one-terms-chapter), a chapter rule this repository declares ([§FS-rules.3.1](FS-rules.md#31-chapter-presence)), fails `grund check` on
a spec that drops the chapter, and four fixed-syntax invariants gate beside it: a word
defined under two groups, a parenthesised word the group it cites does not define, a word
defined twice in one document's own chapter, and a shared word redefined without the
`(narrowed)` form. The invariants are held by
`tests/integration/test_functional_spec_terms_content.py`, and the advisory report of
[§FS-terms.senses](FS-terms.md#senses-what-counts-as-a-use) is its `--advisory` mode. Two failures are left over, a label renamed here
while other documents still use the word and a lean line that names a group the document
does not lean on or omits one it does, and both are reported rather than gated. In the
architecture only the chapter's presence is held, by a test rather than a rule, because no
rule sentence can except the index page.

A reviewer checks those two by hand. For a change that touches shared-term prose, this
file or a Terms chapter, the reviewer runs the advisory report, reconciles what it says
about the documents the change touches with `grund refs FS-terms.terms.N`, and adds to
each such document's lean line, under its group, every shared word the change newly gives
it in the shared sense. The review answers for that and for any pre-existing omission it
is shown, and certifies no other unchanged prose.

### terms.1: Declarations and coordinates

- **declaration** — A heading or doc-comment line that introduces an ID. Its body is the fact. Displaces *spec* and *declaring spec*.
- **ID** — The stable name: kind and slug, with a number where the format has one. Displaces *handle*.
- **kind** — One `[[kinds]]` row: a class of declarations with a home, citable or not. Displaces *prefix* as a name for a class of declarations, and *namespace* as a name for an ID space; *prefix* stays for the literal letters an ID starts with (`MEFS-`), a path prefix, and any other string prefix.
- **home** — The file or folder a kind's declarations live in. Never where one ID is declared: say *declaring file* for that.
- **citable** — Can be the target of a citation.
- **body** — The declaration heading through the heading that closes it.
- **section** — A numbered or named heading inside a body. Its section path is the dotted tail. The front matter of a declaration whose kind declares a `front-matter` field, and each of its top-level keys, are sections too, with no heading and outside the body ([§FS-fields.front-matter.coordinates](FS-fields.md#front-mattercoordinates-its-coordinates)). Displaces *chapter* and *point* in prose: a heading inside a body is a section, a whole declaration is a *declaration*, and a measurement taken per declaration or per section site is *per-coordinate*. *chapter* remains the rule grammar's subject-unit word, and stays for a named second-level section that a chapter rule or an obligation requires by name, such as the Terms chapter or the `placement` chapter.
- **relative heading** — A section heading inside a Markdown declaration body that writes one `.` and one component (`### .1: Username`) and takes the rest of its path from its parent heading ([§FS-declarations.section.relative](FS-declarations.md#sectionrelative-a-heading-may-write-only-the-component-it-adds)).
- **canonical path** — The complete section path a heading declares: the path a complete-path heading writes, or a relative heading's parent path, a `.` and its component. It is the section's one coordinate.
- **section lock** — The committed record, one file per cited declaration under `.grund/sections/`, of what each cited address it enrols ([§FS-section-lock.holds.entry](FS-section-lock.md#holdsentry-one-line-per-locked-address)) held when `grund fmt --write` last recorded it. Bare *lock* stays `grund.lock` ([§FS-remote-projects.terms](FS-remote-projects.md#terms-terms)).
- **field** — A kind's requirement that every declaration in it carry a given named section, as a `[[kinds]]` row, a chapter rule or a version-2 `fields.<name>` table writes it; a version-2 field also carries a strength, a placement and its content ([§FS-fields](FS-fields.md#fs-fields-a-v2-kind-states-the-shape-of-its-declarations-and-check-holds-every-declaration-to-it)). The section itself is a *chapter*; the field is the requirement on it. The one field that is no chapter is `front-matter`, which requires the file's front matter and holds its keys ([§FS-fields.front-matter](FS-fields.md#front-matter-front-matter-is-a-field)). Does not displace the JSON output *field*, which is always qualified by the object it is on.
- **form** — What a kind's declarations are: prose, values, or rules ([§FS-fields.forms](FS-fields.md#forms-form-lowers-to-the-version-1-contracts)).
- **closed** — Said of a kind whose declarations admit only its declared fields and numbered sections as direct sections, at the strength its `closed` key writes ([§FS-fields.declare.closed](FS-fields.md#declareclosed-closed-is-a-strength-on-the-row)).
- **coordinate** — An ID with an optional section path: the complete target of a citation. Displaces *section reference*.
- **lead** — The prose of a declaration or section, cut at its first child section.
- **index** — A kind's index file, and its entries. Every other *index* becomes a table, a map or a cache.
- **catalog** — The scan's shared set of declarations and their citable sections, which query commands select or render. Bare *catalog* is that ID catalog; a compound whose qualifying noun names what it catalogs, such as the code catalog, is a term of its own.

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
- **config root** — The directory config discovery stops at ([§FS-config.1](FS-config.md#1-file-location-and-discovery)).
- **place** — One path a row names as its home, a `files` or `folders` entry (v1's `file` or `folder`), resolved from the config root ([§FS-config-v2.schema.places](FS-config-v2.md#schemaplaces-rows-and-their-places)).
- **complement place** — Every scanned file no other row's place claims, owned by the row `code`; *the complement* for short ([§FS-config-v2.schema.places.complement](FS-config-v2.md#schemaplacescomplement-code-is-the-complement-by-name)).
- **language** — The class a file's syntax puts it in, and therefore how a declaration, a citation and a note are recognized in it. The class, never the keys that spell it; *host language* stays for the language a source declaration is embedded in.
- **workspace, member, alias** — The cross-project tree, one project in it, and the name a citation uses to reach it. *alias* displaces *namespace* in that sense.

### terms.5: Findings

- **finding** — One item of a report, or the single item a failed ID query emits on stderr: a `code`, a `message`, and nullable `path`, `line` and `sites` (its JSON shape and channels are [§FS-errors.5.1](FS-errors.md#51-on-stdout--the-commands-output)), where `sites` names every location only when one item has several. Displaces *diagnostic* in prose except where FS-lsp names the protocol object.
- **severity** — The `error`-or-`warning` classification a finding carries on the errors or warnings channel; a suggestion carries none, and the set stays those two ([§FS-config.6](FS-config.md#6-what-is-not-configured-here)). Displaces *level* for this sense.
- **suggestion** — A finding on the suggestions channel, carrying `"channel":"suggestion"` in place of a severity: a channel, not a third severity ([§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in)). It never changes exit status ([§FS-check.2](FS-check.md#2-outputs)) or replaces the `success` line.
- **caution** — A run-level message with no location. Displaces *CLI-level warning* and *announcement*.
- **verdict** — Whether a run passes.
- **anchor** — A Markdown heading fragment only. Displaces *anchors at* and *anchored at*: a finding is *located at* a site, and a rule fact is *keyed*.

### terms.6: Rules and directions

- **direction, level** — The two parts of a `[citations]` rule: that one kind cites, or never cites, another, and its must / should / avoid / never strength. *level* carries this sense alone, and heading depth and severity take their own words.
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

- **schema** — The concern a config key belongs to when it says what exists and what a well-formed one looks like: a finding from a schema key is about one node on its own. The concern, never the whole key set — that list is [§FS-config.3](FS-config.md#3-keys-version-1).
- **rules** — The concern a config key belongs to when it says how nodes relate: a finding from a rules key needs at least two. Always plural and always the concern; one *rule* is [§FS-terms.terms.6](FS-terms.md#terms6-rules-and-directions)'s word, and a rules key is what makes one.
- **presentation** — The concern a config key belongs to when it decides bytes `grund` writes or shows: it produces no finding except where a written byte has drifted from what the config now renders. *rendering* stays the integrations' word for their own layer.
- **envelope** — The config keys outside the three concerns: the file's version and identity, and the workspace membership that says which projects there are. Read before any concern and constraining no node, so not a fourth one.

## senses: What counts as a use

A document leans on a shared word where it uses that word in the shared sense, and an
occurrence in any other sense is not a lean. Four kinds of occurrence are not uses:

1. A word written in a different sense: `note:` before a stderr hint line is not a lean on
   *note*, which [§FS-terms.terms.3](FS-terms.md#terms3-source-forms) gives to an inline comment block.
2. A word inside a compound, in both directions: "CLI-level error" does not lean on
   *level*, and *declaration-local* does not use *declaration*. A hyphenated compound is
   one name, so a word counts only where it stands on its own, with no word character and
   no hyphen on either side of it.
3. A word used only in the sense its own row retires: that prose is a debt the vocabulary
   has already named, not a lean the lean line is missing.
4. A token that names rather than uses: one occurring only inside a fenced block, a link
   target, a heading anchor or a frozen code name, such as the `value_chapter` config key.
   The retired-word test, `tests/integration/test_functional_spec_retired_words.py`, reads
   prose through the same exclusion.

The second and fourth are matched; the first and third are read from the sentence the
word stands in, so no machine applies them and the completeness of a lean line is never
a gate.

The advisory report applies the second and fourth to each document's prose outside its
own Terms chapter, and reports an occurrence of the first or third kind as a use, for its
reader to discard. It prints a line for a shared word the document uses without leaning
on it, naming the group whose row defines the word, and a line for a word it leans on
without using:

```text
<ID>: uses *<word>* in prose but does not lean on it (terms.<N>)
<ID>: leans on *<word>* but the token appears nowhere in its prose
```

Every line of the first form comes before every line of the second, documents in
ascending ID order, and inside one document the shared words in group order and then in
row order, so two runs over one tree print the same bytes
([§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes)). The report is silent unless it is asked for and is never an
exit condition: a line it prints is a question for an author, and no count it reaches is
asserted, because a frozen count would be a gate.
