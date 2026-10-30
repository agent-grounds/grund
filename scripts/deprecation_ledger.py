"""Read the core-concerns plan for release preflight. §FS-distribution.4.2.2.1

§DISC-core-concerns.9.9 defines the ledger table and the due-notice consistency
rule, and the phase leads of §DISC-core-concerns.9 say whether each phase shipped.
Release preflight and the candidate-CI validator share this interpretation
(§AR-system.principle); removal deadlines remain that validator's responsibility.
"""

import re


LEDGER_PATH = "docs/discussions/proposals/2026-09-30-core-concerns.md"
HEADING = re.compile(r"^(#{2,3}) (\d+(?:\.\d+)?)[. ]", re.MULTILINE)
VERSION = re.compile(r"`?v?(\d+)\.(\d+)\.(\d+)`?")
LEDGER_HEADER = ("surface", "notice", "removal", "status")
PHASE = re.compile(r"^Phase \d+:")
PLANNED_FOR = re.compile(r"\bfor ((?:`\d+\.\d+\.(?:\d+|x)`(?:,? (?:and )?)?)+)")
PLANNED_RELEASE = re.compile(r"`(\d+)\.(\d+)\.(\d+|x)`")


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


def planned_notice_due(notice, status, release):
    """§DISC-core-concerns.9.9: a parsed notice is due at or below the release."""
    return status == "planned" and notice is not None and notice <= release


def notice_problems(root, release):
    """§FS-distribution.4.2.2.1: refuse only planned notices due by this release."""
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
        if planned_notice_due(version(notice), status, version(release)):
            problems.append(
                f"error: {LEDGER_PATH}: {surface}: notice {notice} is still planned "
                f"at or below requested release {release}; land the notice and mark it shipped"
            )
    return problems


def unshipped_phases(text, bound):
    """§FS-distribution.4.2.2.1: each §DISC-core-concerns.9 phase whose lead opens
    `Not shipped` and plans it for a release at or below the bound, as the phase's
    number and name beside the release as the lead writes it."""
    found = []
    for minor in range(1, 10):
        body = section(text, f"9.{minor}")
        if body is None:
            continue
        title, _, rest = body.partition("\n")
        if not PHASE.match(title):
            continue
        lead = rest.strip().split("\n\n", 1)[0]
        if not lead.startswith("Not shipped"):
            continue
        for listed in PLANNED_FOR.finditer(lead):
            for major, minor_part, patch in PLANNED_RELEASE.findall(listed.group(1)):
                # A series `0.N.x` has closed once the bound is past `0.N`.
                last_patch = float("inf") if patch == "x" else int(patch)
                if (int(major), int(minor_part), last_patch) <= bound:
                    phase = f"9.{minor} {title.split(':')[0]}"
                    found.append((phase, f"{major}.{minor_part}.{patch}"))
    return found


def phase_lead_problems(root, release):
    """§FS-distribution.4.2.2.1: refuse only phases still planned for this release."""
    path = root / LEDGER_PATH
    if not path.exists():
        return []
    text = path.read_text(encoding="utf-8")
    return [
        f"error: {LEDGER_PATH}: {phase}: not shipped but planned for {planned}, "
        f"at or below requested release {release}; "
        "record the phase as shipped or move it to a later release"
        for phase, planned in unshipped_phases(text, version(release))
    ]
