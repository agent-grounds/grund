# DF-fetch-source-home: fetch refuses a `file` home the text scan reads as source

**Date:** 2026-11-05

## status: Accepted

## 1. Context

Name a source file as a fetched kind's `file` home and `grund fetch <ID>` appends a
Markdown H2 to it, or creates it holding only that H2, and exits 0 without a word
([issue #1224](https://github.com/agent-grounds/grund/issues/1224)). The writer finds
what it owns with a Markdown reader, and the scan reads that file through its
language's comments, so fetch and the scan read one file two ways. `grund list` and
`grund check` read the file correctly before the fetch and after it, and neither
sees what was written.

Four shapes, one write, with the same `fs.py` on 0.21.0:

| The home | `grund fetch FS-a` | `grund FS-a --full` after it | Python after it |
|---|---|---|---|
| declares `FS-a` in a docstring | exit 0, H2 appended | the docstring's body | `SyntaxError` |
| declares only `FS-b` | exit 0, H2 appended | `ID not found: FS-a` | `SyntaxError` |
| holds only `x = 1` | exit 0, H2 appended | `ID not found: FS-a` | `SyntaxError` |
| absent | exit 0, file created | `ID not found: FS-a` | `SyntaxError` |

Rust, Go, JavaScript, Ruby and `fs.txt` under `txt = "python"` took the same exit and the
same write; only the Python files were put through a parser. No fixture here relies on
the write: every fetched kind with a non-`.md` `file` home in `tests/e2e/cases` and
`examples` is a value source, already refused, or a name mapped to Markdown.

## 2. Decision

[§FS-fetch.4.2](../../functional-spec/FS-fetch.md#42-a-home-read-as-source-is-refused) refuses, at
exit 2 with the tree byte-identical, by the home's path and kind and before the
integration runs. Four answers lose.

- **Refuse when the configuration loads.** A rule kind's non-Markdown home is refused
  there ([§FS-config-v2.schema.sources.rules](../../functional-spec/FS-config-v2.md#schemasourcesrules-a-rule-kinds-file-home-is-markdown-by-its-language)), and that also passes the reproducer. But
  `check`, `list`, `show` and `refs` read this project correctly, so they would stop on a
  project they have nothing against, for a write they never make.
- **Refuse only a home that declares the ID.** That is the reported shape and it passes
  the reproducer, and it leaves the other three rows of the table writing what no read
  returns.
- **Replace the doc-comment in place.** That is a new writer for every comment form the
  scan reads, with its own rules for markers, indentation and the shape of a fetched
  body. Nothing asks for it, and the refusal leaves it open.
- **Refuse a home no reader reads as well.** `fs.dat`, a non-value kind's `fs.json` and an
  unscanned `fs.yaml` take the same write and no read returns it, but they are not read
  two ways: `check` already says the file `is not read`, and a tool other than grund may
  be what reads them. That is a different question and a different ticket.

`check` must not send a reader to a command that refuses, so at such a home its tail
becomes `— declare it in that source file`
([§FS-check.3.1.3.1](../../functional-spec/FS-check.md#3131-a-home-fetch-refuses)). The message's
base, its code, its severity and the exit stay what they are.

## 3. Compatibility

The verdict of `grund fetch <ID>` moves from exit 0 to exit 2 on such a home, and that
takes the route of [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids).
Checking and querying an unchanged tree keep their verdicts. Its five conditions, in order.

### 3.1. The prior prohibition

The prior verdict violated [§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own), a prohibition that already applied when that verdict shipped.
It limits `fetch` to atomically replacing or inserting "only its validated named declaration in
the configured snapshot home", and [§FS-fetch.6](../../functional-spec/FS-fetch.md#6-stability-and-ownership) extends that. A declaration is a
heading or doc-comment line that introduces an ID
([§FS-terms.terms.1](../../functional-spec/FS-terms.md#terms1-declarations-and-coordinates)), and the scan is what says which line does. In a file the scan
reads as source a Markdown heading introduces nothing, as the table's third column shows.
So what the old fetch wrote into such a home was not its validated named declaration
under the project's own reading of that file: it replaced no declaration and inserted
none, and it reported that it had.

That is a reading of the words, and the record should say where it is strongest and
where it is not.

- **The home declares the ID.** Strongest. A declaration of the ID stands in the file and
  fetch neither replaced it nor could: a reader asking for the ID still gets the old body
  after a fetch that said it succeeded.
- **The home declares only other IDs, or nothing.** The same clause holds without any
  declaration of the ID in the file, because it turns on what was written and not on what
  was there. Every byte that was there survived, so nothing is lost in the sense of
  [§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own)'s closing sentence about what lies outside the marker. What
  changed is that a file its language could read now holds text outside any declaration,
  which is what a writer bounded by an ownership marker may not add.
- **The home is absent.** The hardest, and it has no bytes to damage. Nothing is
  overwritten; fetch creates a file the project named and writes accepted bytes into it.
  The claim is only the first one: those bytes are no declaration, so the one write
  [§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own) allows did not happen and a file did appear, under a name that says
  it is source, holding what that language rejects. `grund FS-a` answers `ID not found`, and
  a citing `check` still says `no snapshot in fs.py — run grund fetch FS-a` after that command
  has run. It also cannot be split off from the row above it: a file the fetch created is, on
  the next fetch, a home that declares nothing, so giving it a window of its own would have one
  file exit 0 and then exit 2 with nothing changed. A person who reads "only its validated named
  declaration" as bytes rather than as what the scan reads could call this shape a judgement
  rather than a prohibition, and this record names that so it can be overruled.

[§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise) carries none of the shapes, and this record does not lean
on it: [§FS-fetch.4](../../functional-spec/FS-fetch.md#4-file-home-write) said to insert and to create, so the old write had a defined meaning.

### 3.2. Why neither other route fits

[§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) does not fit. A window is a release that warns and keeps
the old success, and the old success here is the write: there is no way to keep it that
does not go on appending to source files and creating unparsable ones, reporting success
for a fact that never arrives. A warning buys no repository time to move, because
nothing that moves depends on the write.

[§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) does not fit, because its fix must be one documented command the
tool ships. The maintainer is choosing between keeping an authored fact in the source
file, without `fetch`, and getting the fetched fact, in Markdown. No command can make that
choice, and `config migrate` cannot see which they meant.

### 3.3. The named release

The release note below names 0.22.0.

### 3.4. What each finding names

The refusal names the home, spelled from the report base, and the action. Each way out it
offers was run on this tree.

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

The `check` tail names the same home, and it works: with `fs.py` declaring the ID in a
docstring, a citation that failed with that tail resolves and check exits 0. It is a change
to an existing finding's text, which [§FS-errors.3](../../functional-spec/FS-errors.md#3-message-text) sends through a deprecation path. A window
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

## release-note: Release note

- In grund 0.22.0, `grund fetch <ID>` refuses with exit 2 when the kind's `file` home is
  one the text scan reads as source: `fs.py`, or `fs.txt` under `txt = "python"`. In a
  version-2 file the language its name resolves to decides. In a version-1 file it is a
  name other than `.md` whose extension is in the effective `[scan] extensions`. Previously
  it exited 0 and appended a Markdown heading to that file, or created the file holding only
  that heading, which no command ever read back and which left a Python file unparsable. The
  refusal comes before the integration runs, whatever the file holds and whether or not it
  exists, names the home, and leaves every file unchanged. Move the kind and its declarations
  to a Markdown file or a folder, or remove its `fetch`. A file an earlier fetch appended to
  keeps what was written, so restore it from version control. A home read as Markdown under any
  name, and a home the scan does not read, are written as before, and `check`, `list`, `show`
  and `refs` keep their verdicts. The `dangling` and `missing-snapshot` messages at such a home
  now end `— declare it in that source file` in place of `— run grund fetch <ID>`. This verdict
  changes under [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids):
  the prior success violated [§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own),
  and every finding names its location and the action to take
  ([§DF-fetch-source-home](DF-fetch-source-home.md#df-fetch-source-home-fetch-refuses-a-file-home-the-text-scan-reads-as-source)).
