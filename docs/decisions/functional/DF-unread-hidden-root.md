# DF-unread-hidden-root: a configured file root with a hidden name is named, not read

**Status:** Accepted
**Date:** 2026-10-13

## 1. Context

A `[[kinds]]` `file` home, a version-2 `files` entry, or an `[scan] include` entry naming a file is a scan root ([§FS-scan.scope.roots](../../functional-spec/FS-scan.md#scoperoots-a-scan-root-outruns-every-rule-about-descent)). When the file's own name began with `.`, the hidden-file rule refused it, which is the declared blind spot of [§FS-scan.scope.hidden](../../functional-spec/FS-scan.md#scopehidden-a-hidden-file-is-not-read-and-the-rule-is-not-about-descent). `grund check` printed `success` over it all the same:

```console
$ grund check                 # [[kinds]] NOTE file = ".notes.md"
success                       # .notes.md holds a dangling citation
$ grund list --kind NOTE
$
```

[§DF-unread-scan-root](DF-unread-scan-root.md#df-unread-scan-root-a-configured-file-root-whose-extension-is-unlisted-is-named-not-read) had just named a file root refused for its extension, and left this case out on purpose: the hidden rule is asked first, so a hidden root stayed silent. Not reading the file was specified. Saying `success` about it broke [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips): a run that reads none of a file the configuration names must never look like a clean one. Reported in [issue #689](https://github.com/agent-grounds/grund/issues/689).

## 2. Decision

The file stays unread, and `check` names it with the same `unread-scan-root` warning, carrying a second reason ([§FS-scan.checks.unread-scan-root](../../functional-spec/FS-scan.md#checksunread-scan-root-a-configured-file-root-is-not-read)): one located warning at the `grund.toml` line of the row that made it a root, naming the row and the file, saying its name begins with `.`, and ending in the rename that fixes it ([§FS-scan.checks.unread-scan-root.message](../../functional-spec/FS-scan.md#checksunread-scan-rootmessage-the-message)). The exit code does not move and `success` is withheld.

It is the same finding rather than a sibling code because it is the same fact (a file the configuration names was not read), at the same place, on the same channel, with the same shape of fix. A second code would split one question in two, and `--ignore unread-scan-root` would silence only half of the unread roots.

### 2.1 One warning names both rules

A root that is hidden and whose extension the list also refuses, such as `.notes.txt` with `extensions = ["md"]`, earns **one** warning, and its reason names both rules. Naming one of two true causes sends the reader to an edit that does not finish the job: renaming the file alone leaves the extension refusal standing, and listing `txt` alone changes nothing. The `grund check docs/.notes.txt` caution already settled this for a path typed on the command line ([§FS-check.2.2.2.3](../../functional-spec/FS-check.md#2223-a-hidden-file)), so the root gate asks the same extension test it asks and records both answers.

### 2.2 What it leaves as it was

The hidden rule itself does not change: no file whose own name begins with `.` is read, by any run, `--full` included. A hidden file inside a directory root stays silent, because nothing in the configuration named it. A hidden home that does not exist, a `scan = false` row and a value row's JSON home stay silent too ([§FS-scan.checks.unread-scan-root.silent](../../functional-spec/FS-scan.md#checksunread-scan-rootsilent-what-stays-silent)). A path typed on the command line keeps its stderr caution, because it has no row behind it. `list`, `refs`, `show`, `cover` and `fmt` stay silent, as [§DF-unread-scan-root.2.1](DF-unread-scan-root.md#21-check-alone-says-it) ruled.

## 3. Consequences

- A repository whose configuration names a hidden file as a `file` home, an `include` entry or a version-2 `files` entry sees one warning per such file in place of `success`, exit still `0`. A script that greps for `success` notices.
- A non-citable row that names a dotfile only as a place in the Project map, such as `file = ".pre-commit-config.yaml"`, warns until it gains `scan = false`; its message offers that key.
- Nothing new is read anywhere, so no repository gains findings inside a file.

**Which clause licenses the move.** A new warning moves a verdict ([§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered)). [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids) licenses it, because the old silence violated the separately declared prohibition of [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips), which already applied when the verdict shipped. There is no old form to carry beside a new one, so the [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) deprecation path cannot apply, and no command completes the change, because the edit is the repository's choice among the ones the message names. Every finding that replaces the old verdict names its location, the row's `grund.toml` line, and the action the maintainer can take. This is the same reasoning [§DF-unread-scan-root](DF-unread-scan-root.md#df-unread-scan-root-a-configured-file-root-whose-extension-is-unlisted-is-named-not-read) used, and it is a record of its own because a record carries one release-note bullet, and a note already present at a tag is never published again ([§FS-distribution.4.6.4](../../functional-spec/FS-distribution.md#464-compatibility-notices-come-from-the-decisions)).

## 4. Alternatives considered

| Approach | Why rejected |
|---|---|
| Read a hidden file when a row names it | [§FS-scan.scope.hidden](../../functional-spec/FS-scan.md#scopehidden-a-hidden-file-is-not-read-and-the-rule-is-not-about-descent) makes the name the rule so that a dotfile is never read, and editors, `fmt` and stub targets all read that one answer; a root that outran it would make the read predicate answer two ways. The warning names the rename instead. |
| A new `unread-hidden-root` code | One fact under two codes: `--ignore unread-scan-root` would silence only half of the unread roots, for no difference in place, channel or fix. |
| Two warnings for a root both hidden and unlisted | Each would name one cause and offer an edit that does not finish the job ([§DF-unread-hidden-root.2.1](DF-unread-hidden-root.md#21-one-warning-names-both-rules)). |

## release-note: Release note

- [§FS-scan.checks.unread-scan-root](../../functional-spec/FS-scan.md#checksunread-scan-root-a-configured-file-root-is-not-read), [§FS-scan.scope.hidden](../../functional-spec/FS-scan.md#scopehidden-a-hidden-file-is-not-read-and-the-rule-is-not-about-descent): **`check` now warns, exit unchanged, where a configured `file` home, version-2 `files` entry or `include` file has a hidden name.** Each such file earns one `unread-scan-root` warning at its row's `grund.toml` line in place of `success`. **Who this breaks:** a script that greps for `success`, and a repository with such a row; rename the file to a name that is not hidden, drop the `include` entry, or set `scan = false` on a non-citable row that only lists a place. The file is still not read. The verdict changes by the correction route of [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), because the old silence broke [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips), and every finding that replaces it names its location and the fix. Why it is a warning and not a read is [§DF-unread-hidden-root](DF-unread-hidden-root.md#df-unread-hidden-root-a-configured-file-root-with-a-hidden-name-is-named-not-read). Closes [issue #689](https://github.com/agent-grounds/grund/issues/689).
