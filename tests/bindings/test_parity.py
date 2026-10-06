"""Complete data/byte parity (§FS-distribution.3.0.3, §FS-distribution.3.3.6)."""

import os
import json
from pathlib import Path
import shutil
import unittest
import unicodedata
from unittest.mock import patch

from corpus import FILE_READ_CASES, READ_CASES, MUTATIONS
from support import REPO, binding, canonical, fixture, plain, python_call, rust_call, temporary, tree_bytes


def cli_json(value):
    """Ordered frozen CLI JSON, including its control escapes (§FS-distribution.3.0.3)."""
    if isinstance(value, str):
        escapes = {'"': '\\"', '\\': '\\\\', '\n': '\\n', '\r': '\\r', '\t': '\\t'}
        return '"' + ''.join(escapes.get(c, f'\\u{ord(c):04x}'
                                       if unicodedata.category(c) == "Cc" else c)
                             for c in value) + '"'
    if isinstance(value, dict):
        return '{' + ','.join(cli_json(k) + ':' + cli_json(v) for k, v in value.items()) + '}'
    if isinstance(value, list):
        return '[' + ','.join(cli_json(v) for v in value) + ']'
    return json.dumps(value, allow_nan=False, separators=(',', ':'))


class ParityTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.module = binding()

    def compare(self, operation, root, args, options, home=None):
        expected, wire, response = rust_call(operation, root, args, options, home)
        actual = python_call(self.module, operation, root, args, options)
        self.assertEqual(expected, actual, "complete field/order/null parity")
        self.assertEqual(wire, canonical(actual), "independently encoded canonical bytes")
        return actual, response

    def test_shared_read_corpus_complete_data_and_bytes(self):
        for case, operation, args, options in READ_CASES:
            with self.subTest(case=case, operation=operation, options=options), temporary() as temp:
                root = fixture(temp, case)
                before = tree_bytes(root)
                actual, response = self.compare(operation, root, args, options)
                if operation == "check" and actual["result"] is not None:
                    self.assert_frozen_cli_projection(actual["result"]["report"],
                                                      actual["run_cautions"], response)
                self.assertEqual(before, tree_bytes(root), "read operation wrote files")

    def test_shared_file_read_corpus_complete_data_and_bytes(self):
        # §FS-cover.6.1: `cover(lines=...)` names one file, not a tree.
        for case, path, operation, args, options in FILE_READ_CASES:
            with self.subTest(case=case, operation=operation, options=options), temporary() as temp:
                root = fixture(temp, case)
                before = tree_bytes(root)
                self.compare(operation, root / path, args, options)
                self.assertEqual(before, tree_bytes(root), "read operation wrote files")

    def assert_frozen_cli_projection(self, report, cautions, response):
        # Construct expected records from every host field the frozen wire uses,
        # not from the Rust driver's projection. Stable sort preserves channel ties.
        rows = []
        for group in ("warnings", "errors", "suggestions"):
            for finding in report[group]:
                row = {"channel": "suggestion"} if group == "suggestions" else {
                    "severity": finding["severity"]}
                sites = [{"path": site["path"], "line": site["line"]}
                         for site in finding["sites"]]
                row.update(path=finding["path"], line=finding["line"], code=finding["code"],
                           message=finding["message"], sites=sites or None,
                           authority=finding["authority"] or None)
                rows.append(row)
        rows.sort(key=lambda row: (row["path"] is not None, row["path"] or "",
                                   row["line"] or 0, row["message"]))
        # §FS-distribution.3.0.3: cautions precede stderr findings as text.
        caution_text = "".join("warning: " + f["message"] + "\n" for f in cautions)
        for stream, located in (("stdout", True), ("stderr", False)):
            expected = [row for row in rows if (row["line"] is not None) == located]
            prefix = caution_text if stream == "stderr" else ""
            wire = response["cli_" + stream]
            self.assertTrue(wire.startswith(prefix))
            actual = [json.loads(line) for line in wire[len(prefix):].splitlines()]
            self.assertEqual(expected, actual)
            for row in actual:
                self.assertEqual([next(iter(row)), "path", "line", "code", "message",
                                  "sites", "authority"], list(row))
                self.assertNotIn("column", row)
            serialized = "".join(cli_json(row) + "\n" for row in expected)
            self.assertEqual((prefix + serialized).encode("utf-8"), wire.encode("utf-8"))

    def test_cli_json_goldens_remain_authoritative(self):
        for case in ("json-report", "check-invalid-config-json",
                     "workspace-check-json-broken"):
            with self.subTest(case=case), temporary() as temp:
                root = fixture(temp, case)
                _, response = self.compare("check", root, (), {})
                source = REPO / "tests/e2e/cases" / case
                for stream in ("stdout", "stderr"):
                    golden = (source / ("expected." + stream)).read_bytes()
                    # §FS-distribution.3.0.3: the CLI golden reader's lone-LF sentinel.
                    expected = b"" if golden == b"\n" else golden
                    self.assertEqual(expected, response["cli_" + stream].encode("utf-8"))

    def test_mutation_preview_write_and_refusal_bytes_match_core(self):
        for case, operation, args, options in MUTATIONS:
            with self.subTest(case=case, operation=operation, options=options), temporary() as temp:
                root = fixture(temp, case)
                backup = Path(temp) / "backup"
                shutil.copytree(root, backup)
                home = Path(temp) / "home"
                home.mkdir()
                # Preserve manual user content; give the agent gate a real in-use
                # marker so writes exercise both accepted and downgraded targets.
                (home / ".codex").mkdir()
                (home / ".codex/AGENTS.md").write_text("Personal instruction to preserve.\n")
                (home / ".config/kitty").mkdir(parents=True)
                (home / ".config/kitty/kitty.conf").write_text("font_size 13\n")
                home_backup = Path(temp) / "home-backup"
                shutil.copytree(home, home_backup)
                before_home = tree_bytes(home)
                before = tree_bytes(root)
                environment = {"HOME": str(home), "USERPROFILE": str(home),
                               "XDG_CONFIG_HOME": str(home / ".config")}
                with patch.dict(os.environ, environment):
                    expected, wire, _ = rust_call(operation, root, args, options, home)
                    expected_files, expected_home = tree_bytes(root), tree_bytes(home)
                    shutil.rmtree(root)
                    shutil.copytree(backup, root)
                    shutil.rmtree(home)
                    shutil.copytree(home_backup, home)
                    actual = python_call(self.module, operation, root, args, options)
                    self.assertEqual(expected, actual)
                    self.assertEqual(wire, canonical(actual))
                    self.assertEqual(expected_files, tree_bytes(root))
                    self.assertEqual(expected_home, tree_bytes(home))
                    if not options.get("write") or options.get("check"):
                        self.assertEqual(before, tree_bytes(root))
                        self.assertEqual(before_home, tree_bytes(home))
                    if operation == "init" and (backup / "grund.toml").exists():
                        self.assertEqual((backup / "grund.toml").read_bytes(),
                                         (root / "grund.toml").read_bytes(), "force replaced config")

    def test_unicode_control_characters_and_logical_paths(self):
        with temporary() as temp:
            root = fixture(temp)
            source = root / "docs/functional-spec/FS-001-alpha.md"
            target = source.with_name("FS-001-façade-東京.md")
            source.rename(target)
            target.write_text('# FS-001-alpha: façade 東京 "\\\t\n\n'
                              + chr(167) + 'FS-999-missing\n\nBody \b\f\x01\r\n')
            for operation, args, options in (("check", (), {}),
                                              ("show", ("FS-001-alpha",), {"mode": "full"}),
                                              ("scan", (), {})):
                actual, _ = self.compare(operation, root, args, options)
                if operation == "check":
                    findings = actual["result"]["report"]["errors"]
                    self.assertTrue(findings)
                    self.assertTrue(all("\\" not in f["path"] for f in findings if f["path"]))

    def test_canonical_test_encoder_has_exact_escaping(self):
        # Baseline guard: it passes before native support, and proves only scaffolding.
        value = {"z": "é/東京", "a": [None, "\b\f\x01\n\r\t\\b\""]}
        self.assertEqual(b'{"a":[null,"\\u0008\\u000c\\u0001\\n\\r\\t\\\\b\\\""],'
                         b'"z":"\xc3\xa9/\xe6\x9d\xb1\xe4\xba\xac"}\n', canonical(value))
