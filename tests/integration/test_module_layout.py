"""§AR-core-module-layout.1 — the engine is one Rust module per component: every
`.rs` file under `crates/grund-core/src/` is `lib.rs`, the shared test fixtures
in `testing.rs`, or a file inside one of the twelve component directories
§AR-system.2 names; each directory declares its files in a `mod.rs`, which is
the whole of what crosses its boundary; the retired `compat/` directory is not
a component; and `lib.rs` splices nothing at all."""

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CORE = REPO_ROOT / "crates" / "grund-core" / "src"

# One directory per component of §AR-system.2, named after it.
COMPONENTS = (
    "api",
    "checker",
    "config",
    "grammar",
    "model",
    "queries",
    "resolver",
    "rules",
    "scanner",
    "templates",
    "workspace",
    "writers",
)

# §AR-rules.6: `rules` may be absent or test-only before implementation; the
# commit declaring production `mod rules;` removes this marker and enables the
# four contract drivers.
PENDING_COMPONENTS = set()

INCLUDE = re.compile(r'include!\("([^"]+)"\)')


def _relative(path):
    return path.relative_to(CORE).as_posix()


def _production_modules():
    """Modules declared without a neighbouring `#[cfg(test)]` attribute."""
    lines = (CORE / "lib.rs").read_text(encoding="utf-8").splitlines()
    modules = set()
    for index, line in enumerate(lines):
        match = re.fullmatch(r"mod ([a-z_][a-z0-9_]*);", line.strip())
        if not match:
            continue
        attributes = []
        cursor = index - 1
        while cursor >= 0 and lines[cursor].strip().startswith("#["):
            attributes.append(lines[cursor].strip())
            cursor -= 1
        if "#[cfg(test)]" not in attributes:
            modules.add(match.group(1))
    return modules


class ModuleLayoutTests(unittest.TestCase):
    def test_the_deprecated_compat_directory_is_absent(self):
        """§AR-system.2.9.1 leaves no process-frontend module beside `api/`."""
        self.assertFalse(CORE.joinpath("compat").exists(), "compat/ still exists")

    def test_every_component_has_a_directory_with_a_mod_rs(self):
        missing = []
        for component in COMPONENTS:
            directory = CORE / component
            if component in PENDING_COMPONENTS and not directory.exists():
                continue
            if not directory.is_dir():
                missing.append(f"{component}/ is not a directory")
            elif not (directory / "mod.rs").is_file():
                missing.append(f"{component}/mod.rs is missing")
        self.assertEqual([], missing, "\n".join(missing))

    def test_a_declared_component_is_not_still_marked_pending(self):
        """The architecture-only allowance cannot survive implementation."""
        stale = sorted(
            component
            for component in PENDING_COMPONENTS
            if component in _production_modules()
        )
        self.assertEqual(
            [],
            stale,
            "declared components still marked pending; remove the allowance and "
            "enable their boundary contracts",
        )

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
