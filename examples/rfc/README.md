# RFC style: an Internet-Draft held to a grund schema

IETF documents follow a house style. RFC 7322, the RFC Style Guide, names
the sections every RFC has and the order they come in, and BCP 14 (RFC 2119
and RFC 8174) fixes what an all-capitals MUST means. Many
Internet-Drafts are written in Markdown and built with
[kramdown-rfc](https://github.com/cabo/kramdown-rfc). YAML front matter
carries the metadata and the reference list. Headings are written without
numbers (`# Introduction`), and xml2rfc numbers them. A reference is
`{{RFC9110}}`, or `{{Section 13.1.1 of RFC9110}}` for one section. Code that
implements a protocol cites it the same way, in comments such as "RFC 9110
section 15.5.11", and nothing checks those comments. When a section is
renumbered between two revisions of a draft, every comment that cites it
points at the wrong text.

This example is for the editor of an Internet-Draft that has a reference
implementation, and for the implementers who cite the draft. It sets up a
repository whose `grund.toml` holds the draft, the RFCs it depends on and
the code that implements it to one schema
([§FS-examples.2](../../docs/functional-spec/FS-examples.md#2-canonical-use-cases)):

- the draft is one declaration, and its ID is the draft's name;
- each section of the body is numbered, and cited by the number xml2rfc
  prints;
- Security Considerations and IANA Considerations, which RFC 7322 requires,
  are fields;
- each published RFC the draft cites is a snapshot of its section map, so a
  citation of RFC 9110 section 15.5.11 is checked offline;
- the code and its tests must cite the sections they implement.

The draft stays recognizably kramdown-rfc source, but grund forces some
departures from what kramdown-rfc builds as is.
[Where the source departs](#where-the-source-departs-from-kramdown-rfc)
lists every one.

The fixture is an original draft, Widget Synchronization over HTTP. A
client lists the widgets changed since a change token it holds. It updates
a widget with If-Match, so a concurrent edit comes back as a 412 with a
problem details object instead of being lost. The authors, the working
group and every domain are fictional. The example pins three runs over the
same fixture ([§FS-examples.5.5](../../docs/functional-spec/FS-examples.md#55-an-example-may-pin-several-runs)): the schema, the check, and the sites a
revision of Section 4 touches. None of them writes anything.

## The repository

```text
grund.toml                        the schema
draft-ietf-wsync-widget-sync.md   draft-ietf-wsync-widget-sync   the Internet-Draft
references/RFC2119.md             RFC2119   BCP 14
references/RFC8174.md             RFC8174   BCP 14: capitals only
references/RFC6585.md             RFC6585   428 Precondition Required
references/RFC9110.md             RFC9110   HTTP Semantics, every section mapped
references/RFC9457.md             RFC9457   Problem Details for HTTP APIs
scripts/fetch-rfc                 writes one snapshot from the RFC Editor
src/wsync.py                      the reference server
tests/test_wsync.py               its tests, one class per section
```

Open [`grund.toml`](2-check/repo/grund.toml) first, then
[`draft-ietf-wsync-widget-sync.md`](2-check/repo/draft-ietf-wsync-widget-sync.md),
then [`src/wsync.py`](2-check/repo/src/wsync.py): each docstring there
cites the section of the draft it implements.

## How RFC style maps onto grund

| RFC style | grund | In this repository |
|---|---|---|
| an Internet-Draft, named without its `-NN` revision | one declaration of kind `draft`, whose ID is the draft's name | `draft-ietf-wsync-widget-sync` |
| the YAML front matter | text before the declaration, which grund does not read | `title`, `author`, `normative` |
| the Abstract (RFC 7322, section 4.3) | the declaration's lead, which `grund <ID>` prints | the text after `--- abstract` |
| a section, numbered by xml2rfc | a numbered section, its number written in the heading ([§FS-config.3.3](../../docs/functional-spec/FS-config.md#33-section-paths--arbitrary-nesting-depth)) | `### 4.2. Conditional Update` |
| a required section (RFC 7322, sections 4.8.3 and 4.8.5) | a field: a named chapter at H2 ([§FS-fields](../../docs/functional-spec/FS-fields.md#fs-fields-a-v2-kind-states-the-shape-of-its-declarations-and-check-holds-every-declaration-to-it)) | `## security: Security Considerations` |
| an appendix, lettered A, B, ... | a named section whose name is its letter | `## a: Example Exchange` |
| an unnumbered section, `{:numbered="false"}` | a named section | `## acknowledgments: Acknowledgments` |
| a published RFC, in the reference list | a snapshot: its title, abstract and section map | `references/RFC9110.md` |
| a reference to an RFC's section | a citation of the snapshot's section | `§RFC9110.13.1.1` |
| a reference to the draft's own section | a citation with the draft's whole ID | `§draft-ietf-wsync-widget-sync.4.2` |
| "RFC 9110 section 15.5.11" in a code comment | a citation that `grund check` resolves | `src/wsync.py` |

A published RFC's ID is its citation tag. RFC 7322, section 3.5, writes the
tag `[RFC2119]`, with no space, and kramdown-rfc writes `{{RFC2119}}`. The
`RFC` row's `id_format = "{kind}{number}"` gives `RFC2119`, because the
literal between two placeholders may be empty
([§FS-config.3.2](../../docs/functional-spec/FS-config.md#32-id--id-grammar)).

## Step 1: what the repository is held to

```bash
grund schema examples/rfc/1-schema/repo
echo $?    # 0
```

`grund schema` reads the configuration alone ([§FS-schema](../../docs/functional-spec/FS-schema.md#fs-schema-grund-schema-shows-what-a-repository-is-held-to-read-from-its-configuration-alone)):

```text
widget-sync · grund.toml · v2

kinds and places
  draft  kind              draft-ietf-wsync-widget-sync.md
           field security  must
           field iana      must
  RFC    kind              references/
  impl   place             src/
  test   place             tests/
  code   complement        every scanned file no other row claims

citations                  grounded
  draft  must      RFC
  RFC    must not  everything else
  impl   must      draft   must · h3
  test   must      draft
  code   —

must = check error · warn = check warning · should = suggestion (check --suggestions)
may = allowed · must not / warn not / should not = forbidden at that strength
A|B = one obligation, met by citing either · — = no direction rule names it
```

Read it as RFC style:

- **The two fields** are RFC 7322, section 4.8. Every RFC has a Security
  Considerations section, and every Internet-Draft an IANA Considerations
  section, even one that only says "This document has no IANA actions."
  (RFC 8126, section 9.1). Neither field has a position. RFC 7322 lists
  IANA before Security, the widely used
  [I-D template](https://github.com/martinthomson/internet-draft-template)
  lists them the other way round, and drafts use both orders.
- **draft must cite RFC**: a draft cites the RFCs it builds on, and BCP 14
  at least.
- **RFC must not cite anything**: a snapshot is what the RFC Editor
  published, and it cites nothing in this repository.
- **impl must cite draft**, grounded at `h3`: every docstring in `src/`
  cites what it implements. A Python function's docstring is indented under
  its `def`, and `h2` grounds only unindented doc comments, so `h2` would
  hold the module docstring alone ([§FS-check.3.6.2.2](../../docs/functional-spec/FS-check.md#3622-in-a-source-file)).
- **test must cite draft**: a test file cites the sections it tests.

## Step 2: the repository holds

```bash
grund check examples/rfc/2-check/repo
echo $?    # 0
```

```text
success
```

Every citation resolves. That includes the draft's six citations of RFC
9110 sections and the server's citation of section 15.5.11, all against
the committed snapshot, with no network. The draft has both required
fields, and every docstring in `src/` cites the draft.

## Step 3: what a revision of Section 4 touches

The working group revises Section 4 for the next revision of the draft.
`grund refs` lists every site that cites Section 4 or a section beneath it
([§FS-refs](../../docs/functional-spec/FS-refs.md#fs-refs-grund-lists-every-citation-of-an-id)):

```bash
grund refs draft-ietf-wsync-widget-sync.4 --descendants examples/rfc/3-refs/repo
echo $?    # 0
```

```text
draft-ietf-wsync-widget-sync.md:65: §draft-ietf-wsync-widget-sync.4.1
draft-ietf-wsync-widget-sync.md:66: §draft-ietf-wsync-widget-sync.4.2
draft-ietf-wsync-widget-sync.md:67: §draft-ietf-wsync-widget-sync.4.3
draft-ietf-wsync-widget-sync.md:170: §draft-ietf-wsync-widget-sync.4.3
src/wsync.py:39: §draft-ietf-wsync-widget-sync.4.1
src/wsync.py:51: §draft-ietf-wsync-widget-sync.4.2
src/wsync.py:64: §draft-ietf-wsync-widget-sync.4.3
tests/test_wsync.py:13: §draft-ietf-wsync-widget-sync.4.1
tests/test_wsync.py:31: §draft-ietf-wsync-widget-sync.4.2
tests/test_wsync.py:31: §draft-ietf-wsync-widget-sync.4.3
```

That is the review list for the revision: the draft's own cross-references,
including the IANA registration at line 170, which names Section 4.3, and
the code and tests that implement each section. An implementer who reaches
`§draft-ietf-wsync-widget-sync.4.2` in `src/wsync.py` reads the text it
implements with `grund draft-ietf-wsync-widget-sync.4.2`:

```text
### 4.2. Conditional Update

A client updates a widget with a PUT request that MUST carry an If-Match
header field (§RFC9110.13.1.1) holding the entity tag of the widget it
last received. A server MUST respond to a PUT without If-Match with
428 (Precondition Required) (§RFC6585) and leave the widget unchanged.

A server MUST evaluate the precondition as §RFC9110.13.2 specifies. When
it is false, the server MUST respond with 412 (Precondition Failed)
(§RFC9110.15.5.13) and leave the widget unchanged. When it is true, the
server applies the update, increments "revision", and responds with
200 (OK), the new representation, and its new entity tag.
```

## Writing a draft in this shape

- **The title heading** goes after the front matter's closing `...`:
  `# draft-ietf-wsync-widget-sync: Widget Synchronization over HTTP`.
  kramdown-rfc reads everything before `--- abstract` as its YAML header,
  where the line is a comment.
- **Number every section** the way xml2rfc will, with RFC style's trailing
  period: `## 4. Synchronization`, then `### 4.2. Conditional Update`. The
  heading's depth must agree with its number.
- **Name the rest.** A field is `## security: Security Considerations`. An
  appendix is named by its letter, so `## a: Example Exchange` is cited as
  `§draft-ietf-wsync-widget-sync.a`, which the rendered draft calls
  Appendix A. An unnumbered section such as Acknowledgments gets a name of
  its own.
- **Cite a section of an RFC** where kramdown-rfc would write
  `{{Section 13.1.1 of RFC9110}}`: `(§RFC9110.13.1.1)`. Cite a section of
  the draft with its whole ID, `§draft-ietf-wsync-widget-sync.4.2`.
- **Cite from code** in the docstring of what implements the section, and
  inline where a status code is chosen:
  `return 410, {}, None  # §RFC9110.15.5.11: the client relists`.
- **Add an RFC** by writing its snapshot, which needs the network once:
  `scripts/fetch-rfc RFC9111 > references/RFC9111.md`. The script reads
  the RFC Editor's metadata, and for an RFC published in RFCXML v3 (RFC 8650
  onward) its XML too, and prints the title, the abstract and one heading
  per numbered section and appendix. An older RFC, such as RFC 6585, has no
  such XML, so its snapshot has no sections and the draft cites it whole.
  An RFC does not change after publication, so its snapshot does not go
  stale.

## Where the source departs from kramdown-rfc

These are the places where grund's model forces the source away from what
kramdown-rfc and the I-D template expect:

1. **A title heading.** A declaration is a heading, but kramdown-rfc takes
   the title from `title:` in the front matter. So the title is written
   twice, and grund reads only the heading's copy.
2. **Every heading one level down.** A declaration's body ends at the next
   heading as high as its own ([§FS-declarations.body](../../docs/functional-spec/FS-declarations.md#body-where-a-declarations-body-ends)), so a top-level
   section is `##`, not `#`. kramdown-rfc reads levels as written and
   expects the first to be `#`. Its source warns `Bad section nesting`
   otherwise (read from its source, not run here).
3. **Numbers in the headings.** Every heading inside a body must carry a
   section path, or `grund check` reports it as an unmarked heading
   ([§FS-declarations.checks.unmarked-heading](../../docs/functional-spec/FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading)). xml2rfc would number the
   headings again, so a published build has to strip the written numbers
   first.
4. **Handles on the named sections.** A field is the chapter whose name is
   the field's, matched by name and then by exact title
   ([§FS-fields.identity](../../docs/functional-spec/FS-fields.md#identity-identity)). The template's heading
   `# Security Considerations {#security}` gives the same name as a
   kramdown anchor. Kept beside the handle, as
   `## security: Security Considerations {#security}`, it fails, because
   grund reads the anchor as part of the title:

   ```text
   draft-ietf-wsync-widget-sync.md:137: error: draft-ietf-wsync-widget-sync.security is titled "Security Considerations {#security}"; kind draft titles it "Security Considerations"
   ```

   The named sections also have no number. The draft's
   `§draft-ietf-wsync-widget-sync.security` is what the rendered draft calls
   Section 5.
5. **Citations instead of kramdown references.** `{{RFC9110}}` and
   `{{intro}}` are not citations to grund, so the draft writes `§` citations,
   which are plain text to kramdown-rfc. A citation of the draft's own
   section spells the whole ID, because a bare `<§>4.2` is an error
   ([§FS-check.3.24](../../docs/functional-spec/FS-check.md#324-declaration-local-section-citation)). The `normative:` and `informative:` lists stay in the
   front matter for kramdown-rfc, and nothing checks them against the
   citations.
6. **BCP 14 written out.** `{::boilerplate bcp14-tagged}` expands only when
   kramdown-rfc builds the draft, and grund cannot see the two citations it
   makes. The draft writes the RFC 8174 text, citing `§RFC2119` and
   `§RFC8174`.
7. **No `grund fmt` on the draft.** `fmt` would turn each citation into a
   link to a file in this repository, which means nothing in a published
   draft. So `[presentation.fmt] exclude` keeps `fmt` off the draft, and
   `fmt` does not expand a citation there either.
8. **Snapshots written by a script, not by `grund fetch`.** `grund fetch`
   accepts subsections exactly one heading level below the declaration
   ([§FS-fetch.3](../../docs/functional-spec/FS-fetch.md#3-accepted-declaration)), while RFC 9110's section map is four levels deep. With
   `fetch = "scripts/fetch-rfc"` on the `RFC` row and the RFC 9457 snapshot
   removed, `grund fetch RFC9457` exits 2:

   ```text
   error: fetch output heading depth 3 is invalid for a depth-1 declaration
   ```

   Flattening the output to one level would pass `fetch` and fail
   `grund check`, which requires `#### 3.1.1.` for a three-part number
   ([§FS-config.3.3.2](../../docs/functional-spec/FS-config.md#332-section_heading_levels--heading-depth-against-path-depth)). So the script writes the file, and the `RFC` row has
   no `fetch`.

Points 1 to 4 together mean kramdown-rfc cannot build the intended draft
from this file as it stands. A build step has to drop the title heading,
promote every other heading one level, and strip the numbers and handles
first.

## Break it

Copy the fixture somewhere writable and make four edits a draft editor
plausibly makes. Each is an ordinary edit to a kramdown-rfc draft that
kramdown-rfc would not refuse, and `grund check` refuses all four:

```bash
demo_root=$(mktemp -d)
cp -R examples/rfc/2-check/repo/. "$demo_root/"
cd "$demo_root"
```

1. Write the IANA heading as the I-D template writes it, one level down:
   replace `## iana: IANA Considerations` with
   `## IANA Considerations {#iana}`.
2. Promote `### 4.2. Conditional Update` to `## 4.2. Conditional Update`.
   A build would number it Section 5 and renumber everything after it.
3. Merge Section 4.3 into 4.2, as a revision might: delete the line
   `### 4.3. Conflict Reporting` and the blank line after it.
4. In Security Considerations, before the paragraph that starts "Listing
   changes", add a paragraph that cites HTTP caching, which has no snapshot
   yet:

   ```markdown
   A listing MAY be stored by a cache (§RFC9111.3).
   ```

```console
$ grund check
draft-ietf-wsync-widget-sync.md:44: error: draft-ietf-wsync-widget-sync has no iana field; kind draft declares it
draft-ietf-wsync-widget-sync.md:67: error: missing section draft-ietf-wsync-widget-sync.4.3
draft-ietf-wsync-widget-sync.md:116: error: section draft-ietf-wsync-widget-sync.4.2 heading level mismatch: expected ### (level 3), found ## (level 2)
draft-ietf-wsync-widget-sync.md:150: error: unknown reference RFC9111; did you mean RFC9110?
draft-ietf-wsync-widget-sync.md:155: error: unmarked heading inside draft-ietf-wsync-widget-sync; number it (## 5. IANA Considerations {#iana}) as draft-ietf-wsync-widget-sync.5, declare an ID, or use a bold label; this became an error in grund 0.16.0
draft-ietf-wsync-widget-sync.md:170: error: missing section draft-ietf-wsync-widget-sync.4.3
src/wsync.py:64: error: missing section draft-ietf-wsync-widget-sync.4.3
tests/test_wsync.py:31: error: missing section draft-ietf-wsync-widget-sync.4.3
```

The repairs:

- The IANA heading takes the field's form, `## iana: IANA Considerations`.
  Numbering it as the finding at line 155 suggests would leave the field
  missing.
- The promoted heading goes back to `###`. The number written in a heading
  can only stay the number xml2rfc prints while the depth agrees with it.
- The four citations of Section 4.3 are the ones Step 3 listed. Re-point
  them to `§draft-ietf-wsync-widget-sync.4.2`.
- RFC 9111 is HTTP Caching, not a misspelling of RFC 9110. Write its
  snapshot with `scripts/fetch-rfc RFC9111 > references/RFC9111.md`.

## Trade-offs

What the schema buys:

- **Section citations in code are checked.** A comment that cites RFC 9110
  section 15.5.11, or Section 4.2 of the draft, resolves against committed
  text or fails the check. Nothing else reads such a comment.
- **A revision has a review list.** `grund refs` names the code, tests and
  cross-references that cite a section before the section changes.
- **Required sections stay.** A draft cannot lose its Security
  Considerations or IANA Considerations section.
- **The number cited is the number in the heading.** The heading's depth
  must agree with its number, so a heading promoted or demoted by mistake is
  caught.

What it costs:

- **The source no longer builds as is.** It needs a step that undoes the
  title heading, the extra heading level, the numbers and the handles before
  kramdown-rfc sees it. Every departure is listed above.
- **Numbers are written by hand.** grund reports a number written twice,
  but not numbers out of order: headings numbered 1, 7, 3 pass. xml2rfc
  numbers by position, so inserting a section means renumbering every
  heading after it, and every citation of them.
- **Named sections have no number, and numbered ones cannot be required.**
  Security Considerations is cited as `security`, not by the number readers
  of the RFC know it by. RFC 7322 also requires an Introduction as the
  first section, but a field is a named chapter, so requiring it would take
  away its number, Section 1.
- **References are written twice.** The front matter's `normative:` list is
  for kramdown-rfc and the `§` citations are for grund, and nothing checks
  that RFC 7322's rule, a reference for every citation and a citation for
  every reference, holds between them.
- **An older RFC is cited whole.** Only an RFC published in RFCXML v3 has a
  section map, so a section of RFC 6585 or RFC 2119 cannot be cited.
- **Publication renames the draft.** When the draft becomes an RFC, its ID
  becomes an `RFC` tag. Every citation of it changes, and `grund refs`
  lists them.
