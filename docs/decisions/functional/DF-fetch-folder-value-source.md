# DF-fetch-folder-value-source: folder fetch refuses an ID already declared by a value source

**Date:** 2026-11-03

## status: Accepted

## 1. Context

A folder-home fetch creates `<ID>.md` beside a JSON table that already declares
that ID, reporting success while the next check reports a duplicate and show
reports ambiguity ([issue #1139](https://github.com/agent-grounds/grund/issues/1139)).
The catalog reads the table independently of the general scan, but fetch searches
only scanned Markdown. The create branch of [§FS-fetch.5](../../functional-spec/FS-fetch.md#5-folder-home-write)
requires absence; its replace branch cannot mean replacing a multi-member value
table with Markdown.

## 2. Decision and compatibility

[§FS-fetch.5.1](../../functional-spec/FS-fetch.md#51-existing-value-source-declarations-prevent-installation)
requires operational refusal at exit 2, naming the ID and source, with no stdout
and no tree changes. A Markdown match cannot hide a matching value source. The
existing catalog decides which sources declare IDs, so the correction applies to a
recognized source of either format
([§FS-values.2.2.4](../../functional-spec/FS-values.md#224-from-grund-0220-a-yaml-source-is-reported-as-a-json-one-is-after-one-release-of-warnings)),
and it adds no data writer. A table containing only other IDs still permits snapshot
creation.

The prior verdict violated [§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own): fetch may replace
or insert only its validated named declaration. When that declaration already
exists in a value table, installing a second declaration is outside that
replacement or insertion boundary. Replacing the table instead would also erase
other members outside the requested declaration's ownership. Both prohibitions
already applied when the old verdict shipped; neither authorizes a second snapshot to make a previously
resolvable ID ambiguous.

Use [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids)'s
bounded correction route. [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) cannot fit:
a warning window would continue an unsafe installation rather than
make the reported success honor ownership.
[§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) does not fit:
no single grund command can choose whether the repository should
retain the authored table or move the fact to a fetched Markdown snapshot. The
refusal names the source and those choices. Ordinary checking and querying of an
unchanged tree keep their verdicts. This decision licenses only this folder-home
correction; it is not a licence for new refusals for unread files or unrelated findings.

## release-note: Release note

- In grund 0.22.0, `grund fetch <ID>` refuses with exit 2 when a recognized value
  source in the selected folder home, JSON or YAML, already declares the ID.
  Previously it could exit 0 and create a second declaration, breaking subsequent
  check and show. The refusal names the ID and source and leaves every file unchanged.
  Keep the fact in that value source, or move it out of the value home before fetching.
  A source in that home that cannot be read or parsed stops the fetch the same way.
  Tables that declare only other IDs and ordinary Markdown snapshot replacement keep
  working. A snapshot already beside a source that declares its ID remains the
  `duplicate`
  [§DF-yaml-value-window-closes-in-0-22](DF-yaml-value-window-closes-in-0-22.md#df-yaml-value-window-closes-in-0-22-the-yaml-value-source-warning-window-closes-in-0220)
  names, and a fetch refuses rather than repair it
  ([§FS-fetch.5.1](../../functional-spec/FS-fetch.md#51-existing-value-source-declarations-prevent-installation)).
  This verdict changes under [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids):
  the prior success violated [§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own),
  and every finding names its source location and the action to take
  ([§DF-fetch-folder-value-source](DF-fetch-folder-value-source.md#df-fetch-folder-value-source-folder-fetch-refuses-an-id-already-declared-by-a-value-source)).
