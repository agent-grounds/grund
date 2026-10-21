"""Read the existing ledger for release preflight. §FS-distribution.4.2.2

§DISC-core-concerns.9.9 defines the table and the due-notice consistency rule.
The section, table and version interpretation follows the existing ledger
validator; removal deadlines remain that validator's responsibility.
"""

import re


LEDGER_PATH = "docs/discussions/proposals/2026-09-30-core-concerns.md"
HEADING = re.compile(r"^(#{2,3}) (\d+(?:\.\d+)?)[. ]", re.MULTILINE)
VERSION = re.compile(r"`?v?(\d+)\.(\d+)\.(\d+)`?")
LEDGER_HEADER = ("surface", "notice", "removal", "status")


def section(text, number):
    """§DISC-core-concerns.9.9: read the numbered section, before its next heading."""
    for match in HEADING.finditer(text):
        if match.group(2) == number:
            rest = text[match.end():]
            following = HEADING.search(rest)
            return rest[:following.start()] if following else rest
    return None


def ledger_rows(body):
    """§DISC-core-concerns.9.9: find the ledger by its four-column header."""
    rows, inside = [], False
    for line in body.splitlines():
        if not line.startswith("|"):
            if inside:
                break
            continue
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        if not inside:
            inside = tuple(cell.lower() for cell in cells) == LEDGER_HEADER
            continue
        if all(set(cell) <= set("-: ") for cell in cells):
            continue
        rows.append(cells)
    return rows


def version(cell):
    """§DISC-core-concerns.9.9: notice cells hold numeric release versions."""
    match = VERSION.fullmatch(cell.strip())
    return tuple(int(part) for part in match.groups()) if match else None


def notice_problems(root, release):
    """§FS-distribution.4.2.2: refuse only planned notices due by this release."""
    path = root / LEDGER_PATH
    if not path.exists():
        return []
    body = section(path.read_text(encoding="utf-8"), "9.9")
    rows = ledger_rows(body) if body is not None else []
    if not rows:
        raise ValueError(f"{LEDGER_PATH}: section 9.9 must carry the deprecation ledger")
    problems = []
    for cells in rows:
        if len(cells) != len(LEDGER_HEADER) or version(cells[1]) is None:
            raise ValueError(f"{LEDGER_PATH}: invalid ledger row: {cells}")
        surface, notice, _, status = cells
        if status == "planned" and version(notice) <= version(release):
            problems.append(
                f"error: {LEDGER_PATH}: {surface}: notice {notice} is still planned "
                f"at or below requested release {release}; land the notice and mark it shipped"
            )
    return problems
