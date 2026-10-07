"""Related work has a citable home, and every entry in it is grounded. Where grund
comes from is part of its why (§GRUND-grund), so the `REL` kind names the
ground it bears on (§FS-config.3.9), its folder's index fixes the three chapters
every entry carries (§REQ-spec-section-names.shape), and every entry is cited
from outside its folder: an index entry is not a citation of it
(§FS-check.4.1.2), and the release self-check fails on the warning an uncited
entry raises. The roadmap does not count as that citer, because it deletes a
point once the point ships. No architecture point describes this repository's
own documents, so this test cites the points it holds them to instead.

§REQ-readme.1 and §REQ-readme.evidence require a short README path to a sourced,
task-oriented comparison. The assertions below detect the reported misleading
passages and preserve addresses; factual source review remains required."""

import re
import subprocess
import tomllib
import unittest
from collections import defaultdict
from pathlib import Path, PurePosixPath


REPO_ROOT = Path(__file__).resolve().parents[2]
CONFIG = REPO_ROOT / "grund.toml"
HOME = "docs/related-work"
FOLDER = REPO_ROOT / HOME
INDEX = FOLDER / "README.md"
ROADMAP = REPO_ROOT / "docs" / "roadmap.md"
TITLE = "Related work: the ideas grund descends from and the tools beside it"

# The eight bodies of work agent-grounds/grund#384 names, one entry each.
ENTRIES = (
    "incremental-formalization",
    "flexiformality",
    "gradual-typing",
    "information-typing",
    "graph-shapes",
    "schema-later",
    "traceability-tools",
    "spec-driven-development",
)

# The chapters the index lead fixes, in order: what the work is, where grund
# agrees with it, and where grund departs.
CHAPTERS = (
    "work: What the work is",
    "agrees: Where grund agrees",
    "departs: Where grund departs",
)

TRACE_ENTRY = FOLDER / "REL-traceability-tools.md"
README = REPO_ROOT / "README.md"

FENCE_OPEN = re.compile(r"^ {0,3}(`{3,}|~{3,})")
FENCE_CLOSE = re.compile(r"^ {0,3}(`{3,}|~{3,})\s*$")
DECLARATION = re.compile(r"^# (REL-[a-z][a-z0-9-]*):")
CHAPTER = re.compile(r"^## (.+?)\s*$")
GROUND_CITATION = re.compile(r"§(?:GRUND|GOAL)-[a-z]")
REL_CITATION = re.compile(r"§(REL-[a-z][a-z0-9-]*)")
INDEX_CHAPTER = re.compile(r"^- `## ([^`]+)`")
SIBLING_DECLARATION = re.compile(r"^## [A-Z][A-Z0-9]*-[a-z]")


def _config():
    return tomllib.loads(CONFIG.read_text(encoding="utf-8"))


def _prose_lines(text):
    """The lines of a Markdown file outside its fenced code blocks, where a `§`
    is not a citation."""
    fence = None
    for line in text.splitlines():
        if fence is None:
            opening = FENCE_OPEN.match(line)
            if opening:
                fence = opening.group(1)
            else:
                yield line
            continue
        closing = FENCE_CLOSE.match(line)
        if closing and closing.group(1)[0] == fence[0] and len(closing.group(1)) >= len(fence):
            fence = None


def _entry(slug):
    return FOLDER / f"REL-{slug}.md"


def _missing(slug):
    return f"REL-{slug}: no file {HOME}/REL-{slug}.md"


def _home(config, kind):
    for row in config["kinds"]:
        if row["kind"] == kind:
            return row.get("folder") or row.get("file")
    return None


def _scanned_files(config):
    """Every file `grund check` reads under this configuration: the `[scan]`
    roots and every walked kind home, less the excluded directory names, by
    extension, and without what `.gitignore` hides."""
    scan = config["scan"]
    roots = list(scan["include"])
    roots += [_home(config, row["kind"]) for row in config["kinds"]
              if row.get("scan", True) and _home(config, row["kind"])]
    excluded = set(scan["exclude"])
    extensions = {"." + extension for extension in scan["extensions"]}
    listing = subprocess.run(
        ["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard"],
        cwd=REPO_ROOT,
        check=True,
        capture_output=True,
    ).stdout.decode("utf-8")
    for name in sorted(set(listing.split("\0")) - {""}):
        path = PurePosixPath(name)
        if path.suffix not in extensions or excluded & set(path.parts[:-1]):
            continue
        if not any(name == root or name.startswith(root.rstrip("/") + "/") for root in roots):
            continue
        if (REPO_ROOT / name).is_file():
            yield name


def _outside_citers(config):
    """For each REL ID, the files outside its own folder and outside the roadmap
    that cite it."""
    roadmap = _home(config, "RM")
    citers = defaultdict(set)
    for name in _scanned_files(config):
        if name.startswith(HOME + "/") or name == roadmap:
            continue
        text = (REPO_ROOT / name).read_text(encoding="utf-8", errors="replace")
        lines = _prose_lines(text) if name.endswith(".md") else text.splitlines()
        for line in lines:
            for match in REL_CITATION.finditer(line):
                citers[match.group(1)].add(name)
    return citers


def _declaration_body(text, declared):
    """The lines of one `##` declaration in a single-file kind, up to the next
    declaration; `None` once the declaration is gone."""
    body = None
    for line in _prose_lines(text):
        if SIBLING_DECLARATION.match(line):
            if body is not None:
                break
            if line.startswith(f"## {declared}:"):
                body = []
        elif body is not None:
            body.append(line)
    return body


def _positioning_defects(text):
    """Finite regressions from #475, not a factual-quality classifier."""
    patterns = {
        "future gap-report promise": r"^\| \*\*grund\*\* \|.*⏳.*RM-gap-report",
        "coverage-parity promise": r"Coverage parity is one shipping milestone away",
        "blanket schema-check prohibition": r"schema-level custom check rules \(would require severity / exit-code config",
        "blanket atomic-clause claim": r"They model each clause as its own atomic item",
        "blanket coverage-only claim": r"They are optimized for a coverage report|traceability tools optimized for a coverage report",
    }
    return [label for label, pattern in patterns.items() if re.search(pattern, text, re.M)]


def _has_comparison_link(text):
    return bool(re.search(
        r"(?:\]\(|\]:\s*)[^\s]*docs/related-work/REL-traceability-tools\.md(?:[#)\s]|$)",
        text,
    ))


class RelatedWorkTests(unittest.TestCase):
    maxDiff = None

    def test_the_kind_is_configured_with_its_direction(self):
        config = _config()
        rows = [row for row in config["kinds"] if row["kind"] == "REL"]
        self.assertEqual(1, len(rows), "grund.toml declares no REL kind")
        self.assertEqual({"kind": "REL", "folder": HOME, "title": TITLE}, rows[0])
        self.assertEqual({"should": ["GRUND|GOAL"]}, config["citations"].get("REL"))

    def test_the_index_names_the_chapters_entries_are_held_to(self):
        lines = _prose_lines(INDEX.read_text(encoding="utf-8"))
        named = tuple(match.group(1) for line in lines if (match := INDEX_CHAPTER.match(line)))
        self.assertEqual(CHAPTERS, named)

    def test_the_folder_declares_the_eight_entries(self):
        problems, declared = [], set()
        for page in sorted(FOLDER.rglob("*.md")):
            if page == INDEX:
                continue
            lines = _prose_lines(page.read_text(encoding="utf-8"))
            ids = [match.group(1) for line in lines if (match := DECLARATION.match(line))]
            if ids != [page.stem] or page.parent != FOLDER:
                problems.append(f"{page.relative_to(REPO_ROOT)} declares {ids}, not its own name")
            declared.update(ids)
        self.assertEqual([], problems)
        self.assertEqual(sorted(f"REL-{slug}" for slug in ENTRIES), sorted(declared))

    def test_each_entry_carries_the_three_chapters_in_order(self):
        problems = []
        for slug in ENTRIES:
            if not _entry(slug).is_file():
                problems.append(_missing(slug))
                continue
            lines = _prose_lines(_entry(slug).read_text(encoding="utf-8"))
            chapters = tuple(match.group(1) for line in lines if (match := CHAPTER.match(line)))
            if chapters != CHAPTERS:
                problems.append(f"REL-{slug}: chapters {list(chapters)}")
        self.assertEqual([], problems)

    def test_each_entry_cites_the_ground_it_bears_on(self):
        problems = []
        for slug in ENTRIES:
            if not _entry(slug).is_file():
                problems.append(_missing(slug))
                continue
            lines = _prose_lines(_entry(slug).read_text(encoding="utf-8"))
            if not any(GROUND_CITATION.search(line) for line in lines):
                problems.append(f"REL-{slug}: cites no GRUND or GOAL point")
        self.assertEqual([], problems)

    def test_each_entry_is_cited_from_outside_its_folder(self):
        citers = _outside_citers(_config())
        uncited = [f"REL-{slug}" for slug in ENTRIES if not citers.get(f"REL-{slug}")]
        self.assertEqual([], uncited, f"cited only from {HOME}/ or the roadmap, or not at all")

    def test_readme_links_to_the_existing_comparison(self):
        self.assertTrue(
            _has_comparison_link(README.read_text(encoding="utf-8")),
            "README has no Markdown link to docs/related-work/REL-traceability-tools.md",
        )

    def test_traceability_comparison_uses_prose_without_rankings_or_promises(self):
        text = TRACE_ENTRY.read_text(encoding="utf-8")
        self.assertEqual([], _positioning_defects(text), "comparison retains misleading positioning")
        self.assertFalse(
            any(line.lstrip().startswith("|") for line in _prose_lines(text)),
            "comparison retains the ranking matrix instead of task-oriented prose",
        )

    def test_retained_claims_have_the_approved_dated_primary_source_ledger(self):
        text = TRACE_ENTRY.read_text(encoding="utf-8")
        # The finite ledger approved in the proposal; presence is not source verification.
        for source in (
            "2026-10-06", "unversioned", "4.10.0", "8.5.0",
            "https://openfasttrace.itsallcode.org/user_guide/user_guide.html",
            "https://openfasttrace.itsallcode.org/user_guide/use_cases/html_tracing_reports.html",
            "https://sphinx-needs.readthedocs.io/en/8.5.0/directives/needextract.html",
            "https://sphinx-needs.readthedocs.io/en/8.5.0/schema/index.html",
        ):
            with self.subTest(source=source):
                self.assertTrue(source in text, f"comparison missing ledger evidence: {source}")

    def test_comparison_grounds_shipped_tasks_and_separate_boundaries(self):
        text = TRACE_ENTRY.read_text(encoding="utf-8")
        for point in (
            "FS-show.2.2", "FS-check.3.1", "FS-check.3.2", "FS-cover.2",
            "FS-rules.3.1", "FS-rules.3.2", "FS-rules.3.3", "FS-rules.3.4",
            "FS-non-goals.9", "FS-non-goals.12.1",
        ):
            with self.subTest(point=point):
                self.assertTrue("§" + point in text, f"comparison missing shipped support: {point}")

    def test_roadmap_positioning_removes_parity_and_boundary_inferences(self):
        text = ROADMAP.read_text(encoding="utf-8")
        self.assertEqual([], _positioning_defects(text), "roadmap retains misleading positioning")

    def test_published_comparison_and_roadmap_addresses_remain(self):
        text = TRACE_ENTRY.read_text(encoding="utf-8")
        self.assertRegex(text, r"(?m)^### work\.matrix: ")
        roadmap = ROADMAP.read_text(encoding="utf-8")
        for declared in ("RM-positioning", "RM-positioning-trace-tools", "RM-gap-report"):
            with self.subTest(declared=declared):
                body = _declaration_body(roadmap, declared)
                self.assertIsNotNone(body, f"published address {declared} removed")
                for number in (1, 2, 3):
                    self.assertTrue(any(line.startswith(f"### {number}.") for line in body),
                                    f"published address {declared}.{number} removed")
        body = _declaration_body(roadmap, "RM-positioning-trace-tools")
        self.assertTrue(any("§REL-traceability-tools" in line for line in body))


class PositioningDetectorProbes(unittest.TestCase):
    def test_the_reported_misleading_claims_are_detected(self):
        for text in (
            "| **grund** | ⏳ §RM-gap-report |",
            "Coverage parity is one shipping milestone away.",
            "schema-level custom check rules (would require severity / exit-code config)",
            "They model each clause as its own atomic item.",
            "They are optimized for a coverage report.",
        ):
            with self.subTest(text=text):
                self.assertTrue(_positioning_defects(text))

    def test_task_and_boundary_prose_is_not_a_reported_defect(self):
        self.assertEqual([], _positioning_defects(
            "cover groups citations by scanned file. Chapter and citation counts are checked. "
            "Severity is fixed; arbitrary engine scripting is absent."
        ))

    def test_comparison_link_must_be_a_markdown_destination(self):
        target = "docs/related-work/REL-traceability-tools.md"
        for text in ("", target, "[comparison](docs/other.md)"):
            with self.subTest(text=text):
                self.assertFalse(_has_comparison_link(text))
        for text in (f"[comparison]({target}#workmatrix)", f"[comparison]: {target}"):
            with self.subTest(text=text):
                self.assertTrue(_has_comparison_link(text))


if __name__ == "__main__":
    unittest.main()
