# FS-fetch: grund materializes one external fact snapshot

`grund fetch <ID>` asks the selected kind's configured fetcher for one complete Markdown
declaration and stores it in that kind's home. It is the only command that materializes an
external fact; every later scan and query reads what it saved, offline
([§REQ-runs-offline](../requirements/REQ-runs-offline.md#req-runs-offline-verification-never-depends-on-an-external-service)). Projecting a declared remote repository with `--remote <alias>` is
specified ahead of its release ([§FS-fetch.remote](FS-fetch.md#remote-remote-projections)).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, home, body, section, catalog),
[§FS-terms.terms.2](FS-terms.md#terms2-citations) (citation), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, config root, workspace, member, alias),
and [§FS-terms.terms.7](FS-terms.md#terms7-values-and-integrations) (fetcher, snapshot).

## 1. Input and project selection

`grund fetch` takes exactly one ID, local or workspace-qualified, and no section; the `--remote`
form of [§FS-fetch.remote.selector](FS-fetch.md#remoteselector---remote-alias-projects-one-declared-remote) takes none, and naming both is a usage error. The ID selects
its project as an ID query does ([§FS-workspace.8.1](FS-workspace.md#81-grund-aliasid), [§FS-imports.writes.answer](FS-imports.md#writesanswer-where-a-qualified-id-is-fetched-has-one-answer)), but parses by
the kind's effective grammar alone, without the catalog's off-grammar spellings
([§FS-config.3.2](FS-config.md#32-id--id-grammar)). `grund fetch alias/TICKET-1234` loads that member's config and home, names
the qualified ID in its messages, and passes only `TICKET-1234` to the fetcher. A malformed
alias path is refused as `grund show` refuses it ([§FS-workspace.8.11](FS-workspace.md#811-a-malformed-alias-path-names-the-offending-segment)).

An ID that does not parse is a bare query failure, exit `1` ([§FS-errors.2.3](FS-errors.md#23-bare-query-failure)). A kind with no
`fetch`, or one that the selected project does not give exactly one configured `file` or
`folder` home, is refused.

## 2. Integration invocation

The kind's `fetch` names one executable ([§FS-fetch.2.1](FS-fetch.md#21-a-value-that-names-no-executable)), resolved from the selected project's
config root when relative, and run directly, never through a shell, with the local unqualified
ID as its only argument, the user's environment and nothing on stdin: running `grund fetch` is
the authorization to execute what the repository configures. Its stdout is the snapshot and is
never copied to `grund`'s stdout. Exit `0` leads to validation ([§FS-fetch.3](FS-fetch.md#3-accepted-declaration)); a spawn failure,
a signal or a non-zero exit is an error naming the fetcher and the ID ([§FS-fetch.2.2](FS-fetch.md#22-captured-stderr-explains-an-unsuccessful-integration)).

### 2.1 A value that names no executable

The key names an executable under either config version. A value that is empty or holds only
whitespace names none; any other is taken literally, so `" scripts/fetch "` is spawned as
written and fails as a spawn failure if nothing is there. The test is of the string alone, so
version 2 refuses a key that names nothing when the config loads, even where a file of that
very name exists at the config root ([§FS-config-v2.checks.config-fetch-names-nothing](FS-config-v2.md#checksconfig-fetch-names-nothing-a-rows-fetch-names-no-executable)). A
version 1 row loads ([§FS-config.5.2](FS-config.md#52-a-versions-meaning-moves-only-as-backwards-compatibility-allows)). Under it, a whitespace-only value that names a file at
the config root is run, and `grund check` asks the disk the same question ([§FS-check.3.1](FS-check.md#31-dangling-citation));
any other value that names nothing runs nothing, and `grund fetch` prints the spawn failure's
line with the key's location and repair where the operating system's reason would stand:

```text
error: cannot run fetch integration `` for AR-001-power: grund.toml:6: [[kinds]] `AR` fetch names no executable; name the program, or if this kind is not fetched remove the key and any `resolve`, and its missing IDs are then errors
```

The location is the config file that holds the key, a member's own in a workspace, and the
value is quoted as written. The repair is the sentence every surface that reports the key
prints, in the spelling of the project's config version ([§FS-config-migrate.3.1.10](FS-config-migrate.md#3110-a-fetch-that-names-no-executable),
[§FS-check.3.1](FS-check.md#31-dangling-citation), [§DF-fetch-names-an-executable.2.5](../decisions/functional/DF-fetch-names-an-executable.md#25-one-repair-true-of-every-tree)). The test comes immediately before the spawn
would, so every other refusal made before the fetcher runs comes first.

### 2.2 Captured stderr explains an unsuccessful integration

`grund` captures the fetcher's stderr. On a non-zero exit or a signal, a nonempty capture is
appended after the error's verbatim prefix
``error: fetch integration `<fetch>` for <ID> exited <status>`` ([§FS-errors.3](FS-errors.md#3-message-text)), `<status>`
being the exit code or `signal`, with its line breaks kept and invalid UTF-8 shown as
replacement characters, never refused as snapshot data. An empty capture adds nothing, and a successful
fetcher's stderr is never shown.

## 3. Accepted declaration

The fetcher's stdout must be UTF-8 and one Markdown declaration of the requested local ID
beginning on its first line: an H2 (`## TICKET-1234: Title`) for a `file` home, an H1
(`# TICKET-1234: Title`) for a `folder` home. What declares it is the selected project's scan's
answer, never a second reading of `fetch`'s, and the output is accepted only when admission
says so ([§FS-fetch.admission](FS-fetch.md#admission-admission-the-next-scan-reads-back-exactly-what-was-asked-for)). It may hold prose, fenced blocks and subsections
([§FS-fetch.3.2](FS-fetch.md#32-a-subsection-is-a-heading-the-next-scan-reads-as-a-section)), with no depth limit of `fetch`'s own, so a value chapter and its roots are
written ([§FS-config.3.4.13](FS-config.md#3413-value_chapter--the-chapter-whose-named-children-are-values)).

Output that is not the requested declaration alone, opening on its first line at the home's
level, is refused with one of:

```text
error: fetch output must begin with one Markdown declaration
error: fetch output declares `<OTHER>` instead of requested `<ID>`
error: fetch output heading depth <n> is invalid for a depth-<d> declaration
error: fetch output must contain exactly one declaration and no leading content
error: fetch output must contain exactly one Markdown declaration
```

The third is for the declaration's heading at the wrong level, and the fourth for content
before it ([§FS-fetch.admission.1](FS-fetch.md#admission1-the-requested-declaration-is-there-once-and-its-body-takes-in-all-of-the-output)). A section at the wrong level is refused in `check`'s words
([§FS-fetch.admission.3](FS-fetch.md#admission3-the-structural-rules-check-applies-report-nothing-new)).

Validation comes before any write, and the accepted bytes are installed verbatim: no
timestamp, no whitespace normalization, no `fmt` and no citation sanitizing, so a citation in
the body is live on the next scan ([§FS-check.1.1](FS-check.md#11-recognized-citations)).

### 3.1 A nested declaration is refused

A heading inside the requested declaration that the next scan reads as declaring an ID of any
configured kind, whatever its depth below the declaration's own level and whatever its ID, the
requested one included, is refused, naming the nested ID: one `fetch <ID>` owns only that
declaration ([§FS-fetch.6](FS-fetch.md#6-stability-and-ownership)). Which headings declare is the
scan's reading of the output alone ([§FS-fetch.admission.1](FS-fetch.md#admission1-the-requested-declaration-is-there-once-and-its-body-takes-in-all-of-the-output)); subsections and fenced headings
declare nothing.

The rule binds only what `fetch` writes: a nested declaration written by hand stays legal,
and `fetch` never deletes one. A declaration the scan reads in the home inside the range the
requested one owns, at any depth, would be deleted, so the fetch is refused, `<file>:<line>`
naming its heading ([§FS-fetch.admission.2](FS-fetch.md#admission2-every-other-declaration-in-the-home-reads-as-it-did-before)):

```text
error: cannot fetch <ID>: <file>:<line> declares <OTHER> inside <ID>, and replacing <ID> would delete it; move <OTHER> out of <ID>, and fetch again
```

### 3.2. A subsection is a heading the next scan reads as a section

A subsection is a heading inside the declaration that the next scan, under the selected
project's configuration, reads as a section of it: numbered, named where named sections are
enabled, or relative ([§FS-declarations.checks.unmarked-heading](FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading),
[§FS-declarations.section.relative](FS-declarations.md#sectionrelative-a-heading-may-write-only-the-component-it-adds)). Its level is held as `check` holds it
([§FS-declarations.checks.section-heading-level](FS-declarations.md#checkssection-heading-level-section-heading-level-mismatch), [§FS-fetch.admission.3](FS-fetch.md#admission3-the-structural-rules-check-applies-report-nothing-new)). Any other heading
inside it that declares nothing, at any depth and whatever its text, is refused rather than
written for the next `grund check` to turn red:

```text
error: fetch output line <n>: heading `<heading>` is not a section of `<ID>`; number it, fence it, or make it a bold label
```

`<n>` is the heading's 1-based line in the output and `<heading>` that line, trailing
whitespace removed. The line stands in place of `check`'s for the same finding, because
`check`'s suggests declaring an ID there, which [§FS-fetch.3.1](FS-fetch.md#31-a-nested-declaration-is-refused) refuses.

## 4. File-home write

For `file = "<home>"`, `fetch` replaces only the requested declaration, or inserts it among
the home's sibling H2 declarations when there is none ([§FS-fetch.4.order](FS-fetch.md#4order-insertion-follows-the-effective-format-comparing-the-number-numerically)); every byte outside
it stays, line endings included. The owned declaration is the one the selected project's scan
reads in the home, at whatever level and under whatever heading, nested included, and it owns
the range the scan records: its heading line to its body's last non-blank line
([§FS-declarations.body](FS-declarations.md#body-where-a-declarations-body-ends)). The blank lines after that range stay as the separator, except that
blank lines ending the output take its place, so a refetch leaves the same bytes
([§FS-fetch.admission.5](FS-fetch.md#admission5-fetching-the-same-output-again-leaves-the-same-bytes)). An ID declared in another file is refused ([§FS-fetch.5.3](FS-fetch.md#53-a-declaration-outside-the-home-prevents-installation)).

An absent home is read as empty and created holding the declaration alone, except a value
kind's or a `value_chapter` kind's, which must exist ([§FS-fetch.4.1](FS-fetch.md#41-an-absent-value-home-is-refused-with-the-way-through)). A home that is a value
source ([§FS-values.terms](FS-values.md#terms-terms)), in either format, is refused by its path and kind before the
fetcher runs; a value kind whose `file` home is a Markdown file is written like any other. A
home read as source is refused, absent or not ([§FS-fetch.4.2](FS-fetch.md#42-a-home-read-as-source-is-refused)).

The file is installed atomically once the whole result is ready. A home that cannot be read,
parsed unambiguously or replaced atomically is refused, and one that cannot be decoded is
named as [§FS-fetch.5.4](FS-fetch.md#54-a-home-file-that-cannot-be-decoded-is-named) says.

### 4.order: Insertion follows the effective format, comparing the number numerically

An absent ID is inserted before the first sibling H2 declaration whose ID compares greater, or
appended when none does. IDs compare field by field along the requested kind's effective ID
format: `{number}` by numeric value, whatever its zero-padding, and kind and slug text
lexically, digits in a slug included. So `{kind}-{number}` puts `GR-9` before `GR-100`, and
`{kind}-{slug}-{number}` puts `TICKET-alpha-9` before `TICKET-zeta-1`. The rule picks an
insertion point and never sorts: a declared ID is replaced where it stands ([§FS-fetch.4](FS-fetch.md#4-file-home-write)), even
in a file whose declarations are out of order.

### 4.1. An absent value home is refused with the way through

The home of a value kind, or of a kind that sets `value_chapter`, is absent when its lookup
fails because a path component is not there; any other failure is [§FS-fetch.4.3](FS-fetch.md#43-a-value-home-that-cannot-be-looked-up-is-refused-with-its-reason)'s. The
configuration then refuses to load, exit `2`, with a hint below the refusal; `grund check` and
`grund list` print the same two lines, and the hint changes no verdict:

```text
error: grund.toml:6: value home for kind `CONST` does not exist: values/limits.md
hint: a value kind's home must exist; create the empty file, then run `grund fetch <ID>` to write the first snapshot
```

That hint is for a kind whose `fetch` has something to run ([§FS-fetch.2.1](FS-fetch.md#21-a-value-that-names-no-executable)) and whose snapshot
the project reads as Markdown: under version 2, the language that the `file` home's name, or a
`folder` home's `<ID>.md` ([§FS-fetch.5](FS-fetch.md#5-folder-home-write)), resolves to is of class `document`
([§FS-config-v2.schema.sources.extensions](FS-config-v2.md#schemasourcesextensions-schemasourcesextensions-maps-a-suffix-to-a-language)); under version 1, a `file` home named `.md` or a
`folder` home, with `md` in the effective `[scan] extensions` ([§FS-scan.scope.extensions](FS-scan.md#scopeextensions-an-extension-makes-a-file-readable-not-declarable)). A
`folder` home's says `create the empty folder`, and `<ID>` is printed as it stands, since the
configuration loads before the ID is read.

Any other absent home inside the project root gets
`hint: a value kind's home must exist; create the file`, or `create the folder`. One outside
it, by `..` or an absolute path as written, gets
`hint: a value kind's home must be inside the project root; name a path inside it`, whether or
not the kind sets `fetch` ([§FS-config.3.4.9](FS-config.md#349-values--first-class-value-declarations)); version 2 refuses such a place earlier
([§FS-config-v2.schema.places.boundaries](FS-config-v2.md#schemaplacesboundaries-what-a-place-may-name)).

### 4.2. A home read as source is refused

A `file` home the text scan reads as source holds no Markdown declaration, so what `fetch`
wrote there would not be what the scan reads ([§DF-fetch-source-home](../decisions/functional/DF-fetch-source-home.md#df-fetch-source-home-fetch-refuses-a-file-home-the-text-scan-reads-as-source)). It is refused by its
path and kind before the fetcher runs, whatever it holds, and an absent one is not created:

```text
error: kind `<KIND>` cannot fetch into <home>: a file read as source holds no Markdown; move the kind and its declarations to a Markdown file or a folder, or remove its `fetch`
```

Under version 2 that is a home whose name resolves to a language of class `source`
([§FS-config-v2.schema.sources.extensions](FS-config-v2.md#schemasourcesextensions-schemasourcesextensions-maps-a-suffix-to-a-language)), whatever its suffix; under version 1, one not named
`.md` whose extension is in the effective `[scan] extensions` ([§FS-scan.scope.extensions](FS-scan.md#scopeextensions-an-extension-makes-a-file-readable-not-declarable)). The
name alone decides, since a `file` home is a scan root that no `include`, `exclude` or hidden
directory adds or prunes ([§FS-scan.scope.homes](FS-scan.md#scopehomes-every-configured-kind-home-is-scanned), [§FS-scan.scope.roots](FS-scan.md#scoperoots-a-scan-root-outruns-every-rule-about-descent)). A home read as Markdown
is written whatever its name, and one not read at all is [§FS-fetch.4.2.1](FS-fetch.md#421-a-home-the-scan-does-not-read-is-refused)'s.

The refusal is `fetch`'s alone: the configuration loads, and `check`, `list`, `show` and
`refs` keep their verdicts on what the file declares, `check` naming another remedy
([§FS-check.3.1](FS-check.md#31-dangling-citation)).

#### 4.2.1. A home the scan does not read is refused

A `file` home the text scan does not read, such as `fs.dat` or a hidden `.fs.py`
([§FS-scan.scope.hidden](FS-scan.md#scopehidden-a-hidden-file-is-not-read-and-the-rule-is-not-about-descent)), holds nothing any command reads, and no reading says where a
declaration there ends ([§DF-fetch-unread-home](../decisions/functional/DF-fetch-unread-home.md#df-fetch-unread-home-fetch-refuses-a-home-the-scan-does-not-read-before-the-integration-runs)). `fetch` refuses it before the fetcher runs,
the first write included, whatever it holds, and does not create an absent one; `check` stops
offering `run grund fetch` there ([§FS-check.3.1](FS-check.md#31-dangling-citation)):

```text
error: kind `<KIND>` cannot fetch into <home>: <reason>, so the scan would not read what fetch writes; give the file a name a listed document language reads, or remove its `fetch`
```

`<reason>` is the one `check`'s unread warning gives for that name
([§FS-scan.checks.unread-scan-root](FS-scan.md#checksunread-scan-root-a-configured-file-root-is-not-read)), such as
``no language in [schema.sources] languages reads `.dat` ``. Under version 1 the repair says
`a name the scan reads as Markdown` instead.

In a `folder` home the same refusal holds for the `<ID>.md` that [§FS-fetch.5](FS-fetch.md#5-folder-home-write) would create,
when the scan before the fetcher ([§FS-fetch.6.1](FS-fetch.md#61-one-scan-read-before-the-integration-runs-and-installed-against)) finds no file in the home declaring the ID
and does not read that path as Markdown; a declaring file is [§FS-fetch.5](FS-fetch.md#5-folder-home-write)'s to judge. Whether
the walk reads the path is its own read predicate, the one a stub's target asks
([§FS-declarations.stubs.target.3](FS-declarations.md#stubstarget3-a-target-the-scan-does-not-read-declares-nothing)), so an `<ID>.md` that an ignore file
([§FS-scan.scope.ignore-files](FS-scan.md#scopeignore-files-respect_gitignore--the-ignore-files)) or an `exclude` rule skips is refused, present or absent, plain
file or link, while a configured home a rule names is still read. `<reason>` and the repair
are then the rule's:

```text
error: kind `<KIND>` cannot fetch into <home>: <file> ignores it by <pattern>, so the scan would not read what fetch writes; stop ignoring it in <file>, and fetch again
error: kind `<KIND>` cannot fetch into <home>: [schema.sources] exclude matches it by <pattern>, so the scan would not read what fetch writes; remove <pattern> from [schema.sources] exclude, and fetch again
error: kind `<KIND>` cannot fetch into <home>: [scan] exclude names <entry>, so the scan would not read what fetch writes; remove <entry> from [scan] exclude, and fetch again
```

`<file>` is the ignore file as a report path, absolute where it lies outside the loaded root,
and `<pattern>` and `<entry>` stand as written. A link there takes this refusal ahead of
[§FS-fetch.links](FS-fetch.md#links-a-link-at-the-target-is-written-through-when-it-stays-inside-the-config-root).

### 4.3. A value home that cannot be looked up is refused with its reason

A value home whose lookup fails for another reason than absence, such as a directory above it
without search permission, is refused at the same point and config line, since a home `grund`
cannot check is not accepted:

```text
error: <config>:<line>: could not look up value home for kind `<KIND>`: <home>: <reason>
hint: make the home and every directory above it accessible
```

`<reason>` is derived from the I/O error kind as [§FS-check.2.4.1](FS-check.md#241-one-reason-for-one-scan-io-failure) derives a scan failure's,
without an OS error number, such as `permission denied`. The hint is the same for every home,
config version and kind.

## 5. Folder-home write

For `folder = "<home>"`, `fetch` replaces the file the scan reads as Markdown that declares the
requested ID, whatever its name, or creates `<home>/<ID>.md` when nothing declares it
([§FS-fetch.5.5](FS-fetch.md#55-which-file-declares-and-what-idmd-may-hold)); it never replaces a file that declares another ID. The file holds the
accepted output verbatim. More than one existing declaration of the ID is ambiguous and refused:

```text
error: snapshot folder has multiple declarations for requested ID
```

### 5.1 Existing value-source declarations prevent installation

A value source in the home that the selected project's catalog reads as declaring the ID, in
either format, is an existing declaration: `fetch` refuses before writing anything, a Markdown
declaration of the ID beside it notwithstanding, and leaves the source, its other members and
the index as they were ([§FS-fetch.6](FS-fetch.md#6-stability-and-ownership), [§DF-fetch-folder-value-source](../decisions/functional/DF-fetch-folder-value-source.md#df-fetch-folder-value-source-folder-fetch-refuses-an-id-already-declared-by-a-value-source)):

```text
error: cannot fetch CONST-room-limit: value source values/limits.json already declares it; keep the fact in the value source or move it outside the value home before fetching
```

Which sources declare follows the value-source rules, not the scan's filters ([§FS-values.2.2](FS-values.md#22-declarations-from-value-sources)),
and a file the catalog leaves unread declares nothing. A source in the home that cannot be
read or parsed stops the fetch as a discovery failure; one declaring only other IDs, and
findings elsewhere in the project, stop nothing.

### 5.2 A file it does not replace that declares the ID prevents installation

Discovery reads every file the walk yields in the home that the scan does not read as
Markdown, through the scan's own pass in that file's language, as part of the one scan
([§FS-fetch.6.1](FS-fetch.md#61-one-scan-read-before-the-integration-runs-and-installed-against)). A file in which the pass records a declaration of the ID, a stub whatever its
target included, is an existing declaration: creating beside it would duplicate it, and
replacing it would overwrite what surrounds it ([§FS-fetch.6](FS-fetch.md#6-stability-and-ownership), [§DF-fetch-folder-declaring-file](../decisions/functional/DF-fetch-folder-declaring-file.md#df-fetch-folder-declaring-file-a-folder-home-fetch-refuses-an-id-a-file-it-does-not-replace-already-declares)).
`fetch` refuses before writing anything, ahead of judging any Markdown file in the home that
also declares the ID:

```text
error: cannot fetch <ID>: <file> already declares it; keep the fact in that file, or take the declaration out of it and fetch again
```

`<ID>` is the local ID, and of several such files `<file>` is the first in the walk's sorted
order. Whether [§FS-fetch.5](FS-fetch.md#5-folder-home-write) replaces a file or this refuses it is decided by how the scan reads
it, never by its name ([§DF-reading-at-fetch-and-unread-homes.2.1](../decisions/functional/DF-reading-at-fetch-and-unread-homes.md#21-folder-fetch-follows-the-reading)). A file that declares only
other IDs, cites the ID or holds its text where nothing declares, a file the walk does not
yield, and any finding stop nothing. A file the pass cannot read or decode is a discovery
failure, reported as [§FS-fetch.5.4](FS-fetch.md#54-a-home-file-that-cannot-be-decoded-is-named) says.

### 5.3 A declaration outside the home prevents installation

Whether the ID is declared is the selected project's scan's answer, for a `file` home as for a
`folder` home ([§DF-fetch-admission](../decisions/functional/DF-fetch-admission.md#df-fetch-admission-fetch-installs-only-what-the-next-scan-reads-back-as-exactly-what-was-asked-for)). A declaration it records in a file outside the home is an
existing one that `fetch` may neither duplicate nor overwrite ([§FS-fetch.6](FS-fetch.md#6-stability-and-ownership),
[§REQ-no-data-loss.2](../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own), [§DF-fetch-declaration-outside-home](../decisions/functional/DF-fetch-declaration-outside-home.md#df-fetch-declaration-outside-home-a-folder-home-fetch-refuses-an-id-declared-outside-the-home)), so it refuses with [§FS-fetch.5.2](FS-fetch.md#52-a-file-it-does-not-replace-that-declares-the-id-prevents-installation)'s
line, from the one scan before the fetcher runs ([§FS-fetch.6.1](FS-fetch.md#61-one-scan-read-before-the-integration-runs-and-installed-against)): after the refusals that need
no scan and those of [§FS-fetch.4](FS-fetch.md#4-file-home-write) and [§FS-fetch.4.2.1](FS-fetch.md#421-a-home-the-scan-does-not-read-is-refused), ahead of every judgement of the home's
own contents, and with an absent home left absent.

Stubs count as every command pairs them ([§FS-declarations.stubs.pairing.1](FS-declarations.md#stubspairing1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned)): a healthy stub
counts at its target, whether or not the scan reaches it, and is named there
([§FS-declarations.stubs.pairing](FS-declarations.md#stubspairing-how-a-stub-pairs-with-what-it-points-at)), while a broken one pairs with nothing and declares where it
stands. Of several declarations outside, the first in `path:line` order is named.

What the scan does not read refuses nothing, and in a workspace only the selected project's
scan counts ([§FS-fetch.1](FS-fetch.md#1-input-and-project-selection)). No finding stops the fetch, and neither does a file outside the
home that the scan could not read or decode, a directory it could not list, the config root
included, or a link it could not resolve ([§FS-check.2.4](FS-check.md#24-an-incomplete-run),
[§DF-fetch-declaration-outside-home.2](../decisions/functional/DF-fetch-declaration-outside-home.md#2-decision)); a declaration another file holds still refuses. Inside
the home such a file, or a scan that could not run, stops the fetch as a discovery failure.

### 5.4 A home file that cannot be decoded is named

A home file that cannot be decoded as UTF-8 stops the fetch, whether it is a Markdown `file` home or
any file in a `folder` home, the requested ID's own or another, since it may hold the
declaration. The refusal is the line `grund check` prints for that file ([§FS-check.2.4](FS-check.md#24-an-incomplete-run),
[§FS-errors.2.2.5](FS-errors.md#225-one-scan-failure-one-rendering)), so the reader is told which file to repair. Of several in a folder home it
names the first in the walk's sorted order, and a file [§FS-fetch.5.2](FS-fetch.md#52-a-file-it-does-not-replace-that-declares-the-id-prevents-installation)'s pass cannot read is
reported the same way.

### 5.5. Which file declares, and what `<ID>.md` may hold

The declaring file is the one the scan reads as Markdown, whatever its name, in which it reads
the ID declared, at whatever depth and under whatever heading ([§FS-fetch.3](FS-fetch.md#3-accepted-declaration)). Its whole content
is replaced, so it may hold nothing but the range the scan records for that declaration
([§FS-fetch.4](FS-fetch.md#4-file-home-write)) and blank lines; a group heading, prose or another declaration beside it is
refused:

```text
error: snapshot file <path> contains the requested ID beside other content
```

Inside the home a stub and the target it points to are two declarations, so a file holding a
stub whose target in the home declares the ID is refused as ambiguous ([§FS-fetch.5](FS-fetch.md#5-folder-home-write)).

Where nothing declares the ID and `<home>/<ID>.md` exists, a file holding nothing but
whitespace and line endings is filled, since nothing in it is lost
([§DF-fetch-refusals-follow-the-scan](../decisions/functional/DF-fetch-refusals-follow-the-scan.md#df-fetch-refusals-follow-the-scan-four-fetch-refusals-say-what-the-scan-reads-corrected-in-place-and-an-empty-idmd-is-filled)). Any other is refused by what the scan reads there, no
ID or another one:

```text
error: snapshot file <path> already exists and the scan reads no ID in it; move or rename it, and fetch again
error: snapshot file <path> already exists for a different ID
```

The install is atomic, and a missing folder is created only by a successful one, except a
value kind's or a `value_chapter` kind's, which must exist ([§FS-fetch.4.1](FS-fetch.md#41-an-absent-value-home-is-refused-with-the-way-through)).

## 6. Stability and ownership

Fetching unchanged output again leaves the same files and bytes, and output whose second fetch
would change them is refused ([§FS-fetch.admission.5](FS-fetch.md#admission5-fetching-the-same-output-again-leaves-the-same-bytes)). `fetch` owns only the requested
declaration in the configured home ([§REQ-no-data-loss.2](../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own)): it never prunes another snapshot or
rewrites a citation, so repeated and independent fetches give reviewable diffs. One
`fetch <ID>` fetches one ID; there is no `--refresh`, generated timestamp, unused-snapshot
exemption or workspace-wide snapshot ownership.

### 6.1. One scan, read before the integration runs, and installed against

`fetch` runs the selected project's scan once, after every refusal that needs no scan and
before the fetcher runs, and no second discovery scan after it: admission reads only the
output and the candidate ([§FS-fetch.admission](FS-fetch.md#admission-admission-the-next-scan-reads-back-exactly-what-was-asked-for)). That one reading answers every question about
the tree as it stood: where the ID is declared, whether the target is read and what admission
compares against. The target is replaced only if it still holds the bytes the scan read, and
created only if it is still absent; one edited or created while the fetcher ran is left as it
is, and the fetch is refused:

```text
error: <path> changed while the integration ran; fetch again
```

## 7. Output and exits

A successful `grund fetch <ID>` writes nothing to stdout or stderr and exits `0`, and an ID
that does not parse exits `1` ([§FS-fetch.1](FS-fetch.md#1-input-and-project-selection)). Every other refusal of `grund fetch <ID>` is a CLI-level error
on stderr ([§FS-errors.2.2](FS-errors.md#22-cli-level-message)), exit `2`, with stdout empty and the tree byte-identical. Every
path a refusal names is spelled from the report base ([§FS-errors.terms](FS-errors.md#terms-terms)), so a tree's refusal
reads the same from every checkout ([§FS-errors.4](FS-errors.md#4-determinism)). The exits and output of `--remote` are
[§FS-fetch.remote](FS-fetch.md#remote-remote-projections)'s.

## admission: Admission: the next scan reads back exactly what was asked for

`fetch` builds the candidate, the home with the output in place of the owned range
([§FS-fetch.4](FS-fetch.md#4-file-home-write)) or the output alone in a `folder` home ([§FS-fetch.5](FS-fetch.md#5-folder-home-write)), has the selected project's
scan read it, and installs it only if that reading is exactly what was asked for
([§DF-fetch-admission](../decisions/functional/DF-fetch-admission.md#df-fetch-admission-fetch-installs-only-what-the-next-scan-reads-back-as-exactly-what-was-asked-for)). Each clause below is judged from the output alone, as Markdown; from the
tree as the scan read it before the fetcher ran ([§FS-fetch.6.1](FS-fetch.md#61-one-scan-read-before-the-integration-runs-and-installed-against)); or from the home with the
candidate in place and no other file read again. The clauses are judged in order, and only the
first failure is reported. Admission has no reading of its own, since what declares, where a body
ends and what is malformed are the scan's and `check`'s answers, and it refuses output, never
rewriting it.

### admission.1: The requested declaration is there once, and its body takes in all of the output

Read from the output alone, as Markdown, as the scan reads every target `fetch` writes, since
a home it would not read so is refused before the fetcher runs ([§FS-fetch.4.2.1](FS-fetch.md#421-a-home-the-scan-does-not-read-is-refused),
[§FS-fetch.5.2](FS-fetch.md#52-a-file-it-does-not-replace-that-declares-the-id-prevents-installation), [§DF-reading-at-fetch-and-unread-homes.2.2](../decisions/functional/DF-reading-at-fetch-and-unread-homes.md#22-fetch-reads-its-output-as-markdown)). The first line declares the ID at
the home's level ([§FS-fetch.3](FS-fetch.md#3-accepted-declaration)), no later heading declares inside it ([§FS-fetch.3.1](FS-fetch.md#31-a-nested-declaration-is-refused)), and none
ends it before the output does. A heading that ends it, a bare `##` or `#` or any heading at
or above its level that declares nothing, would leave the text after it belonging to nothing,
and is refused:

```text
error: fetch output line <n>: heading `<heading>` ends <ID> before the output does; fence it, make it a bold label, or remove it
```

`<n>` and `<heading>` are as in [§FS-fetch.3.2](FS-fetch.md#32-a-subsection-is-a-heading-the-next-scan-reads-as-a-section). A heading that declares a second ID at the
declaration's own level takes [§FS-fetch.3](FS-fetch.md#3-accepted-declaration)'s fourth line, and so do leading blank lines,
which are content before the declaration and never trimmed. Output with no line but blank ones
takes its fifth.

### admission.2: Every other declaration in the home reads as it did before

Read from the tree before and the home after. With the replaced range cut from both readings,
every other declaration the scan reads in the home is in both, with the same ID, title, level,
sections and body text. One read inside the replaced range would be deleted and takes
[§FS-fetch.3.1](FS-fetch.md#31-a-nested-declaration-is-refused)'s line; any other that would appear, disappear or change is refused, named where
its heading stood before:

```text
error: cannot fetch <ID>: writing the output would change how <file>:<line> reads <OTHER>; change the output or <OTHER> so the two stand apart, and fetch again
```

### admission.3: The structural rules `check` applies report nothing new

`check`'s node-local rules run on the tree before and on the home after ([§FS-check](FS-check.md#fs-check-grund-validates-every-citation-in-a-repo)):
duplicate, misplaced and near-miss declarations, broken stubs, heading levels, duplicate and
orphan sections, unmarked headings, inline citation style, oversized leads, fields, conflict
markers and glob citations, and the scan's own invalid-value records. Its relational rules,
citations, locks, chapter rules and grounding, do not run, since a fetched body's citations are
its reader's to settle ([§DF-fetch-admission.2](../decisions/functional/DF-fetch-admission.md#2-decision)). A finding of any channel, warnings included,
that the after reading holds and the before reading does not is refused, findings compared by
code and message with each line below the replaced range mapped through the splice. One inside
the output is refused as

```text
error: fetch output line <n>: <message>
```

and one elsewhere in the home as `error: cannot fetch <ID>: <file>:<line>: <message>`, where
`<message>` is `check`'s, except that an unmarked heading takes [§FS-fetch.3.2](FS-fetch.md#32-a-subsection-is-a-heading-the-next-scan-reads-as-a-section)'s line.

### admission.4: The output closes every fence and raw-text block it opens

Read from the output alone. A fenced block or an HTML raw-text block (`<pre>`, `<script>`,
`<style>`, `<textarea>`) the output opens and does not close would swallow every line after it
in the home, so it is refused, naming the line that opens it:

```text
error: fetch output line <n> opens a <block> it does not close; close it, and fetch again
```

`<block>` is `fenced block` or `raw-text block`.

### admission.5: Fetching the same output again leaves the same bytes

Read from the home after. The output spliced again into the candidate, at the range the
candidate's own reading records for the requested declaration, must reproduce the candidate
byte for byte. Output that would grow, shrink or move on its next fetch, such as a final bare
carriage return beside a sibling, is refused, naming the first line that would change:

```text
error: cannot fetch <ID>: a second fetch of this output would change <file> at line <n> again, so the first is refused
```

## links: A link at the target is written through when it stays inside the config root

The scan reads a linked file at the link's path ([§FS-scan.scope.links](FS-scan.md#scopelinks-a-symlink-in-the-tree-is-followed)), so a `file` home, a
declaring file or an `<ID>.md` that is a symbolic link is decided by the walk's own record of
it, the one `fmt --write` asks ([§FS-fmt.2.3.2](FS-fmt.md#232-a-link-that-leaves-the-config-root-is-not-written-through)), and no link is resolved a second time. A link
whose target the walk puts inside the selected project's config root is written through: the
write is atomic at the target, the link stays, and admission judges the target. Any other link
is refused by its path, link and target byte-identical, even when the target already holds the
output:

```text
error: cannot fetch <ID> into <path>: the symlink target is outside the config root; replace the link with the file it points to, and fetch again
error: cannot fetch <ID> into <path>: the symlink target does not exist; create the target, or remove the link, and fetch again
error: cannot fetch <ID> into <path>: the scan does not read the symlink as a file; remove the link, or let the scan read it, and fetch again
```

The last is for a link the walk does not read as a file, such as one to a directory, wherever
it leads. An `<ID>.md` link that an ignore file or an `exclude` rule skips, or a link in a home
the scan does not read, takes [§FS-fetch.4.2.1](FS-fetch.md#421-a-home-the-scan-does-not-read-is-refused)'s line instead. Where `fmt` warns about an
outward link and goes on, `fetch`, which writes one file, refuses.

## remote: Remote projections

This chapter is specified ahead of the releases that build it, as all of [§FS-remote-projects](FS-remote-projects.md#fs-remote-projects-a-project-cites-another-repositorys-declarations-from-a-committed-pinned-projection)
is; until they land, `--remote` is refused as an unknown flag with exit 2.

### remote.selector: `--remote <alias>` projects one declared remote

`grund fetch --remote <alias>` reads the repository that the config root's
`[workspace.remotes.<alias>]` table declares ([§FS-remote-projects.declaration](FS-remote-projects.md#declaration-declaration-and-registration)). It
resolves the declared `ref` at the declared `source` to a full commit with the user's `git`,
reads that commit's complete tracked tree from git objects without checking it out, validates
it, writes it to `.grund/remotes/<alias>/` ([§FS-remote-projects.projection](FS-remote-projects.md#projection-the-projection)), and rewrites that
remote's lock entry ([§FS-remote-projects.lock](FS-remote-projects.md#lock-the-lock)), one reviewable diff. It runs nothing the
remote supplies: no hook, checkout filter, script, checker or `fetch` integration. An alias
the config root does not declare is refused with exit 2. `fetch --remote` needs `git` on
`PATH`; no other command does.

### remote.install: Validate the whole projection, then replace the pair

The new projection is validated completely, entry types and byte protection included
([§FS-remote-projects.bytes](FS-remote-projects.md#bytes-exact-bytes)), before anything is replaced. Only then are the projection and its
lock entry replaced together, and the fetch exits 0; a fetch refused at any step exits 2 and
leaves both exactly as they were. Fetch owns only `.grund/remotes/<alias>/` and that remote's
lock entry ([§FS-remote-projects.lock](FS-remote-projects.md#lock-the-lock)).

### remote.findings: Refuse only what this grund cannot load

Fetch refuses to lock a commit whose projected config or required content this grund cannot
load, because installing it would only defer the failure to the next `check`. It locks a
commit whose own content has findings and reports them on stdout, never failing the pin on
them, because gating a pin on upstream housekeeping is the coupling remotes exist to avoid.

### remote.commit: Paths the consumer's git would not commit are named

After installing, fetch asks the consumer's git whether each projected path would be
committed. When any would not, because the remote's own `.gitignore` or a pattern of the
consumer's keeps it from a plain `git add`, fetch prints those paths and the exact
`git add --force` command that commits them, and it never touches the index
([§DF-remote-projects.2.2](../decisions/functional/DF-remote-projects.md#22-the-choices-the-agora-left-open)). A path left uncommitted makes every fresh clone report
`remote-modified` ([§FS-remote-projects.checks.remote-modified](FS-remote-projects.md#checksremote-modified-the-projection-is-not-the-locked-bytes)).

### remote.all: `--remote --all` fetches every declared remote

`grund fetch --remote --all` fetches every remote this config root declares, each as
`--remote <alias>` would. It never fetches a remote declared inside a projection: those are
owned by the projection's own lock and move only when the remote that holds them is
refetched.

### remote.check: `--check` is a read-only drift signal

`grund fetch --remote <alias> --check`, or `--remote --all --check`, resolves each selected
declared ref and compares it with the locked commit, writing nothing. It reports every
difference, a rewind to an older commit included. It exits 0 when every selected pin is
current, 1 on any difference, and 2 when a ref cannot be obtained ([§DF-remote-projects.2.2](../decisions/functional/DF-remote-projects.md#22-the-choices-the-agora-left-open)),
never reporting a pin it could not compare as current. `--check` never runs inside
`grund check`.

### remote.from: `--from <path>` projects a committed revision of a local clone

`grund fetch --remote <alias> --from <path>` reads the declared `ref` from the local clone at
`<path>` instead of from `source`. It refuses a clone whose working tree is dirty, and it reads
committed objects only. The lock records the declared portable `source`, never `<path>`
([§DF-remote-projects.2.3](../decisions/functional/DF-remote-projects.md#23-what-stays-refused)).
