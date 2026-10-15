# DF-unread-scan-root: a configured file root whose extension is unlisted is named, not read

**Date:** 2026-10-13

## status: Accepted

## 1. Context

A `[[kinds]]` `file` home, a version-2 `files` entry, or an `[scan] include` entry naming a file is a scan root ([§FS-scan.scope.roots](../../functional-spec/FS-scan.md#scoperoots-a-scan-root-outruns-every-rule-about-descent)). When its extension was one the extension list leaves out, the scan skipped it, and `grund check` printed `success`:

```console
$ grund check                 # [[kinds]] file = "notes.txt", [scan] extensions = ["md"]
success                       # notes.txt holds a dangling citation
$ grund list --kind TXT
$
```

The specification said two things about it. [§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded) listed non-scanned extensions among the rules that "prune a descendant and never a root", and the second paragraph of [§AR-scanner.1.6](../../architecture/AR-scanner.md#16-the-roots-the-walk-starts-from) said the filter "never prunes a walk root". Its first paragraph said the extension filter applies "identically to every root", and the code followed that. Either way, [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips) was broken: an unread file never looked like anything but a clean run. Reported in [issue #654](https://github.com/agent-grounds/grund/issues/654).

## 2. Decision

The file stays unread, and `check` names it. Each such root earns one located `unread-scan-root` warning at the `grund.toml` line of the row that made it a root, naming the row, the file and the extension, and ending in the edit that makes the row say what it means ([§FS-scan.checks.unread-scan-root](../../functional-spec/FS-scan.md#checksunread-scan-root-a-configured-file-root-is-not-read)). The exit code does not move and `success` is withheld.

The rule this settles: a scan root outruns every rule about *descent* — `exclude`, ignore files, hidden directories — and no rule about the file's *own name*. The hidden-file rule was already one of those ([§FS-scan.scope.hidden](../../functional-spec/FS-scan.md#scopehidden-a-hidden-file-is-not-read-and-the-rule-is-not-about-descent)); the extension list is the second ([§FS-scan.scope.extensions.root](../../functional-spec/FS-scan.md#scopeextensionsroot-a-file-root-whose-extension-is-unlisted-is-not-read-and-check-says-so)). The REQ bullet and the AR point are corrected to say so.

### 2.1 `check` alone says it

`list`, `refs`, `show`, `cover` and `fmt` stay as they are, by the author's ruling on the plan. The fact is about the coverage of `check`'s verdict, the choice [§FS-check.4.9.4](../../functional-spec/FS-check.md#494-a-located-finding-on-stdout-not-a-cli-level-caution) makes for an absent optional member, so `list --kind TXT` stays empty and prints nothing about the root.

### 2.2 A hidden-named file home is decided apart

This decision left a `file` home whose name is hidden out, because the hidden rule is asked first and the root stayed silent. That silence broke [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips) the same way, and [§DF-unread-hidden-root](DF-unread-hidden-root.md#df-unread-hidden-root-a-configured-file-root-with-a-hidden-name-is-named-not-read) closes it with this same warning ([issue #689](https://github.com/agent-grounds/grund/issues/689)).

## 3. Consequences

- A repository whose configuration names such a file sees one warning per file in place of `success`, exit still `0`. A script that greps for `success` notices.
- A non-citable row that names a file only as a place in the Project map, such as `file = "LICENSE"`, warns until it gains `scan = false`; its message offers that key. That is the intended outcome: the row then says what it means.
- Nothing new is read anywhere, so no repository gains findings inside a file.

**Which clause licenses the move.** A new warning moves a verdict ([§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered)). [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids) licenses it, because the old silence violated the separately declared prohibition of [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips), which already applied when the verdict shipped. The [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) deprecation path cannot apply, because there is no old form to carry beside a new one: the file was never read, so there is nothing to warn about first. The [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) mechanical-migration route has no command that completes the change, because the edit is the repository's choice among the ones the message names. The correcting release names the verdict change, and every finding that replaces the old verdict names its location, the row's `grund.toml` line, and the action the maintainer can take. This is the bounded correction route and not a licence for ordinary policy tightening, feature removal, or a newly invented prohibition.

## 4. Alternatives considered

| Approach | Why rejected |
|---|---|
| Read a configured file root whatever its extension | What [§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded) literally said, but an unlisted extension has no comment prefixes to read it with ([§FS-languages.prefixes](../../functional-spec/FS-languages.md#prefixes-comment_prefixes-compose-with-extensions)); the one read predicate would answer two ways, by root and by descent, for stubs, buffers, the editor and `fmt`; and it would start parsing files configured only as places, such as a `LICENSE` row. Widening what is read is a configuration decision, and the warning names it. |
| Make it an error | Nothing the run read is wrong. Unread content is a statement about coverage, like the empty-scan caution and the absent optional member ([§FS-check.2.2](../../functional-spec/FS-check.md#22-empty-scan), [§FS-check.4.9](../../functional-spec/FS-check.md#49-a-workspace-member-declared-optional-is-absent)); [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips) asks that it never look clean, not that it fail. |
| Print it on stderr from every scanning command | Declined by the author's ruling ([§DF-unread-scan-root.2.1](DF-unread-scan-root.md#21-check-alone-says-it)). |

## release-note: Release note

- [§FS-scan.checks.unread-scan-root](../../functional-spec/FS-scan.md#checksunread-scan-root-a-configured-file-root-is-not-read), [§FS-scan.scope.extensions.root](../../functional-spec/FS-scan.md#scopeextensionsroot-a-file-root-whose-extension-is-unlisted-is-not-read-and-check-says-so): **`check` now warns, exit unchanged, where a configured `file` home or `include` file has an extension `[scan] extensions` does not list.** Each such file earns one `unread-scan-root` warning at its row's `grund.toml` line in place of `success`. **Who this breaks:** a script that greps for `success`, and a repository with such a row; add the extension, rename the file, or set `scan = false` on a non-citable row that only lists a place. Nothing new is read. The verdict changes by the correction route of [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), because the old silence broke [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips), and every finding that replaces it names its location and the fix. Why it is a warning and not a wider read is [§DF-unread-scan-root](DF-unread-scan-root.md#df-unread-scan-root-a-configured-file-root-whose-extension-is-unlisted-is-named-not-read). Closes [issue #654](https://github.com/agent-grounds/grund/issues/654).
