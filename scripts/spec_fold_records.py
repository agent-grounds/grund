"""The rewrite records the fold pass reads, and the lines of them it refuses.

A record is the one file a rewrite of a functional-spec page writes beside it
(§REQ-spec-content.fold): one `## <name>` heading per list. The pass reads five lists and
leaves the rest for the reviewer:

- `folds` and `moves`: one `<old> -> <target>` entry per line, optionally a list item, in
  backticks, or followed by `# reason`; `none` says the list is empty. Nothing else may stand
  there, because a line the pass misreads keeps the citations of the coordinate it names.
- `pins`: the test rows a person edits, each printed as a hand edit.
- `overrides`: the single sites that go somewhere other than the map says,
  `<file>:<line>: <old> -> <new>  # reason`. A list item in another shape is printed as it
  stands; prose between them is ignored.
- `lead-promises`: `<parent>: <the promise, in a few words>`, each a parent that keeps
  children and whose lead holds a promise of its own, for the coverage gate to hold
  (§AR-goal-measurement.1). Prose between them is ignored.

Every other list, `notes` included, is for the reviewer and is not read, except that an entry
line under a heading the pass does not read is refused: it is a fold or a move filed under a
misspelled heading, and the pass would otherwise leave its citations behind.
"""

from __future__ import annotations

import re
from dataclasses import dataclass, field
from pathlib import Path

ID = r"[A-Z][A-Z0-9]*-[a-z0-9]+(?:-[a-z0-9]+)*"
COORDINATE = re.compile(rf"^{ID}(?:\.[A-Za-z0-9_-]+)*$")
MARK = r"(?:<§>|§)?"
ENTRY = re.compile(rf"^{MARK}(\S+)\s*(?:->|→)\s*{MARK}(\S+)$")
OVERRIDE = re.compile(
    rf"^(?P<file>[^\s:]+):(?P<line>\d+):\s*{MARK}(?P<old>\S+)\s*(?:->|→)\s*{MARK}(?P<new>\S+)"
    r"(?:\s+#\s*(?P<reason>.*))?$"
)
PROMISE = re.compile(rf"^`?{MARK}(?P<id>{ID}(?:\.[A-Za-z0-9_-]+)*)`?:\s*(?P<reason>.*)$")
OPENS_WITH_ID = re.compile(rf"^{MARK}{ID}\b")
LIST_ITEM = re.compile(r"^\s*(?:[-*+]|\d+[.)])\s+")
LISTS = {"folds": "fold", "moves": "move"}


@dataclass(frozen=True)
class Entry:
    source: str
    target: str
    kind: str
    where: str


@dataclass(frozen=True)
class Override:
    """One site a record sends elsewhere than the map; `file` is empty for a free-form item."""

    where: str
    text: str
    file: str = ""
    line: int = 0
    old: str = ""
    new: str = ""
    reason: str = ""


@dataclass(frozen=True)
class LeadPromise:
    id: str
    reason: str
    where: str


@dataclass
class Record:
    path: Path
    entries: list[Entry]
    hand: list[str]
    overrides: list[Override] = field(default_factory=list)
    promises: list[LeadPromise] = field(default_factory=list)


class RecordError(ValueError):
    pass


def _content(raw: str) -> str:
    """A line as the lists read it: no list marker, no backticks, no surrounding space."""
    return LIST_ITEM.sub("", raw).replace("`", "").strip()


def _entry(line: str) -> re.Match | None:
    entry = ENTRY.match(line.split("#", 1)[0].strip())
    return entry if entry and all(COORDINATE.match(part) for part in entry.groups()) else None


def read_record(path: Path) -> Record:
    """Read one record, or raise `RecordError` naming every line it refuses."""
    record = Record(path, [], [])
    refused: list[str] = []
    heading = None
    fenced = False
    for number, raw in enumerate(path.read_text(encoding="utf-8").split("\n"), 1):
        where = f"{path}:{number}"
        if raw.lstrip().startswith(("```", "~~~")):
            fenced = not fenced
            continue
        # Only a `##` heading opens a list, so a deeper one inside `folds` is a line of it.
        opened = None if fenced else re.match(r"^##\s+(\S+)\s*(.*)$", raw)
        if opened:
            heading = opened.group(1).lower().rstrip(":")
            raw = opened.group(2) if heading in LISTS else ""
        line = _content(raw)
        if not line or line.strip("().").lower() == "none":
            continue
        if heading in LISTS:
            entry = _entry(line)
            if entry is None:
                refused.append(f"{where}: under `## {heading}` only `<old> -> <target>` lines "
                               f"and `none` are read; move this to `## notes`: {raw.strip()}")
            else:
                record.entries.append(Entry(entry.group(1), entry.group(2), LISTS[heading], where))
        elif heading == "pins":
            record.hand.append(f"{where}: pins: {line}")
        elif heading == "overrides":
            if override := OVERRIDE.match(line):
                record.overrides.append(Override(
                    where, line, override.group("file"), int(override.group("line")),
                    override.group("old"), override.group("new"), (override.group("reason") or "").strip()))
            elif LIST_ITEM.match(raw):
                record.overrides.append(Override(where, line))
        elif heading == "lead-promises":
            # The reason keeps its backticks: it is written into the gate's table as it stands.
            promise = PROMISE.match(LIST_ITEM.sub("", raw).strip())
            if promise and promise.group("reason").strip():
                record.promises.append(LeadPromise(promise.group("id"), promise.group("reason").strip(), where))
            elif OPENS_WITH_ID.match(line):
                refused.append(f"{where}: under `## lead-promises` a line naming a section is "
                               f"`<parent>: <the promise>`: {raw.strip()}")
        elif heading is not None and _entry(line):
            refused.append(f"{where}: an entry under `## {heading}`, which the pass does not read; "
                           f"list it under `## folds` or `## moves`: {raw.strip()}")
    if refused:
        raise RecordError("\n".join(refused))
    return record


def read_records(paths: list[Path]) -> tuple[list[Record], list[str]]:
    """Every record that reads, and every line of every record that does not."""
    records, refused = [], []
    for path in paths:
        try:
            records.append(read_record(path))
        except RecordError as error:
            refused += str(error).split("\n")
        except OSError as error:
            refused.append(f"{path}: {error}")
    return records, refused


def applied(records: list[Record], before, after) -> set[Entry]:
    """The entries of the records an earlier pass applied: every source a record lists is gone,
    at the base and in the working tree.

    A run given the records of every rewrite so far, the applied ones with the new, reads these
    as done (§REQ-spec-content.fold): it rewrites nothing for them and checks nothing about
    them, since their sources are gone and their targets may since have folded in turn. A
    record is applied whole or not at all, so a mistyped source in a new record is still
    refused as one the base does not name.
    """
    done: set[Entry] = set()
    for record in records:
        gone = [entry for entry in record.entries
                if "." in entry.source and not before.has(entry.source) and not after.has(entry.source)]
        if record.entries and len(gone) == len(record.entries):
            done.update(gone)
    return done


def active(records: list[Record], done: set[Entry]) -> list[Record]:
    """The records an earlier pass has not applied, whose pins, overrides and promises count."""
    return [record for record in records if not set(record.entries) & done]
