"""What the fold pass checks in the lines it writes: §REQ-spec-content.fold.

The retarget is one token at a time, so it cannot see what it does to a line or a section as
a whole. Four things follow from that, and each is answered here, line by line:

- a fenced block outside the spec is a verbatim quote (a commit subject in a release record, a
  sample output in a user guide) and holds no citation (§FS-citations.recognized.contexts),
  so the pass rewrites nothing in one and names each removed coordinate it quotes instead;
- two citations that the retarget makes the same, side by side (`§A, §A`), keep one, where
  the line is prose the pass changed; a repeat it cannot collapse is named instead;
- a section whose own text now cites its own coordinate cites nothing, and is named;
- a Rust file whose code and whose `#[cfg(test)]` code both changed is named with the lines
  on each side, so that a change keeping code and tests in separate commits can split it.
"""

from __future__ import annotations

import difflib
import re

from spec_fold_map import SPEC_DIR, FoldMap, fenced, owners
from spec_fold_sites import MARKED, TOKEN, repoint_links

MARKDOWN = (".md", ".markdown", ".mdx")
COMMENT = re.compile(r"^\s*(?:///|//!|//|#|\*)\s?")
# Masked in a fenced line, the marker and the anchor sign match no surface's pattern.
MASKS = {"§": "\ue000", "#": "\ue001"}
CITE = re.compile(rf"\[§(?P<link>{TOKEN})\]\([^)\s]*\)|(?<!<)§(?P<bare>{TOKEN})")
SEPARATOR = re.compile(r"^\s*(?:,|;|/|(?:,\s*)?(?:and|or))?\s*$")
CFG_TEST = re.compile(r"^(\s*)#\[cfg\(test\)\]\s*$")


def quoted(path: str, text: str) -> list[bool] | None:
    """Each line of a file outside the spec that a fence quotes, or `None` when none is."""
    if path.startswith(SPEC_DIR + "/"):
        return None
    lines = text.split("\n")
    flags = fenced(lines) if path.endswith(MARKDOWN) else fenced(lines, COMMENT)
    return flags if any(flags) else None


def mask(text: str, flags: list[bool]) -> str | None:
    """The text with its quoted lines masked, or `None` when it holds a mask character already."""
    if any(character in text for character in MASKS.values()):
        return None
    lines = text.split("\n")
    for index, flag in enumerate(flags):
        if flag:
            for plain, masked in MASKS.items():
                lines[index] = lines[index].replace(plain, masked)
    return "\n".join(lines)


def unmask(text: str) -> str:
    for plain, masked in MASKS.items():
        text = text.replace(masked, plain)
    return text


def quoted_sites(path: str, text: str, flags: list[bool], fold: FoldMap, stale: dict) -> list[str]:
    """Each removed coordinate, or link to a removed heading, that a quoted line holds."""
    found = []
    for number, (line, flag) in enumerate(zip(text.split("\n"), flags), 1):
        if not flag:
            continue
        for match in MARKED.finditer(line):
            if target := fold.lookup(match.group(1)):
                found.append(f"{path}:{number}: `{match.group(0)}` is quoted in a fenced block, "
                             f"so the pass left it; it is now `{target}`")
        if repoint_links(line, path, stale, []) != line:
            found.append(f"{path}:{number}: a link in a fenced block names a heading the rewrite removed")
    return found


def cites(line: str) -> list[tuple[int, int, str]]:
    return [(m.start(), m.end(), m.group("link") or m.group("bare")) for m in CITE.finditer(line)]


def in_string(line: str, at: int) -> bool:
    return len(re.findall(r'(?<!\\)"', line[:at])) % 2 == 1


def collapse(path: str, before: str, after: str) -> tuple[str, list[str], list[str]]:
    """Keep once each citation the pass made repeat side by side on a line it changed.

    Returns the text, a line for each repeat collapsed, and a line for each it left: one
    inside a string literal of code, and one split across a line break. A pair counts only
    when it named two coordinates before the pass, so a repeat the author wrote stays.
    """
    olds, news = before.split("\n"), after.split("\n")
    if len(olds) != len(news):
        return after, [], []
    flags = fenced(news) if path.endswith(MARKDOWN) else [False] * len(news)
    collapsed, left = [], []
    for index, (old, new) in enumerate(zip(olds, news)):
        if old == new or flags[index]:
            continue
        was = [coordinate for _, _, coordinate in cites(old)]
        while len(was) == len(now := cites(new)):
            pair = next((i for i in range(1, len(now)) if now[i][2] == now[i - 1][2]
                         and was[i] != was[i - 1] and SEPARATOR.match(new[now[i - 1][1]:now[i][0]])), None)
            if pair is None:
                break
            (start, end, coordinate), (_, stop, _) = now[pair - 1], now[pair]
            if not path.endswith(MARKDOWN) and in_string(new, start):
                left.append(f"{path}:{index + 1}: `{new[start:stop]}` repeats inside a string; keep it once by hand")
                break
            collapsed.append(f"{path}:{index + 1}: `{new[start:stop]}` kept once")
            new = new[:end] + new[stop:]
            del was[pair]
        news[index] = new
    for index in range(len(news) - 1):
        if (olds[index] == news[index] and olds[index + 1] == news[index + 1]) or flags[index] or flags[index + 1]:
            continue
        first, second = cites(news[index]), cites(news[index + 1])
        was_first, was_second = cites(olds[index]), cites(olds[index + 1])
        if not (first and second and was_first and was_second) or first[-1][2] != second[0][2]:
            continue
        head = news[index + 1][:second[0][0]]
        head = head if path.endswith(MARKDOWN) else COMMENT.sub("", head, count=1)
        if was_first[-1][2] != was_second[0][2] and SEPARATOR.match(news[index][first[-1][1]:] + " " + head):
            left.append(f"{path}:{index + 2}: `§{first[-1][2]}` ends line {index + 1} and opens this one; "
                        "keep it once by hand")
    return "\n".join(news), collapsed, left


def self_citations(path: str, before: str, after: str) -> list[str]:
    """Each line of a spec page where the pass made a section cite its own coordinate."""
    olds, news = before.split("\n"), after.split("\n")
    if not path.startswith(SPEC_DIR + "/") or len(olds) != len(news):
        return []
    owner, flags = owners(after, path), fenced(news)
    found = []
    for index, (old, new) in enumerate(zip(olds, news)):
        own = owner[index]
        if old == new or flags[index] or own is None:
            continue
        mine = [m.group(1) for m in MARKED.finditer(new)].count(own)
        if mine > [m.group(1) for m in MARKED.finditer(old)].count(own):
            found.append(f"{path}:{index + 1}: `{own}` now cites itself, from its own text; "
                         "delete or reword the citation")
    return found


def is_test_file(path: str) -> bool:
    name = path.rsplit("/", 1)[-1]
    return (path.startswith("tests/") or "/tests/" in path or name.startswith("tests_")
            or name in ("tests.rs", "testing.rs"))


def test_spans(lines: list[str]) -> list[tuple[int, int]]:
    """The line ranges, ends included, of the `#[cfg(test)]` items of a rustfmt'd file."""
    spans, index = [], 0
    while index < len(lines):
        attribute = CFG_TEST.match(lines[index])
        if attribute is None:
            index += 1
            continue
        indent, item = attribute.group(1), index + 1
        while item < len(lines) and lines[item].startswith(indent + "#["):
            item += 1
        end = item
        if item < len(lines) and not lines[item].rstrip().endswith(";"):
            while end < len(lines) and lines[end].rstrip() not in (indent + "}", indent + "};"):
                end += 1
        spans.append((index, end))
        index = end + 1
    return spans


def spans_text(lines: list[int]) -> str:
    runs: list[list[int]] = []
    for line in lines:
        if runs and line == runs[-1][1] + 1:
            runs[-1][1] = line
        else:
            runs.append([line, line])
    return ", ".join(str(a) if a == b else f"{a}-{b}" for a, b in runs)


def mixed(path: str, before: str, after: str) -> str | None:
    """A line naming the hunks on each side, when the pass changed both the code of a Rust file
    and the `#[cfg(test)]` code in it."""
    if not path.endswith(".rs") or is_test_file(path):
        return None
    olds, news = before.split("\n"), after.split("\n")
    spans = test_spans(news)
    if not spans:
        return None
    code, test = [], []
    for tag, _, _, start, stop in difflib.SequenceMatcher(None, olds, news, autojunk=False).get_opcodes():
        if tag == "equal":
            continue
        for line in range(start, max(stop, start + 1)):
            (test if any(a <= line <= b for a, b in spans) else code).append(line + 1)
    if not (code and test):
        return None
    return f"{path}: code lines {spans_text(code)}; #[cfg(test)] lines {spans_text(test)}"
