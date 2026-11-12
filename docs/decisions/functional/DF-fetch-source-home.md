# DF-fetch-source-home: fetch refuses a `file` home the text scan reads as source

**Date:** 2026-11-05

## status: Accepted

## 1. Context

Name a source file as a fetched kind's `file` home and `grund fetch <ID>` appends the
integration's Markdown H2 and what it printed under it, or creates the file holding only
those, and exits 0 without a word
([issue #1224](https://github.com/agent-grounds/grund/issues/1224)). The writer finds
what it owns with a Markdown reader, and the scan reads that file through its
language's comments, so fetch and the scan read one file two ways. `grund list` and
`grund check` read the file correctly before the fetch and after it: they do not see the
heading, and of what lies under it they see only what is a comment that language reads.

Four shapes, one write, with the same `fs.py` on 0.21.0 and an integration printing an H2
and prose:

| The home | `grund fetch FS-a` | `grund FS-a --full` after it | Python after it |
|---|---|---|---|
| declares `FS-a` in a docstring | exit 0, H2 appended | the docstring's body | `SyntaxError` |
| declares only `FS-b` | exit 0, H2 appended | `ID not found: FS-a` | `SyntaxError` |
| holds only `x = 1` | exit 0, H2 appended | `ID not found: FS-a` | `SyntaxError` |
| absent | exit 0, file created | `ID not found: FS-a` | `SyntaxError` |

The last two columns are that prose's. What a read returns depends on what the integration
printed under the H2:

- A comment that cites is read as a citation: `grund refs` lists it, and one naming an unknown
  ID fails check. It declares nothing, so `grund FS-a` still says `ID not found`. `fs.py` and
  `fs.rb` both parse, because `## FS-a: Alpha` is itself a comment in both.
- A docstring that declares the ID is read as the declaration. Section 3.1 says what that does
  on a home that already declares it, and section 3.6 on any other, where the old fetch worked.

Rust, Go, JavaScript, Ruby and `fs.txt` under `txt = "python"` took the same exit and the
same write; only the Python files were put through a parser. No fixture that predates this
change relies on the write: before it, every fetched kind with a non-`.md` `file` home in
`tests/e2e/cases` and `examples` was a value source, already refused, or a name mapped to
Markdown.

## 2. Decision

[§FS-fetch.4.2](../../functional-spec/FS-fetch.md#42-a-home-read-as-source-is-refused) refuses, at
exit 2 with the tree byte-identical, by the home's path and kind and before the
integration runs. Five answers lose.

- **Refuse when the configuration loads.** A rule kind's non-Markdown home is refused
  there ([§FS-config-v2.schema.sources.rules](../../functional-spec/FS-config-v2.md#schemasourcesrules-every-home-of-a-rule-kind-is-judged-by-its-reading)), and that also passes the reproducer. But
  `check`, `list`, `show` and `refs` read this project correctly, so they would stop on a
  project they have nothing against, for a write they never make.
- **Refuse only a home that declares the ID.** That is the reported shape and it passes
  the reproducer, and it leaves the other three rows of the table writing prose no read
  returns into source files.
- **Refuse only what the next scan would not read back, after running the integration.**
  That is the one answer that keeps the tree section 3.6 names. It runs the integration,
  which may be a network call, before it can say anything about the home, and has one home
  exit 0 for one output and 2 for the next. What the scan reads back is also not what the
  language reads: the same shape in Rust, JavaScript or C reads green in grund under a heading
  the compiler rejects. Whether a fetch installs only what the next scan reads back is
  [issue #1205](https://github.com/agent-grounds/grund/issues/1205)'s, whose admission rule as
  written also asks that the declaration's body take in all of the output, which this tree's
  does not: the H2 and the docstring's title and quotes stay outside it. This record does not
  build that.
- **Replace the doc-comment in place.** That is a new writer for every comment form the
  scan reads, with its own rules for markers, indentation and the shape of a fetched
  body. Nothing asks for it, and the refusal leaves it open.
- **Refuse a home no reader reads as well.** `fs.dat`, a hidden `.fs.py`, a non-value kind's
  `fs.json` and an unscanned `fs.yaml` take the same write and no read returns it, but they
  are not read two ways: `check` already says the file `is not read`, and a tool other than grund may
  be what reads them. That is a different question and a different ticket.

`check` must not send a reader to a command that refuses, so at such a home its tail
becomes `— declare it in that source file`, or `— move the kind to a Markdown file or folder` for
a value kind, which no comment can declare
([§FS-check.3.1.3.1](../../functional-spec/FS-check.md#3131-a-home-fetch-refuses)). The message's
base, its code, its severity and the exit stay what they are.

## 3. Compatibility

The verdict of `grund fetch <ID>` moves from exit 0 to exit 2 on such a home. Checking and
querying an unchanged tree keep their verdicts. On every tree but one that takes the route of
[§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), whose five conditions follow in order, and section 3.6 names the
tree that route does not carry.

### 3.1. The prior prohibition

The prior verdict violated [§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own), a prohibition that already applied when that verdict shipped,
on every tree but the one section 3.6 names.
It limits `fetch` to atomically replacing or inserting "only its validated named declaration in
the configured snapshot home", and [§FS-fetch.6](../../functional-spec/FS-fetch.md#6-stability-and-ownership) extends that. A declaration is a
heading or doc-comment line that introduces an ID
([§FS-terms.terms.1](../../functional-spec/FS-terms.md#terms1-declarations-and-coordinates)), and the scan is what says which line does. In a file the scan
reads as source a Markdown heading introduces nothing: with prose under it a read says
`ID not found`, and with a docstring under it `grund list` names the docstring's line, not
the heading's. So where what was written holds no declaration of the ID, it was not fetch's
validated named declaration under the project's own reading of that file: it replaced no
declaration and inserted none, and it reported that it had.

That is a reading of the words, and the record should say where it is strongest and
where it is not. These shapes are of what was written holding no declaration of the ID, and a
body that only cites is one of them: it adds a citation and still no declaration.

- **The home declares the ID.** Strongest. A declaration of the ID stands in the file and
  fetch neither replaced it nor could: a reader asking for the ID still gets the old body
  after a fetch that said it succeeded.
- **The home declares only other IDs, or nothing.** The same clause holds without any
  declaration of the ID in the file, because it turns on what was written and not on what
  was there. Every byte that was there survived that fetch, so nothing is lost in the sense of
  [§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own)'s closing sentence about what lies outside the marker. What
  changed is that a file its language could read now holds text outside any declaration,
  which is what a writer bounded by an ownership marker may not add.
- **The home is absent.** The hardest, and it has no bytes to damage. Nothing is
  overwritten; fetch creates a file the project named and writes accepted bytes into it.
  The claim is only the first one: those bytes are no declaration, so the one write
  [§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own) allows did not happen and a file did appear, under a name that says
  it is source, holding what that language rejects under a prose body, and under a body that
  only cites, what it accepts but the scan reads no declaration in. `grund FS-a` answers
  `ID not found`, and a citing `check` still says `no snapshot in fs.py — run grund fetch FS-a` after that command
  has run. It also cannot be split off from the row above it: a file the fetch created is, on
  the next fetch, a home that declares nothing, so giving it a window of its own would have one
  file exit 0 and then exit 2 with nothing changed. A person who reads "only its validated named
  declaration" as bytes rather than as what the scan reads could call this shape a judgement
  rather than a prohibition, and this record names that so it can be overruled.

A body that does declare the ID, on a home that already declares it, stands on the same words:
the second declaration replaces nothing and is no insertion of that declaration, as
[§DF-fetch-folder-declaring-file](DF-fetch-folder-declaring-file.md#df-fetch-folder-declaring-file-a-folder-home-fetch-refuses-an-id-a-file-it-does-not-replace-already-declares) reads them, and a green tree fails check on the duplicate, with
`grund FS-a` saying `ambiguous ID`. On any other home that body is section 3.6's.

[§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise) carries none of the shapes, and this record does not lean
on it: [§FS-fetch.4](../../functional-spec/FS-fetch.md#4-file-home-write) said to insert and to create, so the old write had a defined meaning.

### 3.2. Why neither other route fits

[§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) does not fit these trees. A window is a release that warns
and keeps the old success, and the old success here is the write: there is no way to keep it
that does not go on appending to source files what no read returns, leaving a Python file
unparsable under a prose body and reporting success for a fact that never arrives. A warning
buys them no time to move, because nothing on them depends on the write. Section 3.6 names the
tree where something does.

[§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) does not fit, because its fix must be one documented command the
tool ships. The maintainer is choosing between keeping an authored fact in the source
file, without `fetch`, and getting the fetched fact, in Markdown. No command can make that
choice, and `config migrate` cannot see which they meant.

### 3.3. The named release

The release note below names 0.22.0.

### 3.4. What each finding names

The refusal names the home, spelled from the report base, and the action. Each way out it
offers was run on this tree, with a docstring in `fs.py` and an integration printing prose;
section 3.6 says what each does on the tree it names.

- **Move the kind and its declarations to a Markdown file.** With `files = ["fs.md"]` and
  the docstring taken out of `fs.py`, or moved into `fs.md`, fetch exits 0, check exits 0,
  `grund FS-a --full` prints the fetched body and `fs.py` parses. The declarations have to
  move too: with the docstring left in `fs.py`, check already fails on
  `FS-a must be declared in fs.md (single-file kind)`, so a reader is told before fetching.
- **Move them to a folder.** With `folders = ["src"]`, fetch exits 0 and check exits 0 once
  the index the folder needs lists the snapshot, which check names. The folder may hold
  the source file, declaring other IDs or nothing. A folder home takes an H1 where a file
  home takes an H2 ([§FS-fetch.3](../../functional-spec/FS-fetch.md#3-accepted-declaration)), so the integration changes with it, and fetch's own
  refusal says so.
- **Remove `fetch`.** Check exits 0 and `grund FS-a --full` prints the source file's body.
  A `[rules.resolution]` entry, or a version-1 `resolve`, needs `fetch`, so it goes too, and
  the configuration refusal names it.

The `check` tail names the same home, and it works. For a non-value kind, with `fs.py` declaring
the ID in a docstring, a citation that failed with that tail resolves and check exits 0. For a
value kind, with the kind moved to a `fs.md` that exists, or to a folder whose index lists the
snapshot, fetch exits 0 and check exits 0: only a move to Markdown gets a value kind to green,
because a comment in `fs.py` is refused as `value declarations in text homes must be Markdown headings`, and without `fetch` the
citation still dangles.

Either tail is a change to an existing finding's text, which [§FS-errors.3](../../functional-spec/FS-errors.md#3-message-text) sends through a deprecation path. A window
here would have `check` name a command that now refuses, so the tail moves with the
refusal in one release, as the value-source tail did. The message's base survives as a
verbatim prefix, so only a consumer of the tail's own words sees it. Its migration is the
stable `code`, which does not move, and the release note names the change.

### 3.5. What this licenses

Only this correction: a `file` home the text scan reads as source, at `fetch`.
It is not a licence for refusing a file the scan does not read, nor for any refusal when the
configuration loads, nor for moving the verdict of `check`, `list`, `show` or `refs`. The
prohibition is not one this change invented: [§FS-fetch.4](../../functional-spec/FS-fetch.md#4-file-home-write) already refused a value source
because it "holds no Markdown declaration", and [§FS-fetch.4.1](../../functional-spec/FS-fetch.md#41-an-absent-value-home-is-refused-with-the-way-through) already said of a home the
project does not read as Markdown that fetch "writes Markdown and so is not the way through".

### 3.6. The tree that worked

On one tree the old success was real: a non-value kind at `fs.py`, a home that is absent or
holds anything but a declaration of the ID, and an integration that prints the H2 and, under
it, a docstring declaring the ID. With `x = 1` in `fs.py`, 0.21.0 ran:

```text
$ grund fetch FS-a           # exit 0, appends the H2 and the docstring
$ grund list                 # exit 0, the docstring's line, not the H2's
DOC-one  docs/doc.md:1  Doc
FS-a     fs.py:5  Alpha
$ grund FS-a --full          # exit 0, the fetched body
Fetched body citing §DOC-one.
$ grund refs DOC-one         # exit 0, the body's citation counts
fs.py:7: §DOC-one
$ grund check                # exit 0
success
$ grund fetch FS-a           # exit 0 after the upstream changed; only that ID's body follows
```

An absent home and one declaring only another ID ran the same, and Python parses each file,
because the H2 is a comment there and the docstring a string.

Condition 1 of [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids) does not carry this tree, and this record does
not claim it does: what was written there is a declaration the scan reads, so the reading in
section 3.1 does not reach it. Nor does [§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise), because the old write
had a meaning and a read returned it. The path the requirement leaves it is the default one,
[§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): a release that warns and keeps the old success, then a refusal
no earlier than the next. Refusing it now instead is a judgement this record makes rather than
something the requirement grants, named so that a person can overrule it, as
[§DF-fetch-folder-declaring-file](DF-fetch-folder-declaring-file.md#df-fetch-folder-declaring-file-a-folder-home-fetch-refuses-an-id-a-file-it-does-not-replace-already-declares) names its own. Overruling it means that for one release fetch
at such a home runs the integration, installs what the next scan would read back as the ID with
a warning naming the release that refuses, and refuses anything else; that
[§FS-fetch.4.2](../../functional-spec/FS-fetch.md#42-a-home-read-as-source-is-refused) stops deciding before the integration runs; and that the `check` tail at that
home names the command again.

What can be said for the judgement, as far as the runs reach, on 0.21.0 and, for the upgrade,
on this change:

- **A window cannot single this tree out without judging the output.** The refusal goes by path
  and kind before the integration runs, and this tree's path and kind are the reported tree's.
  Telling them apart takes running the integration and reading what it printed, the answer
  section 2 turns down, which is [issue #1205](https://github.com/agent-grounds/grund/issues/1205)'s
  question. A window on every source home goes on writing prose into source files, which is the
  defect reported.
- **The integration has to be written for the home, and print one fact twice.** Fetch takes only
  the H2: a docstring alone is refused with `fetch output must begin with one Markdown declaration`.
  The scan takes only the comment. No fixture, example or guide teaches this.
- **The file stays valid only where the H2 is itself a comment, and the runs found it declaring
  only in Python.** Rust, JavaScript and C read green in `list`, `show` and `check`, under a line
  their compilers reject: `rustc`, `node --check` and `gcc -fsyntax-only` each exit 1. Go reads
  green too and was not compiled here. Ruby and shell accept the line, but no comment fetch
  accepts declares there: `#FS-a: Alpha` declares nothing, and `# FS-a: Alpha` is refused with
  `fetch output heading depth 1 is invalid for a depth-2 declaration`.
- **A refresh takes more than the fact.** The writer reads the file as Markdown, so the ID's block
  runs to the next H2 or the end of the file. With a `def f()` written below the block, a refresh
  exits 0, check says `success`, `fs.py` parses, and `def f()` is gone, which the closing sentence
  of [§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own) forbids on any reading. That is not this tree as the fetch leaves it,
  which holds nothing below the block, so condition 1 still does not carry it, but keeping the
  write keeps this.
- **The upgrade is loud and takes no read away.** On the tree 0.21.0 left, `check`, `list`, `show`
  and `refs` answer as they did, and every fetch exits 2, unchanged, naming the ways out. The
  refresh is what it loses, and that is a real loss. A checkout that does not keep `fs.py` keeps
  nothing: fetch exits 2 and a citing check fails with the tail `declare it in that source file`
  until its owner takes a way out.

Each way out was run on this change, from the bytes 0.21.0 left.

- **Remove `fetch`.** With the file as it stands, check exits 0, `grund FS-a --full` prints the
  body and `fs.py` parses. The fact stops refreshing, and from there it is an authored declaration.
- **Move the kind to a Markdown file.** In one change: `files = ["fs.md"]`, the H2 and the docstring
  out of `fs.py`, and the integration printing the H2 and the body without the docstring. Fetch
  exits 0, check exits 0, `grund FS-a --full` prints the fetched body, `grund refs` finds the
  citation in `fs.md` and `fs.py` parses. The integration has to change: left as it is, fetch and
  check still exit 0 and the read returns the docstring's quotes and title as part of the body.
  With the docstring left in `fs.py`, check fails on
  `FS-a must be declared in fs.md (single-file kind)`, and a fetch exits 0 regardless and adds
  `duplicate declaration of FS-a`.
- **Move them to a folder.** The integration changes here too, to an H1, as section 3.4 says: left
  as it is, fetch refuses with `fetch output heading depth 2 is invalid for a depth-1 declaration`.
  With the H1 and the body, fetch exits 0, check exits 0 and the read returns the body.

## release-note: Release note

- In grund 0.22.0, `grund fetch <ID>` refuses with exit 2 when the kind's `file` home is
  one the text scan reads as source: `fs.py`, or `fs.txt` under `txt = "python"`. In a
  version-2 file the language its name resolves to decides. In a version-1 file it is a
  name other than `.md` whose extension is in the effective `[scan] extensions`. A hidden name
  such as `.fs.py` is not read, so it is no such home. Previously it exited 0 and appended the
  integration's Markdown heading and body to that file, or created the file holding them. Under a
  prose body no command read that text back, and a Python file stopped parsing. The
  refusal comes before the integration runs, whatever the file holds and whether or not it
  exists, names the home, and leaves every file unchanged. Move the kind and its declarations
  to a Markdown file or a folder, or, for a kind that is not a value kind, remove its `fetch`. A
  file an earlier fetch appended to
  keeps what was written: where no read returns it, restore the file from version control. One
  tree loses a fetch that worked: an integration that prints, under its heading, a docstring
  declaring the ID, into a Python home that does not already declare it. `grund list`,
  `grund <ID>` and `check` read that docstring, and a fetch refreshed it. A tree 0.21.0 left that
  way keeps `check`, `list`, `show` and `refs` as they were; only its fetch now exits 2 and changes
  nothing, so the fact no longer refreshes, and a checkout that does not keep the file has no such
  declaration. Keep the file and remove `fetch`, which ends the refresh, or, in one change, move
  the kind to a Markdown file, take the heading and the docstring out of the source file, have the
  integration print the heading and the body without the docstring, and fetch. A home read as Markdown under any
  name, and a home the scan does not read, are written as before, and `check`, `list`, `show`
  and `refs` keep their verdicts. The `dangling` and `missing-snapshot` messages at such a home
  now end `— declare it in that source file`, or `— move the kind to a Markdown file or folder`
  for a value kind, in place of `— run grund fetch <ID>`. This verdict changes under [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids)
  where what the fetch wrote was no declaration of the ID or a duplicate of one: that prior success
  violated [§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own),
  and every finding names its location and the action to take. On the tree that worked it is a
  judgement the record names ([§DF-fetch-source-home](DF-fetch-source-home.md#df-fetch-source-home-fetch-refuses-a-file-home-the-text-scan-reads-as-source)).
