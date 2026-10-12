"""§AR-system.principle.inventory, held as the citation ledger of §AR-system.5:
until a family is unified, its copies cite the defining point, so `grund refs
<point>` lists the owner and the copies left to remove, and nothing else. The
manifest, `citation_ledger.toml`, names each family's points, its owner modules
and its migration rows, each row naming the step of #576 that removes it. The
ledger fails on a new violation, a code file citing a point that is neither an
owner nor a row, and on a row that no longer applies: an owner or a row that
cites no point of its family, a point that does not resolve, or a row left under
a family marked done. The inventory names the manifest as its list."""

import functools
import json
import os
import re
import subprocess
import tomllib
import unittest
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MANIFEST = Path(__file__).resolve().parent / "citation_ledger.toml"
EXE = "grund.exe" if os.name == "nt" else "grund"
STEP = re.compile(r"^#(59[5-9]|600|655)$")


@functools.lru_cache(maxsize=None)
def _grund():
    """The binary under test: the gate's build, or this checkout's built once."""
    built = os.environ.get("GRUND_BUILT")
    if built:
        return built
    target = Path(os.environ.get("CARGO_TARGET_DIR") or REPO_ROOT / "target")
    subprocess.run([os.environ.get("CARGO", "cargo"), "build", "-p", "grund", "--locked"],
                   cwd=REPO_ROOT, check=True)
    return str(target / "debug" / EXE)


def is_code(path):
    """A production source file: under a crate's `src/`, and not a test module."""
    name = path.rsplit("/", 1)[-1]
    return (path.startswith("crates/") and "/src/" in path and "/tests/" not in path
            and not name.startswith("tests_") and name != "testing.rs")


def _citers(citations, point):
    """The files whose citations name the point or a section beneath it."""
    id_, _, section = point.partition(".")
    return {c["path"] for c in citations if c["id"] == id_
            and (not section or (c["section"] or "") == section
                 or (c["section"] or "").startswith(section + "."))}


def _found(points):
    """What `look(point)` answers for every point, from two scans run side by side:
    `grund cover` dumps every citation of the tree, `grund show --batch` says which
    points resolve, rather than a `refs` and a `show` per point."""
    def run(args, stdin=None):
        return subprocess.run([_grund(), *args], cwd=REPO_ROOT, input=stdin,
                              capture_output=True, text=True).stdout
    queries = "".join(json.dumps({"id": p.partition(".")[0], "section": p.partition(".")[2] or None})
                      + "\n" for p in points)
    with ThreadPoolExecutor(max_workers=2) as pool:
        cover = pool.submit(run, ["cover", "--format", "json"])
        shown = pool.submit(run, ["show", "--batch", "--format", "json", "--brief"], queries)
        citations = [c for line in cover.result().splitlines() if line.strip()
                     for c in json.loads(line)["citations"]]
        resolved = [json.loads(line)["ok"] for line in shown.result().splitlines() if line.strip()]
    if len(resolved) != len(points):
        raise AssertionError(f"show --batch answered {len(resolved)} of {len(points)} points")
    return {p: (ok, _citers(citations, p)) for p, ok in zip(points, resolved)}


def problems(families, look):
    """What is wrong with the manifest's `families` against what `look(point)` finds."""
    wrong = []
    for family in families:
        name = family["name"]
        owners = set(family.get("owners", []))
        rows = {row["file"]: row for row in family.get("migration", [])}
        if not owners:
            wrong.append(f"{name}: names no owner; the one module that derives the fact is its owner")
        if family.get("done") and rows:
            wrong.append(f"{name}: marked done with {len(rows)} migration row(s) left")
        for file, row in sorted(rows.items()):
            if not STEP.match(row.get("step", "")):
                wrong.append(f"{name}: row {file} names no step of #576 (#595-#600, #655)")
            if file in owners:
                wrong.append(f"{name}: {file} is both an owner and a migration row")
        cited_by = set()
        for point in family["points"]:
            resolves, citers = look(point)
            if not resolves:
                wrong.append(f"{name}: §{point} does not resolve")
            cited_by |= citers
            for file in sorted(f for f in citers if is_code(f)):
                if file not in owners and file not in rows:
                    wrong.append(f"{name}: {file} cites §{point} and is neither an owner nor a "
                                 "migration row: a second derivation, or a row to add")
        for file in sorted(owners - cited_by):
            wrong.append(f"{name}: owner {file} cites none of the family's points")
        for file in sorted(set(rows) - cited_by):
            wrong.append(f"{name}: migration row {file} no longer cites the family: take it out")
    return wrong


def manifest():
    return tomllib.loads(MANIFEST.read_text(encoding="utf-8"))["family"]


class CitationLedgerTests(unittest.TestCase):
    def test_every_points_citers_are_its_owner_and_its_migration_rows(self):
        families = manifest()
        found = _found([point for family in families for point in family["points"]])
        wrong = problems(families, found.__getitem__)
        self.assertEqual([], wrong, "\n" + "\n".join(wrong))

    def test_the_inventory_names_the_manifest_as_its_list(self):
        inventory = subprocess.run([_grund(), "AR-system.principle.inventory", "--full"],
                                   cwd=REPO_ROOT, capture_output=True, text=True,
                                   check=True).stdout
        self.assertIn("tests/integration/citation_ledger.toml", inventory)
        self.assertNotIn("DA-one-fact-one-point-one-implementation.consequences", inventory)


class CitationLedgerHarnessTests(unittest.TestCase):
    """The ledger fails on a new violation and on a row that no longer applies."""

    OWNER = "crates/grund-core/src/scanner/a.rs"
    COPY = "crates/grund-core/src/api/b.rs"

    def family(self, **extra):
        return dict({"name": "f", "points": ["FS-p"], "owners": [self.OWNER],
                     "migration": [{"file": self.COPY, "step": "#595"}]}, **extra)

    def check(self, citers, resolves=True, **extra):
        return problems([self.family(**extra)], lambda point: (resolves, set(citers)))

    def test_owner_and_rows_citing_pass(self):
        self.assertEqual([], self.check([self.OWNER, self.COPY, "docs/x.md",
                                         "crates/grund-core/src/api/tests_b.rs"]))

    def test_an_extra_code_citer_fails(self):
        wrong = self.check([self.OWNER, self.COPY, "crates/grund-core/src/model/c.rs"])
        self.assertIn("model/c.rs cites \u00a7FS-p and is neither", " ".join(wrong))

    def test_a_row_that_no_longer_cites_fails(self):
        self.assertIn("no longer cites", " ".join(self.check([self.OWNER])))

    def test_an_owner_that_does_not_cite_fails(self):
        self.assertIn("cites none of the family's points", " ".join(self.check([self.COPY])))

    def test_a_row_under_a_done_family_fails(self):
        self.assertIn("marked done", " ".join(self.check([self.OWNER, self.COPY], done=True)))

    def test_a_row_without_its_step_fails(self):
        wrong = self.check([self.OWNER, self.COPY], migration=[{"file": self.COPY, "step": "later"}])
        self.assertIn("names no step", " ".join(wrong))

    def test_a_point_that_does_not_resolve_fails(self):
        self.assertIn("does not resolve", " ".join(self.check([self.OWNER, self.COPY], False)))

    def test_a_family_without_an_owner_fails(self):
        self.assertIn("names no owner", " ".join(self.check([self.COPY], owners=[])))


if __name__ == "__main__":
    unittest.main()
