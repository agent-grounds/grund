"""Each place a removed spec coordinate is written, and how the fold pass rewrites it.

The writing half of `move_spec_sections.py` (§REQ-spec-section-names.permanence): one
function per surface, each taking a file's text and returning the text the pass writes,
with every site it would only have guessed at returned as a line for a person to edit.
The anchors it repoints are the ones §FS-fmt.6.2.2 derives from a heading as written.
"""

from __future__ import annotations

import json
import re

from spec_fold_map import ID, SPEC_DIR, FoldMap, Place, Spec

TOKEN = rf"{ID}(?:\.[A-Za-z0-9_-]+)*"
# The lookbehind is the escape refusal: `<§>` is not a citation (§FS-check.2.3.1).
MARKED = re.compile(rf"(?<!<)§({TOKEN})")
BARE = re.compile(rf"(?<![§\w./-])(?<!<§>)({ID}(?:\.[A-Za-z0-9_-]+)+)")
QUOTED = re.compile(rf'"({TOKEN})"')
SELF = "https://github.com/agent-grounds/grund/blob/main/"
SPEC_LINK = re.compile(
    r"(?P<prefix>[^\s()\[\]<>\"'`]*?)(?P<name>[A-Z][A-Z0-9]*-[a-z0-9-]+\.md)#(?P<anchor>[\w%-]+)"
)
OWN_LINK = re.compile(r"\]\(#(?P<anchor>[\w%-]+)\)")
RELEASE = re.compile(r"^## (?:\d+\. )?\[\d", re.M)


def retarget_marked(text: str, fold: FoldMap) -> str:
    """Point every `§`-marked citation of a removed coordinate at its target."""
    return MARKED.sub(lambda m: "§" + (fold.lookup(m.group(1)) or m.group(1)), text)


def escape_released(text: str, fold: FoldMap) -> str:
    """Escape a removed coordinate where a released record cites it, so it keeps the coordinate
    it shipped with instead of claiming a section that never existed then."""
    return MARKED.sub(lambda m: ("<§>" if fold.lookup(m.group(1)) else "§") + m.group(1), text)


def released_part(path: str, text: str) -> int | None:
    """Where the released text of a changelog starts, or `None` when the file is no changelog.

    Every file under `docs/changelog/` is a released record. `docs/changelog.md` holds its
    conventions first, which are live text, and the releases from the first `## [x.y.z]`
    heading on, the older ones quoted under the last chapter.
    """
    if path.startswith("docs/changelog/") and path.endswith(".md"):
        return 0
    if path == "docs/changelog.md":
        release = RELEASE.search(text)
        return release.start() if release else len(text)
    return None


def rewrite_refs(text: str, fold: FoldMap) -> str:
    """Retarget an e2e case manifest, whose coordinates carry no marker, in either spelling.

    Each whitespace-separated token is looked up, so a line holding several coordinates is
    rewritten too. In a manifest the rewrite changes, a repeated line is kept once, and so is
    a token repeated on one line, because evidence is a set and the second copy says nothing.
    """
    lines, rewritten = [], False
    for line in text.split("\n"):
        tokens = line.split()
        moved = [fold.lookup(fold.canonical(token)) or token for token in tokens]
        if moved != tokens:
            rewritten = True
            line = line[: len(line) - len(line.lstrip())] + " ".join(dict.fromkeys(moved))
        lines.append(line)
    if not rewritten:
        return text
    seen: set[str] = set()
    kept = []
    for line in lines:
        if line.strip() and line.strip() in seen:
            continue
        seen.add(line.strip())
        kept.append(line)
    return "\n".join(kept)


def repoint_links(text: str, path: str, stale: dict[str, dict[str, Place | None]], hand: list[str]) -> str:
    """Point a link at a heading anchor the rewrite removed or retitled at its replacement.

    A link is this repository's spec when its path ends in `functional-spec/<page>`, written
    relative or as a `blob/main` self-link; a link pinned to a commit keeps the anchor it had
    there. A link inside a spec page may name a sibling page or its own anchors bare.
    """
    in_spec = path.startswith(SPEC_DIR + "/")

    def replace(page: str, anchor: str, line: int) -> Place | None:
        table = stale.get(page, {})
        if anchor not in table:
            return None
        if table[anchor] is None:
            hand.append(f"{path}:{line}: links to `{page}#{anchor}`, a heading the rewrite removed that was no coordinate")
        return table[anchor]

    def spec_link(match: re.Match) -> str:
        prefix = match.group("prefix")
        if "://" in prefix and not prefix.startswith(SELF):
            return match.group(0)
        if not (prefix.endswith("functional-spec/") or (in_spec and prefix in ("", "./"))):
            return match.group(0)
        line = text.count("\n", 0, match.start()) + 1
        place = replace(f"{SPEC_DIR}/{match.group('name')}", match.group("anchor"), line)
        return match.group(0) if place is None else f"{prefix}{place[0].rsplit('/', 1)[-1]}#{place[1]}"

    def own_link(match: re.Match) -> str:
        place = replace(path, match.group("anchor"), text.count("\n", 0, match.start()) + 1)
        if place is None:
            return match.group(0)
        page = "" if place[0] == path else place[0].rsplit("/", 1)[-1]
        return f"]({page}#{place[1]})"

    text = SPEC_LINK.sub(spec_link, text)
    return OWN_LINK.sub(own_link, text) if in_spec else text


def drop_repeats(line: str) -> str:
    """A line's quoted items, each kept once: a later copy goes with the comma before it."""
    seen: set[str] = set()

    def keep(match: re.Match) -> str:
        if match.group(2) in seen:
            return ""
        seen.add(match.group(2))
        return match.group(0)

    return re.sub(r'(,\s*)?"([^"]*)"', keep, line)


def retarget_quoted(line: str, fold: FoldMap) -> str:
    return QUOTED.sub(lambda m: f'"{fold.lookup(m.group(1)) or m.group(1)}"', line)


def retarget_ledger(text: str, fold: FoldMap) -> str:
    """Retarget the citation ledger's bare points, each array keeping a point once."""
    out, seen = [], set()
    for line in text.split("\n"):
        if "= [" in line:
            seen = set()
        moved = retarget_quoted(line, fold)
        line = drop_repeats(moved) if moved != line else line
        sole = re.match(r'^\s*"([^"]+)",?\s*$', line)
        if sole and sole.group(1) in seen:
            continue
        seen.update(re.findall(r'"([^"]+)"', line))
        out.append(line)
    return "\n".join(out)


def retarget_pointers(text: str, fold: FoldMap, path: str, hand: list[str]) -> str:
    """Retarget the coordinates a `POINTERS` row says its pointer cites.

    The row's own coordinate is a pointer, which a fold never removes, so a row whose
    pointer is gone is reported for the change that retires it rather than rewritten.
    """
    out = []
    lines = text.split("\n")
    # A row opens at the shallowest `("` of the file; its cited and fragment tuples sit deeper.
    indent = min((len(line) - len(line.lstrip()) for line in lines if line.lstrip().startswith('("')), default=0)
    for number, line in enumerate(lines, 1):
        head = re.match(rf'^ {{{indent}}}\("([^"]+)",', line)
        cut = head.end() if head else 0
        if head and fold.lookup(head.group(1)):
            hand.append(f"{path}:{number}: POINTERS row for the removed `{head.group(1)}`; a pointer is retired by hand")
        moved = retarget_quoted(line[cut:], fold)
        if moved != line[cut:]:
            # A cited tuple left one coordinate keeps its comma, or it becomes a string.
            moved = re.sub(r'\(("[^"]*")\)', r"(\1,)", drop_repeats(moved))
        out.append(line[:cut] + moved)
    return "\n".join(out)


CHAPTERS = re.compile(r"^CHAPTERS = \(\n(?P<body>.*?)^\)", re.M | re.S)


def retarget_chapters(text: str, fold: FoldMap, before: Spec, path: str, hand: list[str]) -> str:
    """Retarget a moves test's `CHAPTERS`, the paths its destination page must declare.

    The page is the one declaration whose base sections hold every listed path. A folded
    path leaves the tuple when its target is the page or already listed, and is otherwise
    replaced by its target's path.
    """
    block = CHAPTERS.search(text)
    if not block:
        return text
    entries = re.findall(r'"([^"]*)"', block.group("body"))
    owners = [d.id for d in before.declarations.values() if entries and all(e in d.sections for e in entries)]
    if len(owners) != 1:
        if any(fold.lookup(f"{d}.{e}") for d in before.declarations for e in entries):
            hand.append(f"{path}: CHAPTERS belongs to no one declaration ({owners}); edit it by hand")
        return text
    owner = owners[0]
    listed = {e for e in entries if not fold.lookup(f"{owner}.{e}")}

    def chapter(match: re.Match) -> str:
        target = fold.lookup(f"{owner}.{match.group(2)}")
        if target is None:
            return match.group(0)
        relative = target[len(owner) + 1:]
        if not relative or relative in listed:
            return ""
        listed.add(relative)
        return f'"{relative}"{match.group(3)}'

    body = re.sub(r'()"([^"]*)"(,?[ ]?)', chapter, block.group("body"))
    if body == block.group("body"):
        return text
    body = "".join(line.rstrip() + "\n" for line in body.splitlines() if line.strip())
    return text[: block.start("body")] + body + text[block.end("body"):]


SCHEMA_SPEC = re.compile(
    r'("x-grund-spec"\s*:\s*\{\s*"id"\s*:\s*")([^"]+)("\s*,\s*"href"\s*:\s*")([^"#]*)#([^"]*)(")'
)


def retarget_schema(text: str, fold: FoldMap, after: Spec, stale: dict, path: str, hand: list[str]) -> str:
    """Retarget the schema's `x-grund-spec` blocks: the id, and the href to the id's heading."""
    seen = []

    def block(match: re.Match) -> str:
        old = match.group(2)
        seen.append(old)
        new = fold.lookup(old) or old
        if new == old and match.group(5) not in stale.get(match.group(4), {}):
            return match.group(0)
        declaration, anchor = after.declaration_of(new), after.anchor_of(new)
        if declaration is None or anchor is None:
            hand.append(f"{path}: x-grund-spec `{old}` -> `{new}`, which has no heading to link")
            return match.group(0)
        return f"{match.group(1)}{new}{match.group(3)}{declaration.file}#{anchor}{match.group(6)}"

    text = SCHEMA_SPEC.sub(block, text)
    for identifier in schema_ids(json.loads(text)):
        if fold.lookup(identifier) and identifier not in seen:
            hand.append(f"{path}: x-grund-spec `{identifier}` is not written id-then-href; edit it by hand")
    return text


def schema_ids(node) -> list[str]:
    if isinstance(node, dict):
        spec = node.get("x-grund-spec")
        found = [spec["id"]] if isinstance(spec, dict) and "id" in spec else []
        return found + [i for value in node.values() for i in schema_ids(value)]
    if isinstance(node, list):
        return [i for value in node for i in schema_ids(value)]
    return []


ROW = re.compile(r'^(?P<indent>\s*)Exception \{ id: "(?P<id>[^"]+)", reason: "(?P<reason>(?:[^"\\]|\\.)*)" \},\s*$')


def retarget_exceptions(text: str, fold: FoldMap, after: Spec, evidence: set[str], path: str, hand: list[str]) -> str:
    """Delete the coverage exception rows of removed leaves.

    The gate holds evidence of leaves alone (§AR-goal-measurement.1), so a target the fold
    leaves a leaf with no evidence takes one row in the place of its children's, with the
    reason they gave; the row is reported, because evidence is read here by pattern, not by
    the scanner the gate runs.
    """
    out, orphaned = [], {}
    for line in text.split("\n"):
        row = ROW.match(line)
        target = fold.lookup(row.group("id")) if row else None
        if target is None:
            out.append(line)
            continue
        slot = orphaned.setdefault(target, [len(out), row.group("indent"), []])
        if row.group("reason") not in slot[2]:
            slot[2].append(row.group("reason"))
    kept = {row.group("id") for row in map(ROW.match, out) if row}
    for target, (index, indent, reasons) in sorted(orphaned.items(), key=lambda item: -item[1][0]):
        if after.is_leaf(target) and target not in evidence and target not in kept:
            out.insert(index, f'{indent}Exception {{ id: "{target}", reason: "{"; ".join(reasons)}" }},')
            hand.append(f"{path}: added the exception row of `{target}`, a leaf with no evidence; check its reason")
    for target in sorted(set(fold.targets.values()) & kept & evidence):
        hand.append(f"{path}: `{target}` keeps an exception row but now has evidence; retire the row")
    return "\n".join(out)


REFS_BYTES = re.compile(r'("spec\.refs"\s*:\s*\{\s*"bytes"\s*:\s*\[)([0-9,\s]*)(\])')


def patch_agreement(text: str, old: bytes, new: bytes) -> str | None:
    """The agreement fact with its case manifest's bytes replaced, or `None` when the fact did
    not hold the manifest's old bytes, which leaves it for a person to regenerate."""
    found = list(REFS_BYTES.finditer(text))
    if len(found) != 1:
        return None
    match = found[0]
    if bytes(int(b) for b in match.group(2).replace(" ", "").split(",") if b.strip()) != old:
        return None
    return text[: match.start(2)] + ",".join(str(b) for b in new) + text[match.end(2):]


def is_live_test_source(path: str) -> bool:
    """The files whose citations `functional_spec_coverage.rs` counts as evidence."""
    name = path.rsplit("/", 1)[-1]
    if path == "tests/integration/functional_spec_coverage.rs":
        return False
    if path.startswith("tests/integration/"):
        return True
    if path.startswith("tests/bindings/") and path.endswith(".py"):
        return name.startswith("test_")
    if not path.startswith("crates/") or not path.endswith(".rs"):
        return False
    return "/tests/" in path or ("/src/" in path and name.startswith("tests_"))


def bare_sites(path: str, text: str, fold: FoldMap) -> list[str]:
    """Each unmarked mention of a removed coordinate the pass left, one line per site."""
    sites = []
    for match in BARE.finditer(text):
        target = fold.lookup(fold.canonical(match.group(1)))
        if target:
            line = text.count("\n", 0, match.start()) + 1
            sites.append(f"{path}:{line}: `{match.group(1)}` is written bare; it is now `{target}`")
    return sites


RUN = re.compile(
    rf"(?<!<)§({ID})(?:\.[A-Za-z0-9_-]+)+"
    r"((?:(?:,|,? and|,? or| /)\s+\.[0-9a-z][A-Za-z0-9_-]*(?:\.[A-Za-z0-9_-]+)*)+)"
)
RUN_ITEM = re.compile(r"\s\.([0-9a-z][A-Za-z0-9_-]*(?:\.[A-Za-z0-9_-]+)*)")


def run_sites(path: str, text: str, fold: FoldMap) -> list[str]:
    """Each removed section named in a run after a citation, `<§>FS-x.2.2, .3.1 and .4`, where
    the items after the first are prose that no scanner reads as a citation."""
    sites = []
    for match in RUN.finditer(text):
        for item in RUN_ITEM.finditer(match.group(2)):
            coordinate = f"{match.group(1)}.{item.group(1)}"
            target = fold.lookup(coordinate)
            if target:
                line = text.count("\n", 0, match.start(2) + item.start()) + 1
                sites.append(f"{path}:{line}: `.{item.group(1)}` in a run after a citation names `{coordinate}`; it is now `{target}`")
    return sites


def relative_sites(path: str, text: str, removed: list[str]) -> list[str]:
    """Lines of a Python test that name a removed section by its path within its page alone,
    as a moves test's `MOVES` and `KEPT` rows do."""
    sites = []
    for coordinate in removed:
        page, _, relative = coordinate.partition(".")
        if f"{page}.md" not in text and f'{page}"' not in text:
            continue
        for number, line in enumerate(text.split("\n"), 1):
            if f'"{relative}"' in line:
                sites.append(f"{path}:{number}: names `{relative}` of `{page}`, a removed section")
    return sites
