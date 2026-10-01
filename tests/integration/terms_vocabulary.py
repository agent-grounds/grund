"""§FS-terms.senses — one parse of a Terms home, for the two checks §FS-terms.senses.5
keeps apart: the four fixed-syntax invariants that gate, and the advisory report that
never may. Both read the same shared groups, the same lean lines and the same prose,
so the gate and the advice can disagree about a document only where the specification
says they should.

A home is a directory holding a `README.md` whose marked entries name the documents
and an `FS-terms.md` that holds the shared groups. `docs/functional-spec` is the home
this repository checks; the tests build fixture homes of their own, so nothing here
reads a path it was not handed.

Beside `test_functional_spec_terms_content.py` rather than in it, and named for what
it holds rather than `test_*`, so `unittest discover -p 'test_*.py'` collects the
contract and not the analysis under it.
"""

import re
from pathlib import Path

from terms_prose import prose


REPO_ROOT = Path(__file__).resolve().parents[2]
# The marker stays out of every literal a scan would read as a citation of a group:
# such a citation from tests/integration/ is coverage evidence for that leaf, and
# every `FS-terms.terms.<N>` leaf is a listed permanent exception.
MARKER = "§"
VOCABULARY = "FS-terms"
VOCABULARY_FILE = VOCABULARY + ".md"

INDEX_ENTRY = re.compile(
    r"\[" + MARKER + r"((?:FS|AR)-[a-z][a-z0-9-]*)(?:\.[^\]]*)?\]\(([^)#]+)(?:#[^)]*)?\)"
)
TERMS_HEADING = re.compile(r"^## terms: Terms\s*$")
GROUP_HEADING = re.compile(r"^### terms\.(\d+): ")
# The vocabulary's own chapter holds the groups, so it is cut at the next chapter;
# a document's holds a lean line and a flat list, so any deeper heading closes it.
CHAPTER = re.compile(r"^#{1,2} ")
ANY_HEADING = re.compile(r"^#{1,3} ")
LABEL_ROW = re.compile(r"^\s*-\s+\*\*(.+?)\*\*")
NARROWED = re.compile(r"^(.+?)\s+\(narrowed\)$")
# Linked form (`[§…](target) (words)`) and bare form (`§… (words)`) both accepted:
# `[fmt.cross_refs]` is on, so the stored line is linked, but what is read here is
# the lean line, not whether `grund fmt --write` has run.
LEAN_CITE = re.compile(
    r"(?:\[" + MARKER + r"FS-terms\.terms\.(\d+)\]\([^)]*\)|"
    + MARKER + r"FS-terms\.terms\.(\d+))\s*\(([^)]*)\)"
)

# The four, in the order the report prints them. The titles are the contract: the
# test asserts this tuple on every call.
INVARIANTS = (
    "shared labels unique",
    "lean words exist in the cited group",
    "local labels unique per document",
    "shared/local collision only as `(narrowed)`",
)


def _split_labels(label):
    """`**workspace, member, alias**` is three labels in one row, in order."""
    return [part.strip() for part in label.split(",") if part.strip()]


def _display(path, home):
    """The path as a reader of a finding sees it: repository-relative in the tree
    this repository checks, home-relative in a fixture built somewhere else."""
    for base in (REPO_ROOT, home):
        try:
            return str(path.relative_to(base))
        except ValueError:
            continue
    return path.name


def _chapter(lines, closes):
    """A `## terms: Terms` chapter as (heading line number, [(line number, line)]),
    cut at the first heading `closes` matches, or (None, None) where there is none."""
    for start, line in enumerate(lines):
        if not TERMS_HEADING.match(line):
            continue
        body = []
        for offset, follow in enumerate(lines[start + 1 :]):
            if closes.match(follow):
                break
            body.append((start + 2 + offset, follow))
        return start + 1, body
    return None, None


def shared_vocabulary(path, home):
    """{group number: [label, …]} in row order, and every label defined twice —
    the first of the four invariants, which is a property of the vocabulary alone."""
    _, body = _chapter(path.read_text(encoding="utf-8").splitlines(), CHAPTER)
    groups, seen, duplicates = {}, {}, []
    current = None
    for number, line in body or []:
        heading = GROUP_HEADING.match(line)
        if heading:
            current = int(heading.group(1))
            groups[current] = []
            continue
        if current is None:
            continue
        row = LABEL_ROW.match(line)
        if not row:
            continue
        for word in _split_labels(row.group(1)):
            if word in seen:
                first_group, first_line = seen[word]
                duplicates.append(
                    f"{_display(path, home)}:{number}: *{word}* is defined under "
                    f"terms.{current} and already under terms.{first_group} "
                    f"(line {first_line})"
                )
            else:
                seen[word] = (current, number)
            groups[current].append(word)
    return groups, duplicates


def _lean_line(body):
    """The lean line as one string: the chapter's first non-blank paragraph, joined."""
    collected, started = [], False
    for _, line in body:
        if not line.strip():
            if started:
                break
            continue
        started = True
        collected.append(line.strip())
    joined = " ".join(collected)
    return joined if joined.startswith("Leans on") else ""


def _leans(lean):
    """(group number, word) for every word a lean line names, in the line's order."""
    named = []
    for linked, bare, words in LEAN_CITE.findall(lean):
        number = int(linked or bare)
        named.extend((number, word) for word in _split_labels(words))
    return named


def _local_labels(body):
    """(word, narrowed, line number) for every bold label after the lean line."""
    labels, started, past_lean = [], False, False
    for number, line in body:
        if not line.strip():
            past_lean = started
            continue
        started = True
        if not past_lean:
            continue
        row = LABEL_ROW.match(line)
        if not row:
            continue
        narrowed = NARROWED.match(row.group(1))
        if narrowed:
            labels.append((narrowed.group(1).strip(), True, number))
            continue
        for word in _split_labels(row.group(1)):
            labels.append((word, False, number))
    return labels


def _token(word):
    """§FS-terms.senses.2 — the word standing on its own, with no word character and
    no hyphen on either side of it, so a compound is one name and not its parts."""
    return re.compile(r"(?<![\w-])" + re.escape(word) + r"(?![\w-])", re.IGNORECASE)


def documents(home):
    """Every (id, path) the home's index links, the vocabulary itself left out: it
    defines the shared words rather than leaning on them."""
    home = Path(home).resolve()
    found = {}
    index = (home / "README.md").read_text(encoding="utf-8")
    for ident, target in INDEX_ENTRY.findall(index):
        if ident == VOCABULARY:
            continue
        found.setdefault(ident, (home / target).resolve())
    return found


def _read(home):
    """The home as the two checks need it: the shared groups in row order, the
    duplicate labels, and per document its Terms chapter and lean line."""
    home = Path(home).resolve()
    groups, duplicates = shared_vocabulary(home / VOCABULARY_FILE, home)
    read = []
    for ident, path in sorted(documents(home).items()):
        if not path.is_file():
            continue
        lines = path.read_text(encoding="utf-8").splitlines()
        heading, body = _chapter(lines, ANY_HEADING)
        if body is None:
            continue
        read.append((ident, path, heading, body))
    return home, groups, duplicates, read


def invariant_findings(home):
    """The four fixed-syntax invariants of the lean-line grammar §FS-terms.terms
    states, as ((title, [finding, …]), …) in INVARIANTS order. Every one of them is
    a property of the text alone, which is what lets them gate where
    §FS-terms.senses.5's report may not."""
    home, groups, duplicates, read = _read(home)
    shared = {word for words in groups.values() for word in words}
    unknown, duplicate_local, bare_collisions = [], [], []

    for ident, path, _, body in read:
        where = _display(path, home)
        for number, word in _leans(_lean_line(body)):
            if word in groups.get(number, []):
                continue
            group = f"terms.{number}" if number in groups else f"terms.{number} (no such group)"
            unknown.append(
                f"{where}: {ident} leans on {group} for *{word}*, "
                f"which that group does not define"
            )
        seen = {}
        for word, narrowed, number in _local_labels(body):
            if word in seen:
                duplicate_local.append(
                    f"{where}:{number}: {ident} defines *{word}* twice in its own "
                    f"Terms chapter (first at line {seen[word]})"
                )
            else:
                seen[word] = number
            if word in shared and not narrowed:
                bare_collisions.append(
                    f"{where}:{number}: {ident} redefines the shared word *{word}* "
                    f"without the `(narrowed)` form"
                )

    return tuple(
        zip(INVARIANTS, (duplicates, unknown, duplicate_local, bare_collisions))
    )


def advisory_report(home):
    """§FS-terms.senses.5 — what a lean line looks like it is missing and what it
    looks like it no longer needs, in the fixed order that makes two runs over one
    tree print the same bytes. Advice, never a verdict: it applies §FS-terms.senses.2
    and §FS-terms.senses.4 and cannot apply the two exclusions that are read rather
    than matched, so a line it prints is a question for an author."""
    home, groups, _, read = _read(home)
    order = [(number, word) for number in sorted(groups) for word in groups[number]]
    patterns = {word: _token(word) for _, word in order}

    uses, leans = [], []
    for ident, path, heading, body in read:
        own = {heading, *(number for number, _ in body)}
        text = "\n".join(line for number, line in prose(path) if number not in own)
        leaned = {word for _, word in _leans(_lean_line(body))}
        for number, word in order:
            appears = bool(patterns[word].search(text))
            if appears and word not in leaned:
                uses.append(
                    f"{ident}: uses *{word}* in prose but does not lean on it "
                    f"(terms.{number})"
                )
            elif not appears and word in leaned:
                leans.append(
                    f"{ident}: leans on *{word}* but the token appears nowhere "
                    f"in its prose"
                )
    return uses + leans
