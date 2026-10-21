"""§REQ-one-answer.proof, the rulings half: every row of every ledger behind
§REQ-one-answer.differences carries a ruling, in the sense that point's lead gives
it. A row carries one when it names a step listed live in `one_answer_steps.toml`
or a `DF-`/`DA-` record that resolves; the copies those ledgers keep are the ones
§AR-system.principle.inventory lists, each naming the step that removes it or the
decision that keeps it.

The ledgers read are the difference rows themselves, the migration rows of
`citation_ledger.toml`, the rows of `primitive_allowlist.toml`, `DEFERRED` and
`WIDENING` of `command_agreement.rs`, and `DEFERRED` of
`tests_grammar_properties.rs`. Each listed step must name a record that resolves
and be named by at least one row, so the list cannot keep a step nobody waits on.
Nothing here asks the forge: a step is live because it is listed.

The tests that read the real ledgers landed one commit before the rows were given
their rulings, marked as expected failures so the pre-commit hook, which runs this
suite, could take them; the commit that ruled the rows took the marks off.
"""

import functools
import json
import re
import subprocess
import tomllib
import unittest
from pathlib import Path

from test_citation_ledger import _grund


REPO_ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent
STEPS = HERE / "one_answer_steps.toml"
REQUIREMENT = REPO_ROOT / "docs" / "requirements" / "REQ-one-answer.md"
CITATION_LEDGER = HERE / "citation_ledger.toml"
ALLOWLIST = HERE / "primitive_allowlist.toml"
AGREEMENT = REPO_ROOT / "crates" / "grund-cli" / "tests" / "command_agreement.rs"
PROPERTIES = REPO_ROOT / "crates" / "grund-core" / "src" / "rules" / "tests_grammar_properties.rs"
# The ledger tests that named their own step set before the list did.
STEP_READERS = (HERE / "test_citation_ledger.py", HERE / "test_primitive_guards.py",
                AGREEMENT, PROPERTIES)

RECORD = re.compile(r"^D[FA]-[a-z0-9-]+$")
MARKER = chr(0xA7)
CITED_RECORD = re.compile(MARKER + r"(D[FA]-[a-z0-9-]+)")
NAMED_STEP = re.compile(r"(?<![\w#])(#\d+|S\d+)(?!\w)")
DIFFERENCE_ROW = re.compile(r"^- \*\*([^*]+?)\.\*\*(.*?)(?=^- \*\*|^#|\Z)", re.M | re.S)
RUST_STRING = re.compile(r'"((?:[^"\\]|\\.)*)"', re.S)


def difference_rows(text):
    """`(row, its text)` for every row of the requirement's differences chapter."""
    chapter = text.split("\n## differences:", 1)[1].split("\n## ", 1)[0]
    return [(row, body) for row, body in DIFFERENCE_ROW.findall(chapter)]


def rust_tuples(text, const):
    """The string fields of each tuple in `const NAME: … = &[ … ];`, in order."""
    match = re.search(rf"const {const}\b[^=]*=\s*&\[(.*?)\];", text, re.S)
    if match is None:
        raise AssertionError(f"no const {const}")
    body = match.group(1)
    tuples, depth, start = [], 0, None
    for index, char in enumerate(body):
        if char == "(" and depth == 0:
            depth, start = 1, index
        elif char == "(":
            depth += 1
        elif char == ")":
            depth -= 1
            if depth == 0:
                tuples.append(RUST_STRING.findall(body[start:index]))
    return tuples


def ledger_rows():
    """`(ledger, row, names)`: every code-ledger row and the step or ruling it names."""
    rows = []
    for family in tomllib.loads(CITATION_LEDGER.read_text(encoding="utf-8"))["family"]:
        rows += [("citation_ledger.toml", f"{family['name']}: {row['file']}",
                  [row[key] for key in ("step", "ruling") if key in row])
                 for row in family.get("migration", [])]
    rows += [("primitive_allowlist.toml", row["file"],
              [row[key] for key in ("step", "ruling") if key in row])
             for row in tomllib.loads(ALLOWLIST.read_text(encoding="utf-8")).get("allow", [])]
    agreement = AGREEMENT.read_text(encoding="utf-8")
    rows += [("command_agreement.rs DEFERRED", fields[0], fields[1:2])
             for fields in rust_tuples(agreement, "DEFERRED")]
    rows += [("command_agreement.rs WIDENING", fields[0][:60], fields[1:2])
             for fields in rust_tuples(agreement, "WIDENING")]
    rows += [("tests_grammar_properties.rs DEFERRED", fields[0], fields[1:2])
             for fields in rust_tuples(PROPERTIES.read_text(encoding="utf-8"), "DEFERRED")]
    return rows


def problems(differences, rows, steps, resolves):
    """What is wrong with the ledgers: `differences` is `(row, text)`, `rows`
    `(ledger, row, names)`, `steps` the listed `{id, ruling}` entries, and
    `resolves(record)` whether a decision record resolves."""
    wrong = []
    listed = {step["id"]: step for step in steps}
    named = set()
    for row, text in differences:
        named |= set(NAMED_STEP.findall(text)) & set(listed)
        cited = CITED_RECORD.findall(text)
        if not cited:
            wrong.append(f"REQ-one-answer.differences {row}: cites no decision record")
        for record in cited:
            if not resolves(record):
                wrong.append(f"REQ-one-answer.differences {row}: {record} does not resolve")
    for ledger, row, names in rows:
        named |= set(names) & set(listed)
        if not any(name in listed or (RECORD.match(name) and resolves(name)) for name in names):
            shown = ", ".join(names) or "nothing"
            wrong.append(f"{ledger} {row}: names {shown}, neither a step listed in "
                         "one_answer_steps.toml nor a decision record that resolves")
    for id_, step in sorted(listed.items()):
        ruling = step.get("ruling", "")
        if not (RECORD.match(ruling) and resolves(ruling)):
            wrong.append(f"one_answer_steps.toml {id_}: ruling {ruling!r} does not resolve")
        if id_ not in named:
            wrong.append(f"one_answer_steps.toml {id_}: no row names it: take it out")
    return wrong


def listed_steps():
    return tomllib.loads(STEPS.read_text(encoding="utf-8"))["step"]


def resolver(records):
    """`resolves(record)` for every record, from one `grund show --batch`."""
    records = sorted(set(records))
    queries = "".join(json.dumps({"id": record, "section": None}) + "\n" for record in records)
    shown = subprocess.run([_grund(), "show", "--batch", "--format", "json", "--brief"],
                           cwd=REPO_ROOT, input=queries, capture_output=True, text=True).stdout
    answers = [json.loads(line)["ok"] for line in shown.splitlines() if line.strip()]
    if len(answers) != len(records):
        raise AssertionError(f"show --batch answered {len(answers)} of {len(records)} records")
    found = dict(zip(records, answers))
    return found.__getitem__


@functools.lru_cache(maxsize=None)
def real_problems():
    differences = difference_rows(REQUIREMENT.read_text(encoding="utf-8"))
    rows, steps = ledger_rows(), listed_steps()
    records = [r for _, text in differences for r in CITED_RECORD.findall(text)]
    records += [n for _, _, names in rows for n in names if RECORD.match(n)]
    records += [s["ruling"] for s in steps if RECORD.match(s.get("ruling", ""))]
    return tuple(problems(differences, rows, steps, resolver(records)))


class OneAnswerLedgersTest(unittest.TestCase):
    def test_every_difference_row_cites_a_decision_record_that_resolves(self):
        wrong = [w for w in real_problems() if w.startswith("REQ-one-answer.differences")]
        self.assertEqual([], wrong, "\n" + "\n".join(wrong))

    def test_every_ledger_row_names_a_listed_step_or_a_resolving_ruling(self):
        wrong = [w for w in real_problems()
                 if not w.startswith(("REQ-one-answer.differences", "one_answer_steps.toml"))]
        self.assertEqual([], wrong, "\n" + "\n".join(wrong))

    def test_every_listed_step_names_a_ruling_and_is_named_by_a_row(self):
        wrong = [w for w in real_problems() if w.startswith("one_answer_steps.toml")]
        self.assertEqual([], wrong, "\n" + "\n".join(wrong))

    def test_every_ledger_test_reads_its_steps_from_the_list(self):
        wrong = [str(path.relative_to(REPO_ROOT)) for path in STEP_READERS
                 if STEPS.name not in path.read_text(encoding="utf-8")]
        self.assertEqual([], wrong, "these keep a step set of their own: " + ", ".join(wrong))


class OneAnswerLedgersHarnessTests(unittest.TestCase):
    """The rule on made-up ledgers: a step or a ruling, and nothing else."""

    STEPS = [{"id": "#1", "ruling": "DF-a"}]
    DIFFERENCE = ("P1", f" Two answers. R. Ruled by [{MARKER}DF-a](x.md#df-a), "
                        "which moves it (#1).")

    def check(self, differences=None, rows=None, steps=None, records=("DF-a", "DF-b")):
        return problems([self.DIFFERENCE] if differences is None else differences,
                        [] if rows is None else rows,
                        self.STEPS if steps is None else steps,
                        lambda record: record in records)

    def test_a_listed_step_and_a_cited_record_pass(self):
        self.assertEqual([], self.check(rows=[("t1", "a.rs", ["#1"])]))

    def test_a_row_naming_an_unlisted_step_fails(self):
        wrong = self.check(rows=[("t1", "a.rs", ["#595"])])
        self.assertIn("t1 a.rs: names #595, neither a step listed", " ".join(wrong))

    def test_a_row_naming_nothing_fails(self):
        self.assertIn("names nothing", " ".join(self.check(rows=[("t1", "a.rs", [])])))

    def test_a_row_with_a_resolving_ruling_passes(self):
        self.assertEqual([], self.check(rows=[("t3", "b.rs", ["DF-b"])]))

    def test_a_row_with_a_ruling_that_does_not_resolve_fails(self):
        self.assertIn("neither", " ".join(self.check(rows=[("t3", "b.rs", ["DF-gone"])])))

    def test_a_difference_row_citing_no_record_fails(self):
        wrong = self.check(differences=[self.DIFFERENCE, ("G6", " Two answers. R. (#1)")])
        self.assertEqual(["REQ-one-answer.differences G6: cites no decision record"], wrong)

    def test_a_difference_row_citing_a_record_that_does_not_resolve_fails(self):
        wrong = self.check(differences=[("P2", f" R. Ruled by {MARKER}DF-gone. (#1)")])
        self.assertIn("P2: DF-gone does not resolve", " ".join(wrong))

    def test_a_step_no_row_names_fails(self):
        wrong = self.check(differences=[("P1", f" R. Ruled by {MARKER}DF-a.")])
        self.assertEqual(["one_answer_steps.toml #1: no row names it: take it out"], wrong)

    def test_a_step_whose_ruling_does_not_resolve_fails(self):
        wrong = self.check(steps=[{"id": "#1", "ruling": "DF-gone"}])
        self.assertIn("#1: ruling 'DF-gone' does not resolve", " ".join(wrong))

    def test_the_rust_ledgers_are_read_field_by_field(self):
        text = ('const W: &[(&str, &str)] = &[(\n'
                '    "every tree, \\\n     and `cover`",\n    "S4",\n)];')
        self.assertEqual([["every tree, \\\n     and `cover`", "S4"]], rust_tuples(text, "W"))

    def test_the_difference_rows_are_read_from_their_chapter(self):
        text = ("# R\n\n## differences: D\n\n### differences.x: X\n\n- **P1.** One\n  more. R.\n"
                "- **U2.** Two. C.\n\n## proof: P\n\n- **Z9.** not a row\n")
        self.assertEqual(["P1", "U2"], [row for row, _ in difference_rows(text)])


if __name__ == "__main__":
    unittest.main()
