"""§AR-core-module-layout.1 — the engine is one Rust module per component: every
`.rs` file under `crates/grund-core/src/` is `lib.rs`, the shared test fixtures
in `testing.rs`, or a file inside one of the twelve component directories
§AR-system.2 names; each of those directories exists and declares its files in a
`mod.rs`, which is the whole of what crosses its boundary; and `lib.rs` splices
in nothing at all, because a test module sits beside the code it pins and is a
`mod` line in its component's `mod.rs` like any other file."""

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CORE = REPO_ROOT / "crates" / "grund-core" / "src"

# One directory per component of §AR-system.2, named after it — the eleven
# components and the deprecated path beside the api (§AR-system.2.9).
COMPONENTS = (
    "api",
    "checker",
    "compat",
    "config",
    "grammar",
    "model",
    "queries",
    "resolver",
    "scanner",
    "templates",
    "workspace",
    "writers",
)

INCLUDE = re.compile(r'include!\("([^"]+)"\)')


def _relative(path):
    return path.relative_to(CORE).as_posix()


class ModuleLayoutTests(unittest.TestCase):
    def test_every_component_has_a_directory_with_a_mod_rs(self):
        missing = []
        for component in COMPONENTS:
            directory = CORE / component
            if not directory.is_dir():
                missing.append(f"{component}/ is not a directory")
            elif not (directory / "mod.rs").is_file():
                missing.append(f"{component}/mod.rs is missing")
        self.assertEqual([], missing, "\n".join(missing))

    def test_every_source_file_is_lib_the_fixtures_or_inside_a_component(self):
        """A file at the crate root that is not `lib.rs` or `testing.rs` belongs
        to some component, and the directory is how it says which. A test module
        is no exception: it belongs to the component whose entry points its cases
        exercise."""
        stray = sorted(
            _relative(path)
            for path in CORE.glob("**/*.rs")
            if path.parent == CORE and path.name not in ("lib.rs", "testing.rs")
        )
        self.assertEqual([], stray, "engine files that belong to no component directory")

    def test_no_file_sits_outside_the_component_directories(self):
        """And nothing hides in a directory the page does not name."""
        outside = sorted(
            _relative(path)
            for path in CORE.glob("**/*.rs")
            if path.parent != CORE and path.relative_to(CORE).parts[0] not in COMPONENTS
        )
        self.assertEqual([], outside, "engine files under a directory that is not a component")

    def test_lib_rs_includes_nothing(self):
        """`include!` is what the flat crate was assembled from, and nothing is
        assembled that way any more: every file, test module included, is a `mod`
        line in its component's `mod.rs`."""
        spliced = INCLUDE.findall((CORE / "lib.rs").read_text(encoding="utf-8"))
        self.assertEqual([], spliced, "lib.rs still splices files into the crate root")


if __name__ == "__main__":
    unittest.main()
