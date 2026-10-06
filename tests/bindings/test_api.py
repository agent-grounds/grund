"""Public Python signatures/data/errors (§FS-distribution.3.3.1, §FS-distribution.3.3.2,
§FS-distribution.3.3.3, §FS-distribution.3.3.5, §FS-distribution.3.3.6)."""

import dataclasses
import inspect
from pathlib import Path
import unittest
import typing

from support import binding, fixture, plain, temporary, tree_bytes

# (positional defaults: Ellipsis means required, keyword defaults). root is added
# to tree operations. This is the approved inventory, not native signature guesses.
SIGNATURES = {
    "check": ({"root": None}, dict(require_grounding=False, suggestions=False, full=False,
                                  rule=None, only=(), ignore=(), only_rule=False)),
    "scan": ({"root": ...}, {}),
    "show": ({"id": ...}, dict(section=None, mode="lead", format="text")),
    "show_batch": ({"queries": None}, dict(mode="lead")),
    "refs": ({"id": ...}, dict(section=None, descendants=False)),
    "list_ids": ({}, dict(kinds=(), projects=(), unused=False, selector=None)),
    "list_sizes": ({}, dict(kinds=(), projects=(), unused=False, selector=None,
                            units=("lines", "words", "bytes"), top=None)),
    "cover": ({}, dict(text=False, lines=())),
    "fmt": ({}, dict(write=False, marker=False, cross_refs=False)),
    "propose_id": ({"kind": ..., "title": ...}, dict(width=3)),
    "init": ({"target": None}, dict(name=None, description=None, docs=False, force=False,
                                   write=False, check=False, no_vcs=False, agents=None)),
    "effective_config": ({}, {}), "validate_config": ({}, {}),
    "fetch": ({"id": ...}, dict(write=False)),
    "integrations": ({"client": None}, dict(write=False, conversation=None,
                                             conversation_target=None, agent=None)),
    "complete_ids": ({"prefix": ""}, dict(sections=False)),
    "reference_style": ({}, {}), "agent_setup_instructions": ({}, {}),
}


class ApiTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.g = binding()

    def test_exact_callable_defaults_keyword_policy_and_types(self):
        for name, (positional, keywords) in SIGNATURES.items():
            with self.subTest(operation=name):
                function = getattr(self.g, name)
                self.assertFalse(inspect.iscoroutinefunction(function))
                sig = inspect.signature(function)
                expected = dict(positional, **keywords)
                if name not in ("check", "scan", "init", "integrations", "agent_setup_instructions"):
                    expected["root"] = None
                self.assertEqual(set(expected), set(sig.parameters))
                for key, value in expected.items():
                    parameter = sig.parameters[key]
                    self.assertEqual(inspect.Parameter.empty if value is ... else value,
                                     parameter.default)
                    self.assertEqual(inspect.Parameter.POSITIONAL_OR_KEYWORD if key in positional
                                     else inspect.Parameter.KEYWORD_ONLY, parameter.kind)
                hints = typing.get_type_hints(function)
                self.assertTrue(set(sig.parameters) <= set(hints), "all operands have type hints")
                self.assertIn("return", hints)

    def test_check_is_immutable_complete_and_iterable(self):
        with temporary() as temp:
            root = fixture(temp)
            result = self.g.check(root)
            self.assertTrue(dataclasses.is_dataclass(result))
            self.assertEqual("CheckResult", type(result).__name__)
            for name in ("report", "selected_report", "had_scan_errors", "output_format", "run_cautions"):
                self.assertTrue(hasattr(result, name), name)
            with self.assertRaises(dataclasses.FrozenInstanceError):
                result.had_scan_errors = True
            report = result.report
            self.assertEqual(tuple(report), report.errors + report.warnings + report.suggestions)
            self.assertEqual(("dangling", 3), (report.errors[0].code, report.errors[0].line))
            for group in (report.errors, report.warnings, report.suggestions, result.run_cautions):
                self.assertIsInstance(group, tuple)
            finding = report.errors[0]
            for field in ("path", "line", "column", "sites", "authority"):
                self.assertIn(field, plain(finding))
            self.assertIsInstance(finding.sites, tuple)
            self.assertIsInstance(finding.authority, tuple)
            self.assertEqual((), finding.authority)
            with self.assertRaises(dataclasses.FrozenInstanceError):
                finding.code = "hidden"
        with temporary() as temp:
            clean = self.g.check(fixture(temp, "clean"))
            self.assertEqual((), tuple(clean.report))
            self.assertEqual((), clean.run_cautions)

    def test_effective_config_mapping_is_read_only(self):
        from collections.abc import Mapping
        with temporary() as temp:
            result = self.g.effective_config(root=fixture(temp))
            values = [getattr(result, f.name) for f in dataclasses.fields(result)]
            mappings = [v for v in values if isinstance(v, Mapping)]
            self.assertTrue(mappings, "effective config has a schema mapping")
            def immutable(mapping):
                with self.assertRaises(TypeError):
                    mapping["new"] = True
                for value in mapping.values():
                    if isinstance(value, Mapping):
                        immutable(value)
            for mapping in mappings:
                immutable(mapping)

    def assert_failure(self, error):
        failure = error.exception.failure
        self.assertTrue(dataclasses.is_dataclass(failure))
        for name in ("code", "message", "path", "line", "column", "sites", "authority",
                     "causes", "run_cautions", "partial_output", "details"):
            self.assertTrue(hasattr(failure, name), name)
        self.assertIsInstance(failure.sites, tuple)
        self.assertIsInstance(failure.causes, tuple)
        self.assertIsInstance(failure.run_cautions, tuple)
        return failure

    def test_structured_query_errors_and_batch_outcomes(self):
        with temporary() as temp:
            root = fixture(temp)
            with self.assertRaises(self.g.QueryError) as error:
                self.g.show("FS-999-missing", root=root)
            failure = self.assert_failure(error)
            self.assertEqual("not-found", failure.code)
            self.assertIsNone(failure.path)
            self.assertIsNone(failure.line)
            self.assertIsNone(failure.column)
            self.assertEqual((), failure.authority)
            batch = self.g.show_batch((self.g.ShowQuery("FS-001-alpha"), "FS-999-missing"), root=root)
            values = [getattr(batch, f.name) for f in dataclasses.fields(batch)]
            records = [v for v in values if isinstance(v, tuple)
                       and v and hasattr(v[0], "failure")]
            self.assertEqual(1, len(records))
            self.assertIsNone(records[0][0].failure)
            self.assertIsNotNone(records[0][0].result)
            self.assertIsNone(records[0][1].result)
            self.assertEqual(plain(failure), plain(records[0][1].failure))
        with temporary() as temp:
            root = fixture(temp, "show-ambiguous-id-json")
            with self.assertRaises(self.g.QueryError) as error:
                self.g.show("FS-001-login", root=root)
            self.assertGreater(len(self.assert_failure(error).sites), 1)
        with temporary() as temp:
            root = fixture(temp, "refs-ambiguous-shorthand-json")
            with self.assertRaises(self.g.QueryError) as error:
                self.g.refs("FS-042", root=root)
            failure = self.assert_failure(error)
            self.assertEqual((), failure.sites)
            self.assertTrue(failure.details, "number-only ambiguity retains structured candidates")

    def test_config_filesystem_and_operation_failures_have_payloads(self):
        with temporary() as temp:
            root = fixture(temp, "check-invalid-config-json")
            with self.assertRaises(self.g.ConfigError) as error:
                self.g.check(root)
            self.assert_failure(error)
            self.g.show_batch((), root=root)  # Must not load invalid configuration.
            with self.assertRaises(self.g.ConfigError):
                self.g.show_batch(("FS-001-alpha",), root=root)
            with self.assertRaises(self.g.FilesystemError) as error:
                self.g.scan(Path(temp) / "does-not-exist")
            self.assert_failure(error)
        with temporary() as temp:
            with self.assertRaises(self.g.OperationError) as error:
                root = fixture(temp)
                self.g.propose_id("UNKNOWN", "Title", root=root)
            self.assert_failure(error)
            with self.assertRaises(self.g.QueryError) as error:
                self.g.propose_id("FS", "", root=root)
            self.assert_failure(error)
        for name in ("ConfigError", "FilesystemError", "QueryError", "OperationError"):
            self.assertTrue(issubclass(getattr(self.g, name), self.g.GrundError))

    def test_invalid_python_arguments_and_fetch_opt_in(self):
        with temporary() as temp:
            root = fixture(temp)
            before = tree_bytes(root)
            for function, args, keywords, exception in (
                ("check", (b"bytes",), {}, TypeError),
                ("check", (42,), {}, TypeError),
                ("check", ("bad\0path",), {}, ValueError),
                ("check", ("bad\udcffpath",), {}, self.g.PathEncodingError),
                ("show", ("FS-001-alpha",), {"mode": "unknown"}, ValueError),
                ("show", ("FS-001-alpha",), {"format": "unknown"}, ValueError),
                ("show", (42,), {}, TypeError),
                ("show_batch", (42,), {}, TypeError),
                ("list_ids", (), {"kinds": 42}, TypeError),
                ("list_sizes", (), {"units": ("pixels",)}, ValueError),
                ("list_sizes", (), {"top": 0}, ValueError),
                ("propose_id", ("FS", "Title"), {"width": -1}, ValueError),
                ("check", (), {"only_rule": True}, ValueError),
                ("fetch", ("FS-001-alpha",), {}, ValueError),
            ):
                with self.subTest(operation=function, args=ascii(args), options=keywords):
                    if function != "integrations" and (function != "check" or not args):
                        keywords = dict(keywords, root=root)
                    with self.assertRaises(exception):
                        getattr(self.g, function)(*args, **keywords)
            class BytesPath:
                def __fspath__(self):
                    return b"bytes"
            with self.assertRaises(TypeError):
                self.g.check(BytesPath())
            self.assertEqual(before, tree_bytes(root))

    def test_integration_invalid_preferences_refuse_without_writes(self):
        import os
        from unittest.mock import patch
        with temporary() as temp:
            home = Path(temp) / "home"
            home.mkdir()
            environment = {"HOME": str(home), "USERPROFILE": str(home),
                           "XDG_CONFIG_HOME": str(home / ".config")}
            with patch.dict(os.environ, environment):
                for args, keywords in (
                    (("unknown-client",), {}),
                    (("kitty",), {"conversation": "plain"}),
                    (("kitty",), {"write": True, "conversation": "invalid-preference"}),
                    (("kitty",), {"write": True, "conversation_target": "unknown"}),
                    (("kitty",), {"write": True, "agent": "unknown"}),
                    ((), {"write": True}),
                ):
                    with self.subTest(args=args, options=keywords), self.assertRaises(ValueError):
                        self.g.integrations(*args, **keywords)
                    self.assertEqual({}, tree_bytes(home))

    def test_fetch_opt_in_precedes_config_loading_and_fetcher_execution(self):
        with temporary() as temp:
            root = fixture(temp, "check-invalid-config-json")
            with self.assertRaises(ValueError):
                self.g.fetch("TICKET-1234", root=root)
        with temporary() as temp:
            import shlex
            root = fixture(temp, "fetch-workspace-folder")
            script = root / "packages/alpha/scripts/fetch-ticket"
            sentinel = Path(temp) / "fetcher-executed"
            script.write_text(script.read_text().replace("set -eu", "set -eu\nprintf executed > "
                                                        + shlex.quote(str(sentinel))))
            before = tree_bytes(root)
            with self.assertRaises(ValueError):
                self.g.fetch("alpha/TICKET-1234", root=root, write=False)
            self.g.check(root)
            self.g.scan(root)
            self.g.list_ids(root=root)
            with self.assertRaises(self.g.QueryError):
                self.g.show("alpha/TICKET-1234", root=root)
            self.assertFalse(sentinel.exists())
            self.assertEqual(before, tree_bytes(root))

    def test_selection_retains_complete_report_and_ignore_wins(self):
        with temporary() as temp:
            root = fixture(temp)
            complete = self.g.check(root)
            selected = self.g.check(root, only=("dangling",), ignore=("dangling",))
            self.assertEqual(plain(complete.report), plain(selected.report))
            self.assertEqual((), selected.selected_report.errors)

    def test_rule_authority_is_retained_in_the_selected_view(self):
        with temporary() as temp:
            root = fixture(temp)
            result = self.g.check(root, rule="Each FS must have exactly one Terms chapter.",
                                  only_rule=True)
            authored = [f for f in result.selected_report if "--rule" in f.authority]
            self.assertTrue(authored)
            self.assertTrue(all(f in tuple(result.report) for f in authored))
            self.assertTrue(all(tuple(sorted(f.authority)) == f.authority for f in authored))

    def test_safety_io_findings_survive_selectors(self):
        import os
        if os.name != "posix":
            self.skipTest("broken symlink safety fixture requires POSIX symlinks")
        with temporary() as temp:
            root = fixture(temp)
            (root / "docs/functional-spec/FS-003-broken.md").symlink_to("missing.md")
            result = self.g.check(root, only=("dangling",), ignore=("io",))
            self.assertTrue(result.had_scan_errors)
            self.assertTrue(any(f.code == "io" for f in result.report))
            self.assertTrue(any(f.code == "io" for f in result.selected_report))

    def test_show_ladder_formats_and_typed_queries(self):
        with temporary() as temp:
            root = fixture(temp)
            for mode in ("lead", "brief", "toc", "full"):
                for format_name in ("text", "md", "json"):
                    with self.subTest(mode=mode, format=format_name):
                        self.assertTrue(dataclasses.is_dataclass(self.g.show(
                            "FS-001-alpha", root=root, mode=mode, format=format_name)))
            query = self.g.ShowQuery("FS-001-alpha")
            self.assertIsNone(query.section)
            with self.assertRaises(dataclasses.FrozenInstanceError):
                query.id = "changed"

    def test_cautions_survive_success_and_failure(self):
        with temporary() as temp:
            result = self.g.check(fixture(temp, "workspace-include-root-false-unread-json"))
            self.assertTrue(result.run_cautions)
            self.assertEqual((), result.report.errors)
            self.assertEqual((), result.report.warnings)
        with temporary() as temp:
            root = fixture(temp, "workspace-nested-include-root-false-list-project")
            with self.assertRaises(self.g.GrundError) as error:
                self.g.list_ids(root=root, projects=("root",))
            self.assertTrue(self.assert_failure(error).run_cautions)
