"""§AR-config.9.1: the check code catalog the installed `grund` prints is the
table of §FS-errors.5.5 rendered. `check-codes.json` and the handles in
`model/codes.rs` are what `scripts/render_check_codes.py` writes from the page
today, and the renderer refuses a cell the grammar of §FS-errors.5.5.1 cannot
read rather than guess at it."""

import sys
import unittest
from pathlib import Path

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


if __name__ == "__main__":
    unittest.main()
