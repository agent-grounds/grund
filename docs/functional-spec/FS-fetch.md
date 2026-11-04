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
the query also reads ([§FS-config.3.2](FS-config.md#32-id--id-grammar)). `grund fetch alias/TICKET-1234` loads that member's config and home, renders the qualified form in user messages, and passes only `TICKET-1234` to the integration.
A malformed alias path is refused as [§FS-workspace.8.11](FS-workspace.md#811-a-malformed-alias-path-names-the-offending-segment) says, naming the offending segment, with exit 2; `grund fetch Bad/TICKET-1234` prints what `grund show Bad/TICKET-1234` prints.

An ID that cannot be parsed is a bare query failure ([§FS-errors.2.3](FS-errors.md#23-bare-query-failure)) and exits 1.
A parsed ID whose kind has no `fetch`, or whose selected project cannot supply exactly
one configured file or folder home, is an operational refusal on stderr and exits 2.

## 2. Integration invocation

`[[kinds]].fetch` names one executable. A relative value is resolved from the selected
project's config root. Grund invokes that path directly, never through a shell, appends
the local unqualified ID as the integration's sole argument, inherits the user's
environment, and provides no implicit input on stdin. Deliberately running `fetch` is
sufficient authorization to execute repository-controlled configuration.

Fetcher stdout is snapshot data and is never copied to grund's stdout. Exit 0 advances
to validation. A spawn failure, signal, or non-zero integration exit is an operational
error on stderr and exit 2. The message identifies the configured integration and ID;
the tree remains byte-identical.

## 3. Accepted declaration

The complete stdout must be UTF-8 and contain exactly one Markdown declaration for the
requested local ID, with no sibling declaration:

- a `file` home accepts one H2 declaration (`## TICKET-1234: Title`);
- a `folder` home accepts one H1 declaration (`# TICKET-1234: Title`).

The declaration may contain ordinary prose, fenced blocks, and subsections exactly one
heading depth below its native declaration depth ([§FS-fetch.3.2](FS-fetch.md#32-a-subsection-is-a-heading-the-next-scan-reads-as-a-section)). Its title must be
non-empty. A malformed heading, wrong ID or kind, wrong native depth, duplicate
declaration, sibling declaration, invalid subsection depth, invalid UTF-8, or trailing content outside the
one declaration is refused on stderr with exit 2.

The whole output is validated before any filesystem mutation. Accepted bytes are
preserved verbatim; grund does not add timestamps, normalize whitespace, run `fmt`, or
sanitize marked citations. A marked citation in the body is therefore a live citation
on the next ordinary scan ([§FS-check.1.1](FS-check.md#11-recognized-citations)).

### 3.1 A nested declaration is refused

"Exactly one declaration" counts every heading the next scan would read as a
declaration, not only those at the native depth. A heading inside the fetched
declaration whose text declares an ID of any configured kind, under the project's ID
grammar, is refused on stderr with exit 2 and the tree byte-identical, whatever its
depth below the native one and whichever ID it carries: an unrequested ID, the
requested ID a second time, an ID the snapshot home already declares, or another
kind's ID. The refusal names the nested ID. It holds under config version 1 and 2 and
for `file` and `folder` homes alike, because `fetch` owns only the requested
declaration and one `fetch <ID>` fetches one ID ([§FS-fetch.6](FS-fetch.md#6-stability-and-ownership)).

A subsection heading ([§FS-fetch.3.2](FS-fetch.md#32-a-subsection-is-a-heading-the-next-scan-reads-as-a-section)), which carries a numbered or named section path rather than an
ID, stays accepted, as does any heading inside a fenced block, which declares nothing.
The rule binds only what `fetch` accepts: a nested declaration written by hand into a
scanned file stays legal.

### 3.2. A subsection is a heading the next scan reads as a section

A subsection is a heading one depth below the declaration that the next ordinary scan
reads as a section of it, so the one rule that decides it is the scanner's
([§FS-declarations.checks.unmarked-heading](FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading)): a numeric section path (`### 1. Notes`),
a named one where `[id] named_sections` enables names (`### notes: Notes`), or a
relative heading ([§FS-declarations.section.relative](FS-declarations.md#sectionrelative-a-heading-may-write-only-the-component-it-adds)) the same configuration admits
(`### .notes: Notes`). Whether a heading qualifies is read with the selected project's
configuration, never by matching its text again, so with `named_sections` unset
`### notes: Notes` is not a subsection while `### 1. Notes` still is.

Any other heading at subsection depth — a prose heading (`### Other`), an ID of a kind
the project does not configure (`### XX-002-other: Other`), or an escaped one
(`### <§>AR-002-other: Other`) — is a malformed heading: the fetch is refused on stderr
with exit 2 and the tree stays byte-identical, rather than accepted and turned red by
the next `grund check`. The refusal names the heading and its line in the output and
says how to keep it:

```text
error: fetch output line <n>: heading `<heading>` is not a section of `<ID>`; number it, fence it, or make it a bold label
```

`<n>` is the heading's 1-based line in the output and `<heading>` that line as written,
trailing whitespace removed. A `file` home's H3 and a `folder` home's H2 are held
alike. A heading inside a fenced block is content and a bold label is no heading, so
both stay accepted, as they are on the next scan.

## 4. File-home write

For `file = "<home>"`, grund replaces only the H2 declaration whose ID exactly matches
the requested ID. If it is absent, grund inserts the declaration among sibling H2
declarations in ID order. The file's preamble, other declarations, line endings, and
all bytes outside the owned declaration remain unchanged.

The final file is installed atomically only after the complete result is available. If
the home cannot be read, parsed unambiguously, or atomically replaced, the command emits
an operational error, exits 2, and leaves it byte-identical.

A home that is a value source ([§FS-values.terms](FS-values.md#terms-terms)), in either format, holds no Markdown
declaration, so it is refused the same way. The refusal is decided by the home's path and
its kind, not by what the file holds, and comes before the integration runs. A value kind
whose `file` home is a Markdown file has no value source there, and takes the write above.

## 5. Folder-home write

For `folder = "<home>"`, grund replaces the file that currently declares the requested
ID, or creates `<home>/<ID>.md` when no declaration exists. It never replaces a
different-ID file. Multiple existing declarations for the requested ID are ambiguous
and refused. The declaring file or new file contains the accepted stdout verbatim.

The replacement is atomic. A missing folder may be created only as part of the
successful final installation; any discovery, validation, or write failure leaves the
tree byte-identical and exits 2.

### 5.1 The declaring file is found by the scan's walk

grund looks for the declaring file by walking `<home>` recursively the way a scan does, so
it reads only what that walk admits: a `.md` file the project's sources read as text
([§FS-config-v2.schema.sources.languages](FS-config-v2.md#schemasourceslanguages-languages-admits-languages-and-only-it-does), [§FS-config-v2.schema.sources.extensions](FS-config-v2.md#schemasourcesextensions-schemasourcesextensions-maps-a-suffix-to-a-language), version
1's `extensions` in [§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned)), with no hidden name, ignore-file rule or exclusion
keeping it out. A file the walk skips is not found, so under any name but `<home>/<ID>.md`
it is neither replaced nor in the way: grund writes `<home>/<ID>.md` beside it. A rule
that covers `<home>` itself keeps nothing out, because the walk starts there.

A link is followed as a scan follows it ([§FS-scan.scope.links](FS-scan.md#scopelinks-a-symlink-in-the-tree-is-followed)), so a link the walk admits
and that declares the requested ID is the declaring file: its path is replaced by a
regular file, and what it led to is left alone.

### 5.2 `<home>/<ID>.md` is read whatever the walk admits

When the walk finds no declaring file the write would land on `<home>/<ID>.md`, so what
already stands at that one path is judged by what it holds, whether or not the walk
admitted it. `md = "json"`, a `languages` list without Markdown, a version 1 `extensions`
list without `md`, an ignore-file rule and an exclusion can each keep the file from the
walk, and none of them says anything about what it declares. It is read as the walk's
files are, and what it holds decides:

- A file declaring the requested ID alone is the declaring file and is replaced, so a
  second `grund fetch` of one ID rewrites the snapshot the first wrote under any of those
  configurations.
- A file declaring the requested ID beside anything else is refused:
  `snapshot file <path> contains the requested ID beside other content`.
- A file declaring another ID is refused:
  `snapshot file <path> already exists for a different ID`.
- A file declaring nothing, empty or prose alike, is refused:
  `snapshot file <path> already exists and does not declare <ID>`. It holds no different
  ID, and `fetch` owns only the requested declaration ([§FS-fetch.6](FS-fetch.md#6-stability-and-ownership)).
- A link that leads anywhere and a directory are refused and left as they are:
  `snapshot file <path> is not a regular file`. What a link leads to is not read here,
  so this read can never replace a link.

Reading the file here admits it to nothing else: a scan still skips it, and so does every
command that scans.

## 6. Stability and ownership

Fetching unchanged output is idempotent: it leaves the same file set and bytes. Stable
ID-order insertion and declaration-local replacement make repeated and independent
fetches produce reviewable diffs. `fetch` owns only the requested declaration in the
configured snapshot home, extending [§REQ-no-data-loss.2](../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own); it never prunes another snapshot or rewrites a citation.

One `fetch <ID>` fetches one ID. There is no `--refresh`, generated timestamp,
unused-snapshot exemption, or workspace-wide snapshot ownership behavior.

## 7. Output and exits

Success writes nothing to stdout or stderr and exits 0. An unparseable ID is a query
failure and exits 1. Missing fetch configuration, a `file` home that is a value source
([§FS-fetch.4](FS-fetch.md#4-file-home-write)), integration failure, rejected output, ambiguous existing content, and
filesystem failure are CLI-level operational errors on stderr and exit 2, following
[§FS-errors.2.2](FS-errors.md#22-cli-level-message).

Once it ships, `fetch --remote` exits 0 when the projection and lock are installed, and prints the
remote's own content findings, which never block the pin
([§FS-fetch.remote.findings](FS-fetch.md#remotefindings-refuse-only-what-this-grund-cannot-load)). A refused remote fetch exits 2 with the old projection and
lock byte-identical. `--check` exits 0 when every selected pin is current, 1 on any
difference, a rewind included, and 2 when a ref cannot be obtained
([§FS-fetch.remote.check](FS-fetch.md#remotecheck---check-is-a-read-only-drift-signal)).

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
