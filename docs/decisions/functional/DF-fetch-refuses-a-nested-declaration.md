# DF-fetch-refuses-a-nested-declaration: `grund fetch` refuses to replace a declaration nested in the one it fetches, in both homes

**Date:** 2026-11-04

## status: Accepted

## context: Context

A fetch replaced everything up to the next heading at the home's own depth, so a declaration written inside the requested one went with the old body, on exit 0, while `grund list` and `grund <ID>` treated it as a declaration of its own. Where nothing cited the lost ID the next `check` stayed green too, and only version control still held the text ([issue #1182](https://github.com/agent-grounds/grund/issues/1182)). [§FS-fetch.4](../../functional-spec/FS-fetch.md#4-file-home-write) promises a file's other declarations survive, [§FS-fetch.6](../../functional-spec/FS-fetch.md#6-stability-and-ownership) says fetch owns only the requested declaration, [§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own) is the requirement behind both, and [§FS-fetch.3.1](../../functional-spec/FS-fetch.md#31-a-nested-declaration-is-refused) calls such a declaration legal. [§FS-fetch.5](../../functional-spec/FS-fetch.md#5-folder-home-write) says a folder home's file holds the accepted stdout verbatim, so there the clauses pull apart.

## decision: Decision

Refuse, in both homes ([§FS-fetch.6.nested](../../functional-spec/FS-fetch.md#6nested-a-declaration-inside-what-a-fetch-replaces-is-refused)). A fetch whose replaced bytes hold a line the next scan records as a declaration exits 2, names the line and how to keep the declaration, and leaves the tree byte-identical. Which lines count is the scanner's answer, so the rule covers everything `grund list` shows, not only deeper headings.

[§FS-fetch.3.1](../../functional-spec/FS-fetch.md#31-a-nested-declaration-is-refused) moves with it, from "heading" to "line the next scan records". A fetcher printing `// AR-002-other: Other` under its declaration passed the old heading test, so without that move the first fetch would exit 0 and the second, over the same output, exit 2, which is what [§FS-fetch.6](../../functional-spec/FS-fetch.md#6-stability-and-ownership)'s idempotence rules out.

### decision.preserve: Rejected: keeping the nested declaration and replacing only the rest

The ticket offered this for a file home. It would leave pieces of the old body in a fetched declaration, because nothing says whose a section after the nested declaration is. Such a section can then collide with one in the output and turn a green tree red. It also cannot reach a folder home without breaking the verbatim of [§FS-fetch.5](../../functional-spec/FS-fetch.md#5-folder-home-write). A refusal can be relaxed later without breaking anyone, and the reverse cannot be done once a merge has written a file.

### decision.folder: A folder home refuses too

[§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own) is a hard requirement and says which clause gives way. Refusing keeps [§FS-fetch.5](../../functional-spec/FS-fetch.md#5-folder-home-write) true of every file fetch does write, and it is the shape the folder write already has for a file holding the requested ID beside other content.

## consequences: Consequences

The only runs whose exit moves are the ones that destroyed a declaration, and each now has a move or a delete to make before it can proceed. Not all of those declarations were written by hand: [§FS-fetch.3.1](../../functional-spec/FS-fetch.md#31-a-nested-declaration-is-refused) is unreleased, so a fetcher under grund 0.21.0 could have written one into a home, which is why the message says how to get out instead of assuming who put it there. A heading the next `check` would have called unmarked declares nothing, so it is replaced as before.

No `check` verdict moves, so the verdict routes of [§REQ-backwards-compatibility](../../requirements/REQ-backwards-compatibility.md#req-backwards-compatibility-an-upgrade-never-changes-a-verdict-quietly) do not apply; the exit of an existing command does, which [§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered) counts as user-visible, so the change carries a release note.

Left open: a requested ID that exists only nested under another declaration is not found at the home's depth, so it is inserted as a new H2. That is a duplicate the next `check` reports, and nothing is lost.

## release-note: Release note

- [§FS-fetch.6.nested](../../functional-spec/FS-fetch.md#6nested-a-declaration-inside-what-a-fetch-replaces-is-refused), [§FS-fetch.3.1](../../functional-spec/FS-fetch.md#31-a-nested-declaration-is-refused), [§DF-fetch-refuses-a-nested-declaration](DF-fetch-refuses-a-nested-declaration.md#df-fetch-refuses-a-nested-declaration-grund-fetch-refuses-to-replace-a-declaration-nested-in-the-one-it-fetches-in-both-homes): **`grund fetch` no longer deletes a declaration nested in the one it replaces.** Where the replaced bytes hold a line the next scan records as a declaration, such as a deeper heading, an indented one or `// AR-002-other: Other`, the fetch exits 2 with the tree byte-identical and says which: ``error: docs/arch.md:4: fetching `AR-001-thing` would delete `AR-002-other`, declared inside it; move `AR-002-other` to its own H2 or delete it, then fetch again``. A `folder` home says `to its own file`. Fetcher output holding such a line is refused the same way, comment-prefixed lines now included. **What does not move:** every other fetch, including a re-fetch over numbered, named or relative subsections, fenced or raw-text headings, bold labels and prose citations, and every `grund check` verdict. **Who this affects:** a script that re-fetches a declaration holding another one, which exited 0 and lost it; move the nested declaration to its own H2 or its own file, or delete it. Closes [issue #1182](https://github.com/agent-grounds/grund/issues/1182).
