"""Fresh Git checkouts with both binaries for §FS-cochange-recipe.examples."""
import os
from pathlib import Path
import shutil
import subprocess
from cochange_fixture import RecipeCase


class CompatibilityTests(RecipeCase):
    def test_fixture_queries_are_healthy_before_recipe_evaluation(self):
        # This control pins existing query/setup invariants and should pass today.
        binaries = [os.environ["GRUND_BUILT"], os.environ["GRUND_RELEASED"]]
        for binary in binaries:
            for query in ("check", "cover", "list"):
                with self.subTest(binary=binary, query=query):
                    result = subprocess.run([binary, query, ".", "--format=json"],
                        cwd=self.fixture.root, capture_output=True, text=True)
                    self.assertEqual(0, result.returncode, result.stdout + result.stderr)
                    self.assertTrue(result.stdout.splitlines())

    def test_fresh_checkout_with_built_and_released_0161(self):
        self.fixture.change(spec=True, test=True)
        self.fixture.commit("Complete evidence")
        original = self.fixture.root
        binaries = (
            ("built", os.environ.get("GRUND_BUILT", str(Path(__file__).resolve().parents[2] / "target/debug/grund"))),
            ("released-0.16.1", os.environ.get("GRUND_RELEASED", shutil.which("grund"))),
        )
        for name, binary in binaries:
            with self.subTest(binary=name):
                clone = Path(self.temp.name) / name
                subprocess.run(["git", "clone", "-q", str(original), str(clone)], check=True)
                self.fixture.root = clone
                if name == "released-0.16.1":
                    version = subprocess.run([binary, "--version"], capture_output=True, text=True, check=True)
                    self.assertEqual("grund 0.16.1\n", version.stdout,
                                     "GRUND_RELEASED must name released 0.16.1, never skip compatibility")
                report = self.report(self.fixture.invoke(grund=binary), 0)
                self.assertEqual([], self.source_row(report)["missing"])
                self.assertEqual("", self.fixture.git("status", "--porcelain"))
