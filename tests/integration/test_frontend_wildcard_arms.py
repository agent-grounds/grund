"""§AR-bindings.2.1, §FS-distribution.3.1.2 — a frontend is another crate, so each
`match` it makes on a promised enum ends in a wildcard arm, and that arm takes
away the compiler's prompt when grund-core adds a variant. An arm with nothing
of the variant to render is `unnamed_variant::<E>()`, and each file that calls
it names every variant `E` has today, so a variant added in this workspace is
red here until the frontend renders it."""

import re
import unittest

from test_public_surface_inventory import (
    FROZEN, REPO_ROOT, _core_sources, _sample, inventory_dispositions,
)


FRONTENDS = ("grund-cli", "grund-lsp", "grund-py", "grund-node")
ARM = re.compile(r"\bunnamed_variant::<([A-Za-z_][A-Za-z0-9_]*)>\(")


def wildcard_arms():
    """`(path, enum)` for each `unnamed_variant::<enum>()` a frontend's source calls."""
    arms = []
    for frontend in FRONTENDS:
        for path in sorted((REPO_ROOT / "crates" / frontend / "src").rglob("*.rs")):
            text = path.read_text(encoding="utf-8")
            arms.extend((path, name) for name in ARM.findall(text))
    return arms


def enum_variants(name):
    """The variants of grund-core's `pub enum <name>`, read at its definition, or
    `None` when grund-core defines no such enum."""
    for _, text in _core_sources():
        body = re.search(rf"^pub enum {name}\b[^{{]*\{{\n(.*?)^\}}", text, re.MULTILINE | re.DOTALL)
        if body:
            return set(re.findall(r"^    ([A-Z][A-Za-z0-9_]*)\b", body.group(1), re.MULTILINE))
    return None


class FrontendWildcardArmTests(unittest.TestCase):
    """§FS-distribution.3.1.2: a wildcard arm is for a variant this release does
    not name, never for one it does."""

    @classmethod
    def setUpClass(cls):
        cls.arms = wildcard_arms()
        cls.frozen = {n for n, d in inventory_dispositions().items() if d == FROZEN}

    def test_the_frontends_have_wildcard_arms_to_read(self):
        self.assertTrue(self.arms, "no `unnamed_variant::<…>()` call in any frontend")

    def test_each_arm_is_for_a_promised_enum(self):
        stray = {name for _, name in self.arms if name not in self.frozen or not enum_variants(name)}
        self.assertEqual("", _sample(stray), "wildcard arms for no promised enum of grund-core")

    def test_each_file_with_an_arm_names_every_variant_of_today(self):
        for path, name in self.arms:
            with self.subTest(file=path.relative_to(REPO_ROOT).as_posix(), enum=name):
                named = set(re.findall(rf"\b{name}::([A-Z][A-Za-z0-9_]*)", path.read_text(encoding="utf-8")))
                unnamed = (enum_variants(name) or set()) - named
                self.assertEqual("", _sample(unnamed), f"{name} variants only its wildcard arm renders")


if __name__ == "__main__":
    unittest.main()
