"""§AR-bindings.2 — the engine returns data and the frontends render it: no
source inside `grund-core` writes to a stream or exits a process, the deprecated
`grund_core::main_entry()` process frontend and its test machinery are absent,
and `grund` calls the CLI's own `main_entry` while the LSP transports engine
data."""

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CORE = REPO_ROOT / "crates" / "grund-core" / "src"
INTEGRATION = REPO_ROOT / "tests" / "integration"
FRONTENDS = (REPO_ROOT / "crates" / "grund-cli" / "src", REPO_ROOT / "crates" / "grund-lsp" / "src")
STREAM_OR_EXIT = re.compile(r"\b(e?println!|e?print!\(|io::stdout|io::stderr|process::exit)")
ENGINE_ONLY_SYMBOLS = ("grund_core::main_entry", "compat_cli", "grund_core::command_")
REMOVED_TEST_PATHS = (
    "check_frontend_parity.rs",
    "compat_frontend.rs",
    "refs_frontend_parity.rs",
)
REMOVED_HARNESS_MARKERS = (
    "grund_core_compat",
    "grund-core-compat-test",
    'path = "check_frontend_parity.rs"',
    'path = "compat_frontend.rs"',
    'path = "refs_frontend_parity.rs"',
)


def _implementation_files():
    """Every engine source file except the crate's own `tests_*` modules."""
    return sorted(
        path
        for path in CORE.glob("**/*.rs")
        if not path.name.startswith("tests")
    )


def _writes_to_a_stream(path):
    return bool(STREAM_OR_EXIT.search(path.read_text(encoding="utf-8")))


def _relative(path):
    return path.relative_to(CORE).as_posix()


class EngineBoundaryTests(unittest.TestCase):
    def test_the_embedding_api_writes_nothing(self):
        api = sorted((CORE / "api").glob("**/*.rs"))
        self.assertTrue(api, "the api module directory is missing")
        for path in api:
            with self.subTest(file=_relative(path)):
                self.assertFalse(_writes_to_a_stream(path), f"{_relative(path)} renders or exits")

    def test_no_engine_source_renders_or_exits(self):
        writers = {
            _relative(path)
            for path in _implementation_files()
            if _writes_to_a_stream(path)
        }
        self.assertEqual(
            set(),
            writers,
            "engine files that write to a stream or exit a process",
        )

    def test_the_compat_module_and_main_entry_export_are_absent(self):
        """§FS-distribution.3.1.1: the completed removal has no module and no
        crate-root spelling through which an embedder can still call it."""
        lib = (CORE / "lib.rs").read_text(encoding="utf-8")
        stale = []
        if (CORE / "compat").exists():
            stale.append("crates/grund-core/src/compat/ still exists")
        for marker in ("mod compat;", "pub use compat::main_entry;"):
            if marker in lib:
                stale.append(f"crates/grund-core/src/lib.rs still contains {marker!r}")
        self.assertEqual([], stale, "\n".join(stale))

    def test_the_compatibility_frontend_test_machinery_is_absent(self):
        """The deleted process frontend has no binary probe, helper, Cargo test
        target, or parity suite left behind (§AR-system.2.9.1)."""
        stale = [
            f"tests/integration/{name} still exists"
            for name in REMOVED_TEST_PATHS
            if (INTEGRATION / name).exists()
        ]
        sources = [INTEGRATION / "Cargo.toml", *sorted(INTEGRATION.glob("*.rs"))]
        for path in sources:
            text = path.read_text(encoding="utf-8")
            for marker in REMOVED_HARNESS_MARKERS:
                if marker in text:
                    stale.append(
                        f"{path.relative_to(REPO_ROOT).as_posix()} still contains {marker!r}"
                    )
        self.assertEqual([], stale, "\n".join(stale))

    def test_the_frontends_import_no_engine_rendering(self):
        for frontend in FRONTENDS:
            for path in sorted(frontend.glob("**/*.rs")):
                text = path.read_text(encoding="utf-8")
                for symbol in ENGINE_ONLY_SYMBOLS:
                    with self.subTest(file=path.relative_to(REPO_ROOT).as_posix(), symbol=symbol):
                        self.assertNotIn(symbol, text)

    def test_the_cli_owns_its_own_main_entry(self):
        cli = REPO_ROOT / "crates" / "grund-cli" / "src"
        self.assertIn("grund::main_entry()", (cli / "main.rs").read_text(encoding="utf-8"))
        self.assertTrue(
            any("pub fn main_entry()" in path.read_text(encoding="utf-8") for path in cli.glob("*.rs")),
            "grund-cli defines the main_entry the binary calls",
        )


if __name__ == "__main__":
    unittest.main()
