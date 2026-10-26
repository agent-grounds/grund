"""§AR-config.9.1: the check code catalog the installed `grund` prints is the
table of §FS-errors.5.5 rendered. `check-codes.json` and the handles in
`model/codes.rs` are what `scripts/render_check_codes.py` writes from the page
today, and the renderer refuses a cell the grammar of §FS-errors.5.5.1 cannot
read rather than guess at it. A code's remedy is its row's *Fix* cell, worded
there once and nowhere else in the checker (§FS-errors.5.5.1, §AR-config.9.1)."""

import json
import re
import sys
import unittest
from pathlib import Path
from unittest import mock

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT / "scripts"))
import render_check_codes  # noqa: E402


class CheckCodeCatalogTests(unittest.TestCase):
    def test_the_embedded_catalog_and_handles_are_the_page_rendered(self):
        for path, text in render_check_codes.render().items():
            self.assertEqual(path.read_text(encoding="utf-8"), text,
                             f"{path.relative_to(REPO_ROOT)} is stale: run "
                             "python scripts/render_check_codes.py")

    def test_a_planned_row_is_neither_rendered_nor_a_handle(self):
        rendered = {entry["code"] for entry in render_check_codes.catalog()}
        planned = {row["Code"].strip("`") for row in render_check_codes.rows()
                   if row["Enabled by"].startswith("planned — ")}
        self.assertTrue(planned)
        self.assertFalse(planned & rendered)

    def test_a_cell_naming_an_id_or_holding_a_link_is_refused(self):
        for cell in ("see §FS-check.3.1", "see [the spec](FS-check.md)"):
            with self.assertRaisesRegex(render_check_codes.CatalogError, "`dangling`"):
                render_check_codes.printed(cell, "dangling", "Meaning")

    def test_a_side_naming_the_other_version_absent_is_refused(self):
        with self.assertRaisesRegex(render_check_codes.CatalogError, "version 2 Severity"):
            render_check_codes.severity("not in version 1", 2, "missing-field")

    def test_a_split_cell_reads_each_side_and_its_clauses(self):
        side = render_check_codes.enabled(
            "`--suggestions` is passed and `[rules.citations.<kind>] should` is set; a remark",
            2, "suggested-citation")
        self.assertEqual(side, {
            "kind": "clauses",
            "clauses": [{"text": "--suggestions is passed", "by": "flag"},
                        {"text": "[rules.citations.<kind>] should is set", "by": "setting"}],
            "remark": "a remark",
        })
        self.assertEqual(render_check_codes.sides("v2: a · v1: b"), ("a", "b"))
        self.assertEqual(render_check_codes.sides("—"), ("—", "—"))


# Runs two cells hold that the grammar cannot yet share. The exempt run is one remedy
# clause both cells word: `add "{entry}" to [workspace] members in {enclosing}`, in
# misplaced-declaration's `unlisted` case and in unlisted-workspace-block's Fix. Each
# follows it with a different other way out, and a reference names a whole template or
# case, so neither cell can take the clause from the other until that Fix is split.
SHARED_SPELLINGS = {'" to [workspace] members in'}

# A run of words: a maximal stretch of a Fix template between its slots and references,
# its case label dropped, of at least this many words once trimmed of edge punctuation.
RUN_WORDS = 4
BRACED = re.compile(r"\{[^{}]*\}")
LABEL = re.compile(r"^[a-z][a-z0-9-]*: ")


def fixed_runs(cell):
    """§FS-errors.5.5.1: the runs of words a Fix cell words once, on each version side."""
    runs = set()
    for side in render_check_codes.sides(cell):
        if side.startswith("none — ") or render_check_codes.ABSENT.match(side):
            continue
        cases = side.split(" ‖ ")
        for case in cases:
            if len(cases) > 1:
                case = LABEL.sub("", case, count=1)
            for run in BRACED.split(case):
                words = run.strip(" ,;:.").split()
                if len(words) >= RUN_WORDS:
                    runs.add(" ".join(words))
    return runs


def string_literals(source):
    """The text of each string literal of Rust `source`, escapes and line continuations
    read and whitespace collapsed; comments and char literals are skipped."""
    literals, index, end = [], 0, len(source)
    while index < end:
        char = source[index]
        if source.startswith("//", index):
            index = source.find("\n", index)
            index = end if index < 0 else index
        elif source.startswith("/*", index):
            index = source.find("*/", index) + 2 if "*/" in source[index:] else end
        elif (opening := re.match(r"b?r(#*)\"", source[index:])) and not (
                index and (source[index - 1].isalnum() or source[index - 1] == "_")):
            close = '"' + opening.group(1)
            start = index + opening.end()
            stop = source.find(close, start)
            literals.append(source[start:stop])
            index = stop + len(close)
        elif char == '"':
            stop = index + 1
            while source[stop] != '"':
                stop += 2 if source[stop] == "\\" else 1
            raw = source[index + 1:stop]
            raw = re.sub(r"\\\n\s*", "", raw)
            literals.append(raw.replace("\\n", " ").replace('\\"', '"'))
            index = stop + 1
        elif char == "'" and re.match(r"'(?:[^'\\]|\\.[^']*)'", source[index:]):
            index += re.match(r"'(?:[^'\\]|\\.[^']*)'", source[index:]).end()
        else:
            index += 1
    return [" ".join(literal.split()) for literal in literals]


# Sources whose literals are another command's own wording rather than a finding's
# remedy: the managed block's teaching, `config explain`'s answer and `help config`.
OWN_WORDING = {
    "crates/grund-core/src/templates/agents_block.rs",
    "crates/grund-cli/src/cli_config_explain.rs",
    "crates/grund-cli/src/cli_help_config.rs",
}


def checker_sources():
    """Every non-test Rust source under `crates/` that words a finding: a `src/` file,
    cut at its first `#[cfg(test)]`, outside any `tests` directory and any `tests.rs`,
    `tests_*.rs` or `*_tests.rs` unit module, whose literals are the messages a test
    expects rather than a second wording. The files of `OWN_WORDING` are skipped too:
    what they print is the wording of the command they belong to, not a remedy."""
    for path in sorted((REPO_ROOT / "crates").glob("*/src/**/*.rs")):
        if ("tests" in path.parts or path.stem == "tests" or path.stem.startswith("tests_")
                or path.stem.endswith("_tests")):
            continue
        if path.relative_to(REPO_ROOT).as_posix() in OWN_WORDING:
            continue
        text = path.read_text(encoding="utf-8")
        yield path, text.split("#[cfg(test)]", 1)[0]


def with_fix(overrides):
    """The page's rows with every Fix a plain template and every Note `—`, and the
    cells `overrides` names, as `{code: (fix, note)}`, in place."""
    rows = []
    for row in render_check_codes.rows():
        code = row["Code"].strip("`")
        fix, note = overrides.get(code, ("run grund check again after the edit", "—"))
        rows.append({**row, "Fix": fix, "Note": note})
    return mock.patch.object(render_check_codes, "rows", lambda: rows)


def entry(code):
    return next(read for read in render_check_codes.catalog() if read["code"] == code)


class CheckCodeRemedyTests(unittest.TestCase):
    """§FS-errors.5.5.1: the Fix and Note cells and what the renderer makes of them."""

    maxDiff = None

    def test_every_row_names_its_fix(self):
        blank = [row["Code"] for row in render_check_codes.rows() if not row.get("Fix")]
        self.assertEqual(blank, [], "rows whose Fix cell is blank")

    def test_the_rendered_remedy_fillers_are_written_beside_the_handles(self):
        remedies = REPO_ROOT / "crates" / "grund-core" / "src" / "model" / "remedies.rs"
        self.assertIn(remedies, render_check_codes.render())

    def test_a_blank_fix_is_refused(self):
        with with_fix({"dangling": ("", "—")}):
            with self.assertRaisesRegex(render_check_codes.CatalogError, "`dangling`"):
                render_check_codes.catalog()

    def test_an_unbalanced_slot_is_refused(self):
        for fix in ("cite {new address instead", "cite new address} instead"):
            with self.subTest(fix=fix), with_fix({"dangling": (fix, "—")}):
                with self.assertRaisesRegex(render_check_codes.CatalogError, "`dangling`"):
                    render_check_codes.catalog()

    def test_an_unknown_or_unusable_reference_is_refused(self):
        for fix, other in (("then {fix of `no-such-code`}", None),
                           ("then {fix of `duplicate`}", "none — nothing clears it"),
                           ("then {fix of `duplicate`}", "a: run one ‖ b: run two")):
            overrides = {"dangling": (fix, "—")}
            if other:
                overrides["duplicate"] = (other, "—")
            with self.subTest(fix=fix, other=other), with_fix(overrides):
                with self.assertRaisesRegex(render_check_codes.CatalogError, "`dangling`"):
                    render_check_codes.catalog()

    def test_a_reference_to_a_case_it_does_not_name_or_a_whole_none_is_refused(self):
        for fix, other in (("then {fix of `duplicate`, case `c`}", "a: run one ‖ b: run two"),
                           ("then {fix of `duplicate`, case `a`}", "run one"),
                           ("{fix of `duplicate`}", "none — nothing clears it")):
            with self.subTest(fix=fix, other=other), \
                    with_fix({"dangling": (fix, "—"), "duplicate": (other, "—")}):
                with self.assertRaisesRegex(render_check_codes.CatalogError, "`dangling`"):
                    render_check_codes.catalog()

    def test_a_case_reference_takes_that_case_and_a_whole_one_takes_every_case(self):
        with with_fix({"duplicate": ("a: run `one` {x} ‖ b: run two", "—"),
                       "dangling": ("then {fix of `duplicate`, case `b`}", "—"),
                       "unused": ("{fix of `duplicate`}", "—")}):
            one, whole = entry("dangling"), entry("unused")
        for version in "21":
            self.assertEqual(one[version]["fix"], {"kind": "template", "text": "then run two"})
            self.assertEqual(whole[version]["fix"], {"kind": "cases", "cases": [
                {"label": "a", "text": "run `one` {x}"}, {"label": "b", "text": "run two"}]})

    def test_an_out_of_scope_code_and_discouraged_citation_name_their_fix_by_reference(self):
        """The approved proposal: an out-of-scope code reuses its in-scope code's fix,
        and a discouraged citation forbidden-citation's re-point, by reference."""
        cells = {row["Code"].strip("`"): row["Fix"] for row in render_check_codes.rows()}
        named = {code: f"`{code.removeprefix('out-of-scope-')}`" for code in cells
                 if code.startswith("out-of-scope-")}
        named["discouraged-citation"] = "`forbidden-citation`"
        for code, other in named.items():
            with self.subTest(code=code):
                self.assertEqual(fixed_runs(cells[code]), set())
                self.assertIn("{fix of " + other, cells[code])

    def test_selected_check_off_migrates_in_config_version_1s_words(self):
        """Author addition 2: each migrate the advice gives is config-version-1's
        template, its slot naming what is migrated."""
        cells = {row["Code"].strip("`"): row["Fix"] for row in render_check_codes.rows()}
        cases = dict(case.split(": ", 1) for case in cells["selected-check-off"].split(" ‖ "))
        self.assertEqual(cases["migrate"], "{fix of `config-version-1`}")
        self.assertNotIn("migrate", " ".join(text for label, text in cases.items()
                                             if label != "migrate"))
        self.assertEqual(entry("config-version-1")["1"]["fix"]["text"],
                         "migrate {files} with grund config migrate --to 2")

    def test_an_id_or_a_link_in_a_fix_or_a_note_is_refused(self):
        for cells in (("see §FS-check.3.1", "—"), ("see [the spec](FS-check.md)", "—"),
                      ("run grund check again", "see §FS-check.3.1")):
            with self.subTest(cells=cells), with_fix({"dangling": cells}):
                with self.assertRaisesRegex(render_check_codes.CatalogError, "`dangling`"):
                    render_check_codes.catalog()

    def test_a_reference_is_expanded_and_backticks_are_kept(self):
        with with_fix({"dangling": ("to rebuild it, {fix of `duplicate`}", "—"),
                       "duplicate": ("run `grund init` in its root", "—")}):
            read = entry("dangling")
        for version in "21":
            self.assertIn("fix", read[version], f"version {version} carries no fix")
            self.assertIn("to rebuild it, run `grund init` in its root",
                          json.dumps(read[version]["fix"], ensure_ascii=False))

    def test_a_split_fix_and_note_read_each_side(self):
        with with_fix({"dangling": ("v2: run the first edit · v1: run the second edit",
                                    "v2: first note · v1: second note")}):
            read = entry("dangling")
        for version, own, other in (("2", "first", "second"), ("1", "second", "first")):
            self.assertIn("fix", read[version], f"version {version} carries no fix")
            said = json.dumps([read[version]["fix"], read[version].get("note")])
            self.assertIn(f"run the {own} edit", said)
            self.assertIn(f"{own} note", said)
            self.assertNotIn(other, said)

    def test_cases_and_none_are_read(self):
        with with_fix({"dangling": ("outdated: run the first edit ‖ missing: run the second edit", "—"),
                       "duplicate": ("none — nothing the user does clears it", "—")}):
            cases, none = entry("dangling"), entry("duplicate")
        self.assertIn("fix", cases["2"], "version 2 carries no fix")
        said = json.dumps(cases["2"]["fix"])
        for word in ("outdated", "run the first edit", "missing", "run the second edit"):
            self.assertIn(word, said)
        self.assertIn("nothing the user does clears it", json.dumps(none["2"]["fix"]))

    def test_no_fix_wording_is_written_twice(self):
        """§AR-config.9.1: no run of words a Fix cell holds is also a string literal's
        in a non-test source under `crates/`, so a remedy is reworded in one cell."""
        runs = {run: row["Code"] for row in render_check_codes.rows()
                for run in fixed_runs(row.get("Fix", ""))}
        self.assertTrue(runs, "no Fix cell holds a run of words")
        copies = [f"{path.relative_to(REPO_ROOT)}: {run!r}, from {code}'s Fix"
                  for path, source in checker_sources()
                  for literal in string_literals(source)
                  for run, code in runs.items() if run in literal]
        self.assertEqual(copies, [], "a remedy is worded outside its Fix cell")


    def test_no_fix_wording_is_written_in_two_cells(self):
        """§AR-system.principle: a run of words two rows' Fix cells would both hold is
        written once and named by `{fix of ...}` from the other."""
        cells = {}
        for row in render_check_codes.rows():
            for run in fixed_runs(row.get("Fix", "")):
                cells.setdefault(run, []).append(row["Code"])
        twice = {run: codes for run, codes in cells.items()
                 if len(set(codes)) > 1 and run not in SHARED_SPELLINGS}
        self.assertEqual(twice, {}, "a remedy is worded in more than one Fix cell")


class FixedRunTests(unittest.TestCase):
    """The controls the no-second-copy test leans on."""

    def test_runs_split_at_slots_references_labels_and_versions(self):
        self.assertEqual(
            fixed_runs("v2: a: move the text back here ‖ b: cite {new address} instead "
                       "of the old one · v1: none — it is not in version 1"),
            {"move the text back here", "instead of the old one"})
        # Three words: too short a run to be a remedy's own wording.
        self.assertEqual(fixed_runs("to rebuild it, {fix of `unrecorded-section-citation`}"), set())

    def test_literals_join_continuations_and_skip_comments_and_chars(self):
        source = ('// "not a literal"\nlet c = \'"\';\n'
                  'let s = "move it \\\n        back, cite {to} instead";\n'
                  'let r = r#"raw "text" here"#;\n')
        self.assertEqual(string_literals(source),
                         ["move it back, cite {to} instead", 'raw "text" here'])


if __name__ == "__main__":
    unittest.main()
