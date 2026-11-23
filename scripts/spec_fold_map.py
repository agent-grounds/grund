"""What a rewrite of the functional spec removed, and where each removed coordinate now points.

The reading half of `move_spec_sections.py` (§REQ-spec-section-names.permanence): the
records a rewrite writes, the section map of every declaration before and after it, and
the one coordinate each removed one is retargeted to. The section map is read the way
`grund <ID> --toc` reads it — ATX headings outside fences, with numbered, named and
relative paths, and each heading's GitHub anchor — so the pass needs no `grund` on the
path, and the map at a base commit is read with `git show`.
"""

from __future__ import annotations

import re
import subprocess
from dataclasses import dataclass, field
from pathlib import Path


SPEC_DIR = "docs/functional-spec"
ID = r"[A-Z][A-Z0-9]*-[a-z0-9]+(?:-[a-z0-9]+)*"
COORDINATE = re.compile(rf"^{ID}(?:\.[A-Za-z0-9_-]+)*$")
# The retired spelling an e2e manifest may still carry: `FS-002-show.2.1` is `FS-show.2.1`.
LEGACY = re.compile(r"^([A-Z][A-Z0-9]*)-\d+-")

FENCE = re.compile(r"^ {0,3}(`{3,}|~{3,})(.*)$")
ATX = re.compile(r"^ {0,3}(#{1,6})(?:[ \t]+(.*?))?[ \t]*$")
CLOSING = re.compile(r"(?:^|[ \t]+)#+[ \t]*$")
DECLARATION = re.compile(rf"^({ID}):(?:\s|$)")
COMPONENT = r"(?:\d+|[a-z][a-z0-9-]*)"
NUMBERED = re.compile(r"^(\d+(?:\.\d+)*)\.?$")
NAMED = re.compile(rf"^({COMPONENT}(?:\.{COMPONENT})*):$")
RELATIVE = re.compile(rf"^\.(\d+)\.?$|^\.({COMPONENT}):$")
LINK_TEXT = re.compile(r"!?\[([^\]]*)\]\([^)]*\)")


@dataclass
class Section:
    path: str
    title: str
    anchor: str


@dataclass
class Declaration:
    id: str
    file: str
    anchor: str
    sections: dict[str, Section] = field(default_factory=dict)


@dataclass
class Spec:
    """Every declaration of the spec directory, and every heading anchor of each file."""

    declarations: dict[str, Declaration]
    anchors: dict[str, set[str]]

    def declaration_of(self, coordinate: str) -> Declaration | None:
        return self.declarations.get(coordinate.split(".", 1)[0])

    def has(self, coordinate: str) -> bool:
        declaration = self.declaration_of(coordinate)
        identifier, _, path = coordinate.partition(".")
        return declaration is not None and (not path or path in declaration.sections)

    def anchor_of(self, coordinate: str) -> str | None:
        declaration = self.declaration_of(coordinate)
        if declaration is None:
            return None
        _, _, path = coordinate.partition(".")
        if not path:
            return declaration.anchor
        section = declaration.sections.get(path)
        return section.anchor if section else None

    def is_leaf(self, coordinate: str) -> bool:
        declaration = self.declaration_of(coordinate)
        _, _, path = coordinate.partition(".")
        if declaration is None or not path:
            return False
        return not any(other.startswith(path + ".") for other in declaration.sections)

    def coordinates(self, identifier: str) -> dict[str, Section]:
        declaration = self.declarations.get(identifier)
        if declaration is None:
            return {}
        return {f"{identifier}.{path}": section for path, section in declaration.sections.items()}


def slug(text: str) -> str:
    """GitHub's anchor for a heading's text, before a repeat takes a `-<n>` suffix."""
    text = LINK_TEXT.sub(r"\1", text).lower()
    return re.sub(r"[^\w\- ]", "", text).replace(" ", "-")


def headings(text: str):
    """Each ATX heading outside a fence, as (level, text)."""
    fence = None
    for line in text.split("\n"):
        opener = FENCE.match(line)
        if fence:
            closes = opener and opener.group(1)[0] == fence[0] and len(opener.group(1)) >= len(fence)
            if closes and not opener.group(2).strip():
                fence = None
            continue
        if opener and not (opener.group(1)[0] == "`" and "`" in opener.group(2)):
            fence = opener.group(1)
            continue
        heading = ATX.match(line)
        if heading:
            yield len(heading.group(1)), CLOSING.sub("", heading.group(2) or "")


def section_path(token: str, parent: str | None) -> str | None:
    """The canonical path a heading's first token declares, or `None` for a plain heading."""
    if numbered := NUMBERED.match(token):
        return numbered.group(1)
    if (named := NAMED.match(token)) and not NUMBERED.match(token[:-1]):
        return named.group(1)
    relative = RELATIVE.match(token)
    if relative is None or parent is None:
        return None
    path = (parent + "." if parent else "") + (relative.group(1) or relative.group(2))
    named_form = token.endswith(":")
    return None if named_form and all(part.isdigit() for part in path.split(".")) else path


def parse(text: str, file: str) -> tuple[dict[str, Declaration], set[str]]:
    """The declarations a Markdown file holds, and the anchor of every heading in it."""
    declarations: dict[str, Declaration] = {}
    used: set[str] = set()
    current: tuple[int, Declaration] | None = None
    stack: list[tuple[int, str | None]] = []
    for level, inner in headings(text):
        base = anchor = slug(inner)
        repeat = 0
        while anchor in used:
            repeat += 1
            anchor = f"{base}-{repeat}"
        used.add(anchor)
        if current and level <= current[0]:
            current = None
        if declared := DECLARATION.match(inner):
            current = (level, Declaration(declared.group(1), file, anchor))
            declarations.setdefault(declared.group(1), current[1])
            stack = [(level, "")]
            continue
        if current is None:
            continue
        while stack and stack[-1][0] >= level:
            stack.pop()
        parent = stack[-1][1] if stack and stack[-1][0] == level - 1 else None
        token, _, title = inner.partition(" ")
        path = section_path(token, parent)
        stack.append((level, path))
        if path is not None:
            current[1].sections.setdefault(path, Section(path, title.strip(), anchor))
    return declarations, used


def read_spec(files: dict[str, str]) -> Spec:
    declarations: dict[str, Declaration] = {}
    anchors: dict[str, set[str]] = {}
    for name, text in sorted(files.items()):
        found, anchors[name] = parse(text, name)
        for identifier, declaration in found.items():
            declarations.setdefault(identifier, declaration)
    return Spec(declarations, anchors)


def git(root: Path, *arguments: str) -> str:
    return subprocess.run(
        ["git", "-C", str(root), *arguments], capture_output=True, check=True, text=True
    ).stdout


def spec_at(root: Path, base: str) -> Spec:
    """The spec directory as the base commit holds it."""
    names = git(root, "ls-tree", "-r", "--name-only", base, "--", SPEC_DIR).split("\n")
    pages = [name for name in names if name.endswith(".md")]
    return read_spec({name: git(root, "show", f"{base}:{name}") for name in pages})


def spec_in(root: Path) -> Spec:
    """The spec directory as the working tree holds it."""
    pages = sorted((root / SPEC_DIR).glob("*.md"))
    return read_spec({f"{SPEC_DIR}/{page.name}": page.read_text(encoding="utf-8") for page in pages})


@dataclass(frozen=True)
class Entry:
    source: str
    target: str
    kind: str
    where: str


@dataclass
class Record:
    path: Path
    entries: list[Entry]
    hand: list[str]


class RecordError(ValueError):
    pass


ENTRY = re.compile(r"^(?:<§>|§)?(\S+)\s*(?:->|→)\s*(?:<§>|§)?(\S+)$")
LISTS = {"folds": "fold", "moves": "move"}
HAND = ("pins",)


def read_record(path: Path) -> Record:
    """The `folds` and `moves` of one rewrite record, and its `pins` to edit by hand.

    A record is a Markdown file with one `## <name>` heading per list. Under `## folds` and
    `## moves`, a line is `<old> -> <target>`, optionally a list item, in backticks, or
    followed by `# reason`; `none` says the list is empty. Any other line in those two lists
    is refused rather than guessed at, because a coordinate the pass misreads keeps its
    citations. Every other list but `pins`, which names the test rows a person edits, is
    for the reviewer and is not read.
    """
    entries: list[Entry] = []
    hand: list[str] = []
    heading = None
    fenced = False
    for number, raw in enumerate(path.read_text(encoding="utf-8").split("\n"), 1):
        where = f"{path}:{number}"
        if raw.lstrip().startswith(("```", "~~~")):
            fenced = not fenced
            continue
        opened = None if fenced else re.match(r"^#{2,6}\s+(\S+)\s*(.*)$", raw)
        if opened:
            heading = opened.group(1).lower().rstrip(":")
            raw = opened.group(2) if heading in LISTS else ""
        line = re.sub(r"^\s*(?:[-*+]|\d+[.)])\s+", "", raw).replace("`", "").strip()
        if not line or line.strip("().").lower() == "none":
            continue
        if heading in HAND:
            hand.append(f"{where}: {heading}: {line}")
        if heading not in LISTS:
            continue
        entry = ENTRY.match(line.split("#", 1)[0].strip())
        if not entry or not all(COORDINATE.match(part) for part in entry.groups()):
            raise RecordError(f"{where}: not `<old> -> <target>` in `{heading}`: {raw.strip()}")
        entries.append(Entry(entry.group(1), entry.group(2), LISTS[heading], where))
    return Record(path, entries, hand)


class FoldMap:
    """Each removed coordinate and the one it is retargeted to.

    A coordinate is looked up by its longest listed prefix, whole components only, so
    `FS-x.2.1` never matches inside `FS-x.2.10`. Below a listed source, an unlisted
    sub-point either goes with it, its tail kept (`keep_suffix`, a move of a subtree), or
    takes the target itself (a fold, whose ancestor has no such sub-point).
    """

    def __init__(self, targets: dict[str, str], known: set[str] = frozenset(), keep_suffix=False):
        self.targets = targets
        self.known = set(known) | {source.split(".", 1)[0] for source in targets}
        self.keep_suffix = keep_suffix
        self.inherited: dict[str, str] = {}

    def canonical(self, token: str) -> str:
        """`FS-002-show.2.1` as `FS-show.2.1`, when only the second names a declaration."""
        identifier = token.split(".", 1)[0]
        modern = LEGACY.sub(r"\1-", token)
        if modern != token and identifier not in self.known and modern.split(".", 1)[0] in self.known:
            return modern
        return token

    def lookup(self, coordinate: str) -> str | None:
        parts = coordinate.split(".")
        for length in range(len(parts), 1, -1):
            source = ".".join(parts[:length])
            if source in self.targets:
                tail = parts[length:]
                if tail and not self.keep_suffix:
                    self.inherited[coordinate] = source
                return ".".join([self.targets[source], *tail] if self.keep_suffix else [self.targets[source]])
        return None


def fold_map(records: list[Record]) -> tuple[FoldMap, list[str]]:
    targets: dict[str, str] = {}
    problems = []
    for entry in (entry for record in records for entry in record.entries):
        if targets.setdefault(entry.source, entry.target) != entry.target:
            problems.append(f"{entry.where}: {entry.source} has two targets, {targets[entry.source]} and {entry.target}")
    return FoldMap(targets), problems


def is_within(target: str, source: str) -> bool:
    return source.startswith(target + ".")


def problems(records: list[Record], before: Spec, after: Spec) -> list[str]:
    """What makes the records unsafe to apply: each names the line to fix."""
    _, found = fold_map(records)
    sources = {entry.source for record in records for entry in record.entries}
    for entry in (entry for record in records for entry in record.entries):
        said = f"{entry.where}: {entry.source} -> {entry.target}"
        if "." not in entry.source:
            found.append(f"{said}: a whole declaration is not folded")
        elif not before.has(entry.source):
            found.append(f"{said}: the base names no such section")
        elif after.has(entry.source):
            found.append(f"{said}: the section is still there; a surviving section keeps its coordinate")
        if entry.kind == "fold" and not is_within(entry.target, entry.source):
            found.append(f"{said}: a fold names its nearest surviving ancestor; list a non-ancestor under moves")
        if entry.target in sources:
            found.append(f"{said}: the target is itself removed; give the final coordinate")
        elif not after.has(entry.target):
            found.append(f"{said}: the working tree has no such target")
    return found


def compare(before: Spec, after: Spec, fold: FoldMap) -> tuple[list[str], list[str]]:
    """Removed coordinates no record lists, and notes on what else the section maps show."""
    missing, notes = [], []
    named = {source.split(".", 1)[0] for source in fold.targets}
    for identifier in sorted(set(before.declarations) | set(after.declarations)):
        old, new = before.coordinates(identifier), after.coordinates(identifier)
        if identifier not in named and {c: s.title for c, s in old.items()} == {c: s.title for c, s in new.items()}:
            continue
        if identifier not in after.declarations:
            missing.append(f"{identifier}: the whole declaration is gone")
            continue
        missing += [f"{c}: removed, and in no record" for c in old if c not in new and c not in fold.targets]
        notes += [f"{c}: added" for c in new if c not in old]
        notes += [f"{c}: retitled, `{old[c].title}` -> `{new[c].title}`"
                  for c in old if c in new and old[c].title != new[c].title]
    return missing, notes


Place = tuple[str, str]


def stale_anchors(before: Spec, after: Spec, fold: FoldMap) -> dict[str, dict[str, Place | None]]:
    """For each spec file, every anchor the base had and the tree lost, and its replacement.

    The replacement is the page and anchor of the coordinate the old heading's coordinate
    now names, its retarget when it was removed and itself when it was retitled; a move may
    land on another page. `None` when the old heading was no coordinate, so a link to it is
    reported rather than guessed at.
    """
    stale: dict[str, dict[str, Place | None]] = {}
    for declaration in before.declarations.values():
        current = after.anchors.get(declaration.file, set())
        old = {declaration.anchor: declaration.id}
        old.update({section.anchor: f"{declaration.id}.{path}" for path, section in declaration.sections.items()})
        for anchor, coordinate in old.items():
            if anchor not in current:
                now = fold.lookup(coordinate) or coordinate
                home, replacement = after.declaration_of(now), after.anchor_of(now)
                place = (home.file, replacement) if home and replacement else None
                stale.setdefault(declaration.file, {})[anchor] = place
    for file, anchors in before.anchors.items():
        for anchor in anchors - after.anchors.get(file, set()):
            stale.setdefault(file, {}).setdefault(anchor, None)
    return stale
