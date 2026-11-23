"""The spec fold pass, run on a small repository it makes: §REQ-spec-section-names.permanence.

A folded section leaves no pointer, so every site that names it moves in the same change,
including the coverage gate's evidence (§AR-goal-measurement.1). Each test builds a base
commit with a page `FS-demo`, folds and moves sections out of it in the working tree, and
runs `scripts/move_spec_sections.py` with a rewrite record: it holds what each surface
becomes, what the pass refuses before it writes, what it reports for a person, what
`--check` fails on, and that the fixed table the script began with still moves as it did.
"""

import contextlib
import io
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT / "scripts"))
import move_spec_sections  # noqa: E402
import spec_fold_coverage  # noqa: E402

# The marker and its escape, spelled so that this file cites nothing the fixture declares.
M = "\N{SECTION SIGN}"
ESC = "<" + M + ">"

BASE_PAGE = """# FS-demo: Demo

Lead.

## terms: Terms

None.

## 1. Reading

Text of one.

### 1.1 Brief

Brief text.

#### 1.1.1 With no lead

Detail.

### 1.2 Kept

Kept text.

## 2. Writing

Two.

### 2.1 Moved away

Moved text.

## 3. Lists

Three.

### 3.1 Only child

Child.
"""

FOLDED_PAGE = f"""# FS-demo: Demo

Lead.

## terms: Terms

None.

## 1. Reading

Text of one. Brief text. Detail.

### 1.2 Kept

Kept text. See [{M}FS-demo.2.1](#21-moved-away).

## 2. Writing

Two.

## 3. Lists

Three. Child.
"""

OTHER_PAGE = """# FS-other: Other

## terms: Terms

None.

## 1. Home

Home. Moved text.
"""

RECORD = """# FS-demo rewrite record

## size: 60 -> 40 words, 400 -> 300 bytes

## folds

- `FS-demo.1.1 -> FS-demo.1`
- FS-demo.1.1.1 -> FS-demo.1
- FS-demo.3.1 -> FS-demo.3

## moves

- FS-demo.2.1 -> FS-other.1  # the fact is the other page's

## anchors

none

## pins

- tests/integration/test_demo_moves.py: KEPT row 1.1 | kept verbatim
"""

GUIDE = f"""See {M}FS-demo.1.2 and [{M}FS-demo.1.1](functional-spec/FS-demo.md#11-brief).
Pinned [x](https://github.com/agent-grounds/grund/blob/abc123/docs/functional-spec/FS-demo.md#11-brief).
Self [y](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-demo.md#111-with-no-lead).
Plain [z](functional-spec/FS-demo.md#21-moved-away).
Escaped {ESC}FS-demo.1.1 stays.
Bare FS-demo.3.1 is reported.
Run {M}FS-demo.1.2, .1.1 and .2 is reported.
"""

CHANGELOG = f"""# Changelog

Conventions cite {M}FS-demo.1.1.

## [0.2.0]

- {M}FS-demo.1.1 changed ([{M}FS-demo.2.1](functional-spec/FS-demo.md#21-moved-away)).
"""

MANIFEST = "FS-demo.1.1\nFS-demo.1\nFS-001-demo.1.1.1\nFS-demo.1.2 FS-demo.3.1\n"


def fact(manifest: str, note: str) -> str:
    data = ",".join(str(b) for b in manifest.encode("utf-8"))
    return (
        '{\n  "inputFiles": {\n    "spec.refs": {"bytes":[' + data + ']}\n  },\n'
        f'  "facts": {{"authoredFrom": "{note}"}}\n}}\n'
    )


SCHEMA = {
    "properties": {
        "a": {"x-grund-spec": {"id": "FS-demo.1.1", "href": "docs/functional-spec/FS-demo.md#11-brief"}},
        "b": {"x-grund-spec": {"id": "FS-demo.1.2", "href": "docs/functional-spec/FS-demo.md#12-kept"}},
    }
}

LEDGER = """[[entry]]
done = false
points = [
  "FS-demo.1",
  "FS-demo.1.1",
  "FS-other.1",
]
inline = ["FS-demo.3.1", "FS-demo.3"]
"""

POINTERS = """POINTERS = (
    ("FS-other.1",
     ("FS-demo.1.1", "FS-demo.1"), 100,
     ("fragment",)),
)
"""

MOVES_TEST = """CHAPTERS = (
    "1", "1.1",
    "2", "3.1",
)
KEPT = (("FS-demo.md", "1.1", "Brief text"),)
"""

POLICY = """pub(super) const PERMANENT_EXCEPTIONS: &[Exception<'static>] = &[
    Exception { id: "FS-demo.1.1.1", reason: "detail" },
    Exception { id: "FS-demo.3.1", reason: "rationale" },
    Exception { id: "FS-demo.1.2", reason: "kept" },
];

#[rustfmt::skip]
pub(super) const LEAD_PROMISES: &[LeadPromise<'static>] = &[
    LeadPromise { id: "FS-demo.1.1", reason: "brief" },
    LeadPromise { id: "FS-demo.3", reason: "lists" },
];
"""

NOTES = """
## notes

Prose a reviewer reads: FS-demo.1.1 went into its parent.

## overrides

- docs/guide.md:1: FS-demo.1.2 -> FS-demo.1  # the guide means the reading
- crates/x.rs:3 (FS-demo.3.1, "a row") -> FS-demo.3
Prose between overrides is ignored.

## lead-promises

- FS-demo.1: a reading prints the lead of its section
- FS-demo.1: listed twice, kept once
Prose between promises is ignored.
"""

QUOTED = f"""Subjects:

```text
abc123 Fold {M}FS-demo.1.1 into [its parent](../functional-spec/FS-demo.md#11-brief)
```

Then {M}FS-demo.1.1 outside.
"""

REPEATS = f"""See {M}FS-demo.1.1 and {M}FS-demo.1.1.1.
Both ({M}FS-demo.1; {M}FS-demo.1.1) hold.
Linked [{M}FS-demo.1.1](functional-spec/FS-demo.md#11-brief), {M}FS-demo.1.1.1.
Authored {M}FS-demo.1.2, {M}FS-demo.1.2 stays.
Ends {M}FS-demo.1.1,
{M}FS-demo.1.1.1 opens.
"""

SOURCE = f"""//! Demo: {M}FS-demo.1.1.
/// ```text
/// {M}FS-demo.1.1
/// ```
pub fn f() -> &'static str {{
    "{M}FS-demo.1.1, {M}FS-demo.1.1.1"
}}

#[cfg(test)]
mod tests {{
    // {M}FS-demo.3.1
    #[test]
    fn t() {{}}
}}
"""

RENDER = """from pathlib import Path
root = Path(__file__).resolve().parents[1]
(root / "docs" / "configuration.md").write_text("rendered\\n")
"""

BASE = {
    "docs/functional-spec/FS-demo.md": BASE_PAGE,
    "docs/functional-spec/FS-other.md": OTHER_PAGE.replace(" Moved text.", ""),
    "docs/guide.md": GUIDE,
    "docs/changelog.md": CHANGELOG,
    "docs/changelog/0.1.0.md": f"- {M}FS-demo.3.1 added.\n",
    "tests/e2e/cases/case-a/spec.refs": MANIFEST,
    "tests/e2e/cases/case-a/repo/notes.md": f"{M}FS-demo.1.1\n",
    "tests/e2e/cases/case-b/spec.refs": "FS-demo.1.2\n",
    "tests/integration/command_agreement_facts/case-a.json": fact(MANIFEST, f"cites {M}FS-demo.2.1 here"),
    "tests/integration/command_agreement_facts/case-b.json": fact("FS-demo.1.2\n", "kept"),
    "schemas/grund.schema.json": json.dumps(SCHEMA, indent=2) + "\n",
    "scripts/render_config_reference.py": RENDER,
    "tests/integration/citation_ledger.toml": LEDGER,
    "tests/integration/test_pointer_guard.py": POINTERS,
    "tests/integration/test_demo_moves.py": MOVES_TEST,
    "tests/integration/functional_spec_coverage_policy.rs": POLICY,
}


def git(root: Path, *arguments: str) -> str:
    return subprocess.run(["git", "-C", str(root), *arguments], capture_output=True, check=True, text=True).stdout


class FoldPassTest(unittest.TestCase):
    def setUp(self):
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.root = Path(scratch.name) / "repo"
        for name, text in BASE.items():
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text, encoding="utf-8")
        git(self.root.parent, "init", "-q", str(self.root))
        git(self.root, "add", "-A")
        git(self.root, "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-m", "base")
        self.base = git(self.root, "rev-parse", "HEAD").strip()
        self.write("docs/functional-spec/FS-demo.md", FOLDED_PAGE)
        self.write("docs/functional-spec/FS-other.md", OTHER_PAGE)
        self.record = Path(scratch.name) / "FS-demo.md"
        self.record.write_text(RECORD, encoding="utf-8")

    def write(self, name: str, text: str) -> None:
        (self.root / name).write_text(text, encoding="utf-8")

    def read(self, name: str) -> str:
        return (self.root / name).read_text(encoding="utf-8")

    def track(self, name: str, text: str) -> None:
        (self.root / name).parent.mkdir(parents=True, exist_ok=True)
        self.write(name, text)
        git(self.root, "add", name)

    def run_pass(self, *extra: str, record: Path | None = None) -> tuple[int, str]:
        arguments = ["--root", str(self.root), "--base", self.base, "--record", str(record or self.record), *extra]
        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            code = move_spec_sections.main(arguments)
        return code, out.getvalue()

    def test_every_mechanical_surface_is_rewritten(self):
        code, out = self.run_pass()
        self.assertEqual(code, 0, out)
        guide = self.read("docs/guide.md").split("\n")
        self.assertEqual(guide[0], f"See {M}FS-demo.1.2 and [{M}FS-demo.1](functional-spec/FS-demo.md#1-reading).")
        self.assertIn("/blob/abc123/docs/functional-spec/FS-demo.md#11-brief", guide[1])
        self.assertIn("/blob/main/docs/functional-spec/FS-demo.md#1-reading)", guide[2])
        self.assertIn("(functional-spec/FS-other.md#1-home)", guide[3])
        self.assertEqual(guide[4], f"Escaped {ESC}FS-demo.1.1 stays.")
        self.assertIn(f"See [{M}FS-other.1](FS-other.md#1-home).", self.read("docs/functional-spec/FS-demo.md"))
        changelog = self.read("docs/changelog.md")
        self.assertIn(f"Conventions cite {M}FS-demo.1.\n", changelog)
        self.assertIn(f"- {ESC}FS-demo.1.1 changed ([{ESC}FS-demo.2.1](functional-spec/FS-other.md#1-home)).", changelog)
        self.assertEqual(self.read("docs/changelog/0.1.0.md"), f"- {ESC}FS-demo.3.1 added.\n")
        self.assertEqual(self.read("tests/e2e/cases/case-a/repo/notes.md"), f"{M}FS-demo.1.1\n")
        manifest = "FS-demo.1\nFS-demo.1.2 FS-demo.3\n"
        self.assertEqual(self.read("tests/e2e/cases/case-a/spec.refs"), manifest)
        self.assertEqual(self.read("tests/integration/command_agreement_facts/case-a.json"),
                         fact(manifest, f"cites {M}FS-other.1 here"))
        self.assertEqual(self.read("tests/integration/command_agreement_facts/case-b.json"), fact("FS-demo.1.2\n", "kept"))
        schema = json.loads(self.read("schemas/grund.schema.json"))["properties"]
        self.assertEqual(schema["a"]["x-grund-spec"], {"id": "FS-demo.1", "href": "docs/functional-spec/FS-demo.md#1-reading"})
        self.assertEqual(schema["b"], SCHEMA["properties"]["b"])
        self.assertEqual(self.read("docs/configuration.md"), "rendered\n")
        self.assertEqual(self.read("tests/integration/citation_ledger.toml"),
                         LEDGER.replace('  "FS-demo.1.1",\n', "").replace('"FS-demo.3.1", "FS-demo.3"', '"FS-demo.3"'))
        self.assertIn('     ("FS-demo.1",), 100,', self.read("tests/integration/test_pointer_guard.py"))
        self.assertIn('CHAPTERS = (\n    "1",\n    "2", "3",\n)', self.read("tests/integration/test_demo_moves.py"))
        self.assertEqual(self.read("tests/integration/functional_spec_coverage_policy.rs"), POLICY
                         .replace('    Exception { id: "FS-demo.1.1.1", reason: "detail" },\n', "")
                         .replace('"FS-demo.3.1", reason: "rationale"', '"FS-demo.3", reason: "rationale"')
                         .replace('"FS-demo.1.1", reason: "brief" },\n    LeadPromise { id: "FS-demo.3", reason: "lists"',
                                  '"FS-demo.1", reason: "brief"'))

    def test_what_is_not_mechanical_is_reported(self):
        code, out = self.run_pass()
        self.assertEqual(code, 0, out)
        report = out.split("need a hand edit:", 1)[1]
        for site in (
            "docs/guide.md:6: `FS-demo.3.1` is written bare; it is now `FS-demo.3`",
            "docs/guide.md:7: `.1.1` in a run after a citation names `FS-demo.1.1`",
            "tests/integration/test_demo_moves.py:5: names `1.1` of `FS-demo`",
            "functional_spec_coverage_policy.rs: added the exception row of `FS-demo.3`",
            "pins: tests/integration/test_demo_moves.py: KEPT row 1.1 | kept verbatim",
            "the lead promise of the removed `FS-demo.1.1` is now `FS-demo.1`'s",
            "dropped the lead promise of `FS-demo.3`: `FS-demo.3` is now a leaf",
        ):
            self.assertIn(site, report)
        self.assertTrue(out.rstrip().endswith("now run `grund fmt --write` so every link target and anchor follows"))

    def test_a_dry_run_writes_nothing(self):
        code, out = self.run_pass("--dry-run")
        self.assertEqual(code, 0, out)
        self.assertIn("would change", out)
        self.assertEqual(self.read("docs/guide.md"), GUIDE)
        self.assertFalse((self.root / "docs/configuration.md").exists())

    def test_an_untracked_file_holding_a_removed_coordinate_stops_the_pass(self):
        self.write("docs/new.md", f"{M}FS-demo.1.1\n")
        code, out = self.run_pass()
        self.assertEqual(code, 1, out)
        self.assertIn("git add -N docs/new.md", out)
        self.assertEqual(self.read("docs/guide.md"), GUIDE)

    def test_an_unsafe_record_is_refused_before_anything_is_written(self):
        unsafe = self.record.with_name("unsafe.md")
        listed_as_fold = (RECORD.replace("- FS-demo.2.1 -> FS-other.1  # the fact is the other page's", "none")
                          .replace("- FS-demo.3.1 -> FS-demo.3\n", "- FS-demo.3.1 -> FS-demo.3\n- FS-demo.2.1 -> FS-other.1\n"))
        unsafe.write_text(listed_as_fold, encoding="utf-8")
        code, out = self.run_pass(record=unsafe)
        self.assertEqual(code, 2, out)
        self.assertIn("a fold names its nearest surviving ancestor", out)
        self.assertEqual(self.read("docs/guide.md"), GUIDE)
        garbled = self.record.with_name("garbled.md")
        garbled.write_text(RECORD.replace("- FS-demo.3.1 -> FS-demo.3", "- FS-demo.3.1 into FS-demo.3"), encoding="utf-8")
        code, out = self.run_pass(record=garbled)
        self.assertEqual(code, 2, out)
        self.assertIn("garbled.md:9: under `## folds` only `<old> -> <target>` lines", out)

    def test_check_fails_on_a_removed_section_no_record_lists(self):
        code, out = self.run_pass("--check")
        self.assertEqual(code, 0, out)
        self.assertIn("check ok: 4 removed coordinate(s) recorded", out)
        partial = self.record.with_name("partial.md")
        partial.write_text(RECORD.replace("- FS-demo.3.1 -> FS-demo.3\n", ""), encoding="utf-8")
        code, out = self.run_pass("--check", record=partial)
        self.assertEqual(code, 1, out)
        self.assertIn("FS-demo.3.1: removed, and in no record", out)
        self.assertEqual(self.read("docs/guide.md"), GUIDE)

    def test_a_record_holds_notes_overrides_and_lead_promises(self):
        self.track("tests/integration/functional_spec_coverage_policy.rs", POLICY.split("    LeadPromise")[0] + "];\n")
        noted = self.record.with_name("noted.md")
        noted.write_text(RECORD + NOTES, encoding="utf-8")
        code, out = self.run_pass(record=noted)
        self.assertEqual(code, 0, out)
        self.assertIn("docs/guide.md:1: write `FS-demo.1` for `FS-demo.1.2`; no record removes it", out)
        self.assertIn("# the guide means the reading", out)
        self.assertIn(f'overrides: crates/x.rs:3 (FS-demo.3.1, "a row") -> FS-demo.3', out)
        self.assertNotIn("Prose", out)
        self.assertIn("1 lead promise(s) added", out)
        self.assertTrue(self.read("tests/integration/functional_spec_coverage_policy.rs").endswith(
            '&[\n    LeadPromise { id: "FS-demo.1", reason: "a reading prints the lead of its section" },\n];\n'))
        self.assertEqual(sorted(["FS-x.10", "FS-x.9.1", "FS-x.9"], key=spec_fold_coverage.natural),
                         ["FS-x.9", "FS-x.9.1", "FS-x.10"])
        leaf = self.record.with_name("leaf.md")
        leaf.write_text(RECORD + "\n## lead-promises\n\n- FS-demo.3: a list\n- FS-demo.1 no colon\n", encoding="utf-8")
        code, out = self.run_pass(record=leaf)
        self.assertEqual(code, 2, out)
        self.assertIn("leaf.md:26: under `## lead-promises` a line naming a section is `<parent>: <the promise>`", out)
        leaf.write_text(RECORD + "\n## lead-promises\n\n- FS-demo.3: a list\n", encoding="utf-8")
        code, out = self.run_pass(record=leaf)
        self.assertEqual(code, 2, out)
        self.assertIn("leaf.md:25: lead promise FS-demo.3: a leaf, which the coverage gate holds already", out)
        misfiled = self.record.with_name("misfiled.md")
        misfiled.write_text(RECORD + "\n## fold-notes\n\n- FS-demo.1.2 -> FS-demo.1\n", encoding="utf-8")
        code, out = self.run_pass(record=misfiled)
        self.assertEqual(code, 2, out)
        self.assertIn("misfiled.md:25: an entry under `## fold-notes`, which the pass does not read", out)

    def test_a_fenced_quote_outside_the_spec_is_reported_not_rewritten(self):
        self.track("docs/releases/1.0-freeze.md", QUOTED)
        self.track("crates/demo/src/lib.rs", SOURCE)
        code, out = self.run_pass()
        self.assertEqual(code, 0, out)
        self.assertEqual(self.read("docs/releases/1.0-freeze.md"),
                         QUOTED.replace(f"Then {M}FS-demo.1.1", f"Then {M}FS-demo.1"))
        self.assertIn(f"/// {M}FS-demo.1.1\n", self.read("crates/demo/src/lib.rs"))
        for site in (f"docs/releases/1.0-freeze.md:4: `{M}FS-demo.1.1` is quoted in a fenced block, so the pass left it",
                     "docs/releases/1.0-freeze.md:4: a link in a fenced block names a heading the rewrite removed",
                     f"crates/demo/src/lib.rs:3: `{M}FS-demo.1.1` is quoted in a fenced block"):
            self.assertIn(site, out)

    def test_a_repeat_the_retarget_makes_is_kept_once(self):
        self.track("docs/repeats.md", REPEATS)
        self.track("crates/demo/src/lib.rs", SOURCE)
        self.track("docs/changelog.md",
                   CHANGELOG.replace(f"cite {M}FS-demo.1.1.", f"cite {M}FS-demo.1.1 and {M}FS-demo.1.1.1."))
        self.write("docs/functional-spec/FS-demo.md",
                   FOLDED_PAGE.replace("Brief text. Detail.", f"Brief text. Detail, as {M}FS-demo.1.1 says."))
        code, out = self.run_pass()
        self.assertEqual(code, 0, out)
        self.assertEqual(self.read("docs/repeats.md").split("\n"), [
            f"See {M}FS-demo.1.", f"Both ({M}FS-demo.1) hold.",
            f"Linked [{M}FS-demo.1](functional-spec/FS-demo.md#1-reading).",
            f"Authored {M}FS-demo.1.2, {M}FS-demo.1.2 stays.", f"Ends {M}FS-demo.1,", f"{M}FS-demo.1 opens.", ""])
        self.assertIn("3 repeated citation(s) collapsed", out)
        self.assertIn(f"docs/repeats.md:6: `{M}FS-demo.1` ends line 5 and opens this one", out)
        self.assertIn(f'"{M}FS-demo.1, {M}FS-demo.1"', self.read("crates/demo/src/lib.rs"))
        self.assertIn(f"crates/demo/src/lib.rs:6: `{M}FS-demo.1, {M}FS-demo.1` repeats inside a string", out)
        self.assertIn(f"cite {M}FS-demo.1 and {M}FS-demo.1.", self.read("docs/changelog.md"))
        self.assertIn("docs/functional-spec/FS-demo.md:11: `FS-demo.1` now cites itself", out)
        self.assertIn("crates/demo/src/lib.rs: code lines 1, 6; #[cfg(test)] lines 11", out)

    def test_a_relative_row_is_reported_only_for_its_own_page(self):
        self.track("tests/integration/test_demo_moves.py",
                   MOVES_TEST + 'OTHER = (("FS-other.md", "1.1", "Home"),)\nRULES = (("3.1", "Child"),)\n')
        code, out = self.run_pass()
        self.assertEqual(code, 0, out)
        self.assertIn("test_demo_moves.py:5: names `1.1` of `FS-demo`", out)
        self.assertIn("test_demo_moves.py:7: names `3.1` of `FS-demo`, a removed section; it is now `FS-demo.3`", out)
        self.assertNotIn("test_demo_moves.py:6:", out)

    def test_a_cumulative_run_skips_the_records_already_applied(self):
        code, out = self.run_pass()
        self.assertEqual(code, 0, out)
        git(self.root, "add", "-A")
        git(self.root, "-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-m", "first")
        applied = git(self.root, "rev-parse", "HEAD").strip()
        page = self.read("docs/functional-spec/FS-demo.md")
        self.write("docs/functional-spec/FS-demo.md", page.replace("### 1.2 Kept\n\n", ""))
        second = self.record.with_name("second.md")
        second.write_text("## folds\n\n- FS-demo.1.2 -> FS-demo.1\n", encoding="utf-8")
        code, out = self.run_pass("--check", "--base", applied, "--record", str(second))
        self.assertEqual(code, 0, out)
        self.assertIn("check ok: 1 removed coordinate(s) recorded, 4 applied before", out)
        code, out = self.run_pass("--base", applied, "--record", str(second))
        self.assertEqual(code, 0, out)
        self.assertIn(f"{self.record}: applied by an earlier pass, all 4 entries gone; skipped", out)
        self.assertNotIn("pins:", out)
        self.assertEqual(self.read("docs/guide.md").split("\n")[0], f"See {M}FS-demo.1.")
        self.assertEqual(self.read("tests/e2e/cases/case-b/spec.refs"), "FS-demo.1\n")

    def test_the_fixed_table_moves_as_it_did(self):
        self.write("docs/guide.md", f"{M}FS-check.4.7.4, {M}FS-check.4.7.10 and {ESC}FS-check.4.7\n")
        self.write("tests/e2e/cases/case-b/spec.refs", "FS-001-check.4.7.2.1\n")
        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            code = move_spec_sections.main(["--root", str(self.root)])
        self.assertEqual(code, 0, out.getvalue())
        self.assertEqual(self.read("docs/guide.md"), f"{M}FS-check.3.30.2, {M}FS-check.3.30.10 and {ESC}FS-check.4.7\n")
        self.assertEqual(self.read("tests/e2e/cases/case-b/spec.refs"), "FS-check.3.30.2.1\n")


if __name__ == "__main__":
    unittest.main()
