"""§FS-terms.senses — the content half of the functional spec's Terms chapters,
beside the presence half `test_functional_spec_terms.py` holds. Two halves of one
subject, and they fail for different reasons: a chapter that is missing, against a
chapter whose words do not add up.

Four fixed-syntax invariants gate over `docs/functional-spec/` — shared labels
unique, every parenthesised word a label of the group its line cites, local labels
unique per document, and a shared word redefined only in the `(narrowed)` form. All
four pass over the tree as it stands, which is what prevention looks like, so each
one is also proved to fire against a fixture home built to break exactly it.

The advisory report is the other half and it never gates. Its contract is pinned on
a fixture whose answer cannot move when the specification's prose is edited, and
nothing here asserts over what it finds in the live tree: a frozen count would make
it a gate under another name, which §FS-terms.senses.5 forbids.

Run the report with `python3 tests/integration/test_functional_spec_terms_content.py
--advisory`; `--home` points it at another tree.
"""

import importlib
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
INTEGRATION = Path(__file__).resolve().parent
FUNCTIONAL_SPEC = REPO_ROOT / "docs" / "functional-spec"
THIS_FILE = Path(__file__).resolve()
MODULE_NAME = THIS_FILE.stem
# The analysis lives beside this file rather than in it, so the contract below can be
# read and committed before the thing it describes exists. `unittest discover` runs
# `test_*.py` only, so naming it this way keeps it out of collection.
VOCABULARY_MODULE = "terms_vocabulary"
GATE_CLASS = "FunctionalSpecTermsInvariantTests"

# The marker stays out of every literal a scan would read as a citation of a group:
# such a citation from tests/integration/ is coverage evidence for that leaf, and
# every `FS-terms.terms.<N>` leaf is a listed permanent exception.
MARKER = "§"

# The four, in the order the report prints them and the order this file asserts them.
INVARIANTS = (
    "shared labels unique",
    "lean words exist in the cited group",
    "local labels unique per document",
    "shared/local collision only as `(narrowed)`",
)

# The two line forms §FS-terms.senses.5 fixes, as a reader of the report sees them.
USES = re.compile(r"^\S+: uses \*[^*]+\* in prose but does not lean on it \(terms\.\d+\)$")
LEANS = re.compile(r"^\S+: leans on \*[^*]+\* but the token appears nowhere in its prose$")


def vocabulary():
    """The module parts (b) and (c) live in. Absent until agent-grounds/grund#290
    implements it, which is why every assertion that calls it is an expected
    failure today."""
    return importlib.import_module(VOCABULARY_MODULE)


# ---------------------------------------------------------------------------
# The fixture home. Two documents and a vocabulary, written so that the report
# over them is four lines and every one of them is argued for.
# ---------------------------------------------------------------------------

INDEX = """# Fixture home

- [{m}FS-first](FS-first.md) — the first document.
- [{m}FS-second](FS-second.md) — the second document.
- [{m}FS-terms](FS-terms.md) — the vocabulary itself, which is not a document.
"""

VOCABULARY = """# FS-terms: the fixture vocabulary

## terms: Terms

**One definition per word.**

### terms.1: Declarations and prose

- **declaration** — A heading that introduces an ID.
- **note** — An inline comment block carrying a citation and its rationale.
- **level** — The strength a rule is held at.

### terms.2: Findings

- **anchor** — A Markdown heading fragment only. Never where a finding is located.
- **catalog** — The scan's shared set of declarations.
"""

# *level* stands on its own here and is not leaned on, so it is reported; *declaration*
# is both used and leaned on, so it is not; *anchor* occurs only inside a fence and a
# code span, so §FS-terms.senses.4 keeps it out.
FIRST = """# FS-first: the first fixture document

## terms: Terms

Leans on {m}FS-terms.terms.1 (declaration).

## 1. What it says

A declaration carries the fact. The level a rule is held at decides whether a run
fails or merely says so.

```text
anchor
```

The `anchor` spelling is a name, not a use.
"""

# *note* and *anchor* stand in the senses §FS-terms.senses.1 and §FS-terms.senses.3 exclude,
# so both are reported; *level* and *declaration* occur only inside compounds; *catalog* is
# leaned on and occurs only where §FS-terms.senses.4 says a token names rather than uses.
SECOND = """# FS-second: the second fixture document

## terms: Terms

Leans on {m}FS-terms.terms.2 (catalog).

## 1. What it says

The run prints a note: line before the hint, and a CLI-level error after it. A
declaration-local override wins over the file's own. Every finding takes its anchor
from the site it is located at.

See [the rows](../ids.md#the-catalog-of-rows) and the `catalog` key.

```text
catalog
```
"""

REPORT = (
    "FS-first: uses *level* in prose but does not lean on it (terms.1)",
    "FS-second: uses *note* in prose but does not lean on it (terms.1)",
    "FS-second: uses *anchor* in prose but does not lean on it (terms.2)",
    "FS-second: leans on *catalog* but the token appears nowhere in its prose",
)


def write_home(directory, index=INDEX, vocabulary=VOCABULARY, first=FIRST, second=SECOND):
    """One fixture home on disk, each file taking the marker it is given so that no
    literal in this file reads as a citation of a vocabulary group."""
    home = Path(directory)
    for name, template in (
        ("README.md", index),
        ("FS-terms.md", vocabulary),
        ("FS-first.md", first),
        ("FS-second.md", second),
    ):
        (home / name).write_text(template.format(m=MARKER), encoding="utf-8")
    return home


class FixtureHomeMixin(unittest.TestCase):
    def home(self, **files):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        return write_home(directory.name, **files)

    def findings(self, home):
        """The four, as {title: findings}, with the order asserted once here."""
        reported = vocabulary().invariant_findings(home)
        self.assertEqual([title for title, _ in reported], list(INVARIANTS))
        return dict(reported)

    def assertOnlyFires(self, home, invariant, pattern):
        found = self.findings(home)
        self.assertRegex("\n".join(found[invariant]), pattern)
        self.assertEqual(1, len(found[invariant]), found[invariant])
        for title in INVARIANTS:
            if title != invariant:
                self.assertEqual([], found[title], title)


class FunctionalSpecTermsInvariantTests(FixtureHomeMixin):
    """Part (b), over the tree it gates. Clean today and meant to stay clean: what
    this class buys is the next rename, not this one."""

    @unittest.expectedFailure  # until #290 lands tests/integration/terms_vocabulary.py
    def test_the_four_invariants_hold_over_the_functional_spec(self):
        found = self.findings(FUNCTIONAL_SPEC)
        self.assertEqual(
            {title: [] for title in INVARIANTS},
            found,
            "\n".join(line for findings in found.values() for line in findings),
        )

    @unittest.expectedFailure  # until #290 lands tests/integration/terms_vocabulary.py
    def test_the_functional_spec_home_is_actually_read(self):
        self.assertGreaterEqual(
            len(vocabulary().documents(FUNCTIONAL_SPEC)), 20, "index entries not found"
        )

    @unittest.expectedFailure  # until #290 lands tests/integration/terms_vocabulary.py
    def test_a_clean_fixture_home_fires_none_of_the_four(self):
        self.assertEqual({title: [] for title in INVARIANTS}, self.findings(self.home()))


class EachInvariantFiresTests(FixtureHomeMixin):
    """A check nobody has seen fail is not a check: one fixture per invariant, each
    breaking exactly it and nothing else."""

    @unittest.expectedFailure  # until #290 lands tests/integration/terms_vocabulary.py
    def test_one_word_under_two_groups(self):
        twice = VOCABULARY.replace(
            "- **catalog** — The scan's shared set of declarations.",
            "- **catalog** — The scan's shared set of declarations.\n- **note** — Said twice.",
        )
        self.assertOnlyFires(
            self.home(vocabulary=twice),
            INVARIANTS[0],
            r"FS-terms\.md:\d+: \*note\* is defined under terms\.2 "
            r"and already under terms\.1 \(line \d+\)",
        )

    @unittest.expectedFailure  # until #290 lands tests/integration/terms_vocabulary.py
    def test_a_leaned_word_the_cited_group_does_not_define(self):
        wrong = FIRST.replace("terms.1 (declaration)", "terms.1 (anchor)")
        self.assertOnlyFires(
            self.home(first=wrong),
            INVARIANTS[1],
            r"FS-first(\.md)?.*leans on terms\.1 for \*anchor\*, "
            r"which that group does not define",
        )

    @unittest.expectedFailure  # until #290 lands tests/integration/terms_vocabulary.py
    def test_one_word_defined_twice_in_a_document(self):
        twice = FIRST.replace(
            "## 1. What it says",
            "- **scope** — The files a run reads.\n"
            "- **scope** — Said twice.\n\n## 1. What it says",
        )
        self.assertOnlyFires(
            self.home(first=twice),
            INVARIANTS[2],
            r"FS-first\.md:\d+: FS-first defines \*scope\* twice in its own Terms chapter "
            r"\(first at line \d+\)",
        )

    @unittest.expectedFailure  # until #290 lands tests/integration/terms_vocabulary.py
    def test_a_shared_word_redefined_without_the_narrowed_form(self):
        bare = FIRST.replace(
            "## 1. What it says",
            "- **declaration** — Redefined flat.\n\n## 1. What it says",
        )
        self.assertOnlyFires(
            self.home(first=bare),
            INVARIANTS[3],
            r"FS-first\.md:\d+: FS-first redefines the shared word \*declaration\* "
            r"without the `\(narrowed\)` form",
        )

    @unittest.expectedFailure  # until #290 lands tests/integration/terms_vocabulary.py
    def test_the_narrowed_form_of_the_same_word_fires_nothing(self):
        narrowed = FIRST.replace(
            "## 1. What it says",
            "- **declaration (narrowed)** — "
            + MARKER
            + "FS-terms.terms.1. In this document, within the fixture, it is the "
            "heading alone; elsewhere the shared definition applies.\n\n## 1. What it says",
        )
        self.assertEqual(
            {title: [] for title in INVARIANTS}, self.findings(self.home(first=narrowed))
        )


class AdvisoryReportTests(FixtureHomeMixin):
    """Part (c), on the fixture and nowhere else. Every assertion here is about the
    fixture's four lines; what the report says about `docs/functional-spec/` is read
    by an author and asserted by nobody."""

    def report(self):
        return list(vocabulary().advisory_report(self.home()))

    @unittest.expectedFailure  # until #290 lands tests/integration/terms_vocabulary.py
    def test_a_different_sense_is_reported_because_no_machine_sees_it(self):
        """§FS-terms.senses.1 — `note:` before a hint line is not the word's shared
        sense, and the report cannot tell. It names the occurrence as a use, which is
        the false line a reader discards and the reason the report never gates."""
        self.assertIn("FS-second: uses *note* in prose but does not lean on it (terms.1)",
                      self.report())

    @unittest.expectedFailure  # until #290 lands tests/integration/terms_vocabulary.py
    def test_a_compound_is_not_a_use_of_the_words_in_it(self):
        """§FS-terms.senses.2 — in both directions: *level* inside "CLI-level" and
        *declaration* inside *declaration-local* are each one name. The same *level*
        standing on its own in the other document is reported, so what the exclusion
        turns on is the compound and not the word."""
        report = self.report()
        self.assertIn("FS-first: uses *level* in prose but does not lean on it (terms.1)",
                      report)
        self.assertEqual([], [line for line in report if line.startswith("FS-second: uses *level*")])
        self.assertEqual([], [line for line in report if "*declaration*" in line])

    @unittest.expectedFailure  # until #290 lands tests/integration/terms_vocabulary.py
    def test_a_retired_sense_is_reported_because_no_machine_sees_it_either(self):
        """§FS-terms.senses.3 — *anchor* standing where a finding is located is the
        sense its own row retires, not a lean the lean line is missing. Mechanically
        indistinguishable from a use, so the report names it."""
        self.assertIn("FS-second: uses *anchor* in prose but does not lean on it (terms.2)",
                      self.report())

    @unittest.expectedFailure  # until #290 lands tests/integration/terms_vocabulary.py
    def test_a_token_that_names_rather_than_uses_is_not_a_use(self):
        """§FS-terms.senses.4 — *catalog* occurs in a link target, a heading anchor, a
        fenced block and a code span and nowhere else, so the document that leans on it
        is reported as not using it; *anchor* in the other document occurs only in a
        fence and a code span, so it is not reported as used."""
        report = self.report()
        self.assertIn("FS-second: leans on *catalog* but the token appears nowhere in its prose",
                      report)
        self.assertEqual([], [line for line in report if line.startswith("FS-first: uses *anchor*")])

    @unittest.expectedFailure  # until #290 lands tests/integration/terms_vocabulary.py
    def test_the_report_has_two_line_forms_in_a_fixed_order(self):
        """§FS-terms.senses.5 — every line of the first form before every line of the
        second, documents ascending, and inside a document the shared words in group
        order and then in row order."""
        report = self.report()
        self.assertEqual(list(REPORT), report)
        self.assertTrue(all(USES.match(line) for line in report[:3]), report)
        self.assertTrue(LEANS.match(report[3]), report)


class AdvisoryIsAskedForTests(FixtureHomeMixin):
    """The report is a surface of this test program and of nothing else: it is printed
    only when it is asked for, it prints the same bytes twice, and it is never an exit
    condition."""

    def run_module(self, *arguments, cwd=INTEGRATION):
        return subprocess.run(
            [sys.executable, *arguments],
            capture_output=True,
            text=True,
            cwd=str(cwd),
        )

    def advisory(self, home):
        return self.run_module(str(THIS_FILE), "--advisory", "--home", str(home))

    @unittest.expectedFailure  # until #290 lands tests/integration/terms_vocabulary.py
    def test_two_runs_over_one_tree_print_the_same_bytes(self):
        """§FS-terms.senses.5 — the fixed order is what makes the report comparable
        between two runs, which is the property §REQ-deterministic-output asks of every
        surface this repository prints."""
        home = self.home()
        first, second = self.advisory(home), self.advisory(home)
        self.assertEqual(0, first.returncode, first.stderr)
        self.assertEqual(first.stdout, second.stdout)
        self.assertEqual(list(REPORT), first.stdout.splitlines())

    @unittest.expectedFailure  # until #290 lands tests/integration/terms_vocabulary.py
    def test_the_report_is_silent_unless_it_is_asked_for(self):
        """§FS-terms.senses.5 — the gating run prints no line of either form and its
        verdict does not turn on the report, so the hook that runs this suite on every
        commit does not grow the report's lines."""
        asked = self.advisory(self.home())
        self.assertTrue(asked.stdout.splitlines(), "--advisory printed nothing")
        gate = self.run_module("-m", "unittest", f"{MODULE_NAME}.{GATE_CLASS}")
        self.assertEqual(0, gate.returncode, gate.stderr)
        printed = gate.stdout.splitlines() + gate.stderr.splitlines()
        self.assertEqual(
            [], [line for line in printed if USES.match(line) or LEANS.match(line)]
        )


class ContractShapeTests(unittest.TestCase):
    """What holds while the module itself is still missing."""

    def test_the_analysis_module_is_not_collected_as_a_test(self):
        self.assertFalse(VOCABULARY_MODULE.startswith("test_"))
        self.assertFalse((INTEGRATION / f"test_{VOCABULARY_MODULE}.py").exists())

    def test_the_fixture_home_names_two_documents_and_a_vocabulary(self):
        with tempfile.TemporaryDirectory() as directory:
            home = write_home(directory)
            self.assertEqual(
                ["FS-first.md", "FS-second.md", "FS-terms.md", "README.md"],
                sorted(path.name for path in home.iterdir()),
            )


def _advisory(argv):
    home = Path(argv[argv.index("--home") + 1]) if "--home" in argv else FUNCTIONAL_SPEC
    for line in vocabulary().advisory_report(home):
        print(line)
    return 0


if __name__ == "__main__":
    if "--advisory" in sys.argv:
        sys.exit(_advisory(sys.argv))
    unittest.main()
