"""§AR-bindings.2 — the engine returns data and the frontends render it: the
embedding API (`api/`) writes to no stream and exits no process; terminal
rendering inside `grund-core` exists only on the deprecated
`grund_core::main_entry()` path, in one directory that may shrink and never
grow unnoticed; and the published CLI and the LSP server import none of it —
`grund` calls its own `main_entry`, never the engine's."""

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CORE = REPO_ROOT / "crates" / "grund-core" / "src"
FRONTENDS = (REPO_ROOT / "crates" / "grund-cli" / "src", REPO_ROOT / "crates" / "grund-lsp" / "src")
STREAM_OR_EXIT = re.compile(r"\b(e?println!|e?print!\(|io::stdout|io::stderr|process::exit)")
# The one directory the deprecated `main_entry()` path renders through, and a
# boundary a file printing elsewhere widens by decision rather than by slip
# (§AR-system.2.9).
COMPAT = "compat"
ENGINE_ONLY_SYMBOLS = ("grund_core::main_entry", "compat_cli", "grund_core::command_")


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

    def test_rendering_in_the_engine_stays_inside_the_compat_directory(self):
        writers = {
            _relative(path)
            for path in _implementation_files()
            if _writes_to_a_stream(path) and path.parent != CORE / COMPAT
        }
        self.assertEqual(
            set(),
            writers,
            "engine files that write to a stream or exit outside the deprecated "
            "main_entry() rendering directory",
        )

    def test_the_compat_directory_is_only_files_that_still_render(self):
        """The set may shrink, and the directory is the set: a file under
        `compat/` that no longer prints belongs with the component it serves. The
        directory's own test module is not one of those files — it pins what the
        renderers do rather than rendering (§AR-core-module-layout.1)."""
        idle = {
            _relative(path)
            for path in _implementation_files()
            if path.parent == CORE / COMPAT
            and path.name != "mod.rs"
            and not _writes_to_a_stream(path)
        }
        self.assertEqual(set(), idle, "compat files that render nothing: move them out")

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
