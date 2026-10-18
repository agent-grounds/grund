"""Citation-site queries on the Python binding: `refs` with `globs` or `from_` and `cover`
with `globs` (§FS-coordinate-globs.bindings.options, §FS-coordinate-globs.bindings.note,
§FS-coordinate-globs.bindings.refusals).

Every test here is red until #810 gives the binding these options, so each carries
`@unittest.expectedFailure`; the implementation removes the markers. Each calls the binding
before the oracle, so today's failure is the binding's own refusal of the new call.
"""

import inspect
import json
import unittest

from adapter import binding, plain, python_call
from corpus import CITATION_SITE_CASES
from support import REPO, canonical, fixture, rust_call, temporary

CASES = REPO / "tests/e2e/cases"
FIELDS = ("project", "path", "line", "column", "id", "section", "marker", "text",
          "enclosing_declaration", "enclosing_section")


def golden(case, stream="stdout"):
    return (CASES / case / f"expected.{stream}").read_text(encoding="utf-8")


def golden_rows(case, stream="stdout"):
    return [json.loads(line) for line in golden(case, stream).splitlines()]


def records(hits):
    return [{field: getattr(hit, field) for field in FIELDS} for hit in hits]


def cli_records(rows):
    return [{field: row.get(field) for field in FIELDS} for row in rows]


class CitationSiteTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.g = binding()

    @unittest.expectedFailure
    def test_the_new_options_are_keyword_only_and_the_id_optional(self):
        refs = inspect.signature(self.g.refs).parameters
        self.assertIsNone(refs["id"].default)
        for name, default in (("globs", ()), ("from_", None)):
            with self.subTest(option=name):
                self.assertEqual(default, refs[name].default)
                self.assertEqual(inspect.Parameter.KEYWORD_ONLY, refs[name].kind)
        cover = inspect.signature(self.g.cover).parameters["globs"]
        self.assertEqual(((), inspect.Parameter.KEYWORD_ONLY), (cover.default, cover.kind))

    @unittest.expectedFailure
    def test_citation_site_reads_match_the_core_data_and_bytes(self):
        for case, operation, args, options in CITATION_SITE_CASES:
            with self.subTest(case=case, operation=operation), temporary() as temp:
                root = fixture(temp, case)
                actual = python_call(self.g, operation, root, args, options)
                expected, wire, _ = rust_call(operation, root, args, options)
                self.assertEqual(expected, actual, "complete field/order/null parity")
                self.assertEqual(wire, canonical(actual), "canonical bytes")

    @unittest.expectedFailure
    def test_hits_are_the_cli_records(self):
        for case, options in (
            ("refs-glob-union-once-json", {"globs": ("FS-login.1{,.**}", "FS-login.1.*")}),
            ("workspace-refs-glob-qualified-json", {"globs": ("target/FS-login{,.**}",)}),
            ("refs-from-json", {"from_": "FS-login.1"}),
            ("workspace-refs-from-keeps-the-alias-json", {"from_": "citer/FS-audit"}),
        ):
            with self.subTest(case=case), temporary() as temp:
                result = self.g.refs(root=fixture(temp, case), **options)
                self.assertEqual(cli_records(golden_rows(case)), records(result.hits))
                self.assertEqual((None, None), (result.kind_title, result.note))
        case = "refs-glob-total-json"
        with temporary() as temp:
            result = self.g.refs(root=fixture(temp, case), globs=("FS-login{,.**}",))
        self.assertEqual(golden_rows(case), [{"sites": result.site_total,
                                              "files": result.file_total}])

    @unittest.expectedFailure
    def test_cover_keeps_every_entry(self):
        case = "cover-glob-json"
        with temporary() as temp:
            result = self.g.cover(root=fixture(temp, case), globs=("FS-ledger.*",))
        self.assertEqual([(row["path"], cli_records(row["citations"]))
                          for row in golden_rows(case)],
                         [(entry.path, records(entry.citations)) for entry in result.entries])
        self.assertIsNone(result.note)
        case = "cover-glob-keeps-every-file"
        with temporary() as temp:
            result = self.g.cover(root=fixture(temp, case), text=True,
                                  globs=("FS-audit{,.**}",))
        rendered = "".join(
            f"{entry.path}:\n" + ("".join(f"  {c.line}:{c.column} {c.text}\n"
                                          for c in entry.citations) or "  (no citations)\n")
            for entry in result.entries)
        self.assertEqual(golden(case), rendered)

    @unittest.expectedFailure
    def test_notes_are_the_cli_note_text(self):
        note = golden("refs-glob-note", "stderr").removeprefix("note: ").removesuffix("\n")
        self.assertEqual(golden("cover-glob-note", "stderr"), f"note: {note}\n")
        with temporary() as temp:
            root = fixture(temp, "refs-glob-note")
            self.assertEqual(note, self.g.refs(root=root, globs=("POLICY-*",)).note)
            self.assertEqual(note, self.g.cover(root=root, globs=("POLICY-*",)).note)
            self.assertIsNone(self.g.cover(root=root).note)
            self.assertIsNone(self.g.refs(root=root, from_="FS-login").note)

    @unittest.expectedFailure
    def test_a_refused_glob_or_source_is_a_query_failure(self):
        case = "refs-glob-refused-before-discovery"
        message = golden(case, "stderr").removeprefix("error: ").removesuffix("\n")
        with temporary() as temp:
            root = fixture(temp, case)
            for call in (lambda: self.g.refs(root=root, globs=("FS-{a,b",)),
                         lambda: self.g.cover(root=root, globs=("FS-{a,b",))):
                with self.assertRaises(self.g.GrundError) as raised:
                    call()
                failure = raised.exception.failure
                self.assertEqual(("query", "invalid-coordinate-glob", message, 4),
                                 (failure.kind, failure.code, failure.message, failure.column))
        for case, source in (("refs-from-not-found-json", "FS-nope"),
                             ("refs-from-ambiguous-json", "FS-convoy")):
            refusal = golden_rows(case, "stderr")[0]
            with self.subTest(case=case), temporary() as temp:
                with self.assertRaises(self.g.GrundError) as raised:
                    self.g.refs(root=fixture(temp, case), from_=source)
                failure = plain(raised.exception.failure)
                self.assertEqual(("query", refusal["code"], refusal["message"]),
                                 (failure["kind"], failure["code"], failure["message"]))

    @unittest.expectedFailure
    def test_selectors_conflict_and_one_is_required(self):
        with temporary() as temp:
            root = fixture(temp, "refs-from-json")
            for options in (
                {"id": "FS-login", "globs": ("FS-*",)},
                {"id": "FS-login", "from_": "FS-login"},
                {"globs": ("FS-*",), "from_": "FS-login"},
                {"globs": ("FS-*",), "section": "1"},
                {"globs": ("FS-*",), "descendants": True},
                {"from_": "FS-login", "section": "1"},
                {"from_": "FS-login", "descendants": True},
            ):
                with self.subTest(options=options), self.assertRaises(ValueError):
                    self.g.refs(root=root, **options)
            with self.assertRaises(ValueError):
                self.g.cover(root=root / "docs/fs/FS-login.md", lines=("1",), globs=("FS-*",))
            for options in ({}, {"globs": ()}):
                with self.subTest(options=options), self.assertRaises(TypeError):
                    self.g.refs(root=root, **options)
            self.assertEqual(records(self.g.refs("FS-login", root=root).hits),
                             records(self.g.refs("FS-login", root=root, globs=()).hits))


if __name__ == "__main__":
    unittest.main()
