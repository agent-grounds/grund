"""§FS-distribution.3.0.3 — complete Rust/Node data and distinct CLI-byte proofs.

§FS-distribution.3.0.3.1: Node field mapping and the frozen CLI wire projection.
"""

import errno
import json
import os
import shutil
import subprocess
import sys
import unittest

from support import CASES, REPO, Rehearsal, checked, command


class NodeParityTests(unittest.TestCase):
    def setUp(self):
        self.r = Rehearsal()
        self.addCleanup(self.r.close)

    def test_complete_data_matches_same_source_rust_oracle(self):
        # #470 owns the shared oracle. Never substitute CLI text for native fields.
        oracle = os.environ.get("GRUND_BINDINGS_ORACLE")
        self.assertTrue(oracle, "shared corpus prerequisite absent: GRUND_BINDINGS_ORACLE")
        metadata = json.loads(checked([oracle, "--metadata"]).stdout)
        self.assertEqual(checked(["git", "rev-parse", "HEAD"]).stdout.strip(),
                         metadata["sourceSha"])
        self.assertEqual("1", str(metadata["protocolVersion"]))
        package = json.loads((self.r.package / "package.json").read_text())
        self.assertEqual(package["version"], metadata["engineVersion"])
        shutil.copyfile(REPO / "tests/bindings/node/adapter.mjs", self.r.consumer / "adapter.mjs")
        requests = [
            ("check", [str(self.r.fixture)]),
            ("check", [str(self.r.fixture), {"only": ["dangling"], "ignore": ["dangling"]}]),
            ("scan", [str(self.r.fixture)]),
            ("show", ["FS-001-alpha", {"root": str(self.r.fixture), "format": "json"}]),
            ("show", ["FS-999-missing", {"root": str(self.r.fixture)}]),
            ("showBatch", [["FS-001-alpha", "FS-999-missing"], {"root": str(self.r.fixture)}]),
            ("refs", ["FS-999-missing", {"root": str(self.r.fixture)}]),
        ]
        for name in ("list", "listSizes", "cover", "fmt", "completeIds",
                     "effectiveConfig", "validateConfig"):
            requests.append((name, [{"root": str(self.r.fixture)}]))
        requests += [
            ("proposeId", ["FS", "雪 and snow", {"root": str(self.r.fixture)}]),
            ("init", [str(self.r.fixture), {"dryRun": True, "noVcs": True}]),
            ("fetch", ["FS-001-alpha", {"root": str(self.r.fixture)}]),
            ("integrations", []), ("referenceStyle", [str(self.r.fixture / "grund.toml")]),
            ("agentSetupInstructions", []),
        ]
        for case, operation, operand, options in [
            ("refs-ambiguous-id-total", "show", "FS-001-alpha", {}),
            ("workspace-refs-ambiguous-section-json", "show", "FS-001-alpha.1", {}),
            ("check-rules-section-target-counts", "check", None, {"suggestions": True}),
            ("check-escaped-citation-resolves-suggested", "check", None, {"suggestions": True}),
            ("config-redundant-pair-warning", "check", None, {}),
            ("config-redundant-pair-warning", "show", "FS-999-missing", {}),
            ("show-line-comment-control-toc-json", "show", "AR-control",
             {"mode": "toc", "format": "json"}),
        ]:
            extra = self.r.root / ("complete-" + case)
            if not extra.exists():
                shutil.copytree(CASES / case / "repo", extra)
            args = ([operand] if operand else []) + [{"root": str(extra), **options}]
            if operation == "check":
                args = [str(extra), options]
            requests.append((operation, args))
        # Authored control bytes exercise canonical C0 versus CLI DEL/C1 escaping.
        controls = self.r.fixture / "docs/functional-spec/FS-004-controls.md"
        controls.write_text("# FS-004-controls: 雪\n\n"
                            "literal \b \f \x7f \x85 and FS-998-control\n", encoding="utf-8")
        requests.append(("show", ["FS-004-controls", {"root": str(self.r.fixture),
                                                    "mode": "full", "format": "json"}]))
        requests.append(("check", [str(self.r.fixture)]))
        # §FS-distribution.3.2.1: both transports retain a root caution on member refusal.
        validation = self.r.root / "validation-cautions"
        (validation / ".agents").mkdir(parents=True)
        (validation / "child").mkdir()
        (validation / "grund.toml").write_text(
            'grund_config_version = 1\n[workspace]\nmembers = ["child"]\n')
        (validation / ".agents/grund.toml").write_text('grund_config_version = 1\n')
        (validation / "child/grund.toml").write_text('grund_config_version = "bad"\n')
        requests.append(("validateConfig", [{"root": str(validation)}]))
        # §FS-distribution.3.2.3.1: representable raw basenames refuse before alias derivation.
        # Windows cannot represent this raw-byte filename; the rest of the corpus runs there.
        raw_workspaces = {}
        if os.name == "posix":
            for redundant in (False, True):
                raw = self.r.root / ("raw-member-caution" if redundant else "raw-member")
                (raw / "members").mkdir(parents=True)
                (raw / "grund.toml").write_text(
                    'grund_config_version = 1\n[workspace]\nmembers = ["members/*"]\n',
                    encoding="utf-8",
                )
                member = os.fsencode(raw / "members") + b"/\xff"
                try:
                    os.mkdir(member)
                except OSError as error:
                    if sys.platform != "darwin" or error.errno != errno.EILSEQ:
                        raise
                    print(
                        "raw-member corpus representation unsupported: "
                        f"platform={sys.platform} path={member!r} errno={error.errno} (EILSEQ); "
                        "continuing the representable corpus",
                        file=sys.stderr, flush=True,
                    )
                    continue
                with open(member + b"/grund.toml", "wb") as config:
                    config.write(b"grund_config_version = 1\n")
                if redundant:
                    (raw / ".agents").mkdir()
                    (raw / ".agents/grund.toml").write_text('grund_config_version = 1\n')
                raw_workspaces[str(raw)] = redundant
                requests.append(("validateConfig", [{"root": str(raw)}]))
        if sys.platform.startswith("linux"):
            self.assertCountEqual([False, True], raw_workspaces.values(),
                                  "Linux requires both raw-member corpus variants")
        exercised_raw = set()
        for operation, args in requests:
            with self.subTest(operation=operation, args=args):
                request = json.dumps({"operation": operation, "args": args}) + "\n"
                native = subprocess.run([oracle], input=request, text=True, capture_output=True)
                node = subprocess.run(
                    ["node", "adapter.mjs"], input=request, text=True, capture_output=True,
                    cwd=self.r.consumer,
                )
                self.assertEqual(0, native.returncode, native.stderr)
                self.assertEqual(0, node.returncode, node.stderr)
                self.assertEqual("", native.stderr)
                self.assertEqual("", node.stderr)
                self.assertEqual(native.stdout.encode(), node.stdout.encode())
                envelope = json.loads(node.stdout)
                self.assertEqual({"failure", "result", "run_cautions"}, set(envelope))
                if operation == "validateConfig" and args[0]["root"] == str(validation):
                    self.assertEqual("config", envelope["failure"]["kind"])
                    self.assertEqual("invalid-config", envelope["failure"]["code"])
                    self.assertIsNone(envelope["failure"]["partial"])
                    self.assertIn("redundant-config",
                                  [f["code"] for f in envelope["run_cautions"]])
                if operation == "validateConfig" and args[0]["root"] in raw_workspaces:
                    # §FS-distribution.3.2.2.1: expected fields come from shared preflight.
                    self.assertIsNone(envelope["result"])
                    self.assertEqual({
                        "kind": "input", "code": "path-encoding", "operation": operation,
                        "message": "unnamed workspace member has a non-Unicode basename; "
                                   "configure project_name",
                        "path": None, "line": None, "column": None, "partial": None,
                        "sites": [], "authority": [], "causes": [], "details": {},
                    }, envelope["failure"])
                    expected = ["redundant-config"] if raw_workspaces[args[0]["root"]] else []
                    self.assertEqual(expected, [f["code"] for f in envelope["run_cautions"]])
                    exercised_raw.add(args[0]["root"])
        self.assertSetEqual(set(raw_workspaces), exercised_raw,
                            "Every appended raw-member variant must pass all comparisons")

    def test_same_version_cli_check_stdout_stderr_and_status(self):
        target = self.r.root / "cli-target"
        checked(["cargo", "build", "-p", "grund", "--locked", "--target-dir", target])
        cli = target / "debug" / ("grund.exe" if os.name == "nt" else "grund")
        manifest = json.loads((self.r.package / "package.json").read_text())
        self.assertEqual("grund " + manifest["version"], checked([cli, "--version"]).stdout.strip())
        shutil.copyfile(REPO / "tests/bindings/node/check-wire.mjs",
                        self.r.consumer / "check-wire.mjs")
        for case, suggestions in [
            ("json-report", False), ("check-empty-scan-warning", False),
            ("check-escaped-citation-resolves-suggested", True),
            ("check-rules-chapter-absent-suggestion", True),
            ("config-redundant-pair-warning", False),
            ("workspace-refs-ambiguous-section-json", False),
        ]:
            with self.subTest(case=case):
                root = self.r.root / case
                shutil.copytree(CASES / case / "repo", root)
                flags = ["--suggestions"] if suggestions else []
                native = command([cli, "check", root, "--format=json", *flags])
                node = command(["node", "check-wire.mjs", root,
                                json.dumps({"suggestions": suggestions})], cwd=self.r.consumer)
                self.assertEqual(native.returncode, node.returncode)
                self.assertEqual(native.stdout.encode(), node.stdout.encode())
                self.assertEqual(native.stderr.encode(), node.stderr.encode())

    def test_command_wire_shapes_controls_query_sites_and_order(self):
        target = self.r.root / "cli-target"
        checked(["cargo", "build", "-p", "grund", "--locked", "--target-dir", target])
        cli = target / "debug" / ("grund.exe" if os.name == "nt" else "grund")
        for script in ("query-wire.mjs", "wire-codec.mjs"):
            shutil.copyfile(REPO / "tests/bindings/node" / script, self.r.consumer / script)
        for case, operation, operand, flags, options in [
            ("json-report", "show", "FS-001-alpha", [], {"format": "json"}),
            ("json-report", "show", "FS-999-missing", [], {"format": "json"}),
            ("refs-ambiguous-id-total", "refs", "FS-001-alpha", [], {}),
            ("workspace-refs-ambiguous-section-json", "refs", "FS-001-alpha.1", [], {}),
            ("show-line-comment-control-toc-json", "show", "AR-control", ["--toc"],
             {"format": "json", "mode": "toc"}),
            ("json-report", "refs", "FS-002-beta", [], {}),
            ("json-report", "refs", "FS-002-beta", ["--summary"], {}),
            ("json-report", "refs", "FS-002-beta", ["--total"], {}),
            ("json-report", "list", None, [], {}),
            ("json-report", "listSizes", None, ["--size=bytes,lines"], {"units": ["bytes", "lines"]}),
            ("json-report", "cover", None, [], {}),
        ]:
            with self.subTest(case=case, operation=operation):
                root = self.r.root / ("wire-" + case + "-" + operation + "-" + str(len(flags)))
                if not root.exists():
                    shutil.copytree(CASES / case / "repo", root)
                args = ([operand] if operand is not None else []) + [{"root": str(root), **options}]
                projection = "summary" if "--summary" in flags else "total" if "--total" in flags else "detail"
                request = json.dumps({"operation": operation, "args": args, "projection": projection})
                cli_name = "list" if operation == "listSizes" else operation
                argv = [cli, cli_name, *([operand] if operand else []), root, "--format=json", *flags]
                native = command(argv)
                node = command(["node", "query-wire.mjs", root, request], cwd=self.r.consumer)
                self.assertEqual(native.returncode, node.returncode, node.stderr)
                self.assertEqual(native.stdout.encode(), node.stdout.encode())
                self.assertEqual(native.stderr.encode(), node.stderr.encode())


if __name__ == "__main__":
    unittest.main()
