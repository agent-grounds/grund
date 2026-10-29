"""§AR-benchmarks.1.1 — the generated large conformant fixture the
instruction-counting benches read: what the generator writes is conformant, and
the same arguments write the same tree, under a manifest that names its schema and
content digest."""

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path


SCRIPT_PATH = (
    Path(__file__).resolve().parents[2] / "scripts" / "generate_large_benchmark_fixture.py"
)
SPEC = importlib.util.spec_from_file_location("generate_large_benchmark_fixture", SCRIPT_PATH)
generate_large_benchmark_fixture = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(generate_large_benchmark_fixture)


class GenerateLargeBenchmarkFixtureTests(unittest.TestCase):
    def test_generates_requested_file_count_and_config(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / "fixture"
            generate_large_benchmark_fixture.generate_fixture(root, 12, 3)

            declaration_files = sorted(
                (root / "docs" / "functional-spec").glob("component-*/*.md")
            )
            self.assertEqual(len(declaration_files), 12)
            self.assertTrue((root / "grund.toml").exists())
            self.assertEqual(
                declaration_files[0].relative_to(root).as_posix(),
                "docs/functional-spec/component-000/FS-00001-feature-00001.md",
            )
            self.assertIn(
                "\u00a7FS-00002-feature-00002",
                declaration_files[0].read_text(encoding="utf-8"),
            )
            self.assertIn(
                "\u00a7FS-00001-feature-00001",
                declaration_files[-1].read_text(encoding="utf-8"),
            )

            # §FS-check.3.18: every declaration must be a linked index entry, or
            # `grund check` errors on the generated fixture instead of measuring it.
            index_text = (root / "docs" / "functional-spec" / "README.md").read_text(
                encoding="utf-8"
            )
            self.assertIn(
                "[\u00a7FS-00001-feature-00001](component-000/FS-00001-feature-00001.md)",
                index_text,
            )
            self.assertIn(
                "[\u00a7FS-00012-feature-00012](component-002/FS-00012-feature-00012.md)",
                index_text,
            )

    def test_replaces_existing_fixture_deterministically(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / "fixture"
            generate_large_benchmark_fixture.generate_fixture(root, 4, 2)
            first = {
                path.relative_to(root).as_posix(): path.read_text(encoding="utf-8")
                for path in sorted(root.rglob("*"))
                if path.is_file()
            }

            (root / "stale.txt").write_text("stale", encoding="utf-8")
            generate_large_benchmark_fixture.generate_fixture(root, 4, 2)
            second = {
                path.relative_to(root).as_posix(): path.read_text(encoding="utf-8")
                for path in sorted(root.rglob("*"))
                if path.is_file()
            }

            self.assertEqual(first, second)
            self.assertNotIn("stale.txt", second)

    def test_rejects_empty_fixture(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            with self.assertRaisesRegex(ValueError, "--files"):
                generate_large_benchmark_fixture.generate_fixture(Path(tmp) / "fixture", 0, 1)


class FixtureManifestTests(unittest.TestCase):
    """§AR-benchmarks.1.1: every fixture names its schema version and content digest."""

    def manifest(self, root: Path) -> dict:
        path = root / "manifest.json"
        self.assertTrue(path.is_file(), f"the generator wrote no {path.name} into the fixture")
        return json.loads(path.read_text(encoding="utf-8"))

    def test_the_manifest_names_the_schema_and_a_sha256_digest(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / "fixture"
            generate_large_benchmark_fixture.generate_fixture(root, 4, 2)
            manifest = self.manifest(root)
            self.assertIsInstance(manifest.get("schema_version"), int)
            self.assertRegex(manifest.get("digest", ""), r"^[0-9a-f]{64}$")
            self.assertEqual(
                generate_large_benchmark_fixture.SCHEMA_VERSION, manifest["schema_version"]
            )
            self.assertEqual(
                generate_large_benchmark_fixture.fixture_digest(root), manifest["digest"]
            )

    def test_equal_arguments_give_equal_digests(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            first, second = Path(tmp) / "first", Path(tmp) / "second"
            generate_large_benchmark_fixture.generate_fixture(first, 4, 2, citations=True)
            generate_large_benchmark_fixture.generate_fixture(second, 4, 2, citations=True)
            self.assertEqual(self.manifest(first)["digest"], self.manifest(second)["digest"])

    def test_other_arguments_give_another_digest(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            plain, cited = Path(tmp) / "plain", Path(tmp) / "cited"
            generate_large_benchmark_fixture.generate_fixture(plain, 4, 2)
            generate_large_benchmark_fixture.generate_fixture(cited, 4, 2, citations=True)
            self.assertNotEqual(self.manifest(plain)["digest"], self.manifest(cited)["digest"])

    def test_one_changed_byte_changes_the_digest(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / "fixture"
            generate_large_benchmark_fixture.generate_fixture(root, 4, 2)
            recorded = self.manifest(root)["digest"]
            path = root / "docs" / "functional-spec" / "component-000" / "FS-00001-feature-00001.md"
            data = bytearray(path.read_bytes())
            data[-2] ^= 1
            path.write_bytes(bytes(data))
            self.assertNotEqual(recorded, generate_large_benchmark_fixture.fixture_digest(root))

    def test_the_digest_leaves_the_manifest_out(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / "fixture"
            generate_large_benchmark_fixture.generate_fixture(root, 4, 2)
            recorded = self.manifest(root)["digest"]
            (root / "manifest.json").write_text("{}", encoding="utf-8")
            self.assertEqual(recorded, generate_large_benchmark_fixture.fixture_digest(root))

if __name__ == "__main__":
    unittest.main()
