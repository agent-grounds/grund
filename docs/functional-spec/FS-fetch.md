# FS-fetch: grund materializes one external fact snapshot

`grund fetch <ID>` deliberately asks the selected kind's configured integration for one
complete Markdown declaration and stores it in that kind's configured home, and
`grund fetch --remote <alias>`, specified ahead of its release, projects a declared remote repository
([§FS-fetch.remote](FS-fetch.md#remote-remote-projections)). Together they are the only external-materialization surface; all later scanning and resolution use the saved
declaration under [§REQ-runs-offline](../requirements/REQ-runs-offline.md#req-runs-offline-verification-never-depends-on-an-external-service).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, home, body, section, catalog),
[§FS-terms.terms.2](FS-terms.md#terms2-citations) (citation), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, config root, workspace, member, alias),
and [§FS-terms.terms.7](FS-terms.md#terms7-values-and-integrations) (fetcher, snapshot).

## 1. Input and project selection

Without `--remote`, the command accepts exactly one local or workspace-qualified ID and no
section; `--remote` takes the place of the ID ([§FS-fetch.remote.selector](FS-fetch.md#remoteselector---remote-alias-projects-one-declared-remote)), and an
invocation naming both is a usage error with exit 2. The ID form uses
the same target-project selection as the ID query ([§FS-workspace.8.1](FS-workspace.md#81-grund-aliasid)) but parses
by the kind's effective grammar only, without the catalog-backed off-grammar spellings
the query also reads ([§FS-config.3.2](FS-config.md#32-id--id-grammar)). `grund fetch alias/TICKET-1234` loads that member's config and home, renders the qualified form in user messages, and passes only `TICKET-1234` to the integration. What that selection answers for an alias path through an import is [§FS-imports.writes.answer](FS-imports.md#writesanswer-where-a-qualified-id-is-fetched-has-one-answer).
A malformed alias path is refused as [§FS-workspace.8.11](FS-workspace.md#811-a-malformed-alias-path-names-the-offending-segment) says, naming the offending segment, with exit 2; `grund fetch Bad/TICKET-1234` prints what `grund show Bad/TICKET-1234` prints.

An ID that cannot be parsed is a bare query failure ([§FS-errors.2.3](FS-errors.md#23-bare-query-failure)) and exits 1.
A parsed ID whose kind has no `fetch`, or whose selected project cannot supply exactly
one configured file or folder home, is an operational refusal on stderr and exits 2.

## 2. Integration invocation

`[[kinds]].fetch` names one executable ([§FS-fetch.2.1](FS-fetch.md#21-a-value-that-names-no-executable)). A relative value is resolved from the selected
project's config root. Grund invokes that path directly, never through a shell, appends
the local unqualified ID as the integration's sole argument, inherits the user's
environment, and provides no implicit input on stdin. Deliberately running `fetch` is
sufficient authorization to execute repository-controlled configuration.

Fetcher stdout is snapshot data and is never copied to grund's stdout. Exit 0 advances
to validation. A spawn failure, signal, or non-zero integration exit is an operational
error on stderr and exit 2. The message identifies the configured integration and ID;
the tree remains byte-identical.

### 2.1 A value that names no executable

The key names an executable, under either config version. A value that is empty or holds only whitespace names none, and one test of the string says so wherever it is asked. A value with any other character names what it spells: `" scripts/fetch "` is a real name with stray spaces, is taken literally, and fails as a spawn failure of [§FS-fetch.2](FS-fetch.md#2-integration-invocation) if nothing is there.

Version 2 refuses such a key when the config loads ([§FS-config-v2.checks.config-fetch-names-nothing](FS-config-v2.md#checksconfig-fetch-names-nothing-a-rows-fetch-names-no-executable)), so under version 2 `grund fetch` meets that refusal and reaches nothing below. A version 1 row keeps loading the key ([§FS-config.5.2](FS-config.md#52-every-older-version-keeps-its-meaning)).

`grund fetch <ID>` on such a version 1 key runs nothing. It exits `2`, the tree stays byte-identical, and the message is the spawn failure's, with what is wrong and the key's line where the operating system's reason stood:

```text
error: cannot run fetch integration `` for AR-001-power: grund.toml:6: [[kinds]] `AR` fetch names no executable; name the program, or if this kind is not fetched remove the key and any `resolve`, and its missing IDs are then errors
```

The path is the config file that holds the key, as a report path ([§FS-errors.4](FS-errors.md#4-determinism)), so a workspace member's names the member's own file. The value is quoted as written.

The repair is one sentence, printed in the same words wherever the key is reported: here, by the version 2 load refusal in that version's spelling ([§FS-config-v2.checks.config-fetch-names-nothing](FS-config-v2.md#checksconfig-fetch-names-nothing-a-rows-fetch-names-no-executable)), by migration ([§FS-config-migrate.3.1.10](FS-config-migrate.md#3110-a-fetch-that-names-no-executable)), and by `grund check` in place of this command ([§FS-check.3.1.3.2](FS-check.md#3132-a-key-fetch-has-nothing-to-run-for)). It holds for every tree, so it does not ask which one it is in. A kind with a `[rules.resolution] <KIND>` (version 1: `resolve`) on it can load only while the key is there ([§FS-config-v2.rules.resolution](FS-config-v2.md#rulesresolution-rulesresolution), [§FS-config.3.4.10.2](FS-config.md#34102-fetch-and-resolve)), so the setting goes with the key, and with both gone a citation of an ID the kind does not hold is an error.

The test is of the string alone. Under version 2 a whitespace-only value is refused even where a file of that very name exists at the config root. Under version 1 such a value, one that names a file literally called `" "`, is still run by `grund fetch`, as it was before this rule was written, and for good ([§FS-config.5.2](FS-config.md#52-every-older-version-keeps-its-meaning)). This is a question `grund fetch` asks of the disk, after the test of the string says the value names nothing, and `grund check` asks it the same way before it names this command ([§FS-check.3.1.3.2](FS-check.md#3132-a-key-fetch-has-nothing-to-run-for)), as the hint of an absent value home does ([§FS-fetch.4.1](FS-fetch.md#41-an-absent-value-home-is-refused-with-the-way-through)). An empty value can never name a file.

The check comes immediately before the spawn would, so every refusal `grund fetch` makes before running the integration stands in front of it, as they stand in front of a spawn failure.

## 3. Accepted declaration

The complete stdout must be UTF-8 and hold one Markdown declaration of the requested local ID,
which begins on its first line:

- a `file` home accepts one H2 declaration (`## TICKET-1234: Title`);
- a `folder` home accepts one H1 declaration (`# TICKET-1234: Title`).

Whether that is so is the selected project's own scan's answer, never a second reading of
`fetch`'s: the output is accepted when admission says so ([[§FS-fetch.admission](FS-fetch.md#admission-admission-the-next-scan-reads-back-exactly-what-was-asked-for)]). A heading is the
requested declaration where the next scan reads it as declaring that ID, so `## TICKET-1234`,
`## TICKET-1234:` and `## TICKET-1234 x: y` are accepted wherever the scan reads them as
`TICKET-1234`, and a heading indented by up to three spaces is a heading as it is on the next
scan. The declaration may contain ordinary prose, fenced blocks, and any subsection the next scan
reads as a section of it at the level `check` holds it to ([[§FS-fetch.3.2](FS-fetch.md#32-a-subsection-is-a-heading-the-next-scan-reads-as-a-section)]); `fetch` keeps no
depth limit of its own, so a value chapter and its value roots are written
([[§FS-config.3.4.13](FS-config.md#3413-value_chapter--the-chapter-whose-named-children-are-values)]).

Output that declares no ID or another ID first, or whose first line sits at the wrong level for
the home, keeps the words it was refused with before admission:

```text
error: fetch output must begin with one Markdown declaration
error: fetch output declares `<OTHER>` instead of requested `<ID>`
error: fetch output heading depth <n> is invalid for a depth-<d> declaration
error: fetch output must contain exactly one declaration and no leading content
error: fetch output must contain exactly one Markdown declaration
```

The third is for the declaration's own heading at the wrong level for the home, and the fourth
for content before it. A section at the wrong level is `check`'s finding, refused under
[[§FS-fetch.admission.3](FS-fetch.md#admission3-the-structural-rules-check-applies-report-nothing-new)] with `check`'s words. Invalid UTF-8 and every other refusal below exit 2
with empty stdout and the tree byte-identical.

The whole output is validated before any filesystem mutation. Accepted bytes are
preserved verbatim; grund does not add timestamps, normalize whitespace, run `fmt`, or
sanitize marked citations. A marked citation in the body is therefore a live citation
on the next ordinary scan ([[§FS-check.1.1](FS-check.md#11-recognized-citations)]).

### 3.1 A nested declaration is refused

"Exactly one declaration" counts every heading the next scan would read as a
declaration, not only those at the native depth. A heading inside the fetched
declaration whose text declares an ID of any configured kind, under the project's ID
grammar, is refused on stderr with exit 2 and the tree byte-identical, whatever its
depth below the native one and whichever ID it carries: an unrequested ID, the
requested ID a second time, an ID the snapshot home already declares, or another
kind's ID. The refusal names the nested ID. It holds under config version 1 and 2 and
for `file` and `folder` homes alike, because `fetch` owns only the requested
declaration and one `fetch <ID>` fetches one ID ([[§FS-fetch.6](FS-fetch.md#6-stability-and-ownership)]). Which headings declare is
the scan's reading of the output alone ([[§FS-fetch.admission.1](FS-fetch.md#admission1-the-requested-declaration-is-there-once-and-its-body-takes-in-all-of-the-output)]).

A subsection heading ([[§FS-fetch.3.2](FS-fetch.md#32-a-subsection-is-a-heading-the-next-scan-reads-as-a-section)]), which carries a numbered or named section path rather than an
ID, stays accepted, as does any heading inside a fenced block, which declares nothing.
The rule binds only what `fetch` accepts: a nested declaration written by hand into a
scanned file stays legal. What `fetch` does not do is delete one. Where the home holds a
declaration the scan reads inside the range the requested one owns, at any depth, under any
indent the scan reads as a heading, replacing that range would delete it, so the fetch is
refused ([[§FS-fetch.admission.2](FS-fetch.md#admission2-every-other-declaration-in-the-home-reads-as-it-did-before)]):

```text
error: cannot fetch <ID>: <file>:<line> declares <OTHER> inside <ID>, and replacing <ID> would delete it; move <OTHER> out of <ID>, and fetch again
```

`<file>:<line>` is that declaration's heading, spelled from the report base ([[§FS-errors.terms](FS-errors.md#terms-terms)]).

### 3.2. A subsection is a heading the next scan reads as a section

A subsection is a heading inside the declaration that the next ordinary scan reads as a
section of it, so the one rule that decides it is the scanner's
([[§FS-declarations.checks.unmarked-heading](FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading)]): a numeric section path (`### 1. Notes`),
a named one where `[id] named_sections` enables names (`### notes: Notes`), or a
relative heading ([[§FS-declarations.section.relative](FS-declarations.md#sectionrelative-a-heading-may-write-only-the-component-it-adds)]) the same configuration admits
(`### .notes: Notes`). Whether a heading qualifies is read with the selected project's
configuration, never by matching its text again, so with `named_sections` unset
`### notes: Notes` is not a subsection while `### 1. Notes` still is. Its level is held where
`check` holds it ([[§FS-declarations.checks.section-heading-level](FS-declarations.md#checkssection-heading-level-section-heading-level-mismatch)]): a `#### 1.1. Detail` under
`### 1. Notes` is a subsection, and `#### 1. Notes` directly under an H2 is refused with
`check`'s line under [[§FS-fetch.admission.3](FS-fetch.md#admission3-the-structural-rules-check-applies-report-nothing-new)].

Any other heading inside the declaration — a prose heading (`### Other`), an ID of a kind
the project does not configure (`### XX-002-other: Other`), or an escaped one
(`### <§>AR-002-other: Other`) — is a malformed heading: the fetch is refused on stderr
with exit 2 and the tree stays byte-identical, rather than accepted and turned red by
the next `grund check`. The refusal names the heading and its line in the output and
says how to keep it:

```text
error: fetch output line <n>: heading `<heading>` is not a section of `<ID>`; number it, fence it, or make it a bold label
```

`<n>` is the heading's 1-based line in the output and `<heading>` that line as written,
trailing whitespace removed. This line stands in place of `check`'s own for the same finding,
because `check`'s suggests declaring an ID there, which [[§FS-fetch.3.1](FS-fetch.md#31-a-nested-declaration-is-refused)] refuses. A `file`
home's H3 and a `folder` home's H2 are held alike, and so is a heading at any depth below them.
A heading inside a fenced block is content and a bold label is no heading, so both stay
accepted, as they are on the next scan.

## 4. File-home write

For `file = "<home>"`, grund replaces only the declaration whose ID exactly matches
the requested ID. If it is absent, grund inserts the declaration among sibling H2
declarations in ID order. The file's preamble, other declarations, line endings, and
all bytes outside the owned declaration remain unchanged.

The owned declaration is the one the selected project's scan reads in the home, wherever it
stands and at whatever heading level, under whatever heading the scan reads as declaring it
([[§FS-fetch.3](FS-fetch.md#3-accepted-declaration)]), and what it owns is the range the scan records for it: from its heading line to
its body's last non-blank line ([[§FS-declarations.body](FS-declarations.md#body-where-a-declarations-body-ends)]). The blank lines after that range stay
as the separator, except where the output itself ends in blank lines: those then take the
separator's place rather than adding to it, so a refetch of the same output leaves the same
bytes ([[§FS-fetch.admission.5](FS-fetch.md#admission5-fetching-the-same-output-again-leaves-the-same-bytes)]). A heading shallower than the declaration's, a sibling under an indent the scan
reads as a heading, and a heading the scan reads as near-missing a declaration therefore end
the range where the scan ends it, and are kept byte for byte. Where the scan reads the
requested declaration inside another declaration it is replaced there, and admission refuses
the write where the output standing there would change how the file reads
([[§FS-fetch.admission.2](FS-fetch.md#admission2-every-other-declaration-in-the-home-reads-as-it-did-before)]).

A `file`-home fetch of an ID the scan reads as declared in a file other than the home is
refused with [[§FS-fetch.5.3](FS-fetch.md#53-a-declaration-outside-the-home-prevents-installation)]'s line. Neither inserting beside that declaration nor replacing it
is the requested declaration's to do; [[§DF-fetch-admission](../decisions/functional/DF-fetch-admission.md#df-fetch-admission-fetch-installs-only-what-the-next-scan-reads-back-as-exactly-what-was-asked-for)] records why the verdict moves.

The final file is installed atomically only after the complete result is available. If
the home cannot be read, parsed unambiguously, or atomically replaced, the command emits
an operational error, exits 2, and leaves it byte-identical.

A home that is a value source ([§FS-values.terms](FS-values.md#terms-terms)), in either format, holds no Markdown
declaration, so it is refused the same way. The refusal is decided by the home's path and
its kind, not by what the file holds, and comes before the integration runs. A value kind
whose `file` home is a Markdown file has no value source there, and takes the write above.

An absent home file is not a failure: grund reads it as an empty file and creates it
in the successful final installation, holding the accepted declaration alone. That
holds for every kind but a value kind and a kind that sets `value_chapter`. Their homes
must already exist ([§FS-config.3.4.9](FS-config.md#349-values--first-class-value-declarations), [§FS-config.3.4.13](FS-config.md#3413-value_chapter--the-chapter-whose-named-children-are-values)), so an absent one is refused when
the configuration loads, before any integration runs, and the refusal names the way
through ([§FS-fetch.4.1](FS-fetch.md#41-an-absent-value-home-is-refused-with-the-way-through)). Nor does it hold for a home the text scan reads as source, which is
refused whether or not it exists ([§FS-fetch.4.2](FS-fetch.md#42-a-home-read-as-source-is-refused)).

### 4.1. An absent value home is refused with the way through

A home is absent when looking it up fails because a path component is not there; a
lookup that fails for any other reason is not an absent home, and
[§FS-fetch.4.3](FS-fetch.md#43-a-value-home-that-cannot-be-looked-up-is-refused-with-its-reason) refuses it instead. The refusal ``<config>:<line>: value home for kind `<KIND>` does not exist: <home>``
keeps that text as its first line and is followed by one repair-hint line. A kind that
sets `value_chapter` ([§FS-config.3.4.13](FS-config.md#3413-value_chapter--the-chapter-whose-named-children-are-values)) reaches the same refusal and the same hint as a
value kind.

For a kind whose `fetch` has something to run ([§FS-fetch.2.1](FS-fetch.md#21-a-value-that-names-no-executable)), with a home whose snapshot the
project reads as Markdown, the hint names the one step that unblocks the first snapshot: create the empty file or the
empty folder, then run `grund fetch <ID>`. In a version-2 file the language decides: the
one a name resolves to ([§FS-config-v2.schema.sources.extensions](FS-config-v2.md#schemasourcesextensions-schemasourcesextensions-maps-a-suffix-to-a-language)) is of class `document`,
where the name is the `file` home's own or, for a `folder` home, the `<ID>.md` fetch writes
there ([§FS-fetch.5](FS-fetch.md#5-folder-home-write)). A version-1 file has no language to ask, so it is a `file` home named
`.md` or a `folder` home, with `md` in its effective `[scan] extensions`
([§FS-scan.scope.extensions](FS-scan.md#scopeextensions-an-extension-makes-a-file-readable-not-declarable)). For a file home it reads:

```text
hint: a value kind's home must exist; create the empty file, then run `grund fetch <ID>` to write the first snapshot
```

and for a folder home `create the empty folder` takes the place of `create the empty
file`. `<ID>` is written as it stands, because the configuration is loaded before the
requested ID is read.

Every other absent home inside the project root gets the shorter hint: a kind without
`fetch` or whose key has nothing to run, neither of which is ever told to run `grund fetch`,
and a home whose snapshot the project
does not read as Markdown, because fetch writes Markdown and so is not the way through
there: a value source's reader refuses it, [§FS-fetch.4.2](FS-fetch.md#42-a-home-read-as-source-is-refused) refuses a home read as source, and
under a name no language or `[scan] extensions` list reads nothing ever sees it. It does
not say `empty`, because an empty
`.json` is itself invalid. For a file home it reads:

```text
hint: a value kind's home must exist; create the file
```

and for a folder home `create the folder` takes the place of `create the file`.

A home outside the project root, written with `..` or as an absolute path, gets a third
hint whether or not the kind sets `fetch`, because creating it only reaches the refusal
that a value home must lie inside the root ([§FS-config.3.4.9](FS-config.md#349-values--first-class-value-declarations)):

```text
hint: a value kind's home must be inside the project root; name a path inside it
```

Outside is read from the path as written, so a dangling symlink inside the root takes the
hint of a home inside it. A version-2 file does not reach this hint, because it refuses
such a place first ([§FS-config-v2.schema.places.boundaries.1](FS-config-v2.md#schemaplacesboundaries1-normalized-before-compared)).

The hint changes no verdict: the exit stays 2, and a caller matching the first line
still matches it. `grund check` and `grund list`, which load the same configuration,
print the same two lines.

### 4.2. A home read as source is refused

A `file` home the text scan reads as source holds no Markdown declaration either. `fetch`
validates the integration's output as Markdown and finds the ID by a heading, and the scan
reads that file through its language's comments: a heading introduces nothing there, and what
lies under it counts only as far as it is a comment that language reads. One file would be read
two ways, and `fetch` would report success whatever the scan made of what it wrote. So `fetch`
refuses it, with exit 2, empty stdout and the whole tree byte-identical, on stderr:

```text
error: kind `<KIND>` cannot fetch into <home>: a file read as source holds no Markdown; move the kind and its declarations to a Markdown file or a folder, or remove its `fetch`
```

`<home>` is spelled from the report base ([§FS-errors.terms](FS-errors.md#terms-terms)).

The refusal is decided by the home's path and its kind, not by what the file holds or what the
integration would print, and comes before the integration runs, as the value-source refusal
does. For a `folder` home's `<ID>.md` ([[§FS-fetch.4.2.1](FS-fetch.md#421-a-home-the-scan-does-not-read-is-refused)]) the path and the kind decide together
with the one scan `fetch` runs before the integration ([[§FS-fetch.6.1](FS-fetch.md#61-one-scan-read-before-the-integration-runs-and-installed-against)]), which says whether a
file in the home already declares the ID. A home that declares
the requested ID, one that declares only other IDs, one that declares nothing and an absent
one are all refused, and an absent one is not created.

In a version-2 file a home is read as source when the language its name resolves to
([§FS-config-v2.schema.sources.extensions](FS-config-v2.md#schemasourcesextensions-schemasourcesextensions-maps-a-suffix-to-a-language)) is of class `source`, whatever the suffix: `fs.py`,
and equally `fs.txt` under `txt = "python"` or `fs.md` under `md = "python"`. A version-1
file has no language to ask, so there it is a home not named `.md` whose extension is in the
effective `[scan] extensions` ([§FS-scan.scope.extensions](FS-scan.md#scopeextensions-an-extension-makes-a-file-readable-not-declarable)). The name decides, and an `exclude`
or `include` entry does not change the answer: a `file` home is a scan root, read whether or
not `include` names it ([§FS-scan.scope.homes](FS-scan.md#scopehomes-every-configured-kind-home-is-scanned)), and neither `exclude` nor a hidden directory prunes
it ([§FS-scan.scope.roots](FS-scan.md#scoperoots-a-scan-root-outruns-every-rule-about-descent)), so `.x/fs.py` is read and refused too.

A home the scan reads as Markdown goes on being written, whatever its name, `fs.txt` under
`txt = "markdown"` included. A home the text scan does not read is refused, first write
included ([[§FS-fetch.4.2.1](FS-fetch.md#421-a-home-the-scan-does-not-read-is-refused)]).

The refusal is `fetch`'s alone. The configuration still loads, and `check`, `list`, `show`
and `refs` keep their verdicts on such a project, reading what the file declares; only the
remedy `check` names moves ([§FS-check.3.1.3.1](FS-check.md#3131-a-home-fetch-refuses)). [§DF-fetch-source-home](../decisions/functional/DF-fetch-source-home.md#df-fetch-source-home-fetch-refuses-a-file-home-the-text-scan-reads-as-source) records the
compatibility correction, and names as a judgement the one tree on which this refusal takes away a
fetch that worked.

#### 4.2.1. A home the scan does not read is refused

A `file` home the text scan does not read holds no declaration any command reads: a name no
language or `[scan] extensions` list claims, such as `fs.dat`, or a hidden name of its own,
such as `.fs.py` ([[§FS-scan.scope.hidden](FS-scan.md#scopehidden-a-hidden-file-is-not-read-and-the-rule-is-not-about-descent)]), where `check` already warns that the file is not
read. No read says where a declaration there ends, so a refresh cannot keep the bytes around
it ([[§FS-fetch.4](FS-fetch.md#4-file-home-write)]), and what a first write leaves is the home the next fetch would cut. So
`fetch` refuses it, first write included, with exit 2, empty stdout and the whole tree
byte-identical, before the integration runs, on stderr:

```text
error: kind `<KIND>` cannot fetch into <home>: <reason>, so the scan would not read what fetch writes; give the file a name a listed document language reads, or remove its `fetch`
```

`<home>` is spelled from the report base ([[§FS-errors.terms](FS-errors.md#terms-terms)]). `<reason>` is the one `check`'s
unread warning gives for that name ([[§FS-scan.checks.unread-scan-root.message](FS-scan.md#checksunread-scan-rootmessage-the-message)]), so a
correction to that reason moves both: ``no language in [schema.sources] languages reads `.dat` ``
or ``its name begins with `.`, and grund reads no hidden file`` in a version-2 file, and
`extension dat is not in [scan] extensions` in a version-1 file, whose line says `a name the
scan reads as Markdown` in place of `a name a listed document language reads`, since version 1
has no language to list. A home that declares the requested ID, one that declares nothing and
an absent one are all refused, and an absent one is not created.

In a `folder` home the same refusal holds for the `<ID>.md` [[§FS-fetch.5](FS-fetch.md#5-folder-home-write)] would create, when
the scan run before the integration ([[§FS-fetch.6.1](FS-fetch.md#61-one-scan-read-before-the-integration-runs-and-installed-against)]) finds no file in the home declaring the
ID and does not read that path as Markdown; `<reason>` is then why it does not. A file in the
home that does declare the ID is replaced or refused as [[§FS-fetch.5](FS-fetch.md#5-folder-home-write)] says, and is not judged
here.

The walk not reading that path is the same refusal, whatever the path holds: an `<ID>.md` an
ignore file ignores ([[§FS-scan.scope.ignore-files](FS-scan.md#scopeignore-files-respect_gitignore--the-ignore-files)]) or an `exclude` rule matches, absent or present, a plain
file or a link. Whether the walk reads it is asked of the walk's own read predicate, the one a
stub's target asks ([[§FS-declarations.stubs.target.3](FS-declarations.md#stubstarget3-a-target-the-scan-does-not-read-declares-nothing)]), from the config root and the roots the walk starts
from, so a configured home a rule names is still read and is written. `<reason>` is then the
rule, in the words of the broken-stub line, and since the file's name is the ID's, the repair is
the rule rather than a name:

```text
error: kind `<KIND>` cannot fetch into <home>: <file> ignores it by <pattern>, so the scan would not read what fetch writes; stop ignoring it in <file>, and fetch again
error: kind `<KIND>` cannot fetch into <home>: [schema.sources] exclude matches it by <pattern>, so the scan would not read what fetch writes; remove <pattern> from [schema.sources] exclude, and fetch again
error: kind `<KIND>` cannot fetch into <home>: [scan] exclude names <entry>, so the scan would not read what fetch writes; remove <entry> from [scan] exclude, and fetch again
```

`<file>` is the ignore file spelled as the report spells a path, from the report base
([§FS-cli.3.4](FS-cli.md#34---path-base--where-report-paths-are-spelled-from)), and absolute where it lies outside the loaded root, such as the global excludes file, so `check`'s tail names it alike; and `<pattern>` and `<entry>` are as written, as the broken-stub line spells them. A link there takes this line ahead of [[§FS-fetch.links](FS-fetch.md#links-a-link-at-the-target-is-written-through-when-it-stays-inside-the-config-root)].
Admission's reading after the integration holds the same predicate: a target the walk would not
read is not read there either, so the requested declaration is not there
([[§FS-fetch.admission.1](FS-fetch.md#admission1-the-requested-declaration-is-there-once-and-its-body-takes-in-all-of-the-output)]).

`check` stops offering `run grund fetch` at such a home ([[§FS-check.3.1.3.1](FS-check.md#3131-a-home-fetch-refuses)]).
[[§DF-fetch-unread-home](../decisions/functional/DF-fetch-unread-home.md#df-fetch-unread-home-fetch-refuses-a-home-the-scan-does-not-read-before-the-integration-runs)] records the compatibility correction.

### 4.3. A value home that cannot be looked up is refused with its reason

A value home whose lookup fails for a reason other than absence, such as a directory
above it without search permission, is refused at the same point, the same located line
([§FS-lsp.2.2.4.1](FS-lsp.md#2241-a-config-that-will-not-load-is-a-diagnostic-on-that-config)) and with the same exit 2 as an absent one, because a home grund
cannot check is not accepted. The refusal says why the lookup failed, and only that:

```text
<config>:<line>: could not look up value home for kind `<KIND>`: <home>: <reason>
hint: make the home and every directory above it accessible
```

`<reason>` is derived from the lookup's I/O error kind, without an OS error number, the
way [§FS-check.2.4.1](FS-check.md#241-one-reason-for-one-scan-io-failure) derives a scan failure's reason, so a denied lookup reads
`permission denied`. The hint is the same for a `file` and a `folder` home, in either
configuration version, whether or not the kind sets `fetch`. Neither line says
`does not exist` or `create the`: the home may be on disk, and creating it fails for the
same reason the lookup did. A home that is absent keeps the refusal and hints of
[§FS-fetch.4.1](FS-fetch.md#41-an-absent-value-home-is-refused-with-the-way-through) unchanged.

### 4.4. A home that cannot be decoded is named

A Markdown home file that cannot be decoded as UTF-8 is a home that cannot be read under
[§FS-fetch.4](FS-fetch.md#4-file-home-write), and it is refused as [§FS-fetch.5.4](FS-fetch.md#54-a-home-file-that-cannot-be-decoded-is-named) refuses a file in a folder home: with the line
`grund check` prints for that file, spelled from the report base.

## 5. Folder-home write

For `folder = "<home>"`, grund replaces the Markdown file that currently declares the requested
ID, or creates `<home>/<ID>.md` when no declaration exists. It never replaces a
different-ID file. Multiple existing declarations for the requested ID are ambiguous
and refused. The declaring file or new file contains the accepted stdout verbatim. Which file
declares, and what an existing `<ID>.md` that declares nothing may hold, is [[§FS-fetch.5.5](FS-fetch.md#55-which-file-declares-and-what-idmd-may-hold)]'s.

### 5.1 Existing value-source declarations prevent installation

A value source that the selected project's catalog recognizes as declaring the
requested ID in this folder home is an existing declaration, not an absent snapshot.
Before creating or replacing a Markdown snapshot, grund refuses this case on stderr
with exit 2, identifying the requested ID and declaring source and advising the user
to keep that fact in the value source or move it out of the value home before fetching.
Stdout is empty and the entire tree, including the source's other members, the index
and unrelated files, remains byte-identical. An additional Markdown declaration of
the same ID does not permit installation.

Discovery follows the existing value-source language, direct-child and ownership
rules, independently of general scan filters
([§FS-values.2.2](FS-values.md#22-declarations-from-value-sources)). It does not turn a file the
catalog leaves unread into a declaration. A source of either format is read and
declares its IDs ([§FS-values.2.2.4](FS-values.md#224-from-grund-0220-a-yaml-source-is-reported-as-a-json-one-is-after-one-release-of-warnings)),
so a YAML source refuses as a JSON one does, and a source in this home that cannot
be read or parsed is a discovery failure under [§FS-fetch.5](FS-fetch.md#5-folder-home-write). A source declaring
only other IDs does not prevent creation. Unrelated project findings do not gate
fetch. Markdown discovery and replacement retain their existing traversal semantics.

This refusal preserves [§FS-fetch.6](FS-fetch.md#6-stability-and-ownership)'s ownership boundary;
[§DF-fetch-folder-value-source](../decisions/functional/DF-fetch-folder-value-source.md#df-fetch-folder-value-source-folder-fetch-refuses-an-id-already-declared-by-a-value-source)
records the compatibility correction.

### 5.2 A file it does not replace that declares the ID prevents installation

Discovery walks the home as the scan does, and reads every file the walk yields under a
name other than `.md` through the scan's own pass, in whichever language that file is
read. A file in which that pass records a declaration of the requested ID is an existing
declaration, not an absent snapshot. Fetch can neither create beside it, which leaves a
duplicate, nor replace it, which overwrites what the file holds around the declaration
([§FS-fetch.6](FS-fetch.md#6-stability-and-ownership)). So before creating or replacing anything grund refuses, with exit 2, empty
stdout and the whole tree byte-identical, on stderr: The pass is the
one scan `fetch` runs before the integration ([[§FS-fetch.6.1](FS-fetch.md#61-one-scan-read-before-the-integration-runs-and-installed-against)]), read once for the whole tree,
not a second pass over each file.

```text
error: cannot fetch <ID>: <file> already declares it; keep the fact in that file, or take the declaration out of it and fetch again
```

`<ID>` is the local ID and `<file>` is spelled from the report base ([§FS-errors.terms](FS-errors.md#terms-terms)).
Where several such files declare it, `<file>` is the first in the walk's sorted order.

A Markdown file in the home that also declares the ID does not permit installation, and
this refusal comes ahead of every judgement of that Markdown file: it names the other
file whichever the walk reaches first, a Markdown file holding the ID beside other content
and a stub whose target is that file included. What the first paragraph of [§FS-fetch.5](FS-fetch.md#5-folder-home-write)
replaces is a file named `.md`, so a file the scan reads as Markdown under another name
takes this refusal rather than a replacement. The refusal holds whether or not the scan
reads `<ID>.md` as Markdown, and a stub the pass records under a name other than `.md` counts
as declaring, whatever its target, including one that does not exist yet.

What declares is the scan's answer, and fetch keeps no second reader of comments. A file
that declares only other IDs, that only cites the ID, or that holds the ID's text where
nothing declares does not stop the fetch, nor does a file the walk does not yield, because
no admitted language or `[scan] extensions` entry reads it or `[scan] exclude` keeps it
out. Findings the pass or a later `grund check` reports against another file do not gate
fetch. This pass asks only the home's own files, and a declaration outside them is
[§FS-fetch.5.3](FS-fetch.md#53-a-declaration-outside-the-home-prevents-installation)'s to refuse.

A file the pass cannot read or decode may hold the declaration, so it is a discovery
failure under [§FS-fetch.5](FS-fetch.md#5-folder-home-write) whatever else the home holds, and it is reported as [§FS-fetch.5.4](FS-fetch.md#54-a-home-file-that-cannot-be-decoded-is-named)
says.

[§DF-fetch-folder-declaring-file](../decisions/functional/DF-fetch-folder-declaring-file.md#df-fetch-folder-declaring-file-a-folder-home-fetch-refuses-an-id-a-file-it-does-not-replace-already-declares) records why this refusal is a compatibility correction.

### 5.3 A declaration outside the home prevents installation

"When no declaration exists" is the selected project's scan's answer, not the home's. It is the
same answer for a `file` home ([[§FS-fetch.4](FS-fetch.md#4-file-home-write)]), whose fetch is refused with the same line, by the
same test. Both are refused before the integration runs, from the one scan
([[§FS-fetch.6.1](FS-fetch.md#61-one-scan-read-before-the-integration-runs-and-installed-against)]), behind the refusals that need no scan and the home's own refusals of
[[§FS-fetch.4](FS-fetch.md#4-file-home-write)] and [[§FS-fetch.4.2.1](FS-fetch.md#421-a-home-the-scan-does-not-read-is-refused)].
Discovery asks that scan for the requested ID, with the ID's stubs paired by their verdicts
([§FS-declarations.stubs.pairing.1](FS-declarations.md#stubspairing1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned)), and a declaration whose record is in a file outside the
home is an existing declaration, not an absent snapshot. Fetch can neither create beside it,
which leaves a duplicate, nor replace it, which overwrites a fact that was written somewhere
fetch does not own ([§FS-fetch.6](FS-fetch.md#6-stability-and-ownership), [§REQ-no-data-loss.2](../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own)). So before creating or replacing
anything grund refuses, with exit 2, empty stdout and the whole tree byte-identical, an absent
home still absent, on stderr:

```text
error: cannot fetch <ID>: <file> already declares it; keep the fact in that file, or take the declaration out of it and fetch again
```

`<ID>` is the local ID and `<file>` is spelled from the report base ([§FS-errors.terms](FS-errors.md#terms-terms)). A
declaration reached through a stub is named at its target, never at the stub
([§FS-declarations.stubs.pairing.3](FS-declarations.md#stubspairing3-a-declaration-reached-through-stubs-is-named-at-its-target)). Where several files outside the home declare it, `<file>`
is the first in `path:line` order.

Which declarations those are is read from the one pairing every command reads, and fetch
keeps no second reader of comments or stubs:

- A comment declaration in a source file and a Markdown declaration outside the home each
  refuse.
- A healthy stub in the home whose target outside it declares the ID refuses, whether or not
  the scan reaches that target. It is the pointer to that declaration, not a declaration of
  the home's ([§FS-declarations.stubs.pairing.1](FS-declarations.md#stubspairing1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned)), so it is not the file [§FS-fetch.5](FS-fetch.md#5-folder-home-write) replaces.
  It stays as it was, and `grund show` answers as it did.
- A healthy stub outside the home whose target in the home declares the ID does not refuse:
  the declaration is the home's, and it is replaced where it is, under the stub. Such a stub
  sits in another kind's home. A heading of that shape outside every home is a declaration
  titled with a link ([§FS-declarations.stubs.shape](FS-declarations.md#stubsshape-what-a-stub-looks-like)), and refuses, named at its own file.
- A broken stub outside the home is a declaration of its own
  ([§FS-declarations.stubs.pairing.1](FS-declarations.md#stubspairing1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned)), so it refuses, named at its own file.
- A declaration in the home beside one outside it does not permit installation. This refusal
  comes ahead of every judgement of the home's contents, so a Markdown file holding the ID
  beside other content takes this refusal rather than its own, and so do a value source
  ([§FS-fetch.5.1](FS-fetch.md#51-existing-value-source-declarations-prevent-installation)) and a file [§FS-fetch.5.2](FS-fetch.md#52-a-file-it-does-not-replace-that-declares-the-id-prevents-installation) would refuse. A stub the pass reads under a name
  other than `.md`, whose target outside the home declares the ID, is therefore named at that
  target. [§FS-fetch.5.2](FS-fetch.md#52-a-file-it-does-not-replace-that-declares-the-id-prevents-installation) names such a stub only when nothing outside the home declares the ID.
- A broken stub in the home, with nothing declared outside the home, stays the home's own
  declaration. In a file named `.md` it is replaced, and under another name
  [§FS-fetch.5.2](FS-fetch.md#52-a-file-it-does-not-replace-that-declares-the-id-prevents-installation) refuses it. That is also the way through for a healthy one in a `.md` file: with the
  declaration taken out of the target the stub is broken, and the fetch replaces it.

What the scan does not read cannot refuse: a file in a language the project does not admit or
under an extension `[scan] extensions` does not list, one under an excluded directory, and one
`[scan] include` does not reach and no stub points at. A file that only cites the ID, or holds
its text in a fence, declares nothing. In a workspace the scan asked is the selected
project's ([§FS-fetch.1](FS-fetch.md#1-input-and-project-selection)), so another project declaring the same local ID does not count, and
`<file>` is still spelled from the report base.

Only the requested ID's declarations are read from that scan. No finding gates fetch, and
neither does a per-file failure ([§FS-check.2.4](FS-check.md#24-an-incomplete-run)) outside the home: a file the scan cannot read
or decode, a directory it cannot list, the config root included, and a link it cannot resolve.
Such a path may hold the declaration, and the fetch goes ahead regardless, because it is
unrelated to the request until shown otherwise and `grund check` already exits 2 on that tree,
so an exit 0 turns nothing red. It hides nothing either: a declaration another file holds still
refuses. Inside the home a file discovery cannot read stays a discovery failure under
[§FS-fetch.5](FS-fetch.md#5-folder-home-write), and so does a scan that could not run at all, which has no answer for any ID.

[§DF-fetch-declaration-outside-home](../decisions/functional/DF-fetch-declaration-outside-home.md#df-fetch-declaration-outside-home-a-folder-home-fetch-refuses-an-id-declared-outside-the-home) records why this refusal is a compatibility correction.

### 5.4 A home file that cannot be decoded is named

Discovery reads the home's files to answer [§FS-fetch.5](FS-fetch.md#5-folder-home-write), so a file in it that cannot be decoded
as UTF-8 stops the fetch whichever file it is: it may hold the requested ID's declaration, and
that the requested ID's own file reads cleanly settles nothing about the others. Whatever the
file's name, the failure is exit 2, empty stdout, the tree byte-identical, and on stderr the
line `grund check` prints for that file ([§FS-check.2.4](FS-check.md#24-an-incomplete-run)), spelled from the report base
([§FS-errors.terms](FS-errors.md#terms-terms)). One broken file reads the same in both commands, so the reader of a fetch
that failed is told which file to repair without running another command.

Where several files cannot be decoded, the line is for the first in the walk's sorted order.
A file [§FS-fetch.5.2](FS-fetch.md#52-a-file-it-does-not-replace-that-declares-the-id-prevents-installation)'s pass cannot read is reported the same way.

### 5.5. Which file declares, and what `<ID>.md` may hold

The declaring file is the file named `.md` in which the selected project's scan reads the
requested ID, under whatever heading the scan reads as declaring it and at whatever depth
([[§FS-fetch.3](FS-fetch.md#3-accepted-declaration)]). Its whole content is replaced, so it may hold nothing beyond the range the
scan records for that declaration ([[§FS-fetch.4](FS-fetch.md#4-file-home-write)]) but blank lines. A file that holds anything
else beside it — a group heading above it, prose before it, another declaration — is refused,
with the path spelled from the report base ([[§FS-errors.terms](FS-errors.md#terms-terms)]):

```text
error: snapshot file <path> contains the requested ID beside other content
```

"When no declaration exists" never licenses overwriting a file that holds text. Where the
scan reads no declaration of the ID and `<home>/<ID>.md` exists, the file is filled when it
holds no bytes, or only whitespace and line endings, since there is nothing in it to lose and
after the write the scan reads the requested declaration there once. Any other such file —
prose, a heading that declares nothing such as `# Notes`, a lone citation — is refused with the
file byte-identical, and the refusal says what the scan reads there:

```text
error: snapshot file <path> already exists and the scan reads no ID in it; move or rename it, and fetch again
```

A file in which the scan reads another ID keeps the words it was refused with before:

```text
error: snapshot file <path> already exists for a different ID
```

[[§DF-fetch-refusals-follow-the-scan](../decisions/functional/DF-fetch-refusals-follow-the-scan.md#df-fetch-refusals-follow-the-scan-four-fetch-refusals-say-what-the-scan-reads-corrected-in-place-and-an-empty-idmd-is-filled)] records the corrected line and the fill.

The replacement is atomic. A missing folder may be created only as part of the
successful final installation; any discovery, validation, or write failure leaves the
tree byte-identical and exits 2.

That does not reach a value kind or a kind that sets `value_chapter`: its folder home
must already exist ([§FS-config.3.4.9](FS-config.md#349-values--first-class-value-declarations), [§FS-config.3.4.13](FS-config.md#3413-value_chapter--the-chapter-whose-named-children-are-values)), and an absent one is refused at
configuration load with the hint of [§FS-fetch.4.1](FS-fetch.md#41-an-absent-value-home-is-refused-with-the-way-through).

## 6. Stability and ownership

Fetching unchanged output is idempotent: it leaves the same file set and bytes. Stable
ID-order insertion and declaration-local replacement make repeated and independent
fetches produce reviewable diffs. `fetch` owns only the requested declaration in the
configured snapshot home, extending [§REQ-no-data-loss.2](../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own); it never prunes another snapshot or rewrites a citation.

Idempotence is checked before anything is installed, as [[§FS-fetch.admission.5](FS-fetch.md#admission5-fetching-the-same-output-again-leaves-the-same-bytes)]: output whose
second fetch would change the bytes its first one wrote is refused, so a refresh never drifts.

One `fetch <ID>` fetches one ID. There is no `--refresh`, generated timestamp,
unused-snapshot exemption, or workspace-wide snapshot ownership behavior.

### 6.1. One scan, read before the integration runs, and installed against

`fetch` runs the selected project's scan once, after every refusal that needs no scan and before
the integration runs. That one reading answers every question about the tree as it stood: which
file declares the ID ([[§FS-fetch.4](FS-fetch.md#4-file-home-write)], [[§FS-fetch.5.5](FS-fetch.md#55-which-file-declares-and-what-idmd-may-hold)]), whether a declaration outside the home or
under another name stops the fetch ([[§FS-fetch.5.2](FS-fetch.md#52-a-file-it-does-not-replace-that-declares-the-id-prevents-installation)], [[§FS-fetch.5.3](FS-fetch.md#53-a-declaration-outside-the-home-prevents-installation)]), whether a folder home's
`<ID>.md` is read ([[§FS-fetch.4.2.1](FS-fetch.md#421-a-home-the-scan-does-not-read-is-refused)]), and what admission compares against
([[§FS-fetch.admission](FS-fetch.md#admission-admission-the-next-scan-reads-back-exactly-what-was-asked-for)]). No second discovery scan runs after the integration.

The integration runs between that reading and the write, so the write is made against what the
scan read: the target is replaced only if it still holds the bytes the scan read there, and
created only if it is still absent. A home changed while the integration ran, edited or created,
is not overwritten: the fetch exits 2, empty stdout, with the change left as it was, and says so
from the report base ([[§FS-errors.terms](FS-errors.md#terms-terms)]):

```text
error: <path> changed while the integration ran; fetch again
```

## 7. Output and exits

Success writes nothing to stdout or stderr and exits 0. An unparseable ID is a query
failure and exits 1. Missing fetch configuration, a `file` home that is a value source
([§FS-fetch.4](FS-fetch.md#4-file-home-write)) or is read as source ([§FS-fetch.4.2](FS-fetch.md#42-a-home-read-as-source-is-refused)), integration failure, rejected output,
ambiguous existing content, a folder-home file that already declares the ID
([§FS-fetch.5.2](FS-fetch.md#52-a-file-it-does-not-replace-that-declares-the-id-prevents-installation)), and filesystem failure are CLI-level operational errors on
stderr and exit 2, following
[§FS-errors.2.2](FS-errors.md#22-cli-level-message).
A fetch of an ID declared outside the home ([§FS-fetch.5.3](FS-fetch.md#53-a-declaration-outside-the-home-prevents-installation)), a home the scan does not read
([[§FS-fetch.4.2.1](FS-fetch.md#421-a-home-the-scan-does-not-read-is-refused)]), output admission refuses ([[§FS-fetch.admission](FS-fetch.md#admission-admission-the-next-scan-reads-back-exactly-what-was-asked-for)]), a link at the target that
leaves the config root or points nowhere ([[§FS-fetch.links](FS-fetch.md#links-a-link-at-the-target-is-written-through-when-it-stays-inside-the-config-root)]), and a home changed while the
integration ran ([[§FS-fetch.6.1](FS-fetch.md#61-one-scan-read-before-the-integration-runs-and-installed-against)]) are refused the same way.

Every path a refusal names is spelled from the report base ([[§FS-errors.terms](FS-errors.md#terms-terms)]), whichever
refusal names it, so one tree's refusal reads the same from every checkout
([[§FS-errors.4](FS-errors.md#4-determinism)]). That covers the folder, file and value-source paths of [[§FS-fetch.5.1](FS-fetch.md#51-existing-value-source-declarations-prevent-installation)],
[[§FS-fetch.5.5](FS-fetch.md#55-which-file-declares-and-what-idmd-may-hold)] and the discovery failures of [[§FS-fetch.5](FS-fetch.md#5-folder-home-write)].

Once it ships, `fetch --remote` exits 0 when the projection and lock are installed, and prints the
remote's own content findings, which never block the pin
([§FS-fetch.remote.findings](FS-fetch.md#remotefindings-refuse-only-what-this-grund-cannot-load)). A refused remote fetch exits 2 with the old projection and
lock byte-identical. `--check` exits 0 when every selected pin is current, 1 on any
difference, a rewind included, and 2 when a ref cannot be obtained
([§FS-fetch.remote.check](FS-fetch.md#remotecheck---check-is-a-read-only-drift-signal)).

## admission: Admission: the next scan reads back exactly what was asked for

`fetch` builds the file it would install — the home with the output in place of the owned range
([[§FS-fetch.4](FS-fetch.md#4-file-home-write)]), or the output whole in a `folder` home ([[§FS-fetch.5](FS-fetch.md#5-folder-home-write)]) — has the selected
project's own scan read it, and installs it only if that reading is exactly what was asked for.
Each of the five points below is one clause, answered by one reading: the output alone, as the
scan reads one file at the target's path; the tree as the scan read it before the integration ran
([[§FS-fetch.6.1](FS-fetch.md#61-one-scan-read-before-the-integration-runs-and-installed-against)]); and the home as it would read after, with the candidate in place of the file
on disk and no other file read again. A clause that fails refuses the fetch with exit 2, empty stdout, the whole tree
byte-identical, and the line the clause names. The clauses are judged in order, and only the
first that fails is reported.

`fetch` keeps no heading test of its own, no second line loop and no depth limit: what a
declaration is, where its body ends and which structure is wrong each have their one answer in
the scan and in `check`, and `fetch` reads those answers rather than copying them. Accepted
bytes stay verbatim ([[§FS-fetch.3](FS-fetch.md#3-accepted-declaration)]): admission refuses output, it never rewrites it.

### admission.1: The requested declaration is there once, and its body takes in all of the output

Read from the output alone. The first line declares the requested ID at the level the home
takes ([[§FS-fetch.3](FS-fetch.md#3-accepted-declaration)]); no later heading the scan reads as declaring an ID stands inside it
([[§FS-fetch.3.1](FS-fetch.md#31-a-nested-declaration-is-refused)]); and no later heading ends it before the output does. A heading that ends it —
a bare `##` or `#`, or any heading at or above the declaration's level that declares nothing —
leaves the text after it belonging to nothing on the next scan, so it is refused:

```text
error: fetch output line <n>: heading `<heading>` ends <ID> before the output does; fence it, make it a bold label, or remove it
```

`<n>` is the heading's 1-based line in the output and `<heading>` that line as written, trailing
whitespace removed. A heading that declares a second ID at the declaration's own level keeps
[[§FS-fetch.3](FS-fetch.md#3-accepted-declaration)]'s `must contain exactly one declaration and no leading content`. Leading blank
lines are content before the declaration, and refused with that same line, never trimmed.

### admission.2: Every other declaration in the home reads as it did before

Read from the tree before and the home after. Once the replaced range is cut from both
readings, every other declaration the scan reads in the home is there in both, with the same ID,
title, level, sections and body text. A declaration that appears, disappears, or changes in any
of those is refused, naming where it stood before. One the scan read inside the replaced range
would be deleted, and takes [[§FS-fetch.3.1](FS-fetch.md#31-a-nested-declaration-is-refused)]'s line; any other takes:

```text
error: cannot fetch <ID>: writing the output would change how <file>:<line> reads <OTHER>; change the output or <OTHER> so the two stand apart, and fetch again
```

`<file>:<line>` is `<OTHER>`'s heading before the fetch, spelled from the report base
([[§FS-errors.terms](FS-errors.md#terms-terms)]). An open fence in the home, an indented sibling the scan reads as a heading,
and an H2 written where an enclosing declaration's tail would fall under it are refused here,
not discovered on the next scan.

### admission.3: The structural rules `check` applies report nothing new

`check`'s node-local rules run on the tree before and on the home after: duplicate, misplaced
and near-miss declarations, broken stubs, heading levels, duplicate and orphan sections,
unmarked headings, inline citation style, oversized leads, fields, conflict markers and glob
citations, and the scan's own invalid-value records ([[§FS-check](FS-check.md#fs-check-grund-validates-every-citation-in-a-repo)]). The relational rules —
citations, locks, chapter rules, grounding — are not run, because a fetched body's citations are
the reader's to settle ([[§FS-fetch.3](FS-fetch.md#3-accepted-declaration)]). A finding of any channel, warning included, that the
after reading holds and the before reading does not is refused; findings are compared by code
and message, with every line a finding names below the replaced range mapped through the splice
first, so a finding the home already held is never new because the output's length moved it.
A new finding inside the output is refused as:

```text
error: fetch output line <n>: <message>
```

and one elsewhere in the home as `error: cannot fetch <ID>: <file>:<line>: <message>`, where
`<message>` is `check`'s message for the finding and `<n>` its line in the output. An unmarked
heading takes [[§FS-fetch.3.2](FS-fetch.md#32-a-subsection-is-a-heading-the-next-scan-reads-as-a-section)]'s line in place of `check`'s. A value chapter's wrong shape, a
section path `check` faults and a duplicate section are refused here with `check`'s words.

### admission.4: The output closes every fence and raw-text block it opens

Read from the output alone. A fenced block or an HTML raw-text block (`<pre>`, `<script>`,
`<style>`, `<textarea>`) the output opens and does not close would swallow every line after it in
the home, so it is refused, naming the line that opens it:

```text
error: fetch output line <n> opens a <block> it does not close; close it, and fetch again
```

`<block>` is `fenced block` or `raw-text block`.

### admission.5: Fetching the same output again leaves the same bytes

Read from the home after. The output spliced again into the candidate, at the range the
candidate's own reading records for the requested declaration, must reproduce the candidate byte
for byte. Output that would grow, shrink or move on its next fetch — a leading blank line, a
final bare carriage return beside a sibling — is refused, naming the first line that would
change:

```text
error: cannot fetch <ID>: a second fetch of this output would change <file> at line <n> again, so the first is refused
```

`<file>` is spelled from the report base ([[§FS-errors.terms](FS-errors.md#terms-terms)]).

## links: A link at the target is written through when it stays inside the config root

The scan follows a file link and reads the declaration at the link's path
([[§FS-scan.scope.links](FS-scan.md#scopelinks-a-symlink-in-the-tree-is-followed)]), so the file behind the link is the declaration, and cutting the link
would change what the scan reads without anyone asking. A `folder` home's declaring file or
`<ID>.md`, and a `file` home, that is a symbolic link is decided by the walk's own record of it,
the one `fmt --write` asks ([[§FS-fmt.2.3.2](FS-fmt.md#232-a-link-that-leaves-the-config-root-is-not-written-through)]); `fetch` resolves no link a second time.

- A link whose target the walk puts inside the selected project's config root is written
  through: the write is atomic at the target, the link stays, and admission judges the written
  target like any other file.
- A link whose target leaves the config root is refused, by its path, with link and target
  byte-identical, also when the target already holds the bytes the integration printed:

  ```text
  error: cannot fetch <ID> into <path>: the symlink target is outside the config root; replace the link with the file it points to, and fetch again
  ```

- A link whose target does not exist is refused the same way, whichever language reads `.md`:

  ```text
  error: cannot fetch <ID> into <path>: the symlink target does not exist; create the target, or remove the link, and fetch again
  ```

- A link the walk does not read as a file for another reason than a rule that skips it, a link
  to a directory, has no record to decide it by, and the scan would not read what `fetch` wrote
  through it. It is refused the same way, wherever it leads. A folder home's `<ID>.md` link an
  ignore file or an `exclude` rule skips is one the scan does not read, and takes
  [[§FS-fetch.4.2.1](FS-fetch.md#421-a-home-the-scan-does-not-read-is-refused)]'s line instead:

  ```text
  error: cannot fetch <ID> into <path>: the scan does not read the symlink as a file; remove the link, or let the scan read it, and fetch again
  ```

`<path>` is the link, spelled from the report base ([[§FS-errors.terms](FS-errors.md#terms-terms)]). Where the home is also
one the scan does not read, [[§FS-fetch.4.2.1](FS-fetch.md#421-a-home-the-scan-does-not-read-is-refused)] speaks first. `fmt` refuses an outward link with a
warning and goes on, while `fetch` refuses the run, because `fetch` writes one file.

## remote: Remote projections

This chapter is specified ahead of the releases that build it, as all of [§FS-remote-projects](FS-remote-projects.md#fs-remote-projects-a-project-cites-another-repositorys-declarations-from-a-committed-pinned-projection) is; until they land, `--remote` is refused as an unknown flag with exit 2.

### remote.selector: `--remote <alias>` projects one declared remote

`grund fetch --remote <alias>` reads the repository that the config root's
`[workspace.remotes.<alias>]` table declares ([§FS-remote-projects.declaration.table](FS-remote-projects.md#declarationtable-the-remote-table)). It
resolves the declared `ref` at the declared `source` to a full commit with the user's
`git`, reads that commit's complete tracked tree from git objects without checking it out,
validates it, writes it to `.grund/remotes/<alias>/` ([§FS-remote-projects.projection](FS-remote-projects.md#projection-the-projection)), and
rewrites that remote's lock entry ([§FS-remote-projects.lock](FS-remote-projects.md#lock-the-lock)) — one reviewable diff. It runs
nothing the remote supplies: no hook, checkout filter, script, checker or `fetch`
integration. An alias the config root does not declare is refused with exit 2. `fetch
--remote` needs `git` on `PATH`; no other command does.

### remote.install: Validate the whole projection, then replace the pair

The new projection is validated completely, entry types and byte protection included
([§FS-remote-projects.bytes](FS-remote-projects.md#bytes-exact-bytes)), before anything is replaced. Only then are the projection
and its lock entry replaced together; a fetch refused at any step leaves both exactly as
they were. Fetch owns only `.grund/remotes/<alias>/` and that remote's lock entry
([§FS-remote-projects.lock.ownership](FS-remote-projects.md#lockownership-the-lock-is-grunds)).

### remote.findings: Refuse only what this grund cannot load

Fetch refuses to lock a commit whose projected config or required content this grund
cannot load, because installing it would only defer the failure to the next `check`. It
locks a commit whose own content has findings and reports them on stdout, because gating a
pin on upstream housekeeping is the coupling remotes exist to avoid.

### remote.commit: Paths the consumer's git would not commit are named

A remote may track a file that its own `.gitignore`, or a pattern of the consumer's, would
keep a plain `git add` from committing in the consumer, and every fresh clone would then
report `remote-modified`. After installing, fetch asks the consumer's git whether each
projected path would be committed and, when any would not, prints those paths and the
exact `git add --force` command that commits them. It never touches the index.

### remote.all: `--remote --all` fetches every declared remote

`grund fetch --remote --all` fetches every remote this config root declares, each as
`--remote <alias>` would. It never fetches a remote declared inside a projection: those are
owned by the projection's own lock and move only when the remote that holds them is
refetched.

### remote.check: `--check` is a read-only drift signal

`grund fetch --remote <alias> --check`, or `--remote --all --check`, resolves each selected
declared ref and compares it with the locked commit, writing nothing. It reports every
difference, a rewind to an older commit included. It exits 0 when every selected pin is
current, 1 on any difference, and 2 when a ref cannot be obtained, never reporting a pin it
could not compare as current. `--check` never runs inside `grund check`.

### remote.from: `--from <path>` projects a committed revision of a local clone

`grund fetch --remote <alias> --from <path>` reads the declared `ref` from the local clone
at `<path>` instead of from `source`, so work not yet pushed can be pinned for review. It
refuses a clone whose working tree is dirty, and it reads committed objects only. The lock
still records the declared portable `source`, never `<path>`, so a colleague or CI can
reproduce the pin.
