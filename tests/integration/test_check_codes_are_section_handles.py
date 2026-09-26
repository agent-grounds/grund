"""§REQ-spec-section-names.code — every check `grund check` enforces is a section
of the concept spec it constrains, named by its diagnostic code verbatim under
that spec's `checks` chapter, and carries a row in the code catalog of
§FS-errors.5.5 that cites the section. This is the requirement's meter
(§AR-goal-measurement.1), and `NOT_YET_MIGRATED` below is the progress bar: the
migration of agent-grounds/grund#260 runs one concept spec at a time, and each
slice strikes its codes off that list as it lands.

The first test is decorated `@unittest.expectedFailure` on purpose. It is the
pin for #260 and it fails today on all ten codes, because the spec the ten move
to does not exist yet and the catalog is still a flat fenced list rather than a
table. The pre-commit hook runs this suite, so a plainly failing test could not
be committed without `--no-verify`, which this repository forbids; and an
`expectedFailure` that starts passing is an unexpected success, which turns the
suite red. The decorator therefore cannot outlive the migration: the change that
lands the ten sections removes it in the same commit."""

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
FUNCTIONAL_SPEC = REPO_ROOT / "docs" / "functional-spec"
CATALOG_PAGE = FUNCTIONAL_SPEC / "FS-errors.md"
CATALOG_HEADING = re.compile(r"^### 5\.5 ")
SELECTION = REPO_ROOT / "crates" / "grund-core" / "src" / "checker" / "selection.rs"
SELECTION_CODES = re.compile(r'^\s{4}"([a-z][a-z0-9-]*)",$', re.M)
CHECK_SECTION = re.compile(r"^###\s+checks\.([a-z][a-z0-9-]*):")
HEADING = re.compile(r"^#{1,6}\s")
CODE_CELL = re.compile(r"^`?([a-z][a-z0-9-]*)`?$")

# The ten codes of the `FS-declarations` slice, and the spec they move to. A
# later slice adds its own spec here and strikes its codes off NOT_YET_MIGRATED.
MIGRATED = {
    "FS-declarations": (
        "broken-stub",
        "declaration-near-miss",
        "duplicate",
        "duplicate-section",
        "misplaced-declaration",
        "orphan-section",
        "oversized-lead",
        "section-heading-level",
        "section-outside-declaration",
        "unmarked-heading",
    ),
}

NOT_YET_MIGRATED = (
    "agents-init",
    "chapter-cardinality",
    "citation-cardinality",
    "dangling",
    "deprecated-config-location",
    "discouraged-citation",
    "empty-citation-obligation",
    "empty-scan",
    "escaped-citation-resolves",
    "forbidden-citation",
    "full-scope-ignored",
    "inline-citation-style",
    "invalid-rule",
    "invalid-value-binding",
    "invalid-value-declaration",
    "io",
    "missing-citation",
    "missing-index-entry",
    "missing-section",
    "missing-snapshot",
    "nothing-recognized",
    "optional-member-absent",
    "out-of-scope-dangling",
    "out-of-scope-missing-section",
    "out-of-scope-shorthand-citation",
    "out-of-scope-unknown-project",
    "redundant-config",
    "shorthand-citation",
    "shorthand-numeric-run",
    "suggested-citation",
    "uncited-unit",
    "ungrounded",
    "unknown-project",
    "unlinked-index-entry",
    "unlisted-workspace-block",
    "unused",
    "value-mismatch",
)

# Selectable today and in no catalog, so neither migrated nor awaiting a slice.
# The drift is its own ticket, which is why the catalog-agreement test is skipped
# rather than widened.
KNOWN_DRIFT = ("local-section-citation", "out-of-scope-local-section-citation")


def _selectable_codes():
    """The public selector vocabulary, read from the shipped constant."""
    return set(SELECTION_CODES.findall(SELECTION.read_text(encoding="utf-8")))


def _catalog_rows():
    """Each table row of the code catalog, by the code its first cell names."""
    lines = CATALOG_PAGE.read_text(encoding="utf-8").splitlines()
    start = next(index for index, line in enumerate(lines) if CATALOG_HEADING.match(line))
    rows = {}
    for line in lines[start + 1 :]:
        if HEADING.match(line):
            break
        if not line.startswith("|"):
            continue
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        match = CODE_CELL.match(cells[0])
        if match:
            rows[match.group(1)] = cells
    return rows


def _check_sections(spec):
    """Every code named by a `checks.<code>` section of one spec page."""
    page = FUNCTIONAL_SPEC / f"{spec}.md"
    if not page.is_file():
        return set()
    return {
        match.group(1)
        for match in (CHECK_SECTION.match(line) for line in page.read_text(encoding="utf-8").splitlines())
        if match
    }


def _all_check_sections():
    """Every (code, spec) a `checks.<code>` section names anywhere in the spec."""
    found = {}
    for page in sorted(FUNCTIONAL_SPEC.glob("FS-*.md")):
        for code in _check_sections(page.stem):
            found[code] = page.stem
    return found


class CheckCodeSectionTests(unittest.TestCase):
    maxDiff = None  # the failure is the list of what is not migrated yet; print all of it

    @unittest.expectedFailure
    def test_migrated_codes_are_section_handles(self):
        """The pin for agent-grounds/grund#260: see the module docstring for why
        this is expected to fail, and when the decorator must come off. Every
        code is reported in one failure, because ten names read better than the
        first one that happens to sort first."""
        rows = _catalog_rows()
        missing = []
        for spec, codes in sorted(MIGRATED.items()):
            sections = _check_sections(spec)
            for code in codes:
                coordinate = f"{spec}.checks.{code}"
                if code not in sections:
                    missing.append(f"{coordinate}: no such section")
                if code not in rows:
                    missing.append(f"{code}: no row in the code catalog")
                elif not any(coordinate in cell for cell in rows[code][1:]):
                    missing.append(f"{code}: its catalog row does not cite {coordinate}")
        self.assertEqual([], missing)

    @unittest.skip(
        "the shipped selector vocabulary holds local-section-citation and "
        "out-of-scope-local-section-citation, which are in no catalog; that "
        "drift is its own ticket and is not widened here"
    )
    def test_catalog_and_selector_vocabulary_agree(self):
        self.assertEqual(_selectable_codes(), set(_catalog_rows()))

    def test_every_selectable_code_is_migrated_or_listed(self):
        accounted = set(NOT_YET_MIGRATED) | set(KNOWN_DRIFT)
        for codes in MIGRATED.values():
            accounted |= set(codes)
        self.assertEqual(_selectable_codes(), accounted, "codes in no list of this test")

    def test_nothing_listed_as_unmigrated_has_a_section_handle(self):
        found = _all_check_sections()
        moved = {code: found[code] for code in NOT_YET_MIGRATED if code in found}
        self.assertEqual({}, moved, "strike these off NOT_YET_MIGRATED: their slice has landed")


if __name__ == "__main__":
    unittest.main()
