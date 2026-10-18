"""Coordinate globs on the Python binding (§FS-coordinate-globs.bindings.options,
§FS-coordinate-globs.bindings.note, §FS-coordinate-globs.bindings.refusals).

Every test here is red until #503 gives the binding `globs` and `note`, so each carries
`@unittest.expectedFailure`; the implementation removes the markers.
"""

import inspect
import json
import unittest

from adapter import binding, python_call
from corpus import GLOB_READ_CASES
from support import REPO, canonical, fixture, rust_call, temporary

CASES = REPO / "tests/e2e/cases"


def golden(case, stream="stdout"):
    return (CASES / case / f"expected.{stream}").read_text(encoding="utf-8")


class CoordinateGlobTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.g = binding()

    @unittest.expectedFailure
    def test_globs_is_keyword_only_and_empty_by_default(self):
        for name in ("list_ids", "list_sizes", "show_batch"):
            with self.subTest(operation=name):
                parameter = inspect.signature(getattr(self.g, name)).parameters["globs"]
                self.assertEqual((), parameter.default)
                self.assertEqual(inspect.Parameter.KEYWORD_ONLY, parameter.kind)

    @unittest.expectedFailure
    def test_glob_reads_match_the_core_data_and_bytes(self):
        for case, operation, args, options in GLOB_READ_CASES:
            with self.subTest(case=case, operation=operation), temporary() as temp:
                root = fixture(temp, case)
                expected, wire, _ = rust_call(operation, root, args, options)
                actual = python_call(self.g, operation, root, args, options)
                self.assertEqual(expected, actual, "complete field/order/null parity")
                self.assertEqual(wire, canonical(actual), "canonical bytes")

    @unittest.expectedFailure
    def test_entries_and_records_are_the_cli_rows(self):
        fields = ("id", "section", "kind", "path", "line", "title")
        case = "list-glob-point-and-subtree-json"
        rows = [json.loads(line) for line in golden(case).splitlines()]
        with temporary() as temp:
            result = self.g.list_ids(root=fixture(temp, case),
                                     globs=("FS-config.requirements{,.**}",))
        self.assertEqual([{f: row[f] for f in fields} for row in rows],
                         [{f: getattr(entry, f) for f in fields} for entry in result.entries])
        self.assertIsNone(result.note)
        case = "show-batch-glob-json"
        queries = [json.loads(line)["query"] for line in golden(case).splitlines()]
        with temporary() as temp:
            result = self.g.show_batch(root=fixture(temp, case),
                                       globs=("FS-config.requirements{,.**}",))
        self.assertEqual([(q["id"], q["section"]) for q in queries],
                         [(r.query.id, r.query.section) for r in result.records])

    @unittest.expectedFailure
    def test_note_is_the_cli_note_text(self):
        case = "list-glob-note-suggests-the-kind"
        note = golden(case, "stderr").removeprefix("note: ").removesuffix("\n")
        with temporary() as temp:
            root = fixture(temp, case)
            for result in (self.g.list_ids(root=root, globs=("fs-*",)),
                           self.g.list_sizes(root=root, globs=("fs-*",)),
                           self.g.show_batch(root=root, globs=("fs-*",))):
                self.assertEqual(note, result.note)
            self.assertIsNone(self.g.list_ids(root=root).note)

    @unittest.expectedFailure
    def test_a_refused_glob_is_a_query_failure_and_queries_conflict(self):
        case = "list-glob-refused-star-run-ends-the-pattern"
        message = golden(case, "stderr").removeprefix("error: ").removesuffix("\n")
        with temporary() as temp:
            root = fixture(temp, case)
            with self.assertRaises(self.g.GrundError) as raised:
                self.g.list_ids(root=root, globs=("FS-**",))
            failure = raised.exception.failure
            self.assertEqual(("query", "invalid-coordinate-glob", message, 4),
                             (failure.kind, failure.code, failure.message, failure.column))
            with self.assertRaises(ValueError):
                self.g.show_batch(("FS-config",), root=root, globs=("FS-*",))


if __name__ == "__main__":
    unittest.main()
