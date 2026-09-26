"""§AR-goal-measurement.3 — the backward-compatibility meter proves that each
verdict route is explicit, bounded, and exercised by its repository record."""

import importlib.util
import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
REQUIREMENT = REPO_ROOT / "docs" / "requirements" / "REQ-backwards-compatibility.md"
REQUIREMENTS = REPO_ROOT / "docs" / "requirements"
DECISIONS = REPO_ROOT / "docs" / "decisions"
COVER_DECISION = DECISIONS / "functional" / "DF-cover-workspace-scope.md"
RELEASE = REPO_ROOT / "docs" / "changelog" / "0.10.1.md"
RELEASES = REPO_ROOT / "docs" / "changelog"
CHANGELOG = REPO_ROOT / "docs" / "changelog.md"
CORRECTION_ROUTE = "§REQ-backwards-compatibility.5"
CONFLICT_PROOF = "§REQ-no-missed-citation.1"
REQUIREMENT_SECTION_RE = re.compile(r"§(REQ-[a-z0-9-]+)\.(\d+(?:\.\d+)*)")
RELEASE_GUARD = REPO_ROOT / "scripts" / "check_release_ramps.py"

_GUARD_SPEC = importlib.util.spec_from_file_location("check_release_ramps", RELEASE_GUARD)
ramps = importlib.util.module_from_spec(_GUARD_SPEC)
assert _GUARD_SPEC.loader is not None
_GUARD_SPEC.loader.exec_module(ramps)

# Where a message's bytes are fixed: every tree the release guard reads
# (§FS-distribution.4.2.2), plus the functional spec, which quotes warnings
# verbatim and which those sources do not cover — the gap this file closes.
MESSAGE_HOMES = ramps.SOURCES + (("docs/functional-spec", "*.md"),)
SPECIFICATION_SITE = "docs/functional-spec/FS-cli.md"

# The bare-`grund` warning as each site writes it. The wildcards stand in for the
# backticks around the command name, which the specification escapes and the
# message does not, so one pattern finds every site. It names no release and no
# clause, because the defect is the absence of a window rather than a phrasing.
FALLBACK_WARNING_RE = re.compile(r"bare .{0,2}grund.{0,2} still runs")
DECISION_CITATION_RE = re.compile(
    r"§((?:DF|DA)-[a-z0-9-]+)(?:\.[a-z0-9-]+)*\b"
)


def _section(text, number):
    match = re.search(
        rf"^## {re.escape(str(number))}\. .+$(.*?)(?=^## \d+\.|\Z)",
        text,
        re.MULTILINE | re.DOTALL,
    )
    if not match:
        raise AssertionError(f"section {number} is missing")
    return match.group(0)


def _message_sites(pattern):
    """Every `(path, line number, line)` in a message home the pattern matches."""
    sites = []
    for directory, glob in MESSAGE_HOMES:
        base = REPO_ROOT / directory
        if not base.is_dir():
            continue
        for path in sorted(base.rglob(glob)):
            text = path.read_text(encoding="utf-8", errors="replace")
            for number, line in enumerate(text.splitlines(), start=1):
                if pattern.search(line):
                    sites.append(
                        (path.relative_to(REPO_ROOT).as_posix(), number, line)
                    )
    return sites


def _named_releases(site):
    path, _, line = site
    return [claim.release for claim in ramps.scan_text(path, line)]


def _requirement_catalog():
    catalog = {}
    for path in REQUIREMENTS.glob("REQ-*.md"):
        text = path.read_text(encoding="utf-8")
        declaration = re.match(r"# (REQ-[a-z0-9-]+):", text)
        if declaration:
            catalog[declaration.group(1)] = set(
                re.findall(r"^## (\d+(?:\.\d+)*)\.", text, re.M)
            )
    return catalog


def _release_entry(text, *markers):
    entries = [
        line
        for line in text.splitlines()
        if line.startswith("- ") and all(marker in line for marker in markers)
    ]
    if len(entries) != 1:
        raise AssertionError(
            f"expected one release entry containing {markers!r}, found {len(entries)}"
        )
    return entries[0]


def _verdict_route_citations(text):
    section = _section(text, 1)
    clause = re.search(r"The \*\*verdict\*\*[^\n]*?\.(?=\s|$)", section)
    if not clause:
        raise AssertionError("section 1 has no verdict-route clause")
    return {
        section
        for requirement, section in REQUIREMENT_SECTION_RE.findall(clause.group(0))
        if requirement == "REQ-backwards-compatibility"
    }


def _release_entries():
    """Every release bullet, the unreleased ones included.

    A record lands with the change it justifies and before the release that
    carries it, so reading only the archived releases under `docs/changelog/`
    would make condition .5.3 unsatisfiable on the day the record merges. The
    `## Unreleased` section of `docs/changelog.md` becomes those release notes
    verbatim when `prepare_changelog_release.py` cuts the version, so it is the
    same text read one release earlier. Every other condition stays conjunctive.
    """
    return [
        line
        for path in sorted(RELEASES.glob("*.md"))
        for line in path.read_text(encoding="utf-8").splitlines()
        if line.startswith("- ")
    ] + [
        line
        for line in _unreleased_section().splitlines()
        if line.startswith("- ")
    ]


def _unreleased_section():
    text = CHANGELOG.read_text(encoding="utf-8")
    match = re.search(r"^## Unreleased$(.*?)(?=^## \d+\.|\Z)", text, re.M | re.S)
    return match.group(1) if match else ""


def _correction_route_errors(text, release_entries, catalog):
    errors = []
    declaration = re.match(r"# ((?:DF|DA)-[a-z0-9-]+):", text)
    if not declaration:
        return ["the decision must declare a DF or DA ID"]
    if "**Status:** Accepted" not in text:
        errors.append("the decision must be accepted")

    route_sections = [
        match.group(0)
        for match in re.finditer(
            r"^## (\d+)\. .+$(.*?)(?=^## \d+\.|\Z)",
            text,
            re.MULTILINE | re.DOTALL,
        )
        if CORRECTION_ROUTE in match.group(0)
    ]
    if len(route_sections) != 1:
        return errors + ["the correction route must be invoked in one numbered section"]

    route = route_sections[0]
    cited_prohibitions = {
        (requirement, section)
        for requirement, section in REQUIREMENT_SECTION_RE.findall(route)
        if requirement != "REQ-backwards-compatibility"
        and section in catalog.get(requirement, set())
    }
    if not cited_prohibitions:
        errors.append(
            "the route must cite a numbered section of a different hard requirement"
        )
    proved_prohibitions = {
        (requirement, section)
        for requirement, section in cited_prohibitions
        if re.search(
            rf"(?:old|prior)[^.\n]*\b(?:violated|forbidden|prohibition)\b"
            rf"[^.\n]*§{re.escape(requirement)}\.{re.escape(section)}\b",
            route,
            re.IGNORECASE,
        )
    }
    if not proved_prohibitions:
        errors.append("the route must prove that the prior verdict violated the citation")
    if not re.search(r"already applied[^.\n]*when[^.\n]*shipped", route):
        errors.append("the route must prove that the prohibition already applied")

    for section, name in (("2", "deprecation"), ("3", "mechanical migration")):
        if not re.search(
            rf"\u00a7REQ-backwards-compatibility\.{section}(?![\w.-])"
            rf"(?:\]\([^\n)]+\))?[^.\n]*(?:cannot|does not|doesn't|has no|there is no)",
            route,
            re.IGNORECASE,
        ):
            errors.append(f"the route must explain why §{section} {name} does not fit")
    if not re.search(r"not a licence|cannot justify", route, re.IGNORECASE):
        errors.append("the route must deny a broader compatibility licence")

    decision_id = declaration.group(1)
    matching_releases = [
        entry
        for entry in release_entries
        if decision_id in DECISION_CITATION_RE.findall(entry)
        and CORRECTION_ROUTE in entry
    ]
    matching_releases = [
        entry
        for entry in matching_releases
        if any(
            f"§{requirement}.{section}" in entry
            for requirement, section in proved_prohibitions
        )
    ]
    if not matching_releases:
        errors.append("the decision must have a matching release record")
    elif not any(
        re.search(
            r"verdicts?[^.]{0,20}(?:change[ds]?|flip(?:s|ped)?|move[sd]?)",
            entry,
            re.IGNORECASE,
        )
        and re.search(r"every finding[^.]*location[^.]*(?:fix|action)", entry, re.I)
        for entry in matching_releases
    ):
        errors.append(
            "the matching release must name the verdict change and located remedies"
        )
    return errors


class RequirementRouteTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.text = REQUIREMENT.read_text(encoding="utf-8")

    def test_verdict_routes_are_exhaustive_and_the_exemption_is_unchanged(self):
        self.assertEqual({"2", "3", "5"}, _verdict_route_citations(self.text))

        exemption = _section(self.text, 4)
        self.assertIn("no defined meaning", exemption)
        self.assertIn("produced no output", exemption)
        self.assertIn("before `1.0`", exemption)
        self.assertIn("not a general escape", exemption)

    def test_unapproved_route_cannot_hide_from_the_exhaustive_check(self):
        mutated = self.text.replace(
            "or [\u00a7REQ-backwards-compatibility.5]",
            "[\u00a7REQ-backwards-compatibility.6]"
            "(REQ-backwards-compatibility.md#6-unapproved-route), "
            "or [\u00a7REQ-backwards-compatibility.5]",
            1,
        )
        self.assertNotEqual(self.text, mutated, "the synthetic route must be inserted")
        self.assertEqual({"2", "3", "5", "6"}, _verdict_route_citations(mutated))

    def test_correction_route_keeps_all_five_gates_conjunctive(self):
        route = _section(self.text, 5)
        self.assertIn("only when all five conditions hold", route)
        conditions = dict(
            re.findall(r"^\d+\. \*\*([^*]+)\.\*\* (.+)$", route, re.MULTILINE)
        )
        self.assertEqual(
            {
                "Prior prohibition",
                "Accepted proof",
                "Named release",
                "Actionable findings",
                "No new licence",
            },
            set(conditions),
        )
        self.assertRegex(
            conditions["Prior prohibition"],
            r"separately declared hard requirement.*already applied.*cited numbered section",
        )
        self.assertRegex(
            conditions["Accepted proof"],
            r"accepted decision record.*proves both the conflict.*"
            r"neither the \[\u00a7REQ-backwards-compatibility\.2\].* "
            r"nor the \[\u00a7REQ-backwards-compatibility\.3\]",
        )
        self.assertRegex(
            conditions["Named release"], r"release names the verdict change"
        )
        self.assertRegex(
            conditions["Actionable findings"],
            r"[Ee]very finding.*names its location.*action the maintainer can take",
        )
        self.assertRegex(
            conditions["No new licence"],
            r"cannot justify ordinary policy tightening, feature removal, or a prohibition invented",
        )


class DecisionRouteTests(unittest.TestCase):
    def test_every_correction_decision_proves_all_five_gates(self):
        catalog = _requirement_catalog()
        release_entries = _release_entries()
        for path in sorted(DECISIONS.rglob("*.md")):
            text = path.read_text(encoding="utf-8")
            if CORRECTION_ROUTE not in text:
                continue
            with self.subTest(path=path.relative_to(REPO_ROOT)):
                self.assertEqual(
                    [], _correction_route_errors(text, release_entries, catalog)
                )

    def test_unproved_synthetic_correction_is_rejected(self):
        synthetic = """# DF-synthetic: invalid correction

**Status:** Accepted

## 1. Consequences

§REQ-backwards-compatibility.5 is invoked. §REQ-never-crashes.1 is related,
but there is no conflict proof, ordinary-route analysis, or release record.
"""
        errors = _correction_route_errors(
            synthetic, _release_entries(), _requirement_catalog()
        )
        self.assertIn(
            "the route must prove that the prior verdict violated the citation", errors
        )
        self.assertIn("the route must explain why \u00a72 deprecation does not fit", errors)
        self.assertIn(
            "the route must explain why \u00a73 mechanical migration does not fit", errors
        )
        self.assertIn("the decision must have a matching release record", errors)

    def test_release_join_requires_the_complete_decision_citation_id(self):
        decision = COVER_DECISION.read_text(encoding="utf-8")
        release_entries = _release_entries()
        catalog = _requirement_catalog()
        self.assertEqual(
            [], _correction_route_errors(decision, release_entries, catalog)
        )

        prefix_collision = decision.replace(
            "# DF-cover-workspace-scope:", "# DF-cover-workspace:", 1
        )
        self.assertIn(
            "the decision must have a matching release record",
            _correction_route_errors(prefix_collision, release_entries, catalog),
        )

    def test_cover_decision_is_the_accepted_worked_case(self):
        consequences = _section(COVER_DECISION.read_text(encoding="utf-8"), 4)
        self.assertTrue(
            CORRECTION_ROUTE in consequences,
            "DF-cover-workspace-scope.4 does not invoke the correction route",
        )
        self.assertIn(CONFLICT_PROOF, consequences)
        self.assertRegex(
            consequences,
            r"workspace whose `\[workspace\]` block cannot be expanded now fails `cover`",
        )
        self.assertIn("duplicate workspace project alias", consequences)
        self.assertIn("broken symlink names the path that cannot be read", consequences)
        self.assertIn("no old form to carry beside a new one", consequences)
        self.assertIn(
            "no command the tool ships that completes the change", consequences
        )


class ReleaseRouteTests(unittest.TestCase):
    def test_0_10_1_names_the_worked_correction_and_its_remedies(self):
        entry = _release_entry(
            RELEASE.read_text(encoding="utf-8"), "PR #114", "§DF-cover-workspace-scope"
        )
        self.assertIn("§DF-cover-workspace-scope", entry)
        self.assertTrue(
            CORRECTION_ROUTE in entry,
            "the 0.10.1 PR #114 entry does not invoke the correction route",
        )
        self.assertIn(CONFLICT_PROOF, entry)
        self.assertIn("`grund cover` indexes every project the run loaded", entry)
        self.assertIn("Verdicts move `0` → `2` in workspaces", entry)
        self.assertIn("names a location and a fix", entry)



class DeprecationWindowTests(unittest.TestCase):
    """§REQ-backwards-compatibility.2's worked example, held to the path it is
    the example of.

    Bare `grund` keeping its historical `check .` behavior through a named window
    is that path's worked example (§FS-cli.1), so every site that fixes the
    warning's bytes has to name the release the fallback stops working in, in a
    clause of §FS-distribution.4.2.3's closed vocabulary — and all of them have
    to name the *same* release. The release guard holds the first half for the
    sources it reads and nothing holds the second at all, which is how the
    specification's own copy came to name no release while promising a window.

    Nothing here asserts a particular release or a particular phrasing: the
    clause vocabulary and the matcher are the gate's own, read out of
    `scripts/check_release_ramps.py`, so no rewording satisfies these tests and
    the maintainer's choice of release is free to be any release.
    """

    def setUp(self):
        self.sites = _message_sites(FALLBACK_WARNING_RE)
        self.assertTrue(
            self.sites,
            "no message home carries the bare `grund` warning, so these tests no "
            "longer ask the question they were written for",
        )

    def test_a_written_in_clause_is_read_at_every_site(self):
        # The positive control. `9.9.9` is a placeholder proving the matcher can
        # see a named release in these exact lines, so a site reported below as
        # naming none means a missing window rather than a blind matcher.
        for site in self.sites:
            path, number, line = site
            with self.subTest(site=f"{path}:{number}"):
                written_in = line.replace(
                    "still runs", "is removed in grund 9.9.9 and still runs"
                )
                self.assertIn("9.9.9", _named_releases((path, number, written_in)))

    def test_an_unlisted_phrasing_names_no_release_the_gate_can_see(self):
        # The negative control, and the reason the wording is not free: the
        # vocabulary is closed, so a release named outside it is named nowhere.
        self.assertEqual(
            [],
            _named_releases(
                (SPECIFICATION_SITE, 1, "this fallback will be removed in grund 9.9.9")
            ),
        )

    def test_every_site_names_the_release_the_fallback_is_removed_in(self):
        unnamed = [
            f"{path}:{number}"
            for path, number, line in self.sites
            if not _named_releases((path, number, line))
        ]
        self.assertEqual(
            [],
            unnamed,
            "these sites fix the bare `grund` warning's bytes and name no release "
            "in the closed clause vocabulary, so the deprecation path of "
            "§REQ-backwards-compatibility.2 owes a window it does not give",
        )

    def test_the_sites_name_one_release_between_them(self):
        # Every site reduced to the releases it names, so a site naming none
        # cannot be satisfied by a sibling that does: the drift this test exists
        # for is two sites promising two different windows, and the gate catches
        # neither of them.
        by_site = {
            f"{path}:{number}": tuple(sorted(set(_named_releases((path, number, line)))))
            for path, number, line in self.sites
        }
        self.assertEqual(
            1,
            len(set(by_site.values())),
            "the sites that fix the bare `grund` warning's bytes must agree on the "
            f"release the fallback is removed in; they name {by_site}",
        )
        (named,) = set(by_site.values())
        self.assertEqual(
            1,
            len(named),
            "every site must name exactly one release in the closed clause "
            f"vocabulary; each of them names {list(named)}",
        )

    def test_the_specification_copy_sits_outside_the_release_guards_sources(self):
        """§FS-distribution.4.2.2: the guard reads message text and the goldens
        that pin it, and nothing under `docs/`. So the specification's verbatim
        copy of this warning is held by the test above and by review alone, which
        is why `MESSAGE_HOMES` extends the guard's sources rather than reusing
        them."""
        self.assertNotIn(
            SPECIFICATION_SITE,
            [
                path.relative_to(REPO_ROOT).as_posix()
                for directory, glob in ramps.SOURCES
                if (REPO_ROOT / directory).is_dir()
                for path in (REPO_ROOT / directory).rglob(glob)
            ],
        )
        self.assertIn(SPECIFICATION_SITE, [path for path, _, _ in self.sites])


if __name__ == "__main__":
    unittest.main()
