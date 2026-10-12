"""§FS-init.5.3 — what the binary embeds is what the tree shows: the `grund-init`
skill under `skills/` and every scaffold template under `templates/`
(§FS-init.2.1) are byte-identical to their copies under
`crates/grund-core/assets/`, in both directions, so neither can drift from the
other unnoticed."""

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ASSETS = REPO_ROOT / "crates" / "grund-core" / "assets"


class AssetSyncTests(unittest.TestCase):
    def test_grund_init_skill_matches_embedded_asset(self):
        repo_skill = REPO_ROOT / "skills" / "grund-init" / "SKILL.md"
        embedded_skill = ASSETS / "skills" / "grund-init" / "SKILL.md"
        self.assertEqual(
            repo_skill.read_bytes(),
            embedded_skill.read_bytes(),
            "skills/grund-init/SKILL.md must stay byte-identical to the embedded asset",
        )

    def test_skill_citation_directions_match_canonical_page(self):
        canonical = REPO_ROOT / "docs" / "user-facing" / "citation-directions.md"
        skill = (REPO_ROOT / "skills" / "grund-init" / "SKILL.md").read_bytes()
        begin = b"<!-- BEGIN citation-directions -->\n"
        end = b"<!-- END citation-directions -->"
        self.assertEqual(skill.count(begin), 1)
        self.assertEqual(skill.count(end), 1)
        start = skill.index(begin) + len(begin)
        finish = skill.index(end, start)
        self.assertEqual(
            skill[start:finish],
            canonical.read_bytes(),
            "the marked skill copy must stay byte-identical to the canonical citation-directions page",
        )

    def test_every_template_matches_its_embedded_asset_in_both_directions(self):
        # §DF-init-templates.2.5: one directory per template, so the two trees
        # are compared by relative path at every depth.
        repo_templates = REPO_ROOT / "templates"
        embedded_templates = ASSETS / "templates"

        def names(root):
            return {path.relative_to(root).as_posix() for path in root.rglob("*") if path.is_file()}

        repo_names = names(repo_templates)
        embedded_names = names(embedded_templates)
        self.assertEqual(repo_names, embedded_names, "templates/ and the embedded copies name different files")
        self.assertGreaterEqual(len(repo_names), 12)
        self.assertIn("software-development/grund.toml", repo_names)
        self.assertIn("none/grund.toml", repo_names)
        for name in sorted(repo_names):
            with self.subTest(template=name):
                self.assertEqual(
                    (repo_templates / name).read_bytes(),
                    (embedded_templates / name).read_bytes(),
                    f"templates/{name} must stay byte-identical to the embedded asset",
                )


if __name__ == "__main__":
    unittest.main()
